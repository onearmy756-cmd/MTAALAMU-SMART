//! FUNDI MOBILE — Daktari wa Simu (agentic, HITL daima)
//!
//! Mifano:
//!   fundi-mobile catalog                          # huduma + bei (TZS)
//!   fundi-mobile brands samsung                   # combos + flash (smartphone)
//!   fundi-mobile button nokia                     # master reset codes (button)
//!   fundi-mobile devices                          # adb devices + iPhone
//!   fundi-mobile diagnostics                      # battery/storage/props (halisi)
//!   fundi-mobile consent add --name "Fatuma" --phone 0712... --imei 354... \
//!                  --service reset_password_recovery --brand samsung --model A12
//!   fundi-mobile consent form                     # fomu ya kusaini (print)
//!   fundi-mobile agentic run --customer Fatuma --brand samsung --model A12 \
//!                  --imei 354... --service reset_password_adb --problem "nimesahau password"
//!   fundi-mobile dashboard
//!   fundi-mobile book <job_id> out.html
//!   fundi-mobile menu                             # interactive

use anyhow::{bail, Result};
use fundi_mobile::{agentic, b2b, brands, consent, datarec, devices, drivers, emailagent, errcodes, govagent, licenses, mlinzi, netagentic, netcalc, netdiag, osremote, payments, procedures, proagentic, proservices, recover, resetagent, restore, scribe, services, shield};
use std::path::{Path, PathBuf};

fn arg_flag(args: &[String], flag: &str) -> Option<String> {
    args.iter()
        .position(|a| a == flag)
        .and_then(|i| args.get(i + 1).cloned())
}

fn data_root() -> PathBuf {
    // fundi-mobile inaendesha kutoka fundi-mobile/ kwa kawaida → data iko ../data
    match std::env::var("FUNDI_DATA") {
        Ok(p) => PathBuf::from(p),
        Err(_) => {
            for c in ["../data", "data"] {
                let p = Path::new(c);
                if p.exists() {
                    return p.to_path_buf();
                }
            }
            PathBuf::from("../data")
        }
    }
}

