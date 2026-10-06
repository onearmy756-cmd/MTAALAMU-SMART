//! clickpesa.rs — MALIPO YA KIOTOMATIKI (H17a): ClickPesa mobile money.
//!
//! Mtiririko (docs.clickpesa.com):
//!   1. CHECKOUT: mteja anaomba kupata salio → server inaomba Hosted Checkout
//!      link kutoka ClickPesa (`checkout-link/generate-checkout-url`, Bearer JWT)
//!      → inahifadhi orderReference kama PENDING + kurudisha link kwa mteja.
//!   2. Mteja analipa kwenye simu (M-Pesa/Tigo/Airtel/halipaid).
//!   3. WEBHOOK: ClickPesa inatuma `PAYMENT RECEIVED` → server inathibitisha
//!      checksum (HMAC-SHA256 wa body na secret), inaona orderReference,
//!      inaandika topup kwenye BILI (billing::topup) → salio la mteja linaongezeka.
//!   4. STATUS: dashboard/portal inauliza hali ya checkout.
//!
//! Bila env (CLICKPESA_CLIENT_ID/SECRET) mfumo unaendelea: checkout inarudisha
//! ujumbe wa configuration (hakuna crash) — sawa na pfsense.rs::from_env().
//!
//! KANUNI YA SIRI: secret/token hazitoki kwenye API wala logs.

use hmac::{Hmac, Mac};
use serde_json::json;
use rand::RngCore as _;
use sha2::Digest as _;
use sha2::Sha256 as Sha2Hash;
use sqlx::SqlitePool;

type HmacSha256 = Hmac<Sha2Hash>;

const API_BASE: &str = "https://api.clickpesa.com/third-parties";

pub struct ClikConfig {
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
}

impl ClikConfig {
    pub fn from_env() -> Self {
        let cid = std::env::var("CLICKPESA_CLIENT_ID").ok().filter(|s| !s.trim().is_empty());
        let secret = std::env::var("CLICKPESA_CLIENT_SECRET").ok().filter(|s| !s.trim().is_empty());
        ClikConfig { client_id: cid, client_secret: secret }
    }
    pub fn configured(&self) -> bool {
        self.client_id.is_some() && self.client_secret.is_some()
    }
}

/// Checksum ya webhook: HMAC-SHA256(body, secret) hex — kama docs (checksum/checksumMethod)
pub fn webhook_checksum(body: &str, secret: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).expect("hmac key");
    mac.update(body.as_bytes());
    mac.finalize().into_bytes().iter().map(|b| format!("{b:02x}")).collect()
}

pub fn verify_webhook(body: &str, secret: &str, given: Option<&str>) -> bool {
    match given {
        // bila checksum iliyotumwa — tunaikataa (hakuna uthibitisho)
        None => false,
        Some(g) => {
            let expected = webhook_checksum(body, secret);
            // constant-time-ish: compare bytes
            expected.len() == g.len()
                && expected.bytes().zip(g.bytes()).fold(0u8, |acc, (a, b)| acc | (a ^ b)) == 0
        }
    }
}

// ---------- DB ----------
pub async fn init_tables(db: &SqlitePool) {
    let _ = sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS clickpesa_orders (
            order_reference TEXT PRIMARY KEY,
            account TEXT NOT NULL,
            amount_tzs INTEGER NOT NULL,
            status TEXT NOT NULL DEFAULT 'PENDING',
            checkout_link TEXT,
            payment_reference TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT
        )"#,
    )
    .execute(db)
    .await;
}

/// ORDER REFERENCE: alphanumeric (ClickPesa rule) — CP + epoch_ms + account-hash fupi
pub fn make_order_reference(account: &str) -> String {
    let ms = chrono::Utc::now().timestamp_millis();
    let mut rnd = [0u8; 2];
    rand::rngs::OsRng.fill_bytes(&mut rnd);
    let rnd_hex: String = rnd.iter().map(|b| format!("{b:02x}")).collect();
    let mut h = Sha2Hash::new();
    h.update(account.as_bytes());
    let acct_hex: String = h.finalize().iter().take(2).map(|b| format!("{b:02x}")).collect();
    format!("CP{ms}{rnd_hex}{acct_hex}").chars().filter(|c| c.is_ascii_alphanumeric()).collect()
}

