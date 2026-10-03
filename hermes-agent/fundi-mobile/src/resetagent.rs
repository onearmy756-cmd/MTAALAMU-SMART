//! resetagent.rs — PASSWORD RESET AGENTIC (njia HALALI tu; consent LAZIMA — HITL).
//!
//! KANUNI (Sheria ya Makosa ya Jinai Tanzania, Kifungu 267):
//!   1. HAKUNA bypass/exploit. Njia zote ni rasmi (recovery, admin, media ya install).
//!   2. Consent LAZIMA kabla ya hatua yoyote: service "password_reset_<target>".
//!      Mteja LAZIMA athibitishe UMILIKI (ID + kifaa mbele ya fundi).
//!   3. Kila hatua inarekodiwa (audit log) — reset agent ni mwongozo wa fundi,
//!      si chombo cha kuvunja simu/kompyuta ya mtu mwingine.
//!
//! Targets: windows_local | windows_ms | android | email | social
//! Mtiririko: receptionist → vision (thibitisha target HALISI) → hitl (consent)
//!            → solver (hatua rasmi za target) → verifier → scribe → learn.

use crate::consent;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResetJob {
    pub id: String,
    pub customer: String,
    pub target: String,      // windows_local | windows_ms | android | email | social
    pub brand: String,       // samsung/xiaomi/gmail/facebook...
    pub account: String,     // jina/user/email (IMEI kwa android)
    pub consent_id: Option<String>,
    pub status: String,      // done | awaiting_consent | guided
    pub frames: Vec<Value>,
    pub summary_sw: String,
    pub created_ts: u64,
}

fn jobs_path() -> PathBuf {
    let base = std::env::var("FUNDI_DATA").unwrap_or_else(|_| "../data".into());
    PathBuf::from(base).join("mobile/reset_jobs.json")
}

pub fn list_jobs() -> Vec<ResetJob> {
    std::fs::read_to_string(jobs_path())
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default()
}

