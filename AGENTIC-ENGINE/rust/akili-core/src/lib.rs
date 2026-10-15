//! MBILINYI AKILI — injini ya modeli ya Qwen2.5-VL (BF16) kwa Rust.
//!
//! Inafanya kazi YOTE ya modeli: upakiaji wa BF16 safetensors (mmap), tokenization,
//! M-RoPE, decoder ya maandishi, kichwa cha macho (ViT), na uzalishaji wa tokeni.

pub mod config;
pub mod image;
pub mod loader;
pub mod memory;
pub mod norm;
pub mod rope;
pub mod sampler;
pub mod text;
pub mod tokenizer;
pub mod vision;

use anyhow::{Context, Result};
use candle_core::{DType, Device, Tensor};
use config::Config;
use loader::WeightLoader;
use memory::MemoryStore;
use std::path::{Path, PathBuf};
use text::TextModel;
use tokenizer::{ChatTokenizer, Turn};
use vision::{Grid, VisionModel};

pub struct EngineOptions {
    pub dtype: DType,
    pub with_vision: bool,
    pub memory_path: PathBuf,
    pub thread_hint: usize,
}

impl Default for EngineOptions {
    fn default() -> Self {
        Self {
            dtype: DType::F16,
            with_vision: true,
            memory_path: PathBuf::from("data/kumbukumbu.jsonl"),
            thread_hint: 0,
        }
    }
}

pub struct GenParams {
    pub max_tokens: usize,
    pub temperature: f32,
    pub top_p: f32,
    pub repetition_penalty: f32,
    pub seed: u64,
}

impl Default for GenParams {
    fn default() -> Self {
        Self {
            max_tokens: 256,
            temperature: 0.2,
            top_p: 0.9,
            repetition_penalty: 1.05,
            seed: 0x5eed_1234_abcd_0001,
        }
    }
}

pub struct GenOutput {
    pub text: String,
    pub prompt_tokens: usize,
    pub generated_tokens: usize,
    pub stopped: bool,
    pub ms: u128,
}

pub struct Engine {
    pub cfg: Config,
    pub dir: PathBuf,
    pub tok: ChatTokenizer,
    text: TextModel,
    vision: Option<VisionModel>,
    pub memory: MemoryStore,
    dev: Device,
    pub dtype: DType,
}

impl Engine {
    pub fn load(dir: impl AsRef<Path>, opts: EngineOptions) -> Result<Self> {
        let dir = dir.as_ref().to_path_buf();
        let cfg = Config::from_dir(&dir)?;
        let dev = Device::Cpu;
        let loader = WeightLoader::from_dir(&dir)?;
        let tw = text::load_weights(&loader, &cfg, opts.dtype)?;
        let text = TextModel::new(&cfg, tw, &dev, opts.dtype)?;
        let vision = if opts.with_vision {
            Some(VisionModel::new(&cfg.vision, &loader, &dev, opts.dtype)?)
        } else {
            None
        };
        let tok = ChatTokenizer::from_dir(
            &dir,
            cfg.image_token_id,
            cfg.vision_start_token_id,
            cfg.vision_end_token_id,
        )?;
        let memory = MemoryStore::open(opts.memory_path)?;
        Ok(Self {
            cfg,
            dir,
            tok,
            text,
            vision,
            memory,
            dev,
            dtype: opts.dtype,
        })
    }

    /// Hupakia kichwa cha macho pekee (bila decoder ya maandishi) — kwa
    /// uthibitisho wa haraka wa ViT kwa F32 bila kutumia RAM ya modeli yote.
    pub fn load_vision(dir: impl AsRef<Path>, dtype: DType) -> Result<VisionModel> {
        let dir = dir.as_ref();
        let cfg = Config::from_dir(dir)?;
        let loader = WeightLoader::from_dir(dir)?;
        VisionModel::new(&cfg.vision, &loader, &Device::Cpu, dtype)
    }

