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

// ---------- BEI ZA SOKO HALISI (TZS) — Rust constants, si JSON ----------
// KANUNI ZA BEI (faida halisi kwa fundi + mmiliki):
//   1. Ugumu wa kazi unaamua bei ya msingi (si kila kazi ni sawa)
//   2. Utumiaji wa computer (RSS/enterprise) unaongeza thamani ya subscription
//   3. Punguzo la batch: mnufaika anayelipa PCs nyingi kwa wakati mmoja
//   4. Bei za soko TZS Tanzania (IT service pricing): OS install 30-60k,
//      per-app 3-10k, drivers 10-25k, scan 5k, repair 20-50k — sisi tuna weka
//      katikati ya soko: fundi apate 60%, mmiliki 30%, reserve 10% (margin halisi)

/// Mgawanyo wa faida (halisi): fundi anayefanya kazi, mmiliki wa mfumo, reserve
pub fn revenue_split(total: u64) -> (u64, u64, u64) {
    let fundi = total * 60 / 100;
    let mmiliki = total * 30 / 100;
    let reserve = total - fundi - mmiliki;
    (fundi, mmiliki, reserve)
}

/// Bei ya msingi ya OS install kwa PC moja — kwa UGUMU (halisi ya soko):
/// Windows Server ndiyo ngumu zaidi (AD/licensing), Win11 (TPM/drivers),
/// Linux nyepesi (bure, config kidogo), Kali (tools nyingi).
fn base_os_price(os: &str) -> u64 {
    match os {
        "windows-server" => 90_000,        // ugumu mkubwa: AD, licensing, roles
        "win11" => 35_000,                 // TPM 2.0, drivers, activation
        "win10" => 28_000,                 // rahisi kidogo kuliko win11
        "kali" => 30_000,                  // tools za cyber + partitioning
        "ubuntu" | "fedora" => 20_000,     // bure, installer mzuri
        "debian" => 22_000,                // server config kidogo zaidi
        _ => 30_000,                       // default (OS nyingine)
    }
}

/// Bei kwa app — kwa UGUMU wa setup (critical = configuration kamili + testing):
fn app_price(critical: bool, os: &str) -> u64 {
    let windows = os.starts_with("win");
    match (critical, windows) {
        // Microsoft Office/Antivirus kwa Windows: licensing + activation ngumu
        (true, true) => 8_000,
        // Critical za Linux (build-essential, docker): repos + config
        (true, false) => 5_000,
        // Non-critical (VLC, 7zip): winget/apt moja kwa moja
        (false, true) => 4_000,
        (false, false) => 3_000,
    }
}

/// Bei ya DRIVERS per-PC (brand): Dell/HP/Lenovo (enterprise tools) ni ngumu
/// kuliko Acer/Asus — soko: 15-30k kwa full driver pack + testing.
fn drivers_price(brand: &str) -> u64 {
    match brand {
        "dell" => 25_000,    // Dell Command Update + BIOS
        "hp" => 25_000,      // HP Support Assistant + BIOS
        "lenovo" => 25_000,  // Lenovo Vantage + BIOS
        "acer" => 18_000,
        "asus" => 18_000,
        _ => 20_000,
    }
}

/// Bei ya scan/diagnosis (low-risk, automatic): 5,000 TZS kwa PC — ndiyo
/// inayolipa ReAct scan + ripoti ya kila siku.
pub const SCAN_PRICE_TZS: u64 = 5_000;

/// Bei ya repair/fix ya tatizo lililogunduliwa (High-risk, HITL): soko 20-50k.
pub const REPAIR_PRICE_TZS: u64 = 30_000;

/// Punguzo la batch (PC nyingi kwa wakati mmoja): 10+ → 10% off, 50+ → 20% off.
/// Faida ya mzzi: agent 100 zinafanya kazi kwa wakati mmoja — gharama ya fundi
/// haiongezeki, kwa hiyo punguzo ni halisi (economies of scale).
fn batch_discount(pcs: usize) -> f64 {
    if pcs >= 50 { 0.20 } else if pcs >= 10 { 0.10 } else { 0.0 }
}

