//! govagent.rs — GOV AGENTIC: AI inajaza fomu za serikali YENYEWE.
//!
//! Kazi ya fundi/mteja: KUJIBU TU maswali ya agent. Kazi ya agent:
//!   1. INTAKE — maswali ya fields (missing pekee), mteja anajibu kwa mstari mmoja
//!   2. SEARXNG — research halisi ya mifumo (updates, ada mpya, deadlines)
//!   3. FILL — inajaza fomu (print-ready HTML) + orodha ya documents
//!   4. VOICE — mwelekeo wa SAUTI (TTS text halisi kwa engine ya mfumo)
//!   5. JAMII — matatizo halisi ya jamii + steps + contacts + barua rasmi
//!
//! Data: data/mobile/gov_systems.json + community_problems.json
//! Sheria: agent HAISAINI kwa mteja na HAIPASI malipo — inaandaa, mteja anaweka sahihi.

use anyhow::{bail, Result};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;

fn data_path(name: &str) -> Result<PathBuf> {
    let base = std::env::var("FUNDI_DATA").unwrap_or_else(|_| "../data".into());
    let p = PathBuf::from(base).join(format!("mobile/{name}"));
    if !p.exists() {
        bail!("{} haipo", p.display());
    }
    Ok(p)
}

fn load_json(name: &str) -> Result<Value> {
    let txt = std::fs::read_to_string(data_path(name)?)?;
    Ok(serde_json::from_str(&txt)?)
}

// ---------- SYSTEMS ----------

pub fn systems() -> String {
    match load_json("gov_systems.json") {
        Ok(v) => {
            let mut out = String::from("🏛️ MIFUMO YA SERIKALI (GOV AGENTIC):\n\n");
            let fee = |s: &Value| {
                let n = s["fee_tzs"].as_i64().unwrap_or(0).to_string();
                let b = n.as_bytes();
                let mut r = String::new();
                for (i, ch) in b.iter().enumerate() {
                    if i > 0 && (b.len() - i) % 3 == 0 { r.push(','); }
                    r.push(*ch as char);
                }
                r
            };
            for s in v["systems"].as_array().unwrap_or(&vec![]) {
                out.push_str(&format!(
                    "  {:<24} ada TZS {:>9} · dakika {:>3} · {}\n",
                    s["id"].as_str().unwrap_or("?"),
                    fee(s),
                    s["minutes"].as_i64().unwrap_or(0),
                    s["name"].as_str().unwrap_or("?")
                ));
            }
            out.push_str("\nAnza: fundi-mobile gov intake <system_id>\n");
            out
        }
        Err(e) => format!("{e}"),
    }
}

/// INTAKE INTERACTIVE — agent anauliza fields zisizopo; mteja anajibu kwa mstari.
pub fn intake_interactive(system_id: &str) -> Result<()> {
    let v = load_json("gov_systems.json")?;
    let sys = v["systems"]
        .as_array()
        .and_then(|a| a.iter().find(|s| s["id"].as_str() == Some(system_id)))
        .ok_or_else(|| anyhow::anyhow!("Mfumo '{system_id}' haipo (tazama: fundi-mobile gov systems)"))?;

    println!("\n🏛️ {} — GOV AGENTIC intake", sys["name"].as_str().unwrap_or("?"));
    println!("   Jibu kila swali kwa mstari mmoja, kisha bonyeza Enter. ('.tazama' = endesha search, '.save' = kamilisha)\n");

    let mut answers: HashMap<String, String> = HashMap::new();
    let fields = sys["fields"].as_array().cloned().unwrap_or_default();
    for f in &fields {
        let fid = f["id"].as_str().unwrap_or("?").to_string();
        let label = f["label_sw"].as_str().unwrap_or("?").to_string();
        let opts = f["options"].as_array().map(|a| {
            a.iter().enumerate().map(|(i, o)| format!("  {}. {}", i + 1, o.as_str().unwrap_or("")))
                .collect::<Vec<_>>()
                .join("\n")
        });
        loop {
            match &opts {
                Some(o) => println!("{label}:\n{o}"),
                None => println!("{label}:"),
            }
            print!("▶ ");
            std::io::stdout().flush()?;
            let mut line = String::new();
            std::io::stdin().read_line(&mut line)?;
            let ans = line.trim();
            if ans.is_empty() {
                if f["required"].as_bool().unwrap_or(false) {
                    println!("  ⚠️ Ni lazima — jibu tena.");
                    continue;
                }
                break; // optional imepita
            }
            if ans == ".tazama" {
                match searxng_search(&format!("{} Tanzania", sys["name"].as_str().unwrap_or(""))) {
                    Ok(r) => println!("{r}"),
                    Err(e) => println!("  ⚠️ {e}"),
                }
                continue; // swali tena
            }
            if ans == ".save" {
                return fill_and_print(system_id, &answers, true);
            }
            if let Some(o) = &opts {
                // chagua kwa namba au maandishi
                if let Ok(n) = ans.parse::<usize>() {
                    if let Some(choice) = f["options"].as_array().and_then(|a| a.get(n - 1)) {
                        answers.insert(fid, choice.as_str().unwrap_or("").to_string());
                        break;
                    }
                }
                let found = f["options"].as_array().and_then(|a| {
                    a.iter().find(|x| x.as_str().unwrap_or("").to_lowercase().contains(&ans.to_lowercase()))
                });
                if let Some(choice) = found {
                    answers.insert(fid, choice.as_str().unwrap_or("").to_string());
                    break;
                }
                println!("  ⚠️ Chagua namba au andika jina la chaguo.");
                continue;
            }
            answers.insert(fid, ans.to_string());
            break;
        }
    }
    fill_and_print(system_id, &answers, true)
}

