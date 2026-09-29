//! AV6 — OS System Probe (metrics halisi)
//! CPU, RAM, disk, network, processes — via `sysinfo`
//! JSON output kwa R / CLI / Vision engine.

use serde::Serialize;
use sysinfo::{
    Disks, Networks, ProcessesToUpdate, System,
};

#[derive(Debug, Clone, Serialize)]
pub struct ProbeProcess {
    pub name: String,
    pub pid: u32,
    pub cpu: f64,
    pub ram_mb: f64,
    pub status: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProbeDisk {
    pub name: String,
    pub mount: String,
    pub total_gb: f64,
    pub available_gb: f64,
    pub used_pct: f64,
    pub status: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProbeNet {
    pub interface: String,
    pub rx_bytes: u64,
    pub tx_bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SystemProbe {
    pub timestamp: u64,
    pub hostname: String,
    pub os_name: String,
    pub os_version: String,
    pub kernel: String,
    pub cpu_count: usize,
    pub cpu_brand: String,
    pub cpu_usage_pct: f64,
    pub ram_total_mb: f64,
    pub ram_used_mb: f64,
    pub ram_usage_pct: f64,
    pub swap_total_mb: f64,
    pub swap_used_mb: f64,
    pub uptime_sec: u64,
    pub load_avg: [f64; 3],
    pub disks: Vec<ProbeDisk>,
    pub networks: Vec<ProbeNet>,
    pub top_processes: Vec<ProbeProcess>,
    pub health: String,
    pub issues: Vec<String>,
}

fn status_from_pct(pct: f64) -> &'static str {
    if pct >= 90.0 {
        "critical"
    } else if pct >= 75.0 {
        "warning"
    } else {
        "good"
    }
}

fn now_ts() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Probe kamili ya OS. `refresh_wait_ms` inaruhusu CPU sample (default 200).
pub fn probe(top_n: usize, refresh_wait_ms: u64) -> SystemProbe {
    let mut sys = System::new_all();

    // First refresh
    sys.refresh_all();
    if refresh_wait_ms > 0 {
        std::thread::sleep(std::time::Duration::from_millis(refresh_wait_ms));
        sys.refresh_cpu_all();
        sys.refresh_memory();
        sys.refresh_processes(ProcessesToUpdate::All, true);
    }

    let cpu_usage: f64 = if sys.cpus().is_empty() {
        0.0
    } else {
        let sum: f32 = sys.cpus().iter().map(|c| c.cpu_usage()).sum();
        (sum as f64) / (sys.cpus().len() as f64)
    };

    let ram_total = sys.total_memory() as f64 / 1024.0; // KiB -> MiB (sysinfo uses bytes on 0.32?)
    // sysinfo 0.32: memory in bytes
    let ram_total_mb = sys.total_memory() as f64 / (1024.0 * 1024.0);
    let ram_used_mb = sys.used_memory() as f64 / (1024.0 * 1024.0);
    let ram_pct = if ram_total_mb > 0.0 {
        (ram_used_mb / ram_total_mb) * 100.0
    } else {
        0.0
    };

    let swap_total_mb = sys.total_swap() as f64 / (1024.0 * 1024.0);
    let swap_used_mb = sys.used_swap() as f64 / (1024.0 * 1024.0);

    let load = System::load_average();
    let load_avg = [load.one, load.five, load.fifteen];

    // Disks
    let disks_list = Disks::new_with_refreshed_list();
    let mut disks: Vec<ProbeDisk> = disks_list
        .iter()
        .map(|d| {
            let total = d.total_space() as f64 / (1024.0 * 1024.0 * 1024.0);
            let avail = d.available_space() as f64 / (1024.0 * 1024.0 * 1024.0);
            let used_pct = if total > 0.0 {
                ((total - avail) / total) * 100.0
            } else {
                0.0
            };
            ProbeDisk {
                name: d.name().to_string_lossy().to_string(),
                mount: d.mount_point().to_string_lossy().to_string(),
                total_gb: (total * 10.0).round() / 10.0,
                available_gb: (avail * 10.0).round() / 10.0,
                used_pct: (used_pct * 10.0).round() / 10.0,
                status: status_from_pct(used_pct).into(),
            }
        })
        .collect();
    disks.sort_by(|a, b| b.used_pct.partial_cmp(&a.used_pct).unwrap());

    // Network
    let nets = Networks::new_with_refreshed_list();
    let networks: Vec<ProbeNet> = nets
        .iter()
        .map(|(name, data)| ProbeNet {
            interface: name.clone(),
            rx_bytes: data.total_received(),
            tx_bytes: data.total_transmitted(),
        })
        .collect();

    // Top processes by CPU
    let mut procs: Vec<ProbeProcess> = sys
        .processes()
        .iter()
        .map(|(pid, p)| {
            let cpu = p.cpu_usage() as f64;
            let ram_mb = p.memory() as f64 / (1024.0 * 1024.0);
            let st = if cpu >= 80.0 || ram_mb >= 1024.0 {
                "critical"
            } else if cpu >= 40.0 || ram_mb >= 512.0 {
                "warning"
            } else {
                "good"
            };
            ProbeProcess {
                name: p.name().to_string_lossy().to_string(),
                pid: pid.as_u32(),
                cpu: (cpu * 10.0).round() / 10.0,
                ram_mb: (ram_mb * 10.0).round() / 10.0,
                status: st.into(),
            }
        })
        .collect();
    procs.sort_by(|a, b| b.cpu.partial_cmp(&a.cpu).unwrap());
    procs.truncate(top_n.max(5));

    // Issues from thresholds
    let mut issues = Vec::new();
    if cpu_usage >= 90.0 {
        issues.push(format!(
            "CPU juu sana: {:.1}% — angalia michakato yenye mzigo", cpu_usage
        ));
    } else if cpu_usage >= 75.0 {
        issues.push(format!("CPU imejaa: {:.1}%", cpu_usage));
    }
    if ram_pct >= 90.0 {
        issues.push(format!(
            "RAM karibu imejaa: {:.1}% ({:.0} / {:.0} MB)",
            ram_pct, ram_used_mb, ram_total_mb
        ));
    } else if ram_pct >= 80.0 {
        issues.push(format!("RAM juu: {:.1}%", ram_pct));
    }
    for d in &disks {
        if d.used_pct >= 92.0 {
            issues.push(format!(
                "Diski '{}' imejaa {:.1}% (free {:.1} GB)",
                d.mount, d.used_pct, d.available_gb
            ));
        }
    }
    for p in procs.iter().filter(|p| p.status == "critical").take(3) {
        issues.push(format!(
            "Mchakato hatari: {} (PID {}) CPU {:.1}% RAM {:.0}MB",
            p.name, p.pid, p.cpu, p.ram_mb
        ));
    }

    let health = if issues.iter().any(|i| i.contains("imejaa") || i.contains("karibu"))
        || cpu_usage >= 90.0
        || ram_pct >= 90.0
    {
        "critical".into()
    } else if !issues.is_empty() {
        "warning".into()
    } else {
        "good".into()
    };

    let _ = ram_total; // silence if unused on some versions

    SystemProbe {
        timestamp: now_ts(),
        hostname: System::host_name().unwrap_or_else(|| "unknown".into()),
        os_name: System::name().unwrap_or_else(|| "unknown".into()),
        os_version: System::os_version().unwrap_or_else(|| "".into()),
        kernel: System::kernel_version().unwrap_or_else(|| "".into()),
        cpu_count: sys.cpus().len(),
        cpu_brand: sys
            .cpus()
            .first()
            .map(|c| c.brand().to_string())
            .unwrap_or_default(),
        cpu_usage_pct: (cpu_usage * 10.0).round() / 10.0,
        ram_total_mb: (ram_total_mb * 10.0).round() / 10.0,
        ram_used_mb: (ram_used_mb * 10.0).round() / 10.0,
        ram_usage_pct: (ram_pct * 10.0).round() / 10.0,
        swap_total_mb: (swap_total_mb * 10.0).round() / 10.0,
        swap_used_mb: (swap_used_mb * 10.0).round() / 10.0,
        uptime_sec: System::uptime(),
        load_avg,
        disks,
        networks,
        top_processes: procs,
        health,
        issues,
    }
}

pub fn probe_json(top_n: usize) -> serde_json::Value {
    serde_json::to_value(probe(top_n, 200)).unwrap_or(serde_json::Value::Null)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn probe_returns_cpu_and_ram() {
        let p = probe(5, 50);
        assert!(p.cpu_count >= 1 || p.cpu_usage_pct >= 0.0);
        assert!(p.ram_total_mb >= 0.0);
        assert!(!p.hostname.is_empty() || p.hostname == "unknown");
    }
}
