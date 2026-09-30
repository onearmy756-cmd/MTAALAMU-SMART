//! Discover hosts — halisi: TCP sweep + arp-scan + arp; lab: demo hosts.
//!
//! TCP sweep: ports zisizo na server (135, 445, 22, 3389) — PC inayojibu
//! (RST badala ya hakuna jibu) inaonekana "live". Kisha ARP inatoa MAC.
//! Matokeo yanakuunganishwa (merge) kwa IP.

use regex::Regex;
use serde::Serialize;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;

#[derive(Debug, Clone, Serialize)]
pub struct Host {
    pub mac: String,
    pub ip: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open_ports: Option<Vec<u16>>,
}

const PROBE_PORTS: [u16; 4] = [135, 445, 22, 3389];
const SWEEP_TIMEOUT_MS: u64 = 250;

pub fn discover_hosts() -> Vec<Host> {
    let mut merged: Vec<Host> = Vec::new();

    // 1. arp-scan (bora — inatoa MAC moja kwa moja)
    if let Ok(o) = std::process::Command::new("arp-scan")
        .args(["--localnet", "--quiet"])
        .output()
    {
        if o.status.success() {
            merged.extend(parse_arp_scan(&String::from_utf8_lossy(&o.stdout)));
        }
    }
    // 2. arp -a
    if let Ok(o) = std::process::Command::new("arp").args(["-a"]).output() {
        let hosts = parse_arp_a(&String::from_utf8_lossy(&o.stdout));
        merge_into(&mut merged, hosts);
    }
    // 3. TCP sweep (live hosts ambazo hazipo kwenye ARP cache bado)
    let sweep = tcp_sweep();
    merge_into(&mut merged, sweep);

    if merged.is_empty() && std::env::var("FUNDI_DEMO_HOSTS").unwrap_or_else(|_| "1".into()) != "0" {
        return demo_hosts();
    }
    merged.sort_by(|a, b| a.ip.cmp(&b.ip));
    merged
}

/// IP ya LAN + prefix (mfano 192.168.1.0/24) kutoka routing ya OS
fn local_subnet() -> Option<(Ipv4Addr, u8)> {
    // Linux: `ip route` → "default via 192.168.1.1 dev eth0 src 192.168.1.50"
    if let Ok(o) = std::process::Command::new("ip").args(["route"]).output() {
        let txt = String::from_utf8_lossy(&o.stdout);
        for line in txt.lines() {
            if line.starts_with("default") {
                if let Some(src) = line.split("src").nth(1).and_then(|s| s.split_whitespace().next()) {
                    if let Ok(ip) = src.parse::<Ipv4Addr>() {
                        return Some((ip, 24));
                    }
                }
            }
        }
    }
    // Windows: `ipconfig`
    if let Ok(o) = std::process::Command::new("ipconfig").output() {
        let txt = String::from_utf8_lossy(&o.stdout);
        let re = Regex::new(r"(?m)^\s*IPv4[^\d]*(\d+\.\d+\.\d+\.\d+)").ok()?;
        if let Some(m) = re.captures(&txt) {
            if let Ok(ip) = m[1].parse::<Ipv4Addr>() {
                return Some((ip, 24));
            }
        }
    }
    None
}

