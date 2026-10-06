//! license.rs — LESENI YA MTECH OS (SEHEMU 8.1): "leseni ni yako — licensed by Mbilinyi Tech".
//!
//! Muundo: MST-XXXX-XXXX-XXXX-XXXX · tiers: Personal / Business / Enterprise
//! Personal na Enterprise zina limits tofauti za PCs; kila key ina expiry.
//! Activation gate: bila leseni halali, deployments hazianzi (inashikamana na pricing.rs).

use rand::RngCore;
use serde::Serialize;
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Tier {
    Personal,
    Business,
    Enterprise,
}

impl Tier {
    pub fn id(&self) -> &'static str {
        match self {
            Tier::Personal => "personal",
            Tier::Business => "business",
            Tier::Enterprise => "enterprise",
        }
    }

    pub fn max_pcs(&self) -> u32 {
        match self {
            Tier::Personal => 5,
            Tier::Business => 50,
            Tier::Enterprise => u32::MAX, // hakuna kikomo
        }
    }

    pub fn price_tzs(&self) -> u64 {
        match self {
            Tier::Personal => 150_000,
            Tier::Business => 900_000,
            Tier::Enterprise => 4_500_000,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct License {
    pub key: String,
    pub tier: String,
    pub owner: String,
    pub max_pcs: u32,
    pub active: bool,
    pub expires: String,
}

const CHARSET: &[u8] = b"ABCDEFGHJKMNPQRSTUVWXYZ23456789"; // hakuna I/L/O/0/1

/// Tengeneza key mpya ya leseni: MST-XXXX-XXXX-XXXX-XXXX (random halisi).
pub fn generate_key() -> String {
    let mut rng = rand::rngs::OsRng;
    let mut group = |out: &mut String| {
        for _ in 0..4 {
            let i = rng.next_u32() as usize % CHARSET.len();
            out.push(CHARSET[i] as char);
        }
    };
    let mut k = String::from("MST-");
    for g in 0..4 {
        let mut s = String::new();
        group(&mut s);
        k.push_str(&s);
        if g < 3 {
            k.push('-');
        }
    }
    k
}

fn key_hash(key: &str) -> String {
    let mut h = Sha256::new();
    h.update(key.as_bytes());
    format!("{:x}", h.finalize())
}

pub async fn init_tables(db: &SqlitePool) {
    let _ = sqlx::query(
        "CREATE TABLE IF NOT EXISTS licenses (
            key_hash TEXT PRIMARY KEY,
            tier TEXT NOT NULL,
            owner TEXT NOT NULL,
            active INTEGER NOT NULL DEFAULT 1,
            expires TEXT NOT NULL
        )",
    )
    .execute(db)
    .await;
}

pub async fn issue(db: &SqlitePool, tier: Tier, owner: &str) -> Result<License, String> {
    if owner.trim().is_empty() {
        return Err("owner ni lazima".into());
    }
    let key = generate_key();
    // Default: mwaka 1
    let expires = (chrono::Local::now() + chrono::Duration::days(365)).to_rfc3339();
    sqlx::query("INSERT INTO licenses (key_hash, tier, owner, active, expires) VALUES (?,?,?,1,?)")
        .bind(key_hash(&key))
        .bind(tier.id())
        .bind(owner.trim())
        .bind(&expires)
        .execute(db)
        .await
        .map_err(|e| e.to_string())?;
    Ok(License {
        key,
        tier: tier.id().into(),
        owner: owner.trim().into(),
        max_pcs: tier.max_pcs(),
        active: true,
        expires,
    })
}

/// Validate halisi: key ipo, active, na bado haijaisha.
pub async fn validate(db: &SqlitePool, key: &str) -> Result<License, String> {
    let row: Option<(String, String, i64, String)> = sqlx::query_as(
        "SELECT tier, owner, active, expires FROM licenses WHERE key_hash = ?",
    )
    .bind(key_hash(key.trim()))
    .fetch_optional(db)
    .await
    .unwrap_or(None);
    let Some((tier, owner, active, expires)) = row else {
        return Err("Leseni si sahihi (haipo kwenye registry)".into());
    };
    if active == 0 {
        return Err("Leseni imesimamishwa".into());
    }
    let exp = chrono::DateTime::parse_from_rfc3339(&expires)
        .map_err(|_| "Expiry ya leseni si sahihi".to_string())?;
    if exp < chrono::Local::now() {
        return Err("Leseni imeisha muda (renew kwa Mbilinyi Tech)".into());
    }
    let t = match tier.as_str() {
        "personal" => Tier::Personal,
        "business" => Tier::Business,
        _ => Tier::Enterprise,
    };
    Ok(License {
        key: key.trim().into(),
        tier: tier,
        owner,
        max_pcs: t.max_pcs(),
        active: true,
        expires,
    })
}

pub async fn revoke(db: &SqlitePool, key: &str) -> bool {
    sqlx::query("UPDATE licenses SET active=0 WHERE key_hash = ?")
        .bind(key_hash(key.trim()))
        .execute(db)
        .await
        .map(|r| r.rows_affected() > 0)
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_format_halisi() {
        for _ in 0..20 {
            let k = generate_key();
            let parts: Vec<&str> = k.split('-').collect();
            assert_eq!(parts.len(), 5, "{k}");
            assert_eq!(parts[0], "MST");
            assert!(parts[1..].iter().all(|p| p.len() == 4));
            // hakuna I/L/O/0/1 (mtumiaasisumbuke)
            assert!(!k.contains(['I', 'L', 'O', '0', '1']), "{k}");
        }
    }

    #[test]
    fn keys_mbili_zinatofautiana() {
        assert_ne!(generate_key(), generate_key());
    }

    #[tokio::test]
    async fn issue_validate_revoke() {
        let db = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        init_tables(&db).await;
        let lic = issue(&db, Tier::Business, "Mbilinyi Tech").await.unwrap();
        assert_eq!(lic.max_pcs, 50);
        let v = validate(&db, &lic.key).await.unwrap();
        assert_eq!(v.owner, "Mbilinyi Tech");
        assert_eq!(v.tier, "business");
        // revoke → validate inashindikana
        assert!(revoke(&db, &lic.key).await);
        assert!(validate(&db, &lic.key).await.is_err());
        // key isiyoipo
        assert!(validate(&db, "MST-AAAA-BBBB-CCCC-DDDD").await.is_err());
    }

    #[test]
    fn tiers_na_bei() {
        assert!(Tier::Personal.max_pcs() < Tier::Business.max_pcs());
        assert_eq!(Tier::Enterprise.max_pcs(), u32::MAX);
        assert!(Tier::Personal.price_tzs() < Tier::Enterprise.price_tzs());
    }
}