/// CHECKOUT: andika PENDING + (kama configured) ita ClickPesa kupata link
pub async fn create_checkout(
    db: &SqlitePool,
    cfg: &ClikConfig,
    account: &str,
    amount_tzs: i64,
) -> Result<serde_json::Value, String> {
    if amount_tzs < 1_000 {
        return Err("kiwango cha chini: TZS 1,000".into());
    }
    let order_ref = make_order_reference(account);
    let now = chrono::Local::now().to_rfc3339();
    sqlx::query("INSERT INTO clickpesa_orders (order_reference, account, amount_tzs, status, created_at) VALUES (?,?,?,'PENDING',?)")
        .bind(&order_ref).bind(account).bind(amount_tzs).bind(&now)
        .execute(db).await.map_err(|e| e.to_string())?;

    if !cfg.configured() {
        return Ok(json!({
            "ok": true, "order_reference": order_ref, "status": "PENDING",
            "configured": false,
            "note_sw": "ClickPesa haijawekwa (CLICKPESA_CLIENT_ID/SECRET) — weka keys kwenye server, kisha checkout inaonyesha link ya malipo.",
        }));
    }
    let link = fetch_checkout_link(cfg, &order_ref, amount_tzs, account).await?;
    sqlx::query("UPDATE clickpesa_orders SET checkout_link=?2, updated_at=?3 WHERE order_reference=?1")
        .bind(&order_ref).bind(&link).bind(&now)
        .execute(db).await.map_err(|e| e.to_string())?;
    Ok(json!({ "ok": true, "order_reference": order_ref, "status": "PENDING", "configured": true, "checkout_link": link }))
}

/// Wito halisi wa ClickPesa: token → checkout link
async fn fetch_checkout_link(cfg: &ClikConfig, order_ref: &str, amount_tzs: i64, account: &str) -> Result<String, String> {
    let client = reqwest::Client::new();
    // 1) token
    let tok: serde_json::Value = client
        .post(format!("{API_BASE}/generate-token"))
        .json(&json!({
            "clientId": cfg.client_id.as_deref().unwrap_or(""),
            "clientSecret": cfg.client_secret.as_deref().unwrap_or(""),
        }))
        .send().await.map_err(|e| e.to_string())?
        .json().await.map_err(|e| e.to_string())?;
    let token = tok.get("access_token").and_then(|v| v.as_str())
        .ok_or_else(|| "ClickPesa token imeshindikana".to_string())?;
    // 2) checkout link
    let res: serde_json::Value = client
        .post(format!("{API_BASE}/checkout-link/generate-checkout-url"))
        .bearer_auth(token)
        .json(&json!({
            "totalPrice": amount_tzs.to_string(),
            "orderReference": order_ref,
            "customerName": account,
            "description": "MTECH OS - Salio la BILI",
        }))
        .send().await.map_err(|e| e.to_string())?
        .json().await.map_err(|e| e.to_string())?;
    res.get("checkoutLink").and_then(|v| v.as_str()).map(|s| s.to_string())
        .ok_or_else(|| format!("ClickPesa checkout imeshindikana: {res}"))
}

// ---------- WEBHOOK: PAYMENT RECEIVED → topup kiotomatiki ----------

