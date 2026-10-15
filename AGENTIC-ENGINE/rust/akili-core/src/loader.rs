//! Upakiaji wa safetensors kwa memory-map (mmap) — bf16 hukaa kwenye mmap,
//! hukonvertiwa kwa dtype lengwa wakati wa `get`.

use anyhow::{Context, Result};
use candle_core::{safetensors::MmapedSafetensors, DType, Device, Tensor};
use std::path::{Path, PathBuf};

pub struct WeightLoader {
    inner: MmapedSafetensors,
}

impl WeightLoader {
    /// Hupakia shards zote kwenye `dir` kulingana na `model.safetensors.index.json`.
    pub fn from_dir(dir: &Path) -> Result<Self> {
        let mut shards: Vec<PathBuf> = std::fs::read_dir(dir)?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.starts_with("model-") && n.ends_with(".safetensors"))
                    .unwrap_or(false)
            })
            .collect();
        shards.sort();
        anyhow::ensure!(!shards.is_empty(), "hakuna shards za safetensors kwenye {dir:?}");
        // SAFETY: faili hazibadiliki wakati wa utekelezaji.
        let inner = unsafe { MmapedSafetensors::multi(&shards) }
            .with_context(|| format!("mmap ya safetensors imeshindikana ({shards:?})"))?;
        Ok(Self { inner })
    }

    pub fn has(&self, name: &str) -> bool {
        self.inner.get(name).is_ok()
    }

    /// Huchukua tensor kwa jina, kisha huibadilisha kwa `dtype`.
    pub fn get(&self, name: &str, dtype: DType) -> Result<Tensor> {
        let t = self
            .inner
            .load(name, &Device::Cpu)
            .with_context(|| format!("weight `{name}` haipatikani"))?;
        if t.dtype() == dtype {
            Ok(t)
        } else {
            Ok(t.to_dtype(dtype)?)
        }
    }

    /// Huchukua tensor kwa dtype yake asili (bf16).
    pub fn get_raw(&self, name: &str) -> Result<Tensor> {
        self.inner
            .load(name, &Device::Cpu)
            .with_context(|| format!("weight `{name}` haipatikani"))
    }

    pub fn names(&self) -> Vec<String> {
        self.inner.tensors().into_iter().map(|(k, _)| k).collect()
    }
}
