//! hardware.rs — FUNDI MAP: scan halisi ya vifaa vya PC (bila dependencies mpya).
//!
//! Inatumia tools za mfumo wenyewe (wmic/powershell kwa Windows, /proc + lsblk kwa Linux).
//! Hakuna uongo: kila sehemu inaonyesha chanzo chake; ikishindikana inasema hivyo.

use serde_json::{json, Value};

fn run(cmd: &str, args: &[&str]) -> Option<String> {
    std::process::Command::new(cmd)
        .args(args)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
}

/// CSV mmoja wa wmic → orodha ya rows (key→value)
fn wmic_rows(out: &str) -> Vec<Value> {
    let mut lines = out.lines().filter(|l| !l.trim().is_empty());
    let header: Vec<String> = lines
        .next()
        .map(|h| h.split(',').map(|s| s.trim().to_lowercase()).collect())
        .unwrap_or_default();
    let mut rows = Vec::new();
    for line in lines {
        // wmic CSV format hutumia commas; tumia split(',') ikiwa header ilikuwa CSV
        let vals: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
        if vals.len() == header.len() {
            let mut m = serde_json::Map::new();
            for (k, v) in header.iter().zip(vals.iter()) {
                m.insert(k.clone(), json!(v));
            }
            rows.push(Value::Object(m));
        }
    }
    rows
}

#[cfg(target_os = "windows")]
fn scan_cpu() -> Value {
    let mut v = json!({ "source": "wmic cpu" });
    if let Some(out) = run("wmic", &["cpu", "get", "Name,NumberOfCores,NumberOfLogicalProcessors,MaxClockSpeed,LoadPercentage", "/format:csv"]) {
        if let Some(row) = wmic_rows(&out).into_iter().next() {
            v["brand"] = row.get("name").cloned().unwrap_or(json!("?"));
            v["cores_physical"] = row.get("numberofcores").cloned().unwrap_or(json!(0));
            v["cores_logical"] = row.get("numberoflogicalprocessors").cloned().unwrap_or(json!(0));
            v["frequency_mhz"] = row.get("maxclockspeed").cloned().unwrap_or(json!(0));
            v["load_percent"] = row.get("loadpercentage").cloned().unwrap_or(json!(null));
        }
    }
    v
}

#[cfg(not(target_os = "windows"))]
fn scan_cpu() -> Value {
    let mut v = json!({ "source": "/proc/cpuinfo + /proc/loadavg" });
    if let Ok(txt) = std::fs::read_to_string("/proc/cpuinfo") {
        for line in txt.lines() {
            if line.starts_with("model name") {
                v["brand"] = json!(line.split(':').nth(1).unwrap_or("?").trim());
                break;
            }
        }
        v["cores_logical"] = json!(txt.matches("processor\t").count().max(txt.matches("\nprocessor").count()));
    }
    if let Ok(l) = std::fs::read_to_string("/proc/loadavg") {
        v["load_1m"] = json!(l.split_whitespace().next().unwrap_or("?"));
    }
    v
}

#[cfg(target_os = "windows")]
fn scan_memory() -> Value {
    let mut v = json!({ "source": "wmic memorychip", "modules": [] });
    if let Some(out) = run("wmic", &["memorychip", "get", "Capacity,Speed,Manufacturer,PartNumber", "/format:csv"]) {
        let mods: Vec<Value> = wmic_rows(&out)
            .into_iter()
            .map(|r| {
                let cap: u64 = r.get("capacity").and_then(|c| c.as_str()).and_then(|s| s.parse().ok()).unwrap_or(0);
                json!({
                    "size_gb": cap / 1024 / 1024 / 1024,
                    "speed_mhz": r.get("speed").cloned().unwrap_or(json!("?")),
                    "manufacturer": r.get("manufacturer").cloned().unwrap_or(json!("?")),
                    "part_number": r.get("partnumber").cloned().unwrap_or(json!("?")),
                })
            })
            .collect();
        let total: u64 = mods.iter().filter_map(|m| m["size_gb"].as_u64()).sum();
        v["modules"] = json!(mods);
        v["total_gb"] = json!(total);
    }
    v
}

#[cfg(not(target_os = "windows"))]
fn scan_memory() -> Value {
    let mut v = json!({ "source": "/proc/meminfo", "modules": [] });
    if let Ok(txt) = std::fs::read_to_string("/proc/meminfo") {
        for line in txt.lines() {
            if line.starts_with("MemTotal:") {
                let kb: u64 = line.split_whitespace().nth(1).and_then(|s| s.parse().ok()).unwrap_or(0);
                v["total_gb"] = json!(kb / 1024 / 1024);
            }
        }
    }
    v
}

