//! credit.rs — MFUMO WA CREDITS (maelezo ya mmiliki): "ITUMIE MFUMO WA CREDIT
//! KULIPIA… nalipwa hela zangu kwa kila zana itakayotumika kutatua tatizo
//! pia na hiyo subscription."
//!
//! Muundo:
//!   - Kila account ya mteja ina **credits**. Kila huduma (service — mteja
//!     anajua jina la HUDUMA pekee, kamwe zana) ina gharama ya credits.
//!   - `authorize()` inazuia kazi KABLA haijaanza (asiye na credits hana
//!     huduma). `spend()` inaandika ledger baada ya kazi.
//!   - Ledger kamili (SQLite) = ripoti ya mapato kwa mmiliki (Mbilinyi Tech);
//!     malipo halisi yanaendelea kupitia ClickPesa (pricing.rs).
//!   - Subscription ya mwezi = credits za kila mwezi; Pay-per-use = packs.
//!
//! KANUNI: hakuna uongo — credits zinazuia kazi HALISI kwenye API.

use sqlx::SqlitePool;

/// Bei za huduma kwenye credits — za kibishara (credit 1 = TZS 500). Mteja anajua jina la huduma tu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServicePrice {
    pub service: &'static str,
    pub credits: i64,
}

pub const SERVICE_PRICES: &[ServicePrice] = &[
    ServicePrice { service: "network_scanner", credits: 1 },   // TZS 500
    ServicePrice { service: "health_check", credits: 1 },      // TZS 500
    ServicePrice { service: "malware_scan", credits: 2 },      // TZS 1,000
    ServicePrice { service: "device_management", credits: 2 }, // TZS 1,000
    ServicePrice { service: "driver_update", credits: 2 },     // TZS 1,000
    ServicePrice { service: "app_install", credits: 3 },       // TZS 1,500
    ServicePrice { service: "digital_forensic", credits: 6 },  // TZS 3,000
    ServicePrice { service: "os_install", credits: 10 },       // TZS 5,000
];

/// Thamani ya credit kwa sarafu (kwa kuonyesha kwenye dashboard).
pub const CREDIT_TZS: i64 = 500;

/// Credits za kila mwezi kwenye subscription (pricing.rs inauza packages).
pub const SUBSCRIPTION_MONTHLY_CREDITS: i64 = 500;

pub async fn init_tables(db: &SqlitePool) {
    let _ = sqlx::query(
        "CREATE TABLE IF NOT EXISTS credit_accounts (
            account TEXT PRIMARY KEY,
            balance INTEGER NOT NULL DEFAULT 0,
            updated_at TEXT NOT NULL
        )",
    )
    .execute(db)
    .await;
    let _ = sqlx::query(
        "CREATE TABLE IF NOT EXISTS credit_ledger (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            account TEXT NOT NULL,
            delta INTEGER NOT NULL,
            reason TEXT NOT NULL,
            service TEXT,
            ref_code TEXT,
            balance_after INTEGER NOT NULL,
            created_at TEXT NOT NULL
        )",
    )
    .execute(db)
    .await;
}

pub async fn ensure_account(db: &SqlitePool, account: &str) {
    let _ = sqlx::query("INSERT OR IGNORE INTO credit_accounts (account, balance, updated_at) VALUES (?,0,?)")
        .bind(account)
        .bind(chrono::Local::now().to_rfc3339())
        .execute(db)
        .await;
}

pub async fn balance(db: &SqlitePool, account: &str) -> i64 {
    ensure_account(db, account).await;
    sqlx::query_scalar::<_, i64>("SELECT balance FROM credit_accounts WHERE account = ?")
        .bind(account)
        .fetch_one(db)
        .await
        .unwrap_or(0)
}

/// Nunua credits — ref_code = marejeo ya malipo (TXN ya ClickPesa / ref ya leseni).
pub async fn purchase(db: &SqlitePool, account: &str, credits: i64, ref_code: &str) -> Result<i64, String> {
    if credits <= 0 {
        return Err("idadi ya credits lazima iwe > 0".into());
    }
    if ref_code.trim().is_empty() {
        return Err("marejeo ya malipo (ref_code) ni lazima".into());
    }
    ensure_account(db, account).await;
    let now = chrono::Local::now().to_rfc3339();
    sqlx::query("UPDATE credit_accounts SET balance = balance + ?, updated_at = ? WHERE account = ?")
        .bind(credits)
        .bind(&now)
        .bind(account)
        .execute(db)
        .await
        .map_err(|e| e.to_string())?;
    ledger_add(db, account, credits, "purchase", None, Some(ref_code)).await?;
    Ok(balance(db, account).await)
}

