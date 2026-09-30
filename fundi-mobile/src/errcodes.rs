//! errcodes.rs — ERROR CODES: lookup + AGENTIC SOLVE (vision halisi kulingana na code).
//!
//! Mtiririko: mteja anatoa code (mf. "0x0000007B") →
//!   vision (sysvision/netdiag/shield kulingana na aina ya code) →
//!   diagnoser (causes halisi kutoka catalog) → planner (fix steps) →
//!   HITL (hatua zenye athari zinahitaji idhini) → scribe.
//!
//! Data: data/mobile/error_codes.json

use crate::{netdiag, shield, sysvision};
use anyhow::{bail, Result};
use serde::Deserialize;
use serde_json::{json, Value};
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct ErrorCode {
    pub code: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub name: String,
    #[serde(default)]
    pub severity: String,
    #[serde(default)]
    pub causes: Vec<String>,
    #[serde(default)]
    pub fix_sw: String,
    #[serde(default)]
    pub agent: String,
}

#[derive(Debug, Deserialize)]
struct File {
    codes: Vec<ErrorCode>,
}

fn load_file() -> Result<Vec<ErrorCode>> {
    let base = std::env::var("FUNDI_DATA").unwrap_or_else(|_| "../data".into());
    let path = PathBuf::from(base).join("mobile/error_codes.json");
    let txt = std::fs::read_to_string(&path)
        .map_err(|e| anyhow::anyhow!("soma {}: {e}", path.display()))?;
    Ok(serde_json::from_str::<File>(&txt)?.codes)
}

fn find_code(query: &str) -> Result<ErrorCode> {
    let q = query.trim().to_uppercase().replace(' ', "");
    load_file()?
        .into_iter()
        .find(|c| {
            c.code.to_uppercase().replace(' ', "") == q
                || c.name.to_uppercase().contains(&q) && q.len() >= 4
        })
        .ok_or_else(|| anyhow::anyhow!(
            "Code '{query}' haipo kwenye catalog ({} codes). Tazama zote: fundi-mobile errors list",
            load_file().map(|l| l.len()).unwrap_or(0)
        ))
}

/// Orodha ya codes zote
pub fn list(filter: Option<&str>) -> String {
    match load_file() {
        Ok(codes) => {
            let mut out = String::from("🚨 ERROR CODES (BSOD/Windows/Network/App/Security):\n\n");
            for c in &codes {
                let show = match filter {
                    Some(f) => c.kind.eq_ignore_ascii_case(f) || c.code.to_uppercase().contains(&f.to_uppercase()),
                    None => true,
                };
                if show {
                    out.push_str(&format!(
                        "  {:<38} [{}/{}] {}\n",
                        c.code, c.kind, c.severity, c.name
                    ));
                }
            }
            out.push_str("\nSuluhisho: fundi-mobile errors solve <code>\n");
            out
        }
        Err(e) => format!("{e}"),
    }
}

/// Solve ya AGENTIC: vision halisi + fix plan + HITL note + scribe.
pub fn solve(query: &str) -> Result<String> {
    let c = find_code(query)?;
    let mut out = format!(
        "\n🚨 {} — {}\nAina: {} · Severity: {}\n\nSABABU (za kawaida):\n",
        c.code, c.name, c.kind, c.severity
    );
    for cause in &c.causes {
        out.push_str(&format!("  • {cause}\n"));
    }

    // VISION halisi kulingana na aina ya code
    let (agent_name, vision) = match c.agent.as_str() {
        "netdiag" => ("netdiag (L1-L7 halisi)", netdiag::diagnose_blocking()),
        "shield" => ("shield (AV/firewall halisi)", shield::audit()?),
        _ => ("sysvision (PC scan halisi)", sysvision::scan()),
    };
    out.push_str(&format!("\n🔎 VISION ({agent_name}) — uchunguzi wa HALISI:\n"));
    match &vision {
        v if v.get("findings").is_some() => {
            let findings: Vec<String> = v["findings"]
                .as_array()
                .map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect())
                .unwrap_or_default();
            if findings.is_empty() {
                out.push_str("  • Hakuna findings — vifaa viko sawa kwa uchunguzi huu\n");
            } else {
                for f in &findings {
                    out.push_str(&format!("  • {f}\n"));
                }
            }
        }
        v if v.get("overall").is_some() => {
            out.push_str(&format!("  • {}\n", v["overall"].as_str().unwrap_or("?")));
            if let Some(recs) = v["recommendations"].as_array() {
                for r in recs {
                    out.push_str(&format!("  • {}\n", r.as_str().unwrap_or("")));
                }
            }
        }
        v => out.push_str(&format!("  {}\n", serde_json::to_string_pretty(v).unwrap_or_default())),
    }

    // PLAN + HITL
    out.push_str("\n🛠️ SULUHISHO (hatua halisi):\n");
    for line in c.fix_sw.split("; ") {
        out.push_str(&format!("  {line}\n"));
    }
    let destructive = ["reset", "wipe", "format", "uninstall", "restore", "futa", "badilisha password"]
        .iter()
        .any(|k| c.fix_sw.to_lowercase().contains(k));
    out.push_str(if destructive {
        "\n⚠️ HITL: Hatua hii ina athari — THIBITISHA NA MMILIKI kabla (consent: fundi-mobile consent add ...)\n"
    } else {
        "\n✅ Hatua hizi ni za usalama (haziharibu data)\n"
    });

    // SCRIBE — save session
    let session = json!({
        "kind": "errcode-solve",
        "ts": chrono::Local::now().to_rfc3339(),
        "code": c.code,
        "agent": c.agent,
        "vision_summary": vision.to_string().chars().take(400).collect::<String>(),
        "destructive": destructive,
    });
    append_learning(&session);
    out.push_str("\n📝 Session imehifadhiwa (errcode_learning.json)");
    Ok(out)
}

fn append_learning(entry: &Value) {
    let base = std::env::var("FUNDI_DATA").unwrap_or_else(|_| "../data".into());
    let p = PathBuf::from(base).join("mobile/errcode_learning.json");
    let mut arr: Vec<Value> = std::fs::read_to_string(&p)
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default();
    arr.push(entry.clone());
    let _ = std::fs::create_dir_all(p.parent().unwrap_or(std::path::Path::new(".")));
    let _ = std::fs::write(&p, serde_json::to_string_pretty(&arr).unwrap_or_default());
}

pub fn bail_help() -> Result<()> {
    bail!("errors: list [bsod|windows|network|app|security] | solve <code>")
}
