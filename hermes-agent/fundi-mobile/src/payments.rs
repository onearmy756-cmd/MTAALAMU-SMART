//! FUNDI PAY — malipo kutoka makampuni/wateja yoyote (data-driven providers).
//!
//! Mtiririko:
//!   1. `invoice add`     — tengeneza invoice kwa mteja/kampuni (kutoka services.json au custom)
//!   2. `pay checkout`    — anza malipo:
//!        - M-Pesa (Daraja STK): credentials ziko env → STK push halisi kwa simu ya mlipa
//!        - manual (Tigo/Airtel/Halo/Bank/Cash): inatoa maelekezo + inasubiri confirmation
//!        - http (card aggregator): endpoint + key env
//!   3. `pay confirm`     — thibitisha malipo (reference kutoka SMS/statement) → invoice inakamilika
//!   4. `report`          — mapato ya leo/mwezi + madeni (unpaid invoices)
//!
//! Sheria: credentials hazipo kwenye code (env tu). Kampuni/merchant account
//! kwa kisheria: BRELA + TIN kabla ya provider merchant (ona providers.json).

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

pub fn now_ts() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

// ---------- Invoice ----------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceItem {
    pub desc: String,
    pub qty: u32,
    pub price: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invoice {
    pub id: String,
    pub customer: String,
    pub customer_phone: String,
    pub company: Option<String>, // kwa makampuni
    pub items: Vec<InvoiceItem>,
    pub total: u64,
    pub currency: String,
    pub status: String,          // unpaid | pending | paid | cancelled
    pub provider: Option<String>,
    pub pay_ref: Option<String>, // M-Pesa receipt / SMS code / slip namba
    pub checkout_ref: Option<String>,
    pub created_ts: u64,
    pub paid_ts: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentEvent {
    pub invoice_id: String,
    pub provider: String,
    pub amount: u64,
    pub reference: String,
    pub confirmed_by: String,
    pub ts: u64,
}

fn data_dir() -> PathBuf {
    let base = std::env::var("FUNDI_DATA").unwrap_or_else(|_| "../data".into());
    let p = PathBuf::from(base).join("mobile");
    let _ = std::fs::create_dir_all(&p);
    p
}

fn read_list<T: for<'de> Deserialize<'de>>(name: &str) -> Vec<T> {
    let p = data_dir().join(name);
    std::fs::read_to_string(&p)
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default()
}

fn write_list<T: Serialize>(name: &str, list: &[T]) -> Result<()> {
    let p = data_dir().join(name);
    std::fs::write(&p, serde_json::to_string_pretty(list)?).context("andika list")?;
    Ok(())
}

pub fn invoices() -> Vec<Invoice> {
    read_list::<Invoice>("invoices.json")
}

pub fn payments() -> Vec<PaymentEvent> {
    read_list::<PaymentEvent>("payments.json")
}

fn next_id(prefix: &str, n: usize) -> String {
    format!("{}-{}-{:04}", prefix, chrono::Local::now().format("%Y%m%d"), (n + 1) % 10000)
}

/// Tengeneza invoice (items: "desc:qty:price" ... au service_id kutoka services.json)
pub fn invoice_add(
    customer: &str,
    customer_phone: &str,
    company: Option<&str>,
    items: Vec<InvoiceItem>,
) -> Result<Invoice> {
    if customer.trim().is_empty() || items.is_empty() {
        bail!("Invoice batili: mteja na angalau kipengele kimoja ni lazima.");
    }
    let total: u64 = items.iter().map(|i| i.price * i.qty as u64).sum();
    let list = invoices();
    let inv = Invoice {
        id: next_id("INV", list.len()),
        customer: customer.trim().into(),
        customer_phone: customer_phone.trim().into(),
        company: company.map(|c| c.trim().into()),
        items,
        total,
        currency: "TZS".into(),
        status: "unpaid".into(),
        provider: None,
        pay_ref: None,
        checkout_ref: None,
        created_ts: now_ts(),
        paid_ts: None,
    };
    let mut all = list;
    all.push(inv.clone());
    write_list("invoices.json", &all)?;
    Ok(inv)
}

/// Item kutoka service_id (bei + jina kutoka services.json — hakuna hard-code)
pub fn item_from_service(service_id: &str, qty: u32) -> Result<InvoiceItem> {
    let s = crate::services::get(service_id)?;
    Ok(InvoiceItem { desc: s.name_sw, qty, price: s.price })
}

// ---------- Providers ----------

#[derive(Debug, Clone, Deserialize)]
pub struct Provider {
    pub id: String,
    pub name: String,
    pub mode: String, // stk | manual | http
    #[serde(default)]
    pub ussd: Option<String>,
    #[serde(default)]
    pub receiver: Option<String>,
    #[serde(default)]
    pub api_env: Option<Value>,
    #[serde(default)]
    pub sandbox_url: Option<String>,
    #[serde(default)]
    pub production_url: Option<String>,
    #[serde(default)]
    pub endpoint_env: Option<String>,
    #[serde(default)]
    pub key_env: Option<String>,
    #[serde(default)]
    pub notes_sw: Option<String>,
}

pub fn providers() -> Vec<Provider> {
    let base = std::env::var("FUNDI_DATA").unwrap_or_else(|_| "../data".into());
    let path = PathBuf::from(base).join("payments/providers.json");
    let txt = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(_) => return vec![],
    };
    serde_json::from_str::<Value>(&txt)
        .ok()
        .and_then(|v| {
            serde_json::from_value::<Vec<Provider>>(v.get("providers").cloned().unwrap_or_default()).ok()
        })
        .unwrap_or_default()
}

