//! Agentic full run — Vision HALISI (sysprobe) + Knowledge + Pipeline + Solve + Report + Learn
//! Kanuni: Agent inafanya kazi zote; msimamizi anaruhusu/kusimamia tu (HITL).

use crate::agents::{AgentOrchestrator, AgentSession};
use crate::knowledge::KnowledgeBase;
use crate::pipeline::PipelineEngine;
use crate::report::ReportEngine;
use crate::solve::{execute_plan, plan_from_problem, solve_message, SolveResult};
use crate::vision::VisionEngine;
use serde_json::{json, Value};
use std::fs;
use std::path::Path;

#[derive(Debug)]
pub struct AgenticResult {
    pub session: AgentSession,
    pub vision: Value,
    pub diagnosis_hits: Value,
    pub pipeline: Value,
    pub report: Value,
    pub report_md: String,
    pub solve: Option<SolveResult>,
}

pub fn run_full(
    data_root: &Path,
    user_message: &str,
    language: &str,
    approve: bool,
) -> Result<AgenticResult, String> {
    let agents_path = data_root.join("agents/agents_10.json");
    let orch = AgentOrchestrator::load(&agents_path)?;
    let mut session = orch.start_session(user_message, language);

    // Vision HALISI — OS sysinfo
    let vision_eng = VisionEngine::load(data_root)?;
    let snap = vision_eng.scan_live();
    let vision_json = serde_json::to_value(&snap).unwrap_or(Value::Null);
    session.context.insert("vision".into(), vision_json.clone());
    session.context.insert("vision_source".into(), json!(snap.source));

    // Knowledge
    let mut kb = KnowledgeBase::new();
    let problems_path = data_root.join("problems.json");
    if problems_path.exists() {
        let _ = kb.load_file(&problems_path);
    }
    let hits = kb.diagnose(&session.trade, &session.symptoms);
    let hits_json = serde_json::to_value(&hits).unwrap_or(json!([]));
    session.context.insert("diagnosis".into(), hits_json.clone());

    let issues_summary: Vec<String> = snap.issues.iter().map(|i| i.title.clone()).collect();
    session.context.insert("issues".into(), json!(issues_summary));

    // Pre-plan software actions from message (agent decides; human only approves)
    let solve_preview = solve_message(data_root, user_message, false);
    session.context.insert(
        "solve_plan".into(),
        serde_json::to_value(&solve_preview.plan).unwrap_or(Value::Null),
    );

    // Pipeline
    let pipe_path = data_root.join("vision/pipeline.json");
    let pipe = PipelineEngine::load(&pipe_path)?;
    let mut rt = pipe.start(&session.id);

    let mut events = Vec::new();
    let mut solve_result: Option<SolveResult> = None;

    for _ in 0..16 {
        if approve && !session.hitl_approved {
            let _ = orch.approve_hitl(&mut session);
        }
        match orch.advance(&mut session) {
            Ok(ev) => {
                // AGENT WORK: when solver step completes and HITL approved → execute real fixes
                if ev.agent_id == "solver" && session.hitl_approved && solve_result.is_none() {
                    let executed = solve_message(data_root, user_message, true);
                    session.context.insert(
                        "solve_executed".into(),
                        serde_json::to_value(&executed).unwrap_or(Value::Null),
                    );
                    session.context.insert(
                        "actions_done".into(),
                        json!(executed.executed.iter().map(|r| &r.action_id).collect::<Vec<_>>()),
                    );
                    solve_result = Some(executed);
                }

                // Tester: re-probe after solve
                if ev.agent_id == "tester" {
                    let after = vision_eng.scan_live();
                    session.context.insert(
                        "post_fix_vision".into(),
                        serde_json::to_value(&after).unwrap_or(Value::Null),
                    );
                }

                events.push(ev.clone());
                let _ = pipe.advance(
                    &mut rt,
                    json!({"agent": ev.agent_id, "event": ev.event}),
                    &ev.message_sw,
                    session.hitl_approved,
                );
                if session.state.as_str() == "completed" || session.state.as_str() == "failed" {
                    break;
                }
                if session.state.as_str() == "awaiting_hitl" && !approve {
                    break;
                }
            }
            Err(e) => {
                session.context.insert("last_error".into(), json!(e));
                break;
            }
        }
    }

    // If approved but solver never ran (edge), still execute
    if approve && solve_result.is_none() {
        let executed = solve_message(data_root, user_message, true);
        session.context.insert(
            "solve_executed".into(),
            serde_json::to_value(&executed).unwrap_or(Value::Null),
        );
        solve_result = Some(executed);
    }

    // Report
    let report_path = data_root.join("vision/report_template.json");
    let report_eng = ReportEngine::load(&report_path)?;
    let top_solution = solve_result
        .as_ref()
        .map(|s| s.summary_sw.clone())
        .or_else(|| hits.first().map(|h| h.solution.clone()))
        .unwrap_or_else(|| {
            snap.issues
                .first()
                .map(|i| i.action.clone())
                .unwrap_or_else(|| "Angalia OS probe na HITL.".into())
        });
    let actions_n = solve_result
        .as_ref()
        .map(|s| s.executed.len())
        .unwrap_or(0);
    let ctx = json!({
        "user_message": user_message,
        "symptoms": session.symptoms,
        "status": session.state.as_str(),
        "summary_sw": format!(
            "Chanzo: {}. Biashara: {}. Matatizo: {}. Vitendo vya agent: {}. Suluhisho: {}",
            snap.source, session.trade, issues_summary.len(), actions_n, top_solution
        ),
        "issues": issues_summary,
        "components_summary": {
            "critical": snap.summary.critical,
            "warning": snap.summary.warning,
            "good": snap.summary.good,
        },
        "processes_critical": snap.summary.critical_processes,
        "posteriors": hits_json,
        "plan_steps": pipe.defs().iter().map(|d| d.name_sw.clone()).collect::<Vec<_>>(),
        "customer": "Mteja",
        "device": snap.probe.as_ref()
            .and_then(|v| v.get("hostname"))
            .and_then(|h| h.as_str())
            .unwrap_or("host"),
        "trade": session.trade,
        "vision_source": snap.source,
        "agent_actions": actions_n,
        "hitl": session.hitl_approved,
    });
    let report = report_eng.build(&session.id, &ctx, language);
    let report_md = report_eng.to_markdown(&report);
    let report_json = serde_json::to_value(&report).unwrap_or(Value::Null);

    // Learner — hifadhi faili HALISI
    if let Some(h) = hits.first() {
        let _ = kb.learn(&h.problem_id, session.state.as_str() == "completed");
    }
    let _ = append_learning_log(
        data_root,
        &session.id,
        user_message,
        &issues_summary,
        session.state.as_str(),
        &snap.source,
        actions_n,
    );

    session.context.insert("events_count".into(), json!(events.len()));

    Ok(AgenticResult {
        session,
        vision: vision_json,
        diagnosis_hits: hits_json,
        pipeline: serde_json::to_value(&rt).unwrap_or(Value::Null),
        report: report_json,
        report_md,
        solve: solve_result,
    })
}

