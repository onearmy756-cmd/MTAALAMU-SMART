//! mtaalamu CLI — production
//!   mtaalamu deep [--top 15]
//!   mtaalamu remediate --action clear_user_temp --approve
//!   mtaalamu agentic --msg "..." --approve

use mtaalamu_engine::{
    AgentOrchestrator, BayesianDiagnoser, DecisionTree, FormulaEngine, GeoEngine,
    HazardsEngine, I18n, KnowledgeBase, NavigationEngine, PipelineEngine, ReportEngine,
    RulesEngine, VisionEngine, deep_probe_json, log_remediation, probe_json,
    remediation_catalog, run_full, run_remediation,
};
use std::path::Path;

fn data_path(name: &str) -> String {
    let candidates = [
        format!("data/{}", name),
        format!("../data/{}", name),
        format!("../../data/{}", name),
    ];
    candidates
        .into_iter()
        .find(|p| Path::new(p).exists())
        .unwrap_or_else(|| format!("data/{}", name))
}

fn data_root() -> String {
    for c in ["data", "../data", "../../data"] {
        if Path::new(c).exists() {
            return c.to_string();
        }
    }
    "data".to_string()
}

fn load_formulas() -> Result<FormulaEngine, String> {
    let mut engine = FormulaEngine::new();
    let mut any = false;
    for d in ["data/formulas", "data/formulas_network"] {
        for candidate in [d.to_string(), format!("../{}", d), format!("../../{}", d)] {
            let p = Path::new(&candidate);
            if p.exists() {
                engine.load_dir(p)?;
                any = true;
                break;
            }
        }
    }
    if !any {
        engine.load_file(Path::new(&data_path("formulas.json")))?;
    }
    Ok(engine)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let cmd = args.get(1).map(|s| s.as_str()).unwrap_or("help");

    let out = match cmd {
        "list" => {
            let eng = load_formulas().unwrap_or_else(|e| fail(&e));
            let mut m = std::collections::BTreeMap::new();
            for t in eng.trades() {
                m.insert(t.clone(), eng.by_trade(&t).len());
            }
            serde_json::json!({"total": eng.count(), "trades": m, "ids": eng.ids()})
        }
        "calc" => {
            let id = args.get(2).unwrap_or_else(|| fail("Usage: calc <id> --inputs '{...}'"));
            let inputs = parse_flag(&args, "--inputs").unwrap_or_else(|| "{}".into());
            let inputs: serde_json::Value =
                serde_json::from_str(&inputs).unwrap_or_else(|e| fail(&format!("{}", e)));
            load_formulas()
                .unwrap_or_else(|e| fail(&e))
                .calculate(id, &inputs)
                .unwrap_or_else(|e| fail(&e))
        }
        "diagnose" => {
            let model = args
                .get(2)
                .unwrap_or_else(|| fail("Usage: diagnose <model> --symptoms a,b"));
            let syms = parse_flag(&args, "--symptoms").unwrap_or_default();
            let ids: Vec<String> = syms
                .split(',')
                .map(|s| s.trim().into())
                .filter(|s: &String| !s.is_empty())
                .collect();
            let mut d = BayesianDiagnoser::new();
            d.load_file(Path::new(&data_path("diagnosis.json")))
                .unwrap_or_else(|e| fail(&e));
            d.diagnose(model, &ids).unwrap_or_else(|e| fail(&e))
        }
        "agents" => {
            let orch = AgentOrchestrator::load(Path::new(&data_path("agents/agents_10.json")))
                .unwrap_or_else(|e| fail(&e));
            serde_json::json!({"summary": orch.summary()})
        }
        "agent" | "agentic" => {
            let msg = parse_flag(&args, "--msg")
                .unwrap_or_else(|| fail("Usage: agentic --msg \"...\" [--approve]"));
            let lang = parse_flag(&args, "--lang").unwrap_or_else(|| "sw".into());
            let approve = args.iter().any(|a| a == "--approve");
            match run_full(Path::new(&data_root()), &msg, &lang, approve) {
                Ok(r) => serde_json::json!({
                    "session": {
                        "id": r.session.id,
                        "state": r.session.state.as_str(),
                        "hitl_approved": r.session.hitl_approved,
                    },
                    "vision": r.vision,
                    "diagnosis": r.diagnosis_hits,
                    "pipeline": r.pipeline,
                    "report_md_preview": r.report_md.chars().take(600).collect::<String>(),
                }),
                Err(e) => fail(&e),
            }
        }
        "vision" => VisionEngine::load(Path::new(&data_root()))
            .unwrap_or_else(|e| fail(&e))
            .to_json(),
        "sysprobe" | "probe" => {
            let top: usize = parse_flag(&args, "--top")
                .and_then(|s| s.parse().ok())
                .unwrap_or(10);
            probe_json(top)
        }
        "deep" | "deepprobe" => {
            let top: usize = parse_flag(&args, "--top")
                .and_then(|s| s.parse().ok())
                .unwrap_or(15);
            deep_probe_json(top)
        }
        "remediate" | "fix" => {
            let action = parse_flag(&args, "--action").unwrap_or_else(|| {
                // list catalog
                return String::new();
            });
            if action.is_empty() {
                serde_json::json!({"actions": remediation_catalog()})
            } else {
                let approve = args.iter().any(|a| a == "--approve");
                let result = run_remediation(&action, approve);
                log_remediation(Path::new(&data_root()), &result);
                serde_json::to_value(&result).unwrap_or(serde_json::Value::Null)
            }
        }
        "pipeline" => {
            let eng = PipelineEngine::load(Path::new(&data_path("vision/pipeline.json")))
                .unwrap_or_else(|e| fail(&e));
            let sid = parse_flag(&args, "--session").unwrap_or_else(|| "AV".into());
            let mut rt = eng.start(&sid);
            let approve = args.iter().any(|a| a == "--approve");
            for _ in 0..eng.defs().len() {
                if eng
                    .advance(
                        &mut rt,
                        serde_json::json!({"ok": true}),
                        "ok",
                        approve,
                    )
                    .is_err()
                {
                    break;
                }
            }
            serde_json::to_value(&rt).unwrap_or(serde_json::Value::Null)
        }
        "report" => {
            let eng = ReportEngine::load(Path::new(&data_path("vision/report_template.json")))
                .unwrap_or_else(|e| fail(&e));
            let report = eng.build(
                "AV",
                &serde_json::json!({
                    "user_message": "probe",
                    "status": "completed",
                    "summary_sw": "OK",
                    "issues": []
                }),
                "sw",
            );
            serde_json::json!({"markdown": eng.to_markdown(&report).chars().take(500).collect::<String>()})
        }
        "geo" => {
            let q = parse_flag(&args, "--q").unwrap_or_default();
            let eng = GeoEngine::load(Path::new(&data_root())).unwrap_or_else(|e| fail(&e));
            serde_json::json!({"hits": eng.search(&q, 10).len(), "total": eng.count()})
        }
        "help" | "--help" | "-h" => serde_json::json!({
            "production_commands": [
                "deep [--top N]         — process tree, sockets, thermal, capabilities",
                "sysprobe [--top N]     — CPU/RAM/disk/processes",
                "vision                 — live vision snapshot from OS",
                "agentic --msg '...' [--approve]",
                "remediate              — list safe actions",
                "remediate --action ID [--approve]",
                "pipeline --approve | report | agents | calc | diagnose"
            ],
            "honesty": [
                "deep_probe = real OS metrics (sysinfo + /proc + /sys where available)",
                "NOT a camera inside the motherboard or kernel video feed",
                "remediation is software-only + HITL for destructive steps"
            ]
        }),
        other => fail(&format!("Amri '{}' haipo. Tumia help.", other)),
    };

    println!("{}", serde_json::to_string_pretty(&out).unwrap());
}

fn parse_flag(args: &[String], flag: &str) -> Option<String> {
    args.iter()
        .position(|a| a == flag)
        .and_then(|i| args.get(i + 1).cloned())
}

fn fail(msg: &str) -> ! {
    eprintln!("ERR: {}", msg);
    std::process::exit(1);
}
