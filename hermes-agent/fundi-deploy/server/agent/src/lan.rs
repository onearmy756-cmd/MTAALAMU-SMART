//! lan.rs — LAN REMOTE: kugundua hosts za LAN (TCP connect scan) + mwongozo wa remote.
//!
//! Hakuna uongo: matokeo ni ya TCP connect halisi. Ports zinazokaguliwa ni za
//! huduma za remote (RDP 3389, VNC 5900, SSH 22, SMB 445, WinRM 5985, RustDesk 21115-21116).
//! Mwongozo unatolewa kulingana na ports zilizofunguka — fundi anajua tool ipi itumie.

use serde_json::{json, Value};
use std::net::{IpAddr, Ipv4Addr};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Semaphore;

/// Ports za huduma za remote-tuma (mtoa huduma anatumia kuunganisha)
pub const REMOTE_PORTS: &[(u16, &str)] = &[
    (3389, "RDP"),
    (5900, "VNC"),
    (22, "SSH"),
    (445, "SMB"),
    (5985, "WinRM"),
    (21115, "RustDesk"),
    (21116, "RustDesk-ID"),
];

fn local_subnet() -> Vec<Ipv4Addr> {
    // Pata IP ya ndani kwa kuunganisha UDP socket (haitumi Trafiki)
    let ip = std::net::UdpSocket::bind("0.0.0.0:0")
        .and_then(|s| {
            s.connect("8.8.8.8:80")?;
            Ok(s.local_addr()?.ip())
        })
        .unwrap_or(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
    match ip {
        IpAddr::V4(v4) => {
            let o = v4.octets();
            let base = [o[0], o[1], o[2], 1];
            // Scan /24 (hosts 1-254)
            (1..=254)
                .map(|i| Ipv4Addr::new(base[0], base[1], base[2], i))
                .collect()
        }
        _ => Vec::new(),
    }
}

async fn probe_host(ip: Ipv4Addr, timeout_ms: u64) -> Vec<(u16, &'static str)> {
    let mut open = Vec::new();
    for (port, name) in REMOTE_PORTS {
        let fut = TcpStream::connect((std::net::IpAddr::V4(ip), *port));
        if tokio::time::timeout(std::time::Duration::from_millis(timeout_ms), fut)
            .await
            .map(|r| r.is_ok())
            .unwrap_or(false)
        {
            open.push((*port, *name));
        }
    }
    open
}

/// Scan /24 nzima kwa huduma za remote (parallel, kila host kwa wakati mmoja).
pub async fn scan_lan(timeout_ms: u64) -> Value {
    let hosts = local_subnet();
    let sem = Arc::new(Semaphore::new(64));
    let scanned = Arc::new(AtomicUsize::new(0));
    let mut handles = Vec::new();

    for ip in hosts {
        let permit = sem.clone().acquire_owned().await.unwrap();
        let scanned = scanned.clone();
        handles.push(tokio::spawn(async move {
            let open = probe_host(ip, timeout_ms).await;
            scanned.fetch_add(1, Ordering::Relaxed);
            drop(permit);
            let mut v = json!({
                "ip": ip.to_string(),
                "alive": !open.is_empty(),
                "services": [],
            });
            if !open.is_empty() {
                let names: Vec<String> = open.iter().map(|(p, n)| format!("{n}:{p}")).collect();
                v["services"] = json!(names);
                // Pendekezo la tool kulingana na port iliyofunguka
                v["recommend"] = json!(if open.iter().any(|(p, _)| *p == 3389) {
                    "RDP (mstsc /v:IP)"
                } else if open.iter().any(|(p, _)| [21115, 21116].contains(p)) {
                    "RustDesk (self-hosted)"
                } else if open.iter().any(|(p, _)| *p == 5900) {
                    "VNC viewer"
                } else if open.iter().any(|(p, _)| *p == 22) {
                    "SSH"
                } else {
                    "Ping tu — hakuna service ya remote"
                });
            }
            v
        }));
    }

    let mut results = Vec::new();
    for h in handles {
        if let Ok(v) = h.await {
            if v["alive"].as_bool().unwrap_or(false) {
                results.push(v);
            }
        }
    }
    json!({
        "kind": "lan-scan",
        "timestamp": chrono::Local::now().to_rfc3339(),
        "hosts_scanned": scanned.load(Ordering::Relaxed),
        "hosts_alive": results.len(),
        "hosts": results,
        "guide": {
            "RDP": "Windows Pro+: Settings → System → Remote Desktop → Enable; kisha mstsc /v:IP",
            "RustDesk": "Self-hosted server: docker run -e KEY=... rustdesk/rustdesk-server; clients zieleke kwenye server",
            "VNC": "Install TightVNC/RealVNC server kwenye PC ya mteja; viewer kutoka PC yako (port 5900)",
            "SSH": "Windows: Install OpenSSH Server (optional features); Linux: sudo apt install openssh-server",
            "security": "Tumia password NGUIO, VPN kwa mbali, na token/2FA pale inapowezekana.",
        }
    })
}

/// Mwongozo wa kina wa kuanzisha remote (kwa UI).
pub fn guide_sw() -> String {
    r#"🖥️  LAN REMOTE — mwongozo wa fundi
1. RDP (bure, Windows Pro+):
   PC ya mteja: Settings → System → Remote Desktop → On
   PC yako:     mstsc /v:192.168.1.11  (andika user/password ya mteja)
2. VNC (Windows Home / Linux):
   PC ya mteja: install TightVNC Server, weka password nguo
   PC yako:     VNC Viewer → 192.168.1.11:5900
3. RustDesk (self-hosted, bila cloud ya nje):
   Server: docker run --name hbbs -p 21115-21116:21115-21116 -d rustdesk/rustdesk-server
   Clients: weka server yako + key kwenye settings
4. AnyDesk (rahisi zaidi, kwa mbali):
   Wote install AnyDesk; mteja akupa ID + password ya kikao
USALAMA: password nguo, washa 2FA, VPN kwa ufikiaji wa mbali."#.to_string()
}
