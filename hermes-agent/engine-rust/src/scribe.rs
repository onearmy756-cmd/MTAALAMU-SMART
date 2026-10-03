//! AI SCRIBE + AUTO WORK (production, backend Rust).
//!
//! Sehemu 2 za spec ya mtumiaji:
//!   1. **Auto-work**: agent inagundua matatizo YENYEW (scan loop), kila issue
//!      ina HITL request (mteja anaruhusu), solver inatekeleza automatic.
//!   2. **Scribe**: maelezo ya Kiswahili fasaha kwa kila hatua ya PIITVD,
//!      yanayoeleza mteja anachoona (wiring/ramani halisi) + voice script sw.
//!
//! Sessions zinahifadhiwa kwenye data/agentic_sessions.json (store ya kweli).

use crate::deep_probe::deep_probe;
use crate::wiring;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

// ============================================================
// AI SCRIBE — narration ya Kiswahili kwa kila hatua
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScribeFrame {
    pub step_code: String,      // P I I T V D
    pub step_name_sw: String,
    pub narration_sw: String,   // ujumbe kwa mteja
    pub voice_sw: String,       // TTS (kiswahili fasaha)
    pub visual_note: String,    // anachoana kwenye ramani/wiring
    pub evidence: Value,        // data halisi ya hatua
    pub ts: u64,
}

fn ts() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

pub struct Scribe {
    pub session_id: String,
    pub frames: Vec<ScribeFrame>,
}

impl Scribe {
    pub fn new(session_id: &str) -> Self {
        Scribe { session_id: session_id.to_string(), frames: vec![] }
    }

    /// P — Plan: maelezo ya mpango
    pub fn plan(&mut self, problem: &str, actions: &[String]) {
        let a = if actions.is_empty() {
            "hakuna vitendo vya moja kwa moja — nitakuongoza mwenyewe".into()
        } else {
            actions.join(", ")
        };
        self.push(
            "P",
            "Mpango",
            &format!("Nimekupanga mpango wa kutatua: {}. Vitendo: {}.", problem, a),
            &format!("Mpango uko tayari. Nitaenda hatua kwa hatua: {}.", a),
            "Ramani ya kifaa inaonekana; sehemu zote zime-labeliwa",
            json!({ "problem": problem, "actions": actions }),
        );
    }

    /// I — Identify: findings halisi za vision
    pub fn identify(&mut self, issues: &[String], health: &str) {
        let n = issues.len();
        let lista = if n == 0 {
            "Hakuna tatizo lililopatikana; vifaa vyote vya kawaida.".to_string()
        } else {
            format!("Nimegundua matatizo {}: {}", n, issues.join("; "))
        };
        self.push(
            "I",
            "Utambuzi",
            &lista,
            &format!("Ninaangalia kifaa chako. {} Afya: {}.", lista, health),
            "Issues zime-wekwa juu ya ramani (overlay) kwenye sehemu zilizoathirika",
            json!({ "issues": issues, "health": health }),
        );
    }

    /// I — Implement: vitendo vya solver
    pub fn implement(&mut self, executed: &[String], skipped: &[String]) {
        let done = if executed.is_empty() {
            "Hakuna vitendo vilivyotekelezwa (hardware = binadamu au HITL bado).".into()
        } else {
            format!("Vitendo vimefanyika: {}.", executed.join("; "))
        };
        let skip = if skipped.is_empty() {
            String::new()
        } else {
            format!(" Vilivyorukwa: {}.", skipped.join("; "))
        };
        self.push(
            "I",
            "Utekelezaji",
            &format!("{}{}", done, skip),
            &format!("Ninatekeleza suluhisho sasa. {}", done),
            "Ramani inaonyesha sehemu zinazorekebishwa (status inabadilika live)",
            json!({ "executed": executed, "skipped": skipped }),
        );
    }

    /// T — Test: metrics za baada ya fix
    pub fn test(&mut self, cpu: f64, ram: f64, issues_n: usize) {
        self.push(
            "T",
            "Kipimo",
            &format!(
                "Baada ya fix: CPU {:.0}%, RAM {:.0}%, issues {}.",
                cpu, ram, issues_n
            ),
            &format!(
                "Napima mfumo: CPU asilimia {:.0}, RAM asilimia {:.0}. Matatizo yaliyobaki: {}.",
                cpu, ram, issues_n
            ),
            "Gauge za CPU/RAM kwenye ramani zimeonyeshwa (halisi)",
            json!({ "cpu_pct": cpu, "ram_pct": ram, "issues": issues_n }),
        );
    }

