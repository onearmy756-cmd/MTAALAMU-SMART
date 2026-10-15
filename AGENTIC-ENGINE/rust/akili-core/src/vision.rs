//! Kichwa cha macho (ViT) cha Qwen2.5-VL: patch embed ya 3D, mzunguko wa 2D (h,w),
//! window attention + full attention, patch merger.

use crate::config::VisionConfig;
use crate::loader::WeightLoader;
use crate::text::{lin, rope_apply};
use anyhow::Result;
use candle_core::{DType, Device, Tensor, D};
use candle_nn::ops;
use crate::norm::RmsNormF;

pub struct Grid {
    pub t: usize,
    pub h: usize,
    pub w: usize,
}

pub struct VisionModel {
    patch_w: Tensor, // (hidden, in_chans, tps, ps, ps)
    blocks: Vec<BlockW>,
    merger_ln: RmsNormF,
    merger_fc1_w: Tensor,
    merger_fc1_b: Tensor,
    merger_fc2_w: Tensor,
    merger_fc2_b: Tensor,
    cfg: VisionConfig,
    dtype: DType,
    device: Device,
}

struct BlockW {
    norm1: RmsNormF,
    norm2: RmsNormF,
    qkv_w: Tensor,
    qkv_b: Tensor,
    proj_w: Tensor,
    proj_b: Tensor,
    gate_w: Tensor,
    gate_b: Tensor,
    up_w: Tensor,
    up_b: Tensor,
    down_w: Tensor,
    down_b: Tensor,
}

fn rms(t: Tensor, eps: f64) -> Result<RmsNormF> {
    RmsNormF::new(t, eps)
}

impl VisionModel {
    pub fn new(cfg: &VisionConfig, loader: &WeightLoader, dev: &Device, dtype: DType) -> Result<Self> {
        let patch_w = loader.get("visual.patch_embed.proj.weight", dtype)?;
        let mut blocks = Vec::with_capacity(cfg.depth);
        for i in 0..cfg.depth {
            let p = format!("visual.blocks.{i}");
            blocks.push(BlockW {
                norm1: rms(loader.get(&format!("{p}.norm1.weight"), dtype)?, 1e-6)?,
                norm2: rms(loader.get(&format!("{p}.norm2.weight"), dtype)?, 1e-6)?,
                qkv_w: loader.get(&format!("{p}.attn.qkv.weight"), dtype)?,
                qkv_b: loader.get(&format!("{p}.attn.qkv.bias"), dtype)?,
                proj_w: loader.get(&format!("{p}.attn.proj.weight"), dtype)?,
                proj_b: loader.get(&format!("{p}.attn.proj.bias"), dtype)?,
                gate_w: loader.get(&format!("{p}.mlp.gate_proj.weight"), dtype)?,
                gate_b: loader.get(&format!("{p}.mlp.gate_proj.bias"), dtype)?,
                up_w: loader.get(&format!("{p}.mlp.up_proj.weight"), dtype)?,
                up_b: loader.get(&format!("{p}.mlp.up_proj.bias"), dtype)?,
                down_w: loader.get(&format!("{p}.mlp.down_proj.weight"), dtype)?,
                down_b: loader.get(&format!("{p}.mlp.down_proj.bias"), dtype)?,
            });
        }
        let merger_ln = rms(loader.get("visual.merger.ln_q.weight", dtype)?, 1e-6)?;
        Ok(Self {
            patch_w,
            blocks,
            merger_ln,
            merger_fc1_w: loader.get("visual.merger.mlp.0.weight", dtype)?,
            merger_fc1_b: loader.get("visual.merger.mlp.0.bias", dtype)?,
            merger_fc2_w: loader.get("visual.merger.mlp.2.weight", dtype)?,
            merger_fc2_b: loader.get("visual.merger.mlp.2.bias", dtype)?,
            cfg: cfg.clone(),
            dtype,
            device: dev.clone(),
        })
    }

