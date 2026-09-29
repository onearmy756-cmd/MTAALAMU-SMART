//! Fundi Deploy Agent — msimamizi anasimamia; agent inafanya kazi

mod ai;
mod discover;
mod pipeline;
mod wol;

use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
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
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    std::fs::create_dir_all("/data").ok();
    std::fs::create_dir_all("/var/lib/tftpboot/pxelinux.cfg").ok();

    let db = SqlitePool::connect("sqlite:///data/fundi.db?mode=rwc").await?;
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS jobs (
            id TEXT PRIMARY KEY,
            device_mac TEXT,
            device_name TEXT,
            os_type TEXT,
            status TEXT,
            stage TEXT,
            progress INTEGER,
            message TEXT
        )"#,
    )
    .execute(&db)
    .await?;

    let state = AppState {
        db,
        jobs: Arc::new(RwLock::new(Vec::new())),
    };

    let ui = ServeDir::new("static").append_index_html_on_directories(true);

    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .route("/deploy", post(deploy))
        .route("/deploy/auto", post(deploy_auto))
        .route("/jobs", get(list_jobs))
        .route("/jobs/:id", get(get_job))
        .route("/jobs/:id/approve", post(approve_job))
        .route("/jobs/:id/cancel", post(cancel_job))
        .route("/computers", get(computers))
        .route("/computers/discover", get(computers))
        .route("/images", get(list_images))
        .route("/supervisor/summary", get(supervisor_summary))
        .nest_service("/ui", ui)
        .layer(CorsLayer::permissive())
        .with_state(state);

    tracing::info!("Fundi Deploy Agent :8080 — supervisor UI /ui");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn root() -> Json<serde_json::Value> {
    Json(json!({
        "name": "Fundi Deploy Agent",
        "version": "1.1.0",
        "role": "Agent inafanya kazi; msimamizi anasimamia (approve/cancel)",
        "ui": "/ui"
    }))
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({
        "status": "ok",
        "version": "1.1.0",
        "services": ["pxe", "tftp", "http", "smb", "ai", "wol"]
    }))
}

async fn list_jobs(State(s): State<AppState>) -> Json<Vec<Job>> {
    Json(s.jobs.read().await.clone())
}

async fn get_job(State(s): State<AppState>, Path(id): Path<String>) -> Json<Option<Job>> {
    Json(s.jobs.read().await.iter().find(|j| j.id == id).cloned())
}

async fn computers() -> Json<serde_json::Value> {
    let hosts = discover::discover_hosts();
    Json(json!({ "computers": hosts, "count": hosts.len() }))
}

async fn list_images() -> Json<serde_json::Value> {
    let root = std::env::var("FUNDI_IMAGES").unwrap_or_else(|_| "./images".into());
    let mut names = Vec::new();
    if let Ok(rd) = std::fs::read_dir(&root) {
        for e in rd.flatten() {
            names.push(e.file_name().to_string_lossy().to_string());
        }
    }
    Json(json!({ "path": root, "images": names }))
}

async fn supervisor_summary(State(s): State<AppState>) -> Json<serde_json::Value> {
    let jobs = s.jobs.read().await;
    Json(json!({
        "role": "msimamizi",
        "awaiting_your_approval": jobs.iter().filter(|j| j.status == "awaiting_approval").count(),
        "running": jobs.iter().filter(|j| j.status == "running").count(),
        "done": jobs.iter().filter(|j| j.status == "done").count(),
        "failed": jobs.iter().filter(|j| j.status == "failed").count(),
        "total": jobs.len()
    }))
}

async fn approve_job(State(s): State<AppState>, Path(id): Path<String>) -> Json<serde_json::Value> {
    let mut jobs = s.jobs.write().await;
    if let Some(j) = jobs.iter_mut().find(|j| j.id == id) {
        if j.status == "awaiting_approval" {
            j.status = "approved".into();
            j.needs_approval = false;
            j.message = "Imeidhinishwa na msimamizi".into();
            let _ = sqlx::query("UPDATE jobs SET status=?, message=? WHERE id=?")
                .bind("approved").bind("Imeidhinishwa na msimamizi").bind(&id).execute(&s.db).await;
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

async fn deploy(State(s): State<AppState>, Json(req): Json<DeployRequest>) -> Json<serde_json::Value> {
    start_deploy(s, req, false).await
}

async fn deploy_auto(State(s): State<AppState>, Json(req): Json<DeployRequest>) -> Json<serde_json::Value> {
    start_deploy(s, req, true).await
}

async fn start_deploy(s: AppState, req: DeployRequest, force_auto: bool) -> Json<serde_json::Value> {
    let auto = force_auto || req.auto_approve.unwrap_or(false);
    let os_default = req.os_type.clone().unwrap_or_else(|| "auto".into());
    let need = req.user_need.clone().unwrap_or_else(|| "office".into());
    let mut ids = Vec::new();

    for c in &req.computers {
        let id = Uuid::new_v4().to_string();
        let job = Job {
            id: id.clone(),
            device_mac: c.mac.clone(),
            device_name: c.name.clone(),
            os_type: os_default.clone(),
            status: "pending".into(),
            stage: "queue".into(),
            progress: 0,
            message: "Katika foleni".into(),
            needs_approval: !auto,
        };
        let _ = sqlx::query(
            "INSERT INTO jobs (id, device_mac, device_name, os_type, status, stage, progress, message) VALUES (?,?,?,?,?,?,?,?)",
        )
        .bind(&job.id).bind(&job.device_mac).bind(&job.device_name).bind(&job.os_type)
        .bind(&job.status).bind(&job.stage).bind(job.progress as i64).bind(&job.message)
        .execute(&s.db).await;

        s.jobs.write().await.push(job);
        ids.push(id.clone());

        let db = s.db.clone();
        let jobs = s.jobs.clone();
        let mac = c.mac.clone();
        let specs = c.specs.clone().unwrap_or_default();
        let need = need.clone();
        let os = os_default.clone();
        tokio::spawn(async move {
            pipeline::run_pipeline(db, jobs, id, mac, os, specs, need, auto).await;
        });
    }

    Json(json!({
        "success": true,
        "job_ids": ids,
        "total": req.computers.len(),
        "mode": if auto { "auto" } else { "supervisor_approval" }
    }))
}
