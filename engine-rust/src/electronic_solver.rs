//! Electronic Devices Solver — 100% from data/devices/* + devices_catalog.json
//! Hardware: guide for human. Mixed: firmware/reset steps listed.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceEntry {
    pub kundi: String,
    pub kifaa: String,
    pub aina: Vec<String>,
    pub matatizo: Vec<String>,
    pub suluhisho: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevicesCatalog {
    pub version: Option<u32>,
    pub jina: Option<String>,
    pub jumla_vifaa_catalog: Option<usize>,
    pub devices: Vec<DeviceEntry>,
    pub kanuni_10: Option<Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeviceHit {
    pub kifaa: String,
    pub kundi: String,
    pub tatizo: String,
    pub score: i32,
    pub suluhisho: Vec<String>,
    pub aina: Vec<String>,
    pub domain: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ElectronicSolveResult {
    pub query: String,
    pub hits: Vec<DeviceHit>,
    pub summary_sw: String,
    pub domain: String,
    pub devices_in_catalog: usize,
}

fn strs_arr(v: &Value, key: &str) -> Vec<String> {
    v.get(key)
        .and_then(|x| x.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|x| x.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default()
}

/// Prefer data/devices/*.json parts (100% split catalog)
fn load_from_devices_dir(data_root: &Path) -> Option<DevicesCatalog> {
    let dir = data_root.join("devices");
    if !dir.is_dir() {
        return None;
    }
    let mut devices = Vec::new();
    let mut kanuni_10 = None;
    if let Ok(rd) = fs::read_dir(&dir) {
        for ent in rd.flatten() {
            let path = ent.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name == "manifest.json" {
                continue;
            }
            let Ok(t) = fs::read_to_string(&path) else {
                continue;
            };
            let Ok(v) = serde_json::from_str::<Value>(&t) else {
                continue;
            };
            if name == "kanuni_10.json" {
                kanuni_10 = Some(v);
                continue;
            }
            // part: { kundi, devices: [...] }
            if let Some(arr) = v.get("devices").and_then(|x| x.as_array()) {
                for d in arr {
                    devices.push(DeviceEntry {
                        kundi: d
                            .get("kundi")
                            .and_then(|x| x.as_str())
                            .unwrap_or("")
                            .to_string(),
                        kifaa: d
                            .get("kifaa")
                            .and_then(|x| x.as_str())
                            .unwrap_or("")
                            .to_string(),
                        aina: strs_arr(d, "aina"),
                        matatizo: strs_arr(d, "matatizo"),
                        suluhisho: strs_arr(d, "suluhisho"),
                    });
                }
            }
        }
    }
    if devices.is_empty() {
        return None;
    }
    Some(DevicesCatalog {
        version: Some(1),
        jina: Some("Electronic Devices Solver".into()),
        jumla_vifaa_catalog: Some(devices.len()),
        devices,
        kanuni_10,
    })
}

fn load_catalog(data_root: &Path) -> Result<DevicesCatalog, String> {
    // 1) 100% parts
    if let Some(c) = load_from_devices_dir(data_root) {
        return Ok(c);
    }
    // 2) monolithic files
    let candidates = [
        data_root.join("devices_catalog.json"),
        data_root.join("electronic_devices_solver.json"),
        data_root.join("devices_solver.json"),
    ];
    for p in &candidates {
        if !p.exists() {
            continue;
        }
        let t = fs::read_to_string(p).map_err(|e| e.to_string())?;
        if let Ok(c) = serde_json::from_str::<DevicesCatalog>(&t) {
            if !c.devices.is_empty() {
                return Ok(c);
            }
        }
        if let Ok(v) = serde_json::from_str::<Value>(&t) {
            let eds = v
                .get("electronic_devices_solver")
                .cloned()
                .unwrap_or(v.clone());
            let mut devices = Vec::new();
            if let Some(obj) = eds.as_object() {
                for (kundi, val) in obj {
                    let Some(vifaa) = val.get("vifaa").and_then(|x| x.as_object()) else {
                        continue;
                    };
                    for (name, dev) in vifaa {
                        devices.push(DeviceEntry {
                            kundi: kundi.clone(),
                            kifaa: name.clone(),
                            aina: strs_arr(dev, "aina"),
                            matatizo: strs_arr(dev, "matatizo"),
                            suluhisho: strs_arr(dev, "suluhisho"),
                        });
                    }
                }
            }
            if !devices.is_empty() {
                return Ok(DevicesCatalog {
                    version: Some(1),
                    jina: eds.get("jina").and_then(|x| x.as_str()).map(|s| s.to_string()),
                    jumla_vifaa_catalog: Some(devices.len()),
                    devices,
                    kanuni_10: v.get("suluhisho_kanuni_10").cloned(),
                });
            }
        }
    }
    Err("devices catalog haipo (data/devices/ au devices_catalog.json)".into())
}

fn score_match(msg: &str, kifaa: &str, tatizo: &str) -> i32 {
    let msg = msg.to_lowercase();
    let mut score = 0i32;
    let k = kifaa.to_lowercase();
    let t = tatizo.to_lowercase();
    for tok in msg.split(|c: char| !c.is_alphanumeric()) {
        if tok.len() < 3 {
            continue;
        }
        if k.contains(tok) {
            score += 3;
        }
        if t.contains(tok) {
            score += 4;
        }
    }
    if msg.contains(&k) {
        score += 5;
    }
    score
}

fn domain_for(suluhisho: &[String], tatizo: &str) -> String {
    let blob = format!("{} {}", tatizo, suluhisho.join(" ")).to_lowercase();
    let soft = ["firmware", "update", "app", "wifi", "software", "reset", "restart"];
    if soft.iter().any(|s| blob.contains(s)) {
        "mixed".into()
    } else {
        "hardware".into()
    }
}

pub fn search_devices(data_root: &Path, msg: &str, limit: usize) -> ElectronicSolveResult {
    let catalog = match load_catalog(data_root) {
        Ok(c) => c,
        Err(e) => {
            return ElectronicSolveResult {
                query: msg.into(),
                hits: vec![],
                summary_sw: e,
                domain: "unknown".into(),
                devices_in_catalog: 0,
            };
        }
    };
    let n = catalog.devices.len();
    let mut hits = Vec::new();
    for d in &catalog.devices {
        for tatizo in &d.matatizo {
            let sc = score_match(msg, &d.kifaa, tatizo);
            if sc <= 0 {
                continue;
            }
            hits.push(DeviceHit {
                kifaa: d.kifaa.clone(),
                kundi: d.kundi.clone(),
                tatizo: tatizo.clone(),
                score: sc,
                suluhisho: d.suluhisho.clone(),
                aina: d.aina.clone(),
                domain: domain_for(&d.suluhisho, tatizo),
            });
        }
    }
    hits.sort_by(|a, b| b.score.cmp(&a.score));
    hits.truncate(limit.max(1));
    let domain = if hits.is_empty() {
        "unknown".into()
    } else if hits.iter().all(|h| h.domain == "hardware") {
        "hardware".into()
    } else {
        "mixed".into()
    };
    let summary_sw = if let Some(h) = hits.first() {
        format!(
            "Kifaa: {} ({}) · Tatizo: {} · Domain: {} · Hatua: {}",
            h.kifaa,
            h.kundi,
            h.tatizo,
            h.domain,
            h.suluhisho
                .iter()
                .take(5)
                .cloned()
                .collect::<Vec<_>>()
                .join(" → ")
        )
    } else {
        format!(
            "Hakuna match (catalog: {} vifaa / 100%). Jaribu: TV, Fridge, Router…",
            n
        )
    };
    ElectronicSolveResult {
        query: msg.into(),
        hits,
        summary_sw,
        domain,
        devices_in_catalog: n,
    }
}

pub fn catalog_stats(data_root: &Path) -> Value {
    match load_catalog(data_root) {
        Ok(c) => {
            let mut m: BTreeMap<String, usize> = BTreeMap::new();
            for d in &c.devices {
                *m.entry(d.kundi.clone()).or_insert(0) += 1;
            }
            let problems: usize = c.devices.iter().map(|d| d.matatizo.len()).sum();
            serde_json::json!({
                "jina": c.jina,
                "devices": c.devices.len(),
                "problems_indexed": problems,
                "coverage": "100% parts under data/devices/",
                "kundis": m
            })
        }
        Err(e) => serde_json::json!({ "error": e }),
    }
}
