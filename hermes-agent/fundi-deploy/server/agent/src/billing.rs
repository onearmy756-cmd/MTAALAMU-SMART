//! billing.rs — MFUMO WA KIBIASHARA (H9, maelezo ya mmiliki):
//! "weka mfumo wa subscription na pay per use kwa bei rahisi, CREDIT toa kabisa,
//!  weka mfumo wa kibiashara ambao wateja watapata faida na mimi nitapata faida,
//!  zidi kwa kuzingatia bei za kitanzania."
//!
//! MFUMO MPYA (credits zimeondokwa kabisa):
//!   - **SUBSCRIPTION** (kwa mwezi, kwa kifaa/PC): Basic / Standard / Business /
//!     Enterprise — kila kifaa kilichounganishwa kinapata huduma zote za mfumo
//!     kwa mwezi. Bei za Kitanzania, za kibishara (wateja wana faida, mmiliki
//!     ana mapato ya kudumu).
//!   - **PAY-PER-USE** (bila subscription): kifaa kinacholipa kwa kazi iliyofanyika
//!     — scan 2,000 TZS, repair 15,000 TZS, os_install 5,000 TZS (bila kodi ya
//!     usubscriptioni) — kwa wale wasio na subscription.
//!   - **AKAUNTI (wallet)**: mteja anaingiza hela (topup kupitia ClickPesa/benki,
//!     ref halisi) — kazi inakutoa gharama kutoka salio. Subscription inaweza
//!     kulipiwa kutoka salio au ref ya mwezi.
//!
//! KANUNI: hakuna uongo — `authorize()` inazuia kazi kabla haijaanza kama
//! salio/subscription haipatikani; kila matumizi yanaandikwa ledger.

use serde::Serialize;
use sqlx::SqlitePool;

// ---------- BEI ZA KITANZANIA (kibishara — zote TZS) ----------

/// SUBSCRIPTION: kwa mwezi, kwa kifaa kimoja. Wateja wana faida:
/// huduma zote za mfumo (monitoring, scan ya kila siku, repair, remoting,
/// AI chat, forensics) kwa bei ndogo kuliko kumnajisi fundi mara moja.
pub const SUB_BASIC_TZS: u64 = 8_000;      // Kifaa 1–10: kila kifaa/mwezi
pub const SUB_STANDARD_TZS: u64 = 6_500;   // Kifaa 11–50 (punguzo la kati)
pub const SUB_BUSINESS_TZS: u64 = 5_000;   // Kifaa 51–200 (biashara)
pub const SUB_ENTERPRISE_TZS: u64 = 4_000; // Kifaa 200+ (volume)

/// PAY-PER-USE (bila subscription) — kila kazi inalipwa mara moja:
pub const PAYG_SCAN_TZS: u64 = 2_000;      // Uchunguzi wa afya/scan moja
pub const PAYG_REPAIR_TZS: u64 = 15_000;   // Kurekebisha tatizo (HITL)
pub const PAYG_REPORT_TZS: u64 = 5_000;     // Ripoti rasmi (PDF) ya kazi/kesi
pub const PAYG_OS_INSTALL_TZS: u64 = 5_000;// OS install kwa kifaa
pub const PAYG_APP_INSTALL_TZS: u64 = 1_500; // Bundle ya apps kwa kifaa
pub const PAYG_FORENSIC_TZS: u64 = 25_000; // Uchunguzi wa kidijitali (kina)
pub const PAYG_NETMGMT_TZS: u64 = 4_000;   // Usimamizi wa kifaa cha mtandao

/// Punguzo la volume kwa pay-per-use (PC nyingi kwa wakati mmoja):
/// 10+ kazi → 10% off, 50+ → 20% off (economies of scale — agents 100 zinafanya
/// kazi kwa wakati mmoja, gharama haiongezeki).
pub fn volume_discount(units: usize) -> f64 {
    if units >= 50 { 0.20 } else if units >= 10 { 0.10 } else { 0.0 }
}

/// Aina ya subscription kulingana na idadi ya vifaa.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SubTier {
    Basic,
    Standard,
    Business,
    Enterprise,
}

impl SubTier {
    pub fn for_devices(devices: usize) -> SubTier {
        match devices {
            0..=10 => SubTier::Basic,
            11..=50 => SubTier::Standard,
            51..=200 => SubTier::Business,
            _ => SubTier::Enterprise,
        }
    }

    pub fn monthly_price_tzs(&self) -> u64 {
        match self {
            SubTier::Basic => SUB_BASIC_TZS,
            SubTier::Standard => SUB_STANDARD_TZS,
            SubTier::Business => SUB_BUSINESS_TZS,
            SubTier::Enterprise => SUB_ENTERPRISE_TZS,
        }
    }