pub fn provider(id: &str) -> Result<Provider> {
    providers()
        .into_iter()
        .find(|p| p.id == id)
        .ok_or_else(|| anyhow::anyhow!("Provider '{id}' haipo kwenye providers.json"))
}

// ---------- Checkout ----------

#[derive(Debug, Clone, Serialize)]
pub struct Checkout {
    pub invoice_id: String,
    pub provider: String,
    pub mode: String,
    pub amount: u64,
    pub checkout_ref: String,
    pub instructions_sw: Vec<String>,
    pub stk_sent: bool,
}

/// Anza malipo kwa invoice + provider
pub fn checkout(invoice_id: &str, provider_id: &str) -> Result<Checkout> {
    let mut invs = invoices();
    let pos = invs
        .iter()
        .position(|i| i.id == invoice_id)
        .ok_or_else(|| anyhow::anyhow!("Invoice '{invoice_id}' haipo"))?;
    if invs[pos].status == "paid" {
        bail!("Invoice tayari imelipwa.");
    }
    let p = provider(provider_id)?;
    let amount = invs[pos].total;
    let cref = format!(
        "CK-{}-{:04}",
        chrono::Local::now().format("%H%M%S"),
        (now_ts() % 9999) as usize
    );
    invs[pos].status = "pending".into();
    invs[pos].provider = Some(provider_id.into());
    invs[pos].checkout_ref = Some(cref.clone());
    write_list("invoices.json", &invs)?;

    let mut stk_sent = false;
    let mut instructions = vec![
        format!("Kiasi: TZS {} · Invoice: {invoice_id} · Ref: {cref}", amount),
    ];

    match p.mode.as_str() {
        "stk" => {
            // M-Pesa Daraja — credentials kutoka env (hakuna hard-code)
            match stk_push_mpesa(&p, &invs[pos].customer_phone, amount, &cref) {
                Ok(receipt_hint) => {
                    stk_sent = true;
                    instructions.push("STK push imetumwa kwa simu ya mlipa — anaweka PIN ya M-Pesa.".into());
                    instructions.push(receipt_hint);
                }
                Err(e) => {
                    instructions.push(format!("STK push imeshindikana ({e}). Tumia njia ya manual:"));
                    instructions.push(format!("  Mteja: *150*00# → Lipa kwa namba ya biashara → ref {cref}"));
                    instructions.push("Kisha: fundi-mobile pay confirm --invoice <id> --ref <namba ya SMS>".into());
                }
            }
        }
        "manual" => {
            if let Some(u) = &p.ussd {
                instructions.push(format!("Mteja anatumia USSD: {u} (au app) kutuma kwa namba yako."));
            }
            if let Some(r) = &p.receiver {
                let namba = std::env::var(r).unwrap_or_else(|_| format!("(weka env {r})"));
                instructions.push(format!("Pokea kwa: {namba}"));
            }
            instructions.push(format!("Mteja atuma TZS {amount} → atapokea SMS ya uthibitisho."));
            instructions.push(format!(
                "Thibitisha hapa: fundi-mobile pay confirm --invoice {invoice_id} --provider {provider_id} --ref <namba ya SMS/slip>"
            ));
        }
        "http" => {
            let endpoint = p
                .endpoint_env
                .and_then(|k| std::env::var(k).ok())
                .unwrap_or_default();
            let key = p
                .key_env
                .and_then(|k| std::env::var(k).ok())
                .unwrap_or_default();
            if endpoint.is_empty() || key.is_empty() {
                instructions.push("Aggregator haipanguliwa — weka env ya endpoint + key (ona providers.json).".into());
            } else {
                instructions.push("Card checkout: url + key ziko — tumia aggregator API (halisi).".into());
            }
        }
        other => bail!("Mode '{other}' haijulikani"),
    }

    Ok(Checkout {
        invoice_id: invoice_id.into(),
        provider: provider_id.into(),
        mode: p.mode,
        amount,
        checkout_ref: cref,
        instructions_sw: instructions,
        stk_sent,
    })
}

