//! Fundi Deploy Agent v3 — P2 kamili + P3 cloud + P4 WIREGUARD VPN + MAJINA KIOTOMATIKI
//!
//! Pipeline: plan (AI+rules) → HITL → backup (halisi) → WOL → PXE → [multicast] → install → report
//! Msimamizi: approve/cancel + dashboard /ui
//! Cloud: tenants + heartbeats + outbox offline-first
//! VPN (P4): kazi ZOTE za mbali zinaenda kupitia WireGuard (wg0) — /api/vpn/*
//! Majina: duplicates (hr, hr 1, hr 2…) kulingana na mpangilio wa IP — /computers

mod admin;
mod ai;
mod ai_config;
mod auth;
mod backup;
mod brain;
mod bundles;
mod cloud;
mod company;
mod discover;
mod chat;
mod daily;
mod hardware;
mod images;
mod lan;
mod language;
mod license;
mod mode;
mod multicast;
mod namer;
mod orchestrator;
mod osselect;
mod pipeline;
mod pricing;
mod reacon;
mod remote;
mod report;
mod remote_view;
mod updates;
mod vector;
mod server_setup;
mod tools;
mod tools_internal;
mod billing;
mod credit;
mod netmgmt;
mod vpn;
mod pfsense;
mod secops;
mod toolkit;
mod wol;

use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use multicast::SharedAcks;
use namer::{assign_names, NamedHost};
use pipeline::{DeployRequest, Job};
use serde_json::json;
use sqlx::SqlitePool;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    db: SqlitePool,
    jobs: Arc<RwLock<Vec<Job>>>,
    acks: SharedAcks,
    orch: Arc<orchestrator::Orchestrator>,
    remote: Arc<remote::Store>,
    brain: Arc<brain::Brain>,
    pfsense: Arc<pfsense::PfClient>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    std::fs::create_dir_all("/data").ok();
    std::fs::create_dir_all("/var/lib/tftpboot/pxelinux.cfg").ok();
    std::fs::create_dir_all(vpn::dir()).ok();

    let db = SqlitePool::connect("sqlite:///data/fundi.db?mode=rwc").await?;
    for ddl in [
        r#"CREATE TABLE IF NOT EXISTS jobs (
            id TEXT PRIMARY KEY,
            device_mac TEXT,
            device_name TEXT,
            os_type TEXT,
            os_reason TEXT,
            status TEXT,
            stage TEXT,
            progress INTEGER,
            message TEXT,
            backup_info TEXT
        )"#,
        r#"CREATE TABLE IF NOT EXISTS agents (
            id TEXT PRIMARY KEY,
            name TEXT,
            role TEXT,
            status TEXT,
            last_seen TEXT
        )"#,
    ] {
        sqlx::query(ddl).execute(&db).await?;
    }
    // Columns mpya kwenye DB ya zamani (migration ndogo)
    for col in ["os_reason TEXT", "backup_info TEXT"] {
        let _ = sqlx::query(&format!("ALTER TABLE jobs ADD COLUMN {col}")).execute(&db).await;
    }
    cloud::init_tables(&db).await;
    vpn::init_tables(&db).await;
    bundles::init_tables(&db).await;
    brain::init_tables(&db).await;
    auth::init_tables(&db).await;
    admin::init_tables(&db).await;
    license::init_tables(&db).await;
    mode::init_tables(&db).await;
    daily::init_tables(&db).await;
    updates::init_tables(&db).await;
    language::init_tables(&db).await;
    company::init_tables(&db).await;
    secops::init_tables(&db).await;
    toolkit::init_tables(&db).await;

    // NEURALIS BRAIN (H5b): SQLite + LanceDB-compatible vector store (cosine semantic search)
    let brain = Arc::new(brain::Brain::new(db.clone(), "/data"));
    ai_config::init_tables(&db).await;
    // PFSENSE API (H5b): client halisi — bila env, kazi za firewall zinarudisha error ya configuration
    let pfsense = Arc::new(pfsense::PfClient::from_env());

    // Seed agents 10 (agentic vision)
    let agents: [(&str, &str); 10] = [
        ("receptionist", "Mpokeaji"),
        ("vision", "Muono"),
        ("diagnoser", "Mgunduzi"),
        ("planner", "Mpangaji"),
        ("solver", "Mtatuzi"),
        ("tester", "Mjaribu"),
        ("verifier", "Mthibitishaji"),
        ("scribe", "Mwandishi"),
        ("reporter", "Mripoti"),
        ("learner", "Mwanafunzi"),
    ];
    for (id, name) in agents {
        let _ = sqlx::query(
            "INSERT OR IGNORE INTO agents (id, name, role, status, last_seen) VALUES (?,?,?,?,?)",
        )
        .bind(id)
        .bind(name)
        .bind("pipeline")
        .bind("ready")
        .bind(chrono::Local::now().to_rfc3339())
        .execute(&db)
        .await;
    }

    let acks: SharedAcks = Arc::new(multicast::AckBoard::default());
    let state = AppState {
        db: db.clone(),
        jobs: Arc::new(RwLock::new(Vec::new())),
        acks: acks.clone(),
        orch: Arc::new(orchestrator::Orchestrator::new(100)),
        remote: Arc::new(remote::Store::new()),
            brain,
            pfsense,
    };

    // Load jobs za zamani kutoka DB
    let old: Vec<(String, String, String, String, String, String, i64, String)> = sqlx::query_as(
        "SELECT id, device_mac, device_name, os_type, status, stage, progress, message FROM jobs",
    )
    .fetch_all(&db)
    .await
    .unwrap_or_default();
    for (id, mac, name, os, status, stage, prog, msg) in old {
        state.jobs.write().await.push(Job {
            id,
            device_mac: mac,
            device_name: name,
            os_type: os,
            status,
            stage,
            progress: prog as u32,
            message: msg,
            needs_approval: false,
            image: None,
            multicast: false,
        });
    }

    cloud::start_sync_loop().await;

    // WG0.CONF: regen kila boot (peers DB → conf halisi); tunnel haianzi yenyewe (HITL: /api/vpn/up)
    match vpn::write_server_conf(&db).await {
        Ok(p) => tracing::info!("vpn: wg conf tayari: {} (up: /api/vpn/up)", p.display()),
        Err(e) => tracing::warn!("vpn: server conf imeshindikana: {e}"),
    }

    let ui = ServeDir::new("static").append_index_html_on_directories(true);

    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        // Deploy
        .route("/deploy", post(deploy))
        .route("/deploy/auto", post(deploy_auto))
        .route("/jobs", get(list_jobs))
        .route("/jobs/:id", get(get_job))
        .route("/jobs/:id/approve", post(approve_job))
        .route("/jobs/:id/cancel", post(cancel_job))
        // Discovery + images
        .route("/computers", get(computers))
        .route("/computers/discover", get(computers))
        .route("/images", get(list_images))
        .route("/images/:name", get(get_image))
        // OS selection (AI + rules)
        .route("/os/select", post(os_select))
        .route("/os/profiles", get(os_profiles))
        // Backup
        .route("/backups", get(list_backups))
        // Ripoti ya PDF (logo ya FUNDI — kama fundi-mobile)
        .route("/report/pdf", get(report_pdf))
        .route("/report/meta", get(report_meta))
        // FUNDI MAP (hardware scan halisi)
        .route("/api/map/scan", get(map_scan))
        // MULTI-AGENT orchestrator
        .route("/api/orch/launch", post(orch_launch))
        .route("/api/orch/run", post(orch_run))
        .route("/api/orch/tasks", get(orch_tasks))
        .route("/api/orch/summary", get(orch_summary))
        // LAN REMOTE
        .route("/api/lan/scan", get(lan_scan))
        .route("/api/lan/guide", get(lan_guide))
        // REMOTE OS INSTALL (app moja ya kwake: mteja ↔ mtaalamu)
        .route("/osinstall/request", post(osi_request))
        .route("/osinstall/sessions", get(osi_sessions))
        .route("/osinstall/status", get(osi_status))
        .route("/osinstall/approve", post(osi_approve))
        .route("/osinstall/cancel", post(osi_cancel))
        .route("/osinstall/bundles", get(osi_bundles))
        // Multicast ACK board
        .route("/api/mc/ack", post(mc_ack))
        .route("/api/mc/status", get(mc_status))
        // BUNDLES + PRICING + REACT (SEHEMU 2/3/13 — code kwa Rust)
        .route("/api/bundles", get(bundles_list))
        .route("/api/bundles", post(bundles_add))
        .route("/api/bundles/:id", axum::routing::delete(bundles_delete))
        .route("/api/bundles/apps", get(bundles_apps))
        .route("/api/bundles/distribute", post(bundles_distribute))
        .route("/api/secops/start", post(secops_start))
        .route("/api/secops/result", post(secops_result))
        .route("/api/secops/summary/:account", get(secops_summary))
        .route("/api/toolkit/catalog", get(toolkit_catalog))
        .route("/api/toolkit/run", post(toolkit_run))
        .route("/api/toolkit/batch", post(toolkit_batch))
        .route("/api/toolkit/runs", get(toolkit_runs))
        .route("/api/quote", post(quote_post))
        .route("/api/react/start", post(react_start))
        .route("/api/react/sweep", post(react_sweep))
        .route("/api/brain/remember", post(brain_remember))
        // PFSENSE (H5b): firewall kwenye mtandao mmoja — rules/aliases/services/status kupitia API halisi
        .route("/api/pfsense/rules", get(pfsense_rules))
        .route("/api/pfsense/rules", post(pfsense_rule_add))
        .route("/api/pfsense/aliases", get(pfsense_aliases))
        .route("/api/pfsense/aliases", post(pfsense_alias_add))
        .route("/api/pfsense/services", post(pfsense_service_restart))
        .route("/api/pfsense/status", get(pfsense_status))
        // HUDUMA ZA AGENT (mteja anaona HUDUMA tu — zana ziko fiche server-side)
        .route("/api/services", get(services_list))
        .route("/api/services/run", post(service_run))
        // ONBOARDING (server/computer/raspberry) — config automatic
        .route("/api/setup/onboard", post(setup_onboard))
        .route("/api/setup/status", get(setup_status))
        // CREDITS (billed per huduma + subscription)
        // NETWORK + SECURITY MANAGEMENT (vifaa vya mtandao + usalama wa kampuni)
        .route("/api/remote/net", post(remote_net))
        .route("/api/brain/recall", get(brain_recall))
        .route("/api/brain/stats", get(brain_stats))
        .route("/api/fleet/exec", post(fleet_exec))
        // AUTH + ADMIN ROLES + LICENSE + MODE (H3)
        .route("/api/auth/login", post(auth_login))
        .route("/api/auth/logout", post(auth_logout))
        .route("/api/auth/password", post(auth_password))
        .route("/api/tasks", get(tasks_list_all))
        .route("/api/tasks", post(task_assign))
        .route("/api/tasks/mine", get(tasks_mine))
        .route("/api/tasks/:id/complete", post(task_complete))
        .route("/api/license/issue", post(license_issue))
        .route("/api/license/validate", post(license_validate))
        // CUSTOM MODEL / API (mteja anaweka model yake au API yake — inatumika mara moja)
        // BILI (H9): SUBSCRIPTION + PAY-PER-USE (TZS — hakuna credits)
        .route("/api/billing/statement/:account", get(billing_statement))
        .route("/api/billing/topup", post(billing_topup))
        .route("/api/billing/subscribe", post(billing_subscribe))
        .route("/api/billing/charge", post(billing_charge))
        .route("/api/billing/prices", get(billing_prices))
        .route("/api/ai/custom", get(ai_custom_get))
        .route("/api/ai/custom", post(ai_custom_set))
        .route("/api/ai/custom", axum::routing::delete(ai_custom_clear))
        .route("/api/mode", get(mode_get))
        .route("/api/mode", post(mode_set))
        // CHAT + AUTO-DAILY + UPDATES (H3)
        .route("/api/chat", post(chat_ask))
        .route("/api/daily/report", get(daily_report))
        .route("/api/daily/permission", post(daily_permission))
        .route("/api/updates/check", post(updates_check))
        // LANGUAGE + REAL REMOTING (maelezo ya mmiliki)
        .route("/api/language", post(lang_set))
        .route("/api/language/:username", get(lang_get))
        .route("/api/translate", post(translate_post))
        .route("/api/remote/pc/:name", get(remote_pc))
        .route("/api/remote/fleet", post(remote_fleet))
        // COMPANY (kampuni/matawi kupitia wg0)
        .route("/api/company/branches", get(company_branches))
        .route("/api/company/branches", post(company_branch_add))
        .route("/api/company/summary", post(company_summary))
        // WIREGUARD VPN (P4) — kazi zote za mbali kupitia wg0
        .route("/api/vpn/init", post(vpn_init))
        .route("/api/vpn/status", get(vpn_status))
        .route("/api/vpn/peers", get(vpn_peers))
        .route("/api/vpn/peers", post(vpn_peer_add))
        .route("/api/vpn/peers/:name/conf", get(vpn_peer_conf))
        .route("/api/vpn/peers/:name", axum::routing::delete(vpn_peer_delete))
        .route("/api/vpn/up", post(vpn_up))
        .route("/api/vpn/down", post(vpn_down))
        .route("/api/vpn/scan", get(vpn_scan))
        .route("/api/vpn/server-conf", get(vpn_server_conf))
        // Agents
        .route("/agents", get(list_agents))
        // Supervisor
        .route("/supervisor/summary", get(supervisor_summary))
        // Cloud (P3)
        .route("/cloud/heartbeat", post(cloud_heartbeat))
        .route("/cloud/tenants", get(cloud_tenants))
        .route("/cloud/tenants", post(cloud_tenant_add))
        .route("/cloud/status", get(cloud_status))
        .nest_service("/ui", ui)
        .layer(CorsLayer::permissive())
        .with_state(state);

    tracing::info!("Fundi Deploy Agent v2 :8080 — UI /ui | cloud: {:?}", cloud::cloud_url());
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn root() -> Json<serde_json::Value> {
    Json(json!({
        "name": "OS AND APP INSTALLATION",
        "product": "MTECHOS",
        "main_modules": ["COMPUTER SOLUTIONS", "OS AND APP INSTALLATION"],
        "licensed_by": "Mbilinyi Tech (mbilinyitech.co.tz)",
        "version": "3.0.0",
        "role": "Agent inafanya kazi; msimamizi anasimamia (approve/cancel)",
        "features": ["os-install", "app-bundles-22", "wireguard-vpn", "real-remoting", "agentic-ai-react", "neuralis-brain-vector", "pricing-tzs", "license-mst", "auto-daily", "language-ai-translate", "white-label-services", "credit-billing", "self-host-onboarding", "network-device-mgmt"],
        "ui": "/ui"
    }))
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({
        "status": "ok",
        "version": "2.0.0",
        "services": ["pxe", "tftp", "http", "smb", "ai", "wol", "backup", "multicast", "cloud"],
        "cloud_outbox": cloud::outbox_len()
    }))
}

