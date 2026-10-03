//! drivers.rs — DRIVER CENTER: enum halisi + scan halisi + mwongozo sahihi wa update.
//!
//! Hakuna uongo: `pnputil /enum-drivers` na `pnputil /scan-devices` ni tools rasmi za Windows.
//! Update guidance: Windows Update optional + vendor tools rasmi TU (hakuna fake updaters).
//!
//! Amri: drivers list | drivers scan | drivers update

use anyhow::{bail, Result};

#[derive(Default)]
struct DriverEntry {
    published: String,
    original: String,
    provider: String,
    class: String,
    version: String,
}

fn parse_pnputil(text: &str) -> Vec<DriverEntry> {
    let mut out = Vec::new();
    let mut cur = DriverEntry::default();
    for line in text.lines() {
        let lower = line.to_lowercase();
        let grab = |l: &str, k: &str| -> Option<String> {
            l.to_lowercase().contains(k)
                .then(|| l.split_once(':').map(|(_, v)| v.trim().to_string()))
                .flatten()
        };
        if let Some(v) = grab(line, "published name") {
            if !cur.published.is_empty() {
                out.push(std::mem::take(&mut cur));
            }
            cur.published = v;
        } else if let Some(v) = grab(line, "original name") {
            cur.original = v;
        } else if let Some(v) = grab(line, "provider name") {
            cur.provider = v;
        } else if let Some(v) = grab(line, "class name") {
            cur.class = v;
        } else if let Some(v) = grab(line, "driver version") {
            // pnputil: "Driver Version: 31.0.15.3623" au tarehe tofauti kwenye versions mbaya
            cur.version = v;
        }
        let _ = lower;
    }
    if !cur.published.is_empty() {
        out.push(cur);
    }
    out
}

/// Orodha HALISI ya drivers za tatu (third-party) — pnputil, fallback driverquery.
pub fn list() -> Result<String> {
    let out = std::process::Command::new("pnputil").args(["/enum-drivers"]).output();
    match out {
        Ok(o) => {
            let text = String::from_utf8_lossy(&o.stdout).to_string();
            let drivers = parse_pnputil(&text);
            if drivers.is_empty() {
                return Ok("\n📦 pnputil: hakuna drivers za tatu zilizopangwa (driver pack za Windows tu).".into());
            }
            let mut counts: std::collections::BTreeMap<String, usize> = Default::default();
            for d in &drivers {
                *counts.entry(d.class.clone()).or_default() += 1;
            }
            let mut s = format!(
                "\n📦 DRIVERS ZA TATU: {} (pnputil /enum-drivers halisi)\n   Kwa class: ",
                drivers.len()
            );
            for (c, n) in &counts {
                s.push_str(&format!("{c}={n}  "));
            }
            s.push_str("\n\n");
            let show = drivers.len().min(80);
            for d in drivers.iter().take(show) {
                s.push_str(&format!(
                    "  {:<10} {:<22} {:<18} {:<14} {}\n",
                    d.published, d.original, d.provider, d.class, d.version
                ));
            }
            if drivers.len() > show {
                s.push_str(&format!("  ... na zingine {} (tazama yote kwa admin: pnputil /enum-drivers)\n", drivers.len() - show));
            }
            s.push_str("\n→ Update: fundi-mobile drivers update (mwongozo rasmi kwa kila vendor)");
            Ok(s)
        }
        Ok(_) => unreachable!(),
        Err(_) => {
            // fallback: driverquery (hata bila admin)
            match std::process::Command::new("driverquery").args(["/fo", "csv", "/nh"]).output() {
                Ok(o) => {
                    let text = String::from_utf8_lossy(&o.stdout).to_string();
                    let n = text.lines().filter(|l| !l.trim().is_empty()).count();
                    Ok(format!("\n📦 DRIVER MODULES: {n} (driverquery halisi; pnputil haikupatikana)\n→ Ukamilisha: fungua PowerShell kama ADMIN → pnputil /enum-drivers"))
                }
                Err(e) => bail!("pnputil na driverquery hazipatikani — hii inafanya kazi kwenye Windows tu: {e}"),
            }
        }
    }
}

