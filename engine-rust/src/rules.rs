//! rules.rs — Rules engine (data-driven), kutoka `data/rules/*.json`.
//! Kila rule: conditions zote zinalazimika kulingana (AND), kisha action.
//! Operators: gt | gte | lt | lte | eq. Kipimo: ±0.001 kwa eq.
//! Input ni JSON (mfano outputs za formula). Hakuna AI.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    pub field: String,
    pub operator: String,
    pub value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleAction {
    pub action_type: String, // recommend | warn | reject
    pub message: String,
    #[serde(default)]
    pub solution: String,
    #[serde(default)]
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub id: String,
    pub name_sw: String,
    #[serde(default)]
    pub priority: i32,
    pub conditions: Vec<Condition>,
    pub action: RuleAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleFile {
    #[serde(default)]
    pub trade: String,
    pub rules: Vec<Rule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleMatch {
    pub id: String,
    pub name_sw: String,
    pub priority: i32,
    pub action: RuleAction,
    pub matched_conditions: usize,
}

pub struct RulesEngine {
    rules: Vec<Rule>,
    trade: String,
}

fn get_field(data: &serde_json::Value, field: &str) -> Option<f64> {
    // rudisha f64: direct number, au njia ya "a.b"
    if let Some(v) = data.get(field) {
        if let Some(n) = v.as_f64() {
            return Some(n);
        }
        if let Some(b) = v.as_bool() {
            return Some(if b { 1.0 } else { 0.0 });
        }
    }
    None
}

impl RulesEngine {
    pub fn new() -> Self {
        RulesEngine { rules: Vec::new(), trade: String::new() }
    }

    pub fn from_json(text: &str) -> Result<Self, String> {
        let f: RuleFile =
            serde_json::from_str(text).map_err(|e| format!("rules JSON si sahihi: {}", e))?;
        let mut e = RulesEngine { rules: f.rules, trade: f.trade };
        e.rules.sort_by(|a, b| b.priority.cmp(&a.priority));
        Ok(e)
    }

    pub fn load_file(path: &std::path::Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path)
            .map_err(|e| format!("Haiwezi kusoma {}: {}", path.display(), e))?;
        Self::from_json(&text)
    }

    pub fn merge_json(&mut self, text: &str) -> Result<(), String> {
        let f: RuleFile =
            serde_json::from_str(text).map_err(|e| format!("rules JSON si sahihi: {}", e))?;
        self.rules.extend(f.rules);
        self.rules.sort_by(|a, b| b.priority.cmp(&a.priority));
        Ok(())
    }

    pub fn len(&self) -> usize { self.rules.len() }
    pub fn trade(&self) -> &str { &self.trade }

    fn matches(rule: &Rule, data: &serde_json::Value) -> Option<usize> {
        if rule.conditions.is_empty() {
            return None; // rule bila condition si halali
        }
        for c in &rule.conditions {
            let v = match get_field(data, &c.field) {
                Some(v) => v,
                None => return None, // field haipo => haijaailingana
            };
            let ok = match c.operator.as_str() {
                "gt" => v > c.value,
                "gte" => v >= c.value,
                "lt" => v < c.value,
                "lte" => v <= c.value,
                "eq" => (v - c.value).abs() < 0.001,
                _ => return None, // operator isiyojulikana
            };
            if !ok {
                return None;
            }
        }
        Some(rule.conditions.len())
    }

    /// Rudisha zote zilizolingana, zikipangwa kwa priority (kubwa kwanza).
    pub fn evaluate(&self, data: &serde_json::Value) -> Vec<RuleMatch> {
        let mut out = Vec::new();
        for r in &self.rules {
            if let Some(n) = Self::matches(r, data) {
                out.push(RuleMatch {
                    id: r.id.clone(),
                    name_sw: r.name_sw.clone(),
                    priority: r.priority,
                    action: r.action.clone(),
                    matched_conditions: n,
                });
            }
        }
        out.sort_by(|a, b| b.priority.cmp(&a.priority));
        out
    }

    /// Kipimo cha kwanza tu (kwa dashboard ya haraka).
    pub fn evaluate_first(&self, data: &serde_json::Value) -> Option<RuleMatch> {
        self.evaluate(data).into_iter().next()
    }

    pub fn ids(&self) -> Vec<String> {
        self.rules.iter().map(|r| r.id.clone()).collect()
    }

    pub fn by_id(&self, id: &str) -> Option<&Rule> {
        self.rules.iter().find(|r| r.id == id)
    }
}

/// Defaults patikanavyo kwa kila field (kwa ajili ya JSON completion).
pub fn defaults(fields: &[String], values: &HashMap<String, f64>) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    for f in fields {
        if let Some(v) = values.get(f) {
            map.insert(f.clone(), serde_json::json!(v));
        }
    }
    serde_json::Value::Object(map)
}

