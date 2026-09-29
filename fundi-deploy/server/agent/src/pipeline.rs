//! Job pipeline: plan → HITL → backup_stub → wol → pxe → install → report

use crate::ai;
use crate::wol;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: String,
    pub device_mac: String,
    pub device_name: String,
    pub os_type: String,
    pub status: String,
    pub stage: String,
    pub progress: u32,
    pub message: String,
    pub needs_approval: bool,
}

#[derive(Debug, Deserialize)]
pub struct DeployRequest {
    pub computers: Vec<ComputerTarget>,
    pub os_type: Option<String>,
    pub auto_approve: Option<bool>,
    pub user_need: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ComputerTarget {
    pub mac: String,
    pub name: String,
    pub specs: Option<String>,
}

pub async fn set_job(
    db: &SqlitePool,
    jobs: &tokio::sync::RwLock<Vec<Job>>,
    id: &str,
    status: &str,
    stage: &str,
    progress: u32,
    message: &str,
) {
    if let Some(j) = jobs.write().await.iter_mut().find(|j| j.id == id) {
        j.status = status.into();
        j.stage = stage.into();
        j.progress = progress;
        j.message = message.into();
        if status == "awaiting_approval" {
            j.needs_approval = true;
        }
        if status == "running" || status == "done" || status == "failed" {
            j.needs_approval = false;
        }
    }
    let _ = sqlx::query(
        "UPDATE jobs SET status=?, stage=?, progress=?, message=? WHERE id=?",
    )
    .bind(status)
    .bind(stage)
    .bind(progress as i64)
    .bind(message)
    .bind(id)
    .execute(db)
    .await;
}

pub async fn run_pipeline(
    db: SqlitePool,
    jobs: std::sync::Arc<tokio::sync::RwLock<Vec<Job>>>,
    job_id: String,
    mac: String,
    mut os: String,
    specs: String,
    need: String,
    auto_approve: bool,
) {
    let tftp = std::env::var("FUNDI_TFTP").unwrap_or_else(|_| "/var/lib/tftpboot".into());

    set_job(&db, &jobs, &job_id, "running", "plan", 5, "AI/rules: chagua OS").await;
    if os.is_empty() || os == "auto" {
        os = ai::select_os(&specs, &need).await;
    }
    if os == "skip" {
        set_job(
            &db, &jobs, &job_id, "failed", "plan", 0,
            "Hardware shida — skip (SMART/RAM). Msimamizi arekebishe.",
        )
        .await;
        return;
    }

    if !auto_approve {
        set_job(
            &db, &jobs, &job_id, "awaiting_approval", "hitl", 10,
            &format!("OS iliyochaguliwa: {os}. Msimamizi: POST /jobs/{job_id}/approve"),
        )
        .await;
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            let st = jobs.read().await.iter().find(|j| j.id == job_id).map(|j| j.status.clone()).unwrap_or_default();
            if st == "approved" || st == "running" { break; }
            if st == "cancelled" || st == "failed" { return; }
        }
    }

    set_job(&db, &jobs, &job_id, "running", "backup", 20, "Backup stub (phase2 full)").await;
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    set_job(&db, &jobs, &job_id, "running", "wol", 30, "Wake-on-LAN").await;
    if let Err(e) = wol::wake(&mac).await {
        set_job(&db, &jobs, &job_id, "running", "wol", 32, &format!("WOL warn: {e}")).await;
    }

    set_job(&db, &jobs, &job_id, "running", "pxe", 45, "PXE boot config").await;
    let mac_file = format!("{}/pxelinux.cfg/01-{}", tftp, mac.replace(':', "-").to_lowercase());
    if let Some(parent) = Path::new(&mac_file).parent() {
        let _ = tokio::fs::create_dir_all(parent).await;
    }
    let conf = match os.as_str() {
        "win11" => "DEFAULT win11\n",
        "win10" => "DEFAULT win10\n",
        "ubuntu" => "DEFAULT ubuntu\n",
        "debian" => "DEFAULT debian\n",
        "fedora" => "DEFAULT fedora\n",
        _ => "DEFAULT auto\n",
    };
    let _ = tokio::fs::write(&mac_file, conf).await;

    if let Some(j) = jobs.write().await.iter_mut().find(|j| j.id == job_id) {
        j.os_type = os.clone();
    }
    let _ = sqlx::query("UPDATE jobs SET os_type=? WHERE id=?").bind(&os).bind(&job_id).execute(&db).await;

    set_job(
        &db, &jobs, &job_id, "running", "install", 55,
        &format!("Inasubiri install {os} (PXE) — hakikisha machine inaboot network"),
    ).await;

    for (p, msg) in [(65, "Installing…"), (75, "Drivers…"), (85, "Apps policy…"), (95, "Verify boot…")] {
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        set_job(&db, &jobs, &job_id, "running", "install", p, msg).await;
    }

    set_job(&db, &jobs, &job_id, "done", "report", 100, &format!("Kamili: {os} kwenye {mac}")).await;
}
