//! updates.rs — UPDATES NA NOTIFICATIONS (SEHEMU 10):
//! "Kila unapofanya update, wateja watapokea version mpya iliyoboreshwa —
//!  watapata notification ili kuupdate."
//!
//! KANUNI: version ya mfumo hii ni ya Rust (CARGO_PKG_VERSION). Check ya
//! server ya updates inafanyika ONLINE pekee (mode.rs gate). Offline: hakuna
//! check — hakuna uongo.

use serde::Serialize;
use sqlx::SqlitePool;

pub const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const UPDATE_SERVER: &str = "https://updates.mbilinyitech.co.tz/latest.json";

#[derive(Debug, Clone, Serialize)]
pub struct UpdateInfo {
    pub current_version: String,
    pub latest_version: Option<String>,
    pub update_available: bool,
    pub notified: bool,
    pub checked_at: String,
    pub note_sw: String,
}

pub async fn init_tables(db: &SqlitePool) {
    let _ = sqlx::query(
        "CREATE TABLE IF NOT EXISTS update_state (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            notified_version TEXT NOT NULL DEFAULT '',
            checked_at TEXT NOT NULL DEFAULT ''
        )",
    )
    .execute(db)
    .await;
    let _ = sqlx::query("INSERT OR IGNORE INTO update_state (id, notified_version, checked_at) VALUES (1, '', '')")
        .execute(db)
        .await;
}

fn semver_parts(v: &str) -> (u64, u64, u64) {
    let p: Vec<u64> = v.split('.').filter_map(|x| x.parse().ok()).collect();
    (p.first().copied().unwrap_or(0), p.get(1).copied().unwrap_or(0), p.get(2).copied().unwrap_or(0))
}

pub fn newer(latest: &str, current: &str) -> bool {
    semver_parts(latest) > semver_parts(current)
}

/// Check halisi (ONLINE pekee): server ya updates ya Mbilinyi Tech.
async fn fetch_latest() -> Option<String> {
    let client = reqwest::Client::new();
    let v: serde_json::Value = client
        .get(UPDATE_SERVER)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .ok()?
        .json()
        .await
        .ok()?;
    v["version"].as_str().map(String::from)
}

/// Check + notify: online → inaangalia server; offline → hakuna check (hakuna uongo).
pub async fn check_and_notify(db: &SqlitePool, online: bool) -> UpdateInfo {
    let checked = chrono::Local::now().to_rfc3339();
    if !online {
        return UpdateInfo {
            current_version: CURRENT_VERSION.into(),
            latest_version: None,
            update_available: false,
            notified: false,
            checked_at: checked,
            note_sw: "Offline — update check imerukwa (washа online kwa ruhusa yako).".into(),
        };
    }
    let latest = fetch_latest().await;
    match latest {
        Some(ref lv) if newer(lv, CURRENT_VERSION) => {
            // Notification: inahifadhiwa (agent inaonyesha kwenye UI + ripoti kwa admin)
            sqlx::query("UPDATE update_state SET notified_version=?, checked_at=? WHERE id=1")
                .bind(lv)
                .bind(&checked)
                .execute(db)
                .await
                .ok();
            UpdateInfo {
                current_version: CURRENT_VERSION.into(),
                latest_version: Some(lv.clone()),
                update_available: true,
                notified: true,
                checked_at: checked,
                note_sw: format!("Version mpya {lv} ipo — wateja wamepewa notification ya kuupdate."),
            }
        }
        Some(lv) => UpdateInfo {
            current_version: CURRENT_VERSION.into(),
            latest_version: Some(lv),
            update_available: false,
            notified: false,
            checked_at: checked,
            note_sw: "Umepakua version ya hivi pishani.".into(),
        },
        None => UpdateInfo {
            current_version: CURRENT_VERSION.into(),
            latest_version: None,
            update_available: false,
            notified: false,
            checked_at: checked,
            note_sw: "Server ya updates haipatikani (hakuna uongo — jaribu baadaye).".into(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn semver_comparison() {
        assert!(newer("2.1.0", "2.0.9"));
        assert!(newer("1.2.0", "1.1.99"));
        assert!(!newer("1.1.0", "1.1.0"));
        assert!(!newer("1.0.9", "1.1.0"));
    }

    #[tokio::test]
    async fn offline_hakuna_check() {
        let db = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        init_tables(&db).await;
        let info = check_and_notify(&db, false).await;
        assert!(!info.update_available);
        assert!(!info.notified);
        assert!(info.note_sw.contains("Offline"));
        assert_eq!(info.current_version, CURRENT_VERSION);
    }
}
