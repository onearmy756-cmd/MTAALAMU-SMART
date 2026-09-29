//! Deep OS observability — production-grade (si bandia)
//! Linux: /proc, /sys | cross-platform: sysinfo
//! Hii SI kamera ya motherboard; ni metrics + topology za kweli za OS.

use serde::Serialize;
use std::fs;
use std::path::Path;
use sysinfo::{Disks, Networks, ProcessesToUpdate, System};

#[derive(Debug, Clone, Serialize)]
pub struct ProcNode {
    pub pid: u32,
    pub ppid: u32,
    pub name: String,
    pub cpu: f64,
    pub ram_mb: f64,
    pub status: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct NetSocket {
    pub local: String,
    pub remote: String,
    pub state: String,
    pub pid: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ThermalZone {
    pub name: String,
    pub temp_c: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeepSnapshot {
    pub timestamp: u64,
    pub hostname: String,
    pub os: String,
    pub kernel: String,
    pub cpu_usage_pct: f64,
    pub cpu_count: usize,
    pub cpu_brand: String,
    pub ram_usage_pct: f64,
    pub ram_used_mb: f64,
    pub ram_total_mb: f64,
    pub load_avg: [f64; 3],
    pub uptime_sec: u64,
    pub processes: Vec<ProcNode>,
    pub sockets: Vec<NetSocket>,
    pub thermals: Vec<ThermalZone>,
    pub disks: Vec<serde_json::Value>,
    pub nets: Vec<serde_json::Value>,
    pub issues: Vec<String>,
    pub health: String,
    pub capabilities: Vec<&'static str>,
}

fn ts() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn status_pct(p: f64) -> &'static str {
    if p >= 90.0 {
        "critical"
    } else if p >= 75.0 {
        "warning"
    } else {
        "good"
    }
}

/// Linux thermal zones from /sys/class/thermal
fn linux_thermals() -> Vec<ThermalZone> {
    let mut out = Vec::new();
    let root = Path::new("/sys/class/thermal");
    if !root.exists() {
        return out;
    }
    if let Ok(entries) = fs::read_dir(root) {
        for e in entries.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            if !name.starts_with("thermal_zone") {
                continue;
            }
            let temp_path = e.path().join("temp");
            let type_path = e.path().join("type");
            let tname = fs::read_to_string(&type_path)
                .unwrap_or(name.clone())
                .trim()
                .to_string();
            let temp_c = fs::read_to_string(&temp_path)
                .ok()
                .and_then(|s| s.trim().parse::<f64>().ok())
                .map(|m| m / 1000.0);
            out.push(ThermalZone {
                name: tname,
                temp_c,
            });
        }
    }
    out
}

/// Best-effort TCP sockets (Linux /proc/net/tcp — partial decode)
fn linux_sockets_sample(limit: usize) -> Vec<NetSocket> {
    let mut out = Vec::new();
    let path = Path::new("/proc/net/tcp");
    let Ok(text) = fs::read_to_string(path) else {
        return out;
    };
    for (i, line) in text.lines().enumerate() {
        if i == 0 {
            continue;
        }
        if out.len() >= limit {
            break;
        }
        let cols: Vec<&str> = line.split_whitespace().collect();
        if cols.len() < 4 {
            continue;
        }
        // local_address remote_address st
        let st = match cols.get(3).map(|s| *s).unwrap_or("00") {
            "01" => "ESTABLISHED",
            "0A" => "LISTEN",
            "06" => "TIME_WAIT",
            other => other,
        };
        out.push(NetSocket {
            local: cols[1].to_string(),
            remote: cols[2].to_string(),
            state: st.into(),
            pid: None,
        });
    }
    out
}

pub fn deep_probe(top_n: usize) -> DeepSnapshot {
    let mut sys = System::new_all();
    sys.refresh_all();
    std::thread::sleep(std::time::Duration::from_millis(200));
    sys.refresh_cpu_all();
    sys.refresh_memory();
    sys.refresh_processes(ProcessesToUpdate::All, true);

    let cpu_usage = if sys.cpus().is_empty() {
        0.0
    } else {
        let sum: f32 = sys.cpus().iter().map(|c| c.cpu_usage()).sum();
        (sum as f64) / (sys.cpus().len() as f64)
    };
    let ram_total_mb = sys.total_memory() as f64 / (1024.0 * 1024.0);
    let ram_used_mb = sys.used_memory() as f64 / (1024.0 * 1024.0);
    let ram_pct = if ram_total_mb > 0.0 {
        (ram_used_mb / ram_total_mb) * 100.0
    } else {
        0.0
    };

    let mut processes: Vec<ProcNode> = sys
        .processes()
        .iter()
        .map(|(pid, p)| {
            let cpu = p.cpu_usage() as f64;
            let ram_mb = p.memory() as f64 / (1024.0 * 1024.0);
            let st = if cpu >= 80.0 || ram_mb >= 1024.0 {
                "critical"
            } else if cpu >= 40.0 {
                "warning"
            } else {
                "good"
            };
            ProcNode {
                pid: pid.as_u32(),
                ppid: p.parent().map(|x| x.as_u32()).unwrap_or(0),
                name: p.name().to_string_lossy().to_string(),
                cpu: (cpu * 10.0).round() / 10.0,
                ram_mb: (ram_mb * 10.0).round() / 10.0,
                status: st.into(),
            }
        })
        .collect();
    processes.sort_by(|a, b| b.cpu.partial_cmp(&a.cpu).unwrap());
    processes.truncate(top_n.max(10));

    let disks_list = Disks::new_with_refreshed_list();
    let disks: Vec<serde_json::Value> = disks_list
        .iter()
        .map(|d| {
            let total = d.total_space() as f64 / 1e9;
            let avail = d.available_space() as f64 / 1e9;
            let used_pct = if total > 0.0 {
                ((total - avail) / total) * 100.0
            } else {
                0.0
            };
            serde_json::json!({
                "name": d.name().to_string_lossy(),
                "mount": d.mount_point().to_string_lossy(),
                "total_gb": (total * 10.0).round() / 10.0,
                "available_gb": (avail * 10.0).round() / 10.0,
                "used_pct": (used_pct * 10.0).round() / 10.0,
                "status": status_pct(used_pct),
            })
        })
        .collect();

    let nets_list = Networks::new_with_refreshed_list();
    let nets: Vec<serde_json::Value> = nets_list
        .iter()
        .map(|(name, data)| {
            serde_json::json!({
                "interface": name,
                "rx_bytes": data.total_received(),
                "tx_bytes": data.total_transmitted(),
            })
        })
        .collect();

    let thermals = linux_thermals();
    let sockets = linux_sockets_sample(40);

    let mut issues = Vec::new();
    if cpu_usage >= 90.0 {
        issues.push(format!("CPU critical: {:.1}%", cpu_usage));
    } else if cpu_usage >= 75.0 {
        issues.push(format!("CPU high: {:.1}%", cpu_usage));
    }
    if ram_pct >= 90.0 {
        issues.push(format!("RAM critical: {:.1}%", ram_pct));
    }
    for d in &disks {
        if d["used_pct"].as_f64().unwrap_or(0.0) >= 92.0 {
            issues.push(format!(
                "Disk full: {} {:.0}%",
                d["mount"].as_str().unwrap_or("?"),
                d["used_pct"].as_f64().unwrap_or(0.0)
            ));
        }
    }
    for t in &thermals {
        if let Some(c) = t.temp_c {
            if c >= 90.0 {
                issues.push(format!("Thermal {}: {:.0}°C", t.name, c));
            }
        }
    }
    for p in processes.iter().filter(|p| p.status == "critical").take(3) {
        issues.push(format!(
            "Process {} pid={} cpu={:.1}%",
            p.name, p.pid, p.cpu
        ));
    }

    let health = if issues.iter().any(|i| i.contains("critical") || i.contains("full"))
        || cpu_usage >= 90.0
        || ram_pct >= 90.0
    {
        "critical".into()
    } else if !issues.is_empty() {
        "warning".into()
    } else {
        "good".into()
    };

    let mut capabilities = vec![
        "sysinfo.cpu",
        "sysinfo.memory",
        "sysinfo.processes",
        "sysinfo.disks",
        "sysinfo.networks",
    ];
    if Path::new("/sys/class/thermal").exists() {
        capabilities.push("linux.thermal_sysfs");
    }
    if Path::new("/proc/net/tcp").exists() {
        capabilities.push("linux.proc_net_tcp");
    }
    if Path::new("/proc").exists() {
        capabilities.push("linux.procfs");
    }

    let load = System::load_average();

    DeepSnapshot {
        timestamp: ts(),
        hostname: System::host_name().unwrap_or_else(|| "unknown".into()),
        os: format!(
            "{} {}",
            System::name().unwrap_or_default(),
            System::os_version().unwrap_or_default()
        ),
        kernel: System::kernel_version().unwrap_or_default(),
        cpu_usage_pct: (cpu_usage * 10.0).round() / 10.0,
        cpu_count: sys.cpus().len(),
        cpu_brand: sys
            .cpus()
            .first()
            .map(|c| c.brand().to_string())
            .unwrap_or_default(),
        ram_usage_pct: (ram_pct * 10.0).round() / 10.0,
        ram_used_mb: (ram_used_mb * 10.0).round() / 10.0,
        ram_total_mb: (ram_total_mb * 10.0).round() / 10.0,
        load_avg: [load.one, load.five, load.fifteen],
        uptime_sec: System::uptime(),
        processes,
        sockets,
        thermals,
        disks,
        nets,
        issues,
        health,
        capabilities,
    }
}

pub fn deep_probe_json(top_n: usize) -> serde_json::Value {
    serde_json::to_value(deep_probe(top_n)).unwrap_or(serde_json::Value::Null)
}
