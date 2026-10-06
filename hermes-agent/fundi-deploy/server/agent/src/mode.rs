//! mode.rs — OFFLINE/ONLINE TOGGLE (SEHEMU 7) — mtu anaamua yeye.
//!
//! Offline: scan/solve/ripoti za ndani zinaendelea (hakuna mtandao unahitajika).
//! Online (kwa ruhusa ya mtu): VPN work, AI, update checks.
//! State inahifadhiwa SQLite — inaendelea hata baada ya restart.

use serde::Serialize;
use sqlx::SqlitePool;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Mode {
    Offline,
    Online,
}

impl Mode {
    pub fn id(&self) -> &'static str {
        match self {
            Mode::Offline => "offline",
            Mode::Online => "online",
        }
    }

    pub fn allows_vpn_remote(&self) -> bool {
        matches!(self, Mode::Online)
    }

    pub fn allows_ai_cloud(&self) -> bool {
        matches!(self, Mode::Online)
    }

    pub fn allows_update_check(&self) -> bool {
        matches!(self, Mode::Online)
    }
}

pub async fn init_tables(db: &SqlitePool) {
    let _ = sqlx::query(
        "CREATE TABLE IF NOT EXISTS system_mode (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            mode TEXT NOT NULL,
            changed_at TEXT NOT NULL
        )",
    )
    .execute(db)
    .await;
    // Default: offline (usalama — online inahitaji uamuzi wa mtu)
    let _ = sqlx::query(
        "INSERT OR IGNORE INTO system_mode (id, mode, changed_at) VALUES (1, 'offline', ?)",
    )
    .bind(chrono::Local::now().to_rfc3339())
    .execute(db)
    .await;
}

pub async fn get_mode(db: &SqlitePool) -> Mode {
    let row: Option<(String,)> = sqlx::query_as("SELECT mode FROM system_mode WHERE id = 1")
        .fetch_optional(db)
        .await
        .unwrap_or(None);
    match row {
        Some((m,)) if m == "online" => Mode::Online,
        _ => Mode::Offline,
    }
}

pub async fn set_mode(db: &SqlitePool, mode: Mode) -> Mode {
    let _ = sqlx::query("UPDATE system_mode SET mode=?, changed_at=? WHERE id=1")
        .bind(mode.id())
        .bind(chrono::Local::now().to_rfc3339())
        .execute(db)
        .await;
    mode
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn default_offline_na_toggle() {
        let db = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        init_tables(&db).await;
        // Default: offline (usalama) — hakuna VPN remote/AI cloud
        assert_eq!(get_mode(&db).await, Mode::Offline);
        assert!(!get_mode(&db).await.allows_vpn_remote());
        assert!(!get_mode(&db).await.allows_ai_cloud());
        // Mtu anaamua online
        let m = set_mode(&db, Mode::Online).await;
        assert_eq!(m, Mode::Online);
        assert!(m.allows_vpn_remote());
        assert!(m.allows_ai_cloud());
        assert!(m.allows_update_check());
        // Rudi offline
        assert_eq!(set_mode(&db, Mode::Offline).await, Mode::Offline);
    }
}