    /// `patches`: (n_patches, in_chans*tps*ps*ps) f32. `grid`: (t,h,w).
    /// Hurudisha (n_merged, out_hidden) f16 — moja kwa moja kuingia kwenye decoder.
    pub fn forward(&self, patches: &Tensor, grid: &Grid) -> Result<Tensor> {
        let sm = self.cfg.spatial_merge_size;
        let ps = self.cfg.patch_size;
        let n = patches.dim(0)?;
        let c = self.cfg.in_chans * self.cfg.temporal_patch_size;
        let x = patches
            .to_dtype(self.dtype)?
            .reshape((n, c, ps, ps))?
            .contiguous()?;
        let w = self.patch_w.reshape((self.cfg.hidden_size, c * ps * ps))?;
        let flat = x.reshape((n, c * ps * ps))?;
        let mut h = lin(&flat, &w, None)?; // (n, hidden)

        // Window index (permutation) na cu_window_seqlens
        let (window_index, cu_window) = window_index(grid, sm, self.cfg.window_size, ps)?;
        let (cu_full, _) = cu_seqlens_full(grid)?;
        let seq_len = n;

        // Position ids za 2D (h,w) zenye mpangilio wa blocks za merge
        let pos = vision_position_ids(grid, sm, &self.device)?; // (seq, 2)
        let (cos, sin) = axial_rope(&pos, self.cfg.hidden_size / self.cfg.num_heads, self.dtype, &self.device)?;

        // Permute kwa window attention
        h = permute_for_window(&h, &window_index, seq_len, sm)?;
        let cos = permute_for_window(&cos, &window_index, seq_len, sm)?;
        let sin = permute_for_window(&sin, &window_index, seq_len, sm)?;

        for (i, blk) in self.blocks.iter().enumerate() {
            let cu = if self.cfg.fullatt_block_indexes.contains(&i) {
                &cu_full
            } else {
                &cu_window
            };
            h = self.block(blk, &h, &cos, &sin, cu)?;
        }

        // Merger + kurudisha mpangilio wa asili
        let merged = self.merger(&h)?;
        let reverse = argsort(&window_index)?;
        Ok(merged.index_select(&reverse, 0)?)
    }

    fn block(&self, b: &BlockW, x: &Tensor, cos: &Tensor, sin: &Tensor, cu: &[usize]) -> Result<Tensor> {
        let residual = x;
        let h = b.norm1.forward(&x)?;
        let n = h.dim(0)?;
        let nh = self.cfg.num_heads;
        let hd = self.cfg.hidden_size / nh;
        let qkv = lin(&h, &b.qkv_w, Some(&b.qkv_b))?;
        let qkv = qkv.reshape((n, 3, nh, hd))?;
        let q = qkv.narrow(1, 0, 1)?.squeeze(1)?.unsqueeze(0)?; // (1,n,nh,hd)
        let k = qkv.narrow(1, 1, 1)?.squeeze(1)?.unsqueeze(0)?;
        let v = qkv.narrow(1, 2, 1)?.squeeze(1)?.unsqueeze(0)?;
        let q = q.transpose(1, 2)?; // (1,nh,n,hd)
        let k = k.transpose(1, 2)?;
        let v = v.transpose(1, 2)?;
        let cos = cos.unsqueeze(0)?.unsqueeze(0)?;
        let sin = sin.unsqueeze(0)?.unsqueeze(0)?;
        let q = rope_apply(&q, &cos, &sin)?;
        let k = rope_apply(&k, &cos, &sin)?;
        let attn = packed_attention(&q, &k, &v, cu, self.dtype)?;
        let attn = attn.transpose(1, 2)?.reshape((n, self.cfg.hidden_size))?;
        let attn = lin(&attn, &b.proj_w, Some(&b.proj_b))?;
        let x = (residual + attn)?;

        let residual = x.clone();
        let h = b.norm2.forward(&x)?;
        let g = lin(&h, &b.gate_w, Some(&b.gate_b))?.silu()?;
        let u = lin(&h, &b.up_w, Some(&b.up_b))?;
        let m = lin(&(g * u)?, &b.down_w, Some(&b.down_b))?;
        Ok((residual + m)?)
    }