// ---------- FILL + PRINT-READY ----------

/// Jaza fomu → HTML print-ready (agent inajaza; mteja anasaini)
pub fn fill_and_print(system_id: &str, answers: &HashMap<String, String>, speak: bool) -> Result<()> {
    let v = load_json("gov_systems.json")?;
    let sys = v["systems"]
        .as_array()
        .and_then(|a| a.iter().find(|s| s["id"].as_str() == Some(system_id)))
        .ok_or_else(|| anyhow::anyhow!("Mfumo '{system_id}' haipo"))?;

    let missing: Vec<String> = sys["fields"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter(|f| f["required"].as_bool().unwrap_or(false))
        .filter(|f| !answers.contains_key(f["id"].as_str().unwrap_or("")))
        .filter_map(|f| f["label_sw"].as_str().map(String::from))
        .collect();

    let mut h = String::from(
        "<!DOCTYPE html><html><head><meta charset='utf-8'><title>FUNDI GOV — Fomu</title>\
         <style>@media print{.noprint{display:none}} body{font-family:Segoe UI,Arial;max-width:800px;margin:32px auto;padding:0 16px;color:#111}\
         h1{color:#0e7490;font-size:22px} table{width:100%;border-collapse:collapse} td,th{border:1px solid #ccc;padding:8px;text-align:left}\
         th{background:#ecfeff;width:38%} .docs{background:#f0fdfa;padding:12px;border-radius:8px} .steps{background:#fffbeb;padding:12px;border-radius:8px}\
         .sign{margin-top:40px;display:flex;justify-content:space-between} .noprint{background:#0f172a;color:#e2e8f0;padding:12px;border-radius:8px;margin-top:16px}</style></head><body>",
    );
    h.push_str(&format!(
        "<h1>🏛️ {} — FOMU ILIYOANDALIWA NA FUNDI AGENT</h1>\
         <p>Ada: TZS {} · Muda: dakika {} · Portal: <b>{}</b></p>",
        sys["name"].as_str().unwrap_or("?"),
        sys["fee_tzs"].as_i64().unwrap_or(0),
        sys["minutes"].as_i64().unwrap_or(0),
        sys["url"].as_str().unwrap_or("?")
    ));
    h.push_str("<table><tr><th>Field</th><th>Jibu</th></tr>");
    for f in sys["fields"].as_array().unwrap_or(&vec![]) {
        let fid = f["id"].as_str().unwrap_or("?");
        let val = answers.get(fid).cloned().unwrap_or_else(|| "— (bado)".into());
        h.push_str(&format!(
            "<tr><th>{}{}</th><td>{}</td></tr>",
            f["label_sw"].as_str().unwrap_or("?"),
            if f["required"].as_bool().unwrap_or(false) { " *" } else { "" },
            html_escape(&val)
        ));
    }
    h.push_str("</table>");

    h.push_str("<h3>📄 Documents za kuchukua</h3><div class='docs'><ul>");
    for d in sys["documents_sw"].as_array().unwrap_or(&vec![]) {
        h.push_str(&format!("<li>{}</li>", d.as_str().unwrap_or("")));
    }
    h.push_str("</ul></div>");

    h.push_str("<h3>🪜 Hatua (agent inakuongoza)</h3><div class='steps'><ol>");
    for s in sys["steps_sw"].as_array().unwrap_or(&vec![]) {
        h.push_str(&format!("<li>{}</li>", s.as_str().unwrap_or("")));
    }
    h.push_str("</ol></div>");

    if !missing.is_empty() {
        h.push_str(&format!(
            "<p style='color:#b91c1c'><b>⚠️ Fields zinazokosekana:</b> {} — jaza kabla ya kuenda ofisi.</p>",
            html_escape(&missing.join(", "))
        ));
    }
    h.push_str("<div class='sign'><div>Sahihi ya mteja: ____________________</div><div>Tarehe: ____________</div><div>Fundi (agent session): ____________________</div></div>");
    h.push_str("</body></html>");

    let out = format!("gov_form_{system_id}.html");
    std::fs::write(&out, h)?;
    println!("\n📄 Fomu print-ready: {out} (fungua kwenye browser → Ctrl+P → PDF/print)");

    // Sauti (TTS ya mfumo)
    if speak {
        let guide = format!(
            "Fomu ya {} iko tayari. Documents: {}. Ada ni shilingi {}. Hatua ya kwanza: {}",
            sys["name"].as_str().unwrap_or("mfumo"),
            sys["documents_sw"].as_array().map(|a| a.len()).unwrap_or(0),
            sys["fee_tzs"].as_i64().unwrap_or(0),
            sys["steps_sw"].as_array().and_then(|a| a.first()).and_then(|s| s.as_str()).unwrap_or("")
        );
        speak_sw(&guide);
        println!("🔊 Sauti: {guide}");
    }
    Ok(())
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

// ---------- SAUTI (TTS halisi ya mfumo) ----------

/// Tamka kwa sauti (Windows: PowerShell SAPI / Linux: espeak au festival)
pub fn speak_sw(text: &str) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        let ps = format!(
            "Add-Type -AssemblyName System.Speech; $s = New-Object System.Speech.Synthesis.SpeechSynthesizer; $s.Speak('{}')",
            text.replace('\'', "''")
        );
        std::process::Command::new("powershell")
            .args(["-NoProfile", "-Command", &ps])
            .spawn()?;
        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        let ok = std::process::Command::new("espeak").arg("-v").arg("sw").arg(text).spawn().is_ok()
            || std::process::Command::new("spd-say").arg(text).spawn().is_ok();
        if !ok {
            println!("(TTS haipatikani — weka espeak: sudo apt install espeak)");
        }
        Ok(())
    }
}

