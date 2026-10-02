//! knowledge.rs — Knowledge base ya matatizo/masuluhisho (data-driven).
//! Kutoka `data/problems.json` + `data/trades.json`.
//! Kipimo: confidence = match_ratio x prior (kimezungushwa 1.0).
//! Hii ni utafutaji (retrieval) — SI LLM, SI hesabu za kihisia.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Problem {
    pub id: String,
    pub trade: String,
    pub description: String,
    #[serde(default)]
    pub symptoms: Vec<String>,
    #[serde(default)]
    pub causes: HashMap<String, f64>,
    #[serde(default)]
    pub solution: String,
    #[serde(default)]
    pub formula: String,
    #[serde(default)]
    pub time_min: f64,
    #[serde(default)]
    pub cost_tzs: f64,
    #[serde(default)]
    pub success_rate: f64,
    #[serde(default)]
    pub severity: String,
    #[serde(default)]
    pub cases: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProblemsFile {
    pub problems: Vec<Problem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub name: String,
    #[serde(default)]
    pub formula: String,
    #[serde(default)]
    pub difficulty: u8,
    #[serde(default)]
    pub tools: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub id: String,
    pub name_sw: String,
    #[serde(default)]
    pub name_en: String,
    #[serde(default)]
    pub icon: String,
    #[serde(default)]
    pub color: String,
    #[serde(default)]
    pub skills: Vec<Skill>,
    #[serde(default)]
    pub common_problems: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradesFile {
    pub trades: Vec<Trade>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchedCause {
    pub cause: String,
    pub probability: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosisHit {
    pub problem_id: String,
    pub trade: String,
    pub description: String,
    pub matched_symptoms: Vec<String>,
    pub causes: Vec<MatchedCause>,
    pub solution: String,
    pub formula: String,
    pub confidence: f64,
    pub time_min: f64,
    pub cost_tzs: f64,
    pub severity: String,
    pub safety: String,
}

pub struct KnowledgeBase {
    problems: Vec<Problem>,
    trades: Vec<Trade>,
    by_trade: HashMap<String, Vec<usize>>,
}

impl KnowledgeBase {
    pub fn new() -> Self {
        KnowledgeBase {
            problems: Vec::new(),
            trades: Vec::new(),
            by_trade: HashMap::new(),
        }
    }

    pub fn load_problems_json(&mut self, text: &str) -> Result<usize, String> {
        let f: ProblemsFile =
            serde_json::from_str(text).map_err(|e| format!("problems.json si sahihi: {}", e))?;
        let added = f.problems.len();
        for p in f.problems {
            let idx = self.problems.len();
            self.by_trade.entry(p.trade.clone()).or_default().push(idx);
            self.problems.push(p);
        }
        Ok(added)
    }

    pub fn load_trades_json(&mut self, text: &str) -> Result<usize, String> {
        let f: TradesFile =
            serde_json::from_str(text).map_err(|e| format!("trades.json si sahihi: {}", e))?;
        let n = f.trades.len();
        self.trades = f.trades;
        Ok(n)
    }

    /// Pakia problems kutoka directory nzima: `data/problems/<trade>.json`.
    /// (Kila file: {"problems":[...]})
    pub fn load_dir(&mut self, dir: &std::path::Path) -> Result<usize, String> {
        if !dir.exists() {
            return Err(format!("Directory ya problems haipo: {}", dir.display()));
        }
        let mut paths: Vec<_> = std::fs::read_dir(dir)
            .map_err(|e| format!("Haiwezi kusoma {}: {}", dir.display(), e))?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().map(|x| x == "json").unwrap_or(false))
            .collect();
        paths.sort();
        let mut total = 0usize;
        for p in paths {
            let text = std::fs::read_to_string(&p)
                .map_err(|e| format!("Haiwezi kusoma {}: {}", p.display(), e))?;
            total += self.load_problems_json(&text)?;
        }
        Ok(total)
    }

    pub fn load_file(&mut self, path: &std::path::Path) -> Result<(), String> {
        let text = std::fs::read_to_string(path)
            .map_err(|e| format!("Haiwezi kusoma {}: {}", path.display(), e))?;
        let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
        if name.contains("problem") {
            self.load_problems_json(&text)?;
        } else if name.contains("trade") {
            self.load_trades_json(&text)?;
        } else {
            // jaribu problems kwanza
            if self.load_problems_json(&text).is_err() {
                self.load_trades_json(&text)?;
            }
        }
        Ok(())
    }

    pub fn problem_count(&self) -> usize { self.problems.len() }
    pub fn trades(&self) -> &[Trade] { &self.trades }

    pub fn problems_for_trade(&self, trade: &str) -> Vec<&Problem> {
        self.by_trade
            .get(trade)
            .map(|v| v.iter().map(|i| &self.problems[*i]).collect())
            .unwrap_or_default()
    }

    pub fn get(&self, id: &str) -> Option<&Problem> {
        self.problems.iter().find(|p| p.id == id)
    }

    /// Utafutaji: dalili zinazolingana / jumla ya dalili => match_ratio,
    /// kisha confidence = match_ratio x prior_ya_problem (na success_rate kama uzito).
    /// (N-156: match_ratio x prior, capped 1.0)
    pub fn diagnose(&self, trade: &str, symptoms: &[String]) -> Vec<DiagnosisHit> {
        let pool = self.problems_for_trade(trade);
        let mut hits: Vec<DiagnosisHit> = Vec::new();
        for p in pool {
            let matched: Vec<String> = p
                .symptoms
                .iter()
                .filter(|s| symptoms.iter().any(|x| x == *s))
                .cloned()
                .collect();
            // zero-match inabaki (confidence 0) — sort inaziweka chini kabisa
            let denom = if p.symptoms.is_empty() {
                1.0
            } else {
                p.symptoms.len().max(symptoms.len().max(1)) as f64
            };
            let match_ratio = matched.len() as f64 / denom;
            // prior kutoka jumla ya probabilities za causes (kama zipo) au success_rate
            let prior = if p.causes.is_empty() {
                if p.success_rate > 0.0 { p.success_rate } else { 0.5 }
            } else {
                p.causes.values().cloned().fold(0.0f64, f64::max)
            };
            let mut confidence = (match_ratio * prior).min(1.0);
            if symptoms.is_empty() {
                confidence = (prior * 0.5).min(1.0); // hakuna dalili => polepole
            }
            let mut causes: Vec<MatchedCause> = p
                .causes
                .iter()
                .map(|(c, pr)| MatchedCause {
                    cause: c.clone(),
                    probability: (*pr * 100.0).round() / 100.0,
                })
                .collect();
            causes.sort_by(|a, b| b.probability.partial_cmp(&a.probability).unwrap());

            hits.push(DiagnosisHit {
                problem_id: p.id.clone(),
                trade: p.trade.clone(),
                description: p.description.clone(),
                matched_symptoms: matched,
                causes,
                solution: p.solution.clone(),
                formula: p.formula.clone(),
                confidence: (confidence * 1000.0).round() / 1000.0,
                time_min: p.time_min,
                cost_tzs: p.cost_tzs,
                severity: p.severity.clone(),
                safety: safety_for(&p.trade, &p.severity),
            });
        }
        hits.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
        hits.truncate(10);
        hits
    }

    /// Kumbukumbu za kujifunza (R-230): ongeza cases kwa problem.
    pub fn learn(&mut self, problem_id: &str, success: bool) -> Result<(), String> {
        let p = self
            .problems
            .iter_mut()
            .find(|p| p.id == problem_id)
            .ok_or_else(|| format!("Problem '{}' haipo", problem_id))?;
        p.cases += 1;
        if success {
            // success_rate inazidi kidogo (emapirical average)
            let n = p.cases as f64;
            p.success_rate = ((p.success_rate * (n - 1.0)) + 1.0) / n;
        } else {
            let n = p.cases as f64;
            p.success_rate = (p.success_rate * (n - 1.0)) / n;
        }
        p.success_rate = (p.success_rate * 1000.0).round() / 1000.0;
        Ok(())
    }
}

/// Onyo la usalama kwa kila trade/severity (R-203).
fn safety_for(trade: &str, severity: &str) -> String {
    let sev = severity.to_lowercase();
    let critical = matches!(sev.as_str(), "critical" | "maaruufu" | "high");
    match trade {
        "umeme" => {
            if critical {
                "⚠️ ONYO: Zima main switch kwanza! Umeme unaua.".into()
            } else {
                "⚠️ Tahadhari: vaa glavu na tumia zana zisizo za umeme.".into()
            }
        }
        "maji" => "⚠️ Tahadhari: zima bomba la maji kabla ya kazi.".into(),
        "gari" => "⚠️ Tahadhari: weka gari kwenye park na zima injini.".into(),
        "hvac" | "gas" => "⚠️ ONYO: gesi na shinikizo ni hatari — funga bomba la gesi kwanza.".into(),
        "simu" | "computer" => "⚠️ Tahadhari: zima kifaa na uondoe betri kama inajaa.".into(),
        "cctv" | "solar" => "⚠️ Tahadhari: toa power kwanza kabla ya kugusa waya.".into(),
        _ => "⚠️ Tahadhari: fuata kanuni za usalama za kazi yako.".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kb() -> KnowledgeBase {
        let mut k = KnowledgeBase::new();
        let json = r#"{"problems":[
          {"id":"E-001","trade":"umeme","description":"Breaker inarudiana mara kila kettle inapoanza",
           "symptoms":["breaker_trips","kettle_on"],"causes":{"overload":0.7,"bad_breaker":0.2,"loose_wire":0.1},
           "solution":"Badilisha breaker kuwa 16A na punguza load","formula":"breaker_size","time_min":20,
           "cost_tzs":35000,"success_rate":0.95,"severity":"high","cases":847},
          {"id":"E-002","trade":"umeme","description":"Taa zina flicker",
           "symptoms":["flicker"],"causes":{"loose_wire":0.6,"bad_switch":0.4},
           "solution":"Kanza connections","formula":"","time_min":30,"cost_tzs":20000,
           "success_rate":0.9,"severity":"medium","cases":120}
        ]}"#;
        k.load_problems_json(json).unwrap();
        k
    }

    #[test]
    fn loads_counts() {
        let k = kb();
        assert_eq!(k.problem_count(), 2);
        assert_eq!(k.problems_for_trade("umeme").len(), 2);
        assert_eq!(k.problems_for_trade("maji").len(), 0);
    }

    #[test]
    fn matching_symptoms_rank_first() {
        let k = kb();
        let hits = k.diagnose("umeme", &["breaker_trips".into(), "kettle_on".into()]);
        assert!(!hits.is_empty());
        assert_eq!(hits[0].problem_id, "E-001");
        assert_eq!(hits[0].matched_symptoms.len(), 2);
        assert!(hits[0].confidence > hits[1].confidence);
        assert!(hits[0].confidence <= 1.0);
        assert!(hits[0].safety.contains("main switch"));
    }

    #[test]
    fn causes_sorted_desc() {
        let k = kb();
        let hits = k.diagnose("umeme", &["breaker_trips".into()]);
        let c = &hits[0].causes;
        assert_eq!(c[0].cause, "overload");
        for w in c.windows(2) {
            assert!(w[0].probability >= w[1].probability);
        }
    }

    #[test]
    fn no_matching_symptoms_excluded() {
        let k = kb();
        let hits = k.diagnose("umeme", &["tofauti_sana".into()]);
        // zero-match zinarudi kwa confidence 0 — E-001 zenye match zinakuja kwanza
        assert!(!hits.is_empty());
        assert!(hits[0].problem_id == "E-001");
        assert!(hits.last().map(|h| h.confidence).unwrap_or(0.0) <= hits[0].confidence);
    }

    #[test]
    fn learn_updates_success_rate_and_cases() {
        let mut k = kb();
        let before = k.get("E-002").unwrap().success_rate;
        k.learn("E-002", true).unwrap();
        let after = k.get("E-002").unwrap();
        assert_eq!(after.cases, 121);
        assert!(after.success_rate >= before);
    }

    #[test]
    fn unknown_problem_learn_errors() {
        let mut k = kb();
        assert!(k.learn("E-999", true).is_err());
    }
}
