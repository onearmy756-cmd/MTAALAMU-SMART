//! Discover hosts — arp-scan / arp + lab demo fallback
//! Agent inagundua; msimamizi anachagua targets pekee.

use regex::Regex;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Host {
    pub mac: String,
    pub ip: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

pub fn discover_hosts() -> Vec<Host> {
    if let Ok(o) = std::process::Command::new("arp-scan")
        .args(["--localnet", "--quiet"])
        .output()
    {
        if o.status.success() {
            let hosts = parse_arp_scan(&String::from_utf8_lossy(&o.stdout));
            if !hosts.is_empty() {
                return hosts;
            }
        }
    }
    if let Ok(o) = std::process::Command::new("arp").args(["-a"]).output() {
        let hosts = parse_arp_a(&String::from_utf8_lossy(&o.stdout));
        if !hosts.is_empty() {
            return hosts;
        }
    }
    // Lab / demo: empty LAN → synthetic hosts so supervisor can test HITL flow
    if std::env::var("FUNDI_DEMO_HOSTS").unwrap_or_else(|_| "1".into()) != "0" {
        return demo_hosts();
    }
    Vec::new()
}

fn demo_hosts() -> Vec<Host> {
    vec![
        Host {
            mac: "aa:bb:cc:dd:ee:01".into(),
            ip: "192.168.1.101".into(),
            name: "LAB-PC-01".into(),
            source: Some("demo".into()),
        },
        Host {
            mac: "aa:bb:cc:dd:ee:02".into(),
            ip: "192.168.1.102".into(),
            name: "LAB-PC-02".into(),
            source: Some("demo".into()),
        },
        Host {
            mac: "aa:bb:cc:dd:ee:03".into(),
            ip: "192.168.1.103".into(),
            name: "LAB-LAPTOP-03".into(),
            source: Some("demo".into()),
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
                });
            }
        }
    }
    out
}