// ---------- SEARXNG (research halisi) ----------

/// Tafuta kwa SearXNG instance (local au remote). Env: FUNDI_SEARXNG_URL (default localhost:8888)
pub fn searxng_search(query: &str) -> Result<String> {
    let base = std::env::var("FUNDI_SEARXNG_URL").unwrap_or_else(|_| "127.0.0.1:8888".into());
    let q = urlencode(query);
    let addr = base.trim_start_matches("http://").to_string();
    let mut stream = std::net::TcpStream::connect(&addr)
        .map_err(|e| anyhow::anyhow!("SearXNG haipatikani ({addr}): {e}\n→ Endesha: docker run -p 8888:8080 searxng/searxng"))?;
    stream.set_read_timeout(Some(std::time::Duration::from_secs(12)))?;
    let req = format!(
        "GET /search?q={q}&format=json&language=sw HTTP/1.1\r\nHost: {addr}\r\nAccept: application/json\r\nConnection: close\r\n\r\n"
    );
    stream.write_all(req.as_bytes())?;
    let mut resp = String::new();
    use std::io::Read as _;
    stream.read_to_string(&mut resp)?;
    let body_start = resp.find("\r\n\r\n").map(|i| i + 4).unwrap_or(0);
    let body = &resp[body_start..];
    let json_start = body.find('{').unwrap_or(0);
    let v: Value = serde_json::from_str(&body[json_start..])
        .map_err(|e| anyhow::anyhow!("SearXNG response si JSON: {e}"))?;

    let mut out = format!("\n🔎 SEARXNG — '{query}' (matokeo {}):\n", v["results"].as_array().map(|a| a.len()).unwrap_or(0));
    for r in v["results"].as_array().unwrap_or(&vec![]).iter().take(5) {
        out.push_str(&format!(
            "  • {} — {}\n    {}\n",
            r["title"].as_str().unwrap_or("?"),
            r["url"].as_str().unwrap_or("?"),
            r["content"].as_str().unwrap_or("").chars().take(140).collect::<String>()
        ));
    }
    Ok(out)
}

