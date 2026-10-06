//! chat.rs — CHAT YA AGENT (SEHEMU 3 Hatua 5): andika tatizo/swali → agent inajibu.
//!
//! Mtiririko (hakuna uongo):
//!   1. Offline: recall kutoka Neuralis Brain (kumbukumbu za agents) + jawabu la rules
//!   2. Online: LLM halisi (Ollama/llama.cpp — Qwen 2.5 3B VL) + muktadha wa brain
//!   3. Kila jawabu lina confidence (chini offline, juu online na LLM)
//!
//! LLM HAIHESABU kamwe — inaeleza/onasih; solutions zinatoka brain/knowledge.

use serde::Serialize;
use sqlx::SqlitePool;

#[derive(Debug, Clone, Serialize)]
pub struct ChatReply {
    pub question: String,
    pub answer_sw: String,
    pub source: String,   // brain | llm | offline_rules
    pub confidence: f64,  // 0.0–1.0
    pub references: Vec<String>, // solutions za agents wenzake
}

const OLLAMA_URL: &str = "http://127.0.0.1:11434/api/generate";
const MODEL: &str = "qwen2.5vl:3b";

/// Swali la LLM halisi — LLM inaishi kwenye SERVER KUU YA LAN (docker),
/// kwa hiyo inafanya kazi hata OFFLINE (mtandao wa ndani unaotosha).
/// Online ni kwa cloud AI PEKEE — LLM ya LAN haitegemei internet.
async fn llm_ask(question: &str, context: &str) -> Option<String> {
    let body = serde_json::json!({
        "model": MODEL,
        "prompt": format!("Wewe ni fundi wa kompyuta wa MTECH OS. Jibu kwa Kiswahili kwa ufupi.\n\nMuktadha (suluhisho za agents wenzake):\n{context}\n\nSwali: {question}\n\nJibu:"),
        "stream": false,
    });
    let client = reqwest::Client::new();
    let resp = client
        .post(OLLAMA_URL)
        .json(&body)
        .timeout(std::time::Duration::from_secs(60))
        .send()
        .await
        .ok()?;
    let v: serde_json::Value = resp.json().await.ok()?;
    v["response"].as_str().map(String::from)
}

/// Chat: andika tatizo/swali → jawabu (offline = brain + rules; online = LLM + brain).
pub async fn ask(db: &sqlx::SqlitePool, question: &str, online: bool) -> ChatReply {
    // 1. Neuralis Brain daima (offline + online context)
    let refs = crate::brain::recall(db, question, 3).await;
    let context: String = refs
        .iter()
        .map(|(m, _)| format!("- {} (PC {}): {}", m.problem, m.pc, m.solution))
        .collect::<Vec<_>>()
        .join("\n");

    // 2. LLM ya SERVER KUU YA LAN: inafanya kazi hata OFFLINE (docker service
    //    ya LLM inaishi ndani ya LAN — kompyuta zote zimeunganishwa nayo).
    //    `online` inaongeza tu uwezo wa cloud AI; LLM ya LAN haitegemei internet.
    if let Some(answer) = llm_ask(question, &context).await {
        return ChatReply {
            question: question.into(),
            answer_sw: answer.trim().into(),
            source: if online { "llm".into() } else { "llm_offline_lan".into() },
            confidence: if refs.is_empty() { 0.7 } else { 0.9 },
            references: refs.iter().map(|(m, _)| m.solution.clone()).collect(),
        };
    }
    // LLM ya LAN haipatikani → offline fallback (hakuna uongo)

    // 3. Offline fallback: brain + rules za msingi
    let (answer, conf) = if let Some((m, score)) = refs.first() {
        (
            format!(
                "Suluhisho lililojaribuwa na kufanya kazi (PC {}, conf {:.0}%): {}",
                m.pc,
                score * 100.0,
                m.solution
            ),
            (0.5 + 0.4 * score).clamp(0.0, 1.0),
        )
    } else if question.to_lowercase().contains("wi-fi") || question.to_lowercase().contains("mtandao") {
        (
            "Hatua za msingi za mtandao: 1) Hakikisha Wi-Fi adapter ipo (ip link) 2) Washa service (netsh wlan start / nmcli networking on) 3) Ping gateway 4) Kama bado — agent anaweza kuscan na kurekebisha kwa ruhusa yako.".into(),
            0.4,
        )
    } else if question.to_lowercase().contains("polepole") || question.to_lowercase().contains("slow") {
        (
            "Kompyuta inaenda polepole — sababu kuu: RAM imejaa, disk 95%+, processes nyingi za background. Anza scan (low-risk, automatic): agent atagundua na kupendekeza fix kwa ruhusa yako.".into(),
            0.4,
        )
    } else {
        (
            "Swali limepokelewa. Kwa sasa hakuna suluhisho lililohifadhiwa kwenye Brain kwa tatizo hili — weka kwa maelezo zaidi, au online + Ollama ili LLM (Qwen 2.5 3B VL) ajibu.".into(),
            0.2,
        )
    };

    ChatReply {
        question: question.into(),
        answer_sw: answer,
        source: "brain_offline".into(),
        confidence: conf,
        references: refs.iter().map(|(m, _)| m.solution.clone()).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn offline_chat_inatumia_brain() {
        let db = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        crate::brain::init_tables(&db).await;
        crate::brain::remember(&db, "agent-hr", "hr", "Wi-Fi haifanyi kazi", "restart ya wlansvc service", 0.9).await.unwrap();
        let r = ask(&db, "wi-fi haifanyi kazi kwenye pc mpya", false).await;
        assert_eq!(r.source, "brain_offline");
        assert!(r.answer_sw.contains("wlansvc"), "{}", r.answer_sw);
        assert!(r.confidence > 0.5);
        assert!(!r.references.is_empty());
    }

    #[tokio::test]
    async fn offline_rules_kwa_maswali_yasiyofanana() {
        let db = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        crate::brain::init_tables(&db).await;
        let r = ask(&db, "kompyuta inaenda polepole sana", false).await;
        assert!(r.answer_sw.contains("polepole") || r.answer_sw.contains("RAM"));
        assert!(r.confidence < 0.9); // rules = uhakika wa chini kuliko brain/LLM
    }

    #[tokio::test]
    async fn online_bila_llm_inarudi_offline_fallback() {
        // Sandbox haina Ollama/llama.cpp kwenye 11434 → lazima ifallback (hakuna uongo)
        let db = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        crate::brain::init_tables(&db).await;
        let r = ask(&db, "swali la kawaida lisilopo kwenye brain", true).await;
        assert_ne!(r.source, "llm"); // LLM haipatikani hapa
        assert!(r.confidence <= 0.9);
    }

    #[tokio::test]
    async fn llm_offline_lan_source_ikifika() {
        // Offline (online=false) lakini LLM ya LAN ikiwa ipo → source llm_offline_lan
        let db = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        crate::brain::init_tables(&db).await;
        let r = ask(&db, "swali la jaribio", false).await;
        // sandbox: LLM haipatikani → fallback; hii inathibitisha hakuna panic na
        // source iko halisi (llm_offline_lan / brain_offline)
        assert!(r.source == "llm_offline_lan" || r.source == "brain_offline");
    }
}
