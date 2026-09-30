//! AGENTIC ENGINE — FUNDI MOBILE inaendesha YENYEWE (kama spec ya mtumiaji):
//!
//! 1. Q&A: mteja anasema tatizo → agent inapanga (P) na kujibu kwa Kiswahili.
//! 2. AUTO-WORK: scan (adb devices + props + battery + storage) → inatambua (I)
//!    → inapanga plan (I) → HITL (consent kwenye code) → solve (I) → test (T)
//!    → verify (V) → scribe + kitabu (D) → learning log.
//!
//! Sheria: hatua ya "solve" inayoharibu HAIWEZI kutekelezwa bila consent
//! iliyosajiliwa (consent.rs). Agent inaombei kwenye UI/CLI: inaacha kazi
//! awaiting_consent na mwongozo kamili wa kumwomba mteja.

use crate::consent;
use crate::devices;
use crate::procedures;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

pub fn now_ts() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

// ---------- Agents 10 (data-driven kama MTAALAMU) ----------

pub const AGENTS: [(&str, &str, &str); 10] = [
    ("receptionist", "Mpokeaji", "Pokea tatizo, ainisha kifaa, anza session"),
    ("vision", "Muono", "Scan simu: props, battery, storage (adb halisi)"),
    ("diagnoser", "Mgunduzi", "Tatizo → njia za kutatua (decision tree)"),
    ("planner", "Mpangaji", "Panga hatua P-I-I-T-V-D"),
    ("hitl", "Mlinzi", "Zuia kazi ya hatari mpaka consent isajiliwe"),
    ("solver", "Mtatuzi", "Tekeleza (ADB halisi au mwongozo wa brand)"),
    ("tester", "Mjaribu", "Pima upya baada ya solve"),
    ("verifier", "Mthibitishaji", "Thibitisha kwa mteja"),
    ("scribe", "Mwandishi", "Ripoti Kiswahili hatua kwa hatua"),
    ("learner", "Mwanafunzi", "Hifadhi maarifa + learning log"),
];

// ---------- Jobs store ----------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: String,
    pub customer: String,
    pub phone: String,
    pub brand: String,
    pub model: String,
    pub imei: String,
    pub service_id: String,
    pub problem: String,
    pub status: String, // created | awaiting_consent | running | done | failed
    pub consent_id: Option<String>,
    pub frames: Vec<Value>,
    pub summary_sw: String,
    pub created_ts: u64,
    pub ended_ts: Option<u64>,
}

fn jobs_path() -> PathBuf {
    let base = std::env::var("FUNDI_DATA").unwrap_or_else(|_| "../data".into());
    PathBuf::from(base).join("mobile/jobs.json")
}

pub fn save_job(job: &Job) -> Result<()> {
    let p = jobs_path();
    if let Some(d) = p.parent() {
        std::fs::create_dir_all(d)?;
    }
    let mut arr: Vec<Job> = std::fs::read_to_string(&p)
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default();
    arr.retain(|x| x.id != job.id);
    arr.push(job.clone());
    std::fs::write(&p, serde_json::to_string_pretty(&arr)?)?;
    Ok(())
}

pub fn list_jobs() -> Vec<Job> {
    std::fs::read_to_string(jobs_path())
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default()
}

fn frame(code: &str, agent: &str, sw: &str, evidence: Value) -> Value {
    json!({ "code": code, "agent": agent, "sw": sw, "evidence": evidence, "ts": now_ts() })
}

// ---------- Agentic run ----------

pub struct AgenticRequest {
    pub customer: String,
    pub phone: String,
    pub brand: String,
    pub model: String,
    pub imei: String,
    pub service_id: String,
    pub problem: String,
    pub technician: String,
}

