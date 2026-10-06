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
use sqlx::SqlitePool;

#[derive(Debug, Clone, Serialize)]
pub struct LlmConfig {
    pub url: String,
    pub model: String,
    pub offline_capable: bool, // daima true — server kuu ya LAN
}

/// CUSTOM MODEL / API (maelezo ya mmiliki: "weka sehemu ya kuweka custom model,
/// local model au API"): mteja anaweza kuweka endpoint/model yake — inahifadhiwa
/// kwenye DB (settings table) na INATUMIKA mara moja kwenye chat/translate.
/// Hakuna AI key inayohifadhiwa kwenye DB (inabaki kwenye env ya server).
pub async fn init_tables(db: &SqlitePool) {
    let _ = sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_settings (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            url TEXT NOT NULL,
            model TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )",
    )
    .execute(db)
    .await;
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct CustomAi {
    pub url: String,
    pub model: String,
}

/// Weka custom model/API (OpenAI-compatible / Ollama / llama.cpp / vLLM — URL yoyote).
pub async fn set_custom(db: &SqlitePool, c: &CustomAi) -> Result<(), String> {
    let url = c.url.trim().trim_end_matches('/').to_string();
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err("URL lazima ianze na http:// au https://".into());
    }
    if c.model.trim().is_empty() {
        return Err("jina la model ni lazima".into());
    }
    let now = chrono::Local::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO ai_settings (id, url, model, updated_at) VALUES (1,?,?,?)
         ON CONFLICT(id) DO UPDATE SET url=excluded.url, model=excluded.model, updated_at=excluded.updated_at",
    )
    .bind(&url)
    .bind(c.model.trim())
    .bind(&now)
    .execute(db)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

/// Ondoa custom config — rejea default ya server kuu ya LAN.
pub async fn clear_custom(db: &SqlitePool) -> Result<(), String> {
    sqlx::query("DELETE FROM ai_settings WHERE id = 1")
        .execute(db)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn default_config() -> LlmConfig {
    LlmConfig {
        url: std::env::var("FUNDI_LLM_URL").unwrap_or_else(|_| "http://127.0.0.1:11434".into()),
        model: std::env::var("FUNDI_LLM_MODEL").unwrap_or_else(|_| "qwen2.5vl:3b".into()),
        offline_capable: true,
    }
}

/// Load: custom (DB) inapita env — mteja ameweka model/API yake → inatumika.
pub async fn load_with_db(db: &SqlitePool) -> LlmConfig {
    let row: Option<(String, String)> =
        sqlx::query_as("SELECT url, model FROM ai_settings WHERE id = 1")
            .fetch_optional(db)
            .await
            .unwrap_or(None);
    match row {
        Some((url, model)) => LlmConfig { url, model, offline_capable: true },
        None => default_config(),
    }
}

/// Compatibility: load() bila DB (env default).
pub fn load() -> LlmConfig {
    default_config()
}

/// Uliza LLM (HALISI — inaitwa na chat.rs/translate):
/// inafanya kazi OFFLINE kama LLM ipo kwenye server kuu ya LAN (docker service).
pub async fn ask_db(db: &SqlitePool, question: &str, context: &str) -> Option<String> {
    let cfg = load_with_db(db).await;
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
pub async fn translate_db(db: &SqlitePool, text: &str, target_lang: &str) -> Option<String> {
    let cfg = load_with_db(db).await;
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
        assert!(cfg.offline_capable, "LLM ya server kuu inafanya kazi offline");
    }

    #[tokio::test]
    async fn custom_model_inatumika_na_inaondolewa() {
        let db = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        init_tables(&db).await;
        // default kwanza
        let cfg = load_with_db(&db).await;
        assert!(cfg.url.contains("127.0.0.1"));
        // weka custom (mteja ameweka API yake/model yake)
        set_custom(&db, &CustomAi { url: "http://192.168.1.50:1234/".into(), model: "mymodel-7b".into() }).await.unwrap();
        let cfg = load_with_db(&db).await;
        assert_eq!(cfg.url, "http://192.168.1.50:1234", "trailing slash inaondolewa");
        assert_eq!(cfg.model, "mymodel-7b");
        // URL mbaya inakataliwa
        assert!(set_custom(&db, &CustomAi { url: "ftp://bad".into(), model: "x".into() }).await.is_err());
        assert!(set_custom(&db, &CustomAi { url: "http://ok".into(), model: "  ".into() }).await.is_err());
        // clear → rejea default
        clear_custom(&db).await.unwrap();
        let cfg = load_with_db(&db).await;
        assert!(cfg.url.contains("127.0.0.1"));
    }
}
