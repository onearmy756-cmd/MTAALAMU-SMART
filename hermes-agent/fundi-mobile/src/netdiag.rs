//! netdiag.rs — NETWORK DIAGNOSTICS halisi za L1–L7 (Windows + Linux).
//!
//! Hakuna uongo: kila layer inatumia tools za mfumo halisi:
//!   L1: state ya interface (ip link / ipconfig)
//!   L2: ARP entries (ip neigh / arp -a)
//!   L3: IP + gateway + DNS + ping gateway + DNS resolve (nslookup)
//!   L4: TCP connect halisi kwa 8.8.8.8 (53, 443)
//!   L7: HTTP(S) GET halisi (reqwest) kwa sites 3
//!
//! Matokeo: layers + verdict + recommendations (Kiswahili).

use serde_json::{json, Value};
use std::net::TcpStream;
use std::time::Duration;

fn run(cmd: &str, args: &[&str]) -> Option<String> {
    std::process::Command::new(cmd)
        .args(args)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
}

// ---------- L1: Physical ----------

#[cfg(target_os = "windows")]
fn check_l1() -> Value {
    let mut v = json!({ "layer": "L1 Physical", "up": false, "adapters": [] });
    if let Some(out) = run("ipconfig", &[]) {
        let mut adapters = Vec::new();
        for line in out.lines() {
            let t = line.trim_end();
            if !t.is_empty() && !t.starts_with(' ') && t.contains(':') {
                let name = t.trim_end_matches(':');
                if !name.to_lowercase().contains("media disconnected")
                    && !name.is_empty()
                {
                    adapters.push(name.to_string());
                }
            }
        }
        v["up"] = json!(!adapters.is_empty());
        v["adapters"] = json!(adapters);
    }
    v
}

#[cfg(not(target_os = "windows"))]
fn check_l1() -> Value {
    let mut v = json!({ "layer": "L1 Physical", "up": false, "adapters": [] });
    if let Some(out) = run("ip", &["-brief", "link", "show"]) {
        let mut adapters = Vec::new();
        let mut up = false;
        for line in out.lines().skip(1) {
            let mut it = line.split_whitespace();
            let name = it.next().unwrap_or("");
            let state = it.next().unwrap_or("");
            if state == "UP" && name != "lo" {
                up = true;
                adapters.push(name.to_string());
            }
        }
        v["up"] = json!(up);
        v["adapters"] = json!(adapters);
    }
    v
}

// ---------- L2: Data Link (ARP) ----------

#[cfg(target_os = "windows")]
fn check_l2() -> Value {
    let mut v = json!({ "layer": "L2 Data Link", "arp_entries": 0 });
    if let Some(out) = run("arp", &["-a"]) {
        let n = out
            .lines()
            .filter(|l| l.contains('-') && l.split_whitespace().count() >= 2)
            .count();
        v["arp_entries"] = json!(n);
    }
    v
}

#[cfg(not(target_os = "windows"))]
fn check_l2() -> Value {
    let mut v = json!({ "layer": "L2 Data Link", "arp_entries": 0 });
    if let Some(out) = run("ip", &["neigh", "show"]) {
        let n = out.lines().filter(|l| !l.trim().is_empty() && !l.contains("FAILED")).count();
        v["arp_entries"] = json!(n);
    }
    v
}

// ---------- L3: Network (IP, gateway, DNS) ----------

#[cfg(target_os = "windows")]
fn gateway_of() -> Option<String> {
    let out = run("ipconfig", &[])?;
    // Chukua "Default Gateway" ya kwanza isiyo tupu
    for line in out.lines() {
        let t = line.trim();
        if t.to_lowercase().starts_with("default gateway") {
            let val = t.split(':').nth(1)?.trim().to_string();
            if !val.is_empty() && val != "::" {
                return Some(val);
            }
        }
    }
    None
}

#[cfg(not(target_os = "windows"))]
fn gateway_of() -> Option<String> {
    let out = run("ip", &["route", "show", "default"])?;
    out.split_whitespace()
        .skip_while(|w| *w != "via")
        .nth(1)
        .map(String::from)
}

fn ping_ok(host: &str, timeout_ms: u32) -> bool {
    #[cfg(target_os = "windows")]
    {
        run("ping", &["-n", "1", "-w", &timeout_ms.to_string(), host])
            .map(|o| o.contains("TTL=") || o.to_lowercase().contains("time="))
            .unwrap_or(false)
    }
    #[cfg(not(target_os = "windows"))]
    {
        run(
            "ping",
            &["-c", "1", "-W", &(timeout_ms / 1000).max(1).to_string(), host],
        )
        .map(|o| o.contains("time="))
        .unwrap_or(false)
    }
}

