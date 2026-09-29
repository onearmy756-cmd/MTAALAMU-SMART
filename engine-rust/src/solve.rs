//! Software solve (baada ya HITL) + hardware guide (binadamu)
//! Knowledge: problems JSON causes/symptoms + solve_policy.json

use crate::remediate::{self, RemediationResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolvePolicy {
    pub software_cause_keywords: Vec<String>,
    pub hardware_cause_keywords: Vec<String>,
    pub software_actions: Vec<PolicyAction>,
    pub hardware_guide_template_sw: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyAction {
    pub id: String,
    pub title_sw: String,
    pub risk: String,
    pub requires_hitl: bool,
    pub matches: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SolvePlan {
    pub domain: String, // software | hardware | mixed | unknown
    pub reasons: Vec<String>,
    pub software_action_ids: Vec<String>,
    pub hardware_guide_sw: Option<String>,
    pub knowledge_solution_sw: Option<String>,
    pub problem_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SolveResult {
    pub plan: SolvePlan,
    pub hitl_approved: bool,
    pub executed: Vec<RemediationResult>,
    pub skipped: Vec<String>,
    pub summary_sw: String,
}

fn load_policy(data_root: &Path) -> SolvePolicy {
    let p = data_root.join("vision/solve_policy.json");
    if p.exists() {
        if let Ok(t) = fs::read_to_string(&p) {
            if let Ok(pol) = serde_json::from_str::<SolvePolicy>(&t) {
                return pol;
            }
        }
    }
    SolvePolicy {
        software_cause_keywords: vec![
            "virus".into(),
            "software_corrupt".into(),
            "temp".into(),
            "slow_pc".into(),
        ],
        hardware_cause_keywords: vec![
            "psu".into(),
            "hdd_fail".into(),
            "ram_bad".into(),
            "motherboard".into(),
            "overheat".into(),
        ],
        software_actions: vec![],
        hardware_guide_template_sw: "HARDWARE: binadamu anasolve. Agent inaonyesha tu.".into(),
    }
}

fn text_blob(problem: &Value) -> String {
    let mut s = String::new();
    if let Some(id) = problem.get("id").and_then(|x| x.as_str()) {
        s.push_str(id);
        s.push(' ');
    }
    if let Some(t) = problem.get("trade").and_then(|x| x.as_str()) {
        s.push_str(t);
        s.push(' ');
    }
    if let Some(arr) = problem.get("symptoms").and_then(|x| x.as_array()) {
        for v in arr {
            if let Some(x) = v.as_str() {
                s.push_str(x);
                s.push(' ');
            }
        }
    }
    if let Some(c) = problem.get("causes").and_then(|x| x.as_object()) {
        for k in c.keys() {
            s.push_str(k);
            s.push(' ');
        }
    }
    if let Some(sol) = problem
        .get("solution")
        .and_then(|x| x.get("sw").or_else(|| x.get("en")))
        .and_then(|x| x.as_str())
    {
        s.push_str(sol);
    }
    s.to_lowercase()
}

fn classify(blob: &str, policy: &SolvePolicy) -> (String, Vec<String>) {
    let mut sw = 0;
    let mut hw = 0;
    let mut reasons = Vec::new();
    for k in &policy.software_cause_keywords {
        if blob.contains(&k.to_lowercase()) {
            sw += 1;
            reasons.push(format!("software_kw:{}", k));
        }
    }
    for k in &policy.hardware_cause_keywords {
        if blob.contains(&k.to_lowercase()) {
            hw += 1;
            reasons.push(format!("hardware_kw:{}", k));
        }
    }
    let domain = if sw > 0 && hw > 0 {
        "mixed"
    } else if sw > 0 {
        "software"
    } else if hw > 0 {
        "hardware"
    } else {
        "unknown"
    };
    (domain.into(), reasons)
}

fn pick_actions(blob: &str, policy: &SolvePolicy) -> Vec<String> {
    let mut ids = Vec::new();
    for a in &policy.software_actions {
        for m in &a.matches {
            if blob.contains(&m.to_lowercase()) {
                if !ids.contains(&a.id) {
                    ids.push(a.id.clone());
                }
                break;
            }
        }
    }
    // default software hygiene if classified software but no match
    if ids.is_empty() && (blob.contains("slow") || blob.contains("temp") || blob.contains("disk"))
    {
        ids.push("report_top_cpu".into());
        ids.push("list_temp".into());
    }
    ids
}

/// Build plan from one matched problem object (JSON)
pub fn plan_from_problem(data_root: &Path, problem: &Value) -> SolvePlan {
    let policy = load_policy(data_root);
    let blob = text_blob(problem);
    let (domain, reasons) = classify(&blob, &policy);
    let software_action_ids = if domain == "software" || domain == "mixed" {
        pick_actions(&blob, &policy)
    } else {
        vec![]
    };
    let hardware_guide_sw = if domain == "hardware" || domain == "mixed" {
        let sol = problem
            .get("solution")
            .and_then(|x| x.get("sw").or_else(|| x.get("en")))
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string();
        Some(format!(
            "{}\n\nKnowledge: {}",
            policy.hardware_guide_template_sw, sol
        ))
    } else {
        None
    };
    let knowledge_solution_sw = problem
        .get("solution")
        .and_then(|x| x.get("sw").or_else(|| x.get("en")))
        .and_then(|x| x.as_str())
        .map(|s| s.to_string());
    let problem_id = problem
        .get("id")
        .and_then(|x| x.as_str())
        .map(|s| s.to_string());

    SolvePlan {
        domain,
        reasons,
        software_action_ids,
        hardware_guide_sw,
        knowledge_solution_sw,
        problem_id,
    }
}

/// Execute software actions only if HITL approved; never "fix" hardware
pub fn execute_plan(data_root: &Path, plan: &SolvePlan, hitl_approved: bool) -> SolveResult {
    let mut executed = Vec::new();
    let mut skipped = Vec::new();

    if plan.domain == "hardware" {
        return SolveResult {
            plan: plan.clone(),
            hitl_approved,
            executed,
            skipped: vec!["hardware: no auto-execute".into()],
            summary_sw: plan
                .hardware_guide_sw
                .clone()
                .unwrap_or_else(|| "Hardware — binadamu anasolve.".into()),
        };
    }

    if !hitl_approved {
        return SolveResult {
            plan: plan.clone(),
            hitl_approved: false,
            executed,
            skipped: plan.software_action_ids.clone(),
            summary_sw: "HITL haijaruhusiwa — hakuna software action.".into(),
        };
    }

    for id in &plan.software_action_ids {
        // clear_user_temp always needs hitl — already approved at session level
        let r = remediate::run_action(id, true);
        remediate::log_result(data_root, &r);
        if r.ok {
            executed.push(r);
        } else {
            skipped.push(format!("{}: {}", id, r.message_sw));
        }
    }

    let summary_sw = if plan.domain == "mixed" {
        format!(
            "SOFTWARE: {} vitendo. HARDWARE: binadamu — {}",
            executed.len(),
            plan.hardware_guide_sw.clone().unwrap_or_default()
        )
    } else if executed.is_empty() {
        format!(
            "Hakuna action iliyotekelezwa. Knowledge: {}",
            plan.knowledge_solution_sw.clone().unwrap_or_default()
        )
    } else {
        format!(
            "Software solve: {} vitendo vimefanikiwa. {}",
            executed.len(),
            plan.knowledge_solution_sw.clone().unwrap_or_default()
        )
    };

    SolveResult {
        plan: plan.clone(),
        hitl_approved,
        executed,
        skipped,
        summary_sw,
    }
}

/// Load problems.json and find by id
pub fn find_problem(data_root: &Path, problem_id: &str) -> Option<Value> {
    let p = data_root.join("problems.json");
    let t = fs::read_to_string(p).ok()?;
    let v: Value = serde_json::from_str(&t).ok()?;
    let arr = v.get("problems").and_then(|x| x.as_array())?;
    arr.iter()
        .find(|p| p.get("id").and_then(|x| x.as_str()) == Some(problem_id))
        .cloned()
}

/// Match simple keyword score (subset of R bridge)
pub fn match_problem_id(data_root: &Path, msg: &str) -> Option<String> {
    let p = data_root.join("problems.json");
    let t = fs::read_to_string(p).ok()?;
    let v: Value = serde_json::from_str(&t).ok()?;
    let arr = v.get("problems").and_then(|x| x.as_array())?;
    let msg_l = msg.to_lowercase();
    let tokens: Vec<&str> = msg_l
        .split(|c: char| !c.is_alphanumeric() && c != '_')
        .filter(|s| s.len() > 2)
        .collect();
    let mut best: Option<(i32, String)> = None;
    for prob in arr {
        let mut score = 0i32;
        let blob = text_blob(prob);
        for t in &tokens {
            if blob.contains(t) {
                score += 1;
            }
        }
        if score > 0 {
            if best.as_ref().map(|(s, _)| score > *s).unwrap_or(true) {
                if let Some(id) = prob.get("id").and_then(|x| x.as_str()) {
                    best = Some((score, id.to_string()));
                }
            }
        }
    }
    best.map(|(_, id)| id)
}

pub fn solve_message(data_root: &Path, msg: &str, hitl_approved: bool) -> SolveResult {
    let plan = if let Some(pid) = match_problem_id(data_root, msg) {
        if let Some(prob) = find_problem(data_root, &pid) {
            plan_from_problem(data_root, &prob)
        } else {
            // synthetic from message
            let synthetic = serde_json::json!({
                "id": "msg",
                "symptoms": [msg],
                "causes": {},
                "solution": { "sw": msg }
            });
            plan_from_problem(data_root, &synthetic)
        }
    } else {
        let synthetic = serde_json::json!({
            "id": "unknown",
            "symptoms": [msg],
            "solution": { "sw": "Hakuna match — OS probe + hygiene tu." }
        });
        plan_from_problem(data_root, &synthetic)
    };
    execute_plan(data_root, &plan, hitl_approved)
}
