//! web.rs — Integration halisi za mtandao (data-driven kutoka data/ai/models.json)
//!
//! Zana:
//!  - DuckDuckGo search (HTML lite — hakuna key)
//!  - SearXNG meta-search (public au self-hosted kwa SEARXNG_URL)
//!  - Ollama chat (cloud kwa OLLAMA_API_KEY au local 127.0.0.1:11434)
//!  - Hugging Face: hub search + Inference chat (HF_TOKEN hiari)
//!
//! Kanuni: LLM haihesabu kamwe (R-1) — inaongea/translate/summarize pekee.
//! Hesabu zote ni za Rust expr engine (skills.rs).

use serde_json::{json, Value};
use std::collections::HashMap;
use std::time::Duration;

/// Client wa HTTP mmoja (timeout 20s, rustls — hakuna CA ya system inahitajika)
fn http() -> reqwest::blocking::Client {
    reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(20))
        .user_agent("Mozilla/5.0 (X11; Linux x86_64) MTAALAMU-SMART/1.1")
        .build()
        .unwrap_or_default()
}

fn env(k: &str) -> Option<String> {
    std::env::var(k).ok().filter(|v| !v.trim().is_empty())
}

// ---------------------------------------------------------------- DuckDuckGo

/// DuckDuckGo HTML — POST inapita bot-check ya GET (hakuna key).
pub fn ddg_search(query: &str, limit: usize) -> Result<Value, String> {
    if query.trim().is_empty() {
        return Err("Tafuta nini? (query tupu)".into());
    }
    let params = [("q", query.to_string()), ("b", "".into())];
    let html = http()
        .post("https://html.duckduckgo.com/html/")
        .header("Accept-Language", "sw,en;q=0.8")
        .form(&params)
        .send()
        .map_err(|e| format!("DDG POST: {}", e))?
        .text()
        .map_err(|e| format!("DDG read: {}", e))?;

    let results = parse_ddg_html(&html, limit);
    if results.is_empty() {
        return Err("DDG: hakuna matokeo (au HTML imebadilika)".into());
    }
    Ok(json!({
        "engine": "duckduckgo",
        "query": query,
        "count": results.len(),
        "results": results,
    }))
}

/// Parser nyepesi wa HTML ya DDG lite (bila crate za HTML — regex ya mikono).
fn parse_ddg_html(html: &str, limit: usize) -> Vec<Value> {
    let mut out = Vec::new();
    // Blocks: <a rel="nofollow" class="result__a" href="...">TITLE</a>
    // Snippets: <a class="result__snippet" ...>SNIPPET</a>
    let mut parts = html.split("class=\"result__a\"").skip(1);
    let mut snippets = html
        .split("class=\"result__snippet\"")
        .skip(1)
        .map(|s| extract_html_text(s))
        .collect::<Vec<_>>()
        .into_iter();
    for (i, chunk) in parts.by_ref().enumerate() {
        if out.len() >= limit {
            break;
        }
        let _ = i;
        // href ya DDG ni redirect: //duckduckgo.com/l/?uddg=<ENCODED>&rut=...
        let href = match chunk.split("href=\"").nth(1) {
            Some(h) => h.split('"').next().unwrap_or("").to_string(),
            None => continue,
        };
        let real_url = decode_ddg_href(&href);
        // title: kata yote hadi kwenye '>' ya tag (attribute href="..." isiingie title)
        let inner = match chunk.split_once('>') {
            Some((_, rest)) => rest,
            None => chunk,
        };
        let title = extract_html_text(inner.split("</a>").next().unwrap_or(""));
        if title.is_empty() {
            continue;
        }
        let snippet = snippets.next().unwrap_or_default();
        out.push(json!({
            "title": title,
            "url": real_url,
            "snippet": snippet.chars().take(300).collect::<String>(),
        }));
    }
    out
}

