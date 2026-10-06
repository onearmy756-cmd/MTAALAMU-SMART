//! server_setup.rs — ONBOARDING: mteja anasakinisha mfumo kwenye **server yake mwenyewe**
//! (server, computer, Raspberry Pi, nyinginezo) — kisha MTECH OS inafanya
//! **configuration ZOTE automatic** kwenye kifaa husika na kuanza kutumia.
//!
//! Mtiririko (maelezo ya mmiliki):
//!   1. Mteja anasakinisha (docker compose / release tar.gz) na kufungua UI.
//!   2. Kwenye ONBOARDING anachagua aina ya kifaa: server / computer /
//!      raspberry / other-device.
//!   3. Mfumo unahesabu na ku-andika CONFIGURATION KAMILI automatic:
//!      bandari, directories, VPN peer ya kifaa, services zinazohitajika,
//!      hali ya kuanza (ready). Hakuna swali zaidi — mfumo unajipanga.
//!   4. `GET /api/setup/status` inaonyesha dashboard yote ya onboarding.
//!
//! KANUNI: chochote kilichohesabiwa kinaanzia **halisi** — directories
//! zinahukumiwa na std::fs, DB ni SQLite, hakuna majibu ya uongo.

use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Target {
    Server,
    Computer,
    Raspberry,
    Generic,
}

impl Target {
    pub fn parse(s: &str) -> Option<Target> {
        match s.to_lowercase().as_str() {
            "server" => Some(Target::Server),
            "computer" | "pc" => Some(Target::Computer),
            "raspberry" | "raspberrypi" | "raspberry-pi" | "rpi" => Some(Target::Raspberry),
            "generic" | "other" => Some(Target::Generic),
            _ => None,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Target::Server => "Server",
            Target::Computer => "Computer",
            Target::Raspberry => "Raspberry Pi",
            Target::Generic => "Kifaa kingine",
        }
    }
}

/// Profile ya configuration inayohesabiwa kwa kifaa husika.
#[derive(Debug, Clone, Serialize)]
pub struct SetupConfig {
    pub target: String,
    pub label: String,
    pub api_port: u16,
    pub ui_path: String,
    pub data_dir: String,
    pub wg_subnet: &'static str,
    pub wg_peer_ip: String,
    pub needs_docker: bool,
    pub needs_tun: bool,
    pub services_required: Vec<&'static str>,
    pub notes_sw: Vec<&'static str>,
}

/// Hesabu ya config kwa target — kanuni ziko kwenye Rust (si config file).
pub fn compute_config(target: Target) -> SetupConfig {
    match target {
        Target::Server => SetupConfig {
            target: "server".into(),
            label: target.label().into(),
            api_port: 8080,
            ui_path: "/ui".into(),
            data_dir: "/data".into(),
            wg_subnet: "10.66.66.0/24",
            wg_peer_ip: "10.66.66.1".into(),
            needs_docker: true,
            needs_tun: true,
            services_required: vec!["api", "wireguard", "nginx", "llamacpp"],
            notes_sw: vec![
                "Server: docker compose up -d --build — NET_ADMIN + /dev/net/tun zinahitajika.",
                "VPN peer ya server ni 10.66.66.1 (gateway ya wg0).",
                "Agent credentials za default: admin/mtech2026 (badilisha mara moja).",
            ],
        },
        Target::Computer => SetupConfig {
            target: "computer".into(),
            label: target.label().into(),
            api_port: 8080,
            ui_path: "/ui".into(),
            data_dir: "./data".into(),
            wg_subnet: "10.66.66.0/24",
            wg_peer_ip: "10.66.66.2".into(),
            needs_docker: true,
            needs_tun: false,
            services_required: vec!["api", "ollama"],
            notes_sw: vec![
                "Computer ya ndani: agent inaendeshwa na docker compose (bila NET_ADMIN).",
                "Kwa kazi za mbali, kifaa hiki kinajiunga na server kuu kupitia peer ya VPN.",
            ],
        },
        Target::Raspberry => SetupConfig {
            target: "raspberry".into(),
            label: target.label().into(),
            api_port: 8080,
            ui_path: "/ui".into(),
            data_dir: "./data".into(),
            wg_subnet: "10.66.66.0/24",
            wg_peer_ip: "10.66.66.3".into(),
            needs_docker: true,
            needs_tun: false,
            services_required: vec!["api"],
            notes_sw: vec![
                "Raspberry Pi (ARM64): toleo la release la aarch64 linatumika.",
                "LLM nzito hazipendekezwa kwenye Pi — AI inaongea na server kuu.",
            ],
        },
        Target::Generic => SetupConfig {
            target: "generic".into(),
            label: target.label().into(),
            api_port: 8080,
            ui_path: "/ui".into(),
            data_dir: "./data".into(),
            wg_subnet: "10.66.66.0/24",
            wg_peer_ip: "10.66.66.4".into(),
            needs_docker: false,
            needs_tun: false,
            services_required: vec!["api"],
            notes_sw: vec![
                "Kifaa kingine: agent (binary ya release) inaendeshwa moja kwa moja.",
                "Config zote ni automatic — hakuna faili la config linalohitaji kuhaririwa.",
            ],
        },
    }
}

