//! mtaalamu CLI — production + OS toolbox
//!   mtaalamu tools
//!   mtaalamu diagnose-os
//!   mtaalamu tool-run --id df

use mtaalamu_engine::{
    AgentOrchestrator, BayesianDiagnoser, FormulaEngine, GeoEngine, PipelineEngine,
    ReportEngine, VisionEngine, auto_diagnose, deep_probe_json, discover_tools,
    log_remediation, probe_json, remediation_catalog, run_diagnostic, run_full,
    run_remediation,
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
            serde_json::json!({"total": eng.count(), "ids": eng.ids()})
        }
        "calc" => {
            let id = args.get(2).unwrap_or_else(|| fail("calc <id> --inputs '{...}'"));
            let inputs = parse_flag(&args, "--inputs").unwrap_or_else(|| "{}".into());
            let inputs: serde_json::Value =
                serde_json::from_str(&inputs).unwrap_or_else(|e| fail(&format!("{}", e)));
            load_formulas()
                .unwrap_or_else(|e| fail(&e))
                .calculate(id, &inputs)
                .unwrap_or_else(|e| fail(&e))
        }
        "diagnose" => {
            let model = args.get(2).unwrap_or_else(|| fail("diagnose <model> --symptoms a,b"));
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
                .unwrap_or_else(|| fail("agentic --msg \"...\" [--approve]"));
            let lang = parse_flag(&args, "--lang").unwrap_or_else(|| "sw".into());
            let approve = args.iter().any(|a| a == "--approve");
            match run_full(Path::new(&data_root()), &msg, &lang, approve) {
                Ok(r) => serde_json::json!({
                    "session": { "id": r.session.id, "state": r.session.state.as_str(),
                                  "hitl_approved": r.session.hitl_approved },
                    "vision": r.vision,
                    "diagnosis": r.diagnosis_hits,
                    "pipeline": r.pipeline,
                    "report_md_preview": r.report_md.chars().take(500).collect::<String>(),
                }),
                Err(e) => fail(&e),
            }
        }
        "vision" => VisionEngine::load(Path::new(&data_root()))
            .unwrap_or_else(|e| fail(&e))
            .to_json(),
        "sysprobe" | "probe" => {
            let top: usize = parse_flag(&args, "--top").and_then(|s| s.parse().ok()).unwrap_or(10);
            probe_json(top)
        }
        "deep" | "deepprobe" => {
            let top: usize = parse_flag(&args, "--top").and_then(|s| s.parse().ok()).unwrap_or(15);
            deep_probe_json(top)
        }
        "tools" | "toolbox" => serde_json::to_value(discover_tools()).unwrap_or(serde_json::Value::Null),
        "diagnose-os" | "os-diagnose" | "autodiag" => auto_diagnose(),
        "tool-run" => {
            let id = parse_flag(&args, "--id").unwrap_or_else(|| fail("tool-run --id df"));
            serde_json::to_value(run_diagnostic(&id)).unwrap_or(serde_json::Value::Null)
        }
        "remediate" | "fix" => {
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
        "pipeline" => {
            let eng = PipelineEngine::load(Path::new(&data_path("vision/pipeline.json")))
                .unwrap_or_else(|e| fail(&e));
            let mut rt = eng.start("AV");
            let approve = args.iter().any(|a| a == "--approve");
            for _ in 0..eng.defs().len() {
                if eng.advance(&mut rt, serde_json::json!({"ok": true}), "ok", approve).is_err() {
                    break;
                }
            }
            serde_json::to_value(&rt).unwrap_or(serde_json::Value::Null)
        }
        "report" => {
            let eng = ReportEngine::load(Path::new(&data_path("vision/report_template.json")))
                .unwrap_or_else(|e| fail(&e));
            let report = eng.build("AV", &serde_json::json!({
                "user_message": "probe", "status": "completed", "summary_sw": "OK", "issues": []
            }), "sw");
            serde_json::json!({"markdown": eng.to_markdown(&report).chars().take(400).collect::<String>()})
        }
        "geo" => {
            let q = parse_flag(&args, "--q").unwrap_or_default();
            let eng = GeoEngine::load(Path::new(&data_root())).unwrap_or_else(|e| fail(&e));
            serde_json::json!({"hits": eng.search(&q, 10).len(), "total": eng.count()})
        }
        "help" | "--help" | "-h" => serde_json::json!({
            "commands": {
                "tools": "Gundua zana kwenye PATH + download hints",
                "diagnose-os": "Endesha diagnostics kwa zana zilizopo (allowlist)",
                "tool-run --id df": "Endesha zana moja (df, free, ss, smartctl, ...)",
                "deep / sysprobe / vision / agentic": "Probe + agentic",
                "remediate [--action ID] [--approve]": "Fix salama"
            },
            "policy": [
                "Hakuna auto-download ya binary — fungua download_url mwenyewe",
                "Allowlist tu — si shell arbitrary",
                "smartctl/sensors/osquery: install kwa mkono kisha tools itazigundua"
            ]
        }),
        other => fail(&format!("Amri '{}' haipo. Tumia help.", other)),
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