// ---------- jobs ----------

async fn list_jobs(State(s): State<AppState>) -> Json<Vec<Job>> {
    Json(s.jobs.read().await.clone())
}

async fn get_job(State(s): State<AppState>, Path(id): Path<String>) -> Json<Option<Job>> {
    Json(s.jobs.read().await.iter().find(|j| j.id == id).cloned())
}

async fn approve_job(State(s): State<AppState>, Path(id): Path<String>) -> Json<serde_json::Value> {
    let mut jobs = s.jobs.write().await;
    if let Some(j) = jobs.iter_mut().find(|j| j.id == id) {
        if j.status == "awaiting_approval" {
            j.status = "approved".into();
            j.needs_approval = false;
            j.message = "Imeidhinishwa na msimamizi".into();
            let _ = sqlx::query("UPDATE jobs SET status=?, message=? WHERE id=?")
                .bind("approved")
                .bind("Imeidhinishwa na msimamizi")
                .bind(&id)
                .execute(&s.db)
                .await;
            return Json(json!({ "ok": true, "id": id, "status": "approved" }));
        }
        return Json(json!({ "ok": false, "error": "Job si awaiting_approval", "status": j.status }));
    }
    Json(json!({ "ok": false, "error": "Job haipo" }))
}

async fn cancel_job(State(s): State<AppState>, Path(id): Path<String>) -> Json<serde_json::Value> {
    pipeline::set_job(&s.db, &s.jobs, &id, "cancelled", "cancel", 0, "Imeghairiwa na msimamizi").await;
    Json(json!({ "ok": true, "id": id, "status": "cancelled" }))
}

// ---------- deploy ----------

async fn deploy(State(s): State<AppState>, Json(req): Json<DeployRequest>) -> Json<serde_json::Value> {
    start_deploy(s, req, false).await
}

async fn deploy_auto(State(s): State<AppState>, Json(req): Json<DeployRequest>) -> Json<serde_json::Value> {
    start_deploy(s, req, true).await
}

async fn start_deploy(s: AppState, req: DeployRequest, force_auto: bool) -> Json<serde_json::Value> {
    let auto = force_auto || req.auto_approve.unwrap_or(false);
    let mut ids = Vec::new();

    for (i, c) in req.computers.iter().enumerate() {
        let id = Uuid::new_v4().to_string();
        let mut opts = pipeline::PipelineOpts::from_request(&req, i);
        if force_auto {
            opts.auto_approve = true;
        }
        let os_show = if opts.os == "auto" { "auto" } else { opts.os.as_str() };
        let job = Job {
            id: id.clone(),
            device_mac: c.mac.clone(),
            device_name: c.name.clone(),
            os_type: os_show.into(),
            status: "pending".into(),
            stage: "queue".into(),
            progress: 0,
            message: "Katika foleni".into(),
            needs_approval: !auto,
            image: opts.image.clone(),
            multicast: opts.multicast,
        };
        let _ = sqlx::query(
            "INSERT INTO jobs (id, device_mac, device_name, os_type, status, stage, progress, message) VALUES (?,?,?,?,?,?,?,?)",
        )
        .bind(&job.id)
        .bind(&job.device_mac)
        .bind(&job.device_name)
        .bind(&job.os_type)
        .bind(&job.status)
        .bind(&job.stage)
        .bind(job.progress as i64)
        .bind(&job.message)
        .execute(&s.db)
        .await;

        s.jobs.write().await.push(job);
        ids.push(id.clone());

        let db = s.db.clone();
        let jobs = s.jobs.clone();
        let acks = s.acks.clone();
        let mac = c.mac.clone();
        let jid = id.clone();
        tokio::spawn(async move {
            pipeline::run_pipeline(db, jobs, acks, jid, mac, opts).await;
        });
    }

    Json(json!({
        "success": true,
        "job_ids": ids,
        "total": req.computers.len(),
        "mode": if auto { "auto" } else { "supervisor_approval" }
    }))
}

// ---------- discovery ----------

async fn computers() -> Json<serde_json::Value> {
    let hosts = discover::discover_hosts();
    // MAJINA YA KIOTOMATIKI: duplicates (hr, hr 1, hr 2…) kwa mpangilio wa IP
    let named: Vec<NamedHost> = assign_names(&hosts);
    Json(json!({ "computers": named, "count": named.len(), "naming": "auto-dedupe (base, 1, 2, 3… kwa mpangilio wa IP)" }))
}

// ---------- images ----------

async fn list_images() -> Json<serde_json::Value> {
    let imgs = images::list_images();
    let valid = imgs.iter().filter(|i| i.valid).count();
    Json(json!({ "path": images::images_root(), "count": imgs.len(), "valid": valid, "images": imgs }))
}

async fn get_image(State(_s): State<AppState>, Path(name): Path<String>) -> Json<serde_json::Value> {
    match images::find_image(&name) {
        Some(i) => Json(json!({ "image": i })),
        None => Json(json!({ "error": "haipatikani", "name": name })),
    }
}

