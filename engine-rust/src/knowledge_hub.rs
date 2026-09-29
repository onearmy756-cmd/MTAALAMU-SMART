//! Unified knowledge search — problems / diagnosis / trades / services / professions / devices

use serde::Serialize;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize)]
pub struct KnowledgeHit {
    pub module: String,
    pub id: String,
    pub title_sw: String,
    pub score: i32,
    pub detail_sw: String,
    pub meta: Value,
}

#[derive(Debug, Clone, Serialize)]
pub struct KnowledgeSearchResult {
    pub query: String,
    pub hits: Vec<KnowledgeHit>,
    pub stats: Value,
    pub summary_sw: String,
}

fn bi(v: &Value, lang: &str) -> String {
    if let Some(s) = v.as_str() {
        return s.to_string();
    }
    if let Some(o) = v.as_object() {
        if lang == "en" {
            if let Some(s) = o.get("en").and_then(|x| x.as_str()) {
                return s.to_string();
            }
        }
        if let Some(s) = o.get("sw").and_then(|x| x.as_str()) {
            return s.to_string();
        }
        if let Some(s) = o.get("en").and_then(|x| x.as_str()) {
            return s.to_string();
        }
    }
    String::new()
}

fn score_text(msg: &str, blob: &str) -> i32 {
    let msg = msg.to_lowercase();
    let blob = blob.to_lowercase();
    let mut score = 0i32;
    for tok in msg.split(|c: char| !c.is_alphanumeric()) {
        if tok.len() < 3 {
            continue;
        }
        if blob.contains(tok) {
            score += 2;
        }
    }
    score
}

fn load_json(path: &Path) -> Option<Value> {
    let t = fs::read_to_string(path).ok()?;
    serde_json::from_str(&t).ok()
}

fn problem_files(data_root: &Path) -> Vec<PathBuf> {
    let dir = data_root.join("knowledge/problems");
    if dir.is_dir() {
        if let Ok(rd) = fs::read_dir(&dir) {
            let files: Vec<PathBuf> = rd
                .flatten()
                .map(|e| e.path())
                .filter(|p| {
                    p.extension().and_then(|e| e.to_str()) == Some("json")
                        && p.file_name().and_then(|n| n.to_str()) != Some("manifest.json")
                })
                .collect();
            if !files.is_empty() {
                return files;
            }
        }
    }
    let mono = data_root.join("problems.json");
    if mono.exists() {
        return vec![mono];
    }
    vec![]
}

