//! mtaalamu — CLI ya mfumo wa MTAALAMU SMART.
//!
//! Matumizi:
//!   mtaalamu list
//!   mtaalamu calc voltage_drop --inputs "{\"L\":30,\"I\":16,\"A\":2.5,\"V\":230}"
//!   mtaalamu diagnose electrical --symptoms breaker_trips,sparks
//!   mtaalamu tree car --answers true,true,true
//!   mtaalamu rules --data "{\"vd_pct\":4.0}"
//!   mtaalamu test
//!   mtaalamu geo --q Kariakoo
//!   mtaalamu nav --maneuver left --distance 200
//!   mtaalamu hazards
//!   mtaalamu agents
//!   mtaalamu agent --msg "Kompyuta inaenda polepole"
//!   mtaalamu vision
//!   mtaalamu pipeline --session AV-xxx
//!   mtaalamu report --session AV-xxx
//!
//! Data: inatafuta data/ kutoka cwd au engine-rust/.

use mtaalamu_engine::{
    AgentOrchestrator, BayesianDiagnoser, DecisionTree, FormulaEngine, GeoEngine,
    HazardsEngine, I18n, KnowledgeBase, NavigationEngine, PipelineEngine, ReportEngine,
    RulesEngine, VisionEngine,
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
                let mut m: std::collections::BTreeMap<String, usize> =
                    std::collections::BTreeMap::new();
                for t in eng.trades() {
                    m.insert(t.clone(), eng.by_trade(&t).len());
                }
                m
            };
            serde_json::json!({
                "total": eng.count(),
                "trades": by_trade,
                "ids": eng.ids(),
            })
        }
        "calc" => {
            let id = args
                .get(2)
                .unwrap_or_else(|| fail("Usage: calc <formula_id> --inputs '{...}'"));
            let inputs = parse_flag(&args, "--inputs").unwrap_or_else(|| "{}".to_string());
            let inputs: serde_json::Value = serde_json::from_str(&inputs)
                .unwrap_or_else(|e| fail(&format!("--inputs si JSON sahihi: {}", e)));
            let eng = load_formulas().unwrap_or_else(|e| fail(&e));
            eng.calculate(id, &inputs).unwrap_or_else(|e| fail(&e))
        }
        "diagnose" => {
            let model = args
                .get(2)
                .unwrap_or_else(|| fail("Usage: diagnose <model> --symptoms a,b"));
            let syms = parse_flag(&args, "--symptoms").unwrap_or_default();
            let ids: Vec<String> = syms
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            let mut d = BayesianDiagnoser::new();
            d.load_file(Path::new(&data_path("diagnosis.json")))
                .unwrap_or_else(|e| fail(&e));
            d.diagnose(model, &ids).unwrap_or_else(|e| fail(&e))
        }
        "tree" => {
            let name = args
                .get(2)
                .unwrap_or_else(|| fail("Usage: tree <name> --answers true,false,true"));
            let ans = parse_flag(&args, "--answers").unwrap_or_default();
            let answers: Vec<bool> = ans
                .split(',')
                .map(|s| s.trim().to_ascii_lowercase())
                .filter(|s| !s.is_empty())
                .map(|s| match s.as_str() {
                    "true" | "1" | "yes" | "ndiyo" | "y" => true,
                    "false" | "0" | "no" | "hapana" | "n" => false,
                    other => fail(&format!("Jibu '{}' si sahihi (true/false)", other)),
                })
                .collect();
            let path = format!("data/decision_trees/{}.json", name);
            let t = DecisionTree::load_file(Path::new(&path))
                .unwrap_or_else(|e| fail(&e));
            t.walk(&answers).unwrap_or_else(|e| fail(&e))
        }
        "rules" => {
            let data = parse_flag(&args, "--data").unwrap_or_else(|| "{}".to_string());
            let data: serde_json::Value = serde_json::from_str(&data)
                .unwrap_or_else(|e| fail(&format!("--data si JSON sahihi: {}", e)));
            let path = data_path("rules/rules_electrical.json");
            let r = RulesEngine::load_file(Path::new(&path))
                .unwrap_or_else(|e| fail(&e));
            serde_json::json!({
                "trade": r.trade(),
                "total_rules": r.len(),
                "matches": r.evaluate(&data),
            })
        }
        "knowledge" => {
            let trade = parse_flag(&args, "--trade").unwrap_or_else(|| "umeme".to_string());
            let syms = parse_flag(&args, "--symptoms").unwrap_or_default();
            let ids: Vec<String> = syms
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            let mut k = KnowledgeBase::new();
            k.load_file(Path::new(&data_path("problems.json")))
                .unwrap_or_else(|e| fail(&e));
            serde_json::json!({
                "problems": k.problem_count(),
                "hits": k.diagnose(&trade, &ids),
            })
        }
        "test" => {
            let eng = load_formulas().unwrap_or_else(|e| fail(&e));
            let (fails, list) = eng.run_tests();
            serde_json::json!({
                "total": eng.count(),
                "failures": fails,
                "details": list,
            })
        }
        "i18n" => {
            let mut i = I18n::new();
            let lang = parse_flag(&args, "--lang").unwrap_or_else(|| "sw".to_string());
            i.set_lang(&lang);
            let n = i
                .load_file(Path::new(&data_path("locales/sw.json")))
                .unwrap_or_else(|e| fail(&e));
            serde_json::json!({
                "keys": n,
                "lang": i.lang().code(),
                "sample": i.bi("calc.button"),
            })
        }
        // === LOCATION & NAVIGATION ===
        "geo" => {
            let q = parse_flag(&args, "--q").unwrap_or_default();
            let limit: usize = parse_flag(&args, "--limit")
                .and_then(|s| s.parse().ok())
                .unwrap_or(15);
            let root = data_root();
            let eng = GeoEngine::load(Path::new(&root)).unwrap_or_else(|e| fail(&e));
            let hits: Vec<_> = eng
                .search(&q, limit)
                .iter()
                .map(|p| {
                    serde_json::json!({
                        "id": p.id,
                        "name_sw": p.name_sw,
                        "name_en": p.name_en,
                        "level": p.level,
                        "region_id": p.region_id,
                        "district_id": p.district_id,
                    })
                })
                .collect();
            serde_json::json!({
                "query": q,
                "total_places": eng.count(),
                "hits": hits,
            })
        }
        "nav" => {
            let maneuver = parse_flag(&args, "--maneuver").unwrap_or_else(|| "continue".into());
            let distance: Option<f64> = parse_flag(&args, "--distance").and_then(|s| s.parse().ok());
            let root = data_root();
            let eng = NavigationEngine::load(Path::new(&root)).unwrap_or_else(|e| fail(&e));
            let text_sw = eng.instruction_sw(&maneuver, distance);
            serde_json::json!({
                "maneuver": maneuver,
                "distance_m": distance,
                "instruction_sw": text_sw,
                "voice_prompts_loaded": eng.prompt_count(),
                "start": eng.voice("start_nav", &[]),
                "arrived": eng.voice("arrived", &[]),
            })
        }
        "hazards" => {
            let root = data_root();
            let eng = HazardsEngine::load(Path::new(&root)).unwrap_or_else(|e| fail(&e));
            let list: Vec<_> = eng
                .all()
                .iter()
                .map(|h| {
                    serde_json::json!({
                        "id": h.id,
                        "name_sw": h.name_sw,
                        "severity": h.severity,
                        "voice_sw": h.voice_sw,
                    })
                })
                .collect();
            serde_json::json!({
                "count": eng.count(),
                "types": list,
            })
        }
        // === AGENTIC VISION ===
        "agents" => {
            let path = data_path("agents/agents_10.json");
            let orch = AgentOrchestrator::load(Path::new(&path)).unwrap_or_else(|e| fail(&e));
            let list: Vec<_> = orch
                .list_agents()
                .iter()
                .map(|a| {
                    serde_json::json!({
                        "id": a.id,
                        "name_sw": a.name_sw,
                        "role": a.role,
                        "auto": a.auto,
                        "requires_hitl": a.requires_hitl,
                    })
                })
                .collect();
            serde_json::json!({
                "summary": orch.summary(),
                "agents": list,
            })
        }
        "agent" => {
            let msg = parse_flag(&args, "--msg").unwrap_or_else(|| {
                fail("Usage: agent --msg \"tatizo lako\" [--lang sw] [--approve]")
            });
            let lang = parse_flag(&args, "--lang").unwrap_or_else(|| "sw".into());
            let approve = args.iter().any(|a| a == "--approve");
            let path = data_path("agents/agents_10.json");
            let orch = AgentOrchestrator::load(Path::new(&path)).unwrap_or_else(|e| fail(&e));
            let mut session = orch.start_session(&msg, &lang);

            // Run pipeline steps (with optional HITL approve)
            let mut steps_log = Vec::new();
            for _ in 0..12 {
                if approve && !session.hitl_approved {
                    let ev = orch.approve_hitl(&mut session);
                    steps_log.push(serde_json::to_value(&ev).unwrap());
                }
                match orch.advance(&mut session) {
                    Ok(ev) => {
                        steps_log.push(serde_json::to_value(&ev).unwrap());
                        if session.state.as_str() == "completed"
                            || session.state.as_str() == "failed"
                        {
                            break;
                        }
                        if session.state.as_str() == "awaiting_hitl" && !approve {
                            break;
                        }
                    }
                    Err(e) => {
                        steps_log.push(serde_json::json!({"error": e}));
                        break;
                    }
                }
            }

            // Attach vision snapshot
            let root = data_root();
            let vision_json = VisionEngine::load(Path::new(&root))
                .map(|v| v.to_json())
                .unwrap_or(serde_json::json!(null));

            serde_json::json!({
                "session": {
                    "id": session.id,
                    "state": session.state.as_str(),
                    "trade": session.trade,
                    "symptoms": session.symptoms,
                    "hitl_approved": session.hitl_approved,
                    "pipeline_index": session.pipeline_index,
                },
                "events": steps_log,
                "vision": vision_json,
            })
        }
        "vision" => {
            let root = data_root();
            let eng = VisionEngine::load(Path::new(&root)).unwrap_or_else(|e| fail(&e));
            eng.to_json()
        }
        "pipeline" => {
            let path = data_path("vision/pipeline.json");
            let eng = PipelineEngine::load(Path::new(&path)).unwrap_or_else(|e| fail(&e));
            let sid = parse_flag(&args, "--session").unwrap_or_else(|| "AV-demo".into());
            let mut rt = eng.start(&sid);
            // Demo: advance all non-HITL or with --approve
            let approve = args.iter().any(|a| a == "--approve");
            let mut log = Vec::new();
            for _ in 0..eng.defs().len() {
                match eng.advance(
                    &mut rt,
                    serde_json::json!({"ok": true}),
                    "Hatua imekamilika.",
                    approve,
                ) {
                    Ok(()) => log.push(serde_json::json!({
                        "index": rt.current_index,
                        "completed": rt.completed,
                    })),
                    Err(e) => {
                        log.push(serde_json::json!({"blocked": e}));
                        break;
                    }
                }
            }
            serde_json::json!({
                "runtime": rt,
                "log": log,
            })
        }
        "report" => {
            let path = data_path("vision/report_template.json");
            let eng = ReportEngine::load(Path::new(&path)).unwrap_or_else(|e| fail(&e));
            let sid = parse_flag(&args, "--session").unwrap_or_else(|| "AV-demo".into());
            let ctx = serde_json::json!({
                "user_message": "Kompyuta inaenda polepole",
                "symptoms": ["slow_performance", "overheating"],
                "status": "completed",
                "summary_sw": "Tatizo limetatuliwa: michakato hatari imesitishwa, feni imesafishwa kwa maelekezo.",
                "issues": ["CPU juu", "Diski imejaa"],
            });
            let report = eng.build(&sid, &ctx, "sw");
            let md = eng.to_markdown(&report);
            serde_json::json!({
                "report": report,
                "markdown_preview": md.chars().take(800).collect::<String>(),
            })
        }
        "help" | "--help" | "-h" => serde_json::json!({
            "usage": [
                "list                              - orodha ya formula zote kwa trade",
                "calc <id> --inputs '{...}'        - hesabu formula",
                "diagnose <model> --symptoms a,b   - utambuzi wa Bayesian",
                "tree <name> --answers true,false  - tembelea decision tree",
                "rules --data '{...}'              - tathmini rules",
                "knowledge --trade umeme --symptoms a,b - knowledge base",
                "test                              - endesha test cases zote",
                "i18n --lang sw|en                 - jaribu translations",
                "geo --q <jina> [--limit N]        - tafuta mahali (Tanzania)",
                "nav --maneuver left --distance 200 - maelekezo Kiswahili",
                "hazards                           - orodha ya aina za hatari",
                "agents                            - orodha Multi-Agent 10",
                "agent --msg 'tatizo' [--approve]  - anza session Agentic Vision",
                "vision                            - live snapshot (ramani/processes/issues)",
                "pipeline --session ID [--approve] - PIITVD pipeline",
                "report --session ID               - digital book report"
            ]
        }),
        other => fail(&format!("Amri '{}' haipo. Tumia 'help'.", other)),
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
