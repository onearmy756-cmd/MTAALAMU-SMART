//! Discover hosts — arp-scan / arp

use regex::Regex;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Host {
    pub mac: String,
    pub ip: String,
    pub name: String,
}

pub fn discover_hosts() -> Vec<Host> {
    if let Ok(o) = std::process::Command::new("arp-scan")
        .args(["--localnet", "--quiet"])
        .output()
    {
        if o.status.success() {
            return parse_arp_scan(&String::from_utf8_lossy(&o.stdout));
        }
    }
    if let Ok(o) = std::process::Command::new("arp").args(["-a"]).output() {
        return parse_arp_a(&String::from_utf8_lossy(&o.stdout));
    }
    Vec::new()
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
                });
            }
        }
    }
    out
}