    fn merger(&self, x: &Tensor) -> Result<Tensor> {
        let sm = self.cfg.spatial_merge_size;
        let cdim = self.cfg.hidden_size;
        let hsize = cdim * sm * sm;
        let n = x.dim(0)?;
        let y = self.merger_ln.forward(&x)?;
        let y = y.reshape((n / (sm * sm), hsize))?;
        let y = lin(&y, &self.merger_fc1_w, Some(&self.merger_fc1_b))?.gelu_erf()?;
        lin(&y, &self.merger_fc2_w, Some(&self.merger_fc2_b))
    }
}

/// `x` (seq, d) -> (seq, d) kwa mpangilio wa windows.
fn permute_for_window(x: &Tensor, window_index: &[usize], seq_len: usize, sm: usize) -> Result<Tensor> {
    let unit = sm * sm;
    let d = x.dim(1)?;
    let x = x.reshape((seq_len / unit, unit, d))?;
    let idx = Tensor::from_vec(
        window_index.iter().map(|v| *v as u32).collect::<Vec<u32>>(),
        (window_index.len(),),
        x.device(),
    )?;
    let x = x.index_select(&idx, 0)?;
    Ok(x.reshape((seq_len, d))?)
}

/// Window index + cu_window_seqlens (kama transformers).
pub fn window_index(grid: &Grid, sm: usize, window_size: usize, patch: usize) -> Result<(Vec<usize>, Vec<usize>)> {
    let vit_win = window_size / sm / patch;
    let unit = sm * sm;
    let (t, hh, ww) = (grid.t, grid.h / sm, grid.w / sm);
    let mut index: Vec<i64> = Vec::new();
    let mut cu: Vec<usize> = vec![0];
    let base = 0usize;
    let idx: Vec<i64> = (0..(t * hh * ww) as i64).collect();
    let pad_h = vit_win - hh % vit_win;
    let pad_w = vit_win - ww % vit_win;
    let nh_win = (hh + pad_h) / vit_win;
    let nw_win = (ww + pad_w) / vit_win;
    let ph = hh + pad_h;
    let pw = ww + pad_w;
    // padded (t, ph, pw)
    let mut padded = vec![-100i64; t * ph * pw];
    for ti in 0..t {
        for hi in 0..hh {
            for wi in 0..ww {
                padded[ti * ph * pw + hi * pw + wi] = idx[ti * hh * ww + hi * ww + wi];
            }
        }
    }
    // permute (t, nh_win, nw_win, vit_win, vit_win)
    for ti in 0..t {
        for wh in 0..nh_win {
            for wwi in 0..nw_win {
                let mut win: Vec<i64> = Vec::with_capacity(vit_win * vit_win);
                for r in 0..vit_win {
                    for c2 in 0..vit_win {
                        let y = wh * vit_win + r;
                        let x = wwi * vit_win + c2;
                        win.push(padded[ti * ph * pw + y * pw + x]);
                    }
                }
                let count = win.iter().filter(|v| **v != -100).count();
                for v in win.iter().filter(|v| **v != -100) {
                    index.push(*v + base as i64);
                }
                cu.push(cu.last().unwrap() + count * unit);
            }
        }
    }
    // unique_consecutive
    let mut uniq: Vec<usize> = Vec::new();
    for v in cu {
        if uniq.last() != Some(&v) {
            uniq.push(v);
        }
    }
    Ok((index.into_iter().map(|v| v as usize).collect(), uniq))
}

/// cu_seqlens za full attention: kila picha ni sequence moja (t*h*w patches).
pub fn cu_seqlens_full(grid: &Grid) -> Result<(Vec<usize>, usize)> {
    let n = grid.t * grid.h * grid.w;
    Ok((vec![0, n], n))
}