fn append_learning_log(
    data_root: &Path,
    session_id: &str,
    msg: &str,
    issues: &[String],
    status: &str,
    source: &str,
    actions: usize,
) -> Result<(), String> {
    let path = data_root.join("learning_log.json");
    let mut arr = if path.exists() {
        let t = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        serde_json::from_str::<Value>(&t)
            .ok()
            .and_then(|v| v.as_array().cloned())
            .unwrap_or_default()
    } else {
        vec![]
    };
    arr.push(json!({
        "session_id": session_id,
        "ts": chrono_ts(),
        "user_message": msg,
        "issues": issues,
        "status": status,
        "vision_source": source,
        "agent_actions": actions,
    }));
    if arr.len() > 200 {
        arr = arr.split_off(arr.len() - 200);
    }
    let pretty = serde_json::to_string_pretty(&arr).map_err(|e| e.to_string())?;
    fs::write(&path, pretty).map_err(|e| e.to_string())?;
    Ok(())
}

fn chrono_ts() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Public helper: plan only (no execute) — for UI preview before HITL
pub fn preview_plan(data_root: &Path, user_message: &str) -> SolveResult {
    solve_message(data_root, user_message, false)
}

/// Public helper: execute after supervisor approve
pub fn execute_approved(data_root: &Path, user_message: &str) -> SolveResult {
    solve_message(data_root, user_message, true)
}

#[allow(dead_code)]
fn _plan_from_first_hit(data_root: &Path, problem: &Value) -> crate::solve::SolvePlan {
    plan_from_problem(data_root, problem)
}

#[allow(dead_code)]
fn _execute(data_root: &Path, plan: &crate::solve::SolvePlan, ok: bool) -> SolveResult {
    execute_plan(data_root, plan, ok)
}