#[allow(clippy::too_many_arguments)]
pub async fn ledger_add(
    db: &SqlitePool,
    account: &str,
    delta: i64,
    reason: &str,
    service: Option<&str>,
    ref_code: Option<&str>,
) -> Result<(), String> {
    let now = chrono::Local::now().to_rfc3339();
    let bal_after = balance(db, account).await;
    sqlx::query(
        "INSERT INTO credit_ledger (account, delta, reason, service, ref_code, balance_after, created_at) VALUES (?,?,?,?,?,?,?)",
    )
    .bind(account)
    .bind(delta)
    .bind(reason)
    .bind(service)
    .bind(ref_code)
    .bind(bal_after)
    .bind(&now)
    .execute(db)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

/// Gharama ya huduma — mteja anapewa jina la HUDUMA (mf. "digital_forensic").
pub fn price_of(service: &str) -> Option<i64> {
    SERVICE_PRICES
        .iter()
        .find(|p| p.service == service)
        .map(|p| p.credits)
}

pub fn service_exists(service: &str) -> bool {
    price_of(service).is_some()
}

/// Zuia kazi KABLA haijaanza — mteja asiye na credits hana huduma.
pub async fn authorize(db: &SqlitePool, account: &str, service: &str) -> Result<i64, String> {
    let price = price_of(service).ok_or_else(|| format!("huduma isiyojulikana: {service}"))?;
    let bal = balance(db, account).await;
    if bal < price {
        return Err(format!(
            "credits hazitoshi: huduma '{service}' inagharimu {price} credits, salio {bal}. Nunua credits au subscription."
        ));
    }
    Ok(price)
}

/// Rekodi matumizi baada ya kazi — ledger yenye huduma + marejeo.
pub async fn spend(db: &SqlitePool, account: &str, service: &str, ref_code: &str) -> Result<i64, String> {
    let price = authorize(db, account, service).await?;
    let now = chrono::Local::now().to_rfc3339();
    sqlx::query("UPDATE credit_accounts SET balance = balance - ?, updated_at = ? WHERE account = ? AND balance >= ?")
        .bind(price)
        .bind(&now)
        .bind(account)
        .bind(price)
        .execute(db)
        .await
        .map_err(|e| e.to_string())?;
    ledger_add(db, account, -price, "usage", Some(service), Some(ref_code)).await?;
    Ok(balance(db, account).await)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn authorize_inazuia_bila_credits() {
        let db = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        init_tables(&db).await;
        let err = authorize(&db, "mteja1", "digital_forensic").await.unwrap_err();
        assert!(err.contains("hazitoshi"), "{err}");
    }

    #[tokio::test]
    async fn purchase_spend_na_ledger_zinafuatana() {
        let db = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        init_tables(&db).await;
        let bal = purchase(&db, "mteja1", 100, "TXN-001").await.unwrap();
        assert_eq!(bal, 100);
        authorize(&db, "mteja1", "network_scanner").await.unwrap(); // 1 credit (TZS 500)
        let bal = spend(&db, "mteja1", "network_scanner", "job-1").await.unwrap();
        assert_eq!(bal, 99);
        let bal = spend(&db, "mteja1", "digital_forensic", "job-2").await.unwrap();
        assert_eq!(bal, 93);
    }

    #[tokio::test]
    async fn spend_bila_credits_inafeli_na_haipunguzi() {
        let db = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        init_tables(&db).await;
        purchase(&db, "mteja2", 3, "TXN-002").await.unwrap();
        assert!(spend(&db, "mteja2", "digital_forensic", "job-x").await.is_err());
        assert_eq!(balance(&db, "mteja2").await, 3, "salio halijabadilika");
    }

    #[test]
    fn bei_za_huduma_ziko_na_ni_za_kibishara() {
        assert_eq!(price_of("network_scanner"), Some(1));
        assert_eq!(price_of("digital_forensic"), Some(6));
        assert_eq!(price_of("os_install"), Some(10));
        // Bei za kibishara: huduma zote kati ya 1-10 credits
        for p in SERVICE_PRICES {
            assert!(p.credits >= 1 && p.credits <= 10, "{}: {} credits", p.service, p.credits);
        }
        assert_eq!(price_of("zana_hisi"), None);
        assert!(service_exists("health_check"));
        assert!(!service_exists("nmap"));
        // Bei za kibishara: huduma zote chini ya credits 12 (os_install = TZS 5,000 pekee)
        for p in SERVICE_PRICES {
            assert!(p.credits >= 1 && p.credits <= 10, "{}: {} credits — nje ya kikundi cha kibishara", p.service, p.credits);
        }
    }

    #[tokio::test]
    async fn purchase_inakataa_thamani_mbaya() {
        let db = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        init_tables(&db).await;
        assert!(purchase(&db, "a", 0, "TXN").await.is_err());
        assert!(purchase(&db, "a", -5, "TXN").await.is_err());
        assert!(purchase(&db, "a", 10, "").await.is_err(), "ref ya malipo ni lazima");
    }
}