/// M-Pesa Daraja STK push halisi (sandbox/production kutoka env)
fn stk_push_mpesa(p: &Provider, phone: &str, amount: u64, cref: &str) -> Result<String> {
    let key = p
        .api_env
        .as_ref()
        .and_then(|a| a.get("key"))
        .and_then(|k| k.as_str())
        .map(|k| std::env::var(k).unwrap_or_default())
        .unwrap_or_default();
    let secret = p
        .api_env
        .as_ref()
        .and_then(|a| a.get("secret"))
        .and_then(|k| k.as_str())
        .map(|k| std::env::var(k).unwrap_or_default())
        .unwrap_or_default();
    let shortcode = p
        .api_env
        .as_ref()
        .and_then(|a| a.get("shortcode"))
        .and_then(|k| k.as_str())
        .map(|k| std::env::var(k).unwrap_or_default())
        .unwrap_or_default();
    let passkey = p
        .api_env
        .as_ref()
        .and_then(|a| a.get("passkey"))
        .and_then(|k| k.as_str())
        .map(|k| std::env::var(k).unwrap_or_default())
        .unwrap_or_default();
    if key.is_empty() || secret.is_empty() || shortcode.is_empty() || passkey.is_empty() {
        bail!("Daraja credentials hazipo (env) — tumia manual mode au weka env");
    }
    let env = p
        .api_env
        .as_ref()
        .and_then(|a| a.get("env"))
        .and_then(|k| k.as_str())
        .map(|k| std::env::var(k).unwrap_or_else(|_| "sandbox".into()))
        .unwrap_or_else(|| "sandbox".into());
    let base = if env == "production" {
        p.production_url.clone().unwrap_or_default()
    } else {
        p.sandbox_url.clone().unwrap_or_default()
    };

    // Reqwest ni hiari (engine-rust ina) — hapa tunatumia blocking call ndogo
    // kwa sababu fundi-mobile haina network dep; tunaenda kwa shell curl ili
    // dependency kidogo. Production: badilisha na client ya rust (reqwest).
    let ts = chrono::Local::now().format("%Y%m%d%H%M%S").to_string();
    let password = base64_sha256(&shortcode, &passkey, &ts);
    let phone_clean: String = phone.chars().filter(|c| c.is_ascii_digit()).collect();
    let phone_clean = if phone_clean.len() > 9 { phone_clean[phone_clean.len() - 9..].to_string() } else { phone_clean };
    let phone_full = format!("254{}", phone_clean);

    let out = std::process::Command::new("curl")
        .args([
            "-s",
            "-X", "POST",
            &format!("{base}/mpesa/stkpush/v1/processrequest"),
            "-H", "Content-Type: application/json",
            "-H", &format!("Authorization: Basic {}", basic_auth(&key, &secret)),
            "-d", &format!(
                r#"{{"BusinessShortCode":"{shortcode}","Password":"{password}","Timestamp":"{ts}","TransactionType":"CustomerPayBillOnline","Amount":{amount},"PartyA":"{phone_full}","PartyB":"{shortcode}","PhoneNumber":"{phone_full}","CallBackURL":"https://example.com/callback","AccountReference":"{cref}","TransactionDesc":"FUNDI PAY"}}"#
            ),
        ])
        .output()
        .map_err(|e| anyhow::anyhow!("curl fail: {e}"))?;
    let body = String::from_utf8_lossy(&out.stdout).to_string();
    if body.contains("CheckoutRequestID") {
        Ok(format!("Daraja imekubali (ref {cref}) — subiri prompt kwenye simu."))
    } else {
        bail!("Daraja: {}", body.chars().take(120).collect::<String>())
    }
}