// ---------- os selection ----------

#[derive(serde::Deserialize)]
struct OsSelectReq {
    specs: String,
    #[serde(default)]
    user_need: Option<String>,
}

async fn os_select(Json(req): Json<OsSelectReq>) -> Json<serde_json::Value> {
    let d = osselect::decide_full(&req.specs, req.user_need.as_deref().unwrap_or("office")).await;
    Json(json!({ "decision": d }))
}

async fn os_profiles() -> Json<serde_json::Value> {
    let cat = osselect::catalog();
    Json(json!({ "profiles": cat.profiles.len(), "skip_rules": cat.skip_rules }))
}

// ---------- backups ----------

async fn list_backups() -> Json<serde_json::Value> {
    let root = backup::backup_root();
    let mut out = Vec::new();
    if let Ok(macdirs) = std::fs::read_dir(&root) {
        for md in macdirs.flatten() {
            if let Ok(stamps) = std::fs::read_dir(md.path()) {
                for st in stamps.flatten() {
                    let mf = st.path().join("manifest.json");
                    if let Ok(txt) = std::fs::read_to_string(&mf) {
                        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&txt) {
                            out.push(v);
                        }
                    }
                }
            }
        }
    }
    out.sort_by(|a, b| {
        let ka = a["finished"].as_str().unwrap_or("");
        let kb = b["finished"].as_str().unwrap_or("");
        kb.cmp(ka)
    });
    Json(json!({ "root": root, "count": out.len(), "backups": out }))
}

// ---------- ripoti ya PDF (logo ya FUNDI) ----------

async fn report_pdf(State(s): State<AppState>) -> impl axum::response::IntoResponse {
    let jobs = s.jobs.read().await.clone();
    let summary = json!({
        "total": jobs.len(),
        "awaiting_your_approval": jobs.iter().filter(|j| j.status == "awaiting_approval").count(),
        "running": jobs.iter().filter(|j| j.status == "running").count(),
        "done": jobs.iter().filter(|j| j.status == "done").count(),
        "failed": jobs.iter().filter(|j| j.status == "failed").count(),
    });
    let rows: Vec<serde_json::Value> = jobs
        .iter()
        .map(|j| {
            json!({
                "id": j.id,
                "device_name": j.device_name,
                "os_type": j.os_type,
                "status": j.status,
                "progress": j.progress,
                "message": j.message,
            })
        })
        .collect();
    let pdf = report::jobs_report_pdf(&summary, &rows);
    (
        [
            (axum::http::header::CONTENT_TYPE, "application/pdf"),
            (
                axum::http::header::CONTENT_DISPOSITION,
                "inline; filename=\"fundi-deploy-report.pdf\"",
            ),
        ],
        pdf,
    )
}

// ---------- FUNDI MAP (hardware scan halisi) ----------

async fn map_scan() -> Json<serde_json::Value> {
    Json(hardware::scan_all().await)
}

// ---------- MULTI-AGENT orchestrator ----------

#[derive(serde::Deserialize)]
struct OrchLaunchReq {
    targets: Vec<String>,
    #[serde(default = "default_concurrency")]
    max_concurrent: usize,
}
fn default_concurrency() -> usize {
    32
}

async fn orch_launch(State(s): State<AppState>, Json(req): Json<OrchLaunchReq>) -> Json<serde_json::Value> {
    let n = s.orch.launch(req.targets).await;
    Json(json!({ "launched": n, "note": "Hatua za usalama tu (diagnose/backup-plan/verify/report); install inahitaji idhini kwenye /jobs" }))
}

async fn orch_run(State(s): State<AppState>) -> Json<serde_json::Value> {
    Json(s.orch.run_pending().await)
}

async fn orch_tasks(State(s): State<AppState>) -> Json<Vec<orchestrator::AgentTask>> {
    Json(s.orch.list().await)
}

async fn orch_summary(State(s): State<AppState>) -> Json<serde_json::Value> {
    Json(s.orch.summary().await)
}

// ---------- LAN REMOTE ----------

async fn lan_scan() -> Json<serde_json::Value> {
    Json(lan::scan_lan(400).await)
}

async fn lan_guide() -> Json<serde_json::Value> {
    Json(json!({ "guide": lan::guide_sw() }))
}

// ---------- REMOTE OS INSTALL (app moja ya kwake) ----------

#[derive(serde::Deserialize)]
struct OsiReq {
    customer: String,
    #[serde(default)]
    company: Option<String>,
    pc: String,
    #[serde(default = "osi_default_os")]
    os: String,
    #[serde(default = "osi_default_bundle")]
    bundle: String,
}
fn osi_default_os() -> String {
    "windows11".into()
}
fn osi_default_bundle() -> String {
    "home".into()
}

async fn osi_request(State(s): State<AppState>, Json(r): Json<OsiReq>) -> Json<serde_json::Value> {
    let session = s
        .remote
        .create(r.customer, r.company, r.pc, r.os, r.bundle)
        .await;
    Json(json!({
        "code": session.code,
        "status": session.status,
        "message": "Session imeundwa — toa code hii kwa mtaalamu; yeye ataipokea kwenye dashboard yake mara moja.",
    }))
}

async fn osi_sessions(State(s): State<AppState>) -> Json<Vec<remote::Session>> {
    Json(s.remote.list().await)
}

async fn osi_status(State(s): State<AppState>, axum::extract::Query(q): axum::extract::Query<std::collections::HashMap<String, String>>) -> Json<serde_json::Value> {
    match q.get("code") {
        Some(code) => match s.remote.get(code).await {
            Some(sess) => Json(json!({ "found": true, "session": sess })),
            None => Json(json!({ "found": false, "error": "Session haipo (angalia code)" })),
        },
        None => Json(json!({ "found": false, "error": "code ni lazima (?code=RMT-...)" })),
    }
}

async fn osi_approve(State(s): State<AppState>, Json(r): Json<serde_json::Value>) -> Json<serde_json::Value> {
    let code = r["code"].as_str().unwrap_or("").to_string();
    let tech = r["technician"].as_str().unwrap_or("mtaalamu").to_string();
    if code.is_empty() {
        return Json(json!({ "ok": false, "error": "code ni lazima" }));
    }
    let sess = s.remote.get(&code).await;
    let Some(sess) = sess else {
        return Json(json!({ "ok": false, "error": "Session haipo" }));
    };
    if sess.status != "requested" {
        return Json(json!({ "ok": false, "error": format!("Session iko '{}' (si requested)", sess.status) }));
    }

    // HITL idhini → OS install job (pipeline halisi ya fundi-deploy)
    let mac = sess.pc.clone();
    let os = sess.os.clone();
    s.remote.update(&code, |s| {
        s.status = "installing_os".into();
        s.progress = 5;
        s.message = "Imeidhinishwa — agent inaandaa OS install".into();
        s.approved_by = Some(tech.clone());
    }).await;

    // Anza pipeline halisi (DeployRequest ya fundi-deploy — PXE/imaging)
    let deploy_req = pipeline::DeployRequest {
        computers: vec![pipeline::ComputerTarget {
            mac: mac.clone(),
            name: sess.pc.clone(),
            specs: None,
        }],
        os_type: Some(os.clone()),
        auto_approve: Some(true), // mtaalamu ameidhinisha — hii ni idhini yenyewe
        user_need: None,
        image: None,
        multicast: Some(false),
        backup_mode: None,
        backup_source: None,
    };
    let _ = start_deploy(s.clone(), deploy_req, true).await;

    // Background: apps bundle baada ya OS (kwa sasa tunaanza sasa — pipeline inaendelea kwa jobs yake)
    let store = s.remote.clone();
    let code2 = code.clone();
    let bundle = sess.bundle.clone();
    tokio::spawn(async move {
        // subiri OS ianze (demo timing; production: drive na events za job)
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        remote::run_bundle(store, code2, bundle).await;
    });

    Json(json!({ "ok": true, "code": code, "status": "installing_os", "note": "OS job imeanzishwa + apps bundle iko kwenye foleni" }))
}

async fn osi_cancel(State(s): State<AppState>, Json(r): Json<serde_json::Value>) -> Json<serde_json::Value> {
    let code = r["code"].as_str().unwrap_or("");
    let ok = s.remote.update(code, |s| {
        s.status = "cancelled".into();
        s.message = "Imeghairiwa".into();
    }).await;
    Json(json!({ "ok": ok }))
}

async fn osi_bundles() -> Json<serde_json::Value> {
    Json(remote::bundle_list())
}

async fn report_meta() -> Json<serde_json::Value> {
    Json(report::report_meta())
}

// ---------- Bundles + Pricing + ReAct (Rust logic) ----------

async fn bundles_list(State(s): State<AppState>) -> Json<serde_json::Value> {
    let cat = bundles::catalog();
    let custom = bundles::list_custom_bundles(&s.db).await;
    Json(json!({
        "categories": bundles::Category::all().iter().map(|c| json!({
            "id": c.id(), "name_sw": c.name_sw(), "icon": c.icon(),
            "bundles": cat.iter().filter(|b| b.category == *c).map(|b| json!({
                "id": b.id, "name_sw": b.name_sw, "description": b.description,
                "apps": b.apps.iter().map(|ap| json!({ "id": ap.id, "name": ap.name, "critical": ap.critical }))
                    .collect::<Vec<_>>(),
            })).collect::<Vec<_>>(),
        })).collect::<Vec<_>>(),
        "total_bundles": cat.len(),
        "custom_bundles": custom,
    }))
}

async fn bundles_apps() -> Json<serde_json::Value> {
    Json(json!({ "apps": bundles::catalog().into_iter().flat_map(|b| b.apps)
        .map(|ap| json!({ "id": ap.id, "name": ap.name, "critical": ap.critical }))
        .collect::<Vec<_>>() }))
}

#[derive(serde::Deserialize)]
struct BundleAddReq {
    id: String,
    name_sw: String,
    #[serde(default)]
    description: String,
    category: String,
    app_ids: Vec<String>,
    #[serde(default = "default_creator")]
    created_by: String,
}
fn default_creator() -> String {
    "admin (dashboard)".into()
}

