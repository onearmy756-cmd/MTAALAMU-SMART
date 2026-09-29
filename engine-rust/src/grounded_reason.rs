//! Grounded Reasoner — majibu marefu, calculus/numeric, data halisi (no hallucination)

use crate::electronic_solver;
use crate::expr::{eval_with, Vars};
use crate::knowledge_hub;
use crate::sysprobe;
use serde::Serialize;
use serde_json::{json, Value};
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
pub struct Evidence {
    pub source: String,
    pub claim_sw: String,
    pub data: Value,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct CalculusNote {
    pub name: String,
    pub explanation_sw: String,
    pub inputs: Value,
    pub result: Value,
    pub formula: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct GroundedAnswer {
    pub query: String,
    pub thinking_sw: String,
    pub evidence: Vec<Evidence>,
    pub calculus: Vec<CalculusNote>,
    pub answer_sw: String,
    pub answer_long_sw: String,
    pub limitations_sw: Vec<String>,
    pub next_steps_sw: Vec<String>,
    pub provenance: Vec<String>,
}

fn clamp01(x: f64) -> f64 {
    x.max(0.0).min(1.0)
}

fn numeric_derivative<F: Fn(f64) -> f64>(f: F, x: f64, h: f64) -> f64 {
    (f(x + h) - f(x - h)) / (2.0 * h)
}

fn rate(a: f64, b: f64) -> Option<f64> {
    if b.abs() < 1e-12 {
        None
    } else {
        Some(a / b)
    }
}

fn evidence_from_knowledge(data_root: &Path, msg: &str) -> Vec<Evidence> {
    let mut out = Vec::new();
    let kr = knowledge_hub::search_all(data_root, msg, 6);
    for h in kr.hits.iter().take(8) {
        out.push(Evidence {
            source: format!("knowledge:{}", h.module),
            claim_sw: format!("{} — {}", h.title_sw, h.detail_sw),
            data: json!({"id": h.id, "score": h.score, "module": h.module, "meta": h.meta}),
            confidence: clamp01(0.45 + (h.score as f64) * 0.03),
        });
    }
    let dev = electronic_solver::search_devices(data_root, msg, 5);
    for h in dev.hits.iter().take(5) {
        out.push(Evidence {
            source: "devices".into(),
            claim_sw: format!(
                "Kifaa {} ({}): {} → {}",
                h.kifaa,
                h.kundi,
                h.tatizo,
                h.suluhisho.iter().take(4).cloned().collect::<Vec<_>>().join(" → ")
            ),
            data: json!({"kifaa": h.kifaa, "domain": h.domain, "score": h.score, "suluhisho": h.suluhisho}),
            confidence: clamp01(0.5 + (h.score as f64) * 0.02),
        });
    }
    out
}

fn evidence_from_probe() -> (Vec<Evidence>, Vec<CalculusNote>) {
    let mut ev = Vec::new();
    let mut calc = Vec::new();
    let probe = sysprobe::probe(8, 200);
    let cpu = probe.cpu_usage_pct;
    let mem = probe.ram_usage_pct;
    let disk = if probe.disks.is_empty() {
        0.0
    } else {
        probe.disks.iter().map(|d| d.used_pct).fold(0.0_f64, f64::max)
    };

    ev.push(Evidence {
        source: "sysprobe".into(),
        claim_sw: format!(
            "Kipimo cha sasa: CPU {:.1}%, RAM {:.1}%, Disk {:.1}% (sysinfo, si makadirio)",
            cpu, mem, disk
        ),
        data: json!({
            "cpu_usage_pct": cpu,
            "ram_usage_pct": mem,
            "disk_used_pct": disk,
            "hostname": probe.hostname,
            "os": probe.os_name,
        }),
        confidence: 0.95,
    });

    let pressure = 0.4 * cpu + 0.35 * mem + 0.25 * disk;
    ev.push(Evidence {
        source: "calculus:load_index".into(),
        claim_sw: format!(
            "Mzigo P = 0.4·CPU + 0.35·RAM + 0.25·Disk = {:.2} (linear, si AGI)",
            pressure
        ),
        data: json!({"P": pressure, "weights": {"cpu": 0.4, "mem": 0.35, "disk": 0.25}}),
        confidence: 0.9,
    });

    calc.push(CalculusNote {
        name: "partial_dP".into(),
        explanation_sw: "Kwa P(c,m,d)=0.4c+0.35m+0.25d: ∂P/∂c=0.4, ∂P/∂m=0.35, ∂P/∂d=0.25.".into(),
        inputs: json!({"form": "P=0.4c+0.35m+0.25d"}),
        result: json!({"dP_dCPU": 0.4, "dP_dMEM": 0.35, "dP_dDISK": 0.25}),
        formula: "∂P/∂c=0.4, ∂P/∂m=0.35, ∂P/∂d=0.25".into(),
    });

    let scores = [cpu, mem, disk];
    let maxv = scores.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let exps: Vec<f64> = scores.iter().map(|s| (s - maxv).exp()).collect();
    let z: f64 = exps.iter().sum();
    let probs: Vec<f64> = exps.iter().map(|e| e / z).collect();
    calc.push(CalculusNote {
        name: "softmax_priority".into(),
        explanation_sw: "Softmax(CPU,RAM,Disk) = uzito wa kipaumbele — hesabu tu.".into(),
        inputs: json!({"cpu": cpu, "mem": mem, "disk": disk}),
        result: json!({"p_cpu": probs[0], "p_mem": probs[1], "p_disk": probs[2]}),
        formula: "softmax(x_i)=exp(x_i-max)/Σexp".into(),
    });

    let r = |p: f64| 1.0 / (1.0 + (-(p - 50.0) / 10.0).exp());
    let r0 = r(pressure);
    let dr = numeric_derivative(r, pressure, 0.5);
    calc.push(CalculusNote {
        name: "logistic_risk".into(),
        explanation_sw: format!(
            "r(P)=1/(1+e^(-(P-50)/10)); r({:.1})={:.3}; r'≈{:.4} (central difference).",
            pressure, r0, dr
        ),
        inputs: json!({"P": pressure}),
        result: json!({"r": r0, "dr_dP": dr}),
        formula: "r=σ((P-50)/10); r'≈(r(P+h)-r(P-h))/(2h)".into(),
    });

    if pressure > 70.0 {
        ev.push(Evidence {
            source: "calculus:risk".into(),
            claim_sw: format!("Mzigo P={:.1} juu; r≈{:.2} — thibitisha na sysprobe.", pressure, r0),
            data: json!({"P": pressure, "r": r0}),
            confidence: 0.75,
        });
    }

    (ev, calc)
}

fn try_formula_eval(msg: &str) -> Option<CalculusNote> {
    let lower = msg.to_lowercase();
    if !(lower.contains('+') || lower.contains('*') || lower.contains('/') || lower.contains("hesabu")) {
        return None;
    }
    let expr: String = msg
        .chars()
        .filter(|c| c.is_ascii_digit() || "+-*/(). ".contains(*c) || c.is_ascii_alphabetic())
        .collect();
    let expr = expr.trim();
    if expr.len() < 3 {
        return None;
    }
    let mut vars = Vars::new();
    vars.insert("x".to_string(), 1.0);
    vars.insert("t".to_string(), 1.0);
    match eval_with(expr, &vars) {
        Ok(v) => Some(CalculusNote {
            name: "expr_eval".into(),
            explanation_sw: format!("Tathmini ya usemi: `{expr}`"),
            inputs: json!({"expr": expr}),
            result: json!({"value": v}),
            formula: expr.to_string(),
        }),
        Err(_) => None,
    }
}

fn build_thinking(msg: &str, n_ev: usize, n_calc: usize) -> String {
    format!(
        "Hatua (grounded):\n1) Soma ujumbe bila kuongeza ukweli.\n2) Knowledge hits≈{}.\n3) Sysprobe CPU/RAM/Disk.\n4) Calculus notes={}.\n5) Jibu refu + chanzo kila dai.\n6) Mipaka: si AGI; hardware=binadamu.\nUjumbe: «{}»",
        n_ev, n_calc, msg
    )
}

fn build_long_answer(msg: &str, evidence: &[Evidence], calc: &[CalculusNote]) -> String {
    let mut s = String::new();
    s.push_str("# Jibu la Mtaalamu (Grounded)\n\n## 1. Uelewa\n");
    s.push_str(&format!("Umeeleza: «{}». Sijabuni dalili zisizokuwepo.\n\n## 2. Ushahidi\n", msg));
    if evidence.is_empty() {
        s.push_str("Hakuna hit imara — si invent; ni matokeo ya search.\n");
    } else {
        for (i, e) in evidence.iter().enumerate() {
            s.push_str(&format!(
                "{}. [{}] (~{:.0}%) {}\n",
                i + 1,
                e.source,
                e.confidence * 100.0,
                e.claim_sw
            ));
        }
    }
    s.push_str("\n## 3. Calculus (Rust)\n");
    if calc.is_empty() {
        s.push_str("Hakuna hesabu ya ziada.\n");
    } else {
        for c in calc {
            s.push_str(&format!(
                "- **{}**: {}\n  - `{}` → {}\n",
                c.name, c.explanation_sw, c.formula, c.result
            ));
        }
    }
    s.push_str("\n## 4. Mapendekezo\n");
    let soft = evidence.iter().any(|e| {
        e.source.contains("problems")
            || e.claim_sw.to_lowercase().contains("virus")
            || e.claim_sw.to_lowercase().contains("update")
    });
    let hard = evidence.iter().any(|e| e.source == "devices");
    if soft {
        s.push_str("Software: baada ya HITL → `mtaalamu solve --approve`.\n");
    }
    if hard {
        s.push_str("Hardware/vifaa: orodha ya ukaguzi; fundi anatekeleza.\n");
    }
    if !soft && !hard {
        s.push_str("Eleza dalili wazi (mf. kompyuta polepole, TV haiwaki).\n");
    }
    s.push_str("\n## 5. Mipaka\n- Si kernel video / motherboard camera.\n- Hesabu ni deterministic.\n- High-voltage / medical: fundi aliyeidhinishwa.\n");
    s
}

pub fn reason(data_root: &Path, msg: &str) -> GroundedAnswer {
    let mut evidence = evidence_from_knowledge(data_root, msg);
    let (mut probe_ev, mut calc) = evidence_from_probe();
    evidence.append(&mut probe_ev);
    if let Some(c) = try_formula_eval(msg) {
        calc.push(c);
    }
    for e in evidence.iter().filter(|e| e.source.starts_with("knowledge:problems")) {
        let cost = e
            .data
            .get("meta")
            .and_then(|m| m.get("cost_tzs"))
            .and_then(|x| x.as_f64().or_else(|| x.as_i64().map(|i| i as f64)));
        let tmin = e
            .data
            .get("meta")
            .and_then(|m| m.get("time_min"))
            .and_then(|x| x.as_f64().or_else(|| x.as_i64().map(|i| i as f64)));
        if let (Some(cost), Some(tmin)) = (cost, tmin) {
            if let Some(r) = rate(cost, tmin) {
                calc.push(CalculusNote {
                    name: "cost_per_minute".into(),
                    explanation_sw: format!("Gharama/muda kutoka knowledge: {:.2} TZS/min.", r),
                    inputs: json!({"cost_tzs": cost, "time_min": tmin}),
                    result: json!({"tzs_per_min": r}),
                    formula: "cost_tzs / time_min".into(),
                });
                break;
            }
        }
    }
    evidence.sort_by(|a, b| {
        b.confidence
            .partial_cmp(&a.confidence)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let thinking_sw = build_thinking(msg, evidence.len(), calc.len());
    let answer_long_sw = build_long_answer(msg, &evidence, &calc);
    let answer_sw = if let Some(top) = evidence.first() {
        format!("{} [chanzo: {} · {:.0}%]", top.claim_sw, top.source, top.confidence * 100.0)
    } else {
        "Hakuna ushahidi wa kutosha — eleza dalili zaidi.".into()
    };
    let provenance: Vec<String> = evidence
        .iter()
        .map(|e| e.source.clone())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();
    GroundedAnswer {
        query: msg.into(),
        thinking_sw,
        evidence,
        calculus: calc,
        answer_sw,
        answer_long_sw,
        limitations_sw: vec![
            "Si AGI; orchestrator + knowledge + probe + hesabu.".into(),
            "Hardware haitengenezwi automatiki.".into(),
            "Kila dai lina source.".into(),
        ],
        next_steps_sw: vec![
            "mtaalamu knowledge --msg \"...\"".into(),
            "mtaalamu solve --msg \"...\" --approve".into(),
            "R: grounded_summary(msg)".into(),
        ],
        provenance,
    }
}

pub fn reason_json(data_root: &Path, msg: &str) -> Value {
    serde_json::to_value(reason(data_root, msg)).unwrap_or(Value::Null)
}
