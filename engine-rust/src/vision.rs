//! Live Vision — ramani + processes + issues kutoka OS probe HALISI

use crate::sysprobe::{probe, SystemProbe};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Component {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub status: String,
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub name: String,
    pub cpu: f64,
    pub ram: f64,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologyNode {
    pub id: String,
    pub label: String,
    pub x: f64,
    pub y: f64,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologyEdge {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Topology {
    pub nodes: Vec<TopologyNode>,
    pub edges: Vec<TopologyEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub title: String,
    pub desc: String,
    pub action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDataSnapshot {
    pub components: Vec<Component>,
    pub processes: Vec<ProcessInfo>,
    pub topology: Topology,
    pub issues: Vec<Issue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisionSnapshot {
    pub timestamp: u64,
    pub source: String,
    pub components: Vec<Component>,
    pub processes: Vec<ProcessInfo>,
    pub topology: Topology,
    pub issues: Vec<Issue>,
    pub summary: VisionSummary,
    pub bus_status: Vec<BusStatus>,
    pub probe: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisionSummary {
    pub total_components: usize,
    pub critical: usize,
    pub warning: usize,
    pub good: usize,
    pub critical_processes: usize,
    pub open_issues: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusStatus {
    pub id: String,
    pub name_sw: String,
    pub health: String,
    pub path: Vec<String>,
}

pub struct VisionEngine {
    snapshot: AgentDataSnapshot,
    bus_defs: Value,
    #[allow(dead_code)]
    device_map: Value,
}

impl VisionEngine {
    pub fn load(data_root: &Path) -> Result<Self, String> {
        let agent_path = data_root.join("agent_data.json");
        let snapshot = if agent_path.exists() {
            let text = fs::read_to_string(&agent_path)
                .map_err(|e| format!("agent_data.json: {}", e))?;
            serde_json::from_str(&text).map_err(|e| format!("agent_data parse: {}", e))?
        } else {
            AgentDataSnapshot {
                components: vec![],
                processes: vec![],
                topology: Topology {
                    nodes: vec![],
                    edges: vec![],
                },
                issues: vec![],
            }
        };

        let bus_path = data_root.join("vision/system_bus.json");
        let bus_defs = if bus_path.exists() {
            let t = fs::read_to_string(&bus_path).map_err(|e| e.to_string())?;
            serde_json::from_str(&t).unwrap_or(Value::Null)
        } else {
            Value::Null
        };

        let map_path = data_root.join("vision/device_map.json");
        let device_map = if map_path.exists() {
            let t = fs::read_to_string(&map_path).map_err(|e| e.to_string())?;
            serde_json::from_str(&t).unwrap_or(Value::Null)
        } else {
            Value::Null
        };

        Ok(Self {
            snapshot,
            bus_defs,
            device_map,
        })
    }

    /// Snapshot ya JSON tu (legacy / offline)
    pub fn scan(&self) -> VisionSnapshot {
        self.finalize_scan(self.snapshot.clone(), "json-static", None)
    }

    /// Snapshot HALISI kutoka OS (sysinfo)
    pub fn scan_live(&self) -> VisionSnapshot {
        let p = probe(12, 200);
        let live = build_from_probe(&p);
        let probe_val = serde_json::to_value(&p).ok();
        self.finalize_scan(live, "os-sysinfo", probe_val)
    }

    fn finalize_scan(
        &self,
        snap: AgentDataSnapshot,
        source: &str,
        probe: Option<Value>,
    ) -> VisionSnapshot {
        let critical = snap.components.iter().filter(|c| c.status == "critical").count();
        let warning = snap.components.iter().filter(|c| c.status == "warning").count();
        let good = snap.components.iter().filter(|c| c.status == "good").count();
        let critical_processes = snap
            .processes
            .iter()
            .filter(|p| p.status == "critical")
            .count();
        let bus_status = self.compute_bus_status(&snap);

        VisionSnapshot {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            source: source.into(),
            components: snap.components,
            processes: snap.processes,
            topology: snap.topology,
            issues: snap.issues,
            summary: VisionSummary {
                total_components: critical + warning + good,
                critical,
                warning,
                good,
                critical_processes,
                open_issues: 0, // set below
            },
            bus_status,
            probe,
        }
        .with_issue_count()
    }

    fn compute_bus_status(&self, snap: &AgentDataSnapshot) -> Vec<BusStatus> {
        let mut out = Vec::new();
        let buses = self
            .bus_defs
            .get("buses")
            .and_then(|b| b.as_array())
            .cloned()
            .unwrap_or_default();

        for b in buses {
            let id = b.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let name_sw = b
                .get("name_sw")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let path: Vec<String> = b
                .get("path")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|x| x.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();

            let mut health = "good".to_string();
            for pid in &path {
                if let Some(c) = snap.components.iter().find(|c| c.id == *pid) {
                    if c.status == "critical" {
                        health = "critical".into();
                        break;
                    }
                    if c.status == "warning" && health != "critical" {
                        health = "warning".into();
                    }
                }
            }
            out.push(BusStatus {
                id,
                name_sw,
                health,
                path,
            });
        }
        out
    }

    pub fn issues_for_auto(&self) -> Vec<&Issue> {
        self.snapshot.issues.iter().collect()
    }

    pub fn to_json(&self) -> Value {
        serde_json::to_value(self.scan_live()).unwrap_or(Value::Null)
    }
}

impl VisionSnapshot {
    fn with_issue_count(mut self) -> Self {
        self.summary.open_issues = self.issues.len();
        self
    }
}

fn status_pct(pct: f64) -> String {
    if pct >= 90.0 {
        "critical".into()
    } else if pct >= 75.0 {
        "warning".into()
    } else {
        "good".into()
    }
}

/// Unda components/processes/topology/issues kutoka SystemProbe — DATA HALISI
pub fn build_from_probe(p: &SystemProbe) -> AgentDataSnapshot {
    let cpu_st = status_pct(p.cpu_usage_pct);
    let ram_st = status_pct(p.ram_usage_pct);
    let disk_pct = p.disks.first().map(|d| d.used_pct).unwrap_or(0.0);
    let disk_st = status_pct(disk_pct);

    let components = vec![
        Component {
            id: "cpu".into(),
            name: format!("CPU {:.0}% ({})", p.cpu_usage_pct, p.cpu_brand),
            icon: "🧠".into(),
            status: cpu_st.clone(),
            x: 300.0,
            y: 180.0,
        },
        Component {
            id: "ram".into(),
            name: format!("RAM {:.0}% ({:.0}/{:.0} MB)", p.ram_usage_pct, p.ram_used_mb, p.ram_total_mb),
            icon: "💾".into(),
            status: ram_st.clone(),
            x: 180.0,
            y: 120.0,
        },
        Component {
            id: "disk".into(),
            name: format!(
                "Disk {:.0}% ({})",
                disk_pct,
                p.disks.first().map(|d| d.mount.as_str()).unwrap_or("?")
            ),
            icon: "🗄️".into(),
            status: disk_st.clone(),
            x: 180.0,
            y: 260.0,
        },
        Component {
            id: "os_kernel".into(),
            name: format!("Kernel {}", p.kernel),
            icon: "⚙️".into(),
            status: "good".into(),
            x: 300.0,
            y: 80.0,
        },
        Component {
            id: "os_userland".into(),
            name: format!("{} {}", p.os_name, p.os_version),
            icon: "📦".into(),
            status: p.health.clone(),
            x: 420.0,
            y: 120.0,
        },
        Component {
            id: "network".into(),
            name: format!("Net ({} ifaces)", p.networks.len()),
            icon: "🌐".into(),
            status: if p.networks.is_empty() {
                "warning".into()
            } else {
                "good".into()
            },
            x: 510.0,
            y: 180.0,
        },
        Component {
            id: "motherboard".into(),
            name: format!("Host: {}", p.hostname),
            icon: "🔩".into(),
            status: "good".into(),
            x: 300.0,
            y: 300.0,
        },
        Component {
            id: "swap".into(),
            name: format!("Swap {:.0}/{:.0} MB", p.swap_used_mb, p.swap_total_mb),
            icon: "📑".into(),
            status: if p.swap_total_mb > 0.0 && (p.swap_used_mb / p.swap_total_mb) > 0.8 {
                "warning".into()
            } else {
                "good".into()
            },
            x: 420.0,
            y: 260.0,
        },
    ];

    let processes: Vec<ProcessInfo> = p
        .top_processes
        .iter()
        .map(|pr| ProcessInfo {
            name: format!("{} [{}]", pr.name, pr.pid),
            cpu: pr.cpu,
            ram: if p.ram_total_mb > 0.0 {
                (pr.ram_mb / p.ram_total_mb) * 100.0
            } else {
                0.0
            },
            status: pr.status.clone(),
        })
        .collect();

    // Topology: Host → CPU/RAM/Disk → top processes
    let mut nodes = vec![
        TopologyNode {
            id: "host".into(),
            label: p.hostname.clone(),
            x: 300.0,
            y: 40.0,
            status: "online".into(),
        },
        TopologyNode {
            id: "cpu_n".into(),
            label: format!("CPU {:.0}%", p.cpu_usage_pct),
            x: 120.0,
            y: 140.0,
            status: if cpu_st == "critical" {
                "offline".into()
            } else {
                "online".into()
            },
        },
        TopologyNode {
            id: "ram_n".into(),
            label: format!("RAM {:.0}%", p.ram_usage_pct),
            x: 300.0,
            y: 140.0,
            status: if ram_st == "critical" {
                "warning".into()
            } else {
                "online".into()
            },
        },
        TopologyNode {
            id: "disk_n".into(),
            label: format!("Disk {:.0}%", disk_pct),
            x: 480.0,
            y: 140.0,
            status: if disk_st == "critical" {
                "warning".into()
            } else {
                "online".into()
            },
        },
    ];
    let mut edges = vec![
        TopologyEdge {
            from: "host".into(),
            to: "cpu_n".into(),
        },
        TopologyEdge {
            from: "host".into(),
            to: "ram_n".into(),
        },
        TopologyEdge {
            from: "host".into(),
            to: "disk_n".into(),
        },
    ];

    for (i, pr) in p.top_processes.iter().take(6).enumerate() {
        let id = format!("p{}", i);
        let x = 80.0 + (i as f64) * 90.0;
        nodes.push(TopologyNode {
            id: id.clone(),
            label: pr.name.chars().take(12).collect(),
            x,
            y: 280.0,
            status: if pr.status == "critical" {
                "warning".into()
            } else {
                "online".into()
            },
        });
        edges.push(TopologyEdge {
            from: "cpu_n".into(),
            to: id,
        });
    }

    let mut issues: Vec<Issue> = p
        .issues
        .iter()
        .map(|t| Issue {
            title: t.clone(),
            desc: format!(
                "Imetambuliwa na OS probe @ {} | host={} | os={}",
                p.timestamp, p.hostname, p.os_name
            ),
            action: if t.contains("CPU") {
                "Angalia michakato yenye CPU juu; funga isiyo muhimu".into()
            } else if t.contains("RAM") {
                "Funga programu zinazotumia RAM nyingi".into()
            } else if t.contains("Diski") || t.contains("Disk") {
                "Futa faili za muda; ongeza nafasi".into()
            } else if t.contains("Mchakato") {
                "Chunguza PID; zima kama siyo ya mfumo".into()
            } else {
                "Chunguza metrics za OS".into()
            },
        })
        .collect();

    if issues.is_empty() {
        issues.push(Issue {
            title: "Mfumo uko sawa".into(),
            desc: format!(
                "CPU {:.1}% · RAM {:.1}% · health={} · uptime {}s",
                p.cpu_usage_pct, p.ram_usage_pct, p.health, p.uptime_sec
            ),
            action: "Hakuna hatua ya dharura".into(),
        });
    }

    AgentDataSnapshot {
        components,
        processes,
        topology: Topology { nodes, edges },
        issues,
    }
}