fn urlencode(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => out.push(b as char),
            b' ' => out.push_str("%20"),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

// ---------- JAMII (matatizo halisi ya jamii) ----------

pub fn jamii_list() -> String {
    match load_json("community_problems.json") {
        Ok(v) => {
            let mut out = String::from("🤝 MATATIZO YA JAMII (agent anaweza kusaidia):\n\n");
            for c in v["categories"].as_array().unwrap_or(&vec![]) {
                out.push_str(&format!("{} {}:\n", c["icon"].as_str().unwrap_or("•"), c["name_sw"].as_str().unwrap_or("?")));
                for (i, p) in c["problems"].as_array().unwrap_or(&vec![]).iter().enumerate() {
                    out.push_str(&format!("  {}. {}\n", i + 1, p["p"].as_str().unwrap_or("?")));
                }
            }
            out.push_str("\nKina: fundi-mobile gov jamii <category_id> <namba>\n");
            out
        }
        Err(e) => format!("{e}"),
    }
}

pub fn jamii_show(cat_id: &str, n: usize) -> Result<String> {
    let v = load_json("community_problems.json")?;
    let cat = v["categories"]
        .as_array()
        .and_then(|a| a.iter().find(|c| c["id"].as_str() == Some(cat_id)))
        .ok_or_else(|| anyhow::anyhow!("Category '{cat_id}' haipo (tazama: fundi-mobile gov jamii)"))?;
    let probs = cat["problems"].as_array().cloned().unwrap_or_default();
    let p = probs
        .get(n.checked_sub(1).unwrap_or(0))
        .ok_or_else(|| anyhow::anyhow!("Namba {n} haipo (1-{})", probs.len()))?;

    let mut out = format!("\n🤝 {} — {}\n\nMSAADA (hatua halisi):\n{}\n\nANUANI/MANABA:\n",
        cat["name_sw"].as_str().unwrap_or("?"), p["p"].as_str().unwrap_or("?"),
        p["s"].as_str().unwrap_or("?"));
    for c in p["contacts"].as_array().unwrap_or(&vec![]) {
        out.push_str(&format!("  • {}\n", c.as_str().unwrap_or("")));
    }

    // TTS: mwelekeo kwa sauti
    let speak_text = format!(
        "Msaada wa {}. Hatua ya kwanza: {}",
        p["p"].as_str().unwrap_or("tatizo"),
        p["s"].as_str().unwrap_or("").chars().take(200).collect::<String>()
    );
    let _ = speak_sw(&speak_text);
    Ok(out)
}

/// Barua rasmi ya mteja kwenda ofisi (print-ready) — jamii/gov
pub fn letter(to_office: &str, subject: &str, body: &str, customer: &str) -> Result<String> {
    let h = format!(
        "<!DOCTYPE html><html><head><meta charset='utf-8'><title>Barua</title>\
         <style>body{{font-family:'Times New Roman',serif;max-width:750px;margin:40px auto;line-height:1.7;color:#111}}\
         .hdr{{text-align:center;border-bottom:2px solid #333;padding-bottom:8px;margin-bottom:24px}}</style></head><body>\
         <div class='hdr'><b>FUNDI DIGITAL SERVICES</b><br/>Huduma za kidijitali na uraia</div>\
         <p>Tarehe: {}</p>\
         <p>Kwa: <b>{}</b></p>\
         <p><b>YAHUSU: {}</b></p>\
         <p>{}</p>\
         <p style='margin-top:40px'>Heshima,</p>\
         <p>____________________<br/>{}</p>\
         </body></html>",
        chrono::Local::now().format("%d/%m/%Y"),
        html_escape(to_office),
        html_escape(subject),
        html_escape(body).replace(". ", ".<br/><br/>"),
        html_escape(customer)
    );
    let out = "gov_letter.html".to_string();
    std::fs::write(&out, h)?;
    Ok(out)
}