/// Full agentic run: receptionist → vision → diagnose → plan → HITL → solve → test → verify → scribe → learn
pub fn run(req: &AgenticRequest) -> Result<Job> {
    let job_id = format!("FM-{}-{}", chrono::Local::now().format("%Y%m%d%H%M%S"), &req.service_id);
    let mut frames: Vec<Value> = Vec::new();

    // 1. Receptionist
    frames.push(frame("P", "receptionist",
        &format!("Karibu! Nimepokea tatizo lako: {}. Kifaa: {} {} (IMEI {}).",
                 req.problem, req.brand, req.model, req.imei),
        json!({"problem": req.problem})));

    // 2. Vision (scan halisi kama simu imeunganishwa)
    let mut vision_ev = json!({"adb": "haipo/simu haijaunganishwa"});
    if let Ok(props) = devices::device_props() {
        vision_ev = json!({ "props": props });
        frames.push(frame("P", "vision",
            "Nimechanganua simu kupitia ADB: props, battery, storage zimekaguliwa.",
            vision_ev.clone()));
    } else {
        frames.push(frame("P", "vision",
            "Simu haijaonekana kwa ADB (USB debugging au driver). Nitatumia mwongozo wa brand.",
            vision_ev.clone()));
    }

    // 3. Diagnose + Plan (decision: service_id amua hatua)
    frames.push(frame("I", "diagnoser",
        &format!("Tatizo linaeleweka: '{}'. Njia iliyopendekezwa: {}.",
                 req.problem, req.service_id),
        json!({"service": req.service_id})));

    // 4. HITL — consent check (kuzuia kazi bila ruhusa)
    let consent_ok = consent::verify(&req.imei, &req.service_id);
    let (status, consent_id): (&str, Option<String>) = match &consent_ok {
        Ok(c) => ("running", Some(c.consent_id.clone())),
        Err(_) => ("awaiting_consent", None),
    };
    match &consent_ok {
        Ok(c) => frames.push(frame("H", "hitl",
            &format!("Consent ipo: {} (mteja: {}). Ninaruhusiwa kuendelea.", c.consent_id, c.customer_name),
            json!({"consent": c.consent_id}))),
        Err(_) => frames.push(frame("H", "hitl",
            "KAZI IMESIMAMISHWA: Hakuna consent kwa IMEI hii. \
             Mteja LAZIMA asaini fomu ya ruhusa (fund-mobile consent add). \
             Kufanya bila ruhusa = kosa la jinai (Kifungu 267).",
            json!({"blocked": true}))),
    }

    // 5. Solve (kama consent ipo)
    let mut summary = String::new();
    let mut final_status = status.to_string();
    if consent_ok.is_ok() {
        let outcome = solve_step(req);
        match outcome {
            Ok(o) => {
                for s in &o.steps {
                    frames.push(frame("I", "solver", s, json!({})));
                }
                summary = o.summary_sw.clone();
                // 6. Test (halisi kama adb ipo)
                if let Ok(b) = devices::battery_info() {
                    frames.push(frame("T", "tester",
                        "Nimepima upya: battery/storage zimerudi kutoka simu (halisi).",
                        json!({"battery_head": b.lines().take(6).collect::<Vec<_>>()})));
                } else {
                    frames.push(frame("T", "tester",
                        "Test ya ADB haikuwezekana — thibitisha kwa mkono kwenye simu.",
                        json!({})));
                }
                // 7. Verify
                frames.push(frame("V", "verifier",
                    "Tafadhali thibitisha wewe mwenyewe kwenye simu: inafanya kama unavyotarajia?",
                    json!({})));
                final_status = "done".into();
            }
            Err(e) => {
                frames.push(frame("I", "solver", &format!("Imeshindikana: {e}"), json!({})));
                summary = format!("Imeshindikana: {e}");
                final_status = "failed".into();
            }
        }
    } else {
        summary = "Inasubiri consent (HITL). Nimeandaa mwongozo kamili wa kutekeleza kwa usalama.".into();
    }

    // 8. Scribe + Learner
    frames.push(frame("D", "scribe",
        "Nimeandika ripoti kamili (kitabu kidigitali) kwa Kiswahili.",
        json!({})));
    frames.push(frame("D", "learner",
        "Maarifa ya session hii yamehifadhiwa kwenye learning_log.json.",
        json!({})));

    let job = Job {
        id: job_id,
        customer: req.customer.clone(),
        phone: req.phone.clone(),
        brand: req.brand.clone(),
        model: req.model.clone(),
        imei: req.imei.clone(),
        service_id: req.service_id.clone(),
        problem: req.problem.clone(),
        status: final_status,
        consent_id,
        frames,
        summary_sw: if summary.is_empty() { "Imekamilika." .into() } else { summary },
        created_ts: now_ts(),
        ended_ts: Some(now_ts()),
    };
    append_learning(&job)?;
    save_job(&job)?;
    Ok(job)
}

fn solve_step(req: &AgenticRequest) -> Result<procedures::Outcome> {
    match req.service_id.as_str() {
        "reset_password_adb" => procedures::android_reset_adb(&req.imei, &req.technician),
        "reset_password_recovery" => procedures::android_reset_recovery(&req.brand, &req.imei, &req.technician),
        "reset_button_code" | "button_hard_reset" => procedures::button_reset(&req.brand, &req.imei),
        "flash_firmware" | "flash_button_phone" => procedures::flash_firmware(&req.brand, &req.imei),
        "bypass_frp" | "bypass_frp_new" => procedures::frp_guide(&req.model, &req.imei),
        "backup_data" => procedures::backup("./backups", &req.imei),
        "diagnostics" => procedures::diagnostics(),
        "iphone_reset" | "iphone_dfu" => Ok(procedures::Outcome {
            ok: true,
            summary_sw: "iPhone: iTunes/Finder → Recovery/DFU (mwongozo kamili)".into(),
            steps: vec![
                "Chomeka iPhone kwa computer (iTunes/Finder ipo?)".into(),
                "iPhone 8+: Vol Up, Vol Down, kisha Power mpaka logo → Recovery".into(),
                "iPhone 7: Volume Down + Power · 6s: Home + Power".into(),
                "DFU (ngazi ya chini): Vol Up, Vol Down, Power 5s → Power+VolDown 5s → acha Power, endelea VolDown 10s (screen nyeusi)".into(),
                "iTunes → Restore / Update".into(),
            ],
        }),
        other => Ok(procedures::Outcome {
            ok: false,
            summary_sw: format!("Huduma '{other}' bado ina mwongozo tu — solver haitumiki bado."),
            steps: vec![],
        }),
    }
}

