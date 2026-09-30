//! remote.rs — REMOTE OS INSTALL: app moja ya kwake (mteja ↔ mtaalamu).
//!
//! Mtiririko:
//!   1. Mteja/kampuni anaomba ndani ya app → session (code ya kipekee, mf. RMT-XXXX-XXXX)
//!   2. Mtaalamu anaona session kwenye dashboard yake (polling kila sekunde 3)
//!   3. Mtaalamu anaidhinisha (HITL) → agent (pipeline ya PXE/imaging) inasakinisha OS
//!   4. BAADA ya OS: agent inasakinisha APPS ZOTE MUHIMU (bundle kutoka app_bundles.json)
//!      — winget (Windows) / apt (Debian/Ubuntu) halisi, moja kwa moja
//!   5. Mteja anaona progress moja kwa moja (channels store + /osinstall/status)
//!
//! KANUNI: Hakuna install bila idhini ya mtaalamu (HITL daima).

use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone, Debug, serde::Serialize)]
pub struct Session {
    pub code: String,          // RMT-XXXX-XXXX
    pub customer: String,
    pub company: Option<String>,
    pub pc: String,
    pub os: String,
    pub bundle: String,
    pub status: String, // requested | approved | installing_os | installing_apps | done | cancelled | failed
    pub progress: u32,
    pub message: String,
    pub created: String,
    pub approved_by: Option<String>,
    pub log: Vec<Value>,
}

#[derive(Default)]
pub struct Store {
    sessions: RwLock<Vec<Session>>,
    seq: AtomicU64,
}

impl Store {
    pub fn new() -> Self {
        Self::default()
    }

    fn gen_code(&self) -> String {
        let n = self.seq.fetch_add(1, Ordering::Relaxed);
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.subsec_nanos() ^ (d.as_secs() as u32))
            .unwrap_or(0) ^ (n as u32);
        let chars = b"ABCDEFGHJKMNPQRSTUVWXYZ23456789"; // hakuna I/L/O/0/1 (mteja asisumbuke)
        let c = &chars;
        format!(
            "RMT-{}{}{}{}-{}{}{}{}",
            c[seed as usize % c.len()] as char,
            c[(seed >> 4) as usize % c.len()] as char,
            c[(seed >> 8) as usize % c.len()] as char,
            c[(seed >> 12) as usize % c.len()] as char,
            c[(seed >> 16) as usize % c.len()] as char,
            c[(seed >> 20) as usize % c.len()] as char,
            c[(seed >> 24) as usize % c.len()] as char,
            c[(seed >> 28) as usize % c.len()] as char,
        )
    }

    pub async fn create(&self, customer: String, company: Option<String>, pc: String, os: String, bundle: String) -> Session {
        let code = self.gen_code();
        let s = Session {
            code: code.clone(),
            customer,
            company,
            pc,
            os,
            bundle,
            status: "requested".into(),
            progress: 0,
            message: "Imeombwa — inasubiri mtaalamu".into(),
            created: chrono::Local::now().to_rfc3339(),
            approved_by: None,
            log: vec![json!({ "ts": chrono::Local::now().to_rfc3339(), "msg": "Session imeundwa" })],
        };
        self.sessions.write().await.push(s.clone());
        s
    }

    pub async fn list(&self) -> Vec<Session> {
        self.sessions.read().await.clone()
    }

    pub async fn get(&self, code: &str) -> Option<Session> {
        self.sessions.read().await.iter().find(|s| s.code.eq_ignore_ascii_case(code)).cloned()
    }

    pub async fn update<F: FnOnce(&mut Session)>(&self, code: &str, f: F) -> bool {
        let mut all = self.sessions.write().await;
        match all.iter_mut().find(|s| s.code.eq_ignore_ascii_case(code)) {
            Some(s) => {
                f(s);
                true
            }
            None => false,
        }
    }
}

// ---------- APP BUNDLES (data-driven) ----------

fn bundles_path() -> PathBuf {
    // fundi-deploy inaendeshwa kutoka server/agent; data iko ../../../data
    let cands = ["../../../data/deploy/app_bundles.json", "../../data/deploy/app_bundles.json", "data/deploy/app_bundles.json"];
    for c in cands {
        let p = PathBuf::from(c);
        if p.exists() {
            return p;
        }
    }
    PathBuf::from("../../../data/deploy/app_bundles.json")
}

pub fn load_bundles() -> Vec<Value> {
    std::fs::read_to_string(bundles_path())
        .ok()
        .and_then(|t| serde_json::from_str::<Value>(&t).ok())
        .and_then(|v| v["bundles"].as_array().cloned())
        .unwrap_or_default()
}

