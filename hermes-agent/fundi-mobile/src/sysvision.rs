//! sysvision.rs — VISION HALISI ya PC (kama agent "vision/Muono" wa Fundi Deploy).
//!
//! FundiPro inahudumia wateja kwa mbali — kabla ya kumpendekeza huduma,
//! vision agent anachunguza PC HALISI (kama fundi angeona screen):
//!   - Hostname + OS + version
//!   - RAM (GB)
//!   - Disks: kiasi kilichotumika/huru (%)
//!   - Network adapters zilizo UP
//!   - Uptime / LastBoot (Linux) au LastBootUpTime (Windows)
//!
//! Hakuna uongo: kila value inatoka kwenye mfumo (wmic/ipconfig, /proc, df).
//! Windows: wmic + ipconfig. Linux: /proc + df + ip.

use serde_json::{json, Value};

fn run(cmd: &str, args: &[&str]) -> Option<String> {
    std::process::Command::new(cmd)
        .args(args)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
}

/// wmic CSV rows → Vec<(key, val)> (header + row moja au zaidi)
fn csv_rows(out: &str) -> Vec<Vec<(String, String)>> {
    let mut lines = out.lines().filter(|l| !l.trim().is_empty());
    let header: Vec<String> = lines
        .next()
        .map(|h| h.split(',').map(|s| s.trim().to_lowercase()).collect())
        .unwrap_or_default();
    let mut rows = Vec::new();
    for line in lines {
        let vals: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
        if vals.len() == header.len() {
            rows.push(header.iter().zip(vals.iter()).map(|(k, v)| (k.clone(), v.to_string())).collect());
        }
    }
    rows
}

fn hostname() -> String {
    std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "?".into())
}

#[cfg(target_os = "windows")]
fn os_info() -> (String, String) {
    if let Some(out) = run("wmic", &["os", "get", "Caption,Version", "/format:csv"]) {
        if let Some(row) = csv_rows(&out).into_iter().next() {
            let name = row.iter().find(|(k, _)| k == "caption").map(|(_, v)| v.clone()).unwrap_or_default();
            let ver = row.iter().find(|(k, _)| k == "version").map(|(_, v)| v.clone()).unwrap_or_default();
            return (name, ver);
        }
    }
    ("Windows".into(), "?".into())
}

#[cfg(not(target_os = "windows"))]
fn os_info() -> (String, String) {
    let mut name = "Linux".into();
    let mut ver = String::new();
    if let Ok(txt) = std::fs::read_to_string("/etc/os-release") {
        for line in txt.lines() {
            if let Some(v) = line.strip_prefix("PRETTY_NAME=") {
                name = v.trim_matches('"').to_string();
            }
        }
    }
    if let Ok(t) = std::fs::read_to_string("/proc/sys/kernel/osrelease") {
        ver = t.trim().to_string();
    }
    (name, ver)
}

#[cfg(target_os = "windows")]
fn ram_gb() -> u64 {
    if let Some(out) = run("wmic", &["computersystem", "get", "TotalPhysicalMemory", "/format:csv"]) {
        if let Some(row) = csv_rows(&out).into_iter().next() {
            if let Some((_, v)) = row.into_iter().find(|(k, _)| k == "totalphysicalmemory") {
                return v.parse::<u64>().map(|b| b / 1024 / 1024 / 1024).unwrap_or(0);
            }
        }
    }
    0
}

#[cfg(not(target_os = "windows"))]
fn ram_gb() -> u64 {
    if let Ok(txt) = std::fs::read_to_string("/proc/meminfo") {
        for line in txt.lines() {
            if let Some(rest) = line.strip_prefix("MemTotal:") {
                let kb: u64 = rest.split_whitespace().next().and_then(|s| s.parse().ok()).unwrap_or(0);
                return kb / 1024 / 1024;
            }
        }
    }
    0
}

#[cfg(target_os = "windows")]
fn disks() -> Vec<Value> {
    let mut out_v = Vec::new();
    if let Some(out) = run("wmic", &["logicaldisk", "get", "DeviceID,FreeSpace,Size", "/format:csv"]) {
        for row in csv_rows(&out) {
            let id = row.iter().find(|(k, _)| k == "deviceid").map(|(_, v)| v.clone()).unwrap_or_default();
            let free: u64 = row.iter().find(|(k, _)| k == "freespace").and_then(|(_, v)| v.parse().ok()).unwrap_or(0);
            let total: u64 = row.iter().find(|(k, _)| k == "size").and_then(|(_, v)| v.parse().ok()).unwrap_or(0);
            if total == 0 {
                continue;
            }
            let used_pct = if total > 0 { (total - free) * 100 / total } else { 0 };
            out_v.push(json!({
                "volume": id,
                "total_gb": total / 1024 / 1024 / 1024,
                "free_gb": free / 1024 / 1024 / 1024,
                "used_percent": used_pct,
            }));
        }
    }
    out_v
}

