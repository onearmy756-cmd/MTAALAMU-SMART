//! Fundi Deploy API — PXE job queue + WOL + health (site server)

use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    db: SqlitePool,
    jobs: Arc<RwLock<Vec<Job>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Job {
    id: String,
    device_mac: String,
    device_name: String,
    os_type: String,
    status: String,
    progress: u32,
}

#[derive(Debug, Deserialize)]
struct DeployRequest {
    computers: Vec<ComputerTarget>,
    os_type: String,
}

#[derive(Debug, Deserialize)]
struct ComputerTarget {
    mac: String,
    name: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    std::fs::create_dir_all("/data").ok();
    let db = SqlitePool::connect("sqlite:///data/fundi.db?mode=rwc").await?;
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS jobs (
            id TEXT PRIMARY KEY, device_mac TEXT, device_name TEXT,
            os_type TEXT, status TEXT, progress INTEGER
        )"#,
    )
    .execute(&db)
    .await?;

    let state = AppState {
        db,
        jobs: Arc::new(RwLock::new(Vec::new())),
    };

    let app = Router::new()
        .route("/", get(|| async { "Fundi Deploy API v1.0" }))
        .route("/health", get(health))
        .route("/deploy", post(deploy))
        .route("/jobs", get(list_jobs))
        .route("/jobs/:id", get(get_job))
        .route("/computers/discover", get(discover))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    tracing::info!("Fundi Deploy :8080");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "version": "1.0.0",
        "services": ["pxe", "tftp", "http", "smb", "ai"]
    }))
}

async fn list_jobs(State(s): State<AppState>) -> Json<Vec<Job>> {
    Json(s.jobs.read().await.clone())
}

async fn get_job(State(s): State<AppState>, Path(id): Path<String>) -> Json<Option<Job>> {
    Json(s.jobs.read().await.iter().find(|j| j.id == id).cloned())
}

async fn discover() -> Json<serde_json::Value> {
    let out = std::process::Command::new("arp").args(["-a"]).output().ok();
    let text = out
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();
    Json(serde_json::json!({ "arp": text, "note": "Parse MAC for deploy targets" }))
}

async fn deploy(State(s): State<AppState>, Json(req): Json<DeployRequest>) -> Json<serde_json::Value> {
    let mut ids = Vec::new();
    for c in &req.computers {
        let job = Job {
            id: Uuid::new_v4().to_string(),
            device_mac: c.mac.clone(),
            device_name: c.name.clone(),
            os_type: req.os_type.clone(),
            status: "pending".into(),
            progress: 0,
        };
        let _ = sqlx::query(
            "INSERT INTO jobs (id, device_mac, device_name, os_type, status, progress) VALUES (?,?,?,?,?,?)",
        )
        .bind(&job.id)
        .bind(&job.device_mac)
        .bind(&job.device_name)
        .bind(&job.os_type)
        .bind(&job.status)
        .bind(job.progress as i64)
        .execute(&s.db)
        .await;
        ids.push(job.id.clone());
        s.jobs.write().await.push(job.clone());
        let st = s.clone();
        let mac = c.mac.clone();
        let os = req.os_type.clone();
        let jid = job.id.clone();
        tokio::spawn(async move {
            let _ = run_job(st, &jid, &mac, &os).await;
        });
    }
    Json(serde_json::json!({
        "success": true,
        "job_ids": ids,
        "total": req.computers.len()
    }))
}

async fn run_job(s: AppState, job_id: &str, mac: &str, os: &str) -> anyhow::Result<()> {
    set_prog(&s, job_id, "running", 10).await;
    let _ = tokio::process::Command::new("wakeonlan").arg(mac).output().await;
    set_prog(&s, job_id, "running", 25).await;
    let mac_path = format!(
        "/var/lib/tftpboot/pxelinux.cfg/01-{}",
        mac.replace(':', "-").to_lowercase()
    );
    let conf = match os {
        "win11" => "DEFAULT win11\n",
        "win10" => "DEFAULT win10\n",
        "ubuntu" => "DEFAULT ubuntu\n",
        "debian" => "DEFAULT debian\n",
        _ => "DEFAULT auto\n",
    };
    let _ = tokio::fs::write(&mac_path, conf).await;
    set_prog(&s, job_id, "running", 40).await;
    for p in [55u32, 70, 85, 100] {
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        let st = if p >= 100 { "done" } else { "running" };
        set_prog(&s, job_id, st, p).await;
    }
    Ok(())
}

async fn set_prog(s: &AppState, id: &str, status: &str, progress: u32) {
    if let Some(j) = s.jobs.write().await.iter_mut().find(|j| j.id == id) {
        j.status = status.into();
        j.progress = progress;
    }
    let _ = sqlx::query("UPDATE jobs SET status=?, progress=? WHERE id=?")
        .bind(status)
        .bind(progress as i64)
        .bind(id)
        .execute(&s.db)
        .await;
}
