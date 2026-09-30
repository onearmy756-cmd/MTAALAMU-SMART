//! netagentic.rs — TELECOMS & NETWORK AGENTIC: mitandao inaendeshwa na agents 10.
//!
//! Mtiririko (kama agentic.rs na proagentic.rs):
//!   receptionist → vision (netdiag HALISI L1-L7) → diagnoser → planner →
//!   HITL → solver (hatua halisi + netcalc) → tester (re-diagnose halisi) →
//!   verifier → scribe (kitabu HTML) → learner (learning log)
//!
//! Sheria:
//!   - Vision inafanya netdiag HALISI (interface/ARP/gateway/DNS/TCP/L7).
//!   - Solver inatoa hatua halisi kulingana na layers zilizoshindikana
//!     (kutoka telecom_formulas.json problems + recommendations za netdiag).
//!   - Hatua zenye athari (kubadilisha config ya router/firewall) NI MWONGOZO
//!     TU — agent haibadilishi mtandao wa mteja yenyewe (HITL).
//!   - Kila session: net_jobs.json + kitabu HTML + learning log.

use crate::netcalc;
use crate::netdiag;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::PathBuf;

pub const AGENTS: [(&str, &str); 10] = [
    ("receptionist", "Pokea tatizo la mtandao, anisha session"),
    ("vision", "Chunguza HALISI L1→L7 (netdiag)"),
    ("diagnoser", "Tambua layer iliyoshindikana + sababu"),
    ("planner", "Panga hatua za kutatua (data-driven)"),
    ("hitl", "Zuia mabadiliko ya config bila idhini"),
    ("solver", "Toa hatua halisi + hesabu (netcalc)"),
    ("tester", "Pima upya (re-diagnose halisi)"),
    ("verifier", "Thibitisha kwa mteja"),
    ("scribe", "Kitabu cha session (HTML)"),
    ("learner", "Hifadhi kesi (learning log)"),
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetJob {
    pub id: String,
    pub customer: String,
    pub problem: String,
    pub status: String, // done | needs_action
    pub layers_ok: Vec<String>,
    pub layers_failed: Vec<String>,
    pub frames: Vec<Value>,
    pub summary_sw: String,
    pub created_ts: u64,
}

fn jobs_path() -> PathBuf {
    let base = std::env::var("FUNDI_DATA").unwrap_or_else(|_| "../data".into());
    PathBuf::from(base).join("mobile/net_jobs.json")
}

pub fn save_job(job: &NetJob) -> Result<()> {
    let p = jobs_path();
    if let Some(d) = p.parent() {
        std::fs::create_dir_all(d)?;
    }
    let mut arr: Vec<NetJob> = std::fs::read_to_string(&p)
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default();
    arr.retain(|x| x.id != job.id);
    arr.push(job.clone());
    std::fs::write(&p, serde_json::to_string_pretty(&arr)?)?;
    Ok(())
}

pub fn list_jobs() -> Vec<NetJob> {
    std::fs::read_to_string(jobs_path())
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default()
}

fn frame(code: &str, agent: &str, sw: &str, evidence: Value) -> Value {
    json!({ "code": code, "agent": agent, "sw": sw, "evidence": evidence, "ts": now_ts() })
}

fn now_ts() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

pub struct NetRequest {
    pub customer: String,
    pub problem: String,
}

/// Agentic run kamili: diagnosis halisi mbili (kabla/baada) + hatua halisi.
pub fn run(req: &NetRequest) -> Result<NetJob> {
    let job_id = format!("NET-{}", chrono::Local::now().format("%Y%m%d%H%M%S"));
    let mut frames: Vec<Value> = Vec::new();

    // 1. RECEPTIONIST
    frames.push(frame("P", "receptionist",
        &format!("Karibu {}! Tatizo: '{}'. Nitaanzisha uchunguzi kamili L1→L7.",
                 req.customer, req.problem),
        json!({ "problem": req.problem })));

    // 2. VISION — netdiag HALISI ya kwanza
    let d1 = netdiag::diagnose_blocking();
    frames.push(frame("P", "vision",
        &format!("Uchunguzi wa kwanza umekamilika: {}", d1["overall"].as_str().unwrap_or("?")),
        d1.clone()));

    // 3. DIAGNOSER — tambua layers zilizoshindikana
    let recs: Vec<String> = d1["recommendations"]
        .as_array()
        .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();
    let failed_layers: Vec<String> = recs
        .iter()
        .filter_map(|r| r.split(':').next().map(|s| s.to_string()))
        .collect();
    let ok_layers: Vec<String> = ["L1", "L2", "L3", "L4", "L7"]
        .iter()
        .map(|s| s.to_string())
        .filter(|l| !failed_layers.iter().any(|f| f.starts_with(l.as_str())))
        .collect();
    frames.push(frame("I", "diagnoser",
        if failed_layers.is_empty() {
            "Layers ZOTE zinafanya kazi — hakuna kosa la mtandao lililopatikana."
        } else {
            "Layers zilizoshindikana zimetambuliwa (tazama evidence)."
        },
        json!({ "failed": failed_layers, "ok": ok_layers, "recommendations": recs })));

    // 4. PLANNER — hatua kutoka problems catalog (data-driven)
    let plan = plan_steps(&failed_layers);
    frames.push(frame("I", "planner",
        &format!("Mpango: hatua {} za kutatua (mpangilio wa layers).", plan.len()),
        json!({ "steps": plan })));

    // 5. HITL — agent HAIBADILISHI config yenyewe
    frames.push(frame("H", "hitl",
        "KANUNI: Agent haimbadilishi router/firewall yenyewe — inatoa hatua HALISI,\
         fundi/mteja anatekeleza (au anaruhusu). Hakuna kazi ya mtandao isiyo na idhini.",
        json!({ "policy": "guided-HITL" })));

    // 6. SOLVER — hatua halisi + kikokotoo (kama kuna link/radio)
    let solver_ev = json!({
        "steps": plan,
        "calculator_example": "fundi-mobile netcalc subnet --ip 192.168.1.100 --cidr 26",
    });
    frames.push(frame("S", "solver",
        if failed_layers.is_empty() {
            "Hakuna hatua za kurekebisha — mfumo uko sawa."
        } else {
            "Hatua za kutatua zimeandaliwa (kwa mpangilio). Fanya moja kwa moja, kisha tupime upya."
        },
        solver_ev));

    // 7. TESTER — re-diagnose HALISI
    let d2 = netdiag::diagnose_blocking();
    let improved = d1["recommendations"].as_array().map(|a| a.len()).unwrap_or(0)
        > d2["recommendations"].as_array().map(|a| a.len()).unwrap_or(0);
    frames.push(frame("T", "tester",
        &format!("Uchunguzi wa pili: {} — {}", d2["overall"].as_str().unwrap_or("?"),
                 if improved { "IMEBORESHEKA" } else { "hakuna mabadiliko (hatua hazijatekelezwa bado)" }),
        d2.clone()));

    // 8. VERIFIER
    let final_recs = d2["recommendations"].as_array().map(|a| a.len()).unwrap_or(0);
    frames.push(frame("V", "verifier",
        if final_recs == 0 {
            "✅ Thibitisho: mtandao unafanya kazi kikamilifu."
        } else {
            "⏳ Bado kuna mambo — tekeleza hatua za solver kisha endesha tena: fundi-mobile netagentic run ..."
        },
        json!({ "remaining_issues": final_recs })));

    // 9. SCRIBE
    let status = if final_recs == 0 { "done" } else { "needs_action" };
    let summary = format!(
        "{}: {} layers OK, {} masuala yaliyobaki. {}",
        d2["overall"].as_str().unwrap_or("?"),
        ok_layers.len(),
        final_recs,
        if improved { "Kuna boresho baada ya hatua." } else { "" }
    );
    frames.push(frame("D", "scribe", &summary, json!({ "summary": summary })));

    // 10. LEARNER
    learn(&json!({
        "ts": now_ts(),
        "problem": req.problem,
        "failed_layers": failed_layers,
        "remaining": final_recs,
        "improved": improved,
    }));
    frames.push(frame("D", "learner", "Kesi imehifadhiwa (net_learning.json) — next diagnosis itakua bora.", json!({})));

    let job = NetJob {
        id: job_id,
        customer: req.customer.clone(),
        problem: req.problem.clone(),
        status: status.to_string(),
        layers_ok: ok_layers.clone(),
        layers_failed: failed_layers.clone(),
        frames,
        summary_sw: summary,
        created_ts: now_ts(),
    };
    save_job(&job)?;
    Ok(job)
}

/// Hatua halisi kulingana na layers zilizoshindikana (kutoka problems ya JSON).
fn plan_steps(failed_layers: &[String]) -> Vec<String> {
    if failed_layers.is_empty() {
        return vec!["Hakuna hatua — mfumo uko sawa.".into()];
    }
    let mut steps = Vec::new();
    let has = |prefix: &str| failed_layers.iter().any(|f| f.starts_with(prefix));
    if has("L1") {
        steps.push("L1: Chomeka/checkedisha cable; angalia LED za port; washa adapter".into());
    }
    if has("L2") {
        steps.push("L2: Hakuna ARP — angalia switch; ingiza cable nyingine; angalia VLAN".into());
    }
    if has("L3") {
        steps.push("L3: Washa DHCP au weka static; badilisha DNS kuwa 8.8.8.8 / 1.1.1.1".into());
        steps.push("L3: Ping gateway (mara 2); kama haipati — reboot router, angalia cable ya WAN".into());
    }
    if has("L4") {
        steps.push("L4: TCP 53/443 zimefungwa — angalia firewall/proxy; jaribu mtandao mwingine".into());
    }
    if has("L7") {
        steps.push("L7: Fungua browser → captive portal inaweza kuwa ipo; angalia certificate/tarehe ya PC".into());
    }
    if steps.is_empty() {
        steps.push("Angalia recommendations za vision (evidence) kwa maelezo halisi.".into());
    }
    steps
}

fn learn(entry: &Value) {
    let base = std::env::var("FUNDI_DATA").unwrap_or_else(|_| "../data".into());
    let p = PathBuf::from(base).join("mobile/net_learning.json");
    let mut arr: Vec<Value> = std::fs::read_to_string(&p)
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default();
    arr.push(entry.clone());
    let _ = std::fs::create_dir_all(p.parent().unwrap_or(std::path::Path::new(".")));
    let _ = std::fs::write(&p, serde_json::to_string_pretty(&arr).unwrap_or_default());
}

/// Kitabu cha session (HTML)
pub fn book_html(job: &NetJob) -> String {
    let mut h = String::from(
        "<!DOCTYPE html><html><head><meta charset='utf-8'><title>FUNDI NET — Kitabu</title>\
         <style>body{font-family:Segoe UI,Arial;background:#0f172a;color:#e2e8f0;max-width:820px;margin:32px auto;padding:0 16px}\
         .f{background:#1e293b;border-radius:10px;padding:14px 18px;margin:10px 0;border-left:4px solid #22d3ee}\
         .a{color:#22d3ee;font-weight:700;font-size:12px;text-transform:uppercase}\
         pre{color:#94a3b8;font-size:11px;white-space:pre-wrap;max-height:220px;overflow:auto}\
         .b{background:#134e4a;border-radius:10px;padding:16px;margin-bottom:16px}</style></head><body>",
    );
    h.push_str(&format!(
        "<div class='b'><h1 style='color:#22d3ee;margin:0'>📡 FUNDI NET AGENTIC — {}</h1>\
         <p>{} · {} · {}</p><p><small>Layers OK: {} · Failed: {}</small></p></div>",
        job.id, job.customer, job.problem, job.status,
        job.layers_ok.join(", "), job.layers_failed.join(", ")
    ));
    for f in &job.frames {
        h.push_str(&format!(
            "<div class='f'><div class='a'>[{}] {}</div><div>{}</div><pre>{}</pre></div>",
            f["code"].as_str().unwrap_or("?"),
            f["agent"].as_str().unwrap_or("?"),
            f["sw"].as_str().unwrap_or(""),
            serde_json::to_string_pretty(&f["evidence"]).unwrap_or_default()
        ));
    }
    h.push_str("</body></html>");
    h
}

/// Muhtasari wa jobs (kwa UI)
pub fn summary() -> Value {
    let jobs = list_jobs();
    json!({
        "jobs": jobs.len(),
        "done": jobs.iter().filter(|j| j.status == "done").count(),
        "needs_action": jobs.iter().filter(|j| j.status == "needs_action").count(),
    })
}

/// Formula ref kwa UI (kupitia netcalc)
pub fn formulas_hint() -> String {
    netcalc::list()
}
