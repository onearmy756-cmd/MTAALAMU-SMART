//! Multi-Agent 10 + Orchestrator — MTAALAMU SMART Agentic Vision
//! Data-driven kutoka data/agents/agents_10.json

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDef {
    pub id: String,
    pub name_sw: String,
    pub name_en: String,
    pub role: String,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub voice_key: String,
    pub auto: bool,
    #[serde(default)]
    pub requires_hitl: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorConfig {
    pub modes: Vec<String>,
    pub default_mode: String,
    pub pipeline_order: Vec<String>,
    pub hitl_gates: Vec<String>,
    pub timeout_sec: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentsFile {
    pub version: String,
    pub orchestrator: OrchestratorConfig,
    pub agents: Vec<AgentDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SessionState {
    Created,
    Scanning,
    Diagnosing,
    Planning,
    AwaitingHitl,
    Implementing,
    Testing,
    Verifying,
    Documenting,
    Completed,
    Failed,
    Cancelled,
}

impl SessionState {
    pub fn as_str(&self) -> &'static str {
        match self {
            SessionState::Created => "created",
            SessionState::Scanning => "scanning",
            SessionState::Diagnosing => "diagnosing",
            SessionState::Planning => "planning",
            SessionState::AwaitingHitl => "awaiting_hitl",
            SessionState::Implementing => "implementing",
            SessionState::Testing => "testing",
            SessionState::Verifying => "verifying",
            SessionState::Documenting => "documenting",
            SessionState::Completed => "completed",
            SessionState::Failed => "failed",
            SessionState::Cancelled => "cancelled",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentEvent {
    pub agent_id: String,
    pub event: String,
    pub message_sw: String,
    pub ts: u64,
    pub data: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSession {
    pub id: String,
    pub state: SessionState,
    pub user_message: String,
    pub trade: String,
    pub symptoms: Vec<String>,
    pub language: String,
    pub current_agent: Option<String>,
    pub pipeline_index: usize,
    pub hitl_approved: bool,
    pub events: Vec<AgentEvent>,
    pub context: HashMap<String, Value>,
    pub created_at: u64,
    pub updated_at: u64,
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn session_id() -> String {
    format!("AV-{}-{}", now_secs(), &uuid_lite())
}

fn uuid_lite() -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    now_secs().hash(&mut h);
    format!("{:x}", h.finish() % 0xFFFF)
}

pub struct AgentOrchestrator {
    config: OrchestratorConfig,
    agents: HashMap<String, AgentDef>,
    order: Vec<String>,
}

impl AgentOrchestrator {
    pub fn load(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|e| format!("agents load: {}", e))?;
        let file: AgentsFile =
            serde_json::from_str(&text).map_err(|e| format!("agents parse: {}", e))?;
        let mut map = HashMap::new();
        for a in file.agents {
            map.insert(a.id.clone(), a);
        }
        Ok(Self {
            order: file.orchestrator.pipeline_order.clone(),
            config: file.orchestrator,
            agents: map,
        })
    }

    pub fn agent_count(&self) -> usize {
        self.agents.len()
    }

    pub fn list_agents(&self) -> Vec<&AgentDef> {
        self.order
            .iter()
            .filter_map(|id| self.agents.get(id))
            .collect()
    }

    pub fn get(&self, id: &str) -> Option<&AgentDef> {
        self.agents.get(id)
    }

    /// Anza session mpya kutoka ujumbe wa mtumiaji
    pub fn start_session(&self, user_message: &str, language: &str) -> AgentSession {
        let ts = now_secs();
        let (trade, symptoms) = infer_trade_symptoms(user_message);
        let mut session = AgentSession {
            id: session_id(),
            state: SessionState::Created,
            user_message: user_message.to_string(),
            trade,
            symptoms,
            language: language.to_string(),
            current_agent: Some("receptionist".into()),
            pipeline_index: 0,
            hitl_approved: false,
            events: vec![],
            context: HashMap::new(),
            created_at: ts,
            updated_at: ts,
        };
        session.events.push(AgentEvent {
            agent_id: "receptionist".into(),
            event: "session_started".into(),
            message_sw: format!(
                "Session {} imeanzishwa. Tatizo limeregistrishwa.",
                session.id
            ),
            ts,
            data: serde_json::json!({
                "trade": session.trade,
                "symptoms": session.symptoms,
            }),
        });
        session
    }

    /// Endeleza pipeline hatua moja. Rudi session iliyosasishwa + event.
    pub fn advance(&self, session: &mut AgentSession) -> Result<AgentEvent, String> {
        if matches!(
            session.state,
            SessionState::Completed | SessionState::Failed | SessionState::Cancelled
        ) {
            return Err("Session imekwisha".into());
        }

        // HITL gate
        if session.state == SessionState::AwaitingHitl && !session.hitl_approved {
            return Err("Inasubiri ruhusa ya HITL".into());
        }

        let idx = session.pipeline_index;
        if idx >= self.order.len() {
            session.state = SessionState::Completed;
            session.updated_at = now_secs();
            let ev = AgentEvent {
                agent_id: "orchestrator".into(),
                event: "completed".into(),
                message_sw: "Pipeline imekamilika.".into(),
                ts: session.updated_at,
                data: Value::Null,
            };
            session.events.push(ev.clone());
            return Ok(ev);
        }

        let agent_id = &self.order[idx];
        let agent = self
            .agents
            .get(agent_id)
            .ok_or_else(|| format!("Agent '{}' haipo", agent_id))?;

        // Ikiwa agent inahitaji HITL na bado haijaidhinishwa
        if agent.requires_hitl && !session.hitl_approved {
            session.state = SessionState::AwaitingHitl;
            session.current_agent = Some(agent_id.clone());
            session.updated_at = now_secs();
            let ev = AgentEvent {
                agent_id: agent_id.clone(),
                event: "hitl_required".into(),
                message_sw: format!(
                    "{} inahitaji ruhusa yako kabla ya kuendelea.",
                    agent.name_sw
                ),
                ts: session.updated_at,
                data: serde_json::json!({"agent": agent_id}),
            };
            session.events.push(ev.clone());
            return Ok(ev);
        }

        // Execute agent step (deterministic stub — integrates with vision/pipeline later)
        let (new_state, message_sw, data) = self.execute_agent(agent, session);
        session.state = new_state;
        session.current_agent = Some(agent_id.clone());
        session.pipeline_index = idx + 1;
        session.updated_at = now_secs();

        let ev = AgentEvent {
            agent_id: agent_id.clone(),
            event: "step_done".into(),
            message_sw,
            ts: session.updated_at,
            data,
        };
        session.events.push(ev.clone());
        Ok(ev)
    }

    pub fn approve_hitl(&self, session: &mut AgentSession) -> AgentEvent {
        session.hitl_approved = true;
        if session.state == SessionState::AwaitingHitl {
            session.state = SessionState::Implementing;
        }
        session.updated_at = now_secs();
        let ev = AgentEvent {
            agent_id: session
                .current_agent
                .clone()
                .unwrap_or_else(|| "orchestrator".into()),
            event: "hitl_approved".into(),
            message_sw: "Ruhusa imetolewa. Ninaendelea.".into(),
            ts: session.updated_at,
            data: Value::Null,
        };
        session.events.push(ev.clone());
        ev
    }

    fn execute_agent(
        &self,
        agent: &AgentDef,
        session: &AgentSession,
    ) -> (SessionState, String, Value) {
        match agent.id.as_str() {
            "receptionist" => (
                SessionState::Scanning,
                format!(
                    "Habari. Nimepokea tatizo lako. Biashara: {}. Ninaanza kuchanganua.",
                    session.trade
                ),
                serde_json::json!({"trade": session.trade, "symptoms": session.symptoms}),
            ),
            "vision" => (
                SessionState::Diagnosing,
                "Nimekamilisha scan ya kifaa. Ninaendelea na utambuzi.".into(),
                serde_json::json!({"snapshot_ready": true}),
            ),
            "diagnoser" => (
                SessionState::Planning,
                "Nimegundua sababu zinazowezekana. Ninaandaa mpango.".into(),
                serde_json::json!({"diagnosed": true}),
            ),
            "planner" => {
                let needs_hitl = self.config.hitl_gates.iter().any(|g| g == "solver");
                if needs_hitl && !session.hitl_approved {
                    (
                        SessionState::AwaitingHitl,
                        "Mpango uko tayari. Ninahitaji ruhusa yako kuanzisha utatuzi.".into(),
                        serde_json::json!({"plan_ready": true, "requires_hitl": true}),
                    )
                } else {
                    (
                        SessionState::Implementing,
                        "Mpango uko tayari. Ninaanza kutekeleza.".into(),
                        serde_json::json!({"plan_ready": true}),
                    )
                }
            }
            "solver" => (
                SessionState::Testing,
                "Nimetekeleza hatua za suluhisho. Ninaenda kujaribu.".into(),
                serde_json::json!({"actions_done": true}),
            ),
            "tester" => (
                SessionState::Verifying,
                "Majaribio yamekamilika. Ninaenda kuthibitisha.".into(),
                serde_json::json!({"tests_passed": true}),
            ),
            "verifier" => {
                if !session.hitl_approved {
                    (
                        SessionState::AwaitingHitl,
                        "Tafadhali thibitisha kwamba tatizo limetatuliwa.".into(),
                        serde_json::json!({"requires_confirm": true}),
                    )
                } else {
                    (
                        SessionState::Documenting,
                        "Imethibitishwa. Ninaandika ripoti.".into(),
                        serde_json::json!({"verified": true}),
                    )
                }
            }
            "scribe" => (
                SessionState::Documenting,
                "Maelezo ya hatua yameandikwa kwa Kiswahili.".into(),
                serde_json::json!({"narration_ready": true}),
            ),
            "reporter" => (
                SessionState::Documenting,
                "Ripoti ya kitabu kidigitali iko tayari.".into(),
                serde_json::json!({"report_ready": true}),
            ),
            "learner" => (
                SessionState::Completed,
                "Maarifa yamehifadhiwa. Kazi imekamilika.".into(),
                serde_json::json!({"learned": true}),
            ),
            _ => (
                session.state.clone(),
                format!("Agent {} imefanya kazi.", agent.name_sw),
                Value::Null,
            ),
        }
    }

    pub fn summary(&self) -> Value {
        serde_json::json!({
            "agents": self.agent_count(),
            "mode": self.config.default_mode,
            "order": self.order,
            "hitl_gates": self.config.hitl_gates,
        })
    }
}

/// Infer trade + symptom keywords kutoka maandishi ya Kiswahili/English (heuristics).
fn infer_trade_symptoms(msg: &str) -> (String, Vec<String>) {
    let lower = msg.to_lowercase();
    let mut symptoms = Vec::new();
    let trade = if lower.contains("umeme")
        || lower.contains("voltage")
        || lower.contains("breaker")
        || lower.contains("spark")
    {
        if lower.contains("breaker") || lower.contains("trip") {
            symptoms.push("breaker_trips".into());
        }
        if lower.contains("spark") || lower.contains("cheche") {
            symptoms.push("sparks".into());
        }
        "umeme".into()
    } else if lower.contains("kompyuta")
        || lower.contains("computer")
        || lower.contains("laptop")
        || lower.contains("cpu")
        || lower.contains("ram")
    {
        if lower.contains("polepole") || lower.contains("slow") {
            symptoms.push("slow_performance".into());
        }
        if lower.contains("joto") || lower.contains("heat") || lower.contains("hot") {
            symptoms.push("overheating".into());
        }
        if lower.contains("disk") || lower.contains("hifadhi") {
            symptoms.push("disk_full".into());
        }
        "computer".into()
    } else if lower.contains("simu") || lower.contains("phone") || lower.contains("android") {
        "simu".into()
    } else if lower.contains("gari") || lower.contains("car") || lower.contains("engine") {
        "gari".into()
    } else if lower.contains("mtandao") || lower.contains("network") || lower.contains("wifi") {
        "network".into()
    } else {
        "general".into()
    };
    if symptoms.is_empty() {
        symptoms.push("user_reported".into());
    }
    (trade, symptoms)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn infer_computer() {
        let (t, s) = infer_trade_symptoms("Kompyuta yangu inaenda polepole na ina joto");
        assert_eq!(t, "computer");
        assert!(s.iter().any(|x| x.contains("slow") || x.contains("heat") || x.contains("overheating") || x.contains("performance")));
    }
}
