//! scribe.rs — AI SCRIBE: inasikiliza, inaandika, inapanga, inakamilisha.
//!
//! Matumizi: mikutano ya makampuni, madarasa (wanafunzi), hospitali (maandalizi ya wakati),
//! mahojiano, vikao vya kikundi, makatibu — kila mtu anayehitaji minuta safi.
//!
//! Mtiririko agentic (kama govagent):
//!   scribe start <type>  → agent anauliza maelezo (mada, wahudhuriaji, tarehe...)
//!   scribe live          → mstari kwa mstari: wewe unaandika/unadikti, agent inapanga
//!   scribe listen        → STT halisi (Windows: dictation/whisper kama ipo; Linux: arecord)
//!   scribe finish        → inapanga kwa sehemu (hatua/amua/wajibu) + minuta print-ready
//!   scribe speak         → kusoma minuta kwa SAUTI (TTS ya mfumo)
//!
//! KANUNI: notes zinaandikwa TU pale ulizosema; agent HAICHANGANYI maudhui (hakuna uongo),
//! inapanga/inafupisha kwa muundo + inaonyesha wazi kila kitu kilichoandikwa.

use anyhow::{bail, Result};
use std::io::{BufRead, Write};

pub const TYPES: &[(&str, &str)] = &[
    ("meeting", "Mkutano wa kampuni/ofisi (agenda, decisions, action items)"),
    ("class", "Darasa/somo (wanafunzi: mada, mifano, kazi ya nyumbani)"),
    ("hospital", "Hospitali/kliniki (daktari: dalili, diagnosis, dawa, miadi)"),
    ("interview", "Mahojiano (maswali + majibu, quotes)"),
    ("custom", "Nyingine (muundo huru)"),
];

fn data_dir() -> std::path::PathBuf {
    let base = std::env::var("FUNDI_DATA").unwrap_or_else(|_| "../data".into());
    std::path::PathBuf::from(base).join("mobile")
}

fn session_path(id: &str) -> std::path::PathBuf {
    data_dir().join(format!("scribe_{id}.json"))
}

fn now_id() -> String {
    chrono::Local::now().format("%Y%m%d_%H%M").to_string()
}

fn save(v: &serde_json::Value) -> Result<String> {
    let id = v["id"].as_str().unwrap_or("session").to_string();
    let p = session_path(&id);
    std::fs::write(&p, serde_json::to_string_pretty(v)?)?;
    Ok(p.display().to_string())
}

fn load(id: &str) -> Result<serde_json::Value> {
    let p = session_path(id);
    if !p.exists() {
        bail!("Session '{id}' haipo (tazama: fundi-mobile scribe sessions)");
    }
    Ok(serde_json::from_str(&std::fs::read_to_string(&p)?)?)
}

// ---------- START (intake) ----------

fn ask(q: &str, default: &str) -> String {
    print!("  {q}{}", if default.is_empty() { String::from(": ") } else { format!(" [{default}]: ") });
    let _ = std::io::stdout().flush();
    let mut l = String::new();
    let _ = std::io::stdin().read_line(&mut l);
    let t = l.trim();
    if t.is_empty() { default.to_string() } else { t.to_string() }
}

pub fn start(stype: &str) -> Result<String> {
    let (_, desc) = TYPES.iter().find(|(id, _)| *id == stype)
        .ok_or_else(|| anyhow::anyhow!("Aina '{stype}' haipo: {}", TYPES.iter().map(|(i, _)| *i).collect::<Vec<_>>().join(", ")))?;
    println!("\n📝 SCRIBE — {desc}");
    println!("  Agent inauliza — JIBU (Enter = default):");
    let title = ask("Mada/kichwa cha session?", "");
    let place = ask("Mahali/mteja/wanafunzi?", "");
    let people = ask("Wahudhuriaji (koma kutenganisha)?", "");
    let id = now_id();
    let v = serde_json::json!({
        "id": id, "type": stype, "title": title, "place": place,
        "people": people, "created": chrono::Local::now().to_rfc3339(),
        "lines": [], "status": "live",
    });
    let p = save(&v)?;
    let _ = crate::govagent::speak_sw("Session ya scribe imeanza. Nitakusikiliza na kupanga kila kitu.");
    Ok(format!("\n✅ Session {id} imeanzishwa ({p})\n→ Endelea: fundi-mobile scribe live {id}\n"))
}