fn search_problems(data_root: &Path, msg: &str, limit: usize) -> Vec<KnowledgeHit> {
    let mut hits = Vec::new();
    for p in problem_files(data_root) {
        let Some(v) = load_json(&p) else { continue };
        let arr = v
            .get("problems")
            .and_then(|x| x.as_array())
            .cloned()
            .unwrap_or_default();
        for pobj in arr {
            let id = pobj.get("id").and_then(|x| x.as_str()).unwrap_or("").to_string();
            let trade = pobj.get("trade").and_then(|x| x.as_str()).unwrap_or("");
            let desc = bi(pobj.get("description").unwrap_or(&Value::Null), "sw");
            let sol = bi(pobj.get("solution").unwrap_or(&Value::Null), "sw");
            let syms: Vec<String> = pobj
                .get("symptoms")
                .and_then(|x| x.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|x| x.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();
            let causes: Vec<String> = pobj
                .get("causes")
                .and_then(|x| x.as_object())
                .map(|o| o.keys().cloned().collect())
                .unwrap_or_default();
            let blob = format!(
                "{} {} {} {} {} {}",
                id,
                trade,
                desc,
                sol,
                syms.join(" "),
                causes.join(" ")
            );
            let score = score_text(msg, &blob);
            if score <= 0 {
                continue;
            }
            hits.push(KnowledgeHit {
                module: "problems".into(),
                id: id.clone(),
                title_sw: if desc.is_empty() { id } else { desc },
                score,
                detail_sw: sol,
                meta: serde_json::json!({
                    "trade": trade,
                    "symptoms": syms,
                    "time_min": pobj.get("time_min"),
                    "cost_tzs": pobj.get("cost_tzs"),
                    "severity": pobj.get("severity"),
                }),
            });
        }
    }
    hits.sort_by(|a, b| b.score.cmp(&a.score));
    hits.truncate(limit);
    hits
}

fn search_diagnosis(data_root: &Path, msg: &str, limit: usize) -> Vec<KnowledgeHit> {
    let mut hits = Vec::new();
    let path = data_root.join("knowledge/diagnosis_index.json");
    let v = if path.exists() {
        load_json(&path)
    } else {
        load_json(&data_root.join("diagnosis.json"))
    };
    let Some(v) = v else {
        return hits;
    };
    let models = v.get("models").cloned().unwrap_or(Value::Null);
    if let Some(arr) = models.as_array() {
        for m in arr {
            let id = m.get("id").and_then(|x| x.as_str()).unwrap_or("").to_string();
            let title = bi(m.get("title").unwrap_or(&Value::Null), "sw");
            let trade = m.get("trade").and_then(|x| x.as_str()).unwrap_or("");
            let blob = format!("{} {} {} {}", id, title, trade, m);
            let score = score_text(msg, &blob);
            if score <= 0 {
                continue;
            }
            hits.push(KnowledgeHit {
                module: "diagnosis".into(),
                id: id.clone(),
                title_sw: title,
                score,
                detail_sw: format!("Bayesian model · trade={}", trade),
                meta: m.clone(),
            });
        }
    } else if let Some(obj) = models.as_object() {
        for (id, m) in obj {
            let title = bi(m.get("title").unwrap_or(&Value::Null), "sw");
            let trade = m.get("trade").and_then(|x| x.as_str()).unwrap_or("");
            let blob = format!("{} {} {} {}", id, title, trade, m);
            let score = score_text(msg, &blob);
            if score <= 0 {
                continue;
            }
            hits.push(KnowledgeHit {
                module: "diagnosis".into(),
                id: id.clone(),
                title_sw: title,
                score,
                detail_sw: format!("Model · trade={}", trade),
                meta: m.clone(),
            });
        }
    }
    hits.sort_by(|a, b| b.score.cmp(&a.score));
    hits.truncate(limit);
    hits
}

fn search_services(data_root: &Path, msg: &str, limit: usize) -> Vec<KnowledgeHit> {
    let mut hits = Vec::new();
    let path = data_root.join("knowledge/services_index.json");
    let v = if path.exists() {
        load_json(&path)
    } else {
        load_json(&data_root.join("services.json"))
    };
    let Some(v) = v else {
        return hits;
    };
    let arr = v
        .get("services")
        .and_then(|x| x.as_array())
        .cloned()
        .unwrap_or_default();
    for s in arr {
        let code = s.get("code").and_then(|x| x.as_str()).unwrap_or("").to_string();
        let name = s.get("name").and_then(|x| x.as_str()).unwrap_or("").to_string();
        let cat = s.get("category").and_then(|x| x.as_str()).unwrap_or("");
        let desc = s.get("description").and_then(|x| x.as_str()).unwrap_or("");
        let blob = format!("{} {} {} {}", code, name, cat, desc);
        let score = score_text(msg, &blob);
        if score <= 0 {
            continue;
        }
        hits.push(KnowledgeHit {
            module: "services".into(),
            id: code.clone(),
            title_sw: name,
            score,
            detail_sw: format!(
                "{} · TZS {} · {} min",
                cat,
                s.get("price_tzs").and_then(|x| x.as_i64()).unwrap_or(0),
                s.get("duration_min").and_then(|x| x.as_i64()).unwrap_or(0)
            ),
            meta: s,
        });
    }
    hits.sort_by(|a, b| b.score.cmp(&a.score));
    hits.truncate(limit);
    hits
}

fn search_trades(data_root: &Path, msg: &str, limit: usize) -> Vec<KnowledgeHit> {
    let mut hits = Vec::new();
    let path = data_root.join("knowledge/trades_index.json");
    let v = if path.exists() {
        load_json(&path)
    } else {
        load_json(&data_root.join("trades.json"))
    };
    let Some(v) = v else {
        return hits;
    };
    let arr = v
        .get("trades")
        .and_then(|x| x.as_array())
        .cloned()
        .unwrap_or_default();
    for t in arr {
        let id = t.get("id").and_then(|x| x.as_str()).unwrap_or("").to_string();
        let name = t
            .get("name_sw")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string();
        let skills = t.get("skills").map(|x| x.to_string()).unwrap_or_default();
        let blob = format!("{} {} {}", id, name, skills);
        let score = score_text(msg, &blob);
        if score <= 0 {
            continue;
        }
        hits.push(KnowledgeHit {
            module: "trades".into(),
            id: id.clone(),
            title_sw: name,
            score,
            detail_sw: format!(
                "Trade · skills={}",
                t.get("skills")
                    .and_then(|x| x.as_array())
                    .map(|a| a.len())
                    .unwrap_or(0)
            ),
            meta: t,
        });
    }
    hits.sort_by(|a, b| b.score.cmp(&a.score));
    hits.truncate(limit);
    hits
}

fn search_professions(data_root: &Path, msg: &str, limit: usize) -> Vec<KnowledgeHit> {
    let mut hits = Vec::new();
    let path = data_root.join("knowledge/professions_index.json");
    let v = if path.exists() {
        load_json(&path)
    } else {
        load_json(&data_root.join("professions.json"))
    };
    let Some(v) = v else {
        return hits;
    };
    let cats = v
        .get("categories")
        .and_then(|x| x.as_array())
        .cloned()
        .unwrap_or_default();
    for cat in cats {
        let cat_name = cat.get("name_sw").and_then(|x| x.as_str()).unwrap_or("");
        let proffs = cat
            .get("professions")
            .and_then(|x| x.as_array())
            .cloned()
            .unwrap_or_default();
        for p in proffs {
            let code = p.get("code").and_then(|x| x.as_str()).unwrap_or("").to_string();
            let name = p
                .get("name_sw")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string();
            let services = p.get("services").map(|x| x.to_string()).unwrap_or_default();
            let blob = format!("{} {} {} {}", code, name, cat_name, services);
            let score = score_text(msg, &blob);
            if score <= 0 {
                continue;
            }
            hits.push(KnowledgeHit {
                module: "professions".into(),
                id: code.clone(),
                title_sw: name,
                score,
                detail_sw: format!("Kategoria: {}", cat_name),
                meta: p,
            });
        }
    }
    hits.sort_by(|a, b| b.score.cmp(&a.score));
    hits.truncate(limit);
    hits
}

pub fn search_all(data_root: &Path, msg: &str, limit_per: usize) -> KnowledgeSearchResult {
    let mut hits = Vec::new();
    hits.extend(search_problems(data_root, msg, limit_per));
    hits.extend(search_diagnosis(data_root, msg, limit_per.min(5)));
    hits.extend(search_services(data_root, msg, limit_per.min(5)));
    hits.extend(search_trades(data_root, msg, limit_per.min(5)));
    hits.extend(search_professions(data_root, msg, limit_per.min(5)));
    let dev = crate::electronic_solver::search_devices(data_root, msg, limit_per.min(5));
    for h in dev.hits {
        hits.push(KnowledgeHit {
            module: "devices".into(),
            id: format!("{}/{}", h.kifaa, h.tatizo),
            title_sw: format!("{} — {}", h.kifaa, h.tatizo),
            score: h.score,
            detail_sw: h
                .suluhisho
                .iter()
                .take(4)
                .cloned()
                .collect::<Vec<_>>()
                .join(" → "),
            meta: serde_json::json!({"kundi": h.kundi, "domain": h.domain, "aina": h.aina}),
        });
    }
    hits.sort_by(|a, b| b.score.cmp(&a.score));
    hits.truncate(limit_per * 3);
    let stats = knowledge_stats(data_root);
    let summary_sw = if hits.is_empty() {
        "Hakuna match kwenye knowledge yote.".into()
    } else {
        format!(
            "{} hits · juu: [{}] {}",
            hits.len(),
            hits[0].module,
            hits[0].title_sw
        )
    };
    KnowledgeSearchResult {
        query: msg.into(),
        hits,
        stats,
        summary_sw,
    }
}

pub fn knowledge_stats(data_root: &Path) -> Value {
    let man = data_root.join("knowledge/manifest.json");
    if let Some(v) = load_json(&man) {
        return v;
    }
    serde_json::json!({
        "modules": ["problems", "diagnosis", "trades", "services", "professions", "devices"]
    })
}