fn main() -> Result<()> {
    std::env::set_var("FUNDI_DATA", data_root());
    banner();
    let args: Vec<String> = std::env::args().skip(1).collect();
    let cmd = args.first().map(|s| s.as_str()).unwrap_or("help");
    let r: Result<()> = match cmd {
        "catalog" | "services" => {
            println!("{}", services::catalog_sw());
            Ok(())
        }
        "brands" => {
            let b = args.get(1).map(|s| s.as_str()).unwrap_or("samsung");
            println!("Recovery combo: {}", brands::recovery_combo(b));
            println!("Download mode : {}", brands::download_mode(b));
            let (tool, site, drv) = brands::flash_info(b);
            println!("Flash         : {tool} · firmware: {site} · drivers: {drv}");
            Ok(())
        }
        "button" => {
            let b = args.get(1).map(|s| s.as_str()).unwrap_or("nokia");
            let info = brands::button_phone(b);
            println!("🔔 BUTTON PHONE — {b}");
            println!("Master reset codes:");
            for c in &info.master_codes {
                println!("  • {c}");
            }
            println!("Hard reset : {}", info.hard_reset_sw);
            println!("Flash tool : {}", info.flash_tool);
            Ok(())
        }
        "button-codes" => {
            // Orodha ya codes zote za button (kwa duka)
            let data = brands::load()?;
            for (k, v) in &data.button_phones {
                println!("{k}:");
                for c in &v.master_codes {
                    println!("  {c}");
                }
                println!("  hard: {}", v.hard_reset_sw);
            }
            Ok(())
        }
        "devices" => {
            match devices::adb_devices() {
                Ok(list) if list.is_empty() => println!("Hakuna simu ya Android kwenye ADB."),
                Ok(list) => {
                    println!("Android (ADB):");
                    for (s, st) in &list {
                        println!("  {s}  {st}");
                    }
                }
                Err(e) => println!("{e}"),
            }
            let iphones = devices::idevice_list();
            if !iphones.is_empty() {
                println!("iPhone (idevice):");
                for i in iphones {
                    println!("  {i}");
                }
            }
            Ok(())
        }
        "diagnostics" => {
            let o = procedures::diagnostics()?;
            println!("{}", o.summary_sw);
            for s in &o.steps {
                println!("  {s}");
            }
            Ok(())
        }
        "consent" => {
            let sub = args.get(1).map(|s| s.as_str()).unwrap_or("form");
            match sub {
                "form" => println!("{}", consent::form_template()),
                "list" => {
                    for c in consent::list() {
                        println!(
                            "{} | {} | {} {} | IMEI {} | {} | TZS {}",
                            c.consent_id, c.ts, c.brand, c.model, c.imei, c.service_id, c.price_tzs
                        );
                    }
                }
                "add" => {
                    let id = consent::record(
                        &arg_flag(&args, "--name").unwrap_or_default(),
                        &arg_flag(&args, "--phone").unwrap_or_default(),
                        arg_flag(&args, "--id").as_deref(),
                        &arg_flag(&args, "--brand").unwrap_or_default(),
                        &arg_flag(&args, "--model").unwrap_or_default(),
                        &arg_flag(&args, "--imei").unwrap_or_default(),
                        &arg_flag(&args, "--service").unwrap_or_default(),
                        args.iter().any(|a| a == "--destroys"),
                        arg_flag(&args, "--price").and_then(|p| p.parse().ok()).unwrap_or(0),
                        &arg_flag(&args, "--tech").unwrap_or_else(|| "fundi".into()),
                    )?;
                    println!("✅ Consent imehifadhiwa: {id}");
                }
                other => bail!("consent '{other}': form | list | add"),
            }
            Ok(())
        }
        "agentic" => {
            let sub = args.get(1).map(|s| s.as_str()).unwrap_or("run");
            match sub {
                "run" => {
                    let req = agentic::AgenticRequest {
                        customer: arg_flag(&args, "--customer").unwrap_or_else(|| "Mteja".into()),
                        phone: arg_flag(&args, "--phone").unwrap_or_default(),
                        brand: arg_flag(&args, "--brand").unwrap_or_else(|| "generic".into()),
                        model: arg_flag(&args, "--model").unwrap_or_else(|| "unknown".into()),
                        imei: arg_flag(&args, "--imei").unwrap_or_default(),
                        service_id: arg_flag(&args, "--service").unwrap_or_else(|| "diagnostics".into()),
                        problem: arg_flag(&args, "--problem").unwrap_or_else(|| "tatizo haijaelezwa".into()),
                        technician: arg_flag(&args, "--tech").unwrap_or_else(|| "fundi".into()),
                    };
                    let job = agentic::run(&req)?;
                    println!("\n▶ AGENTIC RUN — {}", job.id);
                    for f in &job.frames {
                        let code = f["code"].as_str().unwrap_or("?");
                        let agent = f["agent"].as_str().unwrap_or("?");
                        let sw = f["sw"].as_str().unwrap_or("");
                        println!("  [{code}] {agent}: {sw}");
                    }
                    println!("\nHali: {} — {}", job.status, job.summary_sw);
                    println!("Kitabu: fundi-mobile book {} out.html", job.id);
                }
                "jobs" => {
                    for j in agentic::list_jobs() {
                        println!("{} | {} | {} | {}", j.id, j.customer, j.service_id, j.status);
                    }
                }
                other => bail!("agentic '{other}': run | jobs"),
            }
            Ok(())
        }
        "book" => {
            let id = args.get(1).cloned().unwrap_or_default();
            let out = args.get(2).cloned().unwrap_or_else(|| "book.html".into());
            let jobs = agentic::list_jobs();
            let job = jobs
                .iter()
                .find(|j| j.id == id)
                .ok_or_else(|| anyhow::anyhow!("Job '{id}' haipo"))?;
            std::fs::write(&out, agentic::book_html(job))?;
            println!("📕 Kitabu: {out}");
            Ok(())
        }
        "dashboard" => {
            let d = agentic::dashboard();
            println!("{}", serde_json::to_string_pretty(&d)?);
            Ok(())
        }
        "invoice" => {
            let sub = args.get(1).map(|s| s.as_str()).unwrap_or("add");
            match sub {
                "add" => {
                    // --customer N --phone P [--company C] [--item desc:qty:price ...] [--service id[:qty] ...]
                    let items = collect_items(&args)?;
                    let inv = payments::invoice_add(
                        &arg_flag(&args, "--customer").unwrap_or_default(),
                        &arg_flag(&args, "--phone").unwrap_or_default(),
                        arg_flag(&args, "--company").as_deref(),
                        items,
                    )?;
                    println!("✅ Invoice {}: TZS {} — {}", inv.id, inv.total, inv.customer);
                    println!("   Malipo: fundi-mobile pay checkout --invoice {} --provider mpesa", inv.id);
                }
                "list" => {
                    for i in payments::invoices() {
                        println!("{} | {} | TZS {} | {}", i.id, i.customer, i.total, i.status);
                    }
                }
                "html" => {
                    let id = arg_flag(&args, "--invoice").unwrap_or_default();
                    let out = arg_flag(&args, "--out").unwrap_or_else(|| format!("{id}.html"));
                    let inv = payments::invoices()
                        .into_iter()
                        .find(|i| i.id == id)
                        .ok_or_else(|| anyhow::anyhow!("Invoice '{id}' haipo"))?;
                    std::fs::write(&out, payments::invoice_html(&inv))?;
                    println!("🧾 Invoice HTML: {out}");
                }
                other => bail!("invoice '{other}': add | list | html"),
            }
            Ok(())
        }
        "pay" => {
            let sub = args.get(1).map(|s| s.as_str()).unwrap_or("checkout");
            match sub {
                "checkout" => {
                    let id = arg_flag(&args, "--invoice").unwrap_or_else(|| bail_flag("--invoice"));
                    let provider = arg_flag(&args, "--provider").unwrap_or_else(|| "mpesa".into());
                    let ck = payments::checkout(&id, &provider)?;
                    println!("\n💳 CHECKOUT — {} kwa {} (TZS {})\n", ck.provider, ck.invoice_id, ck.amount);
                    println!("   Ref: {}", ck.checkout_ref);
                    for s in &ck.instructions_sw {
                        println!("   • {s}");
                    }
                }
                "confirm" => {
                    let id = arg_flag(&args, "--invoice").unwrap_or_else(|| bail_flag("--invoice"));
                    let provider = arg_flag(&args, "--provider").unwrap_or_else(|| "cash".into());
                    let reference = arg_flag(&args, "--ref").unwrap_or_default();
                    let inv = payments::confirm(&id, &provider, &reference, "fundi")?;
                    println!("✅ Malipo yamepokelewa: TZS {} ({}) — {}", inv.total, inv.provider.clone().unwrap_or_default(), inv.pay_ref.clone().unwrap_or_default());
                }
                "providers" => {
                    for p in payments::providers() {
                        println!("{:<10} {:<28} mode: {}{}",
                            p.id, p.name, p.mode,
                            p.ussd.as_ref().map(|u| format!(" · {u}")).unwrap_or_default());
                    }
                }
                other => bail!("pay '{other}': checkout | confirm | providers"),
            }
            Ok(())
        }
        "report" => {
            // Mapato: leo / mwezi / madeni / kwa provider
            let r = payments::report();
            println!("\n💰 MAPATO (FUNDI PAY)");
            println!("   Leo:        TZS {}", r["today"].as_u64().unwrap_or(0));
            println!("   Mwezi huu:  TZS {}", r["month"].as_u64().unwrap_or(0));
            println!("   Madeni:     TZS {} (invoices {})",
                r["unpaid_total"].as_u64().unwrap_or(0),
                r["unpaid_invoices"].as_array().map(|a| a.len()).unwrap_or(0));
            println!("   Malipo yote: {}", r["payments_count"].as_u64().unwrap_or(0));
            if let Some(bp) = r["by_provider"].as_object() {
                println!("   Kwa provider:");
                for (k, v) in bp {
                    println!("     {k}: TZS {}", v.as_u64().unwrap_or(0));
                }
            }
            Ok(())
        }
        "b2b" => {
            // Zana za fundi kwa MAKAMPUNI: quote → invoice (bei za fundi mwenyewe)
            let sub = args.get(1).map(|s| s.as_str()).unwrap_or("list");
            match sub {
                "quote" => {
                    // --fundi N --company C --contact T [--site S] --item desc:qty:price [--note "..."]
                    let raw_items = collect_items(&args)?;
                    let items: Vec<b2b::B2BItem> = raw_items
                        .into_iter()
                        .map(|i| b2b::B2BItem { desc: i.desc, qty: i.qty, price: i.price })
                        .collect();
                    let q = b2b::quote_add(
                        &arg_flag(&args, "--fundi").unwrap_or_else(|| "fundi".into()),
                        &arg_flag(&args, "--company").unwrap_or_default(),
                        &arg_flag(&args, "--contact").unwrap_or_default(),
                        arg_flag(&args, "--site").as_deref(),
                        items,
                        arg_flag(&args, "--note").as_deref(),
                    )?;
                    println!("📄 Quote {}: TZS {} — {}", q.id, q.total, q.company);
                    println!("   HTML: fundi-mobile b2b html --id {} --out {}.html", q.id, q.id);
                }
                "to-invoice" => {
                    let qid = arg_flag(&args, "--id").unwrap_or_else(|| bail_flag("--id"));
                    let inv = b2b::quote_to_invoice(&qid)?;
                    println!("🧾 Invoice {} (kutoka quote {}) — TZS {}", inv.id, qid, inv.total);
                }
                "status" => {
                    let id = arg_flag(&args, "--id").unwrap_or_else(|| bail_flag("--id"));
                    let st = arg_flag(&args, "--set").unwrap_or_else(|| bail_flag("--set"));
                    let d = b2b::set_status(&id, &st)?;
                    println!("✅ {} → {}", d.id, d.status);
                }
                "list" => {
                    let kind = args.get(2).map(|s| s.as_str());
                    for d in b2b::list(kind) {
                        println!("{} | {} | {} | TZS {} | {}",
                            d.id, d.kind, d.company, d.total, d.status);
                    }
                }
                "html" => {
                    let id = arg_flag(&args, "--id").unwrap_or_else(|| bail_flag("--id"));
                    let out = arg_flag(&args, "--out").unwrap_or_else(|| format!("{id}.html"));
                    let doc = b2b::list(None)
                        .into_iter()
                        .find(|d| d.id == id)
                        .ok_or_else(|| anyhow::anyhow!("'{id}' haipo"))?;
                    std::fs::write(&out, b2b::doc_html(&doc))?;
                    println!("📄 Hati: {out}");
                }
                "pdf" => {
                    // PDF rasmi yenye logo ya FUNDI — kwa kampuni/print
                    let id = arg_flag(&args, "--id").unwrap_or_else(|| bail_flag("--id"));
                    let out = arg_flag(&args, "--out").unwrap_or_else(|| format!("{id}.pdf"));
                    let doc = b2b::list(None)
                        .into_iter()
                        .find(|d| d.id == id)
                        .ok_or_else(|| anyhow::anyhow!("'{id}' haipo"))?;
                    let bytes = b2b::doc_pdf(&doc);
                    std::fs::write(&out, &bytes)?;
                    println!("📑 PDF: {out} ({} KB, logo ya FUNDI)", bytes.len() / 1024);
                }
                "report" => {
                    let r = b2b::report();
                    println!("\n📊 B2B (kazi za kampuni — pesa ni za FUNDI mwenyewe)");
                    println!("   Quotes: {} (zimekubaliwa: {}, TZS {})",
                        r["quotes"]["total"].as_u64().unwrap_or(0),
                        r["quotes"]["accepted"].as_u64().unwrap_or(0),
                        r["quotes"]["value_accepted_tzs"].as_u64().unwrap_or(0));
                    println!("   Invoices: {} (paid: {}, TZS {}; unpaid: TZS {})",
                        r["invoices"]["total"].as_u64().unwrap_or(0),
                        r["invoices"]["paid"].as_u64().unwrap_or(0),
                        r["invoices"]["value_paid_tzs"].as_u64().unwrap_or(0),
                        r["invoices"]["value_unpaid_tzs"].as_u64().unwrap_or(0));
                }
                other => bail!("b2b '{other}': quote | to-invoice | status | list | html | pdf | report"),
            }
            Ok(())
        }
        "pro" => {
            // FUNDI PRO — huduma za ofisi (email/gov/recovery/backup/security)
            let sub = args.get(1).map(|s| s.as_str()).unwrap_or("list");
            let out = match sub {
                "list" => proservices::catalog_sw(),
                "show" => {
                    let id = args.get(2).cloned().unwrap_or_default();
                    if id.is_empty() { proservices::catalog_sw() } else { proservices::show_sw(&id) }
                }
                "price" => {
                    let id = args.get(2).cloned().unwrap_or_else(|| bail_flag("--id"));
                    let subs: Vec<String> = args.iter().skip(3).cloned().collect();
                    match proservices::price(&id, &subs) {
                        Ok(p) => p,
                        Err(e) => format!("❌ {e}"),
                    }
                }
                other => bail!("pro '{other}': list | show <id> | price <id> [sub...]"),
            };
            println!("{out}");
            Ok(())
        }
        "netcalc" => {
            // TELECOMS & NETWORK kikokotoo halisi
            let sub = args.get(1).map(|s| s.as_str()).unwrap_or("list");
            match sub {
                "list" => println!("{}", netcalc::list()),
                "problems" => println!("{}", netcalc::problems_sw()),
                "formula" => {
                    let id = args.get(2).map(|s| s.as_str()).unwrap_or("");
                    println!("{}", netcalc::formula_sw(id));
                }
                compute => {
                    // chakata --key value pairs
                    let mut pairs: Vec<(String, String)> = Vec::new();
                    let mut it = args.iter().skip(2);
                    while let Some(k) = it.next() {
                        if let Some(v) = it.next() {
                            pairs.push((k.trim_start_matches("--").to_string(), v.clone()));
                        }
                    }
                    match netcalc::calc(compute, &pairs) {
                        Ok(out) => println!("{out}"),
                        Err(e) => eprintln!("❌ {e}"),
                    }
                }
            }
            Ok(())
        }
        "shield" => {
            // FUNDI SHIELD — antivirus + security audit (scan halisi; clean ni HITL)
            let sub = args.get(1).map(|s| s.as_str()).unwrap_or("audit");
            match sub {
                "audit" => {
                    let a = shield::audit()?;
                    println!("\n🛡️ FUNDI SHIELD — AUDIT");
                    println!("  Antivirus : {} ({})", a["antivirus"]["product"].as_str().unwrap_or("?"),
                        if a["antivirus"]["antivirus_enabled"].as_bool().unwrap_or(false) { "ON ✅" } else { "OFF ❌" });
                    println!("  Real-time : {}", if a["antivirus"]["realtime_enabled"].as_bool().unwrap_or(false) { "ON ✅" } else { "OFF ❌" });
                    println!("  Firewall  : {}", if a["firewall_on"].as_bool().unwrap_or(false) { "ON ✅" } else { "OFF ❌" });
                    println!("  Startup   : {} entries", a["startup"]["entries"].as_i64().unwrap_or(0));
                    println!("  Temp      : {} MB ({} files)", a["temp"]["size_mb"].as_f64().unwrap_or(0.0), a["temp"]["files"].as_i64().unwrap_or(0));
                    if let Some(recs) = a["recommendations"].as_array() {
                        println!("\n  Mapendekezo:");
                        for r in recs { println!("   {}", r.as_str().unwrap_or("")); }
                    }
                }
                "scan" => {
                    let with_eicar = args.iter().any(|a| a == "--eicar");
                    println!("⏳ Inascan (temp + working dir, EICAR test: {with_eicar})…");
                    let s = shield::scan(with_eicar)?;
                    println!("\n🛡️ SCAN: files {} · detections {}", s["files_scanned"].as_i64().unwrap_or(0), s["detections"].as_array().map(|a| a.len()).unwrap_or(0));
                    if let Some(dets) = s["detections"].as_array() {
                        for d in dets {
                            println!("  ⚠️ {} → {}", d["file"].as_str().unwrap_or("?"), d["detection"].as_str().unwrap_or("?"));
                        }
                    }
                    if let Some(t) = s["eicar_test"].as_object() {
                        println!("\n  EICAR test: {}", serde_json::to_string_pretty(t).unwrap_or_default());
                    }
                }
                "clean" => {
                    let dev = arg_flag(&args, "--device").unwrap_or_else(|| bail_flag("--device"));
                    let c = shield::clean(&dev)?;
                    println!("\n🧹 CLEAN: removed {} · freed {} MB (locked skipped: {})",
                        c["removed_entries"].as_i64().unwrap_or(0),
                        c["freed_mb"].as_f64().unwrap_or(0.0),
                        c["locked_files_skipped"].as_i64().unwrap_or(0));
                }
                "report" => {
                    let out = arg_flag(&args, "--out").unwrap_or_else(|| "shield_report.html".into());
                    let p = shield::report(&out)?;
                    println!("🛡️ Ripoti: {p}");
                }
                "guard" => match args.get(2).map(|s| s.as_str()).unwrap_or("status") {
                    "baseline" => {
                        let b = shield::guard_baseline()?;
                        println!("🛡️ GUARD BASELINE imehifadhiwa ({}): hosts sha256 {}…, startup {}",
                            b["ts"].as_str().unwrap_or("?"),
                            &b["hosts_sha256"].as_str().unwrap_or("?")[..16.min(b["hosts_sha256"].as_str().unwrap_or("?").len())],
                            b["startup"].as_array().map(|a| a.len()).unwrap_or(0));
                    }
                    "check" => {
                        let c = shield::guard_check()?;
                        println!("\n🛡️ GUARD CHECK — {}", c["verdict"].as_str().unwrap_or("?"));
                        if let Some(alerts) = c["alerts"].as_array() {
                            for a in alerts { println!("  {}", a.as_str().unwrap_or("")); }
                        }
                    }
                    "install" => println!("{}", shield::guard_install()?),
                    "status" => println!("{}", serde_json::to_string_pretty(&shield::guard_status())?),
                    other => bail!("shield guard '{other}': baseline | check | install | status"),
                },
                other => shield::bail_help()?,
            }
            Ok(())
        }
        "problems" => {
            // Catalog ya matatizo YOTE: computer + email + social + networking + printers + activation + simu zote
            let mut total = 0usize;
            for f in ["software_problems.json", "more_problems.json", "phone_problems.json"] {
                let data = data_root().join(format!("mobile/{f}"));
                let Ok(txt) = std::fs::read_to_string(&data) else { continue };
                let Ok(v) = serde_json::from_str::<serde_json::Value>(&txt) else { continue };
                let empty2 = vec![];
                for cat in v["categories"].as_array().unwrap_or(&empty2) {
                    let n = cat["problems"].as_array().map(|a| a.len()).unwrap_or(0);
                    total += n;
                    println!("\n{} {} ({} matatizo):", cat["icon"].as_str().unwrap_or("•"), cat["name_sw"].as_str().unwrap_or("?"), n);
                    for (i, p) in cat["problems"].as_array().unwrap_or(&vec![]).iter().enumerate() {
                        println!("  {:>2}. {}", i + 1, p["p"].as_str().unwrap_or("?"));
                    }
                }
            }
            println!("\nJUMLA: {total} matatizo. Suluhisho la moja: fundi-mobile problem <category> <namba>");
            Ok(())
        }
        "problem" => {
            // Suluhisho la tatizo moja — INATAFUTA kwenye faili ZOTE (computer + networking + simu)
            let cat_id = args.get(1).cloned().unwrap_or_else(|| bail_flag("<category>"));
            let idx: usize = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);
            if idx == 0 {
                bail!("Toa namba: fundi-mobile problem <category> <namba> (tazama: fundi-mobile problems)");
            }
            let mut found: Option<serde_json::Value> = None;
            for f in ["software_problems.json", "more_problems.json", "phone_problems.json"] {
                let data = data_root().join(format!("mobile/{f}"));
                let Ok(txt) = std::fs::read_to_string(&data) else { continue };
                let Ok(v) = serde_json::from_str::<serde_json::Value>(&txt) else { continue };
                let empty = vec![];
                if let Some(cat) = v["categories"].as_array().unwrap_or(&empty)
                    .iter().find(|c| c["id"].as_str() == Some(cat_id.as_str()))
                {
                    found = Some(cat.clone());
                    break;
                }
            }
            let Some(cat) = found else {
                bail!("Category '{cat_id}' haipo (tazama: fundi-mobile problems)");
            };
            let probs = cat["problems"].as_array().cloned().unwrap_or_default();
            let p = probs.get(idx - 1)
                .ok_or_else(|| anyhow::anyhow!("Namba {idx} haipo (1-{})", probs.len()))?;
            println!("\n{} — {}", cat["name_sw"].as_str().unwrap_or("?"), p["p"].as_str().unwrap_or("?"));
            println!("Muda: dakika {}\n\nSULUHISHO:", p["minutes"].as_i64().unwrap_or(0));
            println!("{}", p["s"].as_str().unwrap_or("?"));
            Ok(())
        }
        "gov" => {
            // GOV AGENTIC — AI inajaza fomu za serikali; wewe KUJIBU tu
            let sub = args.get(1).map(|s| s.as_str()).unwrap_or("systems");
            match sub {
                "systems" => println!("{}", govagent::systems()),
                "intake" => {
                    let sys = args.get(2).map(|s| s.as_str()).unwrap_or_else(|| bail_flag("<system_id> (tazama: gov systems)"));
                    govagent::intake_interactive(sys)?;
                }
                "fill" => {
                    // fill --system nida --answers "jina_kamili=Juma;tarehe_kuzaliwa=1990-01-01;..."
                    let sys = arg_flag(&args, "--system").unwrap_or_else(|| bail_flag("--system"));
                    let raw = arg_flag(&args, "--answers").unwrap_or_default();
                    let mut answers = std::collections::HashMap::new();
                    for pair in raw.split(';') {
                        if let Some((k, v)) = pair.split_once('=') {
                            answers.insert(k.trim().to_string(), v.trim().to_string());
                        }
                    }
                    govagent::fill_and_print(&sys, &answers, true)?;
                }
                "search" => {
                    let q = args.get(2).cloned().unwrap_or_else(|| "NIDA Tanzania".into());
                    match govagent::searxng_search(&q) {
                        Ok(out) => println!("{out}"),
                        Err(e) => eprintln!("❌ {e}"),
                    }
                }
                "speak" => {
                    let t = args.get(2).cloned().unwrap_or_else(|| "Fundi agent hapa".into());
                    govagent::speak_sw(&t)?;
                    println!("🔊 Sauti imetumwa");
                }
                "jamii" => {
                    match (args.get(2).map(|s| s.as_str()), args.get(3).and_then(|s| s.parse::<usize>().ok())) {
                        (None, _) => println!("{}", govagent::jamii_list()),
                        (Some(cat), Some(n)) => {
                            let out = govagent::jamii_show(cat, n)?;
                            println!("{out}");
                        }
                        (Some(_), None) => bail!("Toa namba: fundi-mobile gov jamii <category> <namba>"),
                    }
                }
                "letter" => {
                    let office = arg_flag(&args, "--to").unwrap_or_else(|| bail_flag("--to"));
                    let subject = arg_flag(&args, "--subject").unwrap_or_else(|| bail_flag("--subject"));
                    let body = arg_flag(&args, "--body").unwrap_or_else(|| bail_flag("--body"));
                    let customer = arg_flag(&args, "--customer").unwrap_or_else(|| "Mteja".into());
                    let out = govagent::letter(&office, &subject, &body, &customer)?;
                    println!("✉️ Barua print-ready: {out}");
                }
                other => bail!("gov '{other}': systems | intake <sys> | fill --system S --answers K=V;.. | search Q | speak T | jamii [cat n] | letter ..."),
            }
            Ok(())
        }
        "osinstall" => {
            // REMOTE OS INSTALL — app moja ya kwake (mteja ↔ mtaalamu)
            let sub = args.get(1).map(|s| s.as_str()).unwrap_or("request");
            match sub {
                "request" => {
                    let r = osremote::request(
                        &arg_flag(&args, "--customer").unwrap_or_else(|| bail_flag("--customer")),
                        arg_flag(&args, "--company").as_deref(),
                        &arg_flag(&args, "--pc").unwrap_or_else(|| bail_flag("--pc (hostname ya mteja)")),
                        &arg_flag(&args, "--os").unwrap_or_else(|| "windows11".into()),
                        &arg_flag(&args, "--bundle").unwrap_or_else(|| "home".into()),
                    )?;
                    println!("\n💿 REMOTE OS INSTALL — session imeundwa");
                    println!("  Code   : {}", r["code"].as_str().unwrap_or("?"));
                    println!("  {}", r["message"].as_str().unwrap_or(""));
                    println!("\n  Mtaalamu: fundi-deploy dashboard (/ui) → IDHINISHA → agent inasakinisha OS + apps zote.");
                    println!("  Mteja: fundi-mobile osinstall status --code {}", r["code"].as_str().unwrap_or(""));
                }
                "status" => {
                    let code = arg_flag(&args, "--code").unwrap_or_else(|| bail_flag("--code"));
                    let v = osremote::status(&code)?;
                    println!("{}", osremote::print_status(&v));
                }
                "watch" => {
                    let code = arg_flag(&args, "--code").unwrap_or_else(|| bail_flag("--code"));
                    let max = arg_flag(&args, "--max-secs").and_then(|s| s.parse().ok()).unwrap_or(600);
                    let final_st = osremote::watch(&code, max)?;
                    println!("\n▶ MWISHO: {final_st}");
                }
                "bundles" => {
                    let v = osremote::bundles()?;
                    for b in v["bundles"].as_array().unwrap_or(&vec![]) {
                        println!("  {:<12} {} ({} apps) — {}", b["id"].as_str().unwrap_or("?"), b["name_sw"].as_str().unwrap_or("?"), b["apps"].as_i64().unwrap_or(0), b["description"].as_str().unwrap_or(""));
                    }
                }
                other => bail!("osinstall '{other}': request | status | watch | bundles"),
            }
            Ok(())
        }
        "email" => {
            // EMAIL AGENTIC — providers zote, setup agentic + sauti, password, matatizo yote
            let sub = args.get(1).map(|s| s.as_str()).unwrap_or("list");
            match sub {
                "list" | "providers" => println!("{}", emailagent::list()),
                "check" => {
                    let p = args.get(2).map(|s| s.as_str()).unwrap_or_else(|| bail_flag("<provider> (tazama: email list)"));
                    println!("{}", emailagent::check(p)?);
                }
                "setup" => {
                    let p = args.get(2).map(|s| s.as_str()).unwrap_or_else(|| bail_flag("<provider> (tazama: email list)"));
                    println!("{}", emailagent::setup(p)?);
                }
                "password" => {
                    let p = args.get(2).map(|s| s.as_str()).unwrap_or("gmail");
                    println!("{}", emailagent::password(p)?);
                }
                "problems" => println!("{}", emailagent::problems()),
                "json" => println!("{}", serde_json::to_string_pretty(&emailagent::providers_json())?),
                other => bail!("email '{other}': list | check <provider> | setup <provider> | password <provider> | problems | json"),
            }
            Ok(())
        }
        "license" => {
            // ACTIVATION ASSISTANT — njia RASMI tu (massgrave/KMS cracks ZIMEKATALIWA)
            let sub = args.get(1).map(|s| s.as_str()).unwrap_or("status");
            match sub {
                "status" => println!("{}", licenses::status()?),
                "office-status" => println!("{}", licenses::office_status()?),
                "activate" => println!("{}", licenses::activate()?),
                "activate-office" => println!("{}", licenses::activate_office()?),
                "genuine" => println!("{}", licenses::genuine()),
                other => bail!("license '{other}': status | office-status | activate | activate-office | genuine"),
            }
            Ok(())
        }
        "drivers" => {
            // DRIVER CENTER — enum halisi + scan halisi + update rasmi
            let sub = args.get(1).map(|s| s.as_str()).unwrap_or("update");
            match sub {
                "list" => println!("{}", drivers::list()?),
                "scan" => println!("{}", drivers::scan()?),
                "update" => println!("{}", drivers::update()),
                other => bail!("drivers '{other}': list | scan | update"),
            }
            Ok(())
        }
        "errors" => {
            // ERROR CODES — lookup + agentic solve (vision halisi kulingana na code)
            let sub = args.get(1).map(|s| s.as_str()).unwrap_or("list");
            match sub {
                "list" => println!("{}", errcodes::list(args.get(2).map(|s| s.as_str()))),
                "solve" => {
                    let code = args.get(2).map(|s| s.as_str()).unwrap_or("");
                    match errcodes::solve(code) {
                        Ok(out) => println!("{out}"),
                        Err(e) => eprintln!("❌ {e}"),
                    }
                }
                other => errcodes::bail_help()?,
            }
            Ok(())
        }
        "mlinzi" => {
            // MLINZI — kinga: forwarding/scams/links/emails + academy + status
            let sub = args.get(1).map(|s| s.as_str()).unwrap_or("status");
            match sub {
                "status" => println!("{}", mlinzi::status()?),
                "forward" => println!("{}", mlinzi::forward_check()),
                "forward-adb" => println!("{}", mlinzi::forward_check_adb()?),
                "forward-stop" => {
                    let yes = args.iter().any(|a| a == "--yes");
                    println!("{}", mlinzi::forward_stop(yes)?);
                }
                "sms" => {
                    let msg = arg_flag(&args, "--text").or_else(|| args.get(2..).map(|r| r.join(" "))).unwrap_or_default();
                    if msg.trim().is_empty() {
                        bail_flag("--text \"ujumbe kamili\" (au: mlinzi sms <ujumbe>...)");
                    }
                    println!("{}", mlinzi::sms(&msg));
                }
                "link" => {
                    let url = args.get(2).map(|s| s.as_str()).unwrap_or_else(|| bail_flag("<url>"));
                    println!("{}", mlinzi::link(url));
                }
                "email" => println!("{}", mlinzi::email()),
                "academy" => println!("{}", mlinzi::academy(args.get(2).and_then(|s| s.parse().ok()))),
                "json" => println!("{}", serde_json::to_string_pretty(&mlinzi::summary_json())?),
                other => bail!("mlinzi '{other}': status | forward | forward-adb | forward-stop [--yes] | sms --text \"...\" | link <url> | email | academy [n] | json"),
            }
            Ok(())
        }
        "datarec" => {
            // DATA RECOVERY — deep-scan halisi + plan + recycle + photorec + phone
            let sub = args.get(1).map(|s| s.as_str()).unwrap_or("plan");
            match sub {
                "scan" => println!("{}", datarec::scan()?),
                "plan" => println!("{}", datarec::plan()),
                "recycle" => {
                    let action = args.get(2).map(|s| s.as_str());
                    println!("{}", datarec::recycle(action)?);
                }
                "photorec" => println!("{}", datarec::photorec()),
                "phone" => println!("{}", datarec::phone()?),
                other => bail!("datarec '{other}': scan | plan | recycle [restore] | photorec | phone"),
            }
            Ok(())
        }
        "scribe" => {
            // AI SCRIBE — notas za mikutano/madarasa/hospitali (inasikiliza, inapanga, inakamilisha)
            let sub = args.get(1).map(|s| s.as_str()).unwrap_or("sessions");
            match sub {
                "start" => {
                    let t = args.get(2).map(|s| s.as_str()).unwrap_or_else(|| bail_flag("<meeting|class|hospital|interview|custom>"));
                    println!("{}", scribe::start(t)?);
                }
                "live" => {
                    let id = args.get(2).map(|s| s.as_str()).unwrap_or_else(|| bail_flag("<session_id>"));
                    println!("{}", scribe::live(id)?);
                }
                "listen" => {
                    let id = args.get(2).map(|s| s.as_str()).unwrap_or_else(|| bail_flag("<session_id>"));
                    let secs = arg_flag(&args, "--secs").and_then(|s| s.parse().ok()).unwrap_or(30);
                    println!("{}", scribe::listen(id, secs)?);
                }
                "add" => {
                    let id = arg_flag(&args, "--id").unwrap_or_else(|| bail_flag("--id"));
                    let kind = arg_flag(&args, "--kind").unwrap_or_else(|| "note".into());
                    let text = arg_flag(&args, "--text").unwrap_or_else(|| bail_flag("--text"));
                    println!("{}", scribe::add(&id, &kind, &text)?);
                }
                "finish" => {
                    let id = args.get(2).map(|s| s.as_str()).unwrap_or_else(|| bail_flag("<session_id>"));
                    let speak = args.iter().any(|a| a == "--speak");
                    println!("{}", scribe::finish(id, speak)?);
                }
                "speak" => {
                    // kusoma minuta iliyokamilika (HTML ya mwisho) — rehash ya decisions
                    let id = args.get(2).map(|s| s.as_str()).unwrap_or_else(|| bail_flag("<session_id>"));
                    let v = scribe::finish(id, true)?;
                    println!("{v}");
                }
                "sessions" => println!("{}", scribe::sessions()),
                other => bail!("scribe '{other}': start <type> | live <id> | listen <id> [--secs N] | add --id X --kind note --text \"...\" | finish <id> [--speak] | speak <id> | sessions"),
            }
            Ok(())
        }
        "license" => {
            // ACTIVATION ASSISTANT — njia RASMI tu (massgrave/KMS cracks ZIMEKATALIWA)
            let sub = args.get(1).map(|s| s.as_str()).unwrap_or("status");
            match sub {
                "status" => println!("{}", licenses::status()?),
                "office-status" => println!("{}", licenses::office_status()?),
                "activate" => println!("{}", licenses::activate()?),
                "activate-office" => println!("{}", licenses::activate_office()?),
                "genuine" => println!("{}", licenses::genuine()),
                "apply" => {
                    let wk = arg_flag(&args, "--win");
                    let ok = arg_flag(&args, "--office");
                    let yes = args.iter().any(|a| a == "--yes");
                    println!("{}", licenses::apply(wk.as_deref(), ok.as_deref(), yes)?);
                }
                "assist" => licenses::assist()?,
                other => bail!("license '{other}': status | office-status | activate | activate-office | genuine | apply --win KEY --office KEY [--yes] | assist"),
            }
            Ok(())
        }
        "reset" => {
            // PASSWORD RESET AGENTIC — consent LAZIMA (HITL), njia rasmi tu
            let sub = args.get(1).map(|s| s.as_str()).unwrap_or("run");
            match sub {
                "run" => {
                    let req = resetagent::ResetRequest {
                        customer: arg_flag(&args, "--customer").unwrap_or_else(|| "Mteja".into()),
                        target: arg_flag(&args, "--target").unwrap_or_else(|| bail_flag("--target (windows_local|windows_ms|android|email|social)")),
                        brand: arg_flag(&args, "--brand").unwrap_or_default(),
                        account: arg_flag(&args, "--account").unwrap_or_else(|| bail_flag("--account (user/email/IMEI)")),
                    };
                    let job = resetagent::run(&req)?;
                    println!("\n▶ RESET AGENTIC — {}", job.id);
                    for f in &job.frames {
                        println!("  [{}] {}: {}", f["code"].as_str().unwrap_or("?"), f["agent"].as_str().unwrap_or("?"), f["sw"].as_str().unwrap_or(""));
                    }
                    println!("\nHali: {} — {}", job.status, job.summary_sw);
                }
                "jobs" => {
                    for j in resetagent::list_jobs() {
                        println!("{} | {} | {} | {}", j.id, j.target, j.account, j.status);
                    }
                }
                other => bail!("reset '{other}': run | jobs"),
            }
            Ok(())
        }
        "backup-verify" => {
            let p = arg_flag(&args, "--file").unwrap_or_else(|| bail_flag("--file"));
            println!("{}", serde_json::to_string_pretty(&restore::verify(&p)?)?);
            Ok(())
        }
        "backup-list" => {
            let root = arg_flag(&args, "--root").unwrap_or_else(|| data_root().display().to_string());
            println!("{}", serde_json::to_string_pretty(&restore::list(&root))?);
            Ok(())
        }
        "restore-plan" => {
            let f = arg_flag(&args, "--file").unwrap_or_else(|| bail_flag("--file"));
            let dest = arg_flag(&args, "--dest").unwrap_or_else(|| "./restored".into());
            println!("{}", serde_json::to_string_pretty(&restore::plan(&f, &dest)?)?);
            Ok(())
        }
        "restore-run" => {
            let f = arg_flag(&args, "--file").unwrap_or_else(|| bail_flag("--file"));
            let dest = arg_flag(&args, "--dest").unwrap_or_else(|| "./restored".into());
            let dev = arg_flag(&args, "--device").unwrap_or_else(|| bail_flag("--device (kwa consent restore_data)"));
            println!("{}", serde_json::to_string_pretty(&restore::execute(&f, &dest, &dev)?)?);
            Ok(())
        }
        "netagentic" => {
            // TELECOM/NETWORK AGENTIC — agents 10 + netdiag halisi mbili
            let sub = args.get(1).map(|s| s.as_str()).unwrap_or("run");
            match sub {
                "run" => {
                    let req = netagentic::NetRequest {
                        customer: arg_flag(&args, "--customer").unwrap_or_else(|| "Mteja".into()),
                        problem: arg_flag(&args, "--problem").unwrap_or_else(|| "internet haifanyi kazi".into()),
                    };
                    let job = netagentic::run(&req)?;
                    println!("\n▶ NET AGENTIC — {}", job.id);
                    for f in &job.frames {
                        println!("  [{}] {}: {}",
                            f["code"].as_str().unwrap_or("?"),
                            f["agent"].as_str().unwrap_or("?"),
                            f["sw"].as_str().unwrap_or(""));
                    }
                    println!("\nHali: {} — {}", job.status, job.summary_sw);
                    println!("Kitabu: fundi-mobile net-book {} out.html", job.id);
                }
                "jobs" => {
                    for j in netagentic::list_jobs() {
                        println!("{} | {} | {} | OK:{} FAILED:{}",
                            j.id, j.customer, j.status,
                            j.layers_ok.len(), j.layers_failed.len());
                    }
                }
                other => bail!("netagentic '{other}': run | jobs"),
            }
            Ok(())
        }
        "net-book" => {
            let id = args.get(1).cloned().unwrap_or_default();
            let out = args.get(2).cloned().unwrap_or_else(|| "net_book.html".into());
            let job = netagentic::list_jobs()
                .into_iter()
                .find(|j| j.id == id)
                .ok_or_else(|| anyhow::anyhow!("NET job '{id}' haipo"))?;
            std::fs::write(&out, netagentic::book_html(&job))?;
            println!("📕 Kitabu cha NET: {out}");
            Ok(())
        }
        "netdiag" => {
            // Diagnostics halisi L1-L7 (inachukua sekunde ~10)
            println!("⏳ Inachunguza mtandao (L1→L7)…");
            let d = netdiag::diagnose_blocking();
            println!("\n{}", d["overall"].as_str().unwrap_or("?"));
            for (key, label) in [
                ("l1", "L1 Physical"),
                ("l2", "L2 Data Link"),
                ("l3", "L3 Network"),
                ("l4", "L4 Transport"),
                ("l7", "L7 Application"),
            ] {
                println!("  {label}: {}", serde_json::to_string(&d[key]).unwrap_or_default());
            }
            if let Some(recs) = d["recommendations"].as_array() {
                if !recs.is_empty() {
                    println!("\n💡 Mapendekezo:");
                    for r in recs {
                        println!("  {}", r.as_str().unwrap_or(""));
                    }
                }
            }
            Ok(())
        }
        "pro-run" => {
            // FUNDI PRO AGENTIC — agents 10 zinaendesha session ya huduma ya ofisi
            let req = proagentic::ProRequest {
                customer: arg_flag(&args, "--customer").unwrap_or_else(|| "Mteja".into()),
                phone: arg_flag(&args, "--phone").unwrap_or_default(),
                service_id: arg_flag(&args, "--service").unwrap_or_else(|| bail_flag("--service")),
                problem: arg_flag(&args, "--problem").unwrap_or_else(|| "tatizo haijaelezwa".into()),
                subs: arg_flag(&args, "--subs")
                    .map(|s| s.split(',').map(|x| x.trim().to_string()).filter(|x| !x.is_empty()).collect())
                    .unwrap_or_default(),
            };
            let job = proagentic::run(&req)?;
            println!("\n▶ FUNDI PRO AGENTIC — {}", job.id);
            for f in &job.frames {
                println!("  [{}] {}: {}",
                    f["code"].as_str().unwrap_or("?"),
                    f["agent"].as_str().unwrap_or("?"),
                    f["sw"].as_str().unwrap_or(""));
            }
            println!("\nHali: {} — {}", job.status, job.summary_sw);
            println!("Kitabu: fundi-mobile pro-book {} out.html", job.id);
            Ok(())
        }
        "pro-book" => {
            let id = args.get(1).cloned().unwrap_or_default();
            let out = args.get(2).cloned().unwrap_or_else(|| "pro_book.html".into());
            let jobs = proagentic::list_jobs();
            let job = jobs
                .iter()
                .find(|j| j.id == id)
                .ok_or_else(|| anyhow::anyhow!("PRO job '{id}' haipo"))?;
            std::fs::write(&out, proagentic::book_html(job))?;
            println!("📕 Kitabu cha PRO: {out}");
            Ok(())
        }
        "pro-dashboard" => {
            println!("{}", serde_json::to_string_pretty(&proagentic::dashboard())?);
            Ok(())
        }
        "recover" => {
            // ANDROID ACCESS RECOVERY — zana 11 kwa 1 (kwa MMILIKI aliyeuthibitishwa)
            // Consent LAZIMA: fundi-mobile consent add --service android_access_recovery ...
            let tool = args.get(1).map(|s| s.as_str()).unwrap_or("list");
            let imei = arg_flag(&args, "--imei").unwrap_or_default();
            let r: Result<String> = match tool {
                "list" | "tools" => Ok(recover::list()),
                "preflight" => recover::preflight(&imei),
                "ownership" => recover::ownership(&imei, arg_flag(&args, "--id").as_deref()),
                "backup-first" => recover::backup_first(
                    &imei,
                    &arg_flag(&args, "--out").unwrap_or_else(|| format!("backup_{}.ab", &imei[..6.min(imei.len())])),
                ),
                "google-remote" => recover::google_remote(&imei),
                "samsung-remote" => recover::samsung_remote(&imei),
                "xiaomi-remote" => recover::xiaomi_remote(&imei),
                "huawei-remote" => recover::huawei_remote(&imei),
                "owner-adb" => recover::owner_adb(&imei),
                "recovery-reset" => recover::recovery_reset(&imei),
                "frp-aftermath" => recover::frp_aftermath(&imei),
                "report" => recover::report(
                    &imei,
                    &arg_flag(&args, "--out").unwrap_or_else(|| format!("recovery_report_{}.html", &imei[..6.min(imei.len())])),
                ),
                other => bail!("recover '{other}': list | preflight | ownership | backup-first | \
                    google-remote | samsung-remote | xiaomi-remote | huawei-remote | owner-adb | \
                    recovery-reset | frp-aftermath | report"),
            };
            match r {
                Ok(msg) => println!("{msg}"),
                Err(e) => eprintln!("❌ {e}"),
            }
            Ok(())
        }
        "check" => {
            let adb = devices::adb_devices().is_ok();
            let fb = devices::fastboot_devices().is_ok();
            println!("ADB:      {}", if adb { "✅" } else { "❌ (choco install adb)" });
            println!("Fastboot: {}", if fb { "✅" } else { "⚠️  haipo" });
            println!("Data:     {} (services/brands/consents)", data_root().display());
            Ok(())
        }
        "menu" => interactive_menu(),
        "help" | _ => {
            help();
            Ok(())
        }
    };
    if let Err(e) = r {
        eprintln!("❌ {e}");
        std::process::exit(1);
    }
    Ok(())
}

