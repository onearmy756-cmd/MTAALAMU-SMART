//! Fundi Deploy Agent v3 — P2 kamili + P3 cloud + P4 WIREGUARD VPN + MAJINA KIOTOMATIKI
//!
//! Pipeline: plan (AI+rules) → HITL → backup (halisi) → WOL → PXE → [multicast] → install → report
//! Msimamizi: approve/cancel + dashboard /ui
//! Cloud: tenants + heartbeats + outbox offline-first
//! VPN (P4): kazi ZOTE za mbali zinaenda kupitia WireGuard (wg0) — /api/vpn/*
//! Majina: duplicates (hr, hr 1, hr 2…) kulingana na mpangilio wa IP — /computers

mod ai;
mod backup;
mod bundles;
mod cloud;
mod discover;
mod hardware;
mod images;
mod lan;
mod multicast;
mod namer;
mod orchestrator;
mod osselect;
mod pipeline;
mod pricing;
mod reacon;
mod remote;
mod report;
mod vpn;
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
        .route("/api/quote", post(quote_post))
        .route("/api/react/start", post(react_start))
        .route("/api/react/sweep", post(react_sweep))
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
        "name": "Fundi Deploy Agent",
        "version": "2.0.0",
        "role": "Agent inafanya kazi; msimamizi anasimamia (approve/cancel)",
        "features": ["backup-halisi", "multicast", "ai-os-select", "lan-discovery", "images", "cloud-multi-tenant"],
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
