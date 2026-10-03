//! CONSENT (HITL) — kanuni kuu ya FUNDI MOBILE.
//!
//! HAKUNA kazi inayoharibu (reset/flash/FRP) bila consent iliyosajiliwa.
//! Consent inahifadhiwa kwenye `data/mobile/consents.json` (immutable log)
//! na ina: mteja, IMEI, kazi, sahihi (jina), tarehe, bei.
//!
//! Sheria: Kufuta password ya simu ya mtu mwingine bila ruhusa = kosa la
//! jinai (Sheria ya Makosa ya Jinai Tanzania, Kifungu 267).

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Consent {
    pub consent_id: String,
    pub customer_name: String,
    pub customer_phone: String,
    pub id_number: Option<String>,
    pub brand: String,
    pub model: String,
    pub imei: String,
    pub service_id: String,
    pub destroys_data: bool,
    pub price_tzs: u64,
    pub statement_sw: String,
    pub signed_by: String, // jina la mteja aliyasaini
    pub technician: String,
    pub ts: String,
}

fn store_path() -> PathBuf {
    let base = std::env::var("FUNDI_DATA").unwrap_or_else(|_| "../data".into());
    PathBuf::from(base).join("mobile/consents.json")
}

fn load_all() -> Vec<Consent> {
    let p = store_path();
    std::fs::read_to_string(&p)
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default()
}

fn save_all(list: &[Consent]) -> Result<()> {
    let p = store_path();
    if let Some(dir) = p.parent() {
        std::fs::create_dir_all(dir).context("unda data/mobile/")?;
    }
    std::fs::write(&p, serde_json::to_string_pretty(list)?).context("andika consents.json")?;
    Ok(())
}

/// Rekodi consent mpya — inarudisha consent_id
pub fn record(
    customer_name: &str,
    customer_phone: &str,
    id_number: Option<&str>,
    brand: &str,
    model: &str,
    imei: &str,
    service_id: &str,
    destroys_data: bool,
    price_tzs: u64,
    technician: &str,
) -> Result<String> {
    if customer_name.trim().is_empty() || imei.trim().len() < 8 {
        bail!("Consent batili: jina na IMEI (namba kamili 15) ni lazima.");
    }
    let now = chrono::Local::now();
    let id = format!("CNS-{}-{:04}", now.format("%Y%m%d"), (load_all().len() + 1) % 10000);
    let c = Consent {
        consent_id: id.clone(),
        customer_name: customer_name.trim().into(),
        customer_phone: customer_phone.trim().into(),
        id_number: id_number.map(|s| s.trim().to_string()),
        brand: brand.trim().into(),
        model: model.trim().into(),
        imei: imei.trim().replace(' ', ""),
        service_id: service_id.into(),
        destroys_data,
        price_tzs,
        statement_sw: if destroys_data {
            "Simu hii ni yangu au nina ruhusa ya mmiliki. Ninakubali data YOTE inaweza kufutwa kama kazi itahitaji.".into()
        } else {
            "Simu hii ni yangu au nina ruhusa ya mmiliki.".into()
        },
        signed_by: customer_name.trim().into(),
        technician: technician.trim().into(),
        ts: now.to_rfc3339(),
    };
    let mut all = load_all();
    all.push(c);
    save_all(&all)?;
    Ok(id)
}

/// Kagua kama kazi inaruhusiwa kwa IMEI+service mahususi
pub fn verify(imei: &str, service_id: &str) -> Result<Consent> {
    let imei = imei.trim().replace(' ', "");
    let hit = load_all()
        .into_iter()
        .find(|c| c.imei == imei && c.service_id == service_id);
    match hit {
        Some(c) => Ok(c),
        None => bail!(
            "HAKUNA CONSENT: {service_id} kwa IMEI {imei}. \
             Kazi imezuiliwa (HITL). Rekodi consent kwanza: fundi-mobile consent add ..."
        ),
    }
}

/// Fomu ya kusaini (print/text) kwa mteja
pub fn form_template() -> &'static str {
    r#"┌──────────────────────────────────────────────────┐
│  FUNDI MOBILE — RUHUSA YA KAZI                   │
│                                                  │
│  Jina la Mteja: ___________________________     │
│  Namba ya Simu: ___________________________     │
│  Namba ya Kitambulisho: ___________________     │
│                                                  │
│  Simu:                                          │
│  Brand: ____________ Model: ____________        │
│  IMEI (*#06#): ______________________           │
│                                                  │
│  Kazi inayotakiwa:                              │
│  ☐ Reset Password   ☐ Bypass FRP                │
│  ☐ Flash Firmware   ☐ Update OS                 │
│  ☐ Backup Data      ☐ Nyingine: __________      │
│                                                  │
│  NAKIRI:                                        │
│  1. Simu hii ni yangu au nina ruhusa ya mmiliki.│
│  2. Fundi anaweza kufuta data yote kama itahitaji│
│  3. Fundi hahusiki na hasara baada ya kazi.     │
│  4. Ninalipa TZS ____________ kwa kazi hii.     │
│                                                  │
│  Sahihi: ________________  Tarehe: __________   │
│  Fundi:  ________________  Tarehe: __________   │
└──────────────────────────────────────────────────┘"#
}

pub fn list() -> Vec<Consent> {
    load_all()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn consent_store_roundtrip() {
        std::env::set_var(
            "FUNDI_DATA",
            std::env::temp_dir().join(format!("fm_test_{}", std::process::id())),
        );
        let id = record(
            "Fatuma Mwinyi",
            "+255712345678",
            Some("19990877"),
            "samsung",
            "A12",
            "354123456789012",
            "reset_password_recovery",
            true,
            30000,
            "Fundi wa Kwanza",
        )
        .unwrap();
        let c = verify("354123456789012", "reset_password_recovery").unwrap();
        assert_eq!(c.consent_id, id);
        assert!(c.destroys_data);
        // Bila consent → imezuiliwa
        assert!(verify("000000000000000", "reset_password_recovery").is_err());
    }
}
