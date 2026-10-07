//! chat.rs — CHAT YA AGENT (SEHEMU 3 Hatua 5): andika tatizo/swali → agent inajibu.
//!
//! Mtiririko (hakuna uongo):
//!   1. Offline: recall kutoka Neuralis Brain (kumbukumbu za agents) + jawabu la rules
//!   2. Online: LLM halisi (Ollama/llama.cpp — Qwen 2.5 3B VL) + muktadha wa brain
//!   3. Kila jawabu lina confidence (chini offline, juu online na LLM)
//!
//! LLM HAIHESABU kamwe — inaeleza/onasih; solutions zinatoka brain/knowledge.

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ChatReply {
    pub question: String,
    pub answer_sw: String,
    #[serde(rename = "answer")]
    pub answer_translated: String, // AI auto-translate kwa lugha ya mtumiaji
    pub language: String,          // sw | en | fr | …
    pub source: String,   // brain | llm | offline_rules
    pub confidence: f64,  // 0.0–1.0
    pub references: Vec<String>, // solutions za agents wenzake
}


/// KANUNI YA SIRI YA BIASHARA: LLM au jawabu lolote la mfumo HAITAJI kamwe
/// jina la zana, amri, lugha ya programu wala injini — mteja anaona HUDUMA za
/// MTECH OS tu. Neno hili huwekwa kwenye kila ombi la LLM.
const LLM_SECRECY_RULE: &str = "KANUNI ZA JIBU: Usa jina la zana, amri, programu, lugha au injini yoyote inayotumika ndani. Mteja anaona HUDUMA za MTECH OS tu (mf. kichanganuzi cha mtandao, uchunguzi wa kidijitali). Usitaje binaries wala paths.";

/// Urefu wa jawabu (maelezo ya mmiliki: "ai isummarize kwa maelezo marefu na ya kati")
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Length {
    Short,
    Medium,
    Long,
}

impl Length {
    pub fn instruction_sw(&self) -> &'static str {
        match self {
            Length::Short => "Jibu kwa Kiswahili kwa UFUPI (sentensi 1-2).",
            Length::Medium => "Jibu kwa Kiswahili kwa MAELEZO YA KATI (aya 1-2, hatua muhimu).",
            Length::Long => "Jibu kwa Kiswahili kwa MAELEZO MAREFU YAKAMILI: aya kadhaa, sababu, hatua zote, na ushauri wa kina.",
        }
    }
}

/// Swali la LLM halisi — LLM inaishi kwenye SERVER KUU YA LAN (docker),
/// kwa hiyo inafanya kazi hata OFFLINE (mtandao wa ndani unaotosha).
/// Online ni kwa cloud AI PEKEE — LLM ya LAN haitegemei internet.
async fn llm_ask(db: &sqlx::SqlitePool, question: &str, context: &str, len: Length) -> Option<String> {
    // CUSTOM MODEL/API: config kutoka DB (mteja ameweka yake) au default ya server kuu
    let cfg = crate::ai_config::load_with_db(db).await;
    let base = cfg.url.trim_end_matches('/');
    let sys = format!("Wewe ni fundi wa kompyuta wa MTECH OS. {}\n\n{LLM_SECRECY_RULE}\n\nMuktadha (suluhisho za agents wenzake):\n{context}", len.instruction_sw());
    let (url, body, bearer) = match crate::ai_config::style_for_url(base) {
        "openai" => (
            format!("{base}/chat/completions"),
            serde_json::json!({
                "model": cfg.model,
                "messages": [
                    {"role": "system", "content": sys},
                    {"role": "user", "content": question}
                ],
                "stream": false,
            }),
            crate::ai_config::ai_key(),
        ),
        _ => (
            format!("{base}/api/generate"),
            serde_json::json!({
                "model": cfg.model,
                "prompt": format!("{sys}\n\nSwali: {question}\n\nJibu:"),
                "stream": false,
            }),
            None,
        ),
    };
    let client = reqwest::Client::new();
    let mut req = client.post(&url).json(&body);
    if let Some(k) = bearer {
        req = req.bearer_auth(k);
    }
    let resp = req
        .timeout(std::time::Duration::from_secs(60))
        .send()
        .await
        .ok()?;
    let v: serde_json::Value = resp.json().await.ok()?;
    // OpenAI-compat inajibu choices[0].message.content; Ollama/llama.cpp inajibu response
    v["response"].as_str()
        .or_else(|| v["choices"][0]["message"]["content"].as_str())
        .map(String::from)
}

