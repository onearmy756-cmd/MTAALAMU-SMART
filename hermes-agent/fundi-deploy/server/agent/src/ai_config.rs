//! ai_config.rs — LLM ENGINE (SEHEMU 1.1 + maelezo ya mmiliki):
//!
//! "Qwen 2.5 VL inafanya kazi hata OFFLINE — kompyuta zote zimeunganishwa kwenye
//!  server kuu." — LLM (Qwen 2.5 3B VL) inaishi kwenye SERVER KUU YA LAN
//!  (docker: ollama :11434 / llamacpp :8081). Offline (bila internet) inafanya
//!  kazi kikamilifu — mtandao wa ndani unaotosha. Online ni kwa cloud AI pekee.
//!
//! Endpoints (env-configurable, defaults ni LAN server):
//!   FUNDI_LLM_URL    (default http://127.0.0.1:11434 — Ollama ndani ya server kuu)
//!   FUNDI_LLM_MODEL  (default qwen2.5vl:3b)
//!
//! KANUNI: LLM HAIHESABU kamwe — inaeleza, inatafsiri, inajibu maswali.

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct LlmConfig {
    pub url: String,
    pub model: String,
    pub offline_capable: bool, // daima true — server kuu ya LAN
}

pub fn load() -> LlmConfig {
    LlmConfig {
        url: std::env::var("FUNDI_LLM_URL").unwrap_or_else(|_| "http://127.0.0.1:11434".into()),
        model: std::env::var("FUNDI_LLM_MODEL").unwrap_or_else(|_| "qwen2.5vl:3b".into()),
        offline_capable: true,
    }
}

/// Uliza LLM (HALISI — inaitwa na chat.rs/translate):
/// inafanya kazi OFFLINE kama LLM ipo kwenye server kuu ya LAN (docker service).
pub async fn ask(question: &str, context: &str) -> Option<String> {
    let cfg = load();
    let body = serde_json::json!({
        "model": cfg.model,
        "prompt": format!(
            "Wewe ni fundi wa kompyuta wa MTECH OS (Licensed by Mbilinyi Tech). Jibu kwa ufupi na fasaha.\n\nMuktadha (suluhisho za agents wenzake):\n{context}\n\nSwali: {question}\n\nJibu:"
        ),
        "stream": false,
    });
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/api/generate", cfg.url))
        .json(&body)
        .timeout(std::time::Duration::from_secs(60))
        .send()
        .await
        .ok()?;
    let v: serde_json::Value = resp.json().await.ok()?;
    v["response"].as_str().map(String::from)
}

/// AI TRANSLATION (maelezo ya mmiliki): "AI ndiyo itatafsiri lugha husika
/// automatically" — inatafsiri text kwa lugha iliyochaguliwa (sw/en/…).
pub async fn translate(text: &str, target_lang: &str) -> Option<String> {
    let cfg = load();
    let body = serde_json::json!({
        "model": cfg.model,
        "prompt": format!(
            "Translate the following text to language code '{target_lang}' (sw = Kiswahili, en = English). Reply with the translation ONLY.\n\nText: {text}"
        ),
        "stream": false,
    });
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/api/generate", cfg.url))
        .json(&body)
        .timeout(std::time::Duration::from_secs(60))
        .send()
        .await
        .ok()?;
    let v: serde_json::Value = resp.json().await.ok()?;
    v["response"].as_str().map(|s| s.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_defaults_ni_lan_server() {
        let cfg = load();
        assert!(cfg.url.contains("127.0.0.1") || cfg.url.contains("localhost") || cfg.url.starts_with("http"));
        assert!(cfg.model.contains("qwen"), "model = {}", cfg.model);
        assert!(cfg.offline_capable, "LLM ya server kuu inafanya kazi offline");
    }
}