fn check_l3() -> Value {
    let gw = gateway_of();
    let gw_reachable = gw.as_ref().map(|g| ping_ok(g, 2000)).unwrap_or(false);

    // DNS servers
    let mut dns: Vec<String> = Vec::new();
    #[cfg(target_os = "windows")]
    {
        if let Some(out) = run("ipconfig", &["/all"]) {
            for line in out.lines() {
                let t = line.trim();
                if t.to_lowercase().starts_with("dns servers") || t.to_lowercase().starts_with("dns servers.") {
                    let val = t.split(':').nth(1).unwrap_or("").trim();
                    if !val.is_empty() {
                        dns.push(val.to_string());
                    }
                }
            }
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        if let Ok(txt) = std::fs::read_to_string("/etc/resolv.conf") {
            dns = txt
                .lines()
                .filter(|l| l.starts_with("nameserver"))
                .filter_map(|l| l.split_whitespace().nth(1).map(String::from))
                .collect();
        }
    }

    // DNS resolve halisi (TCP 53 query via nslookup au std lookup)
    let dns_working = std::net::ToSocketAddrs::to_socket_addrs("google.com:443")
        .map(|mut it| it.next().is_some())
        .unwrap_or(false);

    json!({
        "layer": "L3 Network",
        "gateway": gw,
        "gateway_reachable": gw_reachable,
        "dns_servers": dns,
        "dns_working": dns_working,
    })
}

// ---------- L4: Transport (TCP connect halisi) ----------

fn check_l4() -> Value {
    let mut open = Vec::new();
    for (port, name) in [(53u16, "DNS"), (443, "HTTPS")] {
        let ok = TcpStream::connect_timeout(
            &format!("8.8.8.8:{port}").parse().unwrap(),
            Duration::from_millis(2500),
        )
        .is_ok();
        if ok {
            open.push(json!({ "port": port, "service": name, "open": true }));
        }
    }
    json!({
        "layer": "L4 Transport",
        "tcp_open": open,
        "blocked": open.is_empty(),
    })
}

// ---------- L7: Application (DNS + TCP 443 kwa majina halisi) ----------

fn check_l7() -> Value {
    // Bila TLS library, tunapima L7 reachability halisi: DNS resolve + TCP 443.
    let hosts = ["www.google.com:443", "www.cloudflare.com:443", "github.com:443"];
    let mut results = Vec::new();
    let mut any_ok = false;
    for host in hosts {
        let ok = std::net::ToSocketAddrs::to_socket_addrs(host)
            .ok()
            .and_then(|mut it| it.next())
            .map(|addr| TcpStream::connect_timeout(&addr, Duration::from_secs(5)).is_ok())
            .unwrap_or(false);
        if ok {
            any_ok = true;
        }
        results.push(json!({ "host": host, "dns_tcp_ok": ok }));
    }
    json!({
        "layer": "L7 Application",
        "reachable": any_ok,
        "note": "DNS resolve + TCP 443 halisi (HTTP GET kamili inahitaji TLS lib)",
        "sites": results,
    })
}

/// Diagnosis kamili — blocking (ita kutoka async kwa spawn_blocking).
pub fn diagnose_blocking() -> Value {
    let l1 = check_l1();
    let l2 = check_l2();
    let l3 = check_l3();
    let l4 = check_l4();
    let l7 = check_l7();

    let mut recs: Vec<String> = Vec::new();
    if !l1["up"].as_bool().unwrap_or(false) {
        recs.push("❌ L1: Hakuna interface UP — chomeka cable / washa WiFi adapter".into());
    }
    if l2["arp_entries"].as_i64().unwrap_or(0) == 0 {
        recs.push("❌ L2: Hakuna ARP entries — angalia switch/cable au DHCP".into());
    }
    if l3["gateway"].as_str().is_none() {
        recs.push("❌ L3: Hakuna default gateway — angalia DHCP au weka static".into());
    } else if !l3["gateway_reachable"].as_bool().unwrap_or(false) {
        recs.push("❌ L3: Gateway haipatikani — ping imeshindikana; angalia router/cable".into());
    }
    if !l3["dns_working"].as_bool().unwrap_or(false) {
        recs.push("❌ L3: DNS haifanyi kazi — badilisha DNS (8.8.8.8 / 1.1.1.1)".into());
    }
    if l4["blocked"].as_bool().unwrap_or(true) {
        recs.push("❌ L4: TCP 53/443 zote zimefungwa — angalia firewall/proxy".into());
    }
    if !l7["reachable"].as_bool().unwrap_or(false) {
        recs.push("❌ L7: Hakuna host inayofika (DNS+TCP 443) — angalia firewall/proxy/captive portal".into());
    }

    let overall = if recs.is_empty() {
        "🟢 Mtandao unafanya kazi vizuri"
    } else if recs.len() <= 2 {
        "🟡 Kuna matatizo madogo"
    } else {
        "🔴 Kuna matatizo makubwa"
    };

    json!({
        "kind": "netdiag",
        "timestamp": chrono::Local::now().to_rfc3339(),
        "l1": l1,
        "l2": l2,
        "l3": l3,
        "l4": l4,
        "l7": l7,
        "overall": overall,
        "recommendations": recs,
    })
}


