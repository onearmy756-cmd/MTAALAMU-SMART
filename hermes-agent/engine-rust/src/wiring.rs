//! SYSTEM WIRING — miunganisho halisi ya vifaa kutoka OS (si michoro bandia).
//!
//! Vyanzo (vyote halisi):
//!   - deep_probe: CPU cores, RAM, disks, net + traffic, sockets, thermals, processes
//!   - Linux:   /sys/class/dmi (board), lspci (PCI), lsusb (USB), /proc/partitions
//!   - Windows: wmic (board, GPU, USB)
//!
//! Output: SystemWiring { nodes, edges, buses, flows } — kila node na edge
//! zime-labeliwa, zote kutoka vifaa/vipengele halisi vinavyoonekana kwa OS.

use crate::deep_probe::{deep_probe, DeepSnapshot};
use serde::Serialize;
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
pub struct WiringNode {
    pub id: String,
    pub label: String,
    pub kind: String, // cpu | memory | disk | usb | pci | net | thermal | kernel | process
    pub status: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct WiringEdge {
    pub from: String,
    pub to: String,
    pub bus: String, // sata | pcie | usb | memory | net | kernel
    pub label: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct BusLane {
    pub id: String,
    pub name_sw: String,
    pub nodes: Vec<String>,
    pub health: String,
    pub throughput_note: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DataFlow {
    pub id: String,
    pub path: Vec<String>, // node ids: input → ... → output
    pub kind: String,      // io | net | proc
    pub rate_note: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SystemWiring {
    pub timestamp: u64,
    pub source: String,
    pub hostname: String,
    pub kernel: String,
    pub nodes: Vec<WiringNode>,
    pub edges: Vec<WiringEdge>,
    pub buses: Vec<BusLane>,
    pub flows: Vec<DataFlow>,
    pub capabilities: Vec<String>,
}

fn run_cmd(cmd: &str, args: &[&str]) -> Option<String> {
    let out = std::process::Command::new(cmd).args(args).output().ok()?;
    if out.status.success() {
        Some(String::from_utf8_lossy(&out.stdout).to_string())
    } else {
        None
    }
}

fn ts() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Motherboard halisi: Linux DMI au Windows wmic
fn board_info() -> (String, String) {
    if Path::new("/sys/class/dmi/id/board_vendor").exists() {
        let vendor = std::fs::read_to_string("/sys/class/dmi/id/board_vendor")
            .unwrap_or_default()
            .trim()
            .to_string();
        let name = std::fs::read_to_string("/sys/class/dmi/id/board_name")
            .unwrap_or_default()
            .trim()
            .to_string();
        return (vendor, name);
    }
    if let Some(o) = run_cmd("wmic", &["baseboard", "get", "Manufacturer,Product", "/format:list"]) {
        let mut v = String::new();
        let mut p = String::new();
        for line in o.lines() {
            if let Some(x) = line.strip_prefix("Manufacturer=") {
                v = x.trim().to_string();
            }
            if let Some(x) = line.strip_prefix("Product=") {
                p = x.trim().to_string();
            }
        }
        return (v, p);
    }
    ("unknown".into(), "board".into())
}

/// PCI devices halisi (Linux: lspci; Windows: wmic)
fn pci_devices() -> Vec<String> {
    if let Some(o) = run_cmd("lspci", &[]) {
        return o
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(|l| l.chars().take(60).collect())
            .collect();
    }
    if let Some(o) = run_cmd(
        "wmic",
        &["path", "Win32_VideoController", "get", "Name", "/format:list"],
    ) {
        let gpus: Vec<String> = o
            .lines()
            .filter_map(|l| l.strip_prefix("Name="))
            .map(|x| format!("VGA: {}", x.trim()))
            .collect();
        if !gpus.is_empty() {
            return gpus;
        }
    }
    Vec::new()
}

/// USB devices halisi (Linux: lsusb; Windows: wmic USB hub)
fn usb_devices() -> Vec<String> {
    if let Some(o) = run_cmd("lsusb", &[]) {
        return o
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(|l| l.chars().take(60).collect())
            .collect();
    }
    if let Some(o) = run_cmd("wmic", &["path", "Win32_USBHub", "get", "DeviceID", "/format:list"]) {
        return o
            .lines()
            .filter_map(|l| l.strip_prefix("DeviceID="))
            .map(|x| format!("USB {}", x.trim().chars().take(50).collect::<String>()))
            .collect();
    }
    Vec::new()
}

/// GPU halisi (Windows wmic au lspci VGA)
fn gpu_info(pci: &[String]) -> Option<String> {
    if let Some(o) = run_cmd(
        "wmic",
        &["path", "Win32_VideoController", "get", "Name", "/format:list"],
    ) {
        for line in o.lines() {
            if let Some(x) = line.strip_prefix("Name=") {
                if !x.trim().is_empty() {
                    return Some(x.trim().to_string());
                }
            }
        }
    }
    pci.iter()
        .find(|p| p.to_lowercase().contains("vga"))
        .cloned()
}

pub fn system_wiring() -> SystemWiring {
    system_wiring_top(10)
}

/// Ramani kamili ya wiring ya kifaa — DATA HALISI TU
pub fn system_wiring_top(top_procs: usize) -> SystemWiring {
    let d: DeepSnapshot = deep_probe(top_procs.max(8));
    let mut nodes: Vec<WiringNode> = Vec::new();
    let mut edges: Vec<WiringEdge> = Vec::new();

    // ---- CPU halisi ----
    let cpu_id = "cpu0".to_string();
    nodes.push(WiringNode {
        id: cpu_id.clone(),
        label: format!("CPU {}x", d.cpu_count),
        kind: "cpu".into(),
        status: if d.cpu_usage_pct >= 90.0 {
            "critical".into()
        } else if d.cpu_usage_pct >= 75.0 {
            "warning".into()
        } else {
            "good".into()
        },
        detail: format!("{} · {:.1}% · load {:.2}", d.cpu_brand, d.cpu_usage_pct, d.load_avg[0]),
    });

    // ---- RAM halisi ----
    let ram_id = "ram0".to_string();
    nodes.push(WiringNode {
        id: ram_id.clone(),
        label: format!("RAM {:.0}/{:.0} MB", d.ram_used_mb, d.ram_total_mb),
        kind: "memory".into(),
        status: if d.ram_usage_pct >= 90.0 {
            "critical".into()
        } else if d.ram_usage_pct >= 75.0 {
            "warning".into()
        } else {
            "good".into()
        },
        detail: format!("{:.0}% inatumika", d.ram_usage_pct),
    });
    edges.push(WiringEdge {
        from: cpu_id.clone(),
        to: ram_id.clone(),
        bus: "memory".into(),
        label: "memory controller".into(),
    });

    // ---- Motherboard halisi (DMI/wmic) ----
    let (b_vendor, b_name) = board_info();
    let mb_id = "motherboard".to_string();
    nodes.push(WiringNode {
        id: mb_id.clone(),
        label: format!("Board: {}", b_name),
        kind: "pci".into(),
        status: "good".into(),
        detail: format!("vendor: {}", b_vendor),
    });

    // ---- GPU halisi ----
    let pci = pci_devices();
    if let Some(gpu) = gpu_info(&pci) {
        let id = "gpu0".to_string();
        nodes.push(WiringNode {
            id: id.clone(),
            label: format!("GPU: {}", gpu.chars().take(40).collect::<String>()),
            kind: "pci".into(),
            status: "good".into(),
            detail: "video controller (halisi)".into(),
        });
        edges.push(WiringEdge {
            from: mb_id.clone(),
            to: id,
            bus: "pcie".into(),
            label: "PCIe".into(),
        });
    }

    // ---- PCI devices halisi (za kwanza 6) ----
    for (i, dev) in pci.iter().take(6).enumerate() {
        let id = format!("pci{}", i);
        nodes.push(WiringNode {
            id: id.clone(),
            label: dev.chars().take(48).collect(),
            kind: "pci".into(),
            status: "good".into(),
            detail: "PCI device (lspci/wmic)".into(),
        });
        edges.push(WiringEdge {
            from: mb_id.clone(),
            to: id,
            bus: "pcie".into(),
            label: "PCIe".into(),
        });
    }

    // ---- Disks halisi (block devices) ----
    for (i, disk) in d.disks.iter().enumerate() {
        let id = format!("disk{}", i);
        let mount = disk["mount"].as_str().unwrap_or("?");
        let pct = disk["used_pct"].as_f64().unwrap_or(0.0);
        nodes.push(WiringNode {
            id: id.clone(),
            label: format!("Disk {} {:.0}%", mount, pct),
            kind: "disk".into(),
            status: disk["status"].as_str().unwrap_or("good").to_string(),
            detail: format!(
                "total {:.1} GB · avail {:.1} GB",
                disk["total_gb"].as_f64().unwrap_or(0.0),
                disk["available_gb"].as_f64().unwrap_or(0.0)
            ),
        });
        edges.push(WiringEdge {
            from: mb_id.clone(),
            to: id,
            bus: "sata".into(),
            label: "storage bus".into(),
        });
    }

    // ---- USB devices halisi (za kwanza 4) ----
    let usb = usb_devices();
    for (i, dev) in usb.iter().take(4).enumerate() {
        let id = format!("usb{}", i);
        nodes.push(WiringNode {
            id: id.clone(),
            label: dev.chars().take(44).collect(),
            kind: "usb".into(),
            status: "good".into(),
            detail: "USB device (lsusb/wmic)".into(),
        });
        edges.push(WiringEdge {
            from: mb_id.clone(),
            to: id,
            bus: "usb".into(),
            label: "USB".into(),
        });
    }

    // ---- Network halisi + traffic ----
    for (i, net) in d.nets.iter().enumerate() {
        let id = format!("net{}", i);
        let name = net["interface"].as_str().unwrap_or("?");
        let rx = net["rx_bytes"].as_u64().unwrap_or(0);
        let tx = net["tx_bytes"].as_u64().unwrap_or(0);
        nodes.push(WiringNode {
            id: id.clone(),
            label: format!("Net {}", name),
            kind: "net".into(),
            status: "good".into(),
            detail: format!("RX {:.1} MB · TX {:.1} MB", rx as f64 / 1e6, tx as f64 / 1e6),
        });
        edges.push(WiringEdge {
            from: mb_id.clone(),
            to: id,
            bus: "net".into(),
            label: "I/O".into(),
        });
    }

    // ---- Thermals halisi ----
    for (i, t) in d.thermals.iter().enumerate() {
        let id = format!("thermal{}", i);
        let temp = t.temp_c.unwrap_or(0.0);
        nodes.push(WiringNode {
            id: id.clone(),
            label: format!("{} {:.0}°C", t.name, temp),
            kind: "thermal".into(),
            status: if temp >= 85.0 {
                "critical".into()
            } else if temp >= 70.0 {
                "warning".into()
            } else {
                "good".into()
            },
            detail: "thermal zone (/sys halisi)".into(),
        });
        edges.push(WiringEdge {
            from: cpu_id.clone(),
            to: id,
            bus: "memory".into(),
            label: "sensor".into(),
        });
    }

    // ---- Kernel + processes halisi ----
    let kern_id = "kernel".to_string();
    nodes.push(WiringNode {
        id: kern_id.clone(),
        label: format!("Kernel {}", d.kernel),
        kind: "kernel".into(),
        status: "good".into(),
        detail: format!("{} · uptime {}s", d.os, d.uptime_sec),
    });
    edges.push(WiringEdge {
        from: cpu_id.clone(),
        to: kern_id.clone(),
        bus: "kernel".into(),
        label: "scheduler".into(),
    });

    for (i, p) in d.processes.iter().take(top_procs.min(8)).enumerate() {
        let id = format!("proc{}", i);
        nodes.push(WiringNode {
            id: id.clone(),
            label: format!("{} [{}]", p.name.chars().take(16).collect::<String>(), p.pid),
            kind: "process".into(),
            status: p.status.clone(),
            detail: format!("CPU {:.1}% · RAM {:.0} MB", p.cpu, p.ram_mb),
        });
        edges.push(WiringEdge {
            from: kern_id.clone(),
            to: id,
            bus: "kernel".into(),
            label: "process".into(),
        });
    }

    // ---- Buses (njia halisi zilizogunduliwa) ----
    let has_usb = usb.len() > 0;
    let has_pci = pci.len() > 0;
    let mut buses = vec![
        BusLane {
            id: "mem".into(),
            name_sw: "Memory Bus (RAM ⇄ CPU)".into(),
            nodes: vec!["ram0".into(), "cpu0".into()],
            health: "good".into(),
            throughput_note: "traffic halisi: RAM inayotumika".into(),
        },
        BusLane {
            id: "storage".into(),
            name_sw: "Storage Bus (Disk ⇄ Board)".into(),
            nodes: d.disks.iter().enumerate().map(|(i, _)| format!("disk{}", i)).collect(),
            health: d
                .disks
                .iter()
                .map(|x| x["status"].as_str().unwrap_or("good"))
                .min()
                .unwrap_or("good")
                .to_string(),
            throughput_note: "I/O halisi ya block devices".into(),
        },
        BusLane {
            id: "kernel_bus".into(),
            name_sw: "Kernel Bus (OS ⇄ CPU)".into(),
            nodes: vec!["cpu0".into(), "kernel".into()],
            health: "good".into(),
            throughput_note: format!("processes halisi: {}", d.processes.len()),
        },
    ];
    if has_pci {
        buses.push(BusLane {
            id: "pcie".into(),
            name_sw: "PCIe Bus".into(),
            nodes: nodes
                .iter()
                .filter(|n| n.kind == "pci")
                .map(|n| n.id.clone())
                .collect(),
            health: "good".into(),
            throughput_note: format!("vifaa halisi: {}", pci.len()),
        });
    }
    if has_usb {
        buses.push(BusLane {
            id: "usb".into(),
            name_sw: "USB Bus".into(),
            nodes: nodes
                .iter()
                .filter(|n| n.kind == "usb")
                .map(|n| n.id.clone())
                .collect(),
            health: "good".into(),
            throughput_note: format!("vifaa halisi: {}", usb.len()),
        });
    }
    if !d.nets.is_empty() {
        buses.push(BusLane {
            id: "net_bus".into(),
            name_sw: "Network I/O".into(),
            nodes: nodes
                .iter()
                .filter(|n| n.kind == "net")
                .map(|n| n.id.clone())
                .collect(),
            health: "good".into(),
            throughput_note: "RX/TX halisi (bytes)".into(),
        });
    }

    // ---- Data flows halisi (Input → Output) ----
    let mut flows = Vec::new();
    // 1. Disk I/O flow
    if !d.disks.is_empty() {
        flows.push(DataFlow {
            id: "io_disk".into(),
            path: vec!["disk0".into(), "motherboard".into(), "cpu0".into(), "kernel".into()],
            kind: "io".into(),
            rate_note: "block I/O halisi (mount point)".into(),
        });
    }
    // 2. Network flow
    if !d.nets.is_empty() {
        flows.push(DataFlow {
            id: "net_flow".into(),
            path: vec!["net0".into(), "motherboard".into(), "cpu0".into(), "kernel".into()],
            kind: "net".into(),
            rate_note: format!(
                "RX {:.1} MB / TX {:.1} MB (halisi)",
                d.nets[0]["rx_bytes"].as_u64().unwrap_or(0) as f64 / 1e6,
                d.nets[0]["tx_bytes"].as_u64().unwrap_or(0) as f64 / 1e6
            ),
        });
    }
    // 3. Process CPU flow
    if let Some(p0) = d.processes.first() {
        flows.push(DataFlow {
            id: "proc_flow".into(),
            path: vec!["cpu0".into(), "kernel".into(), "proc0".into()],
            kind: "proc".into(),
            rate_note: format!("CPU {:.1}% ({})", p0.cpu, p0.name),
        });
    }
    // 4. Sockets halisi (network connections za kweli)
    for (i, s) in d.sockets.iter().take(3).enumerate() {
        flows.push(DataFlow {
            id: format!("sock{}", i),
            path: vec!["net0".into(), "kernel".into(), "cpu0".into()],
            kind: "net".into(),
            rate_note: format!("{} → {} ({})", s.local, s.remote, s.state),
        });
    }

    let mut capabilities: Vec<String> = d.capabilities.iter().map(|s| s.to_string()).collect();
    if has_pci {
        capabilities.push("pci.list".into());
    }
    if has_usb {
        capabilities.push("usb.list".into());
    }
    if Path::new("/sys/class/dmi").exists() {
        capabilities.push("dmi.board".into());
    }

    SystemWiring {
        timestamp: ts(),
        source: "os-wiring-live".into(),
        hostname: d.hostname.clone(),
        kernel: d.kernel.clone(),
        nodes,
        edges,
        buses,
        flows,
        capabilities,
    }
}

pub fn system_wiring_json(top: usize) -> serde_json::Value {
    serde_json::to_value(system_wiring_top(top)).unwrap_or(serde_json::Value::Null)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wiring_ina_nodes_halisi() {
        let w = system_wiring_top(8);
        assert!(w.nodes.len() >= 5, "nodes: {}", w.nodes.len());
        assert!(w.edges.len() >= 3);
        assert!(w.buses.len() >= 2);
        // CPU na RAM lazima ziwepo kila OS
        assert!(w.nodes.iter().any(|n| n.kind == "cpu"));
        assert!(w.nodes.iter().any(|n| n.kind == "memory"));
        // Kila node ina label
        assert!(w.nodes.iter().all(|n| !n.label.is_empty()));
    }

    #[test]
    fn flows_zina_path() {
        let w = system_wiring_top(8);
        for f in &w.flows {
            assert!(f.path.len() >= 2, "flow {} haina path", f.id);
        }
    }
}