async fn bundles_add(State(s): State<AppState>, Json(r): Json<BundleAddReq>) -> Json<serde_json::Value> {
    match bundles::add_custom_bundle(&s.db, &r.id, &r.name_sw, &r.description, &r.category, r.app_ids, &r.created_by).await {
        Ok(b) => Json(json!({ "ok": true, "bundle": b })),
        Err(e) => Json(json!({ "ok": false, "error": e })),
    }
}

async fn bundles_delete(State(s): State<AppState>, Path(id): Path<String>) -> Json<serde_json::Value> {
    let ok = bundles::remove_custom_bundle(&s.db, &id).await;
    Json(json!({ "ok": ok, "id": id }))
}

#[derive(serde::Deserialize)]
struct QuoteReq {
    plan: String,
    pcs: usize,
    os: String,
    #[serde(default)]
    critical_apps: usize,
    #[serde(default)]
    apps_count: usize,
}

async fn quote_post(Json(r): Json<QuoteReq>) -> Json<serde_json::Value> {
    let Some(plan) = pricing::Plan::from_id(&r.plan) else {
        return Json(json!({ "ok": false, "error": "plan ni pay_per_use au subscription" }));
    };
    Json(json!({ "ok": true, "quote": pricing::quote(plan, r.pcs, &r.os, r.critical_apps, r.apps_count) }))
}

#[derive(serde::Deserialize)]
struct ReactStartReq {
    problem: String,
    #[serde(default)]
    targets: Vec<String>,
}

async fn react_start(Json(r): Json<ReactStartReq>) -> Json<serde_json::Value> {
    let mut sess = reacon::ReActSession::new(&r.problem, r.targets);
    sess.think_plan();
    Json(json!({ "ok": true, "session": sess }))
}

#[derive(serde::Deserialize)]
struct ReactSweepReq {
    problem: String,
    targets: Vec<String>,
}

async fn react_sweep(Json(r): Json<ReactSweepReq>) -> Json<serde_json::Value> {
    // THINK → ACT (ping sweep halisi /24, low-risk) → OBSERVE
    let cfg = vpn::Config::load();
    let (a, b, c) = cfg.subnet_base();
    let alive = reacon::ping_sweep((a, b, c), 400).await;
    let mut sess = reacon::ReActSession::new(&r.problem, r.targets);
    sess.think_plan();
    sess.observe_sweep(&alive);
    Json(json!({
        "ok": true,
        "session": sess,
        "alive": alive,
        "note_sw": "ReAct THINK→ACT→OBSERVE: sweep ni low-risk (automatic). Install/reboot zinahitaji RUHUSU (bounded autonomy)."
    }))
}

// ---------- Neuralis Brain (kumbukumbu ya pamoja ya agents) ----------

#[derive(serde::Deserialize)]
struct BrainRememberReq {
    agent: String,
    pc: String,
    problem: String,
    solution: String,
    confidence: f64,
}

async fn brain_remember(State(s): State<AppState>, Json(r): Json<BrainRememberReq>) -> Json<serde_json::Value> {
    match brain::remember(&s.brain, &r.agent, &r.pc, &r.problem, &r.solution, r.confidence).await {
        Ok(id) => Json(json!({ "ok": true, "id": id })),
        Err(e) => Json(json!({ "ok": false, "error": e })),
    }
}

async fn brain_recall(State(s): State<AppState>, axum::extract::Query(q): axum::extract::Query<std::collections::HashMap<String, String>>) -> Json<serde_json::Value> {
    let Some(query) = q.get("q") else {
        return Json(json!({ "ok": false, "error": "q ni lazima (?q=wi-fi haifanyi kazi)" }));
    };
    let limit: usize = q.get("limit").and_then(|l| l.parse().ok()).unwrap_or(5);
    let got = brain::recall(&s.brain, query, limit).await;
    Json(json!({
        "ok": true,
        "query": query,
        "results": got.iter().map(|(m, score)| json!({
            "agent": m.agent, "pc": m.pc, "problem": m.problem,
            "solution": m.solution, "confidence": m.confidence,
            "match_score": (score * 1000.0).round() / 1000.0,
        })).collect::<Vec<_>>(),
        "note_sw": "Kumbukumbu za agents wenzako — kila agent anajifunza kutoka kwa wengine (Neuralis Brain)."
    }))
}

async fn brain_stats(State(s): State<AppState>) -> Json<serde_json::Value> {
    Json(brain::stats(&s.brain).await)
}

// ---------- Fleet exec (ombi moja → OS nyingi, kama RDM) ----------

#[derive(serde::Deserialize)]
struct FleetExecReq {
    /// mfano: [{"os":"win11","apps":["util-anydesk"]},{"os":"ubuntu","apps":["dev-python"]}]
    os_groups: Vec<OsGroup>,
    #[serde(default)]
    human_approved: bool,
}
#[derive(serde::Deserialize)]
struct OsGroup {
    os: String,
    apps: Vec<String>,
}

async fn fleet_exec(Json(r): Json<FleetExecReq>) -> Json<serde_json::Value> {
    // Bounded autonomy: install = High risk → lazima idhini ya binadamu
    if !reacon::action_allowed(reacon::Action::InstallApp, r.human_approved) {
        return Json(json!({
            "ok": false,
            "error": "InstallApp ni High-risk — lazima idhini ya binadamu (human_approved: true). Hii ni bounded autonomy.",
            "status": "awaiting_approval"
        }));
    }
    let groups: Vec<(String, Vec<String>)> = r.os_groups.into_iter().map(|g| (g.os, g.apps)).collect();
    let plans = reacon::multi_os_plan(groups);
    Json(json!({
        "ok": true,
        "plans": plans.iter().map(|p| json!({
            "target": p.display_name, "os": p.os,
            "apps": p.app_ids,
            "install_commands": p.install_commands(),
        })).collect::<Vec<_>>(),
        "note_sw": "Ombi moja → OS nyingi kwa wakati mmoja (kama RDM AI assistant)."
    }))
}

// ---------- Auth (login/roles) ----------

#[derive(serde::Deserialize)]
struct LoginReq { username: String, password: String }

async fn auth_login(State(s): State<AppState>, Json(r): Json<LoginReq>) -> Json<serde_json::Value> {
    match auth::login(&s.db, &r.username, &r.password).await {
        Ok(sess) => Json(json!({ "ok": true, "session": {
            "token": sess.token, "username": sess.username, "role": sess.role,
            "role_name": if sess.role == "admin_mkuu" { "Admin Mkuu" } else { "Msaidizi" },
            "can_assign": sess.role == "admin_mkuu", "can_approve_high_risk": sess.role == "admin_mkuu",
        } })),
        Err(e) => Json(json!({ "ok": false, "error": e })),
    }
}

#[derive(serde::Deserialize)]
struct TokenReq { token: String }

async fn auth_logout(State(s): State<AppState>, Json(r): Json<TokenReq>) -> Json<serde_json::Value> {
    Json(json!({ "ok": auth::logout(&s.db, &r.token).await }))
}

#[derive(serde::Deserialize)]
struct PasswordReq { token: String, new_password: String }

async fn auth_password(State(s): State<AppState>, Json(r): Json<PasswordReq>) -> Json<serde_json::Value> {
    match auth::change_password(&s.db, &r.token, &r.new_password).await {
        Ok(()) => Json(json!({ "ok": true, "message": "Password imebadilishwa" })),
        Err(e) => Json(json!({ "ok": false, "error": e })),
    }
}

// ---------- Admin tasks (admin mkuu → wasaidizi) ----------

#[derive(serde::Deserialize)]
struct TaskAssignReq { token: String, pc: String, work: String, details_sw: String, assigned_to: String }

async fn task_assign(State(s): State<AppState>, Json(r): Json<TaskAssignReq>) -> Json<serde_json::Value> {
    // Role gate: token lazima iwe ya admin_mkuu
    let Some(sess) = auth::verify_token(&s.db, &r.token).await else {
        return Json(json!({ "ok": false, "error": "Login kwanza" }));
    };
    match admin::assign(&s.db, &r.pc, &r.work, &r.details_sw, &r.assigned_to, &sess.role).await {
        Ok(t) => Json(json!({ "ok": true, "task": t })),
        Err(e) => Json(json!({ "ok": false, "error": e })),
    }
}

async fn tasks_mine(State(s): State<AppState>, axum::extract::Query(q): axum::extract::Query<std::collections::HashMap<String, String>>) -> Json<serde_json::Value> {
    let Some(username) = q.get("username") else {
        return Json(json!({ "ok": false, "error": "username ni lazima" }));
    };
    let tasks = admin::list_for(&s.db, username).await;
    Json(json!({ "ok": true, "count": tasks.len(), "tasks": tasks }))
}

async fn tasks_list_all(State(s): State<AppState>) -> Json<serde_json::Value> {
    let tasks = admin::list_all(&s.db).await;
    Json(json!({ "ok": true, "count": tasks.len(), "tasks": tasks }))
}

#[derive(serde::Deserialize)]
struct TaskCompleteReq { id: String, status: String, report: String }

async fn task_complete(State(s): State<AppState>, Json(r): Json<TaskCompleteReq>) -> Json<serde_json::Value> {
    match admin::complete(&s.db, &r.id, &r.status, &r.report).await {
        Ok(()) => Json(json!({ "ok": true })),
        Err(e) => Json(json!({ "ok": false, "error": e })),
    }
}

// ---------- License (MST- keys) ----------

#[derive(serde::Deserialize)]
struct LicenseIssueReq { tier: String, owner: String }

async fn license_issue(State(s): State<AppState>, Json(r): Json<LicenseIssueReq>) -> Json<serde_json::Value> {
    let tier = match r.tier.as_str() {
        "personal" => license::Tier::Personal,
        "business" => license::Tier::Business,
        "enterprise" => license::Tier::Enterprise,
        other => return Json(json!({ "ok": false, "error": format!("tier '{other}' si sahihi") })),
    };
    match license::issue(&s.db, tier, &r.owner).await {
        Ok(l) => Json(json!({ "ok": true, "license": l, "branding": "Licensed by Mbilinyi Tech · mbilinyitech.co.tz" })),
        Err(e) => Json(json!({ "ok": false, "error": e })),
    }
}