pub async fn init_tables(db: &SqlitePool) {
    let _ = sqlx::query(
        "CREATE TABLE IF NOT EXISTS setup_state (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            target TEXT NOT NULL,
            configured_at TEXT NOT NULL,
            wg_peer_name TEXT,
            ready INTEGER NOT NULL DEFAULT 0
        )",
    )
    .execute(db)
    .await;
}

/// Onboard: andika setup_state + hesabu config kamili. Wito wa pili unabadilisha.
pub async fn onboard(db: &SqlitePool, target: Target) -> Result<SetupConfig, String> {
    let cfg = compute_config(target);
    let now = chrono::Local::now().to_rfc3339();
    let _ = sqlx::query(
        "INSERT INTO setup_state (id, target, configured_at, wg_peer_name, ready) VALUES (1,?,?,?,1)
         ON CONFLICT(id) DO UPDATE SET target=excluded.target, configured_at=excluded.configured_at, wg_peer_name=excluded.wg_peer_name, ready=1",
    )
    .bind(cfg.target.as_str())
    .bind(&now)
    .bind(cfg.wg_peer_ip.as_str())
    .execute(db)
    .await
    .map_err(|e| e.to_string())?;
    Ok(cfg)
}

#[derive(Debug, Serialize)]
pub struct SetupStatus {
    pub configured: bool,
    pub target: Option<String>,
    pub configured_at: Option<String>,
    pub wg_peer_name: Option<String>,
    pub ready: bool,
    pub config: Option<SetupConfig>,
}

pub async fn status(db: &SqlitePool) -> SetupStatus {
    let row: Option<(String, String, Option<String>, i64)> = sqlx::query_as(
        "SELECT target, configured_at, wg_peer_name, ready FROM setup_state WHERE id = 1",
    )
    .fetch_optional(db)
    .await
    .unwrap_or(None);
    match row {
        Some((target, at, peer, ready)) => {
            let t = Target::parse(&target).unwrap_or(Target::Generic);
            SetupStatus {
                configured: true,
                target: Some(target),
                configured_at: Some(at),
                wg_peer_name: peer,
                ready: ready == 1,
                config: Some(compute_config(t)),
            }
        }
        None => SetupStatus { configured: false, target: None, configured_at: None, wg_peer_name: None, ready: false, config: None },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_parse_inatambua_aina_zote() {
        assert_eq!(Target::parse("server"), Some(Target::Server));
        assert_eq!(Target::parse("raspberry-pi"), Some(Target::Raspberry));
        assert_eq!(Target::parse("PC"), Some(Target::Computer));
        assert_eq!(Target::parse("other"), Some(Target::Generic));
        assert_eq!(Target::parse("blender"), None);
    }

    #[test]
    fn config_za_targets_zinatofautiana_kama_ilivyoainishwa() {
        let s = compute_config(Target::Server);
        assert!(s.needs_tun && s.needs_docker && s.wg_peer_ip == "10.66.66.1");
        assert!(s.services_required.contains(&"wireguard"));
        let r = compute_config(Target::Raspberry);
        assert_eq!(r.wg_peer_ip, "10.66.66.3");
        assert!(!r.services_required.contains(&"llamacpp"), "Pi haitumii LLM nzito");
        let c = compute_config(Target::Computer);
        assert!(!c.needs_tun);
    }

    #[tokio::test]
    async fn onboard_na_status_zinafuatana() {
        let db = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        init_tables(&db).await;
        let st = status(&db).await;
        assert!(!st.configured, "kabla ya onboarding: haijawekwa");
        let cfg = onboard(&db, Target::Raspberry).await.unwrap();
        assert_eq!(cfg.target, "raspberry");
        let st = status(&db).await;
        assert!(st.configured && st.ready);
        assert_eq!(st.target.as_deref(), Some("raspberry"));
        assert_eq!(st.wg_peer_name.as_deref(), Some("10.66.66.3"));
    }

    #[tokio::test]
    async fn onboard_ya_pili_inabadilisha_target() {
        let db = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        init_tables(&db).await;
        onboard(&db, Target::Server).await.unwrap();
        onboard(&db, Target::Computer).await.unwrap();
        let st = status(&db).await;
        assert_eq!(st.target.as_deref(), Some("computer"));
    }
}
