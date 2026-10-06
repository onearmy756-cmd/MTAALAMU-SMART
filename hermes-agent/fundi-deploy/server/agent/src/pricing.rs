//! pricing.rs — BEI PER-COMPUTER + PACKAGES (pay-per-use / subscription) — MSIMBO WA RUST.
//!
//! Mahitaji ya mmiliki (SEHEMU 2 Hatua 5 + SEHEMU 8.2/8.3):
//!   - Kabla ya install: mfumo unakokotoa bei kulingana na computer (OS + apps)
//!   - Mtu analipa (ClickPesa ✅ halisi / Bank / M-Pesa) ndipo mfumo uji-activate
//!   - Packages: pay-per-use (kila PC) na subscription (mwezi) — kila moja na bei yake
//!
//! KANUNI: hesabu yote Rust (deterministic). ClickPesa API ni malipo halisi
//! (hermes-agent/mtaalamu/clickpesa.py); hii ni pricing engine ya Rust inayokokotoa
//! kiasi (TZS) kabla ya ombi la malipo kutumwa.

use serde::Serialize;

// ---------- PACKAGES (enum — Rust ndiyo kanuni) ----------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Plan {
    PayPerUse,  // kila PC inalipa yake
    Subscription, // mwezi — PCs zote za mteja
}

impl Plan {
    pub fn id(&self) -> &'static str {
        match self {
            Plan::PayPerUse => "pay_per_use",
            Plan::Subscription => "subscription",
        }
    }

    pub fn name_sw(&self) -> &'static str {
        match self {
            Plan::PayPerUse => "Lipa kwa matumizi (kila PC)",
            Plan::Subscription => "Usajili (kwa mwezi)",
        }
    }

    pub fn from_id(id: &str) -> Option<Plan> {
        match id {
            "pay_per_use" | "pay-per-use" => Some(Plan::PayPerUse),
            "subscription" | "monthly" => Some(Plan::Subscription),
            _ => None,
        }
    }
}

// ---------- BEI ZA MSINGI (TZS) — Rust constants, si JSON ----------

/// Bei ya msingi ya OS install kwa PC moja (kazi halisi ya agent).
fn base_os_price(os: &str) -> u64 {
    match os {
        "win11" => 25_000,
        "win10" => 20_000,
        "windows-server" => 60_000,
        "ubuntu" | "debian" | "fedora" => 10_000,
        "kali" => 15_000,
        _ => 15_000,
    }
}

/// Bei kwa kila app (critical inagharimu zaidi — setup + configuration kamili).
fn app_price(critical: bool) -> u64 {
    if critical { 5_000 } else { 2_000 }
}

/// Punguzo la batch (PC nyingi kwa wakati mmoja): 10+ → 10% off, 50+ → 20% off.
fn batch_discount(pcs: usize) -> f64 {
    if pcs >= 50 { 0.20 } else if pcs >= 10 { 0.10 } else { 0.0 }
}

/// Subscription ya mwezi: bei ya msingi ya mwezi (inashughulikia maintenance + support).
const SUBSCRIPTION_MONTHLY_TZS: u64 = 150_000;

// ---------- QUOTE (quote ya malipo) ----------

#[derive(Debug, Clone, Serialize)]
pub struct Quote {
    pub plan: String,
    pub pcs: usize,
    pub os: String,
    pub apps_count: usize,
    pub critical_apps: usize,
    pub currency: String,
    pub subtotal_tzs: u64,
    pub discount_pct: u64,
    pub total_tzs: u64,
    pub monthly_tzs: Option<u64>,
    pub note_sw: String,
}

