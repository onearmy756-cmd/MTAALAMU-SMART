//! Cloud multi-tenant (P3) — seva za Fundi nyingi, dashboard moja.
//!
//! Muundo:
//!   - Tenant = fundi server moja (site ya mteja) yenye `tenant_id` + `token`
//!   - Agent hii inaweza kuwa **cloud hub** (inapokea heartbeats) au **edge node**
//!     (inatuma heartbeats kwa hub kupitia FUNDI_CLOUD_URL)
//!   - Offline-first: heartbeats zinaandikwa kwenye outbox (`/data/cloud_outbox.jsonl`)
//!     na zinatumwa pale hub inaporejea (retry loop)
//!   - Data: jobs summaries tu (hakuna data nyeti ya mteja bila idhini)
//!
//! Tables (SQLite): tenants(id, name, token_hash, created_at)
//!                  cloud_events(id, tenant_id, kind, payload, created_at)

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tenant {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing)]
    pub token: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Heartbeat {
    pub tenant_id: String,
    pub jobs_running: u32,
    pub jobs_done: u32,
    pub jobs_failed: u32,
    pub awaiting_approval: u32,
    pub agents_up: u32,
    pub ts: String,
}

pub fn tenant_id() -> String {
    std::env::var("FUNDI_TENANT_ID").unwrap_or_else(|_| "local".into())
}

pub fn cloud_url() -> Option<String> {
    let u = std::env::var("FUNDI_CLOUD_URL").unwrap_or_default();
    if u.is_empty() || u == "disabled" {
        None
    } else {
        Some(u)
    }
}

fn outbox_path() -> PathBuf {
    PathBuf::from(std::env::var("FUNDI_OUTBOX").unwrap_or_else(|_| "/data/cloud_outbox.jsonl".into()))
}

pub fn sha256_hex(s: &str) -> String {
    let mut h = Sha256::new();
    h.update(s.as_bytes());
    format!("{:x}", h.finalize())
}

// ---------- DB ----------

pub async fn init_tables(db: &SqlitePool) {
    let _ = sqlx::query(
        "CREATE TABLE IF NOT EXISTS tenants (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            token_hash TEXT NOT NULL,
            created_at TEXT NOT NULL
        )",
    )
    .execute(db)
    .await;
    let _ = sqlx::query(
        "CREATE TABLE IF NOT EXISTS cloud_events (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            tenant_id TEXT NOT NULL,
            kind TEXT NOT NULL,
            payload TEXT NOT NULL,
            created_at TEXT NOT NULL
        )",
    )
    .execute(db)
    .await;
}

pub async fn upsert_tenant(db: &SqlitePool, id: &str, name: &str, token: &str) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO tenants (id, name, token_hash, created_at) VALUES (?,?,?,?)
         ON CONFLICT(id) DO UPDATE SET name=excluded.name, token_hash=excluded.token_hash",
    )
    .bind(id)
    .bind(name)
    .bind(sha256_hex(token))
    .bind(chrono::Local::now().to_rfc3339())
    .execute(db)
    .await?;
    Ok(())
}

pub async fn verify_tenant(db: &SqlitePool, id: &str, token: &str) -> bool {
    let row: Option<(String,)> =
        sqlx::query_as("SELECT token_hash FROM tenants WHERE id = ?")
            .bind(id)
            .fetch_optional(db)
            .await
            .unwrap_or(None);
    matches!(row, Some((h,)) if h == sha256_hex(token))
}

pub async fn list_tenants(db: &SqlitePool) -> Vec<Tenant> {
    let rows: Vec<(String, String, String)> =
        sqlx::query_as("SELECT id, name, created_at FROM tenants ORDER BY created_at")
            .fetch_all(db)
            .await
            .unwrap_or_default();
    rows.into_iter()
        .map(|(id, name, created_at)| Tenant { id, name, token: String::new(), created_at })
        .collect()
}

// ---------- Outbox (offline-first) ----------

pub fn outbox_push(hb: &Heartbeat) {
    use std::io::Write;
    if let Some(dir) = outbox_path().parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(outbox_path()) {
        if let Ok(line) = serde_json::to_string(hb) {
            let _ = writeln!(f, "{line}");
        }
    }
}

pub fn outbox_len() -> usize {
    std::fs::read_to_string(outbox_path())
        .map(|s| s.lines().count())
        .unwrap_or(0)
}

/// Tuma outbox yote kwa hub — inaitwa na retry loop ya nyuma
pub async fn outbox_flush() -> usize {
    let Some(url) = cloud_url() else { return 0 };
    let Ok(txt) = std::fs::read_to_string(outbox_path()) else { return 0 };
    let mut sent = 0usize;
    let mut keep = Vec::new();
    for line in txt.lines().filter(|l| !l.trim().is_empty()) {
        let client = reqwest::Client::new();
        match client
            .post(format!("{url}/cloud/heartbeat"))
            .json(&serde_json::json!({
                "tenant_id": tenant_id(),
                "events": [line]
            }))
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
        {
            Ok(r) if r.status().is_success() => sent += 1,
            _ => keep.push(line.to_string()),
        }
    }
    if sent > 0 {
        let _ = std::fs::write(
            outbox_path(),
            keep.join("\n") + if keep.is_empty() { "" } else { "\n" },
        );
    }
    sent
}

/// Loop ya nyuma: flush outbox kila dakika
pub async fn start_sync_loop() {
    tokio::spawn(async move {
        loop {
            if cloud_url().is_some() {
                let n = outbox_flush().await;
                if n > 0 {
                    tracing::info!("cloud: outbox {n} events zimetumwa");
                }
            }
            tokio::time::sleep(std::time::Duration::from_secs(60)).await;
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heartbeat_roundtrip() {
        let hb = Heartbeat {
            tenant_id: "t1".into(),
            jobs_running: 2,
            jobs_done: 5,
            jobs_failed: 0,
            awaiting_approval: 1,
            agents_up: 3,
            ts: "2026-09-30T00:00:00Z".into(),
        };
        let line = serde_json::to_string(&hb).unwrap();
        let back: Heartbeat = serde_json::from_str(&line).unwrap();
        assert_eq!(back.tenant_id, "t1");
        assert_eq!(back.awaiting_approval, 1);
    }
}