pub fn bundle_list() -> Value {
    let bs = load_bundles();
    json!({
        "bundles": bs.iter().map(|b| json!({
            "id": b["id"], "name_sw": b["name_sw"], "description": b["description"],
            "apps": b["apps"].as_array().map(|a| a.len()).unwrap_or(0),
        })).collect::<Vec<_>>(),
    })
}

// ---------- APP INSTALL HALISI (winget/apt) ----------

fn run(cmd: &str, args: &[&str], timeout_secs: u64) -> (bool, String) {
    let child = std::process::Command::new(cmd)
        .args(args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn();
    match child {
        Ok(mut c) => {
            let start = std::time::Instant::now();
            loop {
                match c.try_wait() {
                    Ok(Some(status)) => {
                        let mut out = String::new();
                        if let Some(mut o) = c.stdout.take() {
                            use std::io::Read;
                            let _ = o.read_to_string(&mut out);
                        }
                        return (status.success(), out.chars().take(400).collect());
                    }
                    Ok(None) => {
                        if start.elapsed() > std::time::Duration::from_secs(timeout_secs) {
                            let _ = c.kill();
                            return (false, "timeout".into());
                        }
                        std::thread::sleep(std::time::Duration::from_millis(300));
                    }
                    Err(e) => return (false, e.to_string()),
                }
            }
        }
        Err(e) => (false, e.to_string()),
    }
}

/// Install app moja HALISI (winget Windows / apt Linux). Inarudisha (ok, output).
pub fn install_one(app: &Value) -> (bool, String) {
    if let Some(w) = app["winget"].as_str() {
        run("winget", &["install", "--id", w, "--silent", "--accept-package-agreements", "--accept-source-agreements", "--disable-interactivity"], 600)
    } else if let Some(a) = app["apt"].as_str() {
        run("sudo", &["apt-get", "install", "-y", a], 600)
    } else {
        (false, "hakuna winget/apt kwenye bundle".into())
    }
}

/// Endesha bundle nzima kwenye session (inaitwa na background task baada ya OS)
pub async fn run_bundle(store: Arc<Store>, code: String, bundle_id: String) {
    let bundle = load_bundles().into_iter().find(|b| b["id"].as_str() == Some(bundle_id.as_str()));
    let apps: Vec<Value> = bundle
        .and_then(|b| b["apps"].as_array().cloned())
        .unwrap_or_default();

    store.update(&code, |s| {
        s.status = "installing_apps".into();
        s.message = format!("Inasakinisha apps {} (bundle {bundle_id})", apps.len());
    }).await;

    let mut results = Vec::new();
    let total = apps.len().max(1);
    for (i, app) in apps.iter().enumerate() {
        let name = app["name"].as_str().unwrap_or("?").to_string();
        store.update(&code, |s| {
            s.progress = ((i as u32) * 100 / total as u32).max(1);
            s.message = format!("App {}/{}: {name}", i + 1, total);
        }).await;

        // App ya "verify-only" (defender) — hakuna winget/apt
        let ok = if app["winget"].as_str().is_none() && app["apt"].as_str().is_none() {
            true // verification apps: zimepita kwa default (defender config ni ya OS)
        } else {
            let (ok, out) = install_one(app);
            if !ok {
                store.update(&code, |s| {
                    s.log.push(json!({ "ts": chrono::Local::now().to_rfc3339(), "app": name, "ok": false, "out": out }));
                }).await;
            }
            ok
        };
        results.push(json!({ "app": name, "ok": ok }));
    }

    let failed = results.iter().filter(|r| !r["ok"].as_bool().unwrap_or(false)).count();
    // App CRITICAL ikishindikana → session ni "failed" (mteja arudiwe/mtaalamu ajue);
    // zisizo-critical zikishindikana → "done" na onyo kwenye log.
    let crit_failed = apps.iter().zip(results.iter())
        .any(|(a, r)| a["critical"].as_bool().unwrap_or(false) && !r["ok"].as_bool().unwrap_or(false));
    store.update(&code, |s| {
        s.progress = 100;
        s.status = if crit_failed { "failed".into() } else { "done".into() };
        s.message = if failed == 0 {
            "✅ OS + apps zote zimesakinishwa".into()
        } else if crit_failed {
            format!("❌ Apps {failed} zimeshindikana, zikiwemo CRITICAL — angalia log kisha rusha tena")
        } else {
            format!("✅ Imekamilika ({failed} zisizo-critical zimeshindikana — angalia log)")
        };
        s.log.push(json!({ "ts": chrono::Local::now().to_rfc3339(), "results": results }));
    }).await;
}