    /// Huzalisha jibu kutoka kwa mazungumzo na picha (hiari).
    pub fn generate(
        &mut self,
        system: &str,
        turns: &[Turn],
        images: &[image::PlanarImage],
        params: &GenParams,
    ) -> Result<GenOutput> {
        let start = std::time::Instant::now();
        self.text.reset_cache();

        let prompt = self.tok.render(system, turns, true);
        let raw_ids = self.tok.encode(&prompt)?;

        // Picha: preprocess + hesabu tokeni kwa kila picha
        let mut grids: Vec<Grid> = Vec::new();
        let mut vision_feats: Vec<Tensor> = Vec::new();
        let mut counts: Vec<usize> = Vec::new();
        if !images.is_empty() {
            let v = self
                .vision
                .as_ref()
                .context("vision imezimwa kwenye EngineOptions")?;
            let sm = self.cfg.vision.spatial_merge_size;
            let ps = self.cfg.vision.patch_size;
            let tps = self.cfg.vision.temporal_patch_size;
            for img in images {
                let p = image::preprocess(
                    img,
                    ps,
                    tps,
                    sm,
                    self.cfg.min_pixels,
                    self.cfg.max_pixels,
                )?;
                let grid = Grid {
                    t: p.grid_t,
                    h: p.grid_h,
                    w: p.grid_w,
                };
                let t = image::to_tensor(&p, &self.dev)?;
                let feats = v.forward(&t, &grid)?;
                counts.push(feats.dim(0)?);
                vision_feats.push(feats);
                grids.push(grid);
            }
        }
        let ids = self.tok.expand_image_tokens(&raw_ids, &counts);
        let seq = ids.len();
        anyhow::ensure!(seq > 0, "prompt ni tupu");

        // Embeddings + kuingiza vipengele vya macho
        let id_tensor = Tensor::from_vec(ids.clone(), (1, seq), &self.dev)?;
        let mut embeds = self.text.embed_tokens(&id_tensor)?;
        if !vision_feats.is_empty() {
            let cat = Tensor::cat(&vision_feats, 0)?.to_dtype(self.dtype)?;
            embeds = self.scatter_vision(&embeds, &ids, &cat)?;
        }

        // M-RoPE positions
        let positions = self.compute_positions(&ids, &grids)?;

        // Prefill
        let hidden = self.text.forward(&embeds, &positions)?;
        let mut logits = self.text.logits_last(&hidden)?;
        let mut out_ids: Vec<u32> = Vec::new();
        let mut stopped = false;

        for step in 0..params.max_tokens {
            let next = sampler::sample(&mut logits, params, &out_ids);
            if self.cfg.is_eos(next) {
                stopped = true;
                break;
            }
            out_ids.push(next);
            if step + 1 == params.max_tokens {
                break;
            }
            // Decode hatua moja
            let pos = self.decode_positions(seq + out_ids.len() - 1)?;
            let tok_t = Tensor::from_vec(vec![next], (1, 1), &self.dev)?;
            let e = self.text.embed_tokens(&tok_t)?;
            let h = self.text.forward(&e, &pos)?;
            logits = self.text.logits_last(&h)?;
        }

        let text = self.tok.decode(&out_ids)?;
        Ok(GenOutput {
            text,
            prompt_tokens: seq,
            generated_tokens: out_ids.len(),
            stopped,
            ms: start.elapsed().as_millis(),
        })
    }

    /// Hupanga vipengele vya macho kwenye nafasi za `<|image_pad|>`.
    fn scatter_vision(&self, embeds: &Tensor, ids: &[u32], feats: &Tensor) -> Result<Tensor> {
        let (_, seq, hidden) = embeds.dims3()?;
        let flat = embeds.reshape((seq, hidden))?.contiguous()?;
        let img_pos: Vec<u32> = ids
            .iter()
            .enumerate()
            .filter(|(_, id)| **id == self.cfg.image_token_id)
            .map(|(i, _)| i as u32)
            .collect();
        let n_img = feats.dim(0)?;
        anyhow::ensure!(
            img_pos.len() == n_img,
            "tokeni za picha ({}) hazilingani na vipengele vya macho ({n_img})",
            img_pos.len(),
        );
        let mut idx_data = Vec::with_capacity(n_img * hidden);
        for p in &img_pos {
            idx_data.extend(std::iter::repeat(*p).take(hidden));
        }
        let idx = Tensor::from_vec(idx_data, (n_img, hidden), &self.dev)?;
        flat.scatter_set(&idx, &feats.to_dtype(self.dtype)?, 0)?;
        Ok(flat.reshape((1, seq, hidden))?)
    }