fn tcp_sweep() -> Vec<Host> {
    let Some((base, prefix)) = local_subnet() else { return Vec::new() };
    if std::env::var("FUNDI_SWEEP").unwrap_or_else(|_| "1".into()) == "0" {
        return Vec::new();
    }
    let octets = base.octets();
    let net = u32::from_be_bytes(octets) & 0xFFFFFF00; // /24
    let mut live: Vec<(String, Vec<u16>)> = Vec::new();

    let mut handles = Vec::new();
    for i in 1..255u32 {
        let ip = Ipv4Addr::from(((net + i).to_be_bytes()));
        let ip_str = ip.to_string();
        handles.push(std::thread::spawn(move || {
            let mut open = Vec::new();
            for port in PROBE_PORTS {
                let addr = SocketAddr::new(IpAddr::V4(ip), port);
                if std::net::TcpStream::connect_timeout(&addr, Duration::from_millis(SWEEP_TIMEOUT_MS)).is_ok() {
                    open.push(port);
                }
            }
            (ip_str, open)
        }));
        // kikomo cha threads kwa wigo
        if handles.len() >= 64 {
            for h in handles.drain(..) {
                if let Ok((ip, ports)) = h.join() {
                    if !ports.is_empty() {
                        live.push((ip, ports));
                    }
                }
            }
        }
    }
    for h in handles {
        if let Ok((ip, ports)) = h.join() {
            if !ports.is_empty() {
                live.push((ip, ports));
            }
        }
    }

    live.into_iter()
        .map(|(ip, ports)| Host {
            mac: String::new(),
            ip,
            name: format!("PC-{}", ip.rsplit('.').next().unwrap_or("?")),
            source: Some("tcp-sweep".into()),
            open_ports: Some(ports),
        })
        .collect()
}

fn merge_into(base: &mut Vec<Host>, extra: Vec<Host>) {
    for h in extra {
        if h.mac.is_empty() {
            if let Some(existing) = base.iter_mut().find(|x| x.ip == h.ip) {
                if existing.open_ports.is_none() {
                    existing.open_ports = h.open_ports.clone();
                }
                continue;
            }
        } else if let Some(existing) = base.iter_mut().find(|x| x.ip == h.ip || x.mac == h.mac) {
            if existing.mac.is_empty() {
                existing.mac = h.mac.clone();
            }
            if existing.open_ports.is_none() {
                existing.open_ports = h.open_ports.clone();
            }
            continue;
        }
        base.push(h);
    }
}

fn demo_hosts() -> Vec<Host> {
    vec![
        Host {
            mac: "aa:bb:cc:dd:ee:01".into(),
            ip: "192.168.1.101".into(),
            name: "LAB-PC-01".into(),
            source: Some("demo".into()),
            open_ports: None,
        },
        Host {
            mac: "aa:bb:cc:dd:ee:02".into(),
            ip: "192.168.1.102".into(),
            name: "LAB-PC-02".into(),
            source: Some("demo".into()),
            open_ports: None,
        },
        Host {
            mac: "aa:bb:cc:dd:ee:03".into(),
            ip: "192.168.1.103".into(),
            name: "LAB-LAPTOP-03".into(),
            source: Some("demo".into()),
            open_ports: None,
        },
    ]
}

fn parse_arp_scan(text: &str) -> Vec<Host> {
    let mut out = Vec::new();
    for line in text.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 && parts[1].contains(':') {
            out.push(Host {
                ip: parts[0].into(),
                mac: parts[1].to_lowercase(),
                name: format!("PC-{}", parts[1].replace(':', "")),
                source: Some("arp-scan".into()),
                open_ports: None,
            });
        }
    }
    out
}

fn parse_arp_a(text: &str) -> Vec<Host> {
    let re = Regex::new(r"(?i)(\d+\.\d+\.\d+\.\d+).*?([0-9a-f]{2}([-:])){5}[0-9a-f]{2}").ok();
    let mut out = Vec::new();
    let Some(re) = re else { return out };
    for line in text.lines() {
        if let Some(m) = re.find(line) {
            let s = m.as_str();
            let ip_re = Regex::new(r"\d+\.\d+\.\d+\.\d+").unwrap();
            let mac_re = Regex::new(r"(?i)([0-9a-f]{2}[:-]){5}[0-9a-f]{2}").unwrap();
            let ip = ip_re.find(s).map(|x| x.as_str().to_string()).unwrap_or_default();
            let mac = mac_re
                .find(s)
                .map(|x| x.as_str().to_lowercase().replace('-', ":"))
                .unwrap_or_default();
            if !mac.is_empty() {
                out.push(Host {
                    ip,
                    mac: mac.clone(),
                    name: format!("PC-{}", mac.replace(':', "")),
                    source: Some("arp".into()),
                    open_ports: None,
                });
            }
        }
    }
    out
}
