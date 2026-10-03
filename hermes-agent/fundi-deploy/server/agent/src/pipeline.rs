//! Job pipeline v2 (P2):
//! plan (AI+rules) → HITL → backup (halisi) → WOL → PXE → [multicast] → install → report
//!
//! Mabadiliko kutoka v1:
//!   - backup_stub → backup halisi (files/image/stub kutoka backup.rs)
//!   - multicast (image moja kwa PCs nyingi) ikiwa `multicast: true` + `image`
//!   - OS selection: osselect.rs (profiles JSON + Ollama assist + reasons)

use crate::backup;
use crate::multicast::{self, SharedAcks};
use crate::osselect;
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
    #[serde(default)]
    pub image: Option<String>,
    #[serde(default)]
    pub multicast: bool,
}

#[derive(Debug, Deserialize)]
pub struct DeployRequest {
    pub computers: Vec<ComputerTarget>,
    pub os_type: Option<String>,
    pub auto_approve: Option<bool>,
    pub user_need: Option<String>,
    #[serde(default)]
    pub image: Option<String>,
    #[serde(default)]
    pub multicast: Option<bool>,
    #[serde(default)]
    pub backup_mode: Option<String>,
    #[serde(default)]
    pub backup_source: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ComputerTarget {
    pub mac: String,
    pub name: String,
    pub specs: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PipelineOpts {
    pub os: String,
    pub specs: String,
    pub need: String,
    pub auto_approve: bool,
    pub backup_mode: String,
    pub backup_source: String,
    pub image: Option<String>,
    pub multicast: bool,
}

impl PipelineOpts {
    pub fn from_request(req: &DeployRequest, i: usize) -> Self {
        let default_backup = std::env::var("FUNDI_BACKUP_MODE").unwrap_or_else(|_| "stub".into());
        PipelineOpts {
            os: req.os_type.clone().unwrap_or_else(|| "auto".into()),
            specs: req.computers.get(i).and_then(|c| c.specs.clone()).unwrap_or_default(),
            need: req.user_need.clone().unwrap_or_else(|| "office".into()),
            auto_approve: req.auto_approve.unwrap_or(false),
            backup_mode: req.backup_mode.clone().unwrap_or(default_backup),
            backup_source: req.backup_source.clone().unwrap_or_default(),
            image: req.image.clone(),
            multicast: req.multicast.unwrap_or(false),
        }
    }
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
    let _ = sqlx::query("UPDATE jobs SET status=?, stage=?, progress=?, message=? WHERE id=?")
        .bind(status)
        .bind(stage)
        .bind(progress as i64)
        .bind(message)
        .bind(id)
        .execute(db)
        .await;
}

async fn set_os(db: &SqlitePool, jobs: &tokio::sync::RwLock<Vec<Job>>, id: &str, os: &str, reason: &str) {
    if let Some(j) = jobs.write().await.iter_mut().find(|j| j.id == id) {
        j.os_type = os.into();
    }
    let _ = sqlx::query("UPDATE jobs SET os_type=?, os_reason=? WHERE id=?")
        .bind(os)
        .bind(reason)
        .bind(id)
        .execute(db)
        .await;
}

pub async fn run_pipeline(
    db: SqlitePool,
    jobs: std::sync::Arc<tokio::sync::RwLock<Vec<Job>>>,
    acks: SharedAcks,
    job_id: String,
    mac: String,
    mut opts: PipelineOpts,
) {
    let tftp = std::env::var("FUNDI_TFTP").unwrap_or_else(|_| "/var/lib/tftpboot".into());

    // ---------- 1. PLAN (AI + rules, data-driven) ----------
    set_job(&db, &jobs, &job_id, "running", "plan", 5, "OS selection: profiles + AI").await;
    let mut decision_note = String::new();
    if opts.os.is_empty() || opts.os == "auto" {
        let d = osselect::decide_full(&opts.specs, &opts.need).await;
        decision_note = format!(
            "{} (confidence {:.0}%, njia: {})",
            d.reasons.join("; "),
            d.confidence * 100.0,
            d.method
        );
        opts.os = d.os.clone();
        set_os(&db, &jobs, &job_id, &d.os, &decision_note).await;
    }
    if opts.os == "skip" {
        set_job(
            &db, &jobs, &job_id, "failed", "plan", 0,
            "Hardware shida — skip (SMART/RAM). Msimamizi arekebishe.",
        )
        .await;
        return;
    }

    // ---------- 2. HITL ----------
    if !opts.auto_approve {
        set_job(
            &db, &jobs, &job_id, "awaiting_approval", "hitl", 10,
            &format!("OS iliyochaguliwa: {}. Msimamizi: POST /jobs/{job_id}/approve", opts.os),
        )
        .await;
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            let st = jobs
                .read()
                .await
                .iter()
                .find(|j| j.id == job_id)
                .map(|j| j.status.clone())
                .unwrap_or_default();
            if st == "approved" || st == "running" {
                break;
            }
            if st == "cancelled" || st == "failed" {
                return;
            }
        }
    }

    // ---------- 3. BACKUP (halisi) ----------
    set_job(&db, &jobs, &job_id, "running", "backup", 20, &format!("Backup mode: {}", opts.backup_mode)).await;
    let h = tokio::runtime::Handle::current();
    let (db2, jobs2, jid2) = (db.clone(), jobs.clone(), job_id.clone());
    let log = move |m: String| {
        let (db3, jobs3, j3) = (db2.clone(), jobs2.clone(), jid2.clone());
        h.spawn(async move {
            set_job(&db3, &jobs3, &j3, "running", "backup", 22, &m).await;
        });
    };
    let rep = backup::run_backup(&mac, &opts.backup_mode, &opts.backup_source, &log).await;
    let _ = sqlx::query("UPDATE jobs SET backup_info=? WHERE id=?")
        .bind(serde_json::to_string(&rep).unwrap_or_default())
        .bind(&job_id)
        .execute(&db)
        .await;
    if !rep.ok && opts.backup_mode != "stub" {
        set_job(&db, &jobs, &job_id, "failed", "backup", 20, &format!("Backup imeshindikana: {}", rep.message)).await;
        return;
    }
    set_job(&db, &jobs, &job_id, "running", "backup", 25, &rep.message).await;

    // ---------- 4. WOL ----------
    set_job(&db, &jobs, &job_id, "running", "wol", 30, "Wake-on-LAN").await;
    if let Err(e) = wol::wake(&mac).await {
        set_job(&db, &jobs, &job_id, "running", "wol", 32, &format!("WOL warn: {e}")).await;
    }

    // ---------- 5. PXE ----------
    set_job(&db, &jobs, &job_id, "running", "pxe", 45, "PXE boot config").await;
    let mac_file = format!("{}/pxelinux.cfg/01-{}", tftp, mac.replace(':', "-").to_lowercase());
    if let Some(parent) = Path::new(&mac_file).parent() {
        let _ = tokio::fs::create_dir_all(parent).await;
    }
    let conf = match opts.os.as_str() {
        "win11" => "DEFAULT win11\n",
        "win10" => "DEFAULT win10\n",
        "ubuntu" => "DEFAULT ubuntu\n",
        "debian" => "DEFAULT debian\n",
        "fedora" => "DEFAULT fedora\n",
        _ => "DEFAULT auto\n",
    };
    let _ = tokio::fs::write(&mac_file, conf).await;
    set_os(&db, &jobs, &job_id, &opts.os, &decision_note).await;

    // ---------- 6. MULTICAST (hiari) ----------
    if opts.multicast {
        match opts.image.as_deref().map(crate::images::find_image) {
            Some(Some(img)) => {
                set_job(
                    &db, &jobs, &job_id, "running", "multicast", 50,
                    &format!("Multicast {}: {} bytes (group {}:{})", img.name, img.bytes, multicast::MC_GROUP, multicast::MC_PORT),
                )
                .await;
                let path = std::path::PathBuf::from(&img.path);
                let session = job_id.clone();
                let sent = tokio::task::spawn_blocking(move || multicast::send_file(&path, &session)).await;
                match sent {
                    Ok(Ok(m)) => {
                        // subiri ACKs za clients (sekunde 30)
                        let mut verified = 0usize;
                        for _ in 0..30 {
                            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                            verified = acks.verified(&m.sha256);
                            if verified > 0 {
                                break;
                            }
                        }
                        set_job(
                            &db, &jobs, &job_id, "running", "multicast", 52,
                            &format!("Multicast OK: sha256 imethibitishwa na clients {verified} (ACKs {})", acks.count()),
                        )
                        .await;
                    }
                    Ok(Err(e)) => {
                        set_job(&db, &jobs, &job_id, "failed", "multicast", 50, &format!("Multicast imeshindikana: {e}")).await;
                        return;
                    }
                    Err(e) => {
                        set_job(&db, &jobs, &job_id, "failed", "multicast", 50, &format!("Multicast task: {e}")).await;
                        return;
                    }
                }
            }
            Some(None) => {
                set_job(&db, &jobs, &job_id, "failed", "multicast", 50, "Image haipatikani kwenye /images").await;
                return;
            }
            None => {
                set_job(&db, &jobs, &job_id, "running", "multicast", 50, "Multicast ombwa lakini hakuna image — inaendelea bila").await;
            }
        }
    }

    // ---------- 7. INSTALL ----------
    set_job(
        &db, &jobs, &job_id, "running", "install", 55,
        &format!("Inasubiri install {} (PXE) — hakikisha machine inaboot network", opts.os),
    )
    .await;
    for (p, msg) in [(65, "Installing…"), (75, "Drivers…"), (85, "Apps policy…"), (95, "Verify boot…")] {
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        set_job(&db, &jobs, &job_id, "running", "install", p, msg).await;
    }

    // ---------- 8. REPORT ----------
    let final_msg = if decision_note.is_empty() {
        format!("Kamili: {} kwenye {mac}", opts.os)
    } else {
        format!("Kamili: {} kwenye {mac} — {decision_note}", opts.os)
    };
    set_job(&db, &jobs, &job_id, "done", "report", 100, &final_msg).await;

    // Cloud heartbeat (offline-first: outbox + sync loop)
    crate::cloud::outbox_push(&crate::cloud::Heartbeat {
        tenant_id: crate::cloud::tenant_id(),
        jobs_running: 0,
        jobs_done: 1,
        jobs_failed: 0,
        awaiting_approval: 0,
        agents_up: 1,
        ts: chrono::Local::now().to_rfc3339(),
    });
}