    pub fn name_sw(&self) -> &'static str {
        match self {
            SubTier::Basic => "Basic",
            SubTier::Standard => "Standard",
            SubTier::Business => "Biashara",
            SubTier::Enterprise => "Kampuni Kubwa",
        }
    }
}

/// Bei ya kazi (pay-per-use) — punguzo la volume linaangaziwa.
pub fn payg_price_tzs(job: &str, units: usize) -> Option<u64> {
    let base = match job {
        "health_check" | "network_scanner" => PAYG_SCAN_TZS,
        "os_install" => PAYG_OS_INSTALL_TZS,
        "app_install" => PAYG_APP_INSTALL_TZS,
        "digital_forensic" => PAYG_FORENSIC_TZS,
        "device_management" => PAYG_NETMGMT_TZS,
        "malware_scan" => PAYG_SCAN_TZS,
        "driver_update" => PAYG_SCAN_TZS,
        "repair" => PAYG_REPAIR_TZS,
        "report" => PAYG_REPORT_TZS,
        _ => return None,
    };
    let d = volume_discount(units);
    Some(((base as f64) * (1.0 - d)).round() as u64)
}

// ---------- DATABASE ----------

pub async fn init_tables(db: &SqlitePool) {
    let _ = sqlx::query(
        "CREATE TABLE IF NOT EXISTS billing_wallets (
            account TEXT PRIMARY KEY,
            balance_tzs INTEGER NOT NULL DEFAULT 0,
            updated_at TEXT NOT NULL
        )",
    )
    .execute(db)
    .await;
    let _ = sqlx::query(
        "CREATE TABLE IF NOT EXISTS billing_subscriptions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            account TEXT NOT NULL,
            tier TEXT NOT NULL,
            devices INTEGER NOT NULL,
            monthly_tzs INTEGER NOT NULL,
            paid_until TEXT NOT NULL,
            ref_code TEXT NOT NULL,
            created_at TEXT NOT NULL
        )",
    )
    .execute(db)
    .await;
    let _ = sqlx::query(
        "CREATE TABLE IF NOT EXISTS billing_ledger (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            account TEXT NOT NULL,
            kind TEXT NOT NULL,           -- topup | subscription | payg
            amount_tzs INTEGER NOT NULL,  -- + ya topup, - ya matumizi
            job TEXT,
            ref_code TEXT,
            balance_after INTEGER NOT NULL,
            created_at TEXT NOT NULL
        )",
    )
    .execute(db)
    .await;
}

async fn ensure_wallet(db: &SqlitePool, account: &str) {
    let _ = sqlx::query("INSERT OR IGNORE INTO billing_wallets (account, balance_tzs, updated_at) VALUES (?,0,?)")
        .bind(account)
        .bind(chrono::Local::now().to_rfc3339())
        .execute(db)
        .await;
}

pub async fn balance_tzs(db: &SqlitePool, account: &str) -> i64 {
    ensure_wallet(db, account).await;
    sqlx::query_scalar::<_, i64>("SELECT balance_tzs FROM billing_wallets WHERE account = ?")
        .bind(account)
        .fetch_one(db)
        .await
        .unwrap_or(0)
}

/// Topup — ref ya malipo (ClickPesa TXN / benki) ni LAZIMA (hakuna uongo).
pub async fn topup(db: &SqlitePool, account: &str, amount_tzs: i64, ref_code: &str) -> Result<i64, String> {
    if amount_tzs < 1_000 {
        return Err("kima cha chini ni TZS 1,000".into());
    }
    if ref_code.trim().is_empty() {
        return Err("marejeo ya malipo (ref) ni lazima — ClickPesa au benki".into());
    }
    ensure_wallet(db, account).await;
    let now = chrono::Local::now().to_rfc3339();
    sqlx::query("UPDATE billing_wallets SET balance_tzs = balance_tzs + ?, updated_at = ? WHERE account = ?")
        .bind(amount_tzs)
        .bind(&now)
        .bind(account)
        .execute(db)
        .await
        .map_err(|e| e.to_string())?;
    ledger_add(db, account, "topup", amount_tzs, None, Some(ref_code)).await?;
    Ok(balance_tzs(db, account).await)
}