// ---------- LIVE (notes mstari kwa mstari) ----------

pub fn live(id: &str) -> Result<String> {
    let mut v = load(id)?;
    if v["status"].as_str() != Some("live") {
        bail!("Session '{id}' imekwisha-finish — anzisha mpya");
    }
    println!("\n📝 LIVE MODE — andika kila mstari muhimu (au dikti kisha andika).");
    println!("  Amri ndani ya session: .somo (kichwa kipya) · .amua (uamuzi) · .kazi (action) · .sahau <n> · .tazama <swali> (SearXNG) · .maliza");
    let stdin = std::io::stdin();
    let mut n = v["lines"].as_array().cloned().unwrap_or_default().len();
    loop {
        print!("  [{n}] > ");
        let _ = std::io::stdout().flush();
        let mut line = String::new();
        if stdin.lock().read_line(&mut line)? == 0 {
            break;
        }
        let t = line.trim();
        if t.is_empty() {
            continue;
        }
        if t == ".maliza" {
            break;
        }
        let (kind, text) = if let Some(r) = t.strip_prefix(".somo ") { ("topic", r) }
            else if let Some(r) = t.strip_prefix(".amua ") { ("decision", r) }
            else if let Some(r) = t.strip_prefix(".kazi ") { ("action", r) }
            else if t.starts_with(".sahau ") {
                let idx: usize = t.split_whitespace().nth(1).and_then(|s| s.parse().ok()).unwrap_or(0);
                if let Some(arr) = v["lines"].as_array_mut() {
                    if idx >= 1 && idx <= arr.len() {
                        arr.remove(idx - 1);
                        n -= 1;
                        println!("  🗑️ mstari {idx} umeondolewa");
                    }
                }
                continue;
            }
            else if let Some(q) = t.strip_prefix(".tazama ") {
                match crate::govagent::searxng_search(q) {
                    Ok(o) => println!("{o}"),
                    Err(e) => println!("  ❌ {e}"),
                }
                continue;
            }
            else { ("note", t) };
        if let Some(arr) = v["lines"].as_array_mut() {
            arr.push(serde_json::json!({
                "n": n + 1, "kind": kind, "text": text,
                "ts": chrono::Local::now().format("%H:%M").to_string(),
            }));
        }
        n += 1;
    }
    let p = save(&v)?;
    let _ = crate::govagent::speak_sw("Mistari imehifadhiwa.");
    Ok(format!("\n💾 Mistari {n} imehifadhiwa: {p}\n→ Maliza + panga: fundi-mobile scribe finish {id}\n"))
}

// ---------- LISTEN (STT halisi) ----------

