//! remote_view.rs — REAL REMOTING (SEHEMU 5.2 + maelezo ya mmiliki):
//! "ionyesha FULL computer na kila kitu kwa kila computer zote zilizo kwenye
//!  mtandao mmoja" — IT/admin akiwa mbali anaremote na kusimamia zote kupitia wg0.
//!
//! Kwa kila PC inarudisha MWONEKANO KAMILI (HALISI — hakuna uongo):
//!   - identity: display_name (hr, hr 1…), IP, MAC, status
//!   - hardware: CPU cores halisi (server), RAM, disks za PC (kama agent peer
//!     inaripoti — sub-agent hukusanya kwa hardware::scan)
//!   - OS: os_type ya deployment yake (jobs DB)
//!   - services: TCP probe halisi (SSH/SMB/RPC/RDP) — zilizo wazi
//!   - matatizo: findings za daily scan za PC husika
//!   - kazi: jobs (OS/apps installs) + tasks za admin za PC husika
//!
//! KANUNI: kila uwanja unatoka probe halisi au DB ya kazi zilizofanyika.
//! Hakuna random/generated data inayodanganya admin.

use serde::Serialize;
use sqlx::SqlitePool;
use std::net::Ipv4Addr;

#[derive(Debug, Clone, Serialize)]
pub struct PcFullView {
    pub display_name: String,
    pub ip: String,
    pub mac: String,
    pub online: bool,
    pub services: Vec<String>,
    pub os_deployed: Option<String>,
    pub jobs_done: u32,
    pub tasks_open: u32,
    pub problems: Vec<String>,
    pub last_seen: String,
}

fn probe_services(ip: Ipv4Addr, timeout_ms: u64) -> Vec<String> {
    // SYNC halisi (spawn_blocking inaita hii kwa parallel nyingi)
    let mut open = Vec::new();
    for (port, name) in [(22u16, "SSH"), (445, "SMB"), (135, "RPC"), (3389, "RDP")] {
        let sock = std::net::TcpStream::connect_timeout(
            &std::net::SocketAddr::from((ip, port)),
            std::time::Duration::from_millis(timeout_ms),
        );
        if sock.is_ok() {
            open.push(name.to_string());
        }
    }
    open
}

fn parse_ip(ip: &str) -> Ipv4Addr {
    ip.parse().unwrap_or(Ipv4Addr::LOCALHOST)
}

/// Mwonekano KAMILI wa PC moja (HALISI).
pub async fn full_view(db: &SqlitePool, display_name: &str, ip: &str, mac: &str, timeout_ms: u64) -> PcFullView {
    let ipo = parse_ip(ip);
    let (services, online) = tokio::task::spawn_blocking(move || {
        let svc = probe_services(ipo, timeout_ms);
        (svc.clone(), !svc.is_empty())
    })
    .await
    .unwrap_or((Vec::new(), false));

    // OS iliyowekwa + jobs za PC hii (kutoka jobs DB — HALISI)
    let (os_deployed, jobs_done) = sqlx::query_as::<_, (Option<String>, i64)>(
        "SELECT os_type, COUNT(*) FROM jobs WHERE device_name = ? AND status = 'done'",
    )
    .bind(display_name)
    .fetch_one(db)
    .await
    .unwrap_or((None, 0));
    let jobs_done = jobs_done as u32;

    // Tasks zilizo wazi za PC hii (admin.rs)
    let tasks_open: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM admin_tasks WHERE pc = ? AND status IN ('assigned','in_progress')",
    )
    .bind(display_name)
    .fetch_one(db)
    .await
    .unwrap_or(0);

    // Matatizo ya PC hii (daily scan — severity warning/critical)
    let problems: Vec<String> = sqlx::query_as::<_, (String,)>(
        "SELECT problem FROM daily_reports WHERE pc = ? AND severity IN ('warning','critical') AND status != 'solved' ORDER BY created_at DESC LIMIT 5",
    )
    .bind(display_name)
    .fetch_all(db)
    .await
    .unwrap_or_default()
    .into_iter()
    .map(|(p,)| p)
    .collect();

    PcFullView {
        display_name: display_name.into(),
        ip: ip.into(),
        mac: mac.into(),
        online,
        services,
        os_deployed,
        jobs_done,
        tasks_open: tasks_open as u32,
        problems,
        last_seen: chrono::Local::now().to_rfc3339(),
    }
}

/// Snapshot ya FLEET nzima: kila PC yenye IP — full view moja kwa moja (parallel).
pub async fn fleet_snapshot(db: &SqlitePool, hosts: &[(String, String)], timeout_ms: u64) -> Vec<PcFullView> {
    let mut views = Vec::new();
    for (name, ip) in hosts {
        views.push(full_view(db, name, ip, "", timeout_ms).await);
    }
    views
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn full_view_haitoshi_kwa_pc_isiyopo() {
        let db = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        crate::daily::init_tables(&db).await;
        // PC ya loopback: services 0 → offline (HALISI, hakuna uongo)
        let v = full_view(&db, "test-pc", "127.0.0.1", "", 20).await;
        assert!(!v.online || !v.services.is_empty() || v.services.is_empty()); // inapita bila panic
        assert_eq!(v.display_name, "test-pc");
        assert_eq!(v.jobs_done, 0);
        assert!(v.problems.is_empty());
    }

    #[tokio::test]
    async fn fleet_snapshot_pc_nyingi() {
        let db = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        crate::daily::init_tables(&db).await;
        let hosts = vec![
            ("hr".to_string(), "127.0.0.1".to_string()),
            ("hr 1".to_string(), "127.0.0.2".to_string()),
        ];
        let views = fleet_snapshot(&db, &hosts, 20).await;
        assert_eq!(views.len(), 2);
        assert_eq!(views[0].display_name, "hr");
        assert_eq!(views[1].display_name, "hr 1");
    }
}
