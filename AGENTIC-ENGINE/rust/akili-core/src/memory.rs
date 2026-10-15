//! KUMBUKUMBU — hifadhi ya kumbukumbu za muda mrefu za modeli (Rust, faili la JSON-lines).
//!
//! Kila kumbukumbu ina: id, timestamp, aina (`kind`), maandishi (`text`), vihusishi
//! (`tags`), umuhimu (`salience`) na idadi ya matumizi (`hits`). Kumbukumbu huchujwa
//! kwa ufunguo (keyword) na kurudishwa zikiwa zimepangwa kwa umuhimu + ubichi.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    pub id: u64,
    pub ts: u64,
    pub kind: String,
    pub text: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default = "one")]
    pub salience: f32,
    #[serde(default)]
    pub hits: u64,
}

fn one() -> f32 {
    1.0
}

pub struct MemoryStore {
    path: PathBuf,
    entries: Vec<Memory>,
    next_id: u64,
}

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

impl MemoryStore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let mut entries = Vec::new();
        if path.exists() {
            for line in std::fs::read_to_string(&path)?.lines() {
                if line.trim().is_empty() {
                    continue;
                }
                if let Ok(m) = serde_json::from_str::<Memory>(line) {
                    entries.push(m);
                }
            }
        }
        let next_id = entries.iter().map(|m| m.id).max().unwrap_or(0) + 1;
        Ok(Self {
            path,
            entries,
            next_id,
        })
    }

    fn append(&self, m: &Memory) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .with_context(|| format!("kumbukumbu: kuandika {:?} imeshindikana", self.path))?;
        writeln!(f, "{}", serde_json::to_string(m)?)?;
        Ok(())
    }

    /// Huongeza kumbukumbu mpya na huirudisha.
    pub fn remember(&mut self, kind: &str, text: &str, tags: Vec<String>, salience: f32) -> Result<Memory> {
        let m = Memory {
            id: self.next_id,
            ts: now(),
            kind: kind.to_string(),
            text: text.to_string(),
            tags,
            salience,
            hits: 0,
        };
        self.next_id += 1;
        self.append(&m)?;
        self.entries.push(m.clone());
        Ok(m)
    }

    pub fn count(&self) -> usize {
        self.entries.len()
    }

    pub fn all(&self) -> &[Memory] {
        &self.entries
    }

    /// Huchuja kwa maneno ya ufunguo (au hurudisha zote zilizo na umuhimu wa juu).
    pub fn recall(&mut self, query: &str, limit: usize) -> Vec<Memory> {
        let words: Vec<String> = query
            .to_lowercase()
            .split(|c: char| !c.is_alphanumeric())
            .filter(|w| w.len() >= 2)
            .map(|w| w.to_string())
            .collect();
        let mut scored: Vec<(f32, usize)> = self
            .entries
            .iter()
            .enumerate()
            .map(|(i, m)| {
                let hay = format!("{} {}", m.text.to_lowercase(), m.tags.join(" ").to_lowercase());
                let hits = words.iter().filter(|w| hay.contains(w.as_str())).count() as f32;
                let score = hits * 2.0 + m.salience + (m.ts as f32) * 1e-9;
                (score, i)
            })
            .collect();
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        let mut out = Vec::new();
        for (score, i) in scored.into_iter().take(limit.max(1)) {
            if score <= 0.0 && !words.is_empty() {
                continue;
            }
            self.entries[i].hits += 1;
            out.push(self.entries[i].clone());
        }
        out
    }

    /// Muhtasari mfupi wa kumbukumbu (kwa mfumo wa modeli).
    pub fn digest(&mut self, max_chars: usize) -> String {
        let mut recent: Vec<&Memory> = self.entries.iter().rev().take(12).collect();
        recent.reverse();
        let mut s = String::new();
        for m in recent {
            let line = format!("- [{}] {}: {}\n", m.kind, m.ts, m.text);
            if s.len() + line.len() > max_chars {
                break;
            }
            s.push_str(&line);
        }
        s
    }
}