/// Position ids (h,w) zenye mpangilio wa merge blocks: (seq, 2).
pub fn vision_position_ids(grid: &Grid, sm: usize, dev: &Device) -> Result<Tensor> {
    let (h, w) = (grid.h, grid.w);
    let bh = h / sm;
    let bw = w / sm;
    let mut hpos = Vec::with_capacity(h * w);
    let mut wpos = Vec::with_capacity(h * w);
    // meshgrid(h,w) -> reshape(bh, sm, bw, sm) -> transpose(1,2) -> flatten
    for i in 0..bh {
        for j in 0..bw {
            for k in 0..sm {
                for l in 0..sm {
                    hpos.push((i * sm + k) as f32);
                    wpos.push((j * sm + l) as f32);
                }
            }
        }
    }
    // repeat kwa t
    let mut hp = Vec::with_capacity(h * w * grid.t);
    let mut wp = Vec::with_capacity(h * w * grid.t);
    for _ in 0..grid.t {
        hp.extend_from_slice(&hpos);
        wp.extend_from_slice(&wpos);
    }
    let ht = Tensor::from_vec(hp, (h * w * grid.t, 1), dev)?;
    let wt = Tensor::from_vec(wp, (h * w * grid.t, 1), dev)?;
    Ok(Tensor::cat(&[&ht, &wt], 1)?)
}

/// Rope ya axial: cos/sin za (seq, head_dim) kutoka position (seq, 2).
pub fn axial_rope(pos: &Tensor, head_dim: usize, dtype: DType, dev: &Device) -> Result<(Tensor, Tensor)> {
    let spatial = head_dim / 2;
    let inv: Vec<f32> = (0..spatial)
        .step_by(2)
        .map(|i| 1f32 / (10000f32.powf(i as f32 / spatial as f32)))
        .collect();
    let n = inv.len();
    let inv = Tensor::from_vec(inv, (1, n), dev)?;
    let f = pos.to_dtype(DType::F32)?.unsqueeze(2)?.broadcast_mul(&inv)?; // (seq,2,n)
    let fh = f.narrow(1, 0, 1)?.squeeze(1)?;
    let fw = f.narrow(1, 1, 1)?.squeeze(1)?;
    let hw = Tensor::cat(&[&fh, &fw], 1)?; // (seq, 2n)
    let cos = Tensor::cat(&[&hw, &hw], 1)?.cos()?.to_dtype(dtype)?;
    let sin = Tensor::cat(&[&hw, &hw], 1)?.sin()?.to_dtype(dtype)?;
    Ok((cos, sin))
}

fn argsort(v: &[usize]) -> Result<Tensor> {
    let mut idx: Vec<usize> = (0..v.len()).collect();
    idx.sort_by_key(|i| v[*i]);
    Ok(Tensor::from_vec(
        idx.into_iter().map(|i| i as u32).collect::<Vec<u32>>(),
        (v.len(),),
        &Device::Cpu,
    )?)
}

/// Packed attention (flash-style bila mask) kwa chunks zilizoainishwa na `cu`.
fn packed_attention(q: &Tensor, k: &Tensor, v: &Tensor, cu: &[usize], dtype: DType) -> Result<Tensor> {
    let (b, nh, n, hd) = q.dims4()?;
    let mut outs: Vec<Tensor> = Vec::with_capacity(cu.len().saturating_sub(1));
    for w in cu.windows(2) {
        let (s, e) = (w[0], w[1]);
        let len = e - s;
        if len == 0 {
            continue;
        }
        let qc = q.narrow(2, s, len)?.contiguous()?;
        let kc = k.narrow(2, s, len)?.contiguous()?;
        let vc = v.narrow(2, s, len)?.contiguous()?;
        let scale = 1.0 / (hd as f64).sqrt();
        let attn = (qc.matmul(&kc.transpose(2, 3)?.contiguous()?)? * scale)?;
        let attn = ops::softmax(&attn, D::Minus1)?;
        outs.push(attn.matmul(&vc)?);
    }
    anyhow::ensure!(!outs.is_empty(), "packed attention: hakuna chunks");
    let out = if outs.len() == 1 {
        outs.into_iter().next().unwrap()
    } else {
        Tensor::cat(&outs, 2)?
    };
    let _ = (b, nh, n, dtype);
    Ok(out)
}
