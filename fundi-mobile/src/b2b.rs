//! FUNDI B2B — zana za fundi kuzungumza na MAKAMPUNI.
//!
//! Kanuni mpya ya mmiliki:
//!   - Tools zote za mfumo ni BURE kwa fundi yeyote (pricing_mode: free).
//!   - Bei ya kazi kwa mteja/kampuni ni uamuzi wa FUNDI mwenyewe.
//!   - Hii module ni "kalamu na karatasi": quote/invoice/professional letter
//!     ambazo fundi anazituma kwa kampuni — malipo yanaendelea nje ya mfumo
//!     (M-Pesa ya fundi, benki ya fundi) — mfumo HAUpigi pesani.
//!
//! Mtiririko: quote (OLE) → kampuni inakubali → invoice → imeelipwa (record).

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::{Path, PathBuf};

pub fn now_ts() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct B2BItem {
    pub desc: String,
    pub qty: u32,
    pub price: u64, // bei ya FUNDI mwenyewe
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct B2BDoc {
    pub id: String,
    pub kind: String, // quote | invoice
    pub fundi: String,
    pub company: String,
    pub company_contact: String,
    pub site: Option<String>, // mahali/tawi la kampuni
    pub items: Vec<B2BItem>,
    pub total: u64,
    pub currency: String,
    pub status: String, // quote: sent|accepted|rejected ; invoice: unpaid|paid|overdue
    pub ref_quote: Option<String>,
    pub note_sw: Option<String>,
    pub created_ts: u64,
}

fn data_dir() -> PathBuf {
    let base = std::env::var("FUNDI_DATA").unwrap_or_else(|_| "../data".into());
    let p = PathBuf::from(base).join("mobile");
    let _ = std::fs::create_dir_all(&p);
    p
}

fn load() -> Vec<B2BDoc> {
    let p = data_dir().join("b2b_docs.json");
    std::fs::read_to_string(&p)
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default()
}

fn save(list: &[B2BDoc]) -> Result<()> {
    let p = data_dir().join("b2b_docs.json");
    std::fs::write(&p, serde_json::to_string_pretty(list)?)?;
    Ok(())
}

fn next_id(kind: &str, n: usize) -> String {
    let pfx = if kind == "quote" { "QT" } else { "IN" };
    format!("{}-{}-{:04}", pfx, chrono::Local::now().format("%Y%m%d"), (n + 1) % 10000)
}

/// Tengeneza QUOTE (ole kwa kampuni) — bei ni za fundi mwenyewe
pub fn quote_add(
    fundi: &str,
    company: &str,
    company_contact: &str,
    site: Option<&str>,
    items: Vec<B2BItem>,
    note_sw: Option<&str>,
) -> Result<B2BDoc> {
    if company.trim().is_empty() || items.is_empty() {
        bail!("Quote batili: kampuni na angalau kipengele kimoja ni lazima.");
    }
    let list = load();
    let total: u64 = items.iter().map(|i| i.price * i.qty as u64).sum();
    let doc = B2BDoc {
        id: next_id("quote", list.len()),
        kind: "quote".into(),
        fundi: fundi.trim().into(),
        company: company.trim().into(),
        company_contact: company_contact.trim().into(),
        site: site.map(|s| s.trim().into()),
        items,
        total,
        currency: "TZS".into(),
        status: "sent".into(),
        ref_quote: None,
        note_sw: note_sw.map(|s| s.trim().into()),
        created_ts: now_ts(),
    };
    let mut all = list;
    all.push(doc.clone());
    save(&all)?;
    Ok(doc)
}

/// Quote → INVOICE (baada ya kampuni kukubali)
pub fn quote_to_invoice(quote_id: &str) -> Result<B2BDoc> {
    let mut list = load();
    let pos = list
        .iter()
        .position(|d| d.id == quote_id && d.kind == "quote")
        .ok_or_else(|| anyhow::anyhow!("Quote '{quote_id}' haipo"))?;
    if list[pos].status != "accepted" {
        bail!("Quote bado haikubaliwi (status: {}). Washa status kwanza: b2b status --id {quote_id} --set accepted", list[pos].status);
    }
    let q = list[pos].clone();
    let inv = B2BDoc {
        id: next_id("invoice", list.len()),
        kind: "invoice".into(),
        fundi: q.fundi,
        company: q.company,
        company_contact: q.company_contact,
        site: q.site,
        items: q.items,
        total: q.total,
        currency: q.currency,
        status: "unpaid".into(),
        ref_quote: Some(q.id),
        note_sw: q.note_sw,
        created_ts: now_ts(),
    };
    list.push(inv.clone());
    save(&list)?;
    Ok(inv)
}

pub fn set_status(id: &str, status: &str) -> Result<B2BDoc> {
    let allowed = [
        "sent", "accepted", "rejected", // quote
        "unpaid", "paid", "overdue",    // invoice
    ];
    if !allowed.contains(&status) {
        bail!("Status '{status}' si sahihi ({})", allowed.join(", "));
    }
    let mut list = load();
    let pos = list
        .iter()
        .position(|d| d.id == id)
        .ok_or_else(|| anyhow::anyhow!("'{id}' haipo"))?;
    list[pos].status = status.into();
    let doc = list[pos].clone();
    save(&list)?;
    Ok(doc)
}

pub fn list(kind: Option<&str>) -> Vec<B2BDoc> {
    load()
        .into_iter()
        .filter(|d| kind.is_none() || d.kind == kind.unwrap_or_default())
        .collect()
}

/// PDF rasmi ya quote/invoice (logo ya FUNDI + jedwali la items) — kwa makampuni
pub fn doc_pdf(doc: &B2BDoc) -> Vec<u8> {
    crate::pdf::b2b_doc_pdf(doc)
}

/// Risiti/profoma ya HTML (printable) — ina jina la fundi, si la mfumo
pub fn doc_html(doc: &B2BDoc) -> String {
    let title = if doc.kind == "quote" { "QUOTE (OLE BEI)" } else { "INVOICE" };
    let rows: String = doc
        .items
        .iter()
        .map(|i| {
            format!(
                "<tr><td>{}</td><td style='text-align:center'>{}</td><td style='text-align:right'>{}</td><td style='text-align:right'>{}</td></tr>",
                i.desc, i.qty, i.price, i.price * i.qty as u64
            )
        })
        .collect();
    format!(
        "<!DOCTYPE html><html lang='sw'><head><meta charset='utf-8'/><title>{id}</title></head>\
         <body style='font-family:system-ui;max-width:720px;margin:auto;padding:24px;color:#102027'>\
         <h1 style='border-bottom:3px solid #00838f;padding-bottom:8px'>{title} {id}</h1>\
         <p><b>Fundi/Mtoa huduma:</b> {fundi}<br/>\
         <b>Mteja:</b> {company}{contact}<br/>\
         {site}Tarehe: {date}</p>\
         <table style='width:100%;border-collapse:collapse' border='1' cellpadding='8'>\
         <tr style='background:#e0f7fa'><th>Kazi/Vifaa</th><th>Idadi</th><th>Bei</th><th>Jumla</th></tr>{rows}</table>\
         <h2 style='text-align:right'>JUMLA: {cur} {total}</h2>\
         <p>Hali: <b>{status}</b>{rq}{note}</p>\
         <p style='color:#546e7a;font-size:12px'>Malipo yanapokelewa na mtoa huduma (namba za malipo za fundi).\
         Hati hii imetengenezwa na FUNDI MOBILE kwa ajili ya {fundi}.</p>\
         </body></html>",
        id = doc.id, title = title, fundi = doc.fundi,
        company = doc.company,
        contact = if doc.company_contact.is_empty() {
            String::new()
        } else {
            format!(" · {}", doc.company_contact)
        },
        site = doc.site.as_ref().map(|s| format!("<b>Eneo:</b> {s}<br/>")).unwrap_or_default(),
        date = chrono::DateTime::from_timestamp(doc.created_ts as i64, 0)
            .map(|x| x.format("%Y-%m-%d").to_string())
            .unwrap_or_default(),
        rows = rows, cur = doc.currency, total = doc.total, status = doc.status,
        rq = doc.ref_quote.as_ref().map(|r| format!(" · Rejea: {r}")).unwrap_or_default(),
        note = doc
            .note_sw
            .as_ref()
            .map(|n| format!("<br/><i>{n}</i>"))
            .unwrap_or_default()
    )
}

/// Ripoti ya B2B: quotes zilizokubaliwa, invoices unpaid/paid — kwa fundi
pub fn report() -> serde_json::Value {
    let docs = load();
    let invoices: Vec<&B2BDoc> = docs.iter().filter(|d| d.kind == "invoice").collect();
    let quotes: Vec<&B2BDoc> = docs.iter().filter(|d| d.kind == "quote").collect();
    json!({
        "quotes": {
            "total": quotes.len(),
            "accepted": quotes.iter().filter(|q| q.status == "accepted").count(),
            "value_accepted_tzs": quotes.iter().filter(|q| q.status == "accepted").map(|q| q.total).sum::<u64>(),
        },
        "invoices": {
            "total": invoices.len(),
            "paid": invoices.iter().filter(|i| i.status == "paid").count(),
            "unpaid": invoices.iter().filter(|i| i.status == "unpaid").count(),
            "value_paid_tzs": invoices.iter().filter(|i| i.status == "paid").map(|i| i.total).sum::<u64>(),
            "value_unpaid_tzs": invoices.iter().filter(|i| i.status == "unpaid").map(|i| i.total).sum::<u64>(),
        },
        "note_sw": "Malipo yote yanaingia kwa fundi mwenyewe (nje ya mfumo). Mfumo haukipigi pesani."
    })
}
