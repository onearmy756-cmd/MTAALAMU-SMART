//! portal.rs — CUSTOMER PORTAL (H17c): mteja anaingia kwa ACCOUNT + PIN,
//! anaona ripoti ZAKE PEKEE (access control ile ile ya secops) + BILI yake +
//! kuingiza salio (ClickPesa). Admin anaweka PIN kwa mteja (reset).
//!
//! KANUNI: PIN hash ni SHA-256(salt:pin) — kamwe plaintext; kila endpoint
//! inathibitisha token ya portal (random 32-byte hex, DB).

use rand::RngCore;
use serde_json::json;
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;

pub fn hash_pin(pin: &str, salt: &str) -> String {
    let mut h = Sha256::new();
    h.update(salt.as_bytes());
    h.update(b":mtech:");
    h.update(pin.as_bytes());
    format!("{:x}", h.finalize())
}

pub fn new_token() -> String {
    let mut b = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut b);
    b.iter().map(|x| format!("{x:02x}")).collect()
}

pub async fn init_tables(db: &SqlitePool) {
    for ddl in [
        r#"CREATE TABLE IF NOT EXISTS portal_customers (
            account TEXT PRIMARY KEY,
            pin_hash TEXT NOT NULL,
            salt TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )"#,
        r#"CREATE TABLE IF NOT EXISTS portal_sessions (
            token TEXT PRIMARY KEY,
            account TEXT NOT NULL,
            created_at TEXT NOT NULL
        )"#,
    ] {
        let _ = sqlx::query(ddl).execute(db).await;
    }
}

/// ADMIN: weka/badilisha PIN ya mteja (4-8 digits)
pub async fn set_pin(db: &SqlitePool, account: &str, pin: &str) -> Result<(), String> {
    let account = account.trim();
    if account.is_empty() || account.len() > 64 {
        return Err("account si sahihi".into());
    }
    if !pin.chars().all(|c| c.is_ascii_digit()) || pin.len() < 4 || pin.len() > 8 {
        return Err("PIN lazima iwe namba 4-8".into());
    }
    let salt: String = { let mut b = [0u8; 8]; rand::rngs::OsRng.fill_bytes(&mut b); b.iter().map(|x| format!("{x:02x}")).collect() };
    let hash = hash_pin(pin, &salt);
    let now = chrono::Local::now().to_rfc3339();
    sqlx::query("INSERT INTO portal_customers (account, pin_hash, salt, updated_at) VALUES (?,?,?,?) ON CONFLICT(account) DO UPDATE SET pin_hash=?2, salt=?3, updated_at=?4")
        .bind(account).bind(&hash).bind(&salt).bind(&now)
        .execute(db).await.map_err(|e| e.to_string())?;
    Ok(())
}

/// LOGIN: account + PIN → token ya portal
pub async fn login(db: &SqlitePool, account: &str, pin: &str) -> Result<String, String> {
    let row: Option<(String, String)> = sqlx::query_as("SELECT pin_hash, salt FROM portal_customers WHERE account=?1")
        .bind(account.trim())
        .fetch_optional(db)
        .await
        .map_err(|e| e.to_string())?;
    let Some((stored, salt)) = row else { return Err("Account au PIN si sahihi".into()) };
    if hash_pin(pin, &salt) != stored { return Err("Account au PIN si sahihi".into()); }
    let token = new_token();
    sqlx::query("INSERT INTO portal_sessions (token, account, created_at) VALUES (?,?,?)")
        .bind(&token).bind(account.trim()).bind(chrono::Local::now().to_rfc3339())
        .execute(db).await.map_err(|e| e.to_string())?;
    Ok(token)
}

/// AUTH: token → account (kila endpoint ya portal inaita hii)
pub async fn auth(db: &SqlitePool, token: &str) -> Option<String> {
    let row: Option<(String,)> = sqlx::query_as("SELECT account FROM portal_sessions WHERE token=?1")
        .bind(token)
        .fetch_optional(db)
        .await
        .ok()
        .flatten();
    row.map(|(a,)| a)
}

/// MUHTASARI WA MTEJA: ripoti zake (secops), kesi zake, BILI yake — ZAKE PEKEE
pub async fn overview(db: &SqlitePool, account: &str) -> serde_json::Value {
    let sec = crate::secops::summary(db, account).await;
    let bill = crate::billing::statement(db, account).await;
    json!({
        "ok": true,
        "account": account,
        "security": sec,
        "balance_tzs": bill.get("balance_tzs").cloned().unwrap_or(json!(0)),
        "subscription": bill.get("subscription").cloned().unwrap_or(json!(null)),
        "note_sw": "Unaona taarifa ZAKO pekee.",
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn pin_login_na_access_control() {
        let db = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        init_tables(&db).await;
        crate::secops::init_tables(&db).await;
        crate::billing::init_tables(&db).await;
        set_pin(&db, "mteja1", "1234").await.unwrap();
        // PIN mbaya → kataa
        assert!(login(&db, "mteja1", "9999").await.is_err());
        assert!(login(&db, "hakuna", "1234").await.is_err());
        // PIN sahihi → token
        let tok = login(&db, "mteja1", "1234").await.unwrap();
        assert_eq!(tok.len(), 64);
        assert_eq!(auth(&db, &tok).await.as_deref(), Some("mteja1"));
        assert_eq!(auth(&db, "token-mbaya").await, None);
        // PIN mpya inabadilisha ya kale
        set_pin(&db, "mteja1", "5678").await.unwrap();
        assert!(login(&db, "mteja1", "1234").await.is_err());
        assert!(login(&db, "mteja1", "5678").await.is_ok());
        // PIN mibaya inakataliwa
        assert!(set_pin(&db, "mteja2", "12").await.is_err());
        assert!(set_pin(&db, "mteja2", "abcd").await.is_err());
        // overview: ripoti za account hiyo pekee
        let f = crate::secops::Finding { kind: "problem".into(), detail: "x".into(), severity: 10 };
        crate::secops::save_report(&db, "mteja1", "pc-01", "security", &[f]).await.unwrap();
        let o = overview(&db, "mteja1").await;
        assert_eq!(o["security"]["computers"].as_array().unwrap().len(), 1);
        let o2 = overview(&db, "mteja9").await;
        assert_eq!(o2["security"]["computers"].as_array().unwrap().len(), 0, "access control");
    }
}
