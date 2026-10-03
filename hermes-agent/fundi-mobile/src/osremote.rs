//! osremote.rs — REMOTE OS INSTALL client (app moja ya kwake; std-only HTTP).
//!
//! Mteja/kampuni: fundi-mobile osinstall request --customer "Juma" --pc "DESKTOP-AB12" --os windows11 --bundle office
//! Mtaalamu: anaona session kwenye dashboard ya fundi-deploy → IDHINISHA → agent inasakinisha OS + apps.
//!
//! Hakuna deps mpya: HTTP GET/POST halisi za std (TcpStream + HTTP/1.1).

use anyhow::{bail, Result};
use serde_json::{json, Value};
use std::io::{Read, Write};
use std::net::TcpStream;

fn server_base() -> String {
    std::env::var("FUNDI_DEPLOY_URL").unwrap_or_else(|_| "127.0.0.1:8080".into())
}

fn http_request(method: &str, path: &str, body: Option<&Value>) -> Result<Value> {
    let addr = server_base();
    let mut stream = TcpStream::connect(&addr).map_err(|e| {
        anyhow::anyhow!(
            "Fundi Deploy server haipatikani ({addr}): {e}\n→ Endesha server: cd fundi-deploy/server/agent && cargo run"
        )
    })?;
    stream.set_read_timeout(Some(std::time::Duration::from_secs(15)))?;
    let body_s = body.map(|b| b.to_string()).unwrap_or_default();
    let req = format!(
        "{method} {path} HTTP/1.1\r\nHost: {addr}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body_s}",
        body_s.len()
    );
    stream.write_all(req.as_bytes())?;
    let mut resp = String::new();
    stream.read_to_string(&mut resp)?;
    // HTTP response: headers ... \r\n\r\n body
    let body_start = resp.find("\r\n\r\n").map(|i| i + 4).unwrap_or(0);
    let body = &resp[body_start..];
    // JSON inaweza kuwa na chunked — angalizo rahisi: tafuta '{' ya kwanza
    let json_start = body.find('{').unwrap_or(0);
    serde_json::from_str(&body[json_start..])
        .map_err(|e| anyhow::anyhow!("HTTP response si JSON: {e} (server inajibu? {})", &body[..body.len().min(120)]))
}

/// Omba session ya remote OS install
pub fn request(customer: &str, company: Option<&str>, pc: &str, os: &str, bundle: &str) -> Result<Value> {
    let mut b = json!({ "customer": customer, "pc": pc, "os": os, "bundle": bundle });
    if let Some(c) = company {
        b["company"] = json!(c);
    }
    let resp = http_request("POST", "/osinstall/request", Some(&b))?;
    Ok(resp)
}

/// Hali ya session (poll — mteja anajua kila hatua)
pub fn status(code: &str) -> Result<Value> {
    http_request("GET", &format!("/osinstall/status?code={code}"), None)
}

/// Orodha ya bundles (apps muhimu)
pub fn bundles() -> Result<Value> {
    http_request("GET", "/osinstall/bundles", None)
}

pub fn print_status(v: &Value) -> String {
    if !v["found"].as_bool().unwrap_or(false) {
        return format!("❌ {}", v["error"].as_str().unwrap_or("session haipo"));
    }
    let s = &v["session"];
    format!(
        "💿 {} · {} · {} · OS: {} · bundle: {}\n  Hali: {} ({}%) — {}\n",
        s["code"].as_str().unwrap_or("?"),
        s["customer"].as_str().unwrap_or("?"),
        s["pc"].as_str().unwrap_or("?"),
        s["os"].as_str().unwrap_or("?"),
        s["bundle"].as_str().unwrap_or("?"),
        s["status"].as_str().unwrap_or("?"),
        s["progress"].as_i64().unwrap_or(0),
        s["message"].as_str().unwrap_or(""),
    )
}

/// Poll loop: subiri hadi done (kwa mteja)
pub fn watch(code: &str, max_secs: u64) -> Result<String> {
    let start = std::time::Instant::now();
    loop {
        let v = status(code)?;
        println!("{}", print_status(&v));
        let st = v["session"]["status"].as_str().unwrap_or("").to_string();
        if st == "done" || st == "cancelled" || st == "failed" {
            return Ok(st);
        }
        if start.elapsed().as_secs() > max_secs {
            bail!("Timeout ({max_secs}s) — session bado {} (endelea kuwatch: fundi-mobile osinstall status --code {code})", st);
        }
        std::thread::sleep(std::time::Duration::from_secs(5));
    }
}
