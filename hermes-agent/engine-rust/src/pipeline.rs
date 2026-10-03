//! PIITVD Pipeline — Plan, Identify, Implement, Test, Verify, Document

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStepDef {
    pub id: String,
    pub code: String,
    pub name_sw: String,
    pub name_en: String,
    pub description_sw: String,
    pub agent: String,
    pub requires_hitl: bool,
    pub voice_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PipelineFile {
    pub version: String,
    pub steps: Vec<PipelineStepDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepRuntime {
    pub id: String,
    pub code: String,
    pub name_sw: String,
    pub status: String, // pending | active | done | blocked | skipped
    pub started_at: Option<u64>,
    pub ended_at: Option<u64>,
    pub evidence: Vec<Value>,
    pub message_sw: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineRuntime {
    pub session_id: String,
    pub steps: Vec<StepRuntime>,
    pub current_index: usize,
    pub completed: bool,
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

pub struct PipelineEngine {
    defs: Vec<PipelineStepDef>,
}

impl PipelineEngine {
    pub fn load(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|e| format!("pipeline: {}", e))?;
        let file: PipelineFile =
            serde_json::from_str(&text).map_err(|e| format!("pipeline parse: {}", e))?;
        Ok(Self { defs: file.steps })
    }

    pub fn start(&self, session_id: &str) -> PipelineRuntime {
        let steps: Vec<StepRuntime> = self
            .defs
            .iter()
            .enumerate()
            .map(|(i, d)| StepRuntime {
                id: d.id.clone(),
                code: d.code.clone(),
                name_sw: d.name_sw.clone(),
                status: if i == 0 {
                    "active".into()
                } else {
                    "pending".into()
                },
                started_at: if i == 0 { Some(now()) } else { None },
                ended_at: None,
                evidence: vec![],
                message_sw: d.description_sw.clone(),
            })
            .collect();
        PipelineRuntime {
            session_id: session_id.to_string(),
            steps,
            current_index: 0,
            completed: false,
        }
    }

    pub fn advance(
        &self,
        rt: &mut PipelineRuntime,
        evidence: Value,
        message_sw: &str,
        hitl_ok: bool,
    ) -> Result<(), String> {
        if rt.completed {
            return Err("Pipeline imekamilika".into());
        }
        let i = rt.current_index;
        if i >= rt.steps.len() {
            rt.completed = true;
            return Ok(());
        }

        let def = &self.defs[i];
        if def.requires_hitl && !hitl_ok {
            rt.steps[i].status = "blocked".into();
            rt.steps[i].message_sw = format!(
                "{} — inasubiri ruhusa ya HITL.",
                def.name_sw
            );
            return Err("HITL required".into());
        }

        rt.steps[i].status = "done".into();
        rt.steps[i].ended_at = Some(now());
        rt.steps[i].evidence.push(evidence);
        rt.steps[i].message_sw = message_sw.to_string();

        if i + 1 < rt.steps.len() {
            rt.current_index = i + 1;
            rt.steps[i + 1].status = "active".into();
            rt.steps[i + 1].started_at = Some(now());
        } else {
            rt.completed = true;
        }
        Ok(())
    }

    pub fn defs(&self) -> &[PipelineStepDef] {
        &self.defs
    }
}