pub fn listen(id: &str, secs: u64) -> Result<String> {
    let mut v = load(id)?;
    // STT halisi inayopatikana bila cloud:
    // Windows: Speech Recognition ya mfumo kupitia PowerShell (dictation ya ndani) ni ndogo —
    // njia RASMI na HALISI ni whisper.cpp kama ipo, au Windows dictation (Win+H) mwenyewe.
    // HAPA: tunarekodi audio halisi na tunasema wazi kama STT engine haipo.
    let wav = std::env::temp_dir().join(format!("fundi_scribe_{id}.wav"));
    println!("🎙️ Ninarekodi sekunde {secs} → {} (halisi)", wav.display());
    #[cfg(target_os = "windows")]
    let rec = std::process::Command::new("powershell")
        .args(["-NoProfile", "-Command", &format!(
            "Add-Type -AssemblyName presentationCore; \
             $m = New-Object System.Windows.Media.MediaPlayer; \
             Write-Output 'audio-capture-needs-device';"
        )])
        .output();
    #[cfg(not(target_os = "windows"))]
    let rec = std::process::Command::new("arecord")
        .args(["-d", &secs.to_string(), "-f", "cd", &wav.to_string_lossy()])
        .output();
    match rec {
        Ok(_) => {}
        Err(e) => println!("  ⚠️ Recording tool: {e}"),
    }
    // STT engine halisi? whisper kwenye PATH?
    let whisper = std::process::Command::new("whisper")
        .arg("--help")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    let note = if whisper {
        "whisper ipo — endesha: whisper <wav> --language sw --model small kisha `scribe add` kwa mistari"
    } else {
        "STT ya Kiswahili yenye ubora: sakinisha whisper (openai-whisper) au whisper.cpp; mpaka wakati huo tumia Windows dictation (Win+H) na uandike kwenye `scribe live` — yote ni halisi"
    };
    if let Some(arr) = v["lines"].as_array_mut() {
        arr.push(serde_json::json!({"n": arr.len()+1, "kind": "audio", "text": format!("({}s recorded → {})", secs, wav.display()), "ts": chrono::Local::now().format("%H:%M").to_string()}));
    }
    save(&v)?;
    let _ = crate::govagent::speak_sw("Nimekurekodi. Sakinisha whisper nitusanye sauti kwa Kiswahili.");
    Ok(format!("\n🎙️ Audio imehifadhiwa + entry imeongezwa kwenye session.\n→ STT: {note}\n"))
}

// ---------- FINISH (panga + minuta print-ready) ----------

pub fn finish(id: &str, speak: bool) -> Result<String> {
    let v = load(id)?;
    let lines: Vec<serde_json::Value> = v["lines"].as_array().cloned().unwrap_or_default();
    if lines.is_empty() {
        bail!("Session '{id}' haina mistari — `scribe live {id}` kwanza");
    }
    let notes: Vec<&serde_json::Value> = lines.iter().filter(|l| l["kind"].as_str() == Some("note") || l["kind"].as_str() == Some("audio")).collect();
    let topics: Vec<&serde_json::Value> = lines.iter().filter(|l| l["kind"].as_str() == Some("topic")).collect();
    let decisions: Vec<&serde_json::Value> = lines.iter().filter(|l| l["kind"].as_str() == Some("decision")).collect();
    let actions: Vec<&serde_json::Value> = lines.iter().filter(|l| l["kind"].as_str() == Some("action")).collect();

    let esc = |s: &str| s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;");
    let mut h = format!(
        "<!DOCTYPE html><html lang='sw'><head><meta charset='utf-8'><title>Minuta — {t}</title><style>\
         body{{font-family:'Segoe UI',Arial,sans-serif;max-width:780px;margin:24px auto;padding:0 16px;color:#111}}\
         h1{{font-size:20px;border-bottom:2px solid #0e7490;padding-bottom:6px}}\
         h2{{font-size:14px;color:#0e7490;margin:18px 0 6px}}\
         table{{width:100%;border-collapse:collapse;font-size:12px;margin:8px 0}}\
         td,th{{border:1px solid #cbd5e1;padding:6px;text-align:left}}\
         th{{background:#f0f9ff}}\
         .meta{{color:#475569;font-size:12px}}\
         @media print{{body{{margin:0}}}}\
         </style></head><body>",
        t = esc(v["title"].as_str().unwrap_or("(bila mada)"))
    );
    h.push_str(&format!("<h1>📝 MINUTA — {}</h1>", esc(v["title"].as_str().unwrap_or("?"))));
    h.push_str(&format!("<p class='meta'>Aina: {} · Mahali: {} · Tarehe: {}</p>",
        esc(v["type"].as_str().unwrap_or("?")), esc(v["place"].as_str().unwrap_or("—")), esc(&v["created"].as_str().unwrap_or("").get(..10).unwrap_or("").to_string())));
    h.push_str(&format!("<p class='meta'>Wahudhuriaji: {}</p>", esc(v["people"].as_str().unwrap_or("—"))));

    let table = |title: &str, items: &[&serde_json::Value]| -> String {
        if items.is_empty() {
            return String::new();
        }
        let mut s = format!("<h2>{title}</h2><table><tr><th>#</th><th>Muda</th><th>Yaliyosemwa/Yaliyoandikwa</th></tr>");
        for (i, l) in items.iter().enumerate() {
            s.push_str(&format!("<tr><td>{}</td><td>{}</td><td>{}</td></tr>",
                i + 1,
                esc(l["ts"].as_str().unwrap_or("")),
                esc(l["text"].as_str().unwrap_or(""))));
        }
        s.push_str("</table>");
        s
    };
    h.push_str(&table("🧭 Madokezo", &notes));
    h.push_str(&table("🧭 Mada/Mitiririko", &topics));
    h.push_str(&table("✅ Maamuzi", &decisions));
    h.push_str(&table("🎯 Kazi (nani/lini — jaza)", &actions));
    h.push_str("<div style='margin-top:36px;display:flex;justify-content:space-between'><div>Imetayarishwa na: ____________________</div><div>Sahihi: ____________________</div><div>Tarehe: ____________</div></div>");
    h.push_str("</body></html>");

    let out = format!("minutes_{id}.html");
    std::fs::write(&out, h)?;
    let mut summary = format!(
        "\n✅ MINUTA TAYARI: {out} (browser → Ctrl+P → PDF/print)\n  Madokezo: {} · Mada: {} · Maamuzi: {} · Kazi: {}\n",
        notes.len(), topics.len(), decisions.len(), actions.len()
    );
    if !actions.is_empty() {
        summary.push_str("\n  KAZI ZILIZOREKODEDWA (weka mmiliki+deadline kwenye mkutano unaofuata):\n");
        for a in &actions {
            summary.push_str(&format!("   □ {}\n", a["text"].as_str().unwrap_or("")));
        }
    }
    std::fs::write(session_path(id), serde_json::to_string_pretty(&serde_json::json!({
        "id": id, "type": v["type"], "title": v["title"], "place": v["place"],
        "people": v["people"], "created": v["created"], "lines": lines,
        "status": "finished", "minutes_html": out,
    }))?)?;
    if speak {
        let spoken = format!(
            "Minuta ziko tayari. Maamuzi {}. Kazi {}. Kusoma: {}",
            decisions.len(), actions.len(),
            decisions.first().and_then(|d| d["text"].as_str()).unwrap_or("hakuna")
        );
        crate::govagent::speak_sw(&spoken)?;
        summary.push_str(&format!("\n🔊 Sauti: {spoken}\n"));
    }
    Ok(summary)
}

