//! Safe software remediation — HITL required for destructive actions
//! Production: orodha ya vitendo salama + utekelezaji wa hiari

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemediationAction {
    pub id: String,
    pub title_sw: String,
    pub description_sw: String,
    pub risk: String, // low | medium | high
    pub requires_hitl: bool,
    pub platform: String, // all | linux | windows
}

#[derive(Debug, Clone, Serialize)]
pub struct RemediationResult {
    pub action_id: String,
    pub ok: bool,
    pub message_sw: String,
    pub detail: String,
}

pub fn catalog() -> Vec<RemediationAction> {
    vec![
        RemediationAction {
            id: "list_temp".into(),
            title_sw: "Orodhesha faili za muda".into(),
            description_sw: "Inaonyesha folda za temp — haisafishi bila ruhusa".into(),
            risk: "low".into(),
            requires_hitl: false,
            platform: "all".into(),
        },
        RemediationAction {
            id: "clear_user_temp".into(),
            title_sw: "Safisha folda ya temp ya mtumiaji".into(),
            description_sw: "Inafuta faili katika TEMP ya user (HITL)".into(),
            risk: "medium".into(),
            requires_hitl: true,
            platform: "all".into(),
        },
        RemediationAction {
            id: "report_top_cpu".into(),
            title_sw: "Ripoti michakato ya CPU juu".into(),
            description_sw: "Inaandika orodha — haizimi michakato".into(),
            risk: "low".into(),
            requires_hitl: false,
            platform: "all".into(),
        },
        RemediationAction {
            id: "sync_disk".into(),
            title_sw: "Flush disk buffers (sync)".into(),
            description_sw: "Linux: amri sync — salama".into(),
            risk: "low".into(),
            requires_hitl: false,
            platform: "linux".into(),
        },
    ]
}

pub fn run_action(id: &str, hitl_approved: bool) -> RemediationResult {
    let cat = catalog();
    let Some(action) = cat.iter().find(|a| a.id == id) else {
        return RemediationResult {
            action_id: id.into(),
            ok: false,
            message_sw: "Kitendo hakijulikani".into(),
            detail: String::new(),
        };
    };
    if action.requires_hitl && !hitl_approved {
        return RemediationResult {
            action_id: id.into(),
            ok: false,
            message_sw: "HITL inahitajika — bofya RUHUSU kwanza".into(),
            detail: action.description_sw.clone(),
        };
    }

    match id {
        "list_temp" => {
            let tmp = std::env::temp_dir();
            let count = fs::read_dir(&tmp).map(|d| d.count()).unwrap_or(0);
            RemediationResult {
                action_id: id.into(),
                ok: true,
                message_sw: format!("Folda ya temp: {} ({} entries)", tmp.display(), count),
                detail: tmp.display().to_string(),
            }
        }
        "clear_user_temp" => {
            let tmp = std::env::temp_dir();
            let mut removed = 0u64;
            if let Ok(rd) = fs::read_dir(&tmp) {
                for e in rd.flatten() {
                    let p = e.path();
                    // only plain files, not dirs — safer
                    if p.is_file() {
                        if fs::remove_file(&p).is_ok() {
                            removed += 1;
                        }
                    }
                }
            }
            RemediationResult {
                action_id: id.into(),
                ok: true,
                message_sw: format!("Zimefutwa faili {} katika temp", removed),
                detail: tmp.display().to_string(),
            }
        }
        "report_top_cpu" => {
            let snap = crate::deep_probe::deep_probe(8);
            let lines: Vec<String> = snap
                .processes
                .iter()
                .map(|p| format!("{} pid={} cpu={}%", p.name, p.pid, p.cpu))
                .collect();
            RemediationResult {
                action_id: id.into(),
                ok: true,
                message_sw: "Orodha ya CPU juu imeandaliwa".into(),
                detail: lines.join("\n"),
            }
        }
        "sync_disk" => {
            if cfg!(target_os = "linux") || cfg!(target_os = "macos") {
                let out = Command::new("sync").output();
                match out {
                    Ok(o) if o.status.success() => RemediationResult {
                        action_id: id.into(),
                        ok: true,
                        message_sw: "sync imefanikiwa".into(),
                        detail: String::new(),
                    },
                    Ok(o) => RemediationResult {
                        action_id: id.into(),
                        ok: false,
                        message_sw: "sync imeshindikana".into(),
                        detail: String::from_utf8_lossy(&o.stderr).into(),
                    },
                    Err(e) => RemediationResult {
                        action_id: id.into(),
                        ok: false,
                        message_sw: format!("sync error: {}", e),
                        detail: String::new(),
                    },
                }
            } else {
                RemediationResult {
                    action_id: id.into(),
                    ok: false,
                    message_sw: "sync haipatikani kwenye platform hii".into(),
                    detail: String::new(),
                }
            }
        }
        _ => RemediationResult {
            action_id: id.into(),
            ok: false,
            message_sw: "Haijatekelezwa".into(),
            detail: String::new(),
        },
    }
}

/// Write remediation log for audit (production)
pub fn log_result(data_root: &Path, result: &RemediationResult) {
    let path = data_root.join("remediation_log.json");
    let mut arr = if path.exists() {
        fs::read_to_string(&path)
            .ok()
            .and_then(|t| serde_json::from_str::<Vec<serde_json::Value>>(&t).ok())
            .unwrap_or_default()
    } else {
        vec![]
    };
    arr.push(serde_json::json!({
        "ts": std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0),
        "action_id": result.action_id,
        "ok": result.ok,
        "message_sw": result.message_sw,
        "detail": result.detail,
    }));
    if arr.len() > 100 {
        arr = arr.split_off(arr.len() - 100);
    }
    let _ = fs::write(
        path,
        serde_json::to_string_pretty(&arr).unwrap_or_default(),
    );
}