    /// V — Verify: matokeo
    pub fn verify(&mut self, ok: bool, detail_sw: &str) {
        self.push(
            "V",
            "Uthibitisho",
            &format!("Uthibitisho: {}.", detail_sw),
            if ok {
                "Nimekamilisha. Tafadhali thibitisha wewe mwenyewe kama kila kitu kiko sawa."
            } else {
                "Kuna jambo bado. Nimeandika mapendekezo kwenye ripoti."
            },
            "Before/after metrics zimefuatana (kibodo cha kulinganisha)",
            json!({ "ok": ok, "detail": detail_sw }),
        );
    }

    /// D — Document: kitabu kidigitali
    pub fn document(&mut self, report_path: &str, knowledge_saved: bool) {
        self.push(
            "D",
            "Nyaraka",
            &format!("Ripoti kamili imeandikwa: {}.", report_path),
            "Ripoti yako kama kitabu kidigitali iko tayari. Ninahifadhi maarifa haya.",
            "Kitabu: jalada → tatizo → njia → suluhisho → tarehe/muda",
            json!({ "report": report_path, "learned": knowledge_saved }),
        );
    }

    /// Frame ya kawaida (scan/auto-work n.k.)
    pub fn note(&mut self, code: &str, name: &str, narration: &str, visual: &str, evidence: Value) {
        self.push(code, name, narration, narration, visual, evidence);
    }

    /// Frame yenye voice tofauti na narration
    pub fn note_voice(&mut self, code: &str, name: &str, narration: &str, voice: &str, visual: &str, evidence: Value) {
        self.push(code, name, narration, voice, visual, evidence);
    }

    fn push(
        &mut self,
        code: &str,
        name: &str,
        narration: &str,
        voice: &str,
        visual: &str,
        evidence: Value,
    ) {
        self.frames.push(ScribeFrame {
            step_code: code.into(),
            step_name_sw: name.into(),
            narration_sw: narration.into(),
            voice_sw: voice.into(),
            visual_note: visual.into(),
            evidence,
            ts: ts(),
        });
    }

    /// Frame kamili ya voice (mfululizo wa TTS sw)
    pub fn voice_script(&self) -> String {
        self.frames
            .iter()
            .map(|f| f.voice_sw.clone())
            .collect::<Vec<_>>()
            .join(" ")
    }
}