#[derive(serde::Deserialize)]
struct LicenseValidateReq { key: String }

async fn license_validate(State(s): State<AppState>, Json(r): Json<LicenseValidateReq>) -> Json<serde_json::Value> {
    match license::validate(&s.db, &r.key).await {
        Ok(l) => Json(json!({ "ok": true, "license": l })),
        Err(e) => Json(json!({ "ok": false, "error": e })),
    }
}

// ---------- Mode (offline/online — mtu anaamua) ----------

async fn mode_get(State(s): State<AppState>) -> Json<serde_json::Value> {
    let m = mode::get_mode(&s.db).await;
    Json(json!({
        "mode": m.id(),
        "allows_vpn_remote": m.allows_vpn_remote(),
        "allows_ai_cloud": m.allows_ai_cloud(),
        "allows_update_check": m.allows_update_check(),
        "note_sw": "Mtu anaamua yeye: offline = kazi za ndani; online = VPN/AI/updates."
    }))
}

#[derive(serde::Deserialize)]
struct ModeSetReq { mode: String }

async fn mode_set(State(s): State<AppState>, Json(r): Json<ModeSetReq>) -> Json<serde_json::Value> {
    let m = match r.mode.as_str() {
        "offline" => mode::Mode::Offline,
        "online" => mode::Mode::Online,
        other => return Json(json!({ "ok": false, "error": format!("mode '{other}' si sahihi (offline|online)") })),
    };
    let m = mode::set_mode(&s.db, m).await;
    Json(json!({ "ok": true, "mode": m.id() }))
}

// ---------- Chat (agent inajibu) ----------

#[derive(serde::Deserialize)]
struct ChatReq {
    question: String,
    /// urefu wa jawabu: "short" | "medium" | "long" — default medium
    #[serde(default)]
    length: String,
}

async fn chat_ask(State(s): State<AppState>, Json(r): Json<ChatReq>) -> Json<serde_json::Value> {
    let online = mode::get_mode(&s.db).await == mode::Mode::Online;
    let len = match r.length.as_str() {
        "short" => chat::Length::Short,
        "long" => chat::Length::Long,
        _ => chat::Length::Medium,
    };
    let reply = chat::ask(&s.brain, &r.question, online, len).await;
    Json(json!({ "ok": true, "reply": reply }))
}

// ---------- Auto-daily (scan → ripoti → ruhusa → solve) ----------

async fn daily_report(State(s): State<AppState>) -> Json<serde_json::Value> {
    let findings = daily::today_report(&s.db).await;
    let awaiting = findings.iter().filter(|f| f.severity != "info").count();
    Json(json!({
        "ok": true,
        "date": chrono::Local::now().format("%Y-%m-%d").to_string(),
        "count": findings.len(),
        "findings": findings,
        "summary_sw": format!("Scan ya leo: matatizo {awaiting} — admin anaruhusu, agents zinaanza kutatua kwa wakati mmoja."),
    }))
}

async fn daily_permission(State(s): State<AppState>) -> Json<serde_json::Value> {
    let n = daily::grant_permission(&s.db).await;
    Json(json!({
        "ok": true,
        "granted": n,
        "message_sw": if n > 0 { format!("Ruhusa imepewa — matatizo {n} yameanza kutatuliwa kwa wakati mmoja.") } else { "Hakuna mapya yanayosubiri ruhusa.".into() },
    }))
}

// ---------- Updates (notification ya version mpya) ----------

async fn updates_check(State(s): State<AppState>) -> Json<serde_json::Value> {
    let online = mode::get_mode(&s.db).await == mode::Mode::Online;
    Json(json!({ "ok": true, "update": updates::check_and_notify(&s.db, online).await }))
}

// ---------- Language (chagua wakati wa kusajili + AI translate) ----------

#[derive(serde::Deserialize)]
struct LangSetReq { username: String, language: String }

async fn lang_set(State(s): State<AppState>, Json(r): Json<LangSetReq>) -> Json<serde_json::Value> {
    match language::set_language(&s.db, &r.username, &r.language).await {
        Ok(()) => Json(json!({ "ok": true, "username": r.username, "language": r.language })),
        Err(e) => Json(json!({ "ok": false, "error": e })),
    }
}

async fn lang_get(State(s): State<AppState>, Path(username): Path<String>) -> Json<serde_json::Value> {
    Json(json!({ "ok": true, "username": username, "language": language::get_language(&s.db, &username).await }))
}

#[derive(serde::Deserialize)]
struct TranslateReq { username: String, text: String }

async fn translate_post(State(s): State<AppState>, Json(r): Json<TranslateReq>) -> Json<serde_json::Value> {
    let (translated, lang) = language::auto_translate(&s.db, &r.username, &r.text).await;
    Json(json!({ "ok": true, "language": lang, "translated": translated }))
}

// ---------- Real Remoting (full view ya kila PC) ----------

async fn remote_pc(State(s): State<AppState>, Path(name): Path<String>) -> Json<serde_json::Value> {
    // PC: tafuta IP yake kutoka discovery ya mwisho (arp) au subnet ya VPN
    let cfg = vpn::Config::load();
    let (a, b, c) = cfg.subnet_base();
    // Kwa kasi: full view na IP ya VPN (jina == identity); services ni probe halisi
    let ip = format!("{a}.{b}.{c}.2"); // peers huanza .2
    let view = remote_view::full_view(&s.db, &name, &ip, "", 400).await;
    Json(json!({ "ok": true, "pc": view }))
}

#[derive(serde::Deserialize)]
struct RemoteFleetReq { hosts: Vec<RemoteHost> }
#[derive(serde::Deserialize)]
struct RemoteHost { name: String, ip: String }

async fn remote_fleet(State(s): State<AppState>, Json(r): Json<RemoteFleetReq>) -> Json<serde_json::Value> {
    let hosts: Vec<(String, String)> = r.hosts.into_iter().map(|h| (h.name, h.ip)).collect();
    let views = remote_view::fleet_snapshot(&s.db, &hosts, 400).await;
    Json(json!({
        "ok": true,
        "count": views.len(),
        "pcs": views,
        "note_sw": "Full computer view kwa kila PC — kupitia wg0 kwa mbali. KANUNI: kila uwanja ni probe halisi au DB ya kazi."
    }))
}

// ---------- Company (kampuni/matawi kupitia wg0) ----------

async fn company_branches(State(s): State<AppState>) -> Json<serde_json::Value> {
    let bs = company::list_branches(&s.db).await;
    Json(json!({ "ok": true, "count": bs.len(), "branches": bs }))
}

#[derive(serde::Deserialize)]
struct BranchAddReq { id: String, name: String, city: String, vpn_subnet: String }

async fn company_branch_add(State(s): State<AppState>, Json(r): Json<BranchAddReq>) -> Json<serde_json::Value> {
    match company::add_branch(&s.db, &r.id, &r.name, &r.city, &r.vpn_subnet).await {
        Ok(b) => Json(json!({ "ok": true, "branch": b })),
        Err(e) => Json(json!({ "ok": false, "error": e })),
    }
}

#[derive(serde::Deserialize)]
struct CompanySummaryReq {
    /// hosts kwa kila tawi: [{"branch":"dar","hosts":[{"name":"hr","ip":"10.66.66.2"}]}]
    hosts_by_branch: Vec<BranchHosts>,
}
#[derive(serde::Deserialize)]
struct BranchHosts { branch: String, hosts: Vec<RemoteHost> }

async fn company_summary(State(s): State<AppState>, Json(r): Json<CompanySummaryReq>) -> Json<serde_json::Value> {
    let hbb: Vec<(String, Vec<(String, String)>)> = r
        .hosts_by_branch
        .into_iter()
        .map(|b| (b.branch, b.hosts.into_iter().map(|h| (h.name, h.ip)).collect()))
        .collect();
    Json(company::company_summary(&s.db, &hbb, 400).await)
}

// ---------- WireGuard VPN (P4) ----------

async fn vpn_init(State(s): State<AppState>) -> Json<serde_json::Value> {
    match vpn::write_server_conf(&s.db).await {
        Ok(path) => Json(json!({
            "ok": true,
            "conf": path.to_string_lossy(),
            "endpoint_hint": "Weka FUNDI_WG_ENDPOINT (public IP au domain ya server) kwenye docker-compose ili client confs ziwe na Endpoint halisi.",
            "up": "POST /api/vpn/up (inahitaji NET_ADMIN + /dev/net/tun + wireguard-tools kwenye container)"
        })),
        Err(e) => Json(json!({ "ok": false, "error": e })),
    }
}

async fn vpn_status(State(s): State<AppState>) -> Json<serde_json::Value> {
    Json(vpn::status(&s.db).await)
}

async fn vpn_peers(State(s): State<AppState>) -> Json<serde_json::Value> {
    let peers = vpn::list_peers(&s.db).await;
    Json(json!({ "count": peers.len(), "peers": peers.iter().map(|p| json!({
        "name": p.name, "ip": p.ip, "created_at": p.created_at, "enabled": p.enabled,
    })).collect::<Vec<_>>() }))
}

#[derive(serde::Deserialize)]
struct VpnPeerAdd {
    name: String,
    #[serde(default)]
    note: Option<String>,
}

async fn vpn_peer_add(State(s): State<AppState>, Json(r): Json<VpnPeerAdd>) -> Json<serde_json::Value> {
    match vpn::add_peer(&s.db, &r.name, r.note.as_deref().unwrap_or("")).await {
        Ok(p) => Json(json!({
            "ok": true,
            "peer": { "name": p.name, "ip": p.ip },
            "conf_url": format!("/api/vpn/peers/{}/conf", p.name),
            "next": "Pakua conf hii kwenye computer ya mbali → wg-quick up <faili> — kisha kazi zote za mbali zinaenda kupitia VPN"
        })),
        Err(e) => Json(json!({ "ok": false, "error": e })),
    }
}