    /// Hukokotoa position ids (3, seq) kwa mujibu wa `get_rope_index`.
    fn compute_positions(&self, ids: &[u32], grids: &[Grid]) -> Result<Tensor> {
        let sm = self.cfg.vision.spatial_merge_size;
        let seq = ids.len();
        let mut pos = vec![vec![0f32; seq]; 3];
        let mut current = 0usize;
        let mut g = 0usize;
        let mut i = 0usize;
        while i < seq {
            if ids[i] == self.cfg.image_token_id {
                let mut j = i;
                while j < seq && ids[j] == self.cfg.image_token_id {
                    j += 1;
                }
                anyhow::ensure!(g < grids.len(), "idadi ya picha hailingani na grids");
                let grid = &grids[g];
                let (lt, lh, lw) = (grid.t, grid.h / sm, grid.w / sm);
                let mut k = 0usize;
                for ti in 0..lt {
                    for hi in 0..lh {
                        for wi in 0..lw {
                            if i + k >= j {
                                break;
                            }
                            pos[0][i + k] = (ti + current) as f32;
                            pos[1][i + k] = (hi + current) as f32;
                            pos[2][i + k] = (wi + current) as f32;
                            k += 1;
                        }
                    }
                }
                current += lh.max(lw);
                g += 1;
                i = j;
            } else {
                let mut j = i;
                while j < seq && ids[j] != self.cfg.image_token_id {
                    j += 1;
                }
                for (k, item) in pos.iter_mut().enumerate() {
                    let _ = k;
                    for (t, slot) in item.iter_mut().enumerate().take(j).skip(i) {
                        *slot = (t - i + current) as f32;
                    }
                }
                current += j - i;
                i = j;
            }
        }
        let flat: Vec<f32> = pos.into_iter().flatten().collect();
        Ok(Tensor::from_vec(flat, (3, seq), &self.dev)?)
    }

    /// Position za tokeni mpya wakati wa decode (3, 1).
    fn decode_positions(&self, total: usize) -> Result<Tensor> {
        let v = vec![total as f32];
        let t = Tensor::from_vec(v, (1, 1), &self.dev)?;
        Ok(Tensor::cat(&[&t, &t, &t], 0)?)
    }

    pub fn device(&self) -> &Device {
        &self.dev
    }

    pub fn vision_enabled(&self) -> bool {
        self.vision.is_some()
    }

    /// Forward ya kichwa cha macho pekee (kwa uthibitisho).
    pub fn vision_forward(&self, patches: &Tensor, grid: &Grid) -> Result<Tensor> {
        let v = self
            .vision
            .as_ref()
            .context("vision imezimwa kwenye EngineOptions")?;
        v.forward(patches, grid)
    }

    /// Forward ya maandishi kutoka ids (hiari: vipengele vya macho vilivyotayarishwa).
    /// Hurudisha (hidden_states (1,seq,hidden), logits za nafasi ya mwisho).
    pub fn forward_ids(
        &mut self,
        ids: &[u32],
        grids: &[Grid],
        feats: Option<&Tensor>,
    ) -> Result<(Tensor, Vec<f32>)> {
        self.text.reset_cache();
        let seq = ids.len();
        let id_tensor = Tensor::from_vec(ids.to_vec(), (1, seq), &self.dev)?;
        let mut embeds = self.text.embed_tokens(&id_tensor)?;
        if let Some(f) = feats {
            embeds = self.scatter_vision(&embeds, ids, f)?;
        }
        let positions = self.compute_positions(ids, grids)?;
        let hidden = self.text.forward(&embeds, &positions)?;
        let logits = self.text.logits_last(&hidden)?;
        Ok((hidden, logits))
    }
}
