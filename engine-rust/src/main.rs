//! mtaalamu — CLI ya mfumo wa MTAALAMU SMART.
//!
//! Matumizi:
//!   mtaalamu list
//!   mtaalamu calc voltage_drop --inputs "{\"L\":30,\"I\":16,\"A\":2.5,\"V\":230}"
//!   mtaalamu diagnose electrical --symptoms breaker_trips,sparks
//!   mtaalamu tree car --answers true,true,true
//!   mtaalamu rules --data "{\"vd_pct\":4.0}"
//!   mtaalamu test          # endesha test cases zote za JSON
//!
//! Data: inatafuta data/ kutoka cwd au engine-rust/.

use mtaalamu_engine::{
    BayesianDiagnoser, DecisionTree, FormulaEngine, I18n, KnowledgeBase, RulesEngine,
};

fn data_path(name: &str) -> String {
    let candidates = [
        format!("data/{}", name),
        format!("../data/{}", name),
        format!("../../data/{}", name),
        format!("mtaalamu-smart/data/{}", name),
    ];
    candidates
        .into_iter()
        .find(|p| std::path::Path::new(p).exists())
        .unwrap_or_else(|| format!("data/{}", name))
}

fn load_formulas() -> Result<FormulaEngine, String> {
    let mut engine = FormulaEngine::new();
    let mut any = false;
    for d in ["data/formulas", "data/formulas_network"] {
        for candidate in [d.to_string(), format!("../{}", d), format!("../../{}", d)] {
            let p = std::path::Path::new(&candidate);
            if p.exists() {
                engine.load_dir(p)?;
                any = true;
                break;
            }
        }
    }
    // fallback: file moja ya zamani (demo)
    if !any {
        let f = data_path("formulas.json");
        engine.load_file(std::path::Path::new(&f))?;
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
            d.load_file(std::path::Path::new(&data_path("diagnosis.json")))
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
            let t = DecisionTree::load_file(std::path::Path::new(&path))
                .unwrap_or_else(|e| fail(&e));
            t.walk(&answers).unwrap_or_else(|e| fail(&e))
        }
        "rules" => {
            let data = parse_flag(&args, "--data").unwrap_or_else(|| "{}".to_string());
            let data: serde_json::Value = serde_json::from_str(&data)
                .unwrap_or_else(|e| fail(&format!("--data si JSON sahihi: {}", e)));
            let path = data_path("rules/rules_electrical.json");
            let r = RulesEngine::load_file(std::path::Path::new(&path))
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
            k.load_file(std::path::Path::new(&data_path("problems.json")))
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
            let n = i.load_file(std::path::Path::new(&data_path("locales/sw.json")))
                .unwrap_or_else(|e| fail(&e));
            serde_json::json!({
                "keys": n,
                "lang": i.lang().code(),
                "sample": i.bi("calc.button"),
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
                "i18n --lang sw|en                 - jaribu translations"
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