/// SUBSCRIPTION (kwa mwezi) — kwa UTUMIAJI wa computer:
/// - Basic (PC 1-10): monitoring + scan ya kila siku + ripoti
/// - Standard (PC 11-50): + remote fixing + drivers
/// - Enterprise (PC 50+): + OS deployment + cyber (Kali) + SLA 24h
/// Bei zinaongozwa na utumiaji halisi (kila PC = monitoring node ya agent).
pub fn subscription_price_tzs(pcs: usize) -> u64 {
    let per_pc_monthly = 15_000u64; // monitoring + scan + ripoti za kila siku
    let n = pcs.max(1) as u64;
    if n <= 10 {
        per_pc_monthly * n
    } else if n <= 50 {
        // Punguzo la kati: 12,000/PC
        12_000 * n
    } else {
        // Enterprise: 10,000/PC (volume)
        10_000 * n
    }
}

const SUBSCRIPTION_MONTHLY_TZS: u64 = 150_000; // Basic 10 PCs

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
            let apps_total: u64 = (critical_apps as u64) * app_price(true, os)
                + ((apps_count.saturating_sub(critical_apps) as u64) * app_price(false, os));
            let sub = (base_os_price(os) + apps_total) * pcs as u64;
            (sub, (batch_discount(pcs) * 100.0) as u64, None)
        }
        Plan::Subscription => {
            // Subscription kwa utumiaji halisi: per-PC pricing na tiers
            let monthly = subscription_price_tzs(pcs);
            (monthly, 0, Some(monthly))
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
        // OS 35000 + critical 2×8000 (win) + non-critical 2×4000 (win) = 59000
        assert_eq!(q.subtotal_tzs, 59_000);
        assert_eq!(q.total_tzs, 59_000);
        assert_eq!(q.discount_pct, 0);
        assert_eq!(q.currency, "TZS");
    }

    #[test]
    fn punguzo_la_batch_10_pcs() {
        let q = quote(Plan::PayPerUse, 10, "ubuntu", 0, 0);
        // (20000)×10 = 200000 → -10% = 180000
        assert_eq!(q.subtotal_tzs, 200_000);
        assert_eq!(q.discount_pct, 10);
        assert_eq!(q.total_tzs, 180_000);
    }

    #[test]
    fn punguzo_la_batch_50_pcs() {
        let q = quote(Plan::PayPerUse, 50, "ubuntu", 0, 0);
        assert_eq!(q.discount_pct, 20);
        assert_eq!(q.total_tzs, 800_000); // 1000000 − 20%
    }

    #[test]
    fn subscription_kwa_mwezi() {
        // Basic: 10 PCs × 15000 = 150000
        let q = quote(Plan::Subscription, 10, "win11", 0, 0);
        assert_eq!(q.monthly_tzs, Some(150_000));
        assert_eq!(q.total_tzs, 150_000);
        // Standard: 25 PCs × 12000 = 300000
        let q2 = quote(Plan::Subscription, 25, "win11", 0, 0);
        assert_eq!(q2.total_tzs, 300_000);
        // Enterprise: 100 PCs × 10000 = 1000000
        let q3 = quote(Plan::Subscription, 100, "win11", 0, 0);
        assert_eq!(q3.total_tzs, 1_000_000);
    }

    #[test]
    fn bei_za_ugumu_halisi() {
        // Server ngumu zaidi kuliko desktop
        assert!(base_os_price("windows-server") > base_os_price("win11"));
        assert!(base_os_price("win11") > base_os_price("ubuntu"));
        // Apps za Windows (licensing) ghali kuliko Linux
        assert!(app_price(true, "win11") > app_price(true, "ubuntu"));
        // Enterprise tools za brand kubwa
        assert!(drivers_price("dell") > drivers_price("asus"));
        // Scan na repair: repair ni kubwa (High-risk, HITL)
        assert!(REPAIR_PRICE_TZS > SCAN_PRICE_TZS);
        // Faida halisi: fundi 60%, mmiliki 30%, reserve 10%
        let (fundi, mmiliki, reserve) = revenue_split(100_000);
        assert_eq!((fundi, mmiliki, reserve), (60_000, 30_000, 10_000));
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
