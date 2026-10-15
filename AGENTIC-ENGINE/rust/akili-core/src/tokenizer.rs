//! Tokenizer ya Qwen2.5 + kiolezo cha mazungumzo (chat template) + uwekaji wa tokeni za picha.

use anyhow::{Context, Result};
use std::path::Path;
use tokenizers::Tokenizer;

#[derive(Clone, Debug)]
pub struct Turn {
    pub role: String,
    pub content: String,
    /// Idadi ya picha zilizoambatishwa kwenye turn hii (zinaingizwa mwanzoni).
    pub images: usize,
}

impl Turn {
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".into(),
            content: content.into(),
            images: 0,
        }
    }
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: "assistant".into(),
            content: content.into(),
            images: 0,
        }
    }
    pub fn with_images(mut self, n: usize) -> Self {
        self.images = n;
        self
    }
}

pub struct ChatTokenizer {
    tok: Tokenizer,
    pub image_token_id: u32,
    pub vision_start_id: u32,
    pub vision_end_id: u32,
    pub eos_id: u32,
    pub im_end_id: u32,
}

impl ChatTokenizer {
    pub fn from_dir(dir: &Path, image_token_id: u32, vision_start_id: u32, vision_end_id: u32) -> Result<Self> {
        let tok = Tokenizer::from_file(dir.join("tokenizer.json"))
            .map_err(|e| anyhow::anyhow!("tokenizer.json: {e}"))?;
        let get = |s: &str| -> Result<u32> {
            tok.token_to_id(s)
                .with_context(|| format!("tokeni maalum `{s}` haipatikani"))
        };
        let eos_id = get("<|endoftext|>")?;
        let im_end_id = get("<|im_end|>")?;
        Ok(Self {
            tok,
            image_token_id,
            vision_start_id,
            vision_end_id,
            eos_id,
            im_end_id,
        })
    }

    /// Huunda maandishi ya mazungumzo kwa muundo wa Qwen2.5 (ChatML).
    pub fn render(&self, system: &str, turns: &[Turn], add_generation_prompt: bool) -> String {
        let mut s = String::new();
        if !system.is_empty() {
            s.push_str(&format!("<|im_start|>system\n{system}<|im_end|>\n"));
        }
        for t in turns {
            s.push_str(&format!("<|im_start|>{}\n", t.role));
            for _ in 0..t.images {
                s.push_str("<|vision_start|><|image_pad|><|vision_end|>");
            }
            s.push_str(&t.content);
            s.push_str("<|im_end|>\n");
        }
        if add_generation_prompt {
            s.push_str("<|im_start|>assistant\n");
        }
        s
    }

    pub fn encode(&self, text: &str) -> Result<Vec<u32>> {
        let enc = self
            .tok
            .encode(text, false)
            .map_err(|e| anyhow::anyhow!("encode: {e}"))?;
        Ok(enc.get_ids().to_vec())
    }

    pub fn decode(&self, ids: &[u32]) -> Result<String> {
        self.tok
            .decode(ids, true)
            .map_err(|e| anyhow::anyhow!("decode: {e}"))
    }

    /// Hubadilisha kila `<|image_pad|>` moja kuwa `counts[i]` tokeni.
    pub fn expand_image_tokens(&self, ids: &[u32], counts: &[usize]) -> Vec<u32> {
        let mut out = Vec::with_capacity(ids.len());
        let mut img = 0usize;
        for &id in ids {
            if id == self.image_token_id && img < counts.len() {
                for _ in 0..counts[img] {
                    out.push(id);
                }
                img += 1;
            } else {
                out.push(id);
            }
        }
        out
    }
}
