//! mtaalamu CLI — reason / knowledge / devices / solve / agentic

use mtaalamu_engine::{
    AgentOrchestrator, BayesianDiagnoser, FormulaEngine, auto_diagnose, auto_watch, auto_work_once,
    deep_probe_json, deploy, devices_catalog_stats, discover_tools, grounded_reason_json,
    knowledge_search, knowledge_stats, load_sessions, log_remediation, probe_json,
    remediation_catalog, run_diagnostic, run_full, run_remediation, save_session, search_devices,
    solve_message, system_wiring_json,
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
        "reason" | "think" | "fikiri" => {
            let msg = parse_flag(&args, "--msg")
                .unwrap_or_else(|| fail("reason --msg \"...\""));
            grounded_reason_json(Path::new(&data_root()), &msg)
        }
        "agent" | "agentic" => {
            let msg = parse_flag(&args, "--msg").unwrap_or_else(|| fail("agentic --msg"));
            let lang = parse_flag(&args, "--lang").unwrap_or_else(|| "sw".into());
            let approve = args.iter().any(|a| a == "--approve");
            match run_full(Path::new(&data_root()), &msg, &lang, approve) {
                Ok(r) => {
                    let solve_json = r.solve.as_ref().map(|s| {
                        serde_json::json!({
                            "domain": s.plan.domain,
                            "summary_sw": s.summary_sw,
                            "executed": s.executed.iter().map(|a| {
                                serde_json::json!({
                                    "id": a.action_id,
                                    "ok": a.ok,
                                    "message_sw": a.message_sw
                                })
                            }).collect::<Vec<_>>(),
                            "skipped": s.skipped,
                            "hitl_approved": s.hitl_approved,
                        })
                    });
                    serde_json::json!({
                        "session": {
                            "id": r.session.id,
                            "state": r.session.state.as_str(),
                            "hitl_approved": r.session.hitl_approved,
                            "trade": r.session.trade,
                            "symptoms": r.session.symptoms,
                        },
                        "role": "Agent inafanya kazi; msimamizi anaruhusu (--approve) tu",
                        "vision": r.vision,
                        "solve": solve_json,
                        "knowledge": knowledge_search(Path::new(&data_root()), &msg, 5),
                        "devices": search_devices(Path::new(&data_root()), &msg, 5),
                        "grounded": grounded_reason_json(Path::new(&data_root()), &msg),
                        "report_preview": r.report,
                    })
                }
                Err(e) => fail(&e),
            }
        }
        "knowledge" | "know" => {
            let msg = parse_flag(&args, "--msg")
                .unwrap_or_else(|| fail("knowledge --msg \"...\""));
            let limit: usize = parse_flag(&args, "--limit")
                .and_then(|s| s.parse().ok())
                .unwrap_or(8);
            serde_json::to_value(knowledge_search(Path::new(&data_root()), &msg, limit))
                .unwrap_or(serde_json::Value::Null)
        }
        "knowledge-stats" => knowledge_stats(Path::new(&data_root())),
        "devices" | "device-stats" => devices_catalog_stats(Path::new(&data_root())),
        "device-solve" | "electronic" => {
            let msg = parse_flag(&args, "--msg")
                .unwrap_or_else(|| fail("device-solve --msg \"TV haiwaki\""));
            let limit: usize = parse_flag(&args, "--limit")
                .and_then(|s| s.parse().ok())
                .unwrap_or(8);
            serde_json::to_value(search_devices(Path::new(&data_root()), &msg, limit))
                .unwrap_or(serde_json::Value::Null)
        }
        "solve" => {
            let msg = parse_flag(&args, "--msg")
                .unwrap_or_else(|| fail("solve --msg \"...\" [--approve]"));
            let approve = args.iter().any(|a| a == "--approve");
            serde_json::json!({
                "software_solve": solve_message(Path::new(&data_root()), &msg, approve),
                "knowledge": knowledge_search(Path::new(&data_root()), &msg, 5),
                "devices": search_devices(Path::new(&data_root()), &msg, 5),
                "grounded": grounded_reason_json(Path::new(&data_root()), &msg),
            })
        }
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
        "wiring" => {
            // Ramani + miunganisho HALISI ya vifaa (P: spec ya Agentic Vision)
            let top: usize = parse_flag(&args, "--top").and_then(|s| s.parse().ok()).unwrap_or(10);
            system_wiring_json(top)
        }
        "av-auto" => {
            // Auto-work mara moja: scan → HITL → solve (lab: --approve)
            let approve = args.iter().any(|a| a == "--approve");
            let sid = format!(
                "AW-{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0)
            );
            let rep = auto_work_once(Path::new(&data_root()), &sid, approve);
            let _ = save_session(
                Path::new(&data_root()),
                &sid,
                &serde_json::to_value(&rep).unwrap_or(serde_json::Value::Null),
            );
            serde_json::to_value(&rep).unwrap_or(serde_json::Value::Null)
        }
        "av-watch" => {
            // Auto-work loop: kila N sekunde (production daemon)
            let secs: u64 = parse_flag(&args, "--interval").and_then(|s| s.parse().ok()).unwrap_or(60);
            let runs: Option<u32> = parse_flag(&args, "--runs").and_then(|s| s.parse().ok());
            let approve = args.iter().any(|a| a == "--approve");
            auto_watch(Path::new(&data_root()), secs, approve, runs);
            serde_json::json!({"ok": true})
        }
        "av-sessions" => serde_json::to_value(load_sessions(Path::new(&data_root())))
            .unwrap_or(serde_json::Value::Null),
        "deploy" => {
            // Fundi Deploy integration — `mtaalamu deploy <sub> [...flags]`
            let sub = args.get(2).map(|s| s.as_str()).unwrap_or("summary");
            match sub {
                "hosts" | "discover" => deploy::discover().unwrap_or_else(|e| fail(&e)),
                "jobs" => deploy::jobs().unwrap_or_else(|e| fail(&e)),
                "images" => deploy::images().unwrap_or_else(|e| fail(&e)),
                "summary" => deploy::summary().unwrap_or_else(|e| fail(&e)),
                "cloud" => deploy::cloud_status().unwrap_or_else(|e| fail(&e)),
                "os" => {
                    let specs = parse_flag(&args, "--specs").unwrap_or_default();
                    let need = parse_flag(&args, "--need").unwrap_or_else(|| "office".into());
                    deploy::os_select(&specs, &need).unwrap_or_else(|e| fail(&e))
                }
                "start" => {
                    let macs = parse_flag(&args, "--macs").unwrap_or_default();
                    if macs.is_empty() {
                        fail("deploy start --macs aa:bb:cc:dd:ee:01,aa:bb:... [--os auto] [--need office]");
                    }
                    let computers: Vec<serde_json::Value> = macs
                        .split(',')
                        .map(|m| serde_json::json!({ "mac": m.trim(), "name": format!("PC-{}", m.trim().replace(':', "")) }))
                        .collect();
                    let os = parse_flag(&args, "--os").unwrap_or_else(|| "auto".into());
                    let need = parse_flag(&args, "--need").unwrap_or_else(|| "office".into());
                    deploy::deploy_hitl(serde_json::Value::Array(computers), &os, &need)
                        .unwrap_or_else(|e| fail(&e))
                }
                "approve" => {
                    let id = parse_flag(&args, "--id").unwrap_or_else(|| fail("deploy approve --id <job_id>"));
                    deploy::approve(&id).unwrap_or_else(|e| fail(&e))
                }
                "cancel" => {
                    let id = parse_flag(&args, "--id").unwrap_or_else(|| fail("deploy cancel --id <job_id>"));
                    deploy::cancel(&id).unwrap_or_else(|e| fail(&e))
                }
                other => fail(&format!("deploy sub '{}' haipo (hosts|jobs|images|summary|cloud|os|start|approve|cancel)", other)),
            }
        }
        "help" | "--help" | "-h" => serde_json::json!({
            "commands": [
                "reason --msg '...' — fikra + calculus + jibu refu (no hallucination)",
                "knowledge --msg '...'",
                "devices | device-solve --msg '...'",
                "solve --msg '...' [--approve]",
                "agentic --msg '...' [--approve]  — agent inafanya kazi; --approve = msimamizi",
                "wiring — ramani + miunganisho HALISI ya vifaa (PCI/USB/disk/net/thermal)",
                "av-auto [--approve] — auto-work: scan → gundua → HITL → solve",
                "av-watch --interval 60 [--runs 5] — auto-work loop (daemon)",
                "av-sessions — sessions zilizohifadhiwa",
                "deploy hosts|jobs|images|summary|cloud — Fundi Deploy status",
                "deploy os --specs '8gb ram i5' --need office — pendekezo la OS",
                "deploy start --macs aa:bb:... --os auto — anza deploy (HITL)",
                "deploy approve|cancel --id <job> — msimamizi anaruhusu/ghairi"
            ],
            "policy": "Evidence + calculus only. Hardware = human guide. Agent works; supervisor approves."
        }),
        other => fail(&format!("Amri '{}' haipo.", other)),
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
