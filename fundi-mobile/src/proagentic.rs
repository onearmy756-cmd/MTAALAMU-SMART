//! proagentic.rs — FUNDI PRO AGENTIC: huduma za ofisi zinaendeshwa na agents 10.
//!
//! Mtiririko (kama agentic.rs ya simu):
//!   receptionist → diagnoser → planner → hitl → solver → scribe → learner
//!   + payments::invoice_add (FUNDI PAY halisi) kila session.
//!
//! Sheria:
//!   - Bei zinatoka proservices catalog (data-driven, hakuna namba ngumu).
//!   - Huduma nyeti (data_recovery, hacked_account_recovery) ZINAHITAJI consent
//!     ya mteja (consent.rs, service_id "pro_<id>") — agent inasimamisha (HITL).
//!   - Huduma za remote (email/gov/backup/security) ni guided: agent inatoa
//!     hatua halisi kutoka catalog + kuunda invoice — hakuna uongo wa "nimefanya".
//!   - Kila session ina book (HTML) + learning log.

use crate::consent;
use crate::payments;
use crate::proservices;
use crate::sysvision;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::PathBuf;

/// Huduma za PRO zinazohitaji consent (kazi nyeti — inagusa data/account za mteja)
const CONSENT_REQUIRED: [&str; 2] = ["data_recovery", "hacked_account_recovery"];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProJob {
    pub id: String,
    pub customer: String,
    pub phone: String,
    pub service_id: String,
    pub problem: String,
    pub subs: Vec<String>,
    pub price_tzs: u64,
    pub invoice_id: Option<String>,
    pub consent_id: Option<String>,
    pub status: String, // created | awaiting_consent | running | done | failed
    pub frames: Vec<Value>,
    pub summary_sw: String,
    pub created_ts: u64,
    pub ended_ts: Option<u64>,
}

fn jobs_path() -> PathBuf {
    let base = std::env::var("FUNDI_DATA").unwrap_or_else(|_| "../data".into());
    PathBuf::from(base).join("mobile/pro_jobs.json")
}

pub fn save_job(job: &ProJob) -> Result<()> {
    let p = jobs_path();
    if let Some(d) = p.parent() {
        std::fs::create_dir_all(d)?;
    }
    let mut arr: Vec<ProJob> = std::fs::read_to_string(&p)
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default();
    arr.retain(|x| x.id != job.id);
    arr.push(job.clone());
    std::fs::write(&p, serde_json::to_string_pretty(&arr)?)?;
    Ok(())
}