#[cfg(test)]
mod tests {
    use super::*;

    const RULES: &str = r#"{
      "trade": "umeme",
      "rules": [
        {"id":"VOLTAGE_DROP_HIGH","name_sw":"Kushuka kwa voltage kubwa","priority":30,
         "conditions":[{"field":"vd_pct","operator":"gt","value":5}],
         "action":{"action_type":"reject","message":"Kushuka kwa voltage ni kubwa kuliko 5%","solution":"Tumia cable kubwa zaidi","confidence":0.95}},
        {"id":"VOLTAGE_DROP_WARNING","name_sw":"Kushuka kwa voltage ya wastani","priority":20,
         "conditions":[{"field":"vd_pct","operator":"gte","value":3},{"field":"vd_pct","operator":"lt","value":5}],
         "action":{"action_type":"warn","message":"Kushuka kwa voltage kati ya 3-5%","solution":"Ongeza ukubwa wa cable","confidence":0.8}},
        {"id":"VOLTAGE_DROP_GOOD","name_sw":"Kushuka kwa voltage nzuri","priority":10,
         "conditions":[{"field":"vd_pct","operator":"lte","value":3}],
         "action":{"action_type":"recommend","message":"Kushuka kwa voltage chini ya 3%","solution":"Cable inatosha","confidence":0.9}},
        {"id":"BREAKER_UNDERSIZED","name_sw":"Breaker ndogo","priority":25,
         "conditions":[{"field":"breaker_a","operator":"lt","value":16}],
         "action":{"action_type":"warn","message":"Breaker ni ndogo kwa load hii","solution":"Tumia breaker ya angalau 16A","confidence":0.85}}
      ]
    }"#;

    #[test]
    fn loads_sorted_by_priority() {
        let e = RulesEngine::from_json(RULES).unwrap();
        assert_eq!(e.len(), 4);
        assert_eq!(e.ids()[0], "VOLTAGE_DROP_HIGH"); // priority 30 kwanza
    }

    #[test]
    fn high_drop_rejects() {
        let e = RulesEngine::from_json(RULES).unwrap();
        let m = e.evaluate_first(&serde_json::json!({"vd_pct": 12.17})).unwrap();
        assert_eq!(m.id, "VOLTAGE_DROP_HIGH");
        assert_eq!(m.action.action_type, "reject");
    }

    #[test]
    fn warning_needs_both_conditions() {
        let e = RulesEngine::from_json(RULES).unwrap();
        let m = e.evaluate_first(&serde_json::json!({"vd_pct": 4.0})).unwrap();
        assert_eq!(m.id, "VOLTAGE_DROP_WARNING");
        assert_eq!(m.matched_conditions, 2);
        // priority juu zaidi (HIGH, 30) haikulingana => WARNING (20) inashinda BREAKER (25)
        // lakini BREAKER haipati field breaker_a => haikulingani
        let all = e.evaluate(&serde_json::json!({"vd_pct": 4.0, "breaker_a": 10}));
        assert_eq!(all[0].id, "BREAKER_UNDERSIZED"); // priority 25 > 20
        assert_eq!(all[1].id, "VOLTAGE_DROP_WARNING");
    }

    #[test]
    fn good_drop() {
        let e = RulesEngine::from_json(RULES).unwrap();
        let m = e.evaluate_first(&serde_json::json!({"vd_pct": 2.92, "breaker_a": 20})).unwrap();
        assert_eq!(m.id, "VOLTAGE_DROP_GOOD");
        assert_eq!(m.action.action_type, "recommend");
    }

    #[test]
    fn missing_field_means_no_match() {
        let e = RulesEngine::from_json(RULES).unwrap();
        assert!(e.evaluate_first(&serde_json::json!({"tofauti": 1})).is_none());
    }

    #[test]
    fn unknown_operator_never_matches() {
        let bad = r#"{"rules":[{"id":"X","name_sw":"X","priority":1,
          "conditions":[{"field":"a","operator":"like","value":1}],
          "action":{"action_type":"warn","message":"m","solution":"","confidence":0.5}}]}"#;
        let e = RulesEngine::from_json(bad).unwrap();
        assert!(e.evaluate_first(&serde_json::json!({"a": 1})).is_none());
    }
}