async fn ledger_add(
    db: &SqlitePool,
    account: &str,
    kind: &str,
    amount_tzs: i64,
    job: Option<&str>,
    ref_code: Option<&str>,
) -> Result<(), String> {
    let bal_after = balance_tzs(db, account).await;
    let now = chrono::Local::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO billing_ledger (account, kind, amount_tzs, job, ref_code, balance_after, created_at) VALUES (?,?,?,?,?,?,?)",
    )
    .bind(account)
    .bind(kind)
    .bind(amount_tzs)
    .bind(job)
    .bind(ref_code)
    .bind(bal_after)
    .bind(&now)
    .execute(db)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

/// Subscription hai (paid_until > sasa)?
pub async fn active_subscription(db: &SqlitePool, account: &str) -> Option<(String, i64, String)> {
    let now = chrono::Local::now().to_rfc3339();
    let row: Option<(String, i64, String)> = sqlx::query_as(
        "SELECT tier, devices, paid_until FROM billing_subscriptions WHERE account = ? AND paid_until > ? ORDER BY id DESC LIMIT 1",
    )
    .bind(account)
    .bind(&now)
    .fetch_optional(db)
    .await
    .unwrap_or(None);
    row
}

/// Nunua subscription — kwa ref ya malipo (au kutoka salio la wallet).
pub async fn subscribe(
    db: &SqlitePool,
    account: &str,
    devices: usize,
    ref_code: &str,
) -> Result<serde_json::Value, String> {
    let tier = SubTier::for_devices(devices);
    let monthly = tier.monthly_price_tzs() as i64;
    let devices_i = devices.max(1) as i64;
    let total = monthly * devices_i;
    // Lipa kutoka salio kama hakuna ref ya nje
    ensure_wallet(db, account).await;
    if ref_code.trim().is_empty() {
        let bal = balance_tzs(db, account).await;
        if bal < total {
            return Err(format!(
                "salio halitoshi: subscription ya {} kifaa ({}) = TZS {} — salio TZS {}. Ingia topup au tuma ref ya malipo.",
                devices, tier.name_sw(), total, bal
            ));
        }
        sqlx::query("UPDATE billing_wallets SET balance_tzs = balance_tzs - ?, updated_at = ? WHERE account = ?")
            .bind(total)
            .bind(chrono::Local::now().to_rfc3339())
            .bind(account)
            .execute(db)
            .await
            .map_err(|e| e.to_string())?;
    }
    let paid_until = (chrono::Local::now() + chrono::Duration::days(30)).to_rfc3339();
    let now = chrono::Local::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO billing_subscriptions (account, tier, devices, monthly_tzs, paid_until, ref_code, created_at) VALUES (?,?,?,?,?,?,?)",
    )
    .bind(account)
    .bind(match tier { SubTier::Basic => "basic", SubTier::Standard => "standard", SubTier::Business => "business", SubTier::Enterprise => "enterprise" })
    .bind(devices_i)
    .bind(monthly)
    .bind(&paid_until)
    .bind(if ref_code.trim().is_empty() { "wallet" } else { ref_code.trim() })
    .bind(&now)
    .execute(db)
    .await
    .map_err(|e| e.to_string())?;
    if !ref_code.trim().is_empty() {
        ledger_add(db, account, "subscription", -total, None, Some(ref_code.trim())).await?;
    } else {
        ledger_add(db, account, "subscription", -total, None, Some("wallet")).await?;
    }
    Ok(serde_json::json!({
        "ok": true,
        "tier": tier.name_sw(),
        "devices": devices,
        "monthly_tzs": monthly,
        "total_tzs": total,
        "paid_until": paid_until,
    }))
}

/// AUTH: subscription hai inatosha (huduma zote); vinginevyo pay-per-use
/// inakatwa kutoka salio. Inarudisha (chaji TZS, ni_subscription).
pub async fn authorize(db: &SqlitePool, account: &str, job: &str, units: usize) -> Result<(i64, bool), String> {
    if let Some(_) = active_subscription(db, account).await {
        return Ok((0, true)); // subscription = huduma zote bila malipo ya ziada
    }
    let price = payg_price_tzs(job, units)
        .ok_or_else(|| format!("kazi isiyojulikana: {job}"))? as i64;
    let bal = balance_tzs(db, account).await;
    if bal < price {
        return Err(format!(
            "hakuna subscription na salio halitoshi: '{job}' = TZS {price}, salio TZS {bal}. Nunua subscription au ingiza hela."
        ));
    }
    Ok((price, false))
}

