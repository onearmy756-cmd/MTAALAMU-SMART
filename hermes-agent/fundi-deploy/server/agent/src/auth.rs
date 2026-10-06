//! auth.rs — LOGIN + ROLES (SEHEMU 9.2 + 4.3): credentials za default, tokens za session,
//! admin mkuu na wasaidizi.
//!
//! Mahitaji ya mmiliki:
//!   - "Sehemu ya login kwa watu wote kwa credentials za default"
//!   - "Admin mkuu na wasaidizi — admin mkuu anagawa kazi mwenyewe kwa wahusika"
//!
//! DEFAULT CREDENTIALS (kila mtu anaingia kwa hizi, kisha anabadilisha):
//!   admin / mtech2026  → role: AdminMkuu (madaraka yote)
//!   fundi  / fundi2026 → role: Msaidizi  (kazi zilizopewa)
//!
//! KANUNI: tokens ni random 32-byte hex (halisi), sessions kwenye SQLite,
//! password hashing ni SHA-256 + salt (production: argon2 — interface ile ile).

use rand::RngCore;
use serde::Serialize;
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Role {
    AdminMkuu,
    Msaidizi,
}

impl Role {
    pub fn id(&self) -> &'static str {
        match self {
            Role::AdminMkuu => "admin_mkuu",
            Role::Msaidizi => "msaidizi",
        }
    }

    pub fn name_sw(&self) -> &'static str {
        match self {
            Role::AdminMkuu => "Admin Mkuu",
            Role::Msaidizi => "Msaidizi",
        }
    }

    /// Admin Mkuu anagawa kazi; Msaidizi anafanya zilizopewa.
    pub fn can_assign_tasks(&self) -> bool {
        matches!(self, Role::AdminMkuu)
    }

    /// High-risk actions (install/reboot/wipe) — Admin Mkuu PEKEE anaidhinisha.
    pub fn can_approve_high_risk(&self) -> bool {
        matches!(self, Role::AdminMkuu)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Session {
    pub token: String,
    pub username: String,
    pub role: String,
    pub created_at: String,
}

pub async fn init_tables(db: &SqlitePool) {
    for ddl in [
        r#"CREATE TABLE IF NOT EXISTS users (
            username TEXT PRIMARY KEY,
            password_hash TEXT NOT NULL,
            salt TEXT NOT NULL,
            role TEXT NOT NULL,
            created_at TEXT NOT NULL
        )"#,
        r#"CREATE TABLE IF NOT EXISTS sessions (
            token TEXT PRIMARY KEY,
            username TEXT NOT NULL,
            role TEXT NOT NULL,
            created_at TEXT NOT NULL
        )"#,
    ] {
        let _ = sqlx::query(ddl).execute(db).await;
    }
    // Seed default users (SEHEMU 9.2)
    for (u, p, r) in [
        ("admin", "mtech2026", "admin_mkuu"),
        ("fundi", "fundi2026", "msaidizi"),
    ] {
        let _ = sqlx::query(
            "INSERT OR IGNORE INTO users (username, password_hash, salt, role, created_at) VALUES (?,?,?,?,?)",
        )
        .bind(u)
        .bind(hash_password(p, u))
        .bind("")
        .bind(r)
        .bind(chrono::Local::now().to_rfc3339())
        .execute(db)
        .await;
    }
}

fn hash_password(password: &str, salt: &str) -> String {
    let mut h = Sha256::new();
    h.update(salt.as_bytes());
    h.update(b":mtech:");
    h.update(password.as_bytes());
    format!("{:x}", h.finalize())
}