pub fn list_jobs() -> Vec<ProJob> {
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

pub struct ProRequest {
    pub customer: String,
    pub phone: String,
    pub service_id: String,
    pub problem: String,
    pub subs: Vec<String>,
}

/// Agentic run kamili ya huduma ya FUNDI PRO.
pub fn run(req: &ProRequest) -> Result<ProJob> {
    let job_id = format!(
        "PRO-{}-{}",
        chrono::Local::now().format("%Y%m%d%H%M%S"),
        req.service_id
    );
    let svc = proservices::get(&req.service_id)?;
    let mut frames: Vec<Value> = Vec::new();

    // 1. RECEPTIONIST — intake + bei (halisi kutoka catalog)
    let (price, price_note) = if req.subs.is_empty() {
        (svc.price_min_tzs, format!("range TZS {} - {}", svc.price_min_tzs, svc.price_max_tzs))
    } else {
        // kikokotoo halisi cha subs
        let mut total = 0u64;
        let mut names = Vec::new();
        for s in &req.subs {
            let p = find_sub_price(&svc, s)?;
            total += p;
            names.push(format!("{s}=TZS {p}"));
        }
        (total, names.join(", "))
    };
    frames.push(frame("P", "receptionist",
        &format!("Karibu {}! Huduma: {} ({}). Bei: TZS {} ({}). Tatizo: {}.",
                 req.customer, svc.name_sw,
                 if svc.mode == "remote" { "100% remote" } else { "sehemu remote" },
                 price, price_note, req.problem),
        json!({ "service": req.service_id, "price_tzs": price, "mode": svc.mode })));

    // 2. VISION — scan HALISI ya PC ya mteja/fundi (kama Muono wa Fundi Deploy)
    let scan = sysvision::scan();
    let findings: Vec<String> = scan["findings"]
        .as_array()
        .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();
    frames.push(frame("P", "vision",
        if findings.is_empty() {
            "Nimechunguza PC: OS, RAM, disks, network — kila kitu kiko sawa (hakuna findings)."
        } else {
            "Nimechunguza PC na NIMEGUNUA matatizo halisi (tazama evidence)."
        },
        scan.clone()));

    // 3. DIAGNOSER — linganisha tatizo na matatizo yanayojulikana + findings halisi
    let matched = if svc.problems.is_empty() {
        Vec::new()
    } else {
        let pl = req.problem.to_lowercase();
        svc.problems
            .iter()
            .enumerate()
            .filter(|(_, p)| {
                p.to_lowercase()
                    .split_whitespace()
                    .any(|w| w.len() > 3 && pl.contains(&w.to_lowercase()))
            })
            .map(|(i, p)| format!("#{} {}", i + 1, p))
            .collect::<Vec<_>>()
    };
    let mut diag_note = if matched.is_empty() {
        "Tatizo halingani na orodha ya kawaida — nitatumia hatua za jumla za huduma hii.".to_string()
    } else {
        "Tatizo linalingana na matatizo yanayojulikana — nimepanga hatua maalum.".to_string()
    };
    if !findings.is_empty() {
        diag_note.push_str(&format!(" Vision imeongeza findings {}.", findings.len()));
    }
    frames.push(frame("I", "diagnoser",
        &diag_note,
        json!({ "matched": matched, "problem": req.problem, "vision_findings": findings })));

    // renumber steps zilizobaki (planner iko 4 sasa, hitl 5, solver 6, payments 7, scribe 8, learner 9)

    // 4. PLANNER — hatua halisi kutoka catalog (+ findings za vision kama ziada)
    let mut plan_steps: Vec<String> = svc.steps_sw.clone();
    for f in &findings {
        plan_steps.push(format!("[VISION] {f}"));
    }
    frames.push(frame("I", "planner",
        &format!("Mpango wa kazi (hatua {}):", plan_steps.len()),
        json!({ "steps": plan_steps })));

    // 4. HITL — consent kwa huduma nyeti
    let needs_consent = CONSENT_REQUIRED.contains(&req.service_id.as_str());
    let consent_id = if needs_consent {
        match consent::verify(&format!("pro-{}", req.phone), &format!("pro_{}", req.service_id)) {
            Ok(c) => {
                frames.push(frame("H", "hitl",
                    &format!("Consent ipo: {} ({}). Kazi nyeti — ninaruhusiwa kuendelea.", c.consent_id, c.customer_name),
                    json!({ "consent": c.consent_id })));
                Some(c.consent_id)
            }
            Err(_) => {
                frames.push(frame("H", "hitl",
                    "KAZI IMESIMAMISHWA: Huduma hii inagusa data/account zako. Mteja LAZIMA asaini consent kwanza: fundi-mobile consent add --service pro_DATA_RECOVERY ...",
                    json!({ "blocked": true })));
                None
            }
        }
    } else {
        frames.push(frame("H", "hitl",
            "Huduma hii ni guided-remote (hakuna kinachoharibiwa) — consent ya karatasi si lazima; kumbukumbu ya session inatosha.",
            json!({ "guided": true })));
        None
    };

    let blocked = needs_consent && consent_id.is_none();

    // 5. SOLVER — guided steps halisi (au imesimama kwa HITL)
    if blocked {
        frames.push(frame("S", "solver",
            "Nimesimama (HITL). Baada ya consent, endesha tena — nitatokapo hatua zote.",
            json!({})));
    } else {
        frames.push(frame("S", "solver",
            "Hatua za kutatua (fundi anazoongoza kwa screen-sharing/simu):",
            json!({ "steps": plan_steps, "tools": svc.tools })));
    }

    // 6. PAYMENTS — invoice halisi (FUNDI PAY)
    let invoice_id = if !blocked && price > 0 {
        let item = payments::InvoiceItem {
            desc: format!("{} — {}", svc.name_sw, if req.subs.is_empty() { req.problem.clone() } else { req.subs.join(", ") }),
            qty: 1,
            price,
        };
        match payments::invoice_add(&req.customer, &req.phone, None, vec![item]) {
            Ok(inv) => {
                frames.push(frame("D", "scribe",
                    &format!("Invoice {} imeundwa: TZS {} — malipo: fundi-mobile pay checkout --invoice {} --provider mpesa",
                             inv.id, inv.total, inv.id),
                    json!({ "invoice": inv.id, "total": inv.total })));
                Some(inv.id)
            }
            Err(e) => {
                frames.push(frame("D", "scribe",
                    &format!("Invoice imeshindikana: {e} (session inaendelea)"),
                    json!({ "error": e.to_string() })));
                None
            }
        }
    } else {
        None
    };

    // 7. SCRIBE — muhtasari
    let summary = if blocked {
        "Imesimama kwa HITL — consent ya mteja inahitajika kabla ya kazi.".to_string()
    } else {
        format!(
            "{}: hatua {} zimepangwa; invoice {}; fundi anaendelea kwa mwongozo ({}).",
            svc.name_sw,
            svc.steps_sw.len(),
            invoice_id.as_deref().unwrap_or("—"),
            if svc.mode == "remote" { "remote" } else { "onsite/sehemu" }
        )
    };
    frames.push(frame("D", "scribe", &summary, json!({ "summary": summary })));

    // 8. LEARNER — learning log halisi
    learn(&json!({
        "ts": now_ts(),
        "service": req.service_id,
        "problem": req.problem,
        "matched": matched,
        "blocked": blocked,
        "price_tzs": price,
    }));
    frames.push(frame("D", "learner", "Nimehifadhi kesi hii kwenye learning log (pro_learning.json).", json!({})));

    let status = if blocked { "awaiting_consent".to_string() } else { "done".to_string() };
    let job = ProJob {
        id: job_id,
        customer: req.customer.clone(),
        phone: req.phone.clone(),
        service_id: req.service_id.clone(),
        problem: req.problem.clone(),
        subs: req.subs.clone(),
        price_tzs: price,
        invoice_id,
        consent_id,
        status,
        frames,
        summary_sw: summary,
        created_ts: now_ts(),
        ended_ts: Some(now_ts()),
    };
    save_job(&job)?;
    Ok(job)
}

fn find_sub_price(svc: &proservices::ProService, sub: &str) -> Result<u64> {
    let eq = |id: &String| id == sub || id.eq_ignore_ascii_case(sub);
    for x in svc.systems.iter().filter(|x| eq(&x.id)) { return Ok(x.price_tzs); }
    for x in svc.targets.iter().filter(|x| eq(&x.id)) { return Ok(x.price_tzs); }
    for x in svc.cases.iter().filter(|x| eq(&x.id)) { return Ok(x.price_tzs); }
    for x in svc.options.iter().filter(|x| eq(&x.id)) { return Ok(x.price_tzs); }
    for x in svc.items.iter().filter(|x| eq(&x.id)) { return Ok(x.price_tzs); }
    anyhow::bail!("'{sub}' haipo ndani ya {}", svc.name_sw)
}

fn learn(entry: &Value) {
    let base = std::env::var("FUNDI_DATA").unwrap_or_else(|_| "../data".into());
    let p = PathBuf::from(base).join("mobile/pro_learning.json");
    let mut arr: Vec<Value> = std::fs::read_to_string(&p)
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default();
    arr.push(entry.clone());
    let _ = std::fs::create_dir_all(p.parent().unwrap_or(std::path::Path::new(".")));
    let _ = std::fs::write(&p, serde_json::to_string_pretty(&arr).unwrap_or_default());
}

/// Kitabu cha session (HTML) — kwa mteja/fundi
pub fn book_html(job: &ProJob) -> String {
    let mut h = String::from(
        "<!DOCTYPE html><html><head><meta charset='utf-8'><title>FUNDI PRO — Kitabu</title>\
         <style>body{font-family:Segoe UI,Arial;background:#0f172a;color:#e2e8f0;max-width:820px;margin:32px auto;padding:0 16px}\
         .f{background:#1e293b;border-radius:10px;padding:14px 18px;margin:10px 0;border-left:4px solid #22d3ee}\
         .a{color:#22d3ee;font-weight:700;font-size:12px;text-transform:uppercase}\
         .sw{margin:4px 0 0}small{color:#94a3b8}\
         .b{background:#134e4a;border-radius:10px;padding:16px;margin-bottom:16px}</style></head><body>",
    );
    h.push_str(&format!(
        "<div class='b'><h1 style='color:#22d3ee;margin:0'>🩺 FUNDI PRO — Kitabu cha {}</h1>\
         <p>{} · {} · {} · TZS {} · {}</p><p><small>Invoice: {} · Consent: {}</small></p></div>",
        job.id, job.service_id, job.customer, job.phone, job.price_tzs, job.status,
        job.invoice_id.as_deref().unwrap_or("—"),
        job.consent_id.as_deref().unwrap_or("—")
    ));
    for f in &job.frames {
        h.push_str(&format!(
            "<div class='f'><div class='a'>[{}] {}</div><div class='sw'>{}</div></div>",
            f["code"].as_str().unwrap_or("?"),
            f["agent"].as_str().unwrap_or("?"),
            f["sw"].as_str().unwrap_or("")
        ));
    }
    h.push_str("</body></html>");
    h
}

/// Dashboard ndogo ya PRO (JSON)
pub fn dashboard() -> Value {
    let jobs = list_jobs();
    json!({
        "jobs": jobs.len(),
        "done": jobs.iter().filter(|j| j.status == "done").count(),
        "awaiting_consent": jobs.iter().filter(|j| j.status == "awaiting_consent").count(),
        "revenue_tzs": jobs.iter().filter(|j| j.status == "done").map(|j| j.price_tzs).sum::<u64>(),
    })
}