fn decode_ddg_href(href: &str) -> String {
    if let Some(idx) = href.find("uddg=") {
        let rest = &href[idx + 5..];
        let end = rest.find('&').unwrap_or(rest.len());
        return urldecode(&rest[..end]);
    }
    if href.starts_with("//") {
        format!("https:{}", href)
    } else {
        href.to_string()
    }
}

/// Vuta maandishi safi kutoka HTML (bila tags), decode entities chache muhimu.
fn extract_html_text(s: &str) -> String {
    let mut out = String::new();
    let mut depth = 0usize;
    for c in s.chars() {
        match c {
            '<' => depth += 1,
            '>' => depth = depth.saturating_sub(1),
            _ if depth == 0 => out.push(c),
            _ => {}
        }
    }
    let out = out
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#x27;", "'")
        .replace("&#39;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&nbsp;", " ");
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

// ------------------------------------------------------------------- SearXNG

/// SearXNG meta-search. URL: SEARXNG_URL env au public instances (JSON API).
pub fn searxng_search(query: &str, limit: usize) -> Result<Value, String> {
    if query.trim().is_empty() {
        return Err("Tafuta nini? (query tupu)".into());
    }
    let custom = env("SEARXNG_URL");
    let bases: Vec<String> = match custom {
        Some(b) => vec![b],
        None => vec![
            "https://searx.tiekoetter.com".into(),
            "https://searx.be".into(),
            "https://search.bus-hit.me".into(),
            "https://searxng.site".into(),
            "https://searx.be".into(),
        ],
    };
    let mut last_err = String::from("hakuna instance");
    for base in &bases {
        let base = base.trim_end_matches('/');
        let url = format!(
            "{}/search?q={}&format=json&language=sw",
            base,
            urlencode(query)
        );
        let resp = match http().get(&url).send() {
            Ok(r) => r,
            Err(e) => {
                last_err = format!("{}: {}", base, e);
                continue;
            }
        };
        let status = resp.status().as_u16();
        let body = match resp.text() {
            Ok(b) => b,
            Err(e) => {
                last_err = format!("{}: {}", base, e);
                continue;
            }
        };
        if status != 200 {
            last_err = format!("{}: HTTP {}", base, status);
            continue;
        }
        let v: Value = match serde_json::from_str(&body) {
            Ok(v) => v,
            Err(_) => {
                last_err = format!("{}: si JSON (bot-check)", base);
                continue;
            }
        };
        let mut results = Vec::new();
        for r in v["results"].as_array().cloned().unwrap_or_default() {
            if results.len() >= limit {
                break;
            }
            results.push(json!({
                "title": r["title"].as_str().unwrap_or(""),
                "url": r["url"].as_str().unwrap_or(""),
                "snippet": r["content"].as_str().unwrap_or(""),
                "engine": r["engines"].as_str().unwrap_or(""),
            }));
        }
        if results.is_empty() {
            last_err = format!("{}: matokeo tupu", base);
            continue;
        }
        return Ok(json!({
            "engine": "searxng",
            "base": base,
            "query": query,
            "count": results.len(),
            "results": results,
        }));
    }
    Err(format!("SearXNG zote zimeshindwa: {}", last_err))
}

// -------------------------------------------------------------------- Ollama

#[derive(Debug, Clone)]
pub struct OllamaConfig {
    pub base_url: String,
    pub api_key: Option<String>,
}

/// Soma config kutoka data/ai/models.json (providers) + env keys.
pub fn ollama_config(models_path: &std::path::Path) -> OllamaConfig {
    let mut cfg = OllamaConfig {
        base_url: "https://ollama.com".into(),
        api_key: env("OLLAMA_API_KEY"),
    };
    if let Ok(s) = std::fs::read_to_string(models_path) {
        if let Ok(v) = serde_json::from_str::<Value>(&s) {
            if let Some(p) = v["providers"]["ollama_cloud"]["base_url"].as_str() {
                cfg.base_url = p.to_string();
            }
        }
    }
    // Ollama local ipo? tumia hiyo (offline-first) isipokuwa tunayo key na cloud inahitajika
    if cfg.api_key.is_none() {
        cfg.base_url = "http://127.0.0.1:11434".into();
    }
    cfg
}

/// Chat halisi na Ollama (cloud Bearer key au local). Modeli kutoka models.json.
pub fn ollama_chat(cfg: &OllamaConfig, model: &str, system: &str, user: &str) -> Result<Value, String> {
    let url = format!("{}/api/chat", cfg.base_url.trim_end_matches('/'));
    let mut body = json!({
        "model": model,
        "messages": [
            {"role": "system", "content": system},
            {"role": "user", "content": user}
        ],
        "stream": false,
        "options": {"num_predict": 400}
    });
    if !system.trim().is_empty() {
        body["messages"][0]["content"] = json!(system);
    }
    let mut req = http().post(&url).json(&body);
    if let Some(k) = &cfg.api_key {
        req = req.bearer_auth(k);
    }
    let resp = req.send().map_err(|e| format!("Ollama POST: {}", e))?;
    let status = resp.status().as_u16();
    let txt = resp.text().map_err(|e| format!("Ollama read: {}", e))?;
    let v: Value = serde_json::from_str(&txt)
        .map_err(|e| format!("Ollama JSON (HTTP {}): {} | body: {}", status, e, txt.chars().take(200).collect::<String>()))?;
    if let Some(err) = v["error"].as_str() {
        return Err(format!("Ollama: {}", err));
    }
    let content = v["message"]["content"]
        .as_str()
        .unwrap_or("")
        .to_string();
    if content.is_empty() {
        return Err("Ollama: jibu liko tupu".into());
    }
    Ok(json!({
        "provider": if cfg.api_key.is_some() { "ollama_cloud" } else { "ollama_local" },
        "model": model,
        "content": content,
        "eval_count": v["eval_count"].as_i64().unwrap_or(0),
        "total_duration_ms": (v["total_duration"].as_f64().unwrap_or(0.0) / 1_000_000.0).round() as i64,
    }))
}

/// List ya modeli zilizopo (cloud au local) — kutoka /api/tags
pub fn ollama_tags(cfg: &OllamaConfig) -> Result<Value, String> {
    let url = format!("{}/api/tags", cfg.base_url.trim_end_matches('/'));
    let mut req = http().get(&url);
    if let Some(k) = &cfg.api_key {
        req = req.bearer_auth(k);
    }
    let resp = req.send().map_err(|e| format!("Ollama tags: {}", e))?;
    let v: Value = resp.json().map_err(|e| format!("Ollama tags JSON: {}", e))?;
    let models: Vec<String> = v["models"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|m| m["name"].as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();
    Ok(json!({"base": cfg.base_url, "models": models, "count": models.len()}))
}

// -------------------------------------------------------------- Hugging Face

/// Hugging Face hub search — modeli wazi za bure (hakuna key inahitajika).
pub fn hf_search_models(query: &str, limit: usize) -> Result<Value, String> {
    let url = format!(
        "https://huggingface.co/api/models?search={}&limit={}&sort=downloads&direction=-1",
        urlencode(query),
        limit
    );
    let resp = http()
        .get(&url)
        .send()
        .map_err(|e| format!("HF hub: {}", e))?;
    let v: Value = resp.json().map_err(|e| format!("HF hub JSON: {}", e))?;
    let mut out = Vec::new();
    for m in v.as_array().cloned().unwrap_or_default() {
        out.push(json!({
            "id": m["id"].as_str().unwrap_or(""),
            "downloads": m["downloads"].as_i64().unwrap_or(0),
            "likes": m["likes"].as_i64().unwrap_or(0),
            "pipeline": m["pipeline_tag"].as_str().unwrap_or(""),
        }));
    }
    Ok(json!({"engine": "huggingface_hub", "query": query, "count": out.len(), "models": out}))
}

/// Hugging Face Inference chat (router HF modeli ya conversational).
/// HF_TOKEN hiari kwa quota ya bure; bila token HF inarudisha 401 kwa sasa —
/// mfuatano wa fallback iko kwenye skills.rs (ollama tinyllama kwanza).
pub fn hf_chat(model: &str, user: &str) -> Result<Value, String> {
    let url = "https://router.huggingface.co/v1/chat/completions";
    let mut req = http().post(url).json(&json!({
        "model": model,
        "messages": [{"role": "user", "content": user}],
        "max_tokens": 300
    }));
    if let Some(t) = env("HF_TOKEN") {
        req = req.bearer_auth(t);
    }
    let resp = req.send().map_err(|e| format!("HF chat: {}", e))?;
    let status = resp.status().as_u16();
    let txt = resp.text().map_err(|e| format!("HF chat read: {}", e))?;
    let v: Value = serde_json::from_str(&txt).map_err(|e| format!("HF chat JSON (HTTP {}): {}", status, e))?;
    if let Some(err) = v["error"].as_str() {
        return Err(format!("HF (HTTP {}): {}", status, err));
    }
    let content = v["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("")
        .to_string();
    if content.is_empty() {
        return Err(format!("HF: jibu liko tupu (HTTP {})", status));
    }
    Ok(json!({"provider": "huggingface", "model": model, "content": content}))
}

// ------------------------------------------------------------------ Utilities

pub fn urlencode(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

pub fn urldecode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'%' if i + 2 < b.len() + 1 && i + 2 < b.len() + 1 => {
                let hex = std::str::from_utf8(&b[i + 1..(i + 3).min(b.len())]).unwrap_or("");
                match u8::from_str_radix(hex, 16) {
                    Ok(v) => {
                        out.push(v);
                        i += 3;
                    }
                    Err(_) => {
                        out.push(b[i]);
                        i += 1;
                    }
                }
            }
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            c => {
                out.push(c);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Changanua matokeo ya engines mbili mbili — merge bila duplicate ya URL.
pub fn merge_results(a: Value, b: Value) -> Value {
    let engines = [a["engine"].clone(), b["engine"].clone()];
    let query = a["query"]
        .as_str()
        .map(|s| s.to_string())
        .or_else(|| b["query"].as_str().map(|s| s.to_string()))
        .unwrap_or_default();
    let mut seen: HashMap<String, ()> = HashMap::new();
    let mut merged = Vec::new();
    for src in [a, b] {
        if let Some(arr) = src["results"].as_array() {
            for r in arr {
                let url = r["url"].as_str().unwrap_or("").to_string();
                if url.is_empty() || !seen.contains_key(&url) {
                    seen.insert(url, ());
                    merged.push(r.clone());
                }
            }
        }
    }
    json!({
        "engines": engines,
        "query": query,
        "count": merged.len(),
        "results": merged,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_urlencode() {
        assert_eq!(urlencode("umeme breaker"), "umeme%20breaker");
        assert_eq!(urlencode("ka+fa"), "ka%2Bfa");
    }

    #[test]
    fn test_urldecode() {
        assert_eq!(urldecode("umeme%20breaker"), "umeme breaker");
        assert_eq!(urldecode("a+b"), "a b");
    }

    #[test]
    fn test_decode_ddg_href() {
        assert_eq!(
            decode_ddg_href("//duckduckgo.com/l/?uddg=https%3A%2F%2Fexample.com&rut=abc"),
            "https://example.com"
        );
        assert_eq!(decode_ddg_href("//example.com/x"), "https://example.com/x");
    }

    #[test]
    fn test_extract_html_text() {
        assert_eq!(extract_html_text("<b>Umeme</b> &amp; Maji"), "Umeme & Maji");
    }

    #[test]
    fn test_merge_results() {
        let a = json!({"engine":"ddg","query":"q","results":[{"url":"1","title":"a"}]});
        let b = json!({"engine":"sx","query":"q","results":[{"url":"1","title":"a2"},{"url":"2","title":"b"}]});
        let m = merge_results(a, b);
        assert_eq!(m["count"], 2);
        assert_eq!(m["engines"][0], "ddg");
    }
}