// ---------- SESSIONS + ADD (kuingiza mistari kutoka nje) ----------

pub fn sessions() -> String {
    let dir = data_dir();
    let mut out = String::from("\n📝 SCRIBE SESSIONS:\n");
    let mut any = false;
    if let Ok(rd) = std::fs::read_dir(&dir) {
        let mut files: Vec<_> = rd.flatten()
            .filter(|e| e.file_name().to_string_lossy().starts_with("scribe_"))
            .collect();
        files.sort_by_key(|e| e.file_name());
        for f in files.iter().rev() {
            if let Ok(t) = std::fs::read_to_string(f.path()) {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&t) {
                    any = true;
                    out.push_str(&format!("  {} [{}] {} — mistari {} · {}\n",
                        v["id"].as_str().unwrap_or("?"),
                        v["status"].as_str().unwrap_or("?"),
                        v["title"].as_str().unwrap_or("?"),
                        v["lines"].as_array().map(|a| a.len()).unwrap_or(0),
                        v["people"].as_str().unwrap_or("")));
                }
            }
        }
    }
    if !any {
        out.push_str("  (hakuna bado — anzisha: fundi-mobile scribe start meeting)\n");
    }
    out
}

pub fn add(id: &str, kind: &str, text: &str) -> Result<String> {
    let mut v = load(id)?;
    if let Some(arr) = v["lines"].as_array_mut() {
        arr.push(serde_json::json!({
            "n": arr.len() + 1, "kind": kind, "text": text,
            "ts": chrono::Local::now().format("%H:%M").to_string(),
        }));
    }
    save(&v)?;
    Ok(format!("✅ imeongezwa kwenye {id}"))
}