fn banner() {
    println!(
        r#"
╔═══════════════════════════════════════════════════╗
║                                                   ║
║   🩺  FUNDI MOBILE v1.0                           ║
║   ────────────────────                            ║
║   Daktari wa Simu — Android · iPhone · BUTTON     ║
║   Agentic: agent inafanya kazi yenyewe            ║
║                                                   ║
║   ⚠️  Kazi ya hatari = consent LAZIMA (HITL)      ║
║                                                   ║
╚═══════════════════════════════════════════════════╗
"#
    );
}

fn bail_flag(flag: &str) -> ! {
    eprintln!("❌ Flag {flag} ni lazima");
    std::process::exit(1);
}

fn collect_items(args: &[String]) -> Result<Vec<payments::InvoiceItem>> {
    let mut items = Vec::new();
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--item" {
            let raw = args.get(i + 1).cloned().unwrap_or_default();
            let parts: Vec<&str> = raw.split(':').collect();
            if parts.len() < 2 {
                bail!("--item desc:qty:price (mfano 'Reset password:1:30000')");
            }
            let qty: u32 = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(1);
            let price: u64 = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);
            items.push(payments::InvoiceItem { desc: parts[0].into(), qty, price });
        }
        if args[i] == "--service" {
            let raw = args.get(i + 1).cloned().unwrap_or_default();
            let mut it = raw.split(':');
            let sid = it.next().unwrap_or_default();
            let qty: u32 = it.next().and_then(|s| s.parse().ok()).unwrap_or(1);
            items.push(payments::item_from_service(sid, qty)?);
        }
        i += 1;
    }
    Ok(items)
}