async fn vpn_peer_conf(State(s): State<AppState>, Path(name): Path<String>) -> impl axum::response::IntoResponse {
    let peers = vpn::list_peers(&s.db).await;
    let Some(peer) = peers.iter().find(|p| p.name == name) else {
        return ([
            (axum::http::header::CONTENT_TYPE, "text/plain".to_string()),
            (axum::http::header::CONTENT_DISPOSITION, "inline".to_string()),
        ], format!("Peer '{name}' haipo"));
    };
    let cfg = vpn::Config::load();
    let (_, server_pub) = match vpn::server_keys() {
        Ok(k) => k,
        Err(e) => return ([
            (axum::http::header::CONTENT_TYPE, "text/plain".to_string()),
            (axum::http::header::CONTENT_DISPOSITION, "inline".to_string()),
        ], format!("Server keys: {e}")),
    };
    let conf = vpn::client_conf(&cfg, peer, &server_pub);
    ([
        (axum::http::header::CONTENT_TYPE, "text/plain".to_string()),
        (
            axum::http::header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"wg-{name}.conf\""),
        ),
    ], conf)
}

async fn vpn_peer_delete(State(s): State<AppState>, Path(name): Path<String>) -> Json<serde_json::Value> {
    let ok = vpn::remove_peer(&s.db, &name).await;
    if ok {
        let _ = vpn::write_server_conf(&s.db).await; // regen conf bila peer
    }
    Json(json!({ "ok": ok, "name": name }))
}

async fn vpn_up(State(s): State<AppState>) -> Json<serde_json::Value> {
    let _ = vpn::write_server_conf(&s.db).await; // conf ya sasa (peers zote) kabla ya up
    match vpn::up(None).await {
        Ok(msg) => Json(json!({ "ok": true, "message": msg })),
        Err(e) => Json(json!({ "ok": false, "error": e })),
    }
}

async fn vpn_down(State(_s): State<AppState>) -> Json<serde_json::Value> {
    match vpn::down().await {
        Ok(msg) => Json(json!({ "ok": true, "message": msg })),
        Err(e) => Json(json!({ "ok": false, "error": e })),
    }
}

async fn vpn_scan(State(s): State<AppState>) -> Json<serde_json::Value> {
    let cfg = vpn::Config::load();
    let active = tokio::process::Command::new("wg")
        .args(["show", &cfg.interface])
        .output()
        .await
        .map(|o| o.status.success())
        .unwrap_or(false);
    let hosts = vpn::scan_vpn_subnet(&cfg, 400).await;
    let rows: Vec<serde_json::Value> = hosts
        .iter()
        .map(|(ip, ports)| {
            let names: Vec<String> = ports.iter().map(|p| match p {
                22 => "SSH".to_string(),
                445 => "SMB".to_string(),
                135 => "RPC".to_string(),
                3389 => "RDP".to_string(),
                other => format!("port-{other}"),
            }).collect();
            json!({ "ip": ip, "services": names })
        })
        .collect();
    Json(json!({
        "subnet": cfg.subnet,
        "vpn_active": active,
        "hosts_alive": rows.len(),
        "hosts": rows,
        "note_sw": if active { "Scan hii ilifanyika JUU YA TUNNEL ya WireGuard (wg0) — hosts ni za site ya mbali." } else { "Tunnel haipo — hosts hizi ni za LAN/ndani tu. Washa VPN: POST /api/vpn/up" }
    }))
}

async fn vpn_server_conf(State(s): State<AppState>) -> impl axum::response::IntoResponse {
    let peers = vpn::list_peers(&s.db).await;
    let cfg = vpn::Config::load();
    match vpn::server_keys() {
        Ok((priv_key, _)) => {
            let conf = vpn::server_conf(&cfg, &priv_key, &peers);
            ([
                (axum::http::header::CONTENT_TYPE, "text/plain"),
                (axum::http::header::CONTENT_DISPOSITION, "attachment; filename=\"wg0.conf\""),
            ], conf)
        }
        Err(e) => ([
            (axum::http::header::CONTENT_TYPE, "text/plain"),
            (axum::http::header::CONTENT_DISPOSITION, "inline"),
        ], format!("Server keys: {e}")),
    }
}

// ---------- multicast ----------

#[derive(serde::Deserialize)]
struct McAckReq {
    mac: String,
    sha256: String,
    #[serde(default)]
    received_bytes: u64,
    #[serde(default = "default_true")]
    ok: bool,
}

fn default_true() -> bool {
    true
}

async fn mc_ack(State(s): State<AppState>, Json(a): Json<McAckReq>) -> Json<serde_json::Value> {
    s.acks.record(multicast::McAck {
        mac: a.mac,
        sha256: a.sha256,
        received_bytes: a.received_bytes,
        ok: a.ok,
    });
    Json(json!({ "ok": true, "total_acks": s.acks.count() }))
}

async fn mc_status(State(s): State<AppState>) -> Json<serde_json::Value> {
    Json(json!({
        "group": multicast::MC_GROUP,
        "port": multicast::MC_PORT,
        "acks": s.acks.count(),
        "verified_ok": s.acks.ok_count()
    }))
}

// ---------- agents ----------

async fn list_agents(State(s): State<AppState>) -> Json<serde_json::Value> {
    let rows: Vec<(String, String, String, String)> =
        sqlx::query_as("SELECT id, name, role, status FROM agents ORDER BY id")
            .fetch_all(&s.db)
            .await
            .unwrap_or_default();
    let agents: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|(id, name, role, status)| json!({ "id": id, "name": name, "role": role, "status": status }))
        .collect();
    Json(json!({ "agents": agents, "count": agents.len() }))
}

// ---------- supervisor ----------

async fn supervisor_summary(State(s): State<AppState>) -> Json<serde_json::Value> {
    let jobs = s.jobs.read().await;
    Json(json!({
        "role": "msimamizi",
        "awaiting_your_approval": jobs.iter().filter(|j| j.status == "awaiting_approval").count(),
        "running": jobs.iter().filter(|j| j.status == "running").count(),
        "done": jobs.iter().filter(|j| j.status == "done").count(),
        "failed": jobs.iter().filter(|j| j.status == "failed").count(),
        "total": jobs.len(),
        "cloud": {
            "tenant_id": cloud::tenant_id(),
            "outbox_pending": cloud::outbox_len()
        }
    }))
}

// ---------- cloud (P3) ----------

#[derive(serde::Deserialize)]
struct CloudTenantAdd {
    id: String,
    name: String,
    token: String,
}

async fn cloud_heartbeat(
    State(s): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let tenant = body["tenant_id"].as_str().unwrap_or("unknown").to_string();
    let events = body["events"].as_array().cloned().unwrap_or_default();
    for e in &events {
        let _ = sqlx::query("INSERT INTO cloud_events (tenant_id, kind, payload, created_at) VALUES (?,?,?,?)")
            .bind(&tenant)
            .bind("heartbeat")
            .bind(e.to_string())
            .bind(chrono::Local::now().to_rfc3339())
            .execute(&s.db)
            .await;
    }
    Json(json!({ "ok": true, "received": events.len(), "tenant": tenant }))
}

async fn cloud_tenants(State(s): State<AppState>) -> Json<serde_json::Value> {
    let tenants = cloud::list_tenants(&s.db).await;
    Json(json!({ "tenants": tenants, "count": tenants.len() }))
}

async fn cloud_tenant_add(
    State(s): State<AppState>,
    Json(t): Json<CloudTenantAdd>,
) -> Json<serde_json::Value> {
    match cloud::upsert_tenant(&s.db, &t.id, &t.name, &t.token).await {
        Ok(()) => Json(json!({ "ok": true, "id": t.id, "name": t.name })),
        Err(e) => Json(json!({ "ok": false, "error": e.to_string() })),
    }
}

async fn cloud_status(State(s): State<AppState>) -> Json<serde_json::Value> {
    let events: i64 = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM cloud_events")
        .fetch_one(&s.db)
        .await
        .unwrap_or(0);
    Json(json!({
        "hub_mode": true,
        "cloud_url": cloud::cloud_url(),
        "tenant_id": cloud::tenant_id(),
        "outbox_pending": cloud::outbox_len(),
        "events_received": events
    }))
}

// ---------- PFSENSE (H5b): firewall ya mtandao mmoja — API halisi ----------

#[derive(serde::Deserialize)]
struct PfRuleReq {
    /// mf. {"action":"pass","interface":"lan","protocol":"tcp","source":"lan","destination":"any","destination_port":"80"}
    #[serde(flatten)]
    rule: serde_json::Value,
}

async fn pfsense_rules(State(s): State<AppState>) -> Json<serde_json::Value> {
    if !s.pfsense.is_configured() {
        return Json(json!({ "ok": false, "error": "pfSense API haijawekwa: PFSENSE_API_URL + PFSENSE_API_KEY (env)", "configured": false }));
    }
    match s.pfsense.list_rules().await {
        Ok(v) => Json(json!({ "ok": true, "configured": true, "data": v })),
        Err(e) => Json(json!({ "ok": false, "configured": true, "error": e })),
    }
}

async fn pfsense_rule_add(State(s): State<AppState>, Json(r): Json<PfRuleReq>) -> Json<serde_json::Value> {
    if !s.pfsense.is_configured() {
        return Json(json!({ "ok": false, "error": "pfSense API haijawekwa: PFSENSE_API_URL + PFSENSE_API_KEY (env)", "configured": false }));
    }
    match s.pfsense.add_rule(r.rule).await {
        Ok(v) => Json(json!({ "ok": true, "configured": true, "data": v })),
        Err(e) => Json(json!({ "ok": false, "configured": true, "error": e })),
    }
}

