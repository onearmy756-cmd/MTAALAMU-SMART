//! Uchambuzi wa `config.json` ya Qwen2.5-VL (text + vision).

use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub struct VisionConfig {
    pub depth: usize,
    pub hidden_size: usize,
    pub intermediate_size: usize,
    pub num_heads: usize,
    pub in_chans: usize,
    pub out_hidden_size: usize,
    pub patch_size: usize,
    pub spatial_merge_size: usize,
    pub temporal_patch_size: usize,
    pub window_size: usize,
    pub fullatt_block_indexes: Vec<usize>,
    #[serde(default)]
    pub hidden_act: String,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub vocab_size: usize,
    pub hidden_size: usize,
    pub intermediate_size: usize,
    pub num_hidden_layers: usize,
    pub num_attention_heads: usize,
    pub num_key_value_heads: usize,
    pub head_dim: usize,
    pub max_position_embeddings: usize,
    pub rms_norm_eps: f64,
    pub rope_theta: f64,
    pub mrope_section: [usize; 3],
    pub tie_word_embeddings: bool,
    pub image_token_id: u32,
    pub video_token_id: u32,
    pub vision_start_token_id: u32,
    pub vision_end_token_id: u32,
    pub eos_token_ids: Vec<u32>,
    pub vision: VisionConfig,
    pub min_pixels: u64,
    pub max_pixels: u64,
}

#[derive(Deserialize)]
struct RawText {
    #[serde(default)]
    text_config: Option<RawTextInner>,
    #[serde(flatten)]
    flat: RawTextInner,
    vision_config: VisionConfig,
    #[serde(default)]
    image_token_id: Option<u32>,
    #[serde(default)]
    video_token_id: Option<u32>,
    #[serde(default)]
    vision_start_token_id: Option<u32>,
    #[serde(default)]
    vision_end_token_id: Option<u32>,
}

#[derive(Deserialize, Default)]
struct RawTextInner {
    vocab_size: Option<usize>,
    hidden_size: Option<usize>,
    intermediate_size: Option<usize>,
    num_hidden_layers: Option<usize>,
    num_attention_heads: Option<usize>,
    num_key_value_heads: Option<usize>,
    head_dim: Option<usize>,
    max_position_embeddings: Option<usize>,
    rms_norm_eps: Option<f64>,
    rope_theta: Option<f64>,
    rope_scaling: Option<RopeScaling>,
    tie_word_embeddings: Option<bool>,
}

#[derive(Deserialize)]
struct RopeScaling {
    #[serde(default)]
    mrope_section: Option<Vec<usize>>,
}

#[derive(Deserialize)]
struct RawGen {
    #[serde(default)]
    eos_token_id: Option<serde_json::Value>,
}

#[derive(Deserialize)]
struct RawPre {
    #[serde(default)]
    min_pixels: Option<u64>,
    #[serde(default)]
    max_pixels: Option<u64>,
}

impl Config {
    pub fn from_dir(dir: &Path) -> Result<Self> {
        let raw: RawText = serde_json::from_str(
            &std::fs::read_to_string(dir.join("config.json")).context("config.json haipatikani")?,
        )
        .context("config.json si sahihi")?;
        let t = raw.text_config.unwrap_or(raw.flat);
        let req = |v: Option<usize>, k: &str| -> Result<usize> {
            v.with_context(|| format!("config.json haina `{k}`"))
        };
        let hidden_size = req(t.hidden_size, "hidden_size")?;
        let num_attention_heads = req(t.num_attention_heads, "num_attention_heads")?;
        let head_dim = t.head_dim.unwrap_or(hidden_size / num_attention_heads);
        let mrope_section = t
            .rope_scaling
            .as_ref()
            .and_then(|r| r.mrope_section.clone())
            .filter(|s| s.len() == 3)
            .map(|s| [s[0], s[1], s[2]])
            .unwrap_or([16, 24, 24]);

        let (min_pixels, max_pixels) = std::fs::read_to_string(dir.join("preprocessor_config.json"))
            .ok()
            .and_then(|s| serde_json::from_str::<RawPre>(&s).ok())
            .map(|p| (p.min_pixels.unwrap_or(3136), p.max_pixels.unwrap_or(12845056)))
            .unwrap_or((3136, 12845056));

        let eos_token_ids = std::fs::read_to_string(dir.join("generation_config.json"))
            .ok()
            .and_then(|s| serde_json::from_str::<RawGen>(&s).ok())
            .and_then(|g| g.eos_token_id)
            .map(|v| match v {
                serde_json::Value::Number(n) => vec![n.as_u64().unwrap_or(151645) as u32],
                serde_json::Value::Array(a) => {
                    a.iter().filter_map(|x| x.as_u64().map(|n| n as u32)).collect()
                }
                _ => vec![151645],
            })
            .unwrap_or_else(|| vec![151645, 151643]);

        Ok(Self {
            vocab_size: req(t.vocab_size, "vocab_size")?,
            hidden_size,
            intermediate_size: req(t.intermediate_size, "intermediate_size")?,
            num_hidden_layers: req(t.num_hidden_layers, "num_hidden_layers")?,
            num_attention_heads,
            num_key_value_heads: req(t.num_key_value_heads, "num_key_value_heads")?,
            head_dim,
            max_position_embeddings: t.max_position_embeddings.unwrap_or(32768),
            rms_norm_eps: t.rms_norm_eps.unwrap_or(1e-6),
            rope_theta: t.rope_theta.unwrap_or(1_000_000.0),
            mrope_section,
            tie_word_embeddings: t.tie_word_embeddings.unwrap_or(false),
            image_token_id: raw.image_token_id.unwrap_or(151655),
            video_token_id: raw.video_token_id.unwrap_or(151656),
            vision_start_token_id: raw.vision_start_token_id.unwrap_or(151652),
            vision_end_token_id: raw.vision_end_token_id.unwrap_or(151653),
            eos_token_ids,
            vision: raw.vision_config,
            min_pixels,
            max_pixels,
        })
    }

    /// Idadi ya float kwa kila kipengele cha safetensors (bf16 = 2 bytes).
    pub fn is_eos(&self, id: u32) -> bool {
        self.eos_token_ids.contains(&id)
    }
}