// ============================================================
// AUTO WORK — scan loop (agent inagundua yenyewe)
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoFinding {
    pub title: String,
    pub severity: String, // info | warning | critical
    pub hitl_requested: bool,
    pub approved: bool,
    pub resolved: bool,
    pub action_hint_sw: String,
    pub ts: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoWorkReport {
    pub session_id: String,
    pub started_at: u64,
    pub ended_at: u64,
    pub findings: Vec<AutoFinding>,
    pub scribe: Vec<ScribeFrame>,
    pub voice_script_sw: String,
    pub wiring_nodes: usize,
    pub wiring_flows: usize,
    pub summary_sw: String,
}

fn scan_issues() -> (Vec<String>, String, f64, f64) {
    let d = deep_probe(10);
    (d.issues.clone(), d.health.clone(), d.cpu_usage_pct, d.ram_usage_pct)
}

/// LEARNING LOOP halisi: hifadhi maarifa ya session kwenye learning_log.json
fn append_learning(data_root: &Path, session_id: &str, summary: &str, findings: &[AutoFinding]) {
    let path = data_root.join("learning_log.json");
    let mut arr: Vec<Value> = if path.exists() {
        fs::read_to_string(&path)
            .ok()
            .and_then(|t| serde_json::from_str(&t).ok())
            .unwrap_or_default()
    } else {
        vec![]
    };
    arr.push(json!({
        "session_id": session_id,
        "ts": ts(),
        "source": "auto-work",
        "summary_sw": summary,
        "issues": findings.iter().map(|f| f.title.clone()).collect::<Vec<_>>(),
        "severity": findings.iter().map(|f| f.severity.clone()).collect::<Vec<_>>(),
        "resolved": findings.iter().map(|f| f.resolved).collect::<Vec<_>>(),
    }));
    if arr.len() > 200 {
        arr = arr.split_off(arr.len() - 200);
    }
    let _ = fs::write(&path, serde_json::to_string_pretty(&arr).unwrap_or_default());
}

/// AUTO-WORK: scan → gundua → HITL request kwa kila issue → solve (kama approved)
/// `approve: false` = inasubiri msimamizi (production); `true` = lab/auto.
pub fn auto_work_once(data_root: &Path, session_id: &str, approve: bool) -> AutoWorkReport {
    let started = ts();
    let mut scribe = Scribe::new(session_id);
    let (issues, health, cpu, ram) = scan_issues();

    // Wiring snapshot (ramani + miunganisho halisi ya wakati huu)
    let w = wiring::system_wiring_top(8);
    let wiring_nodes = w.nodes.len();
    let wiring_flows = w.flows.len();

    scribe.note(
        "SCAN",
        "Uchanganuzi wa kiotomatiki",
        &format!(
            "Nimechanganua kifaa: nodes {}, flows {}, issues {}.",
            wiring_nodes, wiring_flows, issues.len()
        ),
        "Wiring kamili imeonekana — kila node na edge zime-labeliwa",
        json!({ "nodes": wiring_nodes, "flows": wiring_flows, "source": w.source }),
    );

    let mut findings: Vec<AutoFinding> = Vec::new();
    for i in &issues {
        findings.push(AutoFinding {
            title: i.clone(),
            severity: if i.contains("critical") || i.contains("full") || i.contains("90") {
                "critical".into()
            } else {
                "warning".into()
            },
            hitl_requested: true, // kila finding inaomba ruhusa
            approved: approve,
            resolved: false,
            action_hint_sw: "Ruhusu agent arekebishe (software) au mtaalamu aingilie (hardware)"
                .into(),
            ts: ts(),
        });
    }
    if findings.is_empty() {
        findings.push(AutoFinding {
            title: "Hakuna matatizo — mfumo uko salama".into(),
            severity: "info".into(),
            hitl_requested: false,
            approved: true,
            resolved: true,
            action_hint_sw: "Hakuna hatua".into(),
            ts: ts(),
        });
    }

    scribe.identify(&issues, &health);

    // HITL + solve (ruhusu software fixes tu)
    let titles: Vec<String> = issues.iter().map(|s| s.clone()).collect();
    let blob = titles.join(" ");
    let mut executed: Vec<String> = Vec::new();
    let mut skipped: Vec<String> = Vec::new();
    if approve && !issues.is_empty() {
        let r = crate::solve::solve_message(data_root, &blob, true);
        executed = r.executed.iter().map(|x| x.action_id.clone()).collect();
        skipped = r.skipped.clone();
        scribe.note(
            "HITL",
            "Ruhusa",
            "Ruhusa ya kiotomatiki ipo (auto/lab mode).",
            "Bodi ya HITL imeidhinisha (green)",
            json!({ "auto": true }),
        );
    } else if !issues.is_empty() {
        scribe.note_voice(
            "HITL",
            "Ruhusa",
            "Nimekupa orodha ya matatizo. Niruhusu niendeshe kiotomatiki? (RUHUSU kwenye UI)",
            "Je, unaniruhusu kuanza kurekebisha kiotomatiki?",
            "Bodi ya HITL inasubiri (amber) — RUHUSU au GHAIRI",
            json!({ "awaiting": true, "issues": issues }),
        );
    }

    scribe.implement(&executed, &skipped);

    // Test: pima upya (halisi)
    let (issues_after, _, cpu2, ram2) = scan_issues();
    scribe.test(cpu2, ram2, issues_after.len());

    let ok = issues_after.len() < issues.len() || (issues.is_empty() && issues_after.is_empty());
    let detail = if ok {
        "Matatizo yamepungua au mfumo uko salama"
    } else {
        "Matatizo yamebaki — angalia mapendekezo"
    };
    scribe.verify(ok, detail);

    // Document: hifadhi session + report + LEARNING LOOP (maarifa ya kweli)
    let report_path = save_auto_report(data_root, session_id, &findings, &scribe);
    append_learning(data_root, session_id, &format!("{} — {}", summary_prefix(&issues, &issues_after), detail), &findings);
    scribe.document(&report_path, true);

    AutoWorkReport {
        session_id: session_id.to_string(),
        started_at: started,
        ended_at: ts(),
        findings,
        scribe: scribe.frames.clone(),
        voice_script_sw: scribe.voice_script(),
        wiring_nodes,
        wiring_flows,
        summary_sw: format!(
            "Scan: issues {} → {}. CPU {:.0}%→{:.0}%, RAM {:.0}%→{:.0}%. Wiring: {} nodes, {} flows (halisi).",
            issues.len(),
            issues_after.len(),
            cpu, cpu2, ram, ram2,
            wiring_nodes, wiring_flows
        ),
    }
}

fn summary_prefix(issues_before: &[String], issues_after: &[String]) -> String {
    format!("issues {} → {}", issues_before.len(), issues_after.len())
}

fn sessions_path(data_root: &Path) -> PathBuf {
    data_root.join("agentic_sessions.json")
}

/// Session store ya kweli: ongeza au sasisha session kwenye JSON
pub fn save_session(data_root: &Path, session_id: &str, value: &Value) -> Result<(), String> {
    let p = sessions_path(data_root);
    let mut arr: Vec<Value> = if p.exists() {
        serde_json::from_str(&fs::read_to_string(&p).map_err(|e| e.to_string())?)
            .unwrap_or_default()
    } else {
        vec![]
    };
    arr.retain(|s| s.get("session_id").and_then(|x| x.as_str()) != Some(session_id));
    arr.push(value.clone());
    if arr.len() > 100 {
        arr = arr.split_off(arr.len() - 100);
    }
    fs::write(&p, serde_json::to_string_pretty(&arr).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())
}

pub fn load_sessions(data_root: &Path) -> Vec<Value> {
    let p = sessions_path(data_root);
    fs::read_to_string(&p)
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default()
}

fn save_auto_report(
    data_root: &Path,
    session_id: &str,
    findings: &[AutoFinding],
    scribe: &Scribe,
) -> String {
    let dir = data_root.join("reports");
    let _ = fs::create_dir_all(&dir);
    let file = dir.join(format!("{}.json", session_id));
    let doc = json!({
        "session_id": session_id,
        "created_at": ts(),
        "findings": findings,
        "scribe_frames": scribe.frames,
        "voice_script_sw": scribe.voice_script(),
    });
    let path_str = file.to_string_lossy().to_string();
    let _ = fs::write(&file, serde_json::to_string_pretty(&doc).unwrap_or_default());
    path_str
}

/// AUTO-WATCH: run auto_work_once kila `interval_secs` hadi kufa
pub fn auto_watch(data_root: &Path, interval_secs: u64, approve: bool, runs: Option<u32>) {
    let mut n = 0u32;
    loop {
        let sid = format!("AW-{}-{}", ts(), n);
        let rep = auto_work_once(data_root, &sid, approve);
        println!("{}", serde_json::to_string_pretty(&rep).unwrap_or_default());
        n += 1;
        if let Some(max) = runs {
            if n >= max {
                break;
            }
        }
        std::thread::sleep(std::time::Duration::from_secs(interval_secs));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scribe_frames_zote_piitvd() {
        let mut s = Scribe::new("T1");
        s.plan("PC polepole", &["clear_temp".into()]);
        s.identify(&["CPU high".into()], "warning");
        s.implement(&["clear_temp".into()], &[]);
        s.test(30.0, 50.0, 0);
        s.verify(true, "sawa");
        s.document("/tmp/r.json", true);
        let codes: Vec<&str> = s.frames.iter().map(|f| f.step_code.as_str()).collect();
        assert_eq!(codes, vec!["P", "I", "I", "T", "V", "D"]);
        assert!(!s.voice_script().is_empty());
    }

    #[test]
    fn auto_work_ina_frame_zote() {
        let tmp = std::env::temp_dir().join(format!("fundi_scribe_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&tmp);
        let rep = auto_work_once(&tmp, "AW-test", false);
        assert!(rep.scribe.len() >= 6, "frames: {}", rep.scribe.len());
        assert!(rep.wiring_nodes >= 5);
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