async fn pfsense_aliases(State(s): State<AppState>) -> Json<serde_json::Value> {
    if !s.pfsense.is_configured() {
        return Json(json!({ "ok": false, "error": "pfSense API haijawekwa: PFSENSE_API_URL + PFSENSE_API_KEY (env)", "configured": false }));
    }
    match s.pfsense.list_aliases().await {
        Ok(v) => Json(json!({ "ok": true, "configured": true, "data": v })),
        Err(e) => Json(json!({ "ok": false, "configured": true, "error": e })),
    }
}

async fn pfsense_alias_add(State(s): State<AppState>, Json(r): Json<PfRuleReq>) -> Json<serde_json::Value> {
    if !s.pfsense.is_configured() {
        return Json(json!({ "ok": false, "error": "pfSense API haijawekwa: PFSENSE_API_URL + PFSENSE_API_KEY (env)", "configured": false }));
    }
    match s.pfsense.add_alias(r.rule).await {
        Ok(v) => Json(json!({ "ok": true, "configured": true, "data": v })),
        Err(e) => Json(json!({ "ok": false, "configured": true, "error": e })),
    }
}

#[derive(serde::Deserialize)]
struct PfServiceReq {
    name: String,
}

async fn pfsense_service_restart(State(s): State<AppState>, Json(r): Json<PfServiceReq>) -> Json<serde_json::Value> {
    if !s.pfsense.is_configured() {
        return Json(json!({ "ok": false, "error": "pfSense API haijawekwa: PFSENSE_API_URL + PFSENSE_API_KEY (env)", "configured": false }));
    }
    match s.pfsense.restart_service(&r.name).await {
        Ok(v) => Json(json!({ "ok": true, "configured": true, "service": r.name, "data": v })),
        Err(e) => Json(json!({ "ok": false, "configured": true, "service": r.name, "error": e })),
    }
}

async fn pfsense_status(State(s): State<AppState>) -> Json<serde_json::Value> {
    if !s.pfsense.is_configured() {
        return Json(json!({ "ok": false, "configured": false, "error": "pfSense API haijawekwa: PFSENSE_API_URL + PFSENSE_API_KEY (env)" }));
    }
    match s.pfsense.system_status().await {
        Ok(v) => Json(json!({ "ok": true, "configured": true, "data": v })),
        Err(e) => Json(json!({ "ok": false, "configured": true, "error": e })),
    }
}
// ---------- HUDUMA ZA AGENT + ONBOARDING + NET MGMT ----------

/// Katalogi ya huduma — mteja anaona jina la HUDUMA + credits PEKEE.

/// Katalogi ya huduma — mteja anaona jina la HUDUMA + bei ya TZS PEKEE.
async fn services_list() -> Json<serde_json::Value> {
    Json(json!({
        "ok": true,
        "services": tools::public_service_list(),
        "note_sw": "MTECH OS inachagua vifaa vya ndani vyenyewe kwa kila huduma — wewe ukitumia tu. Subscription au pay-per-use (tab 💰 BILI)."
    }))
}

#[derive(serde::Deserialize)]
struct ServiceRunReq {
    account: String,
    service: String,
    target: String,
    #[serde(default)]
    ref_code: String,
}

async fn service_run(State(s): State<AppState>, Json(r): Json<ServiceRunReq>) -> Json<serde_json::Value> {
    // 1. BILI GATE (H9): subscription hai inatosha; vinginevyo pay-per-use kutoka salio
    match billing::authorize(&s.db, &r.account, &r.service, 1).await {
        Ok(_) => {}
        Err(e) => return Json(json!({ "ok": false, "error": e, "needs_billing": true })),
    }
    // 2. Utekelezaji HALISI (backend fiche) — matokeo yanasafishwa kwa sanitize_output
    let result = tools_internal::run_service(&r.service, &r.target).await;
    // 3. CHARGE: ledger inaandikwa tu kama kazi imefanikiwa (subscription haijachaji)
    if result["ok"] == serde_json::Value::Bool(true) {
        let refc = if r.ref_code.is_empty() { format!("svc-{}", uuid::Uuid::new_v4()) } else { r.ref_code.clone() };
        let _ = billing::charge(&s.db, &r.account, &r.service, 1, &refc).await;
    }
    Json(result)
}

#[derive(serde::Deserialize)]
struct SetupOnboardReq {
    target: String,
}

async fn setup_onboard(State(s): State<AppState>, Json(r): Json<SetupOnboardReq>) -> Json<serde_json::Value> {
    let Some(t) = server_setup::Target::parse(&r.target) else {
        return Json(json!({ "ok": false, "error": "target si sahihi: server | computer | raspberry | other" }));
    };
    match server_setup::onboard(&s.db, t).await {
        Ok(cfg) => Json(json!({ "ok": true, "config": cfg })),
        Err(e) => Json(json!({ "ok": false, "error": e })),
    }
}

async fn setup_status(State(s): State<AppState>) -> Json<serde_json::Value> {
    let st = server_setup::status(&s.db).await;
    Json(json!({ "ok": true, "status": st }))
}

async fn remote_net(State(s): State<AppState>, Json(r): Json<serde_json::Value>) -> Json<serde_json::Value> {
    let result = netmgmt::handle(r).await;
    Json(result)
}

// ---------- CUSTOM MODEL / API (AI ya mteja yenyewe) ----------

async fn ai_custom_get(State(s): State<AppState>) -> Json<serde_json::Value> {
    let cfg = ai_config::load_with_db(&s.db).await;
    Json(json!({
        "ok": true,
        "url": cfg.url,
        "model": cfg.model,
        "offline_capable": cfg.offline_capable,
        "note_sw": "Weka endpoint ya model yako (Ollama / llama.cpp / vLLM / OpenAI-compatible) — inatumika mara moja."
    }))
}

async fn ai_custom_set(State(s): State<AppState>, Json(r): Json<ai_config::CustomAi>) -> Json<serde_json::Value> {
    match ai_config::set_custom(&s.db, &r).await {
        Ok(()) => Json(json!({ "ok": true, "url": r.url.trim().trim_end_matches('/'), "model": r.model.trim() })),
        Err(e) => Json(json!({ "ok": false, "error": e })),
    }
}

async fn ai_custom_clear(State(s): State<AppState>) -> Json<serde_json::Value> {
    match ai_config::clear_custom(&s.db).await {
        Ok(()) => Json(json!({ "ok": true, "note_sw": "Rejea AI ya ndani ya mfumo." })),
        Err(e) => Json(json!({ "ok": false, "error": e })),
    }
}

// ---------- BILI (H9): Subscription + Pay-per-use (TZS) ----------

async fn billing_statement(State(s): State<AppState>, Path(account): Path<String>) -> Json<serde_json::Value> {
    Json(billing::statement(&s.db, &account).await)
}

#[derive(serde::Deserialize)]
struct BillingTopupReq { account: String, amount_tzs: i64, ref_code: String }

async fn billing_topup(State(s): State<AppState>, Json(r): Json<BillingTopupReq>) -> Json<serde_json::Value> {
    match billing::topup(&s.db, &r.account, r.amount_tzs, &r.ref_code).await {
        Ok(bal) => Json(json!({ "ok": true, "account": r.account, "balance_tzs": bal })),
        Err(e) => Json(json!({ "ok": false, "error": e })),
    }
}