fn save_job(job: &ResetJob) {
    let p = jobs_path();
    let _ = std::fs::create_dir_all(p.parent().unwrap_or(std::path::Path::new(".")));
    let mut arr: Vec<ResetJob> = std::fs::read_to_string(&p)
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default();
    arr.retain(|x| x.id != job.id);
    arr.push(job.clone());
    let _ = std::fs::write(&p, serde_json::to_string_pretty(&arr).unwrap_or_default());
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

pub struct ResetRequest {
    pub customer: String,
    pub target: String,   // windows_local | windows_ms | android | email | social
    pub brand: String,
    pub account: String,  // username/email/IMEI
}

/// AGENTIC reset run — kila target na hatua zake rasmi.
pub fn run(req: &ResetRequest) -> Result<ResetJob> {
    let job_id = format!("RST-{}", chrono::Local::now().format("%Y%m%d%H%M%S"));
    let mut frames: Vec<Value> = Vec::new();
    let service_id = format!("password_reset_{}", req.target);

    // 1. RECEPTIONIST
    frames.push(frame("P", "receptionist",
        &format!("Ombi la password reset: target={} brand={} account={}. Njia ni RASMI tu — hakuna bypass.",
                 req.target, req.brand, req.account),
        json!({ "target": req.target, "brand": req.brand })));

    // 2. VISION — thibitisha target HALISI
    let vision_ev = match req.target.as_str() {
        "windows_local" | "windows_ms" => crate::sysvision::scan(),
        "android" => json!({ "adb": crate::devices::adb_devices().map(|d| d.len()).unwrap_or(0) }),
        _ => json!({ "target": "remote-account", "note": "hakuna local scan — njia ni za web recovery" }),
    };
    frames.push(frame("P", "vision",
        "Nimechunguza kifaa/target (evidence chini) — sasa HITL: consent ya MMILIKI ni LAZIMA.",
        vision_ev.clone()));

    // 3. HITL — consent LAZIMA
    let consent_id = match consent::verify(&req.account, &service_id) {
        Ok(c) => {
            frames.push(frame("H", "hitl",
                &format!("Consent ipo: {} — {} amethibitisha umiliki (ID: {}). Njia rasmi zinaruhusiwa.",
                         c.consent_id, c.customer_name, c.id_number.as_deref().unwrap_or("—")),
                json!({ "consent": c.consent_id })));
            Some(c.consent_id)
        }
        Err(_) => {
            frames.push(frame("H", "hitl",
                "KAZI IMESIMAMISHWA: Hakuna consent. Mteja LAZIMA: (1) awe mbele ya fundi, (2) athibitishe umiliki (ID),\
                 (3) asaini fomu. Kisha: fundi-mobile consent add --service PASSWORD_RESET_TARGET --imei <account> ...",
                json!({ "blocked": true })));
            None
        }
    };
    let blocked = consent_id.is_none();

    // 4. SOLVER — hatua rasmi kwa kila target
    let steps: Vec<String> = match req.target.as_str() {
        "windows_local" => vec![
            "A. Kama kuna password reset disk / security questions → tumia (Settings→Sign-in)".into(),
            "B. Admin mwingine kwenye PC: aingie → Control Panel→User Accounts→Badilisha password ya mteja".into(),
            "C. Hakuna admin: Windows install media/USB → Repair → Troubleshoot → Command (utilman trick rasmi ya kurudisha access ya MMILIKI)".into(),
            "D. Mwisho: tengeneza password reset disk + security questions ili isiatoke tena".into(),
        ],
        "windows_ms" => vec![
            "A. account.live.com/password/reset kutoka kifaa chochote".into(),
            "B. Thibitisha kwa recovery email/phone/MS Authenticator".into(),
            "C. Kama 2FA info imepotea: account.live.com/acsr (form ya uthibitisho — siku 1-3)".into(),
            "D. Baada ya reset: login PC na password mpya (inahitaji internet mara ya kwanza)".into(),
        ],
        "android" => vec![
            "A. Samsung: findmymobile.samsung.com → Unlock (rasmi, Android 11+)".into(),
            "B. Xiaomi: i.mi.com → Find device → Unlock. Huawei: cloud.huawei.com".into(),
            "C. Google Find My Device: google.com/android/find → Lock → PIN mpya → fungua kwa PIN hiyo".into(),
            "D. Hakuna yoyote: Recovery mode → factory reset (data INAENDA — consent inasema hivyo) → FRP na MMILIKI (recover frp-aftermath)".into(),
        ],
        "email" => vec![
            "A. Google: accounts.google.com/signin/recovery → jibu maswali (password ya mwisho, recovery phone/email)".into(),
            "B. Microsoft: account.live.com/password/reset".into(),
            "C. Yahoo: login.yahoo.com/forgot".into(),
            "D. Kama recovery info imepotea: support form ya provider (siku 3-5) — ukitume kutoka kifaa/location ulizotumia awali inaongeza mafanikio".into(),
        ],
        "social" => vec![
            "A. Facebook: facebook.com/hacked → 'My account is compromised' (au login/identify + trusted contacts)".into(),
            "B. Instagram: 'Get help logging in' → video selfie verification".into(),
            "C. TikTok: app report / support form; WhatsApp: ingia kwa namba + SMS code (inamtoa aliyeichukua AUTOMATIC)".into(),
            "D. Baada ya kupata: badilisha password, washa 2FA, ondoa devices/sessions za kigeni".into(),
        ],
        other => vec![format!("Target '{other}' haijulikani. Chagua: windows_local | windows_ms | android | email | social")],
    };
    frames.push(frame("S", "solver",
        if blocked { "Imesimama (HITL) — hatua hizi ndizo zitakazotekelezwa BAADA ya consent:" }
        else { "Hatua rasmi (fundi anazoongoza na mteja mbele yake):" },
        json!({ "steps": steps })));

    // 5. VERIFIER
    frames.push(frame("V", "verifier",
        "Baada ya kazi: mteja anatest login MBELE ya fundi; kisha tunawasha 2FA + password manager + recovery options mpya.",
        json!({ "post_check": ["test login", "2FA ON", "recovery updated"] })));

    // 6. SCRIBE + LEARN
    let status = if blocked { "awaiting_consent" } else { "done" };
    let summary = if blocked {
        "Imesimama kwa HITL — consent ya mmiliki inahitajika kabla ya hatua zozote.".to_string()
    } else {
        format!("Reset guided: target={} hatua {} (rasmi tu).", req.target, steps.len())
    };
    frames.push(frame("D", "scribe", &summary, json!({ "summary": summary })));
    learn(&json!({ "ts": now_ts(), "target": req.target, "brand": req.brand, "blocked": blocked }));

    let job = ResetJob {
        id: job_id,
        customer: req.customer.clone(),
        target: req.target.clone(),
        brand: req.brand.clone(),
        account: req.account.clone(),
        consent_id,
        status: status.to_string(),
        frames,
        summary_sw: summary,
        created_ts: now_ts(),
    };
    save_job(&job);
    Ok(job)
}

fn learn(entry: &Value) {
    let base = std::env::var("FUNDI_DATA").unwrap_or_else(|_| "../data".into());
    let p = PathBuf::from(base).join("mobile/reset_learning.json");
    let mut arr: Vec<Value> = std::fs::read_to_string(&p)
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default();
    arr.push(entry.clone());
    let _ = std::fs::create_dir_all(p.parent().unwrap_or(std::path::Path::new(".")));
    let _ = std::fs::write(&p, serde_json::to_string_pretty(&arr).unwrap_or_default());
}

/// Kitabu cha session (HTML)
pub fn book_html(job: &ResetJob) -> String {
    let mut h = String::from(
        "<!DOCTYPE html><html><head><meta charset='utf-8'><title>FUNDI RESET — Kitabu</title>\
         <style>body{font-family:Segoe UI,Arial;background:#0f172a;color:#e2e8f0;max-width:820px;margin:32px auto;padding:0 16px}\
         .f{background:#1e293b;border-radius:10px;padding:14px 18px;margin:10px 0;border-left:4px solid #22d3ee}\
         .a{color:#22d3ee;font-weight:700;font-size:12px;text-transform:uppercase}\
         .b{background:#134e4a;border-radius:10px;padding:16px;margin-bottom:16px}</style></head><body>",
    );
    h.push_str(&format!(
        "<div class='b'><h1 style='color:#22d3ee;margin:0'>🔑 FUNDI RESET AGENTIC — {}</h1>\
         <p>{} · target: {} · {} · {}</p><p><small>Consent: {}</small></p></div>",
        job.id, job.customer, job.target, job.account, job.status,
        job.consent_id.as_deref().unwrap_or("—")
    ));
    for f in &job.frames {
        h.push_str(&format!(
            "<div class='f'><div class='a'>[{}] {}</div><div>{}</div></div>",
            f["code"].as_str().unwrap_or("?"),
            f["agent"].as_str().unwrap_or("?"),
            f["sw"].as_str().unwrap_or("")
        ));
    }
    h.push_str("</body></html>");
    h
}
