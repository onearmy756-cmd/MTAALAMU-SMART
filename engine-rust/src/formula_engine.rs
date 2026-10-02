//! formula_engine.rs — Formula Engine (data-driven).
//! Inasoma `data/formulas/*.json` na `data/formulas_network/*.json`,
//! kisha inahesabu bila kubadilisha code: kuongeza formula = kuongeza JSON.
//!
//! Kila result kurudisha: inputs, outputs (pamoja na steps), status
//! GOOD | WARNING | FAIL | OK, na ujumbe wa Kiswahili.

use crate::expr::{eval, Vars};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

// ---------- JSON shapes ----------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiText {
    pub sw: String,
    pub en: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiTextLists {
    #[serde(default)]
    pub sw: Vec<String>,
    #[serde(default)]
    pub en: Vec<String>,
}

/// Standards inaweza kuwa string moja au orodha (data halisi ina arrays).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Standards {
    Str(BiText),
    List(BiTextLists),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputSpec {
    pub name: String,
    pub label: BiText,
    #[serde(default)]
    pub unit: String,
    #[serde(default)]
    pub default: f64,
    #[serde(default)]
    pub min: Option<f64>,
    #[serde(default)]
    pub max: Option<f64>,
    #[serde(default)]
    pub select: Option<Vec<SelectOption>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectOption {
    pub label: String,
    pub value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputSpec {
    pub name: String,
    pub label: BiText,
    pub expr: String,
    #[serde(default)]
    pub unit: String,
    #[serde(default = "default_digits")]
    pub digits: u32,
}

fn default_digits() -> u32 { 2 }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Step {
    Text { label: String, text: String },
    Expr { label: String, expr: String, template: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub check: String,
    pub status: String, // GOOD | WARNING | FAIL | OK
    pub msg: BiText,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSpec {
    pub inputs: HashMap<String, f64>,
    pub expect: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Formula {
    pub id: String,
    #[serde(default)]
    pub trade: String,
    #[serde(default)]
    pub group: String,
    pub name: BiText,
    pub formula: String,
    pub description: BiText,
    pub inputs: Vec<InputSpec>,
    pub outputs: Vec<OutputSpec>,
    #[serde(default)]
    pub steps: Vec<Step>,
    #[serde(default)]
    pub rules: Vec<Rule>,
    #[serde(default)]
    pub standards: Option<Standards>,
    #[serde(default)]
    pub test: Option<TestSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormulaFile {
    #[serde(default)]
    pub trade: String,
    #[serde(default)]
    pub group: String,
    #[serde(default)]
    pub label: Option<BiText>,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub color: Option<String>,
    pub formulas: Vec<Formula>,
}

// ---------- Results ----------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputValue {
    pub name: String,
    pub label: BiText,
    pub value: f64,
    pub unit: String,
    pub digits: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepValue {
    pub label: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleOutcome {
    pub status: String,
    pub message: BiText,
    pub check: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalcResult {
    pub formula_id: String,
    pub trade: String,
    pub name: BiText,
    pub formula: BiText,          // human-readable (formula + formula_en)
    pub inputs: HashMap<String, f64>,
    pub outputs: Vec<OutputValue>,
    pub steps: Vec<StepValue>,
    pub status: String,
    pub message: BiText,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub standards: Option<Standards>,
    pub warnings: Vec<String>,
}

// ---------- Engine ----------

pub struct FormulaEngine {
    formulas: HashMap<String, Formula>,
    order: Vec<String>,
    pub trade_of: HashMap<String, String>,
}

impl FormulaEngine {
    /// Engine tupu (bila data).
    pub fn new() -> Self {
        FormulaEngine {
            formulas: HashMap::new(),
            order: Vec::new(),
            trade_of: HashMap::new(),
        }
    }

    /// Pakia zote kutoka directory (formulas/ na formulas_network/).
    pub fn load_dir(dir: &Path) -> Result<Self, String> {
        let mut engine = FormulaEngine {
            formulas: HashMap::new(),
            order: Vec::new(),
            trade_of: HashMap::new(),
        };
        if !dir.exists() {
            return Err(format!("Directory haipo: {}", dir.display()));
        }
        let mut entries: Vec<_> = std::fs::read_dir(dir)
            .map_err(|e| format!("Haiwezi kusoma {}: {}", dir.display(), e))?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().map(|x| x == "json").unwrap_or(false))
            .collect();
        entries.sort();
        for path in entries {
            engine.load_file(&path)?;
        }
        Ok(engine)
    }

    pub fn load_file(&mut self, path: &Path) -> Result<(), String> {
        let text = std::fs::read_to_string(path)
            .map_err(|e| format!("Haiwezi kusoma {}: {}", path.display(), e))?;
        let file: FormulaFile = serde_json::from_str(&text)
            .map_err(|e| format!("JSON si sahihi {}: {}", path.display(), e))?;
        for mut f in file.formulas {
            if f.trade.is_empty() { f.trade = file.trade.clone(); }
            if f.group.is_empty() { f.group = file.group.clone(); }
            let key = f.id.clone();
            if self.formulas.contains_key(&key) {
                return Err(format!("Formula '{}' inajirudia ({} ...)", key, path.display()));
            }
            self.trade_of.insert(key.clone(), f.trade.clone());
            self.order.push(key.clone());
            self.formulas.insert(key, f);
        }
        Ok(())
    }

    /// Pakia kutoka JSON string moja (mfano: q.ipya data iliyohifadhiwa).
    pub fn load_json(&mut self, text: &str) -> Result<(), String> {
        let file: FormulaFile =
            serde_json::from_str(text).map_err(|e| format!("JSON si sahihi: {}", e))?;
        for mut f in file.formulas {
            if f.trade.is_empty() { f.trade = file.trade.clone(); }
            if f.group.is_empty() { f.group = file.group.clone(); }
            let key = f.id.clone();
            self.trade_of.insert(key.clone(), f.trade.clone());
            if !self.formulas.contains_key(&key) { self.order.push(key.clone()); }
            self.formulas.insert(key, f);
        }
        Ok(())
    }

    pub fn count(&self) -> usize { self.formulas.len() }

    pub fn ids(&self) -> &[String] { &self.order }

    pub fn get(&self, id: &str) -> Option<&Formula> { self.formulas.get(id) }

    pub fn by_trade(&self, trade: &str) -> Vec<&Formula> {
        self.order
            .iter()
            .filter_map(|id| self.formulas.get(id))
            .filter(|f| f.trade == trade)
            .collect()
    }

    pub fn trades(&self) -> Vec<String> {
        let mut t: Vec<String> = self.trade_of.values().cloned().collect();
        t.sort();
        t.dedup();
        t
    }

    /// Hesabu formula kwa inputs. Hatakiwi kubadilisha JSON — code inasoma tu.
    pub fn calculate(&self, id: &str, inputs: &serde_json::Value) -> Result<CalcResult, String> {
        let f = self
            .formulas
            .get(id)
            .ok_or_else(|| format!("Formula '{}' haipo. Zilizopo: {}", id, self.summary()))?;

        // 1. todhisha inputs (min/max) + defaults
        let mut vars: Vars = HashMap::new();
        let mut warnings = Vec::new();
        let mut given: HashMap<String, f64> = HashMap::new();
        for spec in &f.inputs {
            let val = match inputs.get(&spec.name) {
                Some(v) if v.is_number() => v.as_f64().unwrap(),
                Some(v) if v.is_string() => {
                    // ruhusu "16" au kichujio kama "16 A"
                    let s = v.as_str().unwrap_or("");
                    let cleaned: String = s.chars().filter(|c| c.is_ascii_digit() || *c == '.' || *c == '-').collect();
                    cleaned.parse::<f64>().map_err(|_| {
                        format!("Input '{}': '{}' si namba", spec.name, s)
                    })?
                }
                _ => spec.default,
            };
            if let Some(min) = spec.min {
                if val < min {
                    return Err(format!(
                        "Input '{}' ({}) ni ndogo kuliko kiwango chini {} {}",
                        spec.name, val, min, spec.unit
                    ));
                }
            }
            if let Some(max) = spec.max {
                if val > max {
                    return Err(format!(
                        "Input '{}' ({}) ni kubwa kuliko kiwango cha juu {} {}",
                        spec.name, val, max, spec.unit
                    ));
                }
            }
            vars.insert(spec.name.clone(), val);
            given.insert(spec.name.clone(), val);
        }

        // 2. hesabu outputs (matokeo yote — outputs yote huwekwa kwenye vars ili
        //    rules na output zingine ziweze kuzitumia)
        let mut out_values: Vec<OutputValue> = Vec::new();
        for o in &f.outputs {
            let v = eval(&o.expr, &vars).map_err(|e| {
                format!("Formula '{}' output '{}': {}", f.id, o.name, e)
            })?;
            vars.insert(o.name.clone(), v);
            out_values.push(OutputValue {
                name: o.name.clone(),
                label: o.label.clone(),
                value: round(v, o.digits),
                unit: o.unit.clone(),
                digits: o.digits,
            });
        }

        // 3. hatua (steps)
        let mut step_values = Vec::new();
        for s in &f.steps {
            match s {
                Step::Text { label, text } => step_values.push(StepValue {
                    label: label.clone(),
                    text: text.clone(),
                }),
                Step::Expr { label, expr, template } => {
                    let v = eval(expr, &vars)
                        .map_err(|e| format!("Formula '{}' step '{}': {}", f.id, label, e))?;
                    let rendered = template.replace("{{result}}", &round(v, 2).to_string());
                    step_values.push(StepValue {
                        label: label.clone(),
                        text: rendered,
                    });
                }
            }
        }

        // 4. rules — kwanza inayolingana ndio hali (fall-through ordering)
        let mut status = "OK".to_string();
        let mut message = BiText {
            sw: "Hakuna uamuzi".into(),
            en: "No decision".into(),
        };
        let mut matched = false;
        for r in &f.rules {
            let ok = eval(&r.check, &vars).unwrap_or(0.0) != 0.0;
            if ok {
                status = r.status.clone();
                message = r.msg.clone();
                matched = true;
                break;
            }
        }
        if !matched {
            // hakuna rule iliyolingana — tumia OK na ujumbe wa kwanza wa kawaida
            if f.rules.is_empty() {
                status = "OK".into();
                message = BiText {
                    sw: "Imehesabiwa".into(),
                    en: "Computed".into(),
                };
            } else {
                status = "OK".into();
                warnings.push("Hakuna rule iliyolingana na thamani hizi".into());
            }
        }

        // 5. std::f64::NAN kuwa makosa wazi
        for o in &out_values {
            if !o.value.is_finite() {
                return Err(format!(
                    "Formula '{}' — output '{}' si halali (NaN/Infinity). Angalia inputs.",
                    f.id, o.name
                ));
            }
        }

        Ok(CalcResult {
            formula_id: f.id.clone(),
            trade: f.trade.clone(),
            name: f.name.clone(),
            formula: BiText {
                sw: f.formula.clone(),
                en: f.formula.clone(),
            },
            inputs: given,
            outputs: out_values,
            steps: step_values,
            status,
            message,
            standards: f.standards.clone(),
            warnings,
        })
    }

    /// Endesha test cases zote (CI). Kurudisha idadi ya kushindwa.
    pub fn run_tests(&self) -> (usize, Vec<String>) {
        let mut failures = Vec::new();
        let mut passed = 0usize;
        for id in &self.order {
            let f = &self.formulas[id];
            let t = match &f.test {
                Some(t) => t,
                None => {
                    failures.push(format!("{}: hakuna test case", id));
                    continue;
                }
            };
            let inputs_json = serde_json::to_value(&t.inputs).unwrap();
            match self.calculate(id, &inputs_json) {
                Err(e) => failures.push(format!("{}: {}", id, e)),
                Ok(res) => {
                    let mut ok = true;
                    for (key, expect) in &t.expect {
                        if key == "status" {
                            let want = expect.as_str().unwrap_or("");
                            if res.status != want {
                                ok = false;
                                failures.push(format!(
                                    "{}: status {} (inatarajiwa {})",
                                    id, res.status, want
                                ));
                            }
                            continue;
                        }
                        let want = expect.as_f64();
                        let got = res
                            .outputs
                            .iter()
                            .find(|o| o.name == *key)
                            .map(|o| o.value);
                        match (want, got) {
                            (Some(w), Some(g)) => {
                                let tol = if w.abs() > 1.0 { (w.abs() * 0.01).max(0.01) } else { 0.01 };
                                if (g - w).abs() > tol {
                                    ok = false;
                                    failures.push(format!(
                                        "{}.{}: {} (inatarajiwa {} ±{})",
                                        id, key, g, w, tol
                                    ));
                                }
                            }
                            _ => {
                                ok = false;
                                failures.push(format!("{}: output '{}' haipo kwenye result", id, key));
                            }
                        }
                    }
                    if ok { passed += 1; }
                }
            }
        }
        let _ = passed;
        failures.sort();
        let n = failures.len();
        (n, failures)
    }

    fn summary(&self) -> String {
        let mut ids = self.order.clone();
        ids.truncate(20);
        format!(
            "{} formulas zilizopo (mfano: {})",
            self.order.len(),
            ids.join(", ")
        )
    }
}

fn round(v: f64, digits: u32) -> f64 {
    let m = 10f64.powi(digits as i32);
    (v * m).round() / m
}

#[cfg(test)]
mod tests {
    use super::*;

    fn engine_with(json: &str) -> FormulaEngine {
        let mut e = FormulaEngine {
            formulas: HashMap::new(),
            order: Vec::new(),
            trade_of: HashMap::new(),
        };
        e.load_json(json).unwrap();
        e
    }

    const SAMPLE: &str = r#"{
      "trade": "umeme",
      "formulas": [
        {
          "id": "voltage_drop",
          "trade": "umeme",
          "name": {"sw": "Kushuka kwa Voltage", "en": "Voltage Drop"},
          "formula": "Vd = (2 x L x I x rho) / A",
          "description": {"sw": "Hesabu kushuka kwa voltage", "en": "Calculate voltage drop"},
          "inputs": [
            {"name":"L","label":{"sw":"Urefu","en":"Length"},"unit":"m","default":30,"min":0.1,"max":1000},
            {"name":"I","label":{"sw":"Current","en":"Current"},"unit":"A","default":16,"min":0.1,"max":500},
            {"name":"A","label":{"sw":"Cable","en":"Cable"},"unit":"mm2","default":2.5,"min":0.5,"max":500},
            {"name":"V","label":{"sw":"Voltage","en":"Voltage"},"unit":"V","default":230,"min":1,"max":1000},
            {"name":"rho","label":{"sw":"Resistivity","en":"Resistivity"},"unit":"ohm","default":0.0175,"min":0.001,"max":1}
          ],
          "outputs": [
            {"name":"vd","label":{"sw":"Kushuka","en":"Drop"},"expr":"(2 * L * I * rho) / A","unit":"V","digits":2},
            {"name":"vd_pct","label":{"sw":"Kushuka %","en":"Drop %"},"expr":"((2 * L * I * rho) / A / V) * 100","unit":"%","digits":2}
          ],
          "steps": [
            {"label":"Formula","text":"Vd = (2 x L x I x rho) / A"},
            {"label":"Hesabu","expr":"(2 * L * I * rho) / A","template":"Vd = {{result}} V"}
          ],
          "rules": [
            {"check":"vd_pct < 3","status":"GOOD","msg":{"sw":"Cable inatosha","en":"Cable adequate"}},
            {"check":"vd_pct < 5","status":"WARNING","msg":{"sw":"Ongeza cable","en":"Increase cable"}},
            {"check":"true","status":"FAIL","msg":{"sw":"Cable ndogo sana!","en":"Cable too small!"}}
          ],
          "standards": {"sw":["IEC 60364-5-52"],"en":["IEC 60364-5-52"]},
          "test": {"inputs":{"L":30,"I":16,"A":2.5,"V":230,"rho":0.0175},
                   "expect":{"vd":6.72,"vd_pct":2.92,"status":"GOOD"}}
        }
      ]
    }"#;

    #[test]
    fn loads_and_counts() {
        let e = engine_with(SAMPLE);
        assert_eq!(e.count(), 1);
        assert_eq!(e.trades(), vec!["umeme"]);
        assert!(e.get("voltage_drop").is_some());
    }

    #[test]
    fn test_vector_voltage_drop_good() {
        let e = engine_with(SAMPLE);
        let res = e
            .calculate(
                "voltage_drop",
                &serde_json::json!({"L":30,"I":16,"A":2.5,"V":230,"rho":0.0175}),
            )
            .unwrap();
        let vd = res.outputs.iter().find(|o| o.name == "vd").unwrap().value;
        let pct = res.outputs.iter().find(|o| o.name == "vd_pct").unwrap().value;
        assert!((vd - 6.72).abs() < 0.01, "vd={}", vd);
        assert!((pct - 2.92).abs() < 0.01, "pct={}", pct);
        assert_eq!(res.status, "GOOD");
        assert_eq!(res.steps.len(), 2);
        assert_eq!(res.message.sw, "Cable inatosha");
    }

    #[test]
    fn test_vector_voltage_drop_fail() {
        let e = engine_with(SAMPLE);
        // R-179: 100m / 4mm² / 32A -> 12.17% -> FAIL
        let res = e
            .calculate(
                "voltage_drop",
                &serde_json::json!({"L":100,"I":32,"A":4,"V":230,"rho":0.0175}),
            )
            .unwrap();
        let pct = res.outputs.iter().find(|o| o.name == "vd_pct").unwrap().value;
        assert!((pct - 12.17).abs() < 0.05, "pct={}", pct);
        assert_eq!(res.status, "FAIL");
    }

    #[test]
    fn min_max_validation() {
        let e = engine_with(SAMPLE);
        let err = e
            .calculate("voltage_drop", &serde_json::json!({"L":0,"I":16,"A":2.5,"V":230,"rho":0.0175}))
            .unwrap_err();
        assert!(err.contains("chini"), "err={}", err);
    }

    #[test]
    fn unknown_formula_is_sw_error() {
        let e = engine_with(SAMPLE);
        let err = e.calculate("haipo", &serde_json::json!({})).unwrap_err();
        assert!(err.contains("haipo"), "err={}", err);
    }

    #[test]
    fn defaults_apply_when_inputs_missing() {
        let e = engine_with(SAMPLE);
        let res = e.calculate("voltage_drop", &serde_json::json!({})).unwrap();
        assert_eq!(res.inputs.get("L").copied().unwrap(), 30.0);
        assert_eq!(res.status, "GOOD");
    }

    #[test]
    fn run_tests_pass() {
        let e = engine_with(SAMPLE);
        let (fails, list) = e.run_tests();
        assert_eq!(fails, 0, "zilizoshindwa: {:?}", list);
    }
}