/// Rekodi matumizi baada ya kazi — pay-per-use pekee (subscription haiipangi).
pub async fn charge(db: &SqlitePool, account: &str, job: &str, units: usize, ref_code: &str) -> Result<i64, String> {
    let (price, is_sub) = authorize(db, account, job, units).await?;
    if is_sub {
        return Ok(balance_tzs(db, account).await); // hakuna chaji — subscription
    }
    let now = chrono::Local::now().to_rfc3339();
    sqlx::query("UPDATE billing_wallets SET balance_tzs = balance_tzs - ?, updated_at = ? WHERE account = ? AND balance_tzs >= ?")
        .bind(price)
        .bind(&now)
        .bind(account)
        .bind(price)
        .execute(db)
        .await
        .map_err(|e| e.to_string())?;
    ledger_add(db, account, "payg", -price, Some(job), Some(ref_code)).await?;
    Ok(balance_tzs(db, account).await)
}

/// Ripoti ya akaunti: salio + subscription + ledger.
pub async fn statement(db: &SqlitePool, account: &str) -> serde_json::Value {
    let bal = balance_tzs(db, account).await;
    let sub = active_subscription(db, account).await;
    let rows: Vec<(String, i64, String, Option<String>, Option<String>, i64, String)> = sqlx::query_as(
        "SELECT account, kind, amount_tzs, job, ref_code, balance_after, created_at FROM billing_ledger WHERE account = ? ORDER BY id DESC LIMIT 50",
    )
    .bind(account)
    .fetch_all(db)
    .await
    .unwrap_or_default();
    let ledger: Vec<serde_json::Value> = rows
        .iter()
        .map(|(_a, kind, amt, job, refc, bal_after, at)| {
            serde_json::json!({ "kind": kind, "amount_tzs": amt, "job": job, "ref": refc, "balance_after": bal_after, "at": at })
        })
        .collect();
    serde_json::json!({
        "ok": true,
        "account": account,
        "balance_tzs": bal,
        "subscription": sub.map(|(tier, devices, until)| serde_json::json!({
            "tier": tier, "devices": devices, "paid_until": until,
        })),
        "ledger": ledger,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bei_za_kitanzania_ni_za_kibishara() {
        assert_eq!(SUB_BASIC_TZS, 8_000);
        assert_eq!(SUB_ENTERPRISE_TZS, 4_000);
        assert_eq!(PAYG_SCAN_TZS, 2_000);
        assert_eq!(PAYG_REPAIR_TZS, 15_000);
        // subscription ya PC 10 (Basic) = TZS 80,000/mwezi — nafuu kuliko fundi 1
        assert_eq!(SubTier::for_devices(10).monthly_price_tzs(), 8_000);
        assert_eq!(SubTier::for_devices(60).monthly_price_tzs(), 5_000);
    }

    #[test]
    fn volume_discount_ina_hatua_tatu() {
        assert_eq!(volume_discount(5), 0.0);
        assert_eq!(volume_discount(10), 0.10);
        assert_eq!(volume_discount(50), 0.20);
        // os_install × 10 kazi na punguzo 10% = 45,000
        assert_eq!(payg_price_tzs("os_install", 10), Some(4_500));
    }

    #[tokio::test]
    async fn subscription_inatosha_bila_chaji() {
        let db = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        init_tables(&db).await;
        subscribe(&db, "kampuni1", 25, "TXN-SUB-1").await.unwrap();
        let (price, is_sub) = authorize(&db, "kampuni1", "digital_forensic", 1).await.unwrap();
        assert!(is_sub, "subscription = huduma zote bila malipo ya ziada");
        assert_eq!(price, 0);
        let bal = charge(&db, "kampuni1", "digital_forensic", 1, "job-1").await.unwrap();
        assert_eq!(bal, 0, "salio halijaguzwa");
    }

    #[tokio::test]
    async fn payg_inakata_salio_na_inaandika_ledger() {
        let db = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        init_tables(&db).await;
        // bila subscription na bila salio → inazuia
        assert!(authorize(&db, "mteja2", "health_check", 1).await.is_err());
        topup(&db, "mteja2", 10_000, "TXN-1").await.unwrap();
        let (price, is_sub) = authorize(&db, "mteja2", "health_check", 1).await.unwrap();
        assert!(!is_sub && price == PAYG_SCAN_TZS as i64);
        let bal = charge(&db, "mteja2", "health_check", 1, "job-2").await.unwrap();
        assert_eq!(bal, 8_000);
        // kazi isiyojulikana
        assert!(authorize(&db, "mteja2", "zana-hisi", 1).await.is_err());
    }

    #[tokio::test]
    async fn topup_inakata_thamani_mbaya() {
        let db = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        init_tables(&db).await;
        assert!(topup(&db, "a", 500, "TXN").await.is_err(), "chini ya 1,000");
        assert!(topup(&db, "a", 5_000, "").await.is_err(), "ref ni lazima");
    }
}