/// Kokotoa bei kwa deployment: PCs + OS + apps (SEHEMU 2 Hatua 5).
/// Pay-per-use: (bei_OS + apps) × PCs − punguzo la batch.
/// Subscription: malipo ya mwezi (PCs 1–10 zimejumuishwa; kila 10 ziada = +50%).
pub fn quote(plan: Plan, pcs: usize, os: &str, critical_apps: usize, apps_count: usize) -> Quote {
    let pcs = pcs.max(1);
    let (subtotal, discount_pct, monthly) = match plan {
        Plan::PayPerUse => {
            let apps_total: u64 = (critical_apps as u64) * app_price(true)
                + ((apps_count.saturating_sub(critical_apps) as u64) * app_price(false));
            let sub = (base_os_price(os) + apps_total) * pcs as u64;
            (sub, (batch_discount(pcs) * 100.0) as u64, None)
        }
        Plan::Subscription => {
            let tiers = ((pcs as f64) / 10.0).ceil().max(1.0) as u64;
            (SUBSCRIPTION_MONTHLY_TZS * tiers, 0, Some(SUBSCRIPTION_MONTHLY_TZS * tiers))
        }
    };
    // Punguzo la batch linatumika kwa PayPerUse tu (subscription lina tiers zake)
    let total = match plan {
        Plan::PayPerUse => ((subtotal as f64) * (1.0 - batch_discount(pcs))) as u64,
        Plan::Subscription => subtotal,
    };
    Quote {
        plan: plan.id().into(),
        pcs,
        os: os.into(),
        apps_count,
        critical_apps,
        currency: "TZS".into(),
        subtotal_tzs: subtotal,
        discount_pct,
        total_tzs: total,
        monthly_tzs: monthly,
        note_sw: match plan {
            Plan::PayPerUse => format!(
                "Lipa TZS {total} (ClickPesa/Bank/M-Pesa) — mfumo unaji-activate kisha agents {pcs} zinaanza kazi."
            ),
            Plan::Subscription => format!(
                "Usajili wa mwezi TZS {total} — PCs {pcs} zimejumuishwa; agents zinaanza kazi baada ya malipo."
            ),
        },
    }
}

/// Thibitisha malipo yamekamilika → ruhusa ya activate (HITL ya pesa).
/// ClickPesa halisi inathibitisha SUCCESS (clickpesa.py); hii ni gate ya Rust.
pub fn activation_allowed(payment_status: &str) -> bool {
    matches!(payment_status, "SUCCESS" | "SETTLED" | "paid")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bei_ya_pc_mmoja_win11_na_apps() {
        let q = quote(Plan::PayPerUse, 1, "win11", 2, 4);
        // OS 25000 + critical 2×5000 + non-critical 2×2000 = 39000
        assert_eq!(q.subtotal_tzs, 39_000);
        assert_eq!(q.total_tzs, 39_000);
        assert_eq!(q.discount_pct, 0);
        assert_eq!(q.currency, "TZS");
    }

    #[test]
    fn punguzo_la_batch_10_pcs() {
        let q = quote(Plan::PayPerUse, 10, "ubuntu", 0, 0);
        // (10000)×10 = 100000 → -10% = 90000
        assert_eq!(q.subtotal_tzs, 100_000);
        assert_eq!(q.discount_pct, 10);
        assert_eq!(q.total_tzs, 90_000);
    }

    #[test]
    fn punguzo_la_batch_50_pcs() {
        let q = quote(Plan::PayPerUse, 50, "ubuntu", 0, 0);
        assert_eq!(q.discount_pct, 20);
        assert_eq!(q.total_tzs, 400_000); // 500000 − 20%
    }

    #[test]
    fn subscription_kwa_mwezi() {
        let q = quote(Plan::Subscription, 10, "win11", 0, 0);
        assert_eq!(q.monthly_tzs, Some(150_000));
        assert_eq!(q.total_tzs, 150_000);
        let q2 = quote(Plan::Subscription, 25, "win11", 0, 0); // ceil(25/10)=3 tiers
        assert_eq!(q2.total_tzs, 450_000);
    }

    #[test]
    fn activation_inahitaji_malipo_halisi() {
        assert!(activation_allowed("SUCCESS"));
        assert!(activation_allowed("SETTLED"));
        assert!(!activation_allowed("PROCESSING"));
        assert!(!activation_allowed("FAILED"));
        assert!(!activation_allowed(""));
    }

    #[test]
    fn bei_za_os_zinatofautiana() {
        assert!(base_os_price("windows-server") > base_os_price("win11"));
        assert!(base_os_price("win11") > base_os_price("ubuntu"));
    }
}