/// Scan HALISI ya hardware mpya/badilika: pnputil /scan-devices (inahitaji ADMIN).
pub fn scan() -> Result<String> {
    let out = std::process::Command::new("pnputil").args(["/scan-devices"]).output();
    match out {
        Ok(o) => {
            let text = format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr));
            let text = text.trim().to_string();
            if text.is_empty() {
                return Ok("\n🔎 Scan imekamilika (hakuna ujumbe kutoka Windows). Angalia Device Manager kwa mabadiliko.".into());
            }
            Ok(format!("\n🔎 SCAN HALISI (pnputil /scan-devices):\n  {text}\n\n→ Device Manager: Win+X → Device Manager → vifaa vinavyotambulika hupatikana upya.\n→ Yellow ⚠ kwenye device? `fundi-mobile errors solve <code>` (mfano: Code 28, 43)"))
        }
        Err(e) => bail!("pnputil haipatikani — hii inafanya kazi kwenye Windows tu: {e}"),
    }
}

/// Mwongozo KAMILI na HALI wa update — vendors rasmi tu, hakuna fake driver boosters.
pub fn update() -> String {
    String::from(
        "\n🔧 DRIVER UPDATE — NJIA RASMI PEKEE (kwa utaratibu huu):\n\n  1) WINDOWS UPDATE (salama zaidi, kwanza kabisa):\n     Settings → Windows Update → Advanced options → Optional updates → Driver updates\n     → chagua zote zinazopatikana → Install → Restart.\n\n  2) TOOLS RASMI ZA VENDOR (auto-detect halisi):\n     • Dell     : Dell SupportAssist / dell.com/support (weka Service Tag)\n     • HP       : HP Support Assistant / hp.com/go/tesupport\n     • Lenovo   : Lenovo Vantage / support.lenovo.com\n     • ASUS     : MyASUS app / asus.com/support\n     • Acer     : Acer Care Center / acer.com/support\n     • Intel    : Intel Driver & Support Assistant (intel.com/dsa) — WiFi/BT/Gfx/Chipset\n     • NVIDIA   : NVIDIA App / nvidia.com/drivers (GPU)\n     • AMD      : AMD Software: Adrenalin Edition (amd.com/support)\n     • Realtek  : Audio/LAN — kutoka tovuti ya MOTHERBOARD/laptop vendor (si realtek moja kwa moja)\n\n  3) SAKINISHA MANUALI (kama auto imeshindwa):\n     • Download kutoka vendor → Device Manager → Right-click device → Update driver →\n       Browse my computer → chagua folder → Install.\n     • Kama driver mpya ni mbaya: Properties → Driver tab → Roll Back Driver.\n\n  4) OFFLINE PC (hakuna internet):\n     • Snappy Driver Installer Origin (sdio) — open source, download kwenye PC nyingine,\n       pakiti driverpacks, endesha offline. HALALI.\n\n  5) DRIVERS ZA SIMU KWA PC (kazi yetu ya kila siku):\n     • Android (adb/fastboot): Google USB Driver au driver ya brand (Samsung/Xiaomi n.k.)\n       + USB debugging ON. Test: fundi-mobile devices\n     • iPhone/iPad: sakinisha Apple Devices app (Microsoft Store) au iTunes — inaweka drivers.\n\n  6) USIFANYE HAYA:\n     • DriverBooster/'driver updater' za bahati nasibu kutoka tovuti zisizo rasmi (adware/malware).\n     • Kusakinisha driver ya chipset tofauti na yako (System Vision: fundi-mobile sysvision).\n     • GPU driver BILA kujua model kamili (sysvision inaonyesha).\n\n  Thibitisha baada ya update: fundi-mobile drivers list | fundi-mobile drivers scan",
    )
}
