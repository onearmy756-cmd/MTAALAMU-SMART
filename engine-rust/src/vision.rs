//! Live Vision Engine — ramani ya kifaa, processes, topology, issues
//! Source of truth: data/agent_data.json + data/vision/device_map.json + system_bus.json
//! Metrics: kutoka snapshot (production: sysinfo probe baadaye)

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
    pub components: Vec<Component>,
    pub processes: Vec<ProcessInfo>,
    pub topology: Topology,
    pub issues: Vec<Issue>,
    pub summary: VisionSummary,
    pub bus_status: Vec<BusStatus>,
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
    device_map: Value,
}

impl VisionEngine {
    pub fn load(data_root: &Path) -> Result<Self, String> {
        let agent_path = data_root.join("agent_data.json");
        let text = fs::read_to_string(&agent_path)
            .map_err(|e| format!("agent_data.json: {}", e))?;
        let snapshot: AgentDataSnapshot =
            serde_json::from_str(&text).map_err(|e| format!("agent_data parse: {}", e))?;

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

    pub fn scan(&self) -> VisionSnapshot {
        let critical = self
            .snapshot
            .components
            .iter()
            .filter(|c| c.status == "critical")
            .count();
        let warning = self
            .snapshot
            .components
            .iter()
            .filter(|c| c.status == "warning")
            .count();
        let good = self
            .snapshot
            .components
            .iter()
            .filter(|c| c.status == "good")
            .count();
        let critical_processes = self
            .snapshot
            .processes
            .iter()
            .filter(|p| p.status == "critical")
            .count();

        let bus_status = self.compute_bus_status();

        VisionSnapshot {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            components: self.snapshot.components.clone(),
            processes: self.snapshot.processes.clone(),
            topology: self.snapshot.topology.clone(),
            issues: self.snapshot.issues.clone(),
            summary: VisionSummary {
                total_components: self.snapshot.components.len(),
                critical,
                warning,
                good,
                critical_processes,
                open_issues: self.snapshot.issues.len(),
            },
            bus_status,
        }
    }

    fn compute_bus_status(&self) -> Vec<BusStatus> {
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

            // Health from worst component status on path
            let mut health = "good".to_string();
            for pid in &path {
                if let Some(c) = self.snapshot.components.iter().find(|c| c.id == *pid) {
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

    pub fn critical_components(&self) -> Vec<&Component> {
        self.snapshot
            .components
            .iter()
            .filter(|c| c.status == "critical" || c.status == "warning")
            .collect()
    }

    pub fn to_json(&self) -> Value {
        serde_json::to_value(self.scan()).unwrap_or(Value::Null)
    }
}
