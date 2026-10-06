//! brain.rs — NEURALIS BRAIN: kumbukumbu ya PAMOJA ya agents wote (SEHEMU 13 / A.3).
//!
//! Kama Sentinel Desktop: "kila agent inajifunza kutoka kwa wengine." Agent
//! anayetatua tatizo kwenye PC moja anaandika lesson hapa — agent mwingine
//! anayekutana na tatizo lifanayo anarecall suluhisho mara moja.
//!
//! Hifadhi: SQLite sasa (production), interface ya Rust ile ile itaendelea
//! na LanceDB (vector search) kwenye H4 — swap ni ndani ya module hii tu.
//!
//! KANUNI: kumbukumbu ZINATOKA na kazi HALISI (finding + solution + confidence
//! halisi ya ReAct session). Hakuna kujificha kwenye JSON config.

use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    pub agent: String,       // agent aliyeandika (mf. "reacon-PC01", "hr")
    pub pc: String,          // display_name ya PC (hr, hr 1…)
    pub problem: String,     // dalili/tatizo (mf. "Wi-Fi haifanyi kazi")
    pub solution: String,    // nini kilifanya kazi
    pub confidence: f64,     // 0.0–1.0 (kutoka confidence_score ya reacon)
    pub created_at: String,
}

pub async fn init_tables(db: &SqlitePool) {
    let _ = sqlx::query(
        "CREATE TABLE IF NOT EXISTS brain_memories (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            agent TEXT NOT NULL,
            pc TEXT NOT NULL,
            problem TEXT NOT NULL,
            solution TEXT NOT NULL,
            confidence REAL NOT NULL,
            created_at TEXT NOT NULL
        )",
    )
    .execute(db)
    .await;
}

pub async fn remember(
    db: &SqlitePool,
    agent: &str,
    pc: &str,
    problem: &str,
    solution: &str,
    confidence: f64,
) -> Result<i64, String> {
    if problem.trim().is_empty() || solution.trim().is_empty() {
        return Err("problem na solution ni lazima".into());
    }
    let conf = confidence.clamp(0.0, 1.0);
    let created = chrono::Local::now().to_rfc3339();
    let r = sqlx::query(
        "INSERT INTO brain_memories (agent, pc, problem, solution, confidence, created_at) VALUES (?,?,?,?,?,?)",
    )
    .bind(agent)
    .bind(pc)
    .bind(problem.trim())
    .bind(solution.trim())
    .bind(conf)
    .bind(&created)
    .execute(db)
    .await
    .map_err(|e| e.to_string())?;
    Ok(r.last_insert_rowid())
}

fn overlap_score(query: &str, problem: &str) -> f64 {
    let norm = |s: &str| s.to_lowercase()
        .split(|c: char| c.is_whitespace() || c == ',')
        .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()))
        .filter(|w| w.chars().count() >= 3)
        .map(String::from)
        .collect::<std::collections::BTreeSet<_>>();
    let q = norm(query);
    if q.is_empty() { return 0.0; }
    let p = norm(problem);
    let hits = q.intersection(&p).count();
    hits as f64 / q.len() as f64
}

/// Recall: kumbukumbu zinazofanana na tatizo — confidence ya pamoja =
/// match_ratio × confidence ya kumbukumbu. Iliyoju zaidi juu.
pub async fn recall(db: &SqlitePool, query: &str, limit: usize) -> Vec<(Memory, f64)> {
    let rows: Vec<(String, String, String, String, f64, String)> = sqlx::query_as(
        "SELECT agent, pc, problem, solution, confidence, created_at FROM brain_memories",
    )
    .fetch_all(db)
    .await
    .unwrap_or_default();
    let mut scored: Vec<(Memory, f64)> = rows
        .into_iter()
        .map(|(agent, pc, problem, solution, confidence, created_at)| {
            let m = Memory { agent, pc, problem, solution, confidence, created_at };
            let score = overlap_score(query, &m.problem) * m.confidence;
            (m, score)
        })
        .filter(|(_, s)| *s > 0.0)
        .collect();
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(limit);
    scored
}

pub async fn stats(db: &SqlitePool) -> serde_json::Value {
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM brain_memories")
        .fetch_one(db)
        .await
        .unwrap_or(0);
    let agents: Vec<(String,)> = sqlx::query_as(
        "SELECT DISTINCT agent FROM brain_memories",
    )
    .fetch_all(db)
    .await
    .unwrap_or_default();
    serde_json::json!({
        "memories": total,
        "agents_sharing": agents.len(),
        "note_sw": "Kumbukumbu ya pamoja — kila agent anajifunza kutoka kwa wengine (Neuralis Brain)."
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn remember_na_recall_zinajifunza_kutoka_wengine() {
        let db = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        init_tables(&db).await;
        // Agent A anatatua Wi-Fi kwenye PC ya kwanza
        remember(&db, "agent-hr", "hr", "Wi-Fi haifanyi kazi", "restart ya wlansvc service", 0.9).await.unwrap();
        // Agent B (PC nyingine) anakutana na tatizo lifanalo → anarecall
        let got = recall(&db, "wi-fi haifanyi kazi kwenye laptop", 3).await;
        assert!(!got.is_empty(), "brain lazima ikumbuke suluhisho la tatizo linalofanana");
        assert_eq!(got[0].0.solution, "restart ya wlansvc service");
        assert!(got[0].1 > 0.0, "score ya match lazima iwe > 0");
    }

    #[tokio::test]
    async fn recall_haitoi_isiyofanana() {
        let db = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        init_tables(&db).await;
        remember(&db, "a", "pc1", "disk full", "kufuta temp files", 1.0).await.unwrap();
        let got = recall(&db, "printer haichapi", 5).await;
        assert!(got.is_empty());
    }

    #[tokio::test]
    async fn remember_inakataa_mtupu() {
        let db = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        init_tables(&db).await;
        assert!(remember(&db, "a", "pc", "", "sol", 1.0).await.is_err());
        assert!(remember(&db, "a", "pc", "prob", "", 1.0).await.is_err());
    }

    #[test]
    fn confidence_inapunguzwa_kati_0_na_1() {
        let db = tokio::runtime::Runtime::new().unwrap();
        db.block_on(async {
            let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
            init_tables(&pool).await;
            remember(&pool, "a", "p", "printer haichapi kabisa", "washa spooler", 5.0).await.unwrap(); // 5.0 → clamp 1.0
            let got = recall(&pool, "printer haichapi", 1).await;
            assert!(!got.is_empty());
            assert!(got[0].0.confidence <= 1.0);
        });
    }
}