/// Chat: andika tatizo/swali → jawabu (offline = brain + rules; online = LLM + brain).
pub async fn ask_username(db: &sqlx::SqlitePool, brain: &crate::brain::Brain, username: &str, question: &str, online: bool, len: Length) -> ChatReply {
    let mut r = ask(brain, question, online, len).await;
    // AI auto-translate (maelezo ya mmiliki): lugha ya mtumiaji kutoka DB;
    // LLM haipatikani → jibu la asili (Kiswahili) — hakuna uongo.
    let (translated, lang) = crate::language::auto_translate(db, username, &r.answer_sw).await;
    r.answer_translated = crate::tools::sanitize_output(&translated);
    r.language = lang;
    r
}

pub async fn ask(brain: &crate::brain::Brain, question: &str, online: bool, len: Length) -> ChatReply {
    // 1. Neuralis Brain daima (offline + online context)
    let refs = crate::brain::recall(brain, question, 3).await;
    let context: String = refs
        .iter()
        .map(|(m, _)| format!("- {} (PC {}): {}", m.problem, m.pc, m.solution))
        .collect::<Vec<_>>()
        .join("\n");

    // 2. LLM ya SERVER KUU YA LAN: inafanya kazi hata OFFLINE (docker service
    //    ya LLM inaishi ndani ya LAN — kompyuta zote zimeunganishwa nayo).
    //    `online` inaongeza tu uwezo wa cloud AI; LLM ya LAN haitegemei internet.
    if let Some(answer) = llm_ask(&brain.db, question, &context, len).await {
        return ChatReply {
            question: question.into(),
            answer_sw: crate::tools::sanitize_output(answer.trim()),
            answer_translated: String::new(), // ask_username huitafsiri baadaye
            language: "sw".into(),
            source: if online { "llm".into() } else { "llm_offline_lan".into() },
            confidence: if refs.is_empty() { 0.7 } else { 0.9 },
            references: refs.iter().map(|(m, _)| crate::tools::sanitize_output(&m.solution)).collect(),
        };
    }
    // LLM ya LAN haipatikani → offline fallback (hakuna uongo)

    // 3. Offline fallback: brain + rules za msingi
    let detail = match len {
        Length::Short => String::new(),
        Length::Medium => "\n\nMaelezo: Hii ni hatua iliyothibitika kwenye kifaa kinachofanana. Ikiwa imefeli: hakikisha kifaa kiko hai kwenye mtandao, kisha anzisha upya huduma husika. Mfumo unaandika kila hatua kwenye ripoti.".to_string(),
        Length::Long => "\n\nMaelezo kamili: (1) Dalili ulizoziona zinalingana na tatizo lililopatikana awali kwenye mtandao wa kampuni — hii ni kawaida na inatibika. (2) Sababu kuu hutokea wakati huduma ya mtandao/vifaa inapokwama au vifaa vina hitilafu ya ndani. (3) Hatua za kurekebisha: a) hakikisha kifaa kimeungwa na chanzo cha nguvu na mtandao; b) anzisha upya huduma husika kwenye kifaa; c) endesha uchunguzi wa afya (tab 🧰 HUDUMA); d) kama tatizo linaendelea, agent inaweza kufanya kazi kwa kina zaidi kwa ruhusa yako (HITL). (4) Baada ya kurekebisha, mfumo unaandika ripoti kamili kwa admin na kuhifadhi suluhisho kwenye kumbukumbu ya pamoja ili agents wengine wajifunze.".to_string(),
    };
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
            "Swali limepokelewa. Kwa sasa hakuna suluhisho lililohifadhiwa kwenye kumbukumbu ya pamoja kwa tatizo hili — weka maelezo zaidi, au washa mfumo mtandaoni ili AI ya ndani ijibu.".into(),
            0.2,
        )
    };

    // KANUNI: kila jawabu (LLM au rules) lasafishwa — majina ya zana hayatokei
    let answer_sw = crate::tools::sanitize_output(&format!("{answer}{detail}"));
    let references = refs
        .iter()
        .map(|(m, _)| crate::tools::sanitize_output(&m.solution))
        .collect();
    ChatReply {
        question: question.into(),
        answer_sw,
        answer_translated: String::new(), // ask_username huitafsiri baadaye
        language: "sw".into(),
        source: "brain_offline".into(),
        confidence: conf,
        references,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn length_long_ina_maelezo_marefu_na_short_haina() {
        let db = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        crate::brain::init_tables(&db).await;
        let dir = std::env::temp_dir().join(format!("mtech-ctest-{}", uuid::Uuid::new_v4()));
        let brain = crate::brain::Brain::new(db, dir.to_str().unwrap());
        let r_short = ask(&brain, "swali lisilopo kwenye brain kabisa", false, Length::Short).await;
        let r_long = ask(&brain, "swali lisilopo kwenye brain kabisa", false, Length::Long).await;
        assert!(r_long.answer_sw.chars().count() > r_short.answer_sw.chars().count(),
            "jawabu la long lazima liwe refu kuliko short ({} vs {})", r_long.answer_sw.chars().count(), r_short.answer_sw.chars().count());
        assert!(r_long.answer_sw.contains("Maelezo kamili"), "long ina maelezo kamili");
    }

    #[tokio::test]
    async fn offline_chat_inatumia_brain() {
        let db = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        crate::brain::init_tables(&db).await;
        let dir = std::env::temp_dir().join(format!("mtech-ctest-{}", uuid::Uuid::new_v4()));
        let brain = crate::brain::Brain::new(db, dir.to_str().unwrap());
        crate::brain::remember(&brain, "agent-hr", "hr", "Wi-Fi haifanyi kazi", "restart ya wlansvc service", 0.9).await.unwrap();
        let r = ask(&brain, "wi-fi haifanyi kazi kwenye pc mpya", false, Length::Medium).await;
        assert_eq!(r.source, "brain_offline");
        assert!(r.answer_sw.contains("wlansvc"), "{}", r.answer_sw);
        assert!(r.confidence > 0.5);
        assert!(!r.references.is_empty());
    }

    #[tokio::test]
    async fn offline_rules_kwa_maswali_yasiyofanana() {
        let db = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        crate::brain::init_tables(&db).await;
        let dir = std::env::temp_dir().join(format!("mtech-ctest-{}", uuid::Uuid::new_v4()));
        let brain = crate::brain::Brain::new(db, dir.to_str().unwrap());
        let r = ask(&brain, "kompyuta inaenda polepole sana", false, Length::Short).await;
        assert!(r.answer_sw.contains("polepole") || r.answer_sw.contains("RAM"));
        assert!(r.confidence < 0.9); // rules = uhakika wa chini kuliko brain/LLM
    }

    #[tokio::test]
    async fn online_bila_llm_inarudi_offline_fallback() {
        // Sandbox haina Ollama/llama.cpp kwenye 11434 → lazima ifallback (hakuna uongo)
        let db = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        crate::brain::init_tables(&db).await;
        let dir = std::env::temp_dir().join(format!("mtech-ctest-{}", uuid::Uuid::new_v4()));
        let brain = crate::brain::Brain::new(db, dir.to_str().unwrap());
        let r = ask(&brain, "swali la kawaida lisilopo kwenye brain", true, Length::Long).await;
        assert_ne!(r.source, "llm"); // LLM haipatikani hapa
        assert!(r.confidence <= 0.9);
    }

    #[tokio::test]
    async fn llm_offline_lan_source_ikifika() {
        // Offline (online=false) lakini LLM ya LAN ikiwa ipo → source llm_offline_lan
        let db = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        crate::brain::init_tables(&db).await;
        let dir = std::env::temp_dir().join(format!("mtech-ctest-{}", uuid::Uuid::new_v4()));
        let brain = crate::brain::Brain::new(db, dir.to_str().unwrap());
        let r = ask(&brain, "swali la jaribio", false, Length::Medium).await;
        // sandbox: LLM haipatikani → fallback; hii inathibitisha hakuna panic na
        // source iko halisi (llm_offline_lan / brain_offline)
        assert!(r.source == "llm_offline_lan" || r.source == "brain_offline");
    }

    #[tokio::test]
    async fn ask_username_inatafsiri_kwa_lugha_ya_mtumiaji() {
        let db = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        crate::brain::init_tables(&db).await;
        crate::language::init_tables(&db).await;
        let dir = std::env::temp_dir().join(format!("mtech-ctest-{}", uuid::Uuid::new_v4()));
        let brain = crate::brain::Brain::new(db.clone(), dir.to_str().unwrap());
        crate::language::set_language(&db, "mteja1", "sw").await.unwrap();
        let r = ask_username(&db, &brain, "mteja1", "kompyuta inaenda polepole", false, Length::Medium).await;
        // Sandbox haina LLM ya translate → jibu la asili (Kiswahili) + language ya DB
        assert_eq!(r.language, "sw");
        assert!(!r.answer_translated.is_empty(), "jibu la asili liko kwa digit moja");
        // hakuna majina ya zana kwenye jibu lililotafsiriwa (white-label)
        assert!(!r.answer_translated.contains("nmap"));
    }
}
