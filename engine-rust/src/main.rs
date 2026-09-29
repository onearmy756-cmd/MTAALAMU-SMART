//! mtaalamu — CLI
//!   mtaalamu sysprobe [--top 10]
//!   mtaalamu agentic --msg "..." --approve

use mtaalamu_engine::{
    AgentOrchestrator, BayesianDiagnoser, DecisionTree, FormulaEngine, GeoEngine,
    HazardsEngine, I18n, KnowledgeBase, NavigationEngine, PipelineEngine, ReportEngine,
    RulesEngine, VisionEngine, probe_json, run_full,
};
use std::path::Path;

fn data_path(name: &str) -> String {
    let candidates = [
        format!("data/{}", name),
        format!("../data/{}", name),
        format!("../../data/{}", name),
        format!("mtaalamu-smart/data/{}", name),
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
        let f = data_path("formulas.json");
        engine.load_file(Path::new(&f))?;
    }
    Ok(engine)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let cmd = args.get(1).map(|s| s.as_str()).unwrap_or("help");

    let out = match cmd {
        "list" => {
            let eng = load_formulas().unwrap_or_else(|e| fail(&e));
            let by_trade: std::collections::BTreeMap<String, usize> = {
                let mut m = std::collections::BTreeMap::new();
                for t in eng.trades() {
                    m.insert(t.clone(), eng.by_trade(&t).len());
                }
                m
            };
            serde_json::json!({"total": eng.count(), "trades": by_trade, "ids": eng.ids()})
        }
        "calc" => {
            let id = args.get(2).unwrap_or_else(|| fail("Usage: calc <id> --inputs '{...}'"));
            let inputs = parse_flag(&args, "--inputs").unwrap_or_else(|| "{}".into());
            let inputs: serde_json::Value = serde_json::from_str(&inputs)
                .unwrap_or_else(|e| fail(&format!("--inputs: {}", e)));
            load_formulas().unwrap_or_else(|e| fail(&e)).calculate(id, &inputs).unwrap_or_else(|e| fail(&e))
        }
        "diagnose" => {
            let model = args.get(2).unwrap_or_else(|| fail("Usage: diagnose <model> --symptoms a,b"));
            let syms = parse_flag(&args, "--symptoms").unwrap_or_default();
            let ids: Vec<String> = syms.split(',').map(|s| s.trim().into()).filter(|s: &String| !s.is_empty()).collect();
            let mut d = BayesianDiagnoser::new();
            d.load_file(Path::new(&data_path("diagnosis.json"))).unwrap_or_else(|e| fail(&e));
            d.diagnose(model, &ids).unwrap_or_else(|e| fail(&e))
        }
        "tree" => {
            let name = args.get(2).unwrap_or_else(|| fail("Usage: tree <name> --answers true,false"));
            let ans = parse_flag(&args, "--answers").unwrap_or_default();
            let answers: Vec<bool> = ans.split(',').map(|s| s.trim().to_ascii_lowercase()).filter(|s| !s.is_empty())
                .map(|s| match s.as_str() {
                    "true" | "1" | "yes" | "ndiyo" | "y" => true,
                    "false" | "0" | "no" | "hapana" | "n" => false,
                    other => fail(&format!("Jibu '{}'", other)),
                }).collect();
            DecisionTree::load_file(Path::new(&format!("data/decision_trees/{}.json", name)))
                .unwrap_or_else(|e| fail(&e)).walk(&answers).unwrap_or_else(|e| fail(&e))
        }
        "rules" => {
            let data = parse_flag(&args, "--data").unwrap_or_else(|| "{}".into());
            let data: serde_json::Value = serde_json::from_str(&data).unwrap_or_else(|e| fail(&format!("{}", e)));
            let r = RulesEngine::load_file(Path::new(&data_path("rules/rules_electrical.json"))).unwrap_or_else(|e| fail(&e));
            serde_json::json!({"trade": r.trade(), "total_rules": r.len(), "matches": r.evaluate(&data)})
        }
        "knowledge" => {
            let trade = parse_flag(&args, "--trade").unwrap_or_else(|| "umeme".into());
            let syms = parse_flag(&args, "--symptoms").unwrap_or_default();
            let ids: Vec<String> = syms.split(',').map(|s| s.trim().into()).filter(|s: &String| !s.is_empty()).collect();
            let mut k = KnowledgeBase::new();
            k.load_file(Path::new(&data_path("problems.json"))).unwrap_or_else(|e| fail(&e));
            serde_json::json!({"problems": k.problem_count(), "hits": k.diagnose(&trade, &ids)})
        }
        "test" => {
            let eng = load_formulas().unwrap_or_else(|e| fail(&e));
            let (fails, list) = eng.run_tests();
            serde_json::json!({"total": eng.count(), "failures": fails, "details": list})
        }
        "i18n" => {
            let mut i = I18n::new();
            let lang = parse_flag(&args, "--lang").unwrap_or_else(|| "sw".into());
            i.set_lang(&lang);
            let n = i.load_file(Path::new(&data_path("locales/sw.json"))).unwrap_or_else(|e| fail(&e));
            serde_json::json!({"keys": n, "lang": i.lang().code(), "sample": i.bi("calc.button")})
        }
        "geo" => {
            let q = parse_flag(&args, "--q").unwrap_or_default();
            let limit: usize = parse_flag(&args, "--limit").and_then(|s| s.parse().ok()).unwrap_or(15);
            let eng = GeoEngine::load(Path::new(&data_root())).unwrap_or_else(|e| fail(&e));
            let hits: Vec<_> = eng.search(&q, limit).iter().map(|p| serde_json::json!({
                "id": p.id, "name_sw": p.name_sw, "name_en": p.name_en, "level": p.level
            })).collect();
            serde_json::json!({"query": q, "total_places": eng.count(), "hits": hits})
        }
        "nav" => {
            let maneuver = parse_flag(&args, "--maneuver").unwrap_or_else(|| "continue".into());
            let distance: Option<f64> = parse_flag(&args, "--distance").and_then(|s| s.parse().ok());
            let eng = NavigationEngine::load(Path::new(&data_root())).unwrap_or_else(|e| fail(&e));
            serde_json::json!({
                "maneuver": maneuver, "distance_m": distance,
                "instruction_sw": eng.instruction_sw(&maneuver, distance),
                "voice_prompts_loaded": eng.prompt_count()
            })
        }
        "hazards" => {
            let eng = HazardsEngine::load(Path::new(&data_root())).unwrap_or_else(|e| fail(&e));
            let list: Vec<_> = eng.all().iter().map(|h| serde_json::json!({
                "id": h.id, "name_sw": h.name_sw, "severity": h.severity, "voice_sw": h.voice_sw
            })).collect();
            serde_json::json!({"count": eng.count(), "types": list})
        }
        "agents" => {
            let orch = AgentOrchestrator::load(Path::new(&data_path("agents/agents_10.json"))).unwrap_or_else(|e| fail(&e));
            let list: Vec<_> = orch.list_agents().iter().map(|a| serde_json::json!({
                "id": a.id, "name_sw": a.name_sw, "role": a.role, "auto": a.auto, "requires_hitl": a.requires_hitl
            })).collect();
            serde_json::json!({"summary": orch.summary(), "agents": list})
        }
        "agent" | "agentic" => {
            let msg = parse_flag(&args, "--msg").unwrap_or_else(|| fail("Usage: agentic --msg \"...\" [--approve]"));
            let lang = parse_flag(&args, "--lang").unwrap_or_else(|| "sw".into());
            let approve = args.iter().any(|a| a == "--approve");
            match run_full(Path::new(&data_root()), &msg, &lang, approve) {
                Ok(r) => serde_json::json!({
                    "session": {
                        "id": r.session.id,
                        "state": r.session.state.as_str(),
                        "trade": r.session.trade,
                        "symptoms": r.session.symptoms,
                        "hitl_approved": r.session.hitl_approved,
                    },
                    "vision": r.vision,
                    "diagnosis": r.diagnosis_hits,
                    "pipeline": r.pipeline,
                    "report": r.report,
                    "report_md_preview": r.report_md.chars().take(600).collect::<String>(),
                }),
                Err(e) => fail(&e),
            }
        }
        "vision" => VisionEngine::load(Path::new(&data_root())).unwrap_or_else(|e| fail(&e)).to_json(),
        "sysprobe" | "probe" => {
            let top: usize = parse_flag(&args, "--top").and_then(|s| s.parse().ok()).unwrap_or(10);
            probe_json(top)
        }
        "pipeline" => {
            let eng = PipelineEngine::load(Path::new(&data_path("vision/pipeline.json"))).unwrap_or_else(|e| fail(&e));
            let sid = parse_flag(&args, "--session").unwrap_or_else(|| "AV-demo".into());
            let mut rt = eng.start(&sid);
            let approve = args.iter().any(|a| a == "--approve");
            let mut log = Vec::new();
            for _ in 0..eng.defs().len() {
                match eng.advance(&mut rt, serde_json::json!({"ok": true}), "Hatua imekamilika.", approve) {
                    Ok(()) => log.push(serde_json::json!({"index": rt.current_index, "completed": rt.completed})),
                    Err(e) => { log.push(serde_json::json!({"blocked": e})); break; }
                }
            }
            serde_json::json!({"runtime": rt, "log": log})
        }
        "report" => {
            let eng = ReportEngine::load(Path::new(&data_path("vision/report_template.json"))).unwrap_or_else(|e| fail(&e));
            let sid = parse_flag(&args, "--session").unwrap_or_else(|| "AV-demo".into());
            let ctx = serde_json::json!({
                "user_message": "Kompyuta inaenda polepole",
                "symptoms": ["slow_performance"],
                "status": "completed",
                "summary_sw": "Tatizo limetatuliwa.",
                "issues": ["CPU juu"]
            });
            let report = eng.build(&sid, &ctx, "sw");
            let md = eng.to_markdown(&report);
            serde_json::json!({"report": report, "markdown_preview": md.chars().take(800).collect::<String>()})
        }
        "help" | "--help" | "-h" => serde_json::json!({
            "usage": [
                "list | calc | diagnose | tree | rules | knowledge | test | i18n",
                "geo | nav | hazards",
                "agents | agentic --msg '...' [--approve] | vision | sysprobe [--top N] | pipeline | report",
            ]
        }),
        other => fail(&format!("Amri '{}' haipo. Tumia 'help'.", other)),
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
