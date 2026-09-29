//! mtaalamu CLI
//!   mtaalamu solve --msg "..." --approve

use mtaalamu_engine::{
    AgentOrchestrator, BayesianDiagnoser, FormulaEngine, GeoEngine, PipelineEngine,
    ReportEngine, VisionEngine, auto_diagnose, deep_probe_json, discover_tools,
    log_remediation, probe_json, remediation_catalog, run_diagnostic, run_full,
    run_remediation, solve_message,
};
use std::path::Path;

fn data_path(name: &str) -> String {
    for c in [
        format!("data/{}", name),
        format!("../data/{}", name),
        format!("../../data/{}", name),
    ] {
        if Path::new(&c).exists() {
            return c;
        }
    }
    format!("data/{}", name)
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
        for candidate in [d.to_string(), format!("../{}", d)] {
            if Path::new(&candidate).exists() {
                engine.load_dir(Path::new(&candidate))?;
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
            serde_json::json!({"total": eng.count()})
        }
        "calc" => {
            let id = args.get(2).unwrap_or_else(|| fail("calc <id> --inputs"));
            let inputs = parse_flag(&args, "--inputs").unwrap_or_else(|| "{}".into());
            let inputs: serde_json::Value =
                serde_json::from_str(&inputs).unwrap_or_else(|e| fail(&format!("{}", e)));
            load_formulas()
                .unwrap_or_else(|e| fail(&e))
                .calculate(id, &inputs)
                .unwrap_or_else(|e| fail(&e))
        }
        "diagnose" => {
            let model = args.get(2).unwrap_or_else(|| fail("diagnose <model> --symptoms"));
            let syms = parse_flag(&args, "--symptoms").unwrap_or_default();
            let ids: Vec<String> = syms.split(',').map(|s| s.trim().into()).filter(|s: &String| !s.is_empty()).collect();
            let mut d = BayesianDiagnoser::new();
            d.load_file(Path::new(&data_path("diagnosis.json"))).unwrap_or_else(|e| fail(&e));
            d.diagnose(model, &ids).unwrap_or_else(|e| fail(&e))
        }
        "agents" => {
            let orch = AgentOrchestrator::load(Path::new(&data_path("agents/agents_10.json")))
                .unwrap_or_else(|e| fail(&e));
            serde_json::json!({"summary": orch.summary()})
        }
        "agent" | "agentic" => {
            let msg = parse_flag(&args, "--msg").unwrap_or_else(|| fail("agentic --msg"));
            let lang = parse_flag(&args, "--lang").unwrap_or_else(|| "sw".into());
            let approve = args.iter().any(|a| a == "--approve");
            match run_full(Path::new(&data_root()), &msg, &lang, approve) {
                Ok(r) => serde_json::json!({
                    "session": { "id": r.session.id, "state": r.session.state.as_str(),
                                  "hitl_approved": r.session.hitl_approved },
                    "vision": r.vision,
                    "report_md_preview": r.report_md.chars().take(400).collect::<String>(),
                }),
                Err(e) => fail(&e),
            }
        }
        "solve" => {
            let msg = parse_flag(&args, "--msg")
                .unwrap_or_else(|| fail("solve --msg \"...\" [--approve]"));
            let approve = args.iter().any(|a| a == "--approve");
            let result = solve_message(Path::new(&data_root()), &msg, approve);
            serde_json::to_value(&result).unwrap_or(serde_json::Value::Null)
        }
        "vision" => VisionEngine::load(Path::new(&data_root()))
            .unwrap_or_else(|e| fail(&e))
            .to_json(),
        "sysprobe" | "probe" => {
            let top: usize = parse_flag(&args, "--top").and_then(|s| s.parse().ok()).unwrap_or(10);
            probe_json(top)
        }
        "deep" => {
            let top: usize = parse_flag(&args, "--top").and_then(|s| s.parse().ok()).unwrap_or(15);
            deep_probe_json(top)
        }
        "tools" => serde_json::to_value(discover_tools()).unwrap_or(serde_json::Value::Null),
        "diagnose-os" => auto_diagnose(),
        "tool-run" => {
            let id = parse_flag(&args, "--id").unwrap_or_else(|| fail("tool-run --id df"));
            serde_json::to_value(run_diagnostic(&id)).unwrap_or(serde_json::Value::Null)
        }
        "remediate" => {
            let action = parse_flag(&args, "--action").unwrap_or_default();
            if action.is_empty() {
                serde_json::json!({"actions": remediation_catalog()})
            } else {
                let approve = args.iter().any(|a| a == "--approve");
                let result = run_remediation(&action, approve);
                log_remediation(Path::new(&data_root()), &result);
                serde_json::to_value(&result).unwrap_or(serde_json::Value::Null)
            }
        }
        "help" | "--help" | "-h" => serde_json::json!({
            "commands": [
                "solve --msg '...' [--approve]  — software execute / hardware guide",
                "agentic --msg '...' [--approve]",
                "tools | diagnose-os | deep | remediate | sysprobe"
            ],
            "policy": "Software: allowlist after HITL. Hardware: human only."
        }),
        other => fail(&format!("Amri '{}' haipo.", other)),
    };

    println!("{}", serde_json::to_string_pretty(&out).unwrap());
}

fn parse_flag(args: &[String], flag: &str) -> Option<String> {
    args.iter().position(|a| a == flag).and_then(|i| args.get(i + 1).cloned())
}

fn fail(msg: &str) -> ! {
    eprintln!("ERR: {}", msg);
    std::process::exit(1);
}