#[cfg(not(target_os = "windows"))]
fn disks() -> Vec<Value> {
    let mut out_v = Vec::new();
    if let Some(out) = run("df", &["-k", "-x", "tmpfs", "-x", "devtmpfs"]) {
        for line in out.lines().skip(1) {
            let it: Vec<&str> = line.split_whitespace().collect();
            if it.len() < 6 {
                continue;
            }
            let total_kb: u64 = it[1].parse().unwrap_or(0);
            let free_kb: u64 = it[3].parse().unwrap_or(0);
            if total_kb == 0 {
                continue;
            }
            out_v.push(json!({
                "volume": it[5],
                "total_gb": total_kb / 1024 / 1024,
                "free_gb": free_kb / 1024 / 1024,
                "used_percent": (total_kb - free_kb) * 100 / total_kb,
            }));
        }
    }
    out_v
}

#[cfg(target_os = "windows")]
fn adapters_up() -> Vec<String> {
    run("ipconfig", &[])
        .map(|out| {
            out.lines()
                .filter(|l| l.contains("adapter") && l.contains(':'))
                .map(|l| {
                    l.trim()
                        .trim_end_matches(':')
                        .split("adapter ")
                        .nth(1)
                        .unwrap_or(l)
                        .to_string()
                })
                .filter(|s| !s.to_lowercase().contains("disconnected"))
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(not(target_os = "windows"))]
fn adapters_up() -> Vec<String> {
    run("ip", &["-brief", "link", "show"])
        .map(|out| {
            out.lines()
                .skip(1)
                .filter_map(|l| {
                    let mut it = l.split_whitespace();
                    let name = it.next()?;
                    let state = it.next()?;
                    (state == "UP" && name != "lo").then(|| name.to_string())
                })
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(target_os = "windows")]
fn boot_info() -> String {
    run("wmic", &["os", "get", "LastBootUpTime", "/format:csv"])
        .and_then(|out| {
            csv_rows(&out)
                .into_iter()
                .next()?
                .into_iter()
                .find(|(k, _)| k == "lastbootuptime")
                .map(|(_, v)| v)
        })
        .unwrap_or_else(|| "?".into())
}

#[cfg(not(target_os = "windows"))]
fn boot_info() -> String {
    std::fs::read_to_string("/proc/uptime")
        .ok()
        .and_then(|t| t.split_whitespace().next().and_then(|s| s.parse::<f64>().ok()))
        .map(|secs| format!("uptime_hours: {:.0}", secs / 3600.0))
        .unwrap_or_else(|| "?".into())
}

/// Scan HALISI ya PC — evidence ya vision agent.
pub fn scan() -> Value {
    let disks = disks();
    // Findings halisi zinazotokana na scan:
    let mut findings: Vec<String> = Vec::new();
    for d in &disks {
        let used = d["used_percent"].as_u64().unwrap_or(0);
        let vol = d["volume"].as_str().unwrap_or("?");
        if used >= 90 {
            findings.push(format!("Disk {vol} imejaa ({used}% tumika) — backup/cleanup inahitajika"));
        } else if used >= 80 {
            findings.push(format!("Disk {vol} karibu kujaa ({used}%)"));
        }
    }
    let ram = ram_gb();
    if ram > 0 && ram < 4 {
        findings.push(format!("RAM ndogo ({ram} GB) — performance inaweza kuwa dhaifu"));
    }

    json!({
        "kind": "sysvision",
        "timestamp": chrono::Local::now().to_rfc3339(),
        "hostname": hostname(),
        "os": { "name": os_info().0, "version": os_info().1, "arch": std::env::consts::ARCH },
        "ram_gb": ram,
        "disks": disks,
        "adapters_up": adapters_up(),
        "boot": boot_info(),
        "findings": findings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan_ina_fields() {
        let v = scan();
        assert!(v.get("hostname").is_some());
        assert!(v.get("ram_gb").is_some());
        assert!(v.get("findings").unwrap().is_array());
    }
}