fn help() {
    println!(
        r#"AMRI:
  catalog                # huduma zote + bei (TZS)
  brands <brand>         # combos + flash tool (samsung/xiaomi/techno/...)
  button <brand>         # simu za button: master reset codes (nokia/itel/...)
  button-codes           # codes zote za button (kwa duka)
  devices                # adb devices + iPhones
  diagnostics            # battery/storage/props (halisi kwa adb)
  consent form|list|add  # HITL: fomu ya kusaini / kusajili ruhusa
  agentic run ...        # agent inaendesha P→I→I→T→V→D (kwa consent)
  agentic jobs           # orodha ya kazi
  dashboard              # muhtasari (jobs + consents + agents)
  book <job> <out.html>  # kitabu kidigitali

MALIPO (FUNDI PAY — makampuni/wateja yoyote):
  pay providers          # mpesa/tigo/airtel/halo/bank/card/cash
  invoice add --customer "Kampuni X" --phone 0712... \
      [--company "X Ltd"] --service reset_password_adb --item "Usafiri:1:10000"
  invoice list | invoice html --invoice INV-... --out inv.html
  pay checkout --invoice INV-... --provider mpesa   # STK push au manual steps
  pay confirm --invoice INV-... --provider tigo --ref TIGO123
  report                 # mapato: leo/mwezi/madeni/kwa provider

  b2b quote --fundi N --company "Kampuni" --contact 0712... \
      --item "Screen replacement:3:60000" [--site "Kariakoo"]
  b2b status --id QT-... --set accepted | b2b to-invoice --id QT-...
  b2b list [quote|invoice] | b2b html --id ... --out doc.html | b2b report
  b2b pdf --id QT-... --out quote.pdf    # PDF rasmi (logo ya FUNDI)

  check                  # angalia tools + data
  menu                   # interactive menu

GOV AGENTIC (AI inajaza fomu za serikali — wewe JIBU tu):
  gov systems                      # mifumo 7 + ada
  gov intake nida                  # INTERACTIVE: agent anauliza, wewe jibu (.​tazama=. search, .save=kamilisha)
  gov fill --system nida --answers "jina_kamili=Juma;tarehe_kuzaliwa=1990-01-01"
  gov search "BRELA fees 2026"     # SearXNG research halisi
  gov speak "Karibu ofisini"        # sauti (TTS ya mfumo)
  gov jamii                        # matatizo ya jamii (kategoria 5)
  gov jamii rights_disputes 2      # msaada + steps + contacts + SAUTI
  gov letter --to "Ofisi ya Wilaya" --subject "Ombi" --body "..." --customer "Juma"

REMOTE OS INSTALL (app moja ya kwake — mteja ↔ mtaalamu):
  osinstall request --customer "Juma" --pc "DESKTOP-AB12" --os windows11 --bundle office
  osinstall bundles                # apps muhimu (home/office/developer/school/cybercafe)
  osinstall status --code RMT-...  # hali ya session
  osinstall watch --code RMT-...   # subiri hadi done (poll kila sekunde 5)
  (Mtaalamu: fundi-deploy /ui → REMOTE OS INSTALL → IDHINISHA)

EMAIL AGENTIC (providers zote + setup + password + matatizo yote):
  email list                       # Gmail/Outlook/Yahoo/Zoho/iCloud/Custom + IMAP/SMTP
  email check gmail                # TCP connect HALISI kwa IMAP/SMTP + banner
  email setup gmail                # agent anaongoza hatua kwa hatua + SAUTI
  email password gmail             # password nguo + policy + reset rasmi + 2FA
  email problems                   # matatizo yote ya email + suluhisho

ACTIVATION (HALALI — slmgr/OSPP rasmi; cracks/massgrave ZIMEKATALIWA):
  license status                   # Windows activation status (slmgr /dli + /xpr)
  license office-status            # Office status (OSPP /dstatus)
  license activate                 # activation RASMI ya Windows (slmgr /ato; admin)
  license activate-office          # activation RASMI ya Office (OSPP /act; admin)
  license genuine                  # mwongozo wa kununua leseni GENUINE
  license apply --win KEY --office KEY [--yes]  # kusakinisha key GENUINE rasmi (/ipk, /inpkey)
  license assist                   # AGENTIC: anakagua + anakuongoza mwanzo hadi mwisho (+sauti)

DRIVERS (enum halisi + scan + update rasmi):
  drivers list                     # pnputil /enum-drivers (drivers zote za tatu)
  drivers scan                     # pnputil /scan-devices (vifaa vipya/vilivyoharibika)
  drivers update                   # mwongozo rasmi: Windows Update + Dell/HP/Lenovo/Intel/NVIDIA

MLINZI WA MTEJA (kinga: utapeli/wizi/phishing — kwa kila mtu):
  mlinzi status                     # hali ya usalama (Defender halisi + checklist)
  mlinzi forward                    # codes RASMI za call forwarding (angalia/zuia)
  mlinzi forward-adb                # agent anaipiga USSD kwenye simu (adb)
  mlinzi forward-stop --yes         # ZIMA forwarding zote (##002#) kwa adb
  mlinzi sms --text "Ushindi..."    # SMS SHIELD: alama za utapeli + sauti
  mlinzi link https://...           # LINK CHECK: phishing halisi (https/punycode/shortener)
  mlinzi email                      # alama za phishing email + hatua
  mlinzi academy [1-20]             # SHULE YA USALAMA (sauti) — wengi hawajui!

DATA RECOVERY (deep-scan + kurudisha data):
  datarec scan                      # disks + afya (SMART) halisi
  datarec plan                      # mpango wa hatua (usifanye writes kwanza!)
  datarec recycle [restore]         # Recycle Bin halisi (list/restore)
  datarec photorec                  # PhotoRec/TestDisk hatua kwa hatua + sauti
  datarec phone                     # adb pull halisi + cloud trash (Drive/iCloud)

AI SCRIBE (mikutano/madarasa/hospitali — inasikiliza, inapanga, inakamilisha):
  scribe start meeting              # intake: mada, wahudhuriaji... (au class/hospital/interview)
  scribe live <id>                  # notes live (.somo .amua .kazi .sahau .tazama .maliza)
  scribe listen <id> --secs 30      # recording halisi + whisper guidance (STT)
  scribe finish <id> [--speak]      # panga: madokezo/maamuzi/kazi → MINUTA print-ready
  scribe sessions                   # historia yote

ERROR CODES (BSOD/Windows/Network/App/Security):
  errors list [bsod|windows|network|app|security]  # orodha + filters
  errors solve 0x0000007B           # agentic solve: vision halisi + causes + fix + HITL

PASSWORD RESET AGENTIC (njia RASMI tu; consent LAZIMA):
  reset run --target windows_local --customer "Juma" --account juma
  reset run --target android --brand samsung --account 354123456789012
  reset run --target email --account juma@gmail.com | --target social --brand facebook
  reset jobs                        # sessions zote

RESTORE YA NGUVU (verify halisi + HITL restore):
  backup-verify --file backup.ab    # magic bytes halisi (.ab/.zip/7z/rar)
  backup-list --root ../data        # tafuta backups zote
  restore-plan --file b.zip --dest ./out   # hatua salama (bila kuandika)
  restore-run --file b --dest ./out --device <id>  # HITL: consent restore_data LAZIMA

FUNDI SHIELD (antivirus yako — scan halisi):
  shield audit                     # AV/firewall/startup/temp halisi + mapendekezo
  shield scan --eicar              # scan + EICAR test ( Defender inaweza kufuta yenyewe = ✅)
  shield clean --device <id>       # futa temp (HITL: consent shield_clean LAZIMA)
  shield report --out shield.html  # ripoti ya usalama (HTML)
  shield guard baseline            # pima hali SALAMA (hosts/startup/temp) + hifadhi
  shield guard check               # linganisha leo na baseline — drift = dalili za malware
  shield guard install             # scheduled task: check kila siku 09:00 (kinga ya kila siku)
  shield guard status              # baseline ipo? lini?

MATATIZO YA SOFTWARE (maintenance/computer/email/social):
  problems                         # kategoria 4 + orodha kamili
  problem software_maintenance 1   # suluhisho la tatizo #1 (hatua kamili)
  problem email_problems 11        # mfano: nimehackiwa (haraka)

TELECOMS & NETWORK AGENTIC (agents 10 + diagnosis halisi mbili):
  netagentic run --customer "Juma" --problem "internet haifanyi kazi"
  netagentic jobs                 # sessions zote
  net-book <NET-...> out.html     # kitabu cha session

TELECOMS & NETWORK (kikokotoo + diagnostics halisi):
  netcalc list                     # formula zote (FSPL, Shannon, Erlang B, subnet...)
  netcalc fspl --d_km 10 --f_mhz 5800      # path loss halisi
  netcalc subnet --ip 192.168.1.100 --cidr 26  # network/broadcast/hosts
  netcalc erlang_b --erlangs 5 --channels 10   # GoS + verdict
  netcalc shannon --bw_mhz 20 --snr_db 20  # capacity Mbps
  netcalc problems                 # matatizo 100+ + suluhisho
  netdiag                          # L1→L7 halisi: interface/ARP/gateway/DNS/TCP/HTTPS

FUNDI PRO AGENTIC (agents 10 zinaendesha huduma za ofisi):
  pro-run --service email_troubleshooting --customer "Juma" --phone 0712... \
          --problem "nimesahau password"
  pro-run --service government_applications --subs tra,brela --customer "Neema"
  pro-run --service data_recovery --case flash_format ...   # HITL: inasimama bila consent
  pro-book <PRO-...> out.html     # kitabu cha session
  pro-dashboard                   # jobs + mapato ya PRO
  pro list / pro show / pro price # catalog + kikokotoo cha bei

FUNDI PRO (huduma za ofisi — bei za TZS + hatua):
  pro list                          # huduma 6 + bei + mode (remote/onsite)
  pro show email_troubleshooting    # matatizo 15 + hatua + tools
  pro show government_applications  # mifumo 7 + bei kila moja
  pro price government_applications tra brela   # kikokotoo: TZS 130,000
  pro price data_recovery dying_disk water_damage

ACCESS RECOVERY (zana 11 kwa 1 — kwa MMILIKI aliyeuthibitishwa, bila exploit):
  recover list                            # orodha ya zana zote 11
  recover preflight --imei 354...         # adb/props/version halisi
  recover ownership --imei ... [--id NIDA]# thibitisha umiliki (consent HITL)
  recover backup-first --imei ...         # backup halisi kabla ya kazi
  recover google-remote --imei ...        # Google Find My Device (rasmi)
  recover samsung-remote --imei ...       # Samsung Find My Mobile (Android 11+)
  recover xiaomi-remote --imei ...        # Mi Cloud (Xiaomi/Redmi/POCO)
  recover huawei-remote --imei ...        # HUAWEI Find Device
  recover owner-adb --imei ...            # zima screen-lock kwa adb ya mmiliki
  recover recovery-reset --imei ...       # hard reset + combo sahihi ya brand
  recover frp-aftermath --imei ...        # mmiliki anaingia account yake (FRP)
  recover report --imei ... --out r.html  # ripoti ya huduma (uthibitisho)"#
    );
}