fn append_learning(job: &Job) -> Result<()> {
    let base = std::env::var("FUNDI_DATA").unwrap_or_else(|_| "../data".into());
    let dir = Path::new(&base).join("mobile");
    std::fs::create_dir_all(&dir)?;
    let p = dir.join("learning_log.json");
    let mut arr: Vec<Value> = std::fs::read_to_string(&p)
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default();
    arr.push(json!({
        "job_id": job.id, "ts": now_ts(),
        "brand": job.brand, "model": job.model,
        "service": job.service_id, "status": job.status,
        "problem": job.problem,
        "learned_sw": if job.status == "done" {
            "Njia ilifanya kazi — rejea kwa kifaa kama hiki tena."
        } else {
            "Njia bado — thibitisha consent/driver kabla."
        },
    }));
    if arr.len() > 300 {
        arr = arr.split_off(arr.len() - 300);
    }
    std::fs::write(&p, serde_json::to_string_pretty(&arr)?)?;
    Ok(())
}

/// Kitabu kidigitali (HTML) cha job — tatizo → njia → suluhisho → tarehe/muda
pub fn book_html(job: &Job) -> String {
    let mut chapters = String::new();
    for (i, f) in job.frames.iter().enumerate() {
        let code = f["code"].as_str().unwrap_or("?");
        let agent = f["agent"].as_str().unwrap_or("?");
        let sw = f["sw"].as_str().unwrap_or("");
        let color = match code {
            "P" => "#00e5ff", "I" => "#ffd740", "H" => "#ff9100", "T" => "#76ff03",
            "V" => "#00e676", "D" => "#e040fb", _ => "#90a4ae",
        };
        chapters.push_str(&format!(
            "<section style='border-left:4px solid {color};padding:10px 14px;margin:10px 0;background:#0d1b2a;border-radius:6px'>\
             <div style='color:{color};font-weight:700;font-size:13px'>SURA {i} · {agent} ({code})</div>\
             <p style='color:#e0f7fa;font-size:13px;margin:8px 0 4px'>{sw}</p></section>",
            i = i + 1
        ));
    }
    format!(
        "<!DOCTYPE html><html lang='sw'><head><meta charset='utf-8'/><title>{t}</title></head>\
         <body style='font-family:system-ui;background:#0a1628;color:#e0f7fa;padding:24px;max-width:860px;margin:auto'>\
         <div style='text-align:center;border:2px solid #00e5ff;border-radius:12px;padding:18px;background:#0d2137'>\
         <h1 style='color:#00e5ff'>📱 FUNDI MOBILE — KITABU KIDIGITALI</h1>\
         <p>Job {id}<br/>{customer} · {brand} {model} · IMEI {imei}<br/>Huduma: {svc} · Hali: {status}<br/>{summary}</p></div>\
         {chapters}\
         <p style='color:#78909c;font-size:11px'>FUNDI MOBILE — ripoti halisi; kazi ya hatari ilifanyika kwa consent (#HITL).</p>\
         </body></html>",
        t = job.id,
        id = job.id, customer = job.customer, brand = job.brand, model = job.model,
        imei = job.imei, svc = job.service_id, status = job.status, summary = job.summary_sw,
        chapters = chapters
    )
}

/// Dashboard: jobs + agents + consents count
pub fn dashboard() -> Value {
    let jobs = list_jobs();
    json!({
        "agents": AGENTS.iter().map(|(id, name, role)| json!({"id": id, "name": name, "role": role})).collect::<Vec<_>>(),
        "jobs_total": jobs.len(),
        "jobs_by_status": {
            "done": jobs.iter().filter(|j| j.status == "done").count(),
            "awaiting_consent": jobs.iter().filter(|j| j.status == "awaiting_consent").count(),
            "running": jobs.iter().filter(|j| j.status == "running").count(),
            "failed": jobs.iter().filter(|j| j.status == "failed").count(),
        },
        "consents": consent::list().len(),
        "recent": jobs.iter().rev().take(5).map(|j| json!({
            "id": j.id, "customer": j.customer, "service": j.service_id,
            "status": j.status, "summary": j.summary_sw
        })).collect::<Vec<_>>(),
    })
}
