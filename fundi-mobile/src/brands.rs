//! BRANDS — data layer ya data/mobile/brands.json (smartphones + button phones).

use anyhow::{bail, Result};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct Smartphone {
    pub recovery_combo_sw: String,
    pub download_mode: String,
    pub flash_tool: String,
    pub firmware_site: String,
    pub drivers: String,
}

#[derive(Debug, Deserialize)]
pub struct ButtonPhone {
    #[serde(default)]
    pub master_codes: Vec<String>,
    pub hard_reset_sw: String,
    pub flash_tool: String,
    pub firmware_site: String,
}

#[derive(Debug, Deserialize)]
pub struct Brands {
    #[serde(default)]
    pub smartphones: std::collections::BTreeMap<String, Smartphone>,
    #[serde(default)]
    pub button_phones: std::collections::BTreeMap<String, ButtonPhone>,
}

pub fn load() -> Result<Brands> {
    let base = std::env::var("FUNDI_DATA").unwrap_or_else(|_| "../data".into());
    let path = PathBuf::from(base).join("mobile/brands.json");
    let txt = std::fs::read_to_string(&path)
        .map_err(|e| anyhow::anyhow!("soma {}: {e}", path.display()))?;
    Ok(serde_json::from_str(&txt)?)
}

pub fn recovery_combo(brand: &str) -> String {
    let b = brand.to_lowercase();
    load()
        .ok()
        .and_then(|x| x.smartphones.get(&b).map(|s| s.recovery_combo_sw.clone()))
        .unwrap_or_else(|| "Volume Up + Power (generic)".into())
}

pub fn download_mode(brand: &str) -> String {
    let b = brand.to_lowercase();
    load()
        .ok()
        .and_then(|x| x.smartphones.get(&b).map(|s| s.download_mode.clone()))
        .unwrap_or_else(|| "Volume Down + Power (generic)".into())
}

/// (tool, firmware_site, drivers)
pub fn flash_info(brand: &str) -> (String, String, String) {
    let b = brand.to_lowercase();
    if let Ok(x) = load() {
        if let Some(s) = x.smartphones.get(&b) {
            return (s.flash_tool.clone(), s.firmware_site.clone(), s.drivers.clone());
        }
        // kama ni button phone → flash tools zake
        if let Some(s) = x.button_phones.get(&b) {
            return (s.flash_tool.clone(), s.firmware_site.clone(), "MTK/SPD USB".into());
        }
    }
    ("SP Flash Tool".into(), "firmwarefile.com".into(), "MTK/SPD USB".into())
}

pub struct ButtonInfo {
    pub master_codes: Vec<String>,
    pub hard_reset_sw: String,
    pub flash_tool: String,
}

pub fn button_phone(brand: &str) -> ButtonInfo {
    let b = brand.to_lowercase();
    if let Ok(x) = load() {
        let direct = x.button_phones.get(&b).map(|s| ButtonInfo {
            master_codes: s.master_codes.clone(),
            hard_reset_sw: s.hard_reset_sw.clone(),
            flash_tool: s.flash_tool.clone(),
        });
        if let Some(s) = direct {
            return ButtonInfo {
                master_codes: s.master_codes,
                hard_reset_sw: s.hard_reset_sw,
                flash_tool: s.flash_tool,
            };
        }
        // fuzzy: nokia → nokia, "techno button" → techno_button, n.k.
        for (k, v) in &x.button_phones {
            if k.contains(&b) || b.contains(k) {
                return ButtonInfo {
                    master_codes: v.master_codes.clone(),
                    hard_reset_sw: v.hard_reset_sw.clone(),
                    flash_tool: v.flash_tool.clone(),
                };
            }
        }
    }
    ButtonInfo {
        master_codes: vec!["*#7370# (generic S30+/S40+)".into()],
        hard_reset_sw: "Power + Volume Up (shikilia 10s)".into(),
        flash_tool: "Miracle Box / CM2".into(),
    }
}

pub fn button_brand_known(brand: &str) -> bool {
    let b = brand.to_lowercase();
    matches!(load(), Ok(x) if x.button_phones.contains_key(&b))
}

pub fn ensure_brand(brand: &str) -> Result<()> {
    let b = brand.to_lowercase();
    if let Ok(x) = load() {
        if x.smartphones.contains_key(&b) || x.button_phones.contains_key(&b) {
            return Ok(());
        }
    }
    bail!("Brand '{brand}' haipatikani kwenye data/mobile/brands.json")
}