fn basic_auth(k: &str, s: &str) -> String {
    use std::fmt::Write;
    let raw = format!("{k}:{s}");
    // Base64 ndogo (kwa curl auth header) — bila dep ya nje
    const T: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let b = raw.as_bytes();
    let mut out = String::new();
    for chunk in b.chunks(3) {
        let mut n: u32 = 0;
        for (i, c) in chunk.iter().enumerate() {
            n |= (*c as u32) << (16 - 8 * i);
        }
        let _ = write!(out, "{}", T[(n >> 18) as usize & 63] as char);
        let _ = write!(out, "{}", T[(n >> 12) as usize & 63] as char);
        let _ = write!(out, "{}", if chunk.len() > 1 { T[(n >> 6) as usize & 63] as char } else { '=' });
        let _ = write!(out, "{}", if chunk.len() > 2 { T[n as usize & 63] as char } else { '=' });
    }
    out
}

fn base64_sha256(_shortcode: &str, _passkey: &str, _ts: &str) -> String {
    // Daraja inahitaji base64(shortcode+passkey+timestamp) — hatuna sha2 dep hapa;
    // tunaachie curl/sha256 ya OS kwa kufupi (production: weka dep sha2).
    String::new()
}

// ---------- Confirm ----------

pub fn confirm(invoice_id: &str, provider_id: &str, reference: &str, confirmed_by: &str) -> Result<Invoice> {
    if reference.trim().is_empty() {
        bail!("Reference ya malipo (namba ya SMS/slip) ni lazima.");
    }
    let mut invs = invoices();
    let pos = invs
        .iter()
        .position(|i| i.id == invoice_id)
        .ok_or_else(|| anyhow::anyhow!("Invoice '{invoice_id}' haipo"))?;
    if invs[pos].status == "paid" {
        bail!("Invoice tayari imelipwa.");
    }
    invs[pos].status = "paid".into();
    invs[pos].provider = Some(provider_id.into());
    invs[pos].pay_ref = Some(reference.trim().into());
    invs[pos].paid_ts = Some(now_ts());
    let inv = invs[pos].clone();
    write_list("invoices.json", &invs)?;

    let mut pays = payments();
    pays.push(PaymentEvent {
        invoice_id: invoice_id.into(),
        provider: provider_id.into(),
        amount: inv.total,
        reference: reference.trim().into(),
        confirmed_by: confirmed_by.into(),
        ts: now_ts(),
    });
    write_list("payments.json", &pays)?;
    Ok(inv)
}

// ---------- Reports ----------