#[derive(serde::Deserialize)]
struct BillingSubReq { account: String, devices: usize, #[serde(default)] ref_code: String }

async fn billing_subscribe(State(s): State<AppState>, Json(r): Json<BillingSubReq>) -> Json<serde_json::Value> {
    match billing::subscribe(&s.db, &r.account, r.devices, &r.ref_code).await {
        Ok(v) => Json(v),
        Err(e) => Json(json!({ "ok": false, "error": e })),
    }
}

#[derive(serde::Deserialize)]
struct BillingChargeReq { account: String, job: String, #[serde(default)] units: usize, #[serde(default)] ref_code: String }

async fn billing_charge(State(s): State<AppState>, Json(r): Json<BillingChargeReq>) -> Json<serde_json::Value> {
    let refc = if r.ref_code.is_empty() { format!("job-{}", uuid::Uuid::new_v4()) } else { r.ref_code };
    match billing::charge(&s.db, &r.account, &r.job, r.units, &refc).await {
        Ok(bal) => Json(json!({ "ok": true, "account": r.account, "balance_tzs": bal })),
        Err(e) => Json(json!({ "ok": false, "error": e })),
    }
}

async fn billing_prices() -> Json<serde_json::Value> {
    Json(json!({
        "ok": true,
        "subscription_monthly_per_device_tzs": { "basic(1-10)": billing::SUB_BASIC_TZS, "standard(11-50)": billing::SUB_STANDARD_TZS, "business(51-200)": billing::SUB_BUSINESS_TZS, "enterprise(200+)": billing::SUB_ENTERPRISE_TZS },
        "pay_per_use_tzs": { "scan": billing::PAYG_SCAN_TZS, "repair": billing::PAYG_REPAIR_TZS, "os_install": billing::PAYG_OS_INSTALL_TZS, "app_install": billing::PAYG_APP_INSTALL_TZS, "forensic": billing::PAYG_FORENSIC_TZS, "netmgmt": billing::PAYG_NETMGMT_TZS },
        "volume_discount": { "10+": "10%", "50+": "20%" },
        "note_sw": "Subscription = huduma ZOTE kwa mwezi kwa kila kifaa. Pay-per-use = ulipa kazi uliyofanyika. Zote zinatumia wallet (ClickPesa/benki)."
    }))
}

// ---------- BUNDLE REPOSITORY: DISTRIBUTE (H10) ----------

#[derive(serde::Deserialize)]
struct BundleDistributeReq {
    bundle: String,
    targets: Vec<String>,
    account: String,
    /// os ya kifaa (win11/ubuntu/kali…) — inatumika kwenye install commands
    #[serde(default = "default_os")]
    os: String,
}

fn default_os() -> String { "win11".into() }

async fn bundles_distribute(State(s): State<AppState>, Json(r): Json<BundleDistributeReq>) -> Json<serde_json::Value> {
    if r.targets.is_empty() {
        return Json(json!({ "ok": false, "error": "chagua kompyuta angalau moja" }));
    }
    // BILI GATE: subscription inatosha; vinginevyo app_install × targets kutoka salio
    match billing::authorize(&s.db, &r.account, "app_install", r.targets.len()).await {
        Ok(_) => {}
        Err(e) => return Json(json!({ "ok": false, "error": e, "needs_billing": true })),
    }
    // Thibitisha bundle ipo (katalogi halisi ya Rust)
    let bundle = match bundles::find_bundle(&r.bundle) {
        Some(b) => b,
        None => return Json(json!({ "ok": false, "error": format!("bundle '{}' haipo kwenye repository", r.bundle) })),
    };
    // Unda kazi za distribusheni (HITL — needs_approval) kwa kila target
    let mut jobs = s.jobs.write().await;
    let mut ids = Vec::new();
    for name in &r.targets {
        let id = Uuid::new_v4().to_string();
        jobs.push(Job {
            id: id.clone(),
            device_mac: String::new(),
            device_name: name.clone(),
            os_type: r.os.clone(),
            status: "queued".into(),
            stage: format!("bundle:{}", bundle.id),
            progress: 0,
            message: format!("Distribution ya '{}' inasubiri idhini (HITL)", bundle.name_sw),
            needs_approval: true,
            image: None,
            multicast: false,
        });
        ids.push(id);
    }
    drop(jobs);
    // Chaji pay-per-use BAADA ya kazi kuundwa (subscription haijachaji)
    let refc = format!("dist-{}", uuid::Uuid::new_v4());
    let bal = billing::charge(&s.db, &r.account, "app_install", r.targets.len(), &refc).await.unwrap_or(0);
    Json(json!({
        "ok": true,
        "jobs": ids,
        "count": ids.len(),
        "bundle": bundle.name_sw,
        "balance_tzs": bal,
        "note_sw": "Kazi zimesubiri RUHUSU (HITL) kwenye tab Jobs — agent inasakinisha apps kwa wakati mmoja baada ya idhini."
    }))
}

// ---------- CYBER SECURITY & FORENSICS CENTER (H11) ----------

#[derive(serde::Deserialize)]
struct SecOpsStartReq {
    account: String,
    targets: Vec<String>,
    mode: String,
}

async fn secops_start(State(s): State<AppState>, Json(r): Json<SecOpsStartReq>) -> Json<serde_json::Value> {
    let targets: Vec<String> = r.targets.iter().map(|t| t.trim().to_string()).filter(|t| !t.is_empty()).collect();
    if targets.is_empty() {
        return Json(json!({ "ok": false, "error": "chagua kompyuta angalau moja" }));
    }
    let mode = match secops::Mode::parse(&r.mode) {
        Some(m) => m,
        None => return Json(json!({ "ok": false, "error": "chagua hali: security | forensics | both" })),
    };
    // BILI GATE: subscription inatosha; vinginevyo salio (kila op ina bei yake)
    for op in mode.ops() {
        if let Err(e) = billing::authorize(&s.db, &r.account, secops::bill_key(op), targets.len()).await {
            return Json(json!({ "ok": false, "error": e, "needs_billing": true }));
        }
    }
    // Kazi za HITL — kila target × op, zinaendeshwa kwa WAKATI MMOJA baada ya idhini
    let mut jobs = s.jobs.write().await;
    let mut ids = Vec::new();
    for name in &targets {
        for op in mode.ops() {
            let id = Uuid::new_v4().to_string();
            let label = if *op == "forensics" { "Uchunguzi wa kidijitali (ushahidi)" } else { "Uchunguzi wa usalama" };
            jobs.push(Job {
                id: id.clone(),
                device_mac: String::new(),
                device_name: name.clone(),
                os_type: String::new(),
                status: "queued".into(),
                stage: format!("secops:{}", op),
                progress: 0,
                message: format!("{} inasubiri idhini (HITL)", label),
                needs_approval: true,
                image: None,
                multicast: false,
            });
            ids.push(id);
        }
    }
    drop(jobs);
    // Malipo BAADA ya kazi kuundwa (subscription haijachaji)
    let mut bal = 0i64;
    for op in mode.ops() {
        let refc = format!("sec-{}", Uuid::new_v4());
        bal = billing::charge(&s.db, &r.account, secops::bill_key(op), targets.len(), &refc).await.unwrap_or(0);
    }
    Json(json!({
        "ok": true,
        "jobs": ids,
        "count": ids.len(),
        "mode": mode.as_str(),
        "prices": { "security_tzs": secops::PRICE_SECURITY_TZS, "forensics_tzs": secops::PRICE_FORENSIC_TZS },
        "balance_tzs": bal,
        "note_sw": "Kazi zinaendeshwa kwa WAKATI MMOJA kwenye kompyuta zote baada ya RUHUSU (HITL, tab Jobs)."
    }))
}

#[derive(serde::Deserialize)]
struct SecOpsResultReq {
    account: String,
    target: String,
    mode: String,
    #[serde(default)]
    findings: Vec<secops::Finding>,
    #[serde(default)]
    sources: String,
    #[serde(default)]
    note: String,
}

async fn secops_result(State(s): State<AppState>, Json(mut r): Json<SecOpsResultReq>) -> Json<serde_json::Value> {
    if !tools_internal::valid_target(&r.target) {
        return Json(json!({ "ok": false, "error": "jina la kompyuta si salama" }));
    }
    r.findings = r.findings.drain(..).map(|f| f.normalized()).collect();
    let mode = match secops::Mode::parse(&r.mode) {
        Some(m) => m,
        None => return Json(json!({ "ok": false, "error": "hali si sahihi: security | forensics" })),
    };
    match mode {
        secops::Mode::Security => match secops::save_report(&s.db, &r.account, &r.target, "security", &r.findings).await {
            Ok((id, health, sev)) => Json(json!({ "ok": true, "report_id": id, "health": health, "severity": sev })),
            Err(e) => Json(json!({ "ok": false, "error": e })),
        },
        secops::Mode::Forensics => match secops::save_case(&s.db, &r.account, &r.target, &r.sources, &r.note).await {
            Ok((id, hash)) => Json(json!({ "ok": true, "case_id": id, "evidence_hash": hash, "sealed": true })),
            Err(e) => Json(json!({ "ok": false, "error": e })),
        },
        secops::Mode::Both => Json(json!({ "ok": false, "error": "tuma matokeo ya kila hali peke yake (security au forensics)" })),
    }
}

async fn secops_summary(State(s): State<AppState>, Path(account): Path<String>) -> Json<serde_json::Value> {
    Json(secops::summary(&s.db, &account).await)
}

// ---------- TOOLKIT LAYER (H12) — registry + executor + batch wakati mmoja ----------

#[derive(serde::Deserialize)]
struct ToolkitRunReq {
    account: String,
    tool: String,
    target: String,
}

async fn toolkit_run(State(s): State<AppState>, Json(r): Json<ToolkitRunReq>) -> Json<serde_json::Value> {
    // BILI GATE: zana za mfumo (system/*) ni bure kwa wateja wa subscription;
    // nyingine = scan (malware_scan TZS 2,000) au forensic (digital_forensic TZS 25,000)
    let bill_key = if r.tool.starts_with("system") { "health_check" } else if r.tool.starts_with("forensic") { "digital_forensic" } else { "malware_scan" };
    match billing::authorize(&s.db, &r.account, bill_key, 1).await {
        Ok(_) => {}
        Err(e) => return Json(json!({ "ok": false, "error": e, "needs_billing": true })),
    }
    let out = toolkit::run_single(&s.db, toolkit::BatchJob {
        tool: r.tool, targets: vec![r.target], account: r.account.clone(),
    }).await;
    if out["ok"] == serde_json::Value::Bool(true) {
        let bk = if out["tool"].as_str().unwrap_or("").starts_with("forensic") { "digital_forensic" } else { "malware_scan" };
        let refc = format!("tool-{}", Uuid::new_v4());
        let _ = billing::charge(&s.db, &r.account, bk, 1, &refc).await;
    }
    Json(out)
}

#[derive(serde::Deserialize)]
struct ToolkitBatchReq {
    account: String,
    tool: String,
    targets: Vec<String>,
}

async fn toolkit_batch(State(s): State<AppState>, Json(r): Json<ToolkitBatchReq>) -> Json<serde_json::Value> {
    let targets: Vec<String> = r.targets.iter().map(|t| t.trim().to_string()).filter(|t| !t.is_empty()).collect();
    if targets.is_empty() {
        return Json(json!({ "ok": false, "error": "chagua kompyuta angalau moja" }));
    }
    let bill_key = if r.tool.starts_with("system") { "health_check" } else if r.tool.starts_with("forensic") { "digital_forensic" } else { "malware_scan" };
    match billing::authorize(&s.db, &r.account, bill_key, targets.len()).await {
        Ok(_) => {}
        Err(e) => return Json(json!({ "ok": false, "error": e, "needs_billing": true })),
    }
    let tool_id = r.tool.clone();
    let out = toolkit::run_batch(&s.db, toolkit::BatchJob { tool: r.tool, targets, account: r.account.clone() }).await;    let okc = out.iter().filter(|o| o["ok"] == serde_json::Value::Bool(true)).count();
    if okc > 0 {
        let bk = if tool_id.starts_with("forensic") { "digital_forensic" } else { "malware_scan" };
        let refc = format!("toolb-{}", Uuid::new_v4());
        let _ = billing::charge(&s.db, &r.account, bk, okc, &refc).await;
    }
    Json(json!({
        "ok": true, "tool": tool_id, "results": out, "count": out.len(), "success": okc,
        "note_sw": "Kazi zinaendeshwa kwa WAKATI MMOJA kwenye kompyuta zote.",
    }))
}

async fn toolkit_catalog(State(s): State<AppState>) -> Json<serde_json::Value> {
    let _ = &s.db;
    Json(toolkit::catalog_json(None))
}

async fn toolkit_runs(State(s): State<AppState>) -> Json<serde_json::Value> {
    Json(toolkit::runs_json(&s.db).await)
}
