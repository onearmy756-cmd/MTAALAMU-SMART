//! SERVICES — data layer ya data/mobile/services.json (bei za TZS + risk + consent).

use anyhow::{bail, Result};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
pub struct Service {
    pub id: String,
    pub name_sw: String,
    pub price: u64,
    pub minutes: u32,
    pub risk: String,
    pub requires_consent: bool,
    pub destroys_data: bool,
}

#[derive(Debug, Deserialize)]
struct File {
    #[serde(default)]
    services: Vec<Service>,
}

pub fn load() -> Result<Vec<Service>> {
    let base = std::env::var("FUNDI_DATA").unwrap_or_else(|_| "../data".into());
    let path = PathBuf::from(base).join("mobile/services.json");
    let txt = std::fs::read_to_string(&path)
        .map_err(|e| anyhow::anyhow!("soma {}: {e}", path.display()))?;
    Ok(serde_json::from_str::<File>(&txt)?.services)
}

pub fn get(id: &str) -> Result<Service> {
    load()?
        .into_iter()
        .find(|s| s.id == id)
        .ok_or_else(|| anyhow::anyhow!("Huduma '{id}' haipo kwenye services.json"))
}

pub fn price_of(id: &str) -> Result<u64> {
    Ok(get(id)?.price)
}

pub fn catalog_sw() -> String {
    match load() {
        Ok(list) => {
            let mut out = String::from("HUDUMA ZA FUNDI MOBILE (TZS):\n");
            for s in &list {
                out.push_str(&format!(
                    "  {:<28} {:>10} · dakika {} · risk {}\n",
                    s.name_sw, s.price, s.minutes, s.risk
                ));
            }
            out
        }
        Err(e) => format!("services.json haijapatikana: {e}"),
    }
}

pub fn require(id: &str) -> Result<Service> {
    let s = get(id)?;
    if !s.requires_consent {
        bail!("Huduma {id} haitaji consent")
    }
    Ok(s)
}