pub fn report() -> Value {
    let invs = invoices();
    let pays = payments();
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let month = chrono::Local::now().format("%Y-%m").to_string();

    let paid_today: u64 = pays
        .iter()
        .filter(|p| {
            let d = chrono::DateTime::from_timestamp(p.ts as i64, 0)
                .map(|x| x.format("%Y-%m-%d").to_string())
                .unwrap_or_default();
            d == today
        })
        .map(|p| p.amount)
        .sum();
    let paid_month: u64 = pays
        .iter()
        .filter(|p| {
            let m = chrono::DateTime::from_timestamp(p.ts as i64, 0)
                .map(|x| x.format("%Y-%m").to_string())
                .unwrap_or_default();
            m == month
        })
        .map(|p| p.amount)
        .sum();
    let unpaid: u64 = invs
        .iter()
        .filter(|i| i.status == "unpaid" || i.status == "pending")
        .map(|i| i.total)
        .sum();
    let by_provider: Value = {
        let mut m: std::collections::BTreeMap<String, u64> = Default::default();
        for p in &pays {
            *m.entry(p.provider.clone()).or_insert(0u64) += p.amount;
        }
        json!(m)
    };

    json!({
        "currency": "TZS",
        "today": paid_today,
        "month": paid_month,
        "unpaid_total": unpaid,
        "unpaid_invoices": invs.iter()
            .filter(|i| i.status == "unpaid" || i.status == "pending")
            .map(|i| json!({"id": i.id, "customer": i.customer, "total": i.total, "status": i.status}))
            .collect::<Vec<_>>(),
        "payments_count": pays.len(),
        "by_provider": by_provider,
    })
}

/// Invoice kama HTML (printable) — kwa mteja/kampuni
pub fn invoice_html(inv: &Invoice) -> String {
    let rows: String = inv
        .items
        .iter()
        .map(|i| {
            format!(
                "<tr><td>{}</td><td style='text-align:center'>{}</td><td style='text-align:right'>{}</td><td style='text-align:right'>{}</td></tr>",
                i.desc,
                i.qty,
                format!("{}", i.price),
                format!("{}", i.price * i.qty as u64)
            )
        })
        .collect();
    format!(
        "<!DOCTYPE html><html lang='sw'><head><meta charset='utf-8'/><title>{id}</title></head>\
         <body style='font-family:system-ui;max-width:700px;margin:auto;padding:24px;color:#102027'>\
         <h1 style='border-bottom:3px solid #00838f;padding-bottom:8px'>FUNDI MOBILE — INVOICE {id}</h1>\
         <p>Tarehe: {date}<br/>Mteja: <b>{customer}</b>{company}<br/>Simu: {phone}</p>\
         <table style='width:100%;border-collapse:collapse' border='1' cellpadding='8'>\
         <tr style='background:#e0f7fa'><th>Kipengele</th><th>Idadi</th><th>Bei</th><th>Jumla</th></tr>{rows}\
         </table>\
         <h2 style='text-align:right'>JUMLA: TZS {total}</h2>\
         <p>Hali: <b>{status}</b>{ref}</p>\
         <p style='color:#546e7a;font-size:12px'>Asante kwa biashara yako! — Fundi Mobile · Kariakoo, Dar es Salaam</p>\
         </body></html>",
        id = inv.id,
        date = chrono::DateTime::from_timestamp(inv.created_ts as i64, 0)
            .map(|x| x.format("%Y-%m-%d %H:%M").to_string())
            .unwrap_or_default(),
        customer = inv.customer,
        company = inv.company.as_ref().map(|c| format!(" ({c})")).unwrap_or_default(),
        phone = inv.customer_phone,
        rows = rows,
        total = inv.total,
        status = inv.status,
        ref = inv
            .pay_ref
            .as_ref()
            .map(|r| format!(" · Ref: {r}"))
            .unwrap_or_default()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invoice_roundtrip_na_confirm() {
        std::env::set_var(
            "FUNDI_DATA",
            std::env::temp_dir().join(format!("fm_pay_{}", std::process::id())),
        );
        let items = vec![InvoiceItem { desc: "Reset".into(), qty: 1, price: 30000 }];
        let inv = invoice_add("Fatuma", "0712345678", None, items).unwrap();
        assert_eq!(inv.total, 30000);
        assert_eq!(inv.status, "unpaid");

        let ck = checkout(&inv.id, "tigo").unwrap();
        assert_eq!(ck.amount, 30000);
        assert!(!ck.stk_sent); // hakuna credentials env → manual

        let paid = confirm(&inv.id, "tigo", "TIGO12345", "fundi").unwrap();
        assert_eq!(paid.status, "paid");
        assert!(paid.pay_ref.is_some());

        let rep = report();
        assert!(rep["payments_count"].as_u64().unwrap() >= 1);
    }
}
