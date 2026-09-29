//! bayes.rs — Bayesian inference (data-driven).
//! P(Cause|Symptoms) = P(Symptoms|Cause) x P(Cause) / P(Symptoms)
//!
//! Data kutoka `data/diagnosis.json` (priors + likelihood matrix).
//! Kanuni: AI inapendekeza, binadamu anathibitisha (HITL). Hesabu ni deterministic.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fix {
    pub sw: String,
    pub en: String,
    #[serde(default)]
    pub cost_tzs: f64,
    #[serde(default)]
    pub time_min: f64,
    #[serde(default)]
    pub tools: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model {
    pub priors: HashMap<String, f64>,
    pub symptoms: Vec<String>,
    pub causes: Vec<String>,
    /// likelihood[symptom][cause] — thamani 0..1: P(symptom | cause)
    pub likelihood: HashMap<String, Vec<f64>>,
    #[serde(default)]
    pub fixes: HashMap<String, Fix>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosisFile {
    pub models: HashMap<String, Model>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Posterior {
    pub cause: String,
    pub probability: f64, // 0..1 (rounded 2 dp)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fix: Option<Fix>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnoseResult {
    pub model: String,
    pub matched_symptoms: Vec<String>,
    pub posteriors: Vec<Posterior>,
    pub top_cause: String,
    pub top_probability: f64,
    /// > 0.8 => "Fanya kazi", vinginevyo "Chunguza zaidi" (R-202)
    pub confidence_gate: String,
}

pub struct BayesianDiagnoser {
    models: HashMap<String, Model>,
    /// kumbukumbu za kujifunza: cause -> (confirmed_count, total)
    learned: HashMap<String, (u32, u32)>,
}

impl BayesianDiagnoser {
    pub fn new() -> Self {
        BayesianDiagnoser {
            models: HashMap::new(),
            learned: HashMap::new(),
        }
    }

    pub fn load_json(&mut self, text: &str) -> Result<(), String> {
        let file: DiagnosisFile =
            serde_json::from_str(text).map_err(|e| format!("diagnosis.json si sahihi: {}", e))?;
        for (k, v) in file.models {
            self.models.insert(k, v);
        }
        Ok(())
    }

    pub fn load_file(&mut self, path: &std::path::Path) -> Result<(), String> {
        let text =
            std::fs::read_to_string(path).map_err(|e| format!("Haiwezi kusoma {}: {}", path.display(), e))?;
        self.load_json(&text)
    }

    pub fn model_names(&self) -> Vec<String> {
        let mut n: Vec<String> = self.models.keys().cloned().collect();
        n.sort();
        n
    }

    pub fn has(&self, name: &str) -> bool { self.models.contains_key(name) }

    /// Posterior kwa symptoms zilizotolewa.
    /// Kila symptom: multiply likelihood; isipokuwa: multiply (1 - likelihood);
    /// zisha na prior; normalize; panga descending; round 2 dp. (R-210)
    pub fn diagnose(&self, model: &str, symptoms: &[String]) -> Result<DiagnoseResult, String> {
        let m = self
            .models
            .get(model)
            .ok_or_else(|| format!("Model '{}' haipo. Zilizopo: {:?}", model, self.model_names()))?;

        // thibitisha symptoms
        for s in symptoms {
            if !m.symptoms.contains(s) {
                return Err(format!(
                    "Symptom '{}' haijulikani kwenye model '{}'. Zilizopo: {:?}",
                    s, model, m.symptoms
                ));
            }
        }

        let mut probs: Vec<(String, f64)> = Vec::new();
        let mut sum = 0.0f64;
        for cause in &m.causes {
            let prior = m.priors.get(cause).copied().unwrap_or(0.0);
            let mut p = prior;
            for sym in &m.symptoms {
                let idx = m
                    .symptoms
                    .iter()
                    .position(|x| x == sym)
                    .expect("symptom index");
                let row = m
                    .likelihood
                    .get(sym)
                    .ok_or_else(|| format!("Likelihood ya symptom '{}' haipo", sym))?;
                let l = row
                    .get(m.causes.iter().position(|c| c == cause).expect("cause index"))
                    .copied()
                    .unwrap_or(0.0);
                let l = l.clamp(0.0, 1.0);
                let factor = if symptoms.contains(sym) { l } else { 1.0 - l };
                p *= factor;
            }
            // learning (R-211): ongeza uzito kwa mafanikio yaliyothibitishwa
            if let Some((conf, total)) = self.learned.get(cause) {
                if *total > 0 {
                    let rate = (*conf as f64) / (*total as f64);
                    p *= 0.95 + 0.10 * rate; // 0.95 .. 1.05
                }
            }
            probs.push((cause.clone(), p));
            sum += p;
        }

        if sum <= 0.0 {
            return Err(
                "Hakuna sababu inayowezekana — thamani zote ni 0. Angalia inputs".into(),
            );
        }

        let mut out: Vec<Posterior> = probs
            .into_iter()
            .map(|(cause, p)| {
                let fix = m.fixes.get(&cause).cloned();
                Posterior {
                    cause,
                    probability: (p / sum * 100.0).round() / 100.0,
                    fix,
                }
            })
            .collect();
        out.sort_by(|a, b| b.probability.partial_cmp(&a.probability).unwrap());

        let top = out.first().cloned().unwrap_or(Posterior {
            cause: "hakuna".into(),
            probability: 0.0,
            fix: None,
        });

        Ok(DiagnoseResult {
            model: model.to_string(),
            matched_symptoms: symptoms.to_vec(),
            posteriors: out,
            top_probability: top.probability,
            top_cause: top.cause,
            confidence_gate: if top.probability > 0.8 {
                "Fanya kazi".into()
            } else {
                "Chunguza zaidi".into()
            },
        })
    }

    /// Kujifunza kutoka kazi halisi (R-211): confirmed => prior x 1.05 (max 0.95),
    /// confirmed => prior x 0.95 (min 0.01), kisha renormalize.
    pub fn learn(&mut self, model: &str, cause: &str, confirmed: bool) -> Result<(), String> {
        let m = self
            .models
            .get_mut(model)
            .ok_or_else(|| format!("Model '{}' haipo", model))?;
        let prior = m
            .priors
            .get(cause)
            .copied()
            .ok_or_else(|| format!("Cause '{}' haipo", cause))?;
        let new_p = if confirmed {
            (prior * 1.05).min(0.95)
        } else {
            (prior * 0.95).max(0.01)
        };
        m.priors.insert(cause.to_string(), new_p);
        // renormalize jumla = 1
        let total: f64 = m.priors.values().sum();
        if total > 0.0 {
            for v in m.priors.values_mut() {
                *v = (*v / total * 100.0).round() / 100.0;
            }
            // renormalize mara ya pili kwa rounding
            let total2: f64 = m.priors.values().sum();
            if (total2 - 1.0).abs() > 1e-9 && total2 > 0.0 {
                if let Some(first) = m.priors.keys().next().cloned() {
                    let others: f64 = m
                        .priors
                        .iter()
                        .filter(|(k, _)| **k != first)
                        .map(|(_, v)| *v)
                        .sum();
                    let adj = 1.0 - others;
                    if adj > 0.0 {
                        m.priors.insert(first, (adj * 100.0).round() / 100.0);
                    }
                }
            }
        }
        let e = self.learned.entry(cause.to_string()).or_insert((0, 0));
        e.1 += 1;
        if confirmed {
            e.0 += 1;
        }
        Ok(())
    }

    pub fn priors(&self, model: &str) -> Option<&HashMap<String, f64>> {
        self.models.get(model).map(|m| &m.priors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Thamani halisi kutoka SMART SYSTEM 1 (R-208…R-210)
    fn electrical_json() -> String {
        r#"{
          "models": {
            "electrical": {
              "priors": {"overload":0.30,"short_circuit":0.20,"earth_leak":0.15,"loose_wire":0.15,"bad_breaker":0.10,"bad_switch":0.10},
              "symptoms": ["breaker_trips","sparks","shock","no_power","flicker","sound"],
              "causes":   ["overload","short_circuit","earth_leak","loose_wire","bad_breaker","bad_switch"],
              "likelihood": {
                "breaker_trips": [0.90,0.85,0.20,0.30,0.95,0.10],
                "sparks":        [0.10,0.90,0.30,0.60,0.20,0.70],
                "shock":         [0.00,0.10,0.95,0.60,0.00,0.20],
                "no_power":      [0.40,0.80,0.30,0.70,0.90,0.60],
                "flicker":       [0.30,0.20,0.10,0.80,0.40,0.85],
                "sound":         [0.20,0.30,0.10,0.40,0.70,0.50]
              },
              "fixes": {
                "overload": {"sw":"Punguza load","en":"Reduce load","cost_tzs":35000,"time_min":20,"tools":["screwdriver","multimeter"]},
                "short_circuit": {"sw":"Tatua mzunguko mfupi","en":"Fix short circuit","cost_tzs":50000,"time_min":40,"tools":["multimeter","insulation tape"]}
              }
            }
          }
        }"#
        .to_string()
    }

    #[test]
    fn breaker_trips_and_sparks() {
        // R-212: Short circuit 62.3%, Overload 21.5%
        let mut d = BayesianDiagnoser::new();
        d.load_json(&electrical_json()).unwrap();
        let r = d
            .diagnose("electrical", &["breaker_trips".to_string(), "sparks".to_string()])
            .unwrap();
        let get = |cause: &str| {
            r.posteriors
                .iter()
                .find(|p| p.cause == cause)
                .map(|p| p.probability)
                .unwrap()
        };
        assert!((get("short_circuit") - 0.623).abs() < 0.01, "{}", get("short_circuit"));
        assert!((get("overload") - 0.215).abs() < 0.01, "{}", get("overload"));
        assert_eq!(r.top_cause, "short_circuit");
        assert_eq!(r.confidence_gate, "Fanya kazi");
    }

    #[test]
    fn sum_is_one() {
        let mut d = BayesianDiagnoser::new();
        d.load_json(&electrical_json()).unwrap();
        let r = d
            .diagnose("electrical", &["no_power".to_string()])
            .unwrap();
        let total: f64 = r.posteriors.iter().map(|p| p.probability).sum();
        assert!((total - 1.0).abs() < 0.02, "jumla={}", total);
        // sorted descending
        for w in r.posteriors.windows(2) {
            assert!(w[0].probability >= w[1].probability);
        }
    }

    #[test]
    fn absent_symptoms_use_complement() {
        let mut d = BayesianDiagnoser::new();
        d.load_json(&electrical_json()).unwrap();
        let r = d.diagnose("electrical", &[]).unwrap();
        // hakuna symptoms => posterior = prior (normalized)
        let overload = r
            .posteriors
            .iter()
            .find(|p| p.cause == "overload")
            .unwrap()
            .probability;
        assert!((overload - 0.30).abs() < 0.02, "{}", overload);
    }

    #[test]
    fn learning_adjusts_and_renormalizes() {
        let mut d = BayesianDiagnoser::new();
        d.load_json(&electrical_json()).unwrap();
        for _ in 0..5 {
            d.learn("electrical", "overload", true).unwrap();
        }
        let priors = d.priors("electrical").unwrap();
        let total: f64 = priors.values().sum();
        assert!((total - 1.0).abs() < 0.03, "jumla={}", total);
        assert!(*priors.get("overload").unwrap() > 0.30);
        assert!(*priors.get("overload").unwrap() <= 0.95);
    }

    #[test]
    fn unknown_symptom_errors_in_swahili() {
        let mut d = BayesianDiagnoser::new();
        d.load_json(&electrical_json()).unwrap();
        let err = d
            .diagnose("electrical", &["tofauti".to_string()])
            .unwrap_err();
        assert!(err.contains("haijulikani"), "{}", err);
    }

    #[test]
    fn unknown_model_errors() {
        let d = BayesianDiagnoser::new();
        let err = d.diagnose("hakuna", &[]).unwrap_err();
        assert!(err.contains("haipo"), "{}", err);
    }
}