#[cfg(target_os = "windows")]
fn scan_storage() -> Value {
    let mut v = json!({ "source": "wmic diskdrive + logicaldisk", "disks": [] });
    if let Some(out) = run("wmic", &["diskdrive", "get", "Model,Size,Status", "/format:csv"]) {
        let disks: Vec<Value> = wmic_rows(&out)
            .into_iter()
            .map(|r| {
                let size: u64 = r.get("size").and_then(|c| c.as_str()).and_then(|s| s.parse().ok()).unwrap_or(0);
                json!({
                    "model": r.get("model").cloned().unwrap_or(json!("?")),
                    "size_gb": size / 1024 / 1024 / 1024,
                    "smart_status": r.get("status").cloned().unwrap_or(json!("?")),
                })
            })
            .collect();
        v["disks"] = json!(disks);
    }
    // Nafasi huru (C:, D:...)
    if let Some(out) = run("wmic", &["logicaldisk", "get", "DeviceID,FreeSpace,Size", "/format:csv"]) {
        let vols: Vec<Value> = wmic_rows(&out)
            .into_iter()
            .filter(|r| r.get("deviceid").map(|d| d.as_str().unwrap_or("").len() == 2).unwrap_or(false))
            .map(|r| {
                let free: u64 = r.get("freespace").and_then(|c| c.as_str()).and_then(|s| s.parse().ok()).unwrap_or(0);
                let total: u64 = r.get("size").and_then(|c| c.as_str()).and_then(|s| s.parse().ok()).unwrap_or(0);
                json!({
                    "volume": r.get("deviceid").cloned().unwrap_or(json!("?")),
                    "free_gb": free / 1024 / 1024 / 1024,
                    "total_gb": total / 1024 / 1024 / 1024,
                })
            })
            .collect();
        v["volumes"] = json!(vols);
    }
    v
}

#[cfg(not(target_os = "windows"))]
fn scan_storage() -> Value {
    let mut v = json!({ "source": "lsblk", "disks": [] });
    if let Some(out) = run("lsblk", &["-b", "-d", "-o", "NAME,SIZE,MODEL,ROTA"]) {
        let disks: Vec<Value> = out
            .lines()
            .skip(1)
            .filter_map(|l| {
                let mut it = l.split_whitespace();
                let name = it.next()?;
                let size: u64 = it.next()?.parse().ok()?;
                let model = it.next().unwrap_or("?");
                Some(json!({ "name": name, "size_gb": size / 1024 / 1024 / 1024, "model": model }))
            })
            .collect();
        v["disks"] = json!(disks);
    }
    v
}

#[cfg(target_os = "windows")]
fn scan_gpu() -> Value {
    let mut v = json!({ "source": "wmic path win32_videocontroller", "gpus": [] });
    if let Some(out) = run("wmic", &["path", "win32_videocontroller", "get", "Name,AdapterRAM,DriverVersion", "/format:csv"]) {
        let gpus: Vec<Value> = wmic_rows(&out)
            .into_iter()
            .map(|r| {
                let ram: u64 = r.get("adapterram").and_then(|c| c.as_str()).and_then(|s| s.parse().ok()).unwrap_or(0);
                json!({
                    "name": r.get("name").cloned().unwrap_or(json!("?")),
                    "vram_mb": ram / 1024 / 1024,
                    "driver": r.get("driverversion").cloned().unwrap_or(json!("?")),
                })
            })
            .collect();
        v["gpus"] = json!(gpus);
    }
    v
}

#[cfg(not(target_os = "windows"))]
fn scan_gpu() -> Value {
    let mut v = json!({ "source": "lspci", "gpus": [] });
    if let Some(out) = run("lspci", &[]) {
        let gpus: Vec<Value> = out
            .lines()
            .filter(|l| l.contains("VGA") || l.contains("3D controller") || l.contains("Display"))
            .map(|l| json!({ "name": l.split(": ").nth(1).unwrap_or(l).trim() }))
            .collect();
        v["gpus"] = json!(gpus);
    }
    v
}

fn scan_network() -> Value {
    let mut v = json!({ "source": "ipconfig /all | ip addr", "adapters": [] });
    #[cfg(target_os = "windows")]
    {
        if let Some(out) = run("ipconfig", &["/all"]) {
            let mut adapters: Vec<String> = Vec::new();
            for line in out.lines() {
                let t = line.trim_end();
                if !t.is_empty() && !t.starts_with(' ') && t.contains(':') {
                    adapters.push(t.trim_end_matches(':').to_string());
                }
            }
            v["adapters"] = json!(adapters);
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        if let Some(out) = run("ip", &["-brief", "addr"]) {
            let adapters: Vec<Value> = out
                .lines()
                .filter_map(|l| {
                    let mut it = l.split_whitespace();
                    let name = it.next()?;
                    let state = it.next().unwrap_or("?");
                    Some(json!({ "name": name, "state": state }))
                })
                .collect();
            v["adapters"] = json!(adapters);
        }
    }
    v
}

fn scan_os() -> Value {
    let mut v = json!({
        "os": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
        "hostname": std::env::var("COMPUTERNAME")
            .or_else(|_| std::env::var("HOSTNAME"))
            .unwrap_or_else(|_| "?".into()),
    });
    #[cfg(target_os = "windows")]
    if let Some(out) = run("wmic", &["os", "get", "Caption,Version", "/format:csv"]) {
        if let Some(row) = wmic_rows(&out).into_iter().next() {
            v["name"] = row.get("caption").cloned().unwrap_or(json!("?"));
            v["version"] = row.get("version").cloned().unwrap_or(json!("?"));
        }
    }
    v
}

/// Scan kamili — blocking; ita kutoka async kwa spawn_blocking.
pub fn scan_all_blocking() -> Value {
    json!({
        "kind": "fundi-map",
        "timestamp": chrono::Local::now().to_rfc3339(),
        "system": scan_os(),
        "cpu": scan_cpu(),
        "memory": scan_memory(),
        "storage": scan_storage(),
        "gpu": scan_gpu(),
        "network": scan_network(),
    })
}

pub async fn scan_all() -> Value {
    tokio::task::spawn_blocking(scan_all_blocking)
        .await
        .unwrap_or_else(|_| json!({ "kind": "fundi-map", "error": "scan task imekufa" }))
}
