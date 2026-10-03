//! skills.rs — SkillsEngine: skills ZOTE kutoka data/skills/skills.json (data-driven, KANUNI 2/5)
//!
//! Kazi:
//!  - load(data_dir)            — soma skills zote (138 za trades + 8 za AI/web/PDF)
//!  - search(query)             — tafuta skill kwa jina/trade/tool
//!  - estimate(skill, inputs)   — hesabu halisi ya muda kwa expr engine (LLM HAIHESABU)
//!  - plan(msg)                 — ujumbe wa mtumiaji → skills + mpango wa kazi (PIITVD)
//!  - run(skill_id, inputs)     — TEKELEZA action halisi:
//!        duckduckgo_search | searxng_search | pdf_search | ollama_ask | hf_search
//!        pytorch_train | tensorflow_train (bridges za python3)
//!        skill_plan | guide (mwongozo + hesabu)
//!
//! Kanuni: LLM haihesabu (R-1); content yote kutoka JSON; HITL kwa guide za hardware.

use crate::expr;
use crate::pdf_extract;
use crate::web;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SkillInput {
    pub name: String,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub en: String,
    #[serde(default = "d_one")]
    pub default: f64,
}

fn d_one() -> f64 {
    1.0
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Skill {
    pub id: String,
    pub trade: String,
    pub name_sw: String,
    #[serde(default)]
    pub name_en: String,
    #[serde(default = "d_two")]
    pub difficulty: u8,
    #[serde(default)]
    pub formula: String,
    #[serde(default)]
    pub inputs: Vec<SkillInput>,
    #[serde(default)]
    pub tools: Vec<String>,
    #[serde(default)]
    pub action: String,
    #[serde(default)]
    pub runtime: String,
    #[serde(default)]
    pub script: String,
}

fn d_two() -> u8 {
    2
}

#[derive(Debug, Clone)]
pub struct SkillsEngine {
    pub skills: Vec<Skill>,
    data_dir: PathBuf,
}

#[derive(Debug, Clone, Serialize)]
pub struct SkillMatch {
    pub skill: Skill,
    pub score: i64,
    pub muda_min: f64,
}

impl SkillsEngine {
    /// Soma data/skills/skills.json — skills zote (trades + AI/web/PDF).
    pub fn load(data_dir: &Path) -> Result<Self, String> {
        let path = data_dir.join("skills").join("skills.json");
        let s = std::fs::read_to_string(&path)
            .map_err(|e| format!("skills.json: {}", e))?;
        let v: Value = serde_json::from_str(&s).map_err(|e| format!("skills parse: {}", e))?;
        let skills: Vec<Skill> = serde_json::from_value(v["skills"].clone())
            .map_err(|e| format!("skills schema: {}", e))?;
        if skills.is_empty() {
            return Err("skills.json haina skills".into());
        }
        Ok(SkillsEngine {
            skills,
            data_dir: data_dir.to_path_buf(),
        })
    }

    pub fn stats(&self) -> Value {
        let mut by_trade: HashMap<String, i64> = HashMap::new();
        let mut by_action: HashMap<String, i64> = HashMap::new();
        for s in &self.skills {
            *by_trade.entry(s.trade.clone()).or_default() += 1;
            *by_action.entry(s.action.clone()).or_default() += 1;
        }
        json!({
            "total": self.skills.len(),
            "trades": by_trade.len(),
            "actions": by_action,
            "by_trade": by_trade,
        })
    }

    /// Tafuta skill kwa maneno (jina sw/en, trade, id, tools).
    pub fn search(&self, query: &str, limit: usize) -> Vec<SkillMatch> {
        let terms: Vec<String> = query
            .to_lowercase()
            .split_whitespace()
            .map(|s| s.to_string())
            .filter(|s| !s.is_empty())
            .collect();
        if terms.is_empty() {
            return Vec::new();
        }
        let mut out = Vec::new();
        for s in &self.skills {
            let hay = format!(
                "{} {} {} {} {}",
                s.id,
                s.trade,
                s.name_sw.to_lowercase(),
                s.name_en.to_lowercase(),
                s.tools.join(" ").to_lowercase()
            );
            let mut score = 0i64;
            for t in &terms {
                if hay.contains(t.as_str()) {
                    score += 10;
                }
                if s.id.to_lowercase().contains(t.as_str()) {
                    score += 15;
                }
                if s.trade == *t {
                    score += 25;
                }
            }
            if score > 0 {
                let muda = self.estimate_defaults(s);
                out.push(SkillMatch {
                    skill: s.clone(),
                    score,
                    muda_min: muda,
                });
            }
        }
        out.sort_by(|a, b| b.score.cmp(&a.score));
        out.truncate(limit);
        out
    }

    /// Hesabu muda kwa formula ya skill na inputs halisi (expr engine — deterministic).
    pub fn estimate(&self, skill_id: &str, inputs: &HashMap<String, f64>) -> Result<Value, String> {
        let s = self.get(skill_id)?;
        let mut vars: expr::Vars = HashMap::new();
        let mut used: Vec<String> = Vec::new();
        for inp in &s.inputs {
            let v = inputs.get(&inp.name).copied().unwrap_or(inp.default);
            vars.insert(inp.name.clone(), v);
            used.push(format!("{}={}", inp.name, v));
        }
        if s.formula.is_empty() {
            return Ok(json!({"skill": s.id, "muda_min": 30.0, "inputs": used}));
        }
        let result = expr::eval(&strip_assignment(&s.formula), &vars)
            .map_err(|e| format!("formula: {}", e))?;
        Ok(json!({
            "skill": s.id,
            "trade": s.trade,
            "name_sw": s.name_sw,
            "formula": s.formula,
            "inputs": used,
            "muda_min": (result * 100.0).round() / 100.0,
        }))
    }

    fn estimate_defaults(&self, s: &Skill) -> f64 {
        let mut vars: expr::Vars = HashMap::new();
        for inp in &s.inputs {
            vars.insert(inp.name.clone(), inp.default);
        }
        expr::eval(&strip_assignment(&s.formula), &vars).unwrap_or(30.0)
    }

    pub fn get(&self, skill_id: &str) -> Result<Skill, String> {
        self.skills
            .iter()
            .find(|s| s.id == skill_id || s.id.ends_with(skill_id))
            .cloned()
            .ok_or_else(|| format!("skill '{}' haipo (tumia: mtaalamu skills --q ...)", skill_id))
    }

    /// Mpango wa kazi: ujumbe wa mtumiaji → skills zinazolingana + hatua PIITVD.
    pub fn plan(&self, msg: &str) -> Value {
        let matches = self.search(msg, 5);
        let steps: Vec<Value> = matches
            .iter()
            .map(|m| {
                json!({
                    "step": m.skill.id,
                    "name_sw": m.skill.name_sw,
                    "trade": m.skill.trade,
                    "action": m.skill.action,
                    "muda_min": m.muda_min,
                    "tools": m.skill.tools,
                })
            })
            .collect();
        let total: f64 = matches.iter().map(|m| m.muda_min).sum();
        json!({
            "msg": msg,
            "matches": matches.len(),
            "plan": steps,
            "total_min": (total * 10.0).round() / 10.0,
            "pipeline": ["plan", "identify", "implement", "test", "verify", "document"],
            "note_sw": "Kila hatua inaweza kutekelezwa: mtaalamu skills-run --skill <id> --inputs '{...}'",
        })
    }

    /// TEKELEZA skill halisi (action). Guide = mwongozo + hesabu (HITL kwa hardware).
    pub fn run(
        &self,
        skill_id: &str,
        inputs: &HashMap<String, f64>,
        msg: &str,
        approve: bool,
    ) -> Result<Value, String> {
        let s = self.get(skill_id)?;
        let est = self.estimate(&s.id, inputs)?;
        let base = json!({
            "skill": s.id,
            "trade": s.trade,
            "name_sw": s.name_sw,
            "action": s.action,
            "estimate": est,
        });
        match s.action.as_str() {
            "duckduckgo_search" => {
                let q = query_from(msg, inputs, "kompyuta matatizo Tanzania");
                let limit = inputs.get("pages").copied().unwrap_or(5.0) as usize;
                let r = web::ddg_search(&q, limit.max(1).min(20))?;
                Ok(merged(base, json!({"status": "ok", "result": r})))
            }
            "searxng_search" => {
                let q = query_from(msg, inputs, "fundi Tanzania");
                let limit = inputs.get("engines").copied().unwrap_or(4.0) as usize;
                let r = web::searxng_search(&q, (limit * 3).max(3).min(30))?;
                Ok(merged(base, json!({"status": "ok", "result": r})))
            }
            "pdf_search" => {
                let q = if msg.trim().is_empty() { "mtaalamu" } else { msg };
                let dir = self
                    .data_dir
                    .join("pdfs");
                let limit = inputs.get("files").copied().unwrap_or(20.0) as usize;
                let r = pdf_extract::search_pdfs(&dir, q, 10, limit.max(1))?;
                let v = serde_json::to_value(&r).unwrap_or(Value::Null);
                Ok(merged(base, json!({"status": "ok", "result": v})))
            }
            "ollama_ask" => {
                let cfg = web::ollama_config(&self.data_dir.join("ai").join("models.json"));
                let model = model_from_inputs(inputs, &self.data_dir)
                    .unwrap_or_else(|| default_model(&self.data_dir));
                let r = web::ollama_chat(&cfg, &model, SYSTEM_SW, msg)?;
                Ok(merged(base, json!({"status": "ok", "result": r})))
            }
            "hf_search" => {
                let q = query_from(msg, inputs, "swahili translation");
                let limit = inputs.get("downloads").copied().unwrap_or(100.0) as usize;
                let limit = limit.clamp(3, 25);
                let hub = web::hf_search_models(&q, limit)?;
                Ok(merged(base, json!({"status": "ok", "result": hub})))
            }
            "pytorch_train" | "tensorflow_train" => {
                run_python_bridge(&s, msg, approve).map(|r| merged(base, json!({"status": "ok", "result": r})))
            }
            "skill_plan" => Ok(merged(base, json!({"status": "ok", "result": self.plan(msg)}))),
            _ => {
                // guide: mwongozo wa binadamu (HITL) + hesabu halisi + zana
                if !approve {
                    let steps = vec![
                        format!("1. Kusanya zana: {}", s.tools.join(", ")),
                        "2. Pima kwanza (usiharibu): angalia dalili zote".to_string(),
                        format!("3. Tekeleza kwa mpango: {}", s.name_sw),
                        "4. Jaribu + thibitisha kwa mteja".to_string(),
                    ];
                    return Ok(merged(base, json!({
                        "status": "awaiting_approval",
                        "hitl_sw": "Hii ni skill ya mwongozo (hardware/kazi ya mikono). Idhini ya msimamizi inahitajika: ongeza --approve",
                        "tools": s.tools,
                        "steps_sw": steps,
                    })));
                }
                Ok(merged(base, json!({
                    "status": "approved_guide",
                    "tools": s.tools,
                    "note_sw": "Mwongozo umepitishwa (HITL). Tekeleza kwa usalama.",
                })))
            }
        }
    }
}

/// Formula za skills ni "muda = 45 + (x * 5)" — expr engine inataka RHS tu.
fn strip_assignment(f: &str) -> &str {
    match f.split_once('=') {
        // assignment: "muda = ..." — lhs ni jina moja, rhs si "= ..." (==)
        Some((lhs, rhs))
            if !lhs.is_empty()
                && lhs
                    .chars()
                    .all(|c| c.is_alphanumeric() || c == '_' || c == ' ')
                && !rhs.trim_start().starts_with('=') =>
        {
            rhs.trim()
        }
        _ => f.trim(),
    }
}

const SYSTEM_SW: &str = "Wewe ni msaidizi wa MTAALAMU SMART (Tanzania). Jibu kwa Kiswahili fasaha, fupi na cha ukweli. HUHESABU hesabu za kiufundi — hesabu zinafanywa na injini ya Rust. Kazi yako: kueleza, kutafsiri, kufupisha.";

/// Unganisha Value mbili za JSON (extra inashinda juu ya base).
fn merged(mut base: Value, extra: Value) -> Value {
    if let (Some(a), Some(b)) = (base.as_object_mut(), extra.as_object()) {
        for (k, v) in b {
            a.insert(k.clone(), v.clone());
        }
    }
    base
}

fn query_from(msg: &str, inputs: &HashMap<String, f64>, fallback: &str) -> String {
    // --input q=N kama string? inputs ni f64; hivyo msg ndiyo chanzo.
    let _ = inputs;
    let m = msg.trim();
    if m.is_empty() {
        fallback.to_string()
    } else {
        m.to_string()
    }
}

/// Modeli default kutoka models.json (routing.default_chat) — fallback: gpt-oss:20b cloud.
fn default_model(data_dir: &Path) -> String {
    if let Ok(s) = std::fs::read_to_string(data_dir.join("ai").join("models.json")) {
        if let Ok(v) = serde_json::from_str::<Value>(&s) {
            if let Some(m) = v["routing"]["default_chat"].as_str() {
                return m.to_string();
            }
        }
    }
    "gpt-oss:20b".into()
}

fn model_from_inputs(inputs: &HashMap<String, f64>, data_dir: &Path) -> Option<String> {
    // model inaweza kuchaguliwa kwa input "model_id" (f64 index kwenye models.json)
    let idx = inputs.get("model_id").copied()?;
    let s = std::fs::read_to_string(data_dir.join("ai").join("models.json")).ok()?;
    let v: Value = serde_json::from_str(&s).ok()?;
    let models = v["models"].as_array()?;
    let i = (idx as usize).min(models.len().saturating_sub(1));
    models.get(i)?["id"].as_str().map(|s| s.to_string())
}

/// Bridges za PyTorch/TensorFlow: python3 scripts/ai_pytorch.py / ai_tensorflow.py
fn run_python_bridge(s: &Skill, msg: &str, approve: bool) -> Result<Value, String> {
    if !approve {
        return Err("HITL: training ya ML inahitaji --approve (CPU/muda halisi unatumika)".into());
    }
    let script = if s.script.is_empty() {
        format!("scripts/ai_{}.py", s.action.trim_end_matches("_train"))
    } else {
        s.script.clone()
    };
    let path = PathBuf::from(&script);
    let path = if path.exists() {
        path
    } else {
        PathBuf::from("..").join(&script)
    };
    if !path.exists() {
        return Err(format!("script haipo: {} (sakinisha torch/tensorflow kwanza)", script));
    }
    let out = std::process::Command::new("python3")
        .arg(&path)
        .arg("--task")
        .arg(if msg.trim().is_empty() { "demo" } else { msg })
        .output()
        .map_err(|e| format!("python3: {}", e))?;
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    if !out.status.success() {
        return Err(format!(
            "bridge imeshindwa ({}): {} | {}",
            out.status,
            stdout.chars().take(300).collect::<String>(),
            stderr.chars().take(500).collect::<String>()
        ));
    }
    // script inatoa JSON kwenye mstari wa mwisho
    let json_line = stdout
        .lines()
        .rev()
        .find(|l| l.trim_start().starts_with('{'))
        .unwrap_or("{}");
    let v: Value = serde_json::from_str(json_line).unwrap_or(json!({"raw": stdout}));
    Ok(v)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn engine() -> SkillsEngine {
        // data/ iko juu ya engine-rust/
        let data = Path::new("../data");
        SkillsEngine::load(data).or_else(|_| SkillsEngine::load(Path::new("data")))
            .expect("skills.json imeshindikana kupatikana")
    }

    #[test]
    fn test_load_all_skills() {
        let e = engine();
        assert!(e.skills.len() >= 146, "skills: {}", e.skills.len());
        let ai = e.skills.iter().filter(|s| s.trade == "ai").count();
        assert_eq!(ai, 8);
    }

    #[test]
    fn test_search_finds_umeme() {
        let e = engine();
        let r = e.search("cable voltage", 5);
        assert!(!r.is_empty());
        assert!(r[0].skill.trade == "umeme" || r[0].skill.id.contains("cable"));
    }

    #[test]
    fn test_search_ai_skills() {
        let e = engine();
        for q in ["pytorch", "searxng", "duckduckgo", "hugging", "pdf"] {
            let r = e.search(q, 3);
            assert!(!r.is_empty(), "query '{}' haikupata kitu", q);
        }
    }

    #[test]
    fn test_estimate_deterministic() {
        let e = engine();
        let s = e.get("ai.pytorch").unwrap();
        assert!(s.formula.contains("epochs"));
        let mut inputs = HashMap::new();
        inputs.insert("epochs".to_string(), 5.0);
        let v = e.estimate("ai.pytorch", &inputs).unwrap();
        assert_eq!(v["muda_min"], 40.0); // 30 + 5*2
    }

    #[test]
    fn test_strip_assignment() {
        assert_eq!(strip_assignment("muda = 45 + (x * 5)"), "45 + (x * 5)");
        assert_eq!(strip_assignment("30 + x"), "30 + x");
        assert_eq!(strip_assignment("vd == 3"), "vd == 3");
    }

    #[test]
    fn test_plan_generates_steps() {
        let e = engine();
        let p = e.plan("betri ya simu imeharibika");
        assert!(p["matches"].as_i64().unwrap_or(0) >= 1);
    }
}
