//! admin.rs — ADMIN MKUU ANAGAWA KAZI KWA WASAIDIZI (SEHEMU 4.3) — Rust structs + SQLite.
//!
//! Mahitaji ya mmiliki: "admin mkuu atagawa kazi mwenyewe kwa wahusika"
//!   - Admin Mkuu anaunda task (PC + kazi + maelekezo) → anampelekea Msaidizi
//!   - Msaidizi anaona tasks zake → anafanya → anaandika ripoti
//!   - Kazi za High-risk (install/reboot) zinahitaji idhini (auth::Role gate)

use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub pc: String,          // display_name ya PC (hr, hr 1…)
    pub work: String,        // kazi: install / configure / scan / repair
    pub details_sw: String,  // maelekezo ya Kiswahili
    pub assigned_to: String, // username ya Msaidizi
    pub assigned_by: String, // admin mkuu
    pub status: String,      // assigned | in_progress | done | failed
    pub report: String,      // ripoti ya Msaidizi baada ya kazi
    pub created_at: String,
    pub finished_at: Option<String>,
}

pub async fn init_tables(db: &SqlitePool) {
    let _ = sqlx::query(
        "CREATE TABLE IF NOT EXISTS admin_tasks (
            id TEXT PRIMARY KEY,
            pc TEXT NOT NULL,
            work TEXT NOT NULL,
            details_sw TEXT NOT NULL,
            assigned_to TEXT NOT NULL,
            assigned_by TEXT NOT NULL,
            status TEXT NOT NULL,
            report TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL,
            finished_at TEXT
        )",
    )
    .execute(db)
    .await;
}

/// Admin Mkuu PEKEE anagawa kazi (gate ya auth::Role).
pub fn can_assign(role: &str) -> bool {
    role == "admin_mkuu"
}

pub async fn assign(
    db: &SqlitePool,
    pc: &str,
    work: &str,
    details_sw: &str,
    assigned_to: &str,
    assigned_by_role: &str,
) -> Result<Task, String> {
    if !can_assign(assigned_by_role) {
        return Err("Ni Admin Mkuu PEKEE anayegawa kazi".into());
    }
    if pc.trim().is_empty() || work.trim().is_empty() || assigned_to.trim().is_empty() {
        return Err("pc, work na assigned_to ni lazima".into());
    }
    let t = Task {
        id: format!("TASK-{}", chrono::Local::now().format("%Y%m%d%H%M%S%3f")),
        pc: pc.trim().into(),
        work: work.trim().into(),
        details_sw: details_sw.into(),
        assigned_to: assigned_to.trim().into(),
        assigned_by: "admin-mkuu".into(),
        status: "assigned".into(),
        report: String::new(),
        created_at: chrono::Local::now().to_rfc3339(),
        finished_at: None,
    };
    sqlx::query(
        "INSERT INTO admin_tasks (id, pc, work, details_sw, assigned_to, assigned_by, status, report, created_at) VALUES (?,?,?,?,?,?,?,?,?)",
    )
    .bind(&t.id)
    .bind(&t.pc)
    .bind(&t.work)
    .bind(&t.details_sw)
    .bind(&t.assigned_to)
    .bind(&t.assigned_by)
    .bind(&t.status)
    .bind("")
    .bind(&t.created_at)
    .execute(db)
    .await
    .map_err(|e| e.to_string())?;
    Ok(t)
}

pub async fn list_for(db: &SqlitePool, username: &str) -> Vec<Task> {
    sqlx::query_as::<_, (String, String, String, String, String, String, String)>(
        "SELECT id, pc, work, details_sw, assigned_to, status, report FROM admin_tasks WHERE assigned_to = ? ORDER BY created_at DESC",
    )
    .bind(username)
    .fetch_all(db)
    .await
    .unwrap_or_default()
    .into_iter()
    .map(|(id, pc, work, details_sw, assigned_to, status, report)| Task {
        id, pc, work, details_sw, assigned_to, status, report,
        assigned_by: String::new(),
        created_at: String::new(),
        finished_at: None,
    })
    .collect()
}

pub async fn list_all(db: &SqlitePool) -> Vec<Task> {
    let rows: Vec<(String, String, String, String, String, String, String)> = sqlx::query_as(
        "SELECT id, pc, work, assigned_to, status, report, created_at FROM admin_tasks ORDER BY created_at DESC",
    )
    .fetch_all(db)
    .await
    .unwrap_or_default();
    rows.into_iter()
        .map(|(id, pc, work, assigned_to, status, report, created_at)| Task {
            id, pc, work, assigned_to, status, report, created_at,
            details_sw: String::new(),
            assigned_by: String::new(),
            finished_at: None,
        })
        .collect()
}

/// Msaidizi anaandika ripoti + kubadilisha status (done/failed).
pub async fn complete(db: &SqlitePool, task_id: &str, status: &str, report: &str) -> Result<(), String> {
    if !matches!(status, "done" | "failed" | "in_progress") {
        return Err(format!("status '{status}' si sahihi (done|failed|in_progress)"));
    }
    let finished = if status == "in_progress" { None } else { Some(chrono::Local::now().to_rfc3339()) };
    let n = sqlx::query("UPDATE admin_tasks SET status=?, report=?, finished_at=? WHERE id=?")
        .bind(status)
        .bind(report)
        .bind(finished)
        .bind(task_id)
        .execute(db)
        .await
        .map_err(|e| e.to_string())?;
    if n.rows_affected() == 0 {
        return Err(format!("Task '{task_id}' haipo"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn admin_mkuu_anagawa_msaidizi_anaripoti() {
        let db = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        init_tables(&db).await;
        // Admin Mkuu anagawa
        let t = assign(&db, "hr", "install", "Sakinisha office bundle", "fundi", "admin_mkuu").await.unwrap();
        assert_eq!(t.status, "assigned");
        // Msaidizi hafanyi assign
        assert!(assign(&db, "hr", "install", "x", "fundi", "msaidizi").await.is_err());
        // Msaidizi anaona kazi yake
        let mine = list_for(&db, "fundi").await;
        assert_eq!(mine.len(), 1);
        assert_eq!(mine[0].work, "install");
        // Anamaliza + ripoti
        complete(&db, &t.id, "done", "Office imesakinishwa, chrome pia").await.unwrap();
        let mine = list_for(&db, "fundi").await;
        assert_eq!(mine[0].status, "done");
        assert!(mine[0].report.contains("chrome"));
    }

    #[tokio::test]
    async fn task_isiyopo_inakataliwa() {
        let db = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        init_tables(&db).await;
        assert!(complete(&db, "TASK-HAKUNA", "done", "x").await.is_err());
    }
}