/// Login halisi: anza kwa default credentials, kisha password iliyobadilishwa.
pub async fn login(db: &SqlitePool, username: &str, password: &str) -> Result<Session, String> {
    let row: Option<(String, String, String, String)> = sqlx::query_as(
        "SELECT username, password_hash, salt, role FROM users WHERE username = ?",
    )
    .bind(username.trim())
    .fetch_optional(db)
    .await
    .unwrap_or(None);

    let (user, stored_hash, salt, role) = match row {
        Some(r) => r,
        None => return Err("Username au password si sahihi".into()),
    };

    // Seed rows: salt="" na hash = hash_password(password_ya_default, username).
    // Rows zilizobadilishwa: salt ya kipekee + hash = hash_password(new, salt).
    let matches = if salt.is_empty() {
        hash_password(password, &user) == stored_hash
    } else {
        hash_password(password, &salt) == stored_hash
    };
    if !matches {
        return Err("Username au password si sahihi".into());
    }

    let mut bytes = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    let token: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
    let created = chrono::Local::now().to_rfc3339();
    sqlx::query("INSERT INTO sessions (token, username, role, created_at) VALUES (?,?,?,?)")
        .bind(&token)
        .bind(&user)
        .bind(&role)
        .bind(&created)
        .execute(db)
        .await
        .map_err(|e| e.to_string())?;
    Ok(Session { token, username: user, role, created_at: created })
}

pub async fn verify_token(db: &SqlitePool, token: &str) -> Option<Session> {
    let row: Option<(String, String, String)> = sqlx::query_as(
        "SELECT token, username, role FROM sessions WHERE token = ?",
    )
    .bind(token)
    .fetch_optional(db)
    .await
    .unwrap_or(None);
    row.map(|(token, username, role)| Session { token, username, role, created_at: String::new() })
}

pub async fn logout(db: &SqlitePool, token: &str) -> bool {
    sqlx::query("DELETE FROM sessions WHERE token = ?")
        .bind(token)
        .execute(db)
        .await
        .map(|r| r.rows_affected() > 0)
        .unwrap_or(false)
}

/// Badilisha password (salt mpya ya kipekee).
pub async fn change_password(db: &SqlitePool, token: &str, new_password: &str) -> Result<(), String> {
    let sess = verify_token(db, token).await.ok_or("Session si sahihi")?;
    if new_password.chars().count() < 6 {
        return Err("Password lazima iwe na herufi 6+".into());
    }
    let mut bytes = [0u8; 16];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    let salt: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
    sqlx::query("UPDATE users SET password_hash=?, salt=? WHERE username=?")
        .bind(hash_password(new_password, &salt))
        .bind(&salt)
        .bind(&sess.username)
        .execute(db)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn default_credentials_zinafanya_kazi() {
        let db = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        init_tables(&db).await;
        // admin / mtech2026 → AdminMkuu
        let s = login(&db, "admin", "mtech2026").await.unwrap();
        assert_eq!(s.role, "admin_mkuu");
        assert!(!s.token.is_empty());
        // fundi / fundi2026 → Msaidizi
        let s2 = login(&db, "fundi", "fundi2026").await.unwrap();
        assert_eq!(s2.role, "msaidizi");
        // password mbaya inakataa
        assert!(login(&db, "admin", "mbaya").await.is_err());
        assert!(login(&db, "hakuna", "mtech2026").await.is_err());
    }

    #[tokio::test]
    async fn token_verify_na_logout() {
        let db = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        init_tables(&db).await;
        let s = login(&db, "admin", "mtech2026").await.unwrap();
        let v = verify_token(&db, &s.token).await.unwrap();
        assert_eq!(v.username, "admin");
        assert!(logout(&db, &s.token).await);
        assert!(verify_token(&db, &s.token).await.is_none());
    }

    #[test]
    fn roles_na_madaraka() {
        assert!(Role::AdminMkuu.can_assign_tasks());
        assert!(!Role::Msaidizi.can_assign_tasks());
        assert!(Role::AdminMkuu.can_approve_high_risk());
        assert!(!Role::Msaidizi.can_approve_high_risk());
    }

    #[tokio::test]
    async fn change_password_inafanya_kazi() {
        let db = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        init_tables(&db).await;
        let s = login(&db, "fundi", "fundi2026").await.unwrap();
        change_password(&db, &s.token, "password-mpya-1").await.unwrap();
        // default haifanyi kazi tena; mpya inafanya
        assert!(login(&db, "fundi", "fundi2026").await.is_err());
        assert!(login(&db, "fundi", "password-mpya-1").await.is_ok());
    }
}