/// Ondoa checksum header (kama docs: checksum / checksumMethod)
pub fn checksum_from_headers(headers: &axum::http::HeaderMap) -> Option<String> {
    headers.get("x-clickpesa-signature")
        .or_else(|| headers.get("checksum"))
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

/// Ondoa order kutoka webhook payload + weka topup (idempotent: SUCCESS mara moja)
pub async fn handle_webhook(
    db: &SqlitePool,
    cfg: &ClikConfig,
    body: &str,
    given_checksum: Option<&str>,
) -> serde_json::Value {
    let secret = match &cfg.client_secret {
        Some(s) => s.as_str(),
        None => return json!({ "ok": false, "error": "webhook haijawekwa (secret)" }),
    };
    if !verify_webhook(body, secret, given_checksum) {
        return json!({ "ok": false, "error": "checksum si sahihi" });
    }
    let v: serde_json::Value = match serde_json::from_str(body) {
        Ok(v) => v,
        Err(_) => return json!({ "ok": false, "error": "payload si JSON" }),
    };
    let event = v.get("event").and_then(|e| e.as_str()).unwrap_or("");
    if event != "PAYMENT RECEIVED" {
        // tunakubali (2xx) lakini hatufanyi kitu — kama docs vinavyoeleza
        return json!({ "ok": true, "ignored": event });
    }
    let data = v.get("data").cloned().unwrap_or(json!({}));
    let order_ref = data.get("orderReference").and_then(|o| o.as_str()).unwrap_or("");
    let status = data.get("status").and_then(|s| s.as_str()).unwrap_or("");
    let amount: i64 = data.get("collectedAmount").and_then(|a| a.as_str()).and_then(|s| s.parse().ok())
        .or_else(|| data.get("collectedAmount").and_then(|a| a.as_i64())).unwrap_or(0);
    let pay_ref = data.get("paymentReference").and_then(|p| p.as_str()).unwrap_or("");
    if order_ref.is_empty() || status != "SUCCESS" || amount <= 0 {
        return json!({ "ok": true, "ignored": "event si malipo yaliyofanikiwa" });
    }
    // order ipo na PENDING?
    let row: Option<(String, i64, String)> = sqlx::query_as(
        "SELECT account, amount_tzs, status FROM clickpesa_orders WHERE order_reference=?1",
    )
    .bind(order_ref)
    .fetch_optional(db)
    .await
    .unwrap_or(None);
    let Some((account, stored_amount, st)) = row else {
        return json!({ "ok": false, "error": "order haijulikani" });
    };
    if st == "SUCCESS" {
        return json!({ "ok": true, "idempotent": true, "note_sw": "malipo yamekwishahifadhiwa" });
    }
    // amount ya webhook NDIYO ya kuaminika (mteja alilipa hicho)
    let now = chrono::Local::now().to_rfc3339();
    let ref_code = format!("cp:{order_ref}:{pay_ref}");
    match crate::billing::topup(db, &account, amount, &ref_code).await {
        Ok(balance) => {
            let _ = sqlx::query("UPDATE clickpesa_orders SET status='SUCCESS', payment_reference=?2, updated_at=?3 WHERE order_reference=?1")
                .bind(order_ref).bind(pay_ref).bind(&now)
                .execute(db).await;
            json!({ "ok": true, "account": account, "amount_tzs": amount, "balance_tzs": balance })
        }
        Err(e) => {
            let _ = sqlx::query("UPDATE clickpesa_orders SET status='FAILED', updated_at=?2 WHERE order_reference=?1")
                .bind(order_ref).bind(&now)
                .execute(db).await;
            json!({ "ok": false, "error": e })
        }
    }
}

/// STATUS kwa dashboard/portal
pub async fn order_status(db: &SqlitePool, order_ref: &str) -> serde_json::Value {
    let row: Option<(String, i64, String, Option<String>)> = sqlx::query_as(
        "SELECT account, amount_tzs, status, checkout_link FROM clickpesa_orders WHERE order_reference=?1",
    )
    .bind(order_ref)
    .fetch_optional(db)
    .await
    .unwrap_or(None);
    match row {
        Some((account, amount, status, link)) => json!({
            "ok": true, "order_reference": order_ref, "account": account,
            "amount_tzs": amount, "status": status, "checkout_link": link,
        }),
        None => json!({ "ok": false, "error": "order haijulikani" }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checksum_inabadilika_na_secret() {
        let body = r#"{"event":"PAYMENT RECEIVED"}"#;
        let a = webhook_checksum(body, "secret-1");
        let b = webhook_checksum(body, "secret-1");
        let c = webhook_checksum(body, "secret-2");
        assert_eq!(a, b);
        assert_ne!(a, c);
        assert_eq!(a.len(), 64);
        assert!(verify_webhook(body, "secret-1", Some(&a)));
        assert!(!verify_webhook(body, "secret-1", Some(&c)));
        assert!(!verify_webhook(body, "secret-1", None), "bila checksum → kataa");
    }

    #[test]
    fn order_reference_ni_alphanumeric_na_unique() {
        let a = make_order_reference("mteja1");
        let b = make_order_reference("mteja1");
        assert_ne!(a, b, "kila order ni mpya");
        assert!(a.chars().all(|c| c.is_ascii_alphanumeric()), "ClickPesa rule");
        assert!(a.starts_with("CP"));
    }

    #[tokio::test]
    async fn checkout_bila_env_inaandika_pending_bila_crash() {
        let db = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        init_tables(&db).await;
        let cfg = ClikConfig::from_env(); // sandbox: hakuna keys
        let r = create_checkout(&db, &cfg, "mteja1", 10_000).await.unwrap();
        assert_eq!(r["status"], serde_json::json!("PENDING"));
        assert_eq!(r["configured"], serde_json::Value::Bool(cfg.configured()));
        // chini ya 1,000 → kataa
        assert!(create_checkout(&db, &cfg, "mteja1", 500).await.is_err());
        // status inarudi
        let s = order_status(&db, r["order_reference"].as_str().unwrap()).await;
        assert_eq!(s["status"], serde_json::json!("PENDING"));
    }

    #[tokio::test]
    async fn webhook_inaandika_topup_kiotomatiki_na_idempotent() {
        let db = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        init_tables(&db).await;
        crate::billing::init_tables(&db).await;
        let cfg = ClikConfig { client_id: Some("id".into()), client_secret: Some("sek".into()) };
        // andika order kwa moja kwa moja (create_checkout ya configured inaita intaneti — sandbox haina)
        let order_ref = make_order_reference("mteja1");
        sqlx::query("INSERT INTO clickpesa_orders (order_reference, account, amount_tzs, status, created_at) VALUES (?,?,20000,'PENDING',?)")
            .bind(&order_ref).bind("mteja1").bind(chrono::Local::now().to_rfc3339())
            .execute(&db).await.unwrap();
        // bila checksum → kataa
        let body = format!(r#"{{"event":"PAYMENT RECEIVED","data":{{"orderReference":"{order_ref}","status":"SUCCESS","collectedAmount":"20000","paymentReference":"PAY1"}}}}"#);
        let bad = handle_webhook(&db, &cfg, &body, None).await;
        assert_eq!(bad["ok"], serde_json::Value::Bool(false));
        // checksum mbaya → kataa
        let bad2 = handle_webhook(&db, &cfg, &body, Some("deadbeef")).await;
        assert_eq!(bad2["ok"], serde_json::Value::Bool(false));
        // checksum sahihi → topup
        let sig = webhook_checksum(&body, "sek");
        let good = handle_webhook(&db, &cfg, &body, Some(&sig)).await;
        assert_eq!(good["ok"], serde_json::Value::Bool(true));
        assert_eq!(good["amount_tzs"], serde_json::json!(20000));
        assert_eq!(good["balance_tzs"], serde_json::json!(20000));
        // idempotent: webhook ya pili haiongezi tena
        let again = handle_webhook(&db, &cfg, &body, Some(&sig)).await;
        assert_eq!(again["idempotent"], serde_json::Value::Bool(true));
        let bal = crate::billing::balance_tzs(&db, "mteja1").await;
        assert_eq!(bal, 20_000, "salio limeongezeka MARA MOJA tu");
        // event nyingine → ignored lakini 2xx
        let other = r#"{"event":"PAYOUT INITIATED","data":{}}"#;
        let sig2 = webhook_checksum(other, "sek");
        let ig = handle_webhook(&db, &cfg, other, Some(&sig2)).await;
        assert_eq!(ig["ok"], serde_json::Value::Bool(true));
        assert!(ig.get("ignored").is_some());
    }
}