fn interactive_menu() -> Result<()> {
    loop {
        println!("\n1) Catalog  2) Brands  3) Button codes  4) Devices  5) Diagnostics");
        println!("6) Consent form  7) Dashboard  8) Toka");
        print!("> ");
        use std::io::Write;
        std::io::stdout().flush()?;
        let mut line = String::new();
        std::io::stdin().read_line(&mut line)?;
        match line.trim() {
            "1" => println!("{}", services::catalog_sw()),
            "2" => {
                let d = brands::load()?;
                println!("Smartphones: {:?}", d.smartphones.keys().collect::<Vec<_>>());
                println!("Button phones: {:?}", d.button_phones.keys().collect::<Vec<_>>());
            }
            "3" => {
                let d = brands::load()?;
                for (k, v) in &d.button_phones {
                    println!("{k}: {}", v.master_codes.join(", "));
                }
            }
            "4" => {
                let _ = main_cmd_devices();
            }
            "5" => match procedures::diagnostics() {
                Ok(o) => {
                    println!("{}", o.summary_sw);
                    for s in &o.steps {
                        println!("  {s}");
                    }
                }
                Err(e) => println!("❌ {e}"),
            },
            "6" => println!("{}", consent::form_template()),
            "7" => println!("{}", serde_json::to_string_pretty(&agentic::dashboard())?),
            "8" | "q" | "" => return Ok(()),
            _ => println!("Chagua 1-8"),
        }
    }
}

fn main_cmd_devices() {
    match devices::adb_devices() {
        Ok(l) => {
            for (s, st) in &l {
                println!("  {s}  {st}");
            }
        }
        Err(e) => println!("  {e}"),
    }
}
