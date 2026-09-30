//! licenses.rs — ACTIVATION ASSISTANT (HALALI 100%)
//!
//! Njia RASMI pekee: `slmgr.vbs` (Windows) na `OSPP.VBS` (Office) — tools rasmi za Microsoft.
//! HAKUNA: massgrave, MAS, KMS-cracks, HWID tricks, activators (ni PIRACY — zimekataliwa).
//! Kama haijaamilika → agent inaelekeza kununua leseni GENUINE (Microsoft Store / wauzaji rasmi).
//!
//! Amri: license status | office-status | activate | activate-office | genuine

use anyhow::{bail, Result};

fn slmgr_path() -> String {
    let root = std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".into());
    format!("{root}\\System32\\slmgr.vbs")
}

/// Endesha script ya VBS kwa cscript (Windows pekee). Hakuna uongo: kosa halisi linaonyeshwa.
fn run_vbs(script: &str, arg: &str) -> Result<String> {
    let out = std::process::Command::new("cscript")
        .args(["//nologo", script, arg])
        .output()
        .map_err(|e| anyhow::anyhow!("cscript haipatikani — hii inafanya kazi kwenye Windows tu: {e}"))?;
    let mut text = String::from_utf8_lossy(&out.stdout).to_string();
    let err = String::from_utf8_lossy(&out.stderr).to_string();
    if !err.trim().is_empty() {
        text.push('\n');
        text.push_str(&err);
    }
    if text.trim().is_empty() {
        bail!("cscript imerudisha tupu (exit {:?}) — fungua terminal kama ADMIN kisha jaribu tena", out.status.code());
    }
    Ok(text)
}

/// Pata thamani baada ya ':' kwenye mstari unaotaja key (kwa kutofautisha case).
fn license_line(text: &str, key: &str) -> Option<String> {
    for line in text.lines() {
        if line.to_lowercase().contains(&key.to_lowercase()) {
            if let Some((_, v)) = line.split_once(':') {
                return Some(v.trim().trim_matches('-').trim().to_string());
            }
        }
    }
    None
}

fn status_emoji(v: &str) -> &'static str {
    if v.eq_ignore_ascii_case("licensed") {
        "✅ IMEAMILIWA RASMI"
    } else if v.to_lowercase().contains("unlicensed") {
        "❌ HAIJAAMILIWA"
    } else if v.to_lowercase().contains("grace") {
        "⏳ GRACE PERIOD (muda unaisha)"
    } else if v.to_lowercase().contains("notification") || v.to_lowercase().contains("tolerance") {
        "⚠️ NOTIFICATION MODE (inahitaji activation)"
    } else {
        "❓ angalia ujumbe hapa chini"
    }
}

/// Windows activation status (slmgr /dli + /xpr) — halisi, hakuna uongo.
pub fn status() -> Result<String> {
    let slmgr = slmgr_path();
    let dli = run_vbs(&slmgr, "/dli")?;
    let xpr = run_vbs(&slmgr, "/xpr").unwrap_or_default();
    let edition = license_line(&dli, "Name").unwrap_or_else(|| "?".into());
    let partial = license_line(&dli, "Partial Product Key").unwrap_or_else(|| "(hakuna — haijawahi kuamilishwa)".into());
    let lic = license_line(&dli, "License Status").unwrap_or_else(|| "?".into());
    let mut out = format!(
        "\n🪪 WINDOWS ACTIVATION STATUS (slmgr.vbs — rasmi Microsoft)\n  Edition        : {edition}\n  Partial Key    : {partial}\n  License Status : {lic} → {}\n",
        status_emoji(&lic)
    );
    let xpr_clean = xpr.lines().find(|l| !l.trim().is_empty()).unwrap_or("").trim();
    if !xpr_clean.is_empty() {
        out.push_str(&format!("  Expiry (/xpr)  : {xpr_clean}\n"));
    }
    if !lic.eq_ignore_ascii_case("licensed") {
        out.push_str("\n→ HAIJAAMILIWA: endelea na `fundi-mobile license genuine` (nunua key HALALI).\n");
        out.push_str("⚠️ massgrave/MAS/activators ni PIRACY — hazitumiki kwenye mfumo huu.\n");
    }
    Ok(out)
}

/// Tafuta OSPP.VBS (Office Click-to-Run na MSI, x64 + x86).
fn ospp_path() -> Option<String> {
    let pf = std::env::var("ProgramFiles").unwrap_or_else(|_| "C:\\Program Files".into());
    let pf86 = std::env::var("ProgramFiles(x86)").unwrap_or_else(|_| "C:\\Program Files (x86)".into());
    let mut cands = Vec::new();
    for root in [pf, pf86] {
        for v in ["Office16", "Office15", "Office14"] {
            cands.push(format!("{root}\\Microsoft Office\\{v}\\OSPP.VBS"));
        }
    }
    cands.into_iter().find(|p| std::path::Path::new(p).exists())
}

/// Office activation status (OSPP /dstatus) — halisi.
pub fn office_status() -> Result<String> {
    let Some(ospp) = ospp_path() else {
        bail!(
            "OSPP.VBS haipatikani — Office haijasakinishwa (au ni toleo la Microsoft Store). \
             Thibitisha: Settings → Apps → Installed apps."
        );
    };
    let dstatus = run_vbs(&ospp, "/dstatus")?;
    let lic = license_line(&dstatus, "LICENSE STATUS").unwrap_or_else(|| "?".into());
    let key = license_line(&dstatus, "Last 5 characters").unwrap_or_else(|| "(hakuna key)".into());
    let mut out = format!(
        "\n📊 OFFICE ACTIVATION STATUS (OSPP.VBS — rasmi Microsoft)\n  Script   : {ospp}\n  Key      : inaishia {key}\n  Status   : {lic} → {}\n",
        status_emoji(&lic)
    );
    if let Some(errc) = license_line(&dstatus, "ERROR CODE") {
        out.push_str(&format!("  Error    : {errc} (solve: fundi-mobile errors solve <code>)\n"));
    }
    if !lic.to_lowercase().contains("licensed") {
        out.push_str("\n→ HAIJAAMILIWA: `fundi-mobile license activate-office` (admin) au nunua genuine.\n");
    }
    Ok(out)
}

/// Activation RASMI ya Windows: slmgr /ato (inahitaji ADMIN + internet + key halali).
pub fn activate() -> Result<String> {
    let slmgr = slmgr_path();
    let dli = run_vbs(&slmgr, "/dli")?;
    let lic = license_line(&dli, "License Status").unwrap_or_default();
    if lic.eq_ignore_ascii_case("licensed") {
        return Ok("✅ Windows tayari imeamilishwa rasmi — hakuna haja ya kufanya kitu.".into());
    }
    let out = run_vbs(&slmgr, "/ato")?;
    let success = out.to_lowercase().contains("successfully");
    if success {
        return Ok("\n✅ IMEAMILIWA RASMI (slmgr /ato) — license yako halali imethibitishwa na Microsoft.".into());
    }
    let hint = if out.contains("0x80070005") {
        "Fungua terminal kama ADMIN kisha jaribu tena."
    } else if out.contains("0xC004F074") || out.contains("0xC004C008") {
        "Key imetumika kwenye mashine nyingine au KMS server si ya Microsoft — angalia key yako."
    } else if out.contains("0x8007007B") || out.contains("0xC004F210") {
        "Key si sahihi kwa edition hii — thibitisha umenunua key ya edition sahihi."
    } else {
        "Angalia internet, saa/tarehe za PC, kisha jaribu tena."
    };
    Ok(format!(
        "\n❌ Activation haijakamilika (slmgr /ato):\n  {out}\n→ {hint}\n→ Kama huna key halali: fundi-mobile license genuine\n⚠️ Cracks/KMS za piracy ZIMEKATALIWA — hazitatua tatizo, zinaleta malware + ukiukaji wa sheria."
    ))
}

/// Activation RASMI ya Office: OSPP /act (inahitaji ADMIN + internet + key halali).
pub fn activate_office() -> Result<String> {
    let Some(ospp) = ospp_path() else {
        bail!("OSPP.VBS haipatikani — Office haijasakinishwa au ni toleo la Microsoft Store.");
    };
    let out = run_vbs(&ospp, "/act")?;
    let success = out.to_lowercase().contains("successful");
    if success {
        return Ok("\n✅ OFFICE IMEAMILIWA RASMI (OSPP /act).".into());
    }
    Ok(format!(
        "\n❌ Office activation haijakamilika (OSPP /act):\n  {out}\n→ Angalia internet + fungua terminal kama ADMIN.\n→ Kama huna license: Microsoft 365 subscription au key ya kudumu (tazama `license genuine`)."
    ))
}

/// Mwongozo wa kununua leseni GENUINE — njia pekee halali ya kuamilisha milele.
pub fn genuine() -> String {
    String::from(
        "\n🛒 NUNUA LESENI GENUINE (njia pekee HALALI):\n\n  WINDOWS:\n   1. PC mpya: nunua na Windows iliyosakinishwa rasmi (OEM license).\n   2. Microsoft Store kwenye PC: Settings → System → Activation → Open Store.\n   3. Tovuti: microsoft.com/store → Windows 11 Home/Pro (digital license).\n   4. Wauzaji walioruhusiwa nchini: maduka makubwa ya kompyuta + resellers rasmi.\n   5. Baada ya kununua: Settings → Activation → Change product key → weka key.\n\n  OFFICE:\n   1. Microsoft 365 (subscription): microsoft.com/microsoft-365 — ina Outlook, Word, Excel n.k.\n   2. Office Home & Student / Professional (key ya kudumu) — Microsoft Store au maduka rasmi.\n   3. BURE kabisa: office.com kwa browser (Word/Excel/PowerPoint online) + LibreOffice.\n\n  ⚠️ USDOMO:\n   • Keys 'za bei ndogo' kutoka tovuti zisizo rasmi = mara nyingi volume keys zilizoibiwa\n     — Microsoft inaweza kuzizima (deactivated) baada ya muda.\n   • Cracks/activators (massgrave n.k.) = PIRACY + malware risk + ukiukaji wa sheria.\n   • Digital license inafunga kwa akaunti yako ya Microsoft — ichunga kama hardware.\n",
    )
}

// ---------- APPLY KEY GENUINE (njia rasmi: /ipk + /ato, /inpkey + /act) ----------

fn key_valid(k: &str) -> bool {
    let parts: Vec<&str> = k.trim().split('-').collect();
    parts.len() == 5 && parts.iter().all(|p| p.len() == 5 && p.chars().all(|c| c.is_ascii_alphanumeric()))
}

fn key_masked(k: &str) -> String {
    // Tunaficha sehemu kuu — mmiliki anajua key yake; log isibeuze siri
    let parts: Vec<&str> = k.split('-').collect();
    if parts.len() != 5 {
        return "?????".into();
    }
    format!("{}-{}-XXXXX-XXXXX-{}", parts[0], parts[1], parts[4])
}

/// Kusakinisha key GENUINE rasmi (HITL: lazima uthibitisho, isipokuwa assume_yes).
pub fn apply(win_key: Option<&str>, office_key: Option<&str>, assume_yes: bool) -> Result<String> {
    let mut out = String::from("\n🔑 APPLY GENUINE KEY (njia rasmi ya Microsoft)\n");
    if let Some(k) = win_key {
        if !key_valid(k) {
            bail!("Windows key si sahihi — muundo: XXXXX-XXXXX-XXXXX-XXXXX-XXXXX");
        }
        out.push_str(&format!("  Windows key : {}\n", key_masked(k)));
    }
    if let Some(k) = office_key {
        if !key_valid(k) {
            bail!("Office key si sahihi — muundo: XXXXX-XXXXX-XXXXX-XXXXX-XXXXX");
        }
        out.push_str(&format!("  Office key  : {}\n", key_masked(k)));
    }
    if win_key.is_none() && office_key.is_none() {
        bail!("Toa key: license apply --win KEY --office KEY");
    }
    if !assume_yes {
        print!("\n  Thibitisha ku-install keys hizi RASMI? (y/ndiyo): ");
        use std::io::Write;
        let _ = std::io::stdout().flush();
        let mut l = String::new();
        let _ = std::io::stdin().read_line(&mut l);
        if !matches!(l.trim().to_lowercase().as_str(), "y" | "yes" | "ndiyo") {
            return Ok("Imesitishwa — hakuna kilichobadilishwa.".into());
        }
    }
    let _ = crate::govagent::speak_sw("Ninasakinisha keys rasmi. Subiri kidogo.");
    if let Some(k) = win_key {
        let slmgr = slmgr_path();
        let ipk = run_vbs(&slmgr, &format!("/ipk {k}"))?;
        out.push_str(&format!("\n  Windows /ipk : {}\n", ipk.lines().find(|l| !l.trim().is_empty()).unwrap_or("ok").trim()));
        match activate() {
            Ok(a) => out.push_str(&format!("  Windows /ato : {}\n", a.trim())),
            Err(e) => out.push_str(&format!("  Windows /ato : ❌ {e}\n")),
        }
    }
    if let Some(k) = office_key {
        if let Some(ospp) = ospp_path() {
            let inp = run_vbs(&ospp, &format!("/inpkey:{k}"))?;
            out.push_str(&format!("\n  Office /inpkey : {}\n", inp.lines().find(|l| !l.trim().is_empty()).unwrap_or("ok").trim()));
            match activate_office() {
                Ok(a) => out.push_str(&format!("  Office /act    : {}\n", a.trim())),
                Err(e) => out.push_str(&format!("  Office /act    : ❌ {e}\n")),
            }
        } else {
            out.push_str("\n  Office       : OSPP.VBS haipo — Office haijasakinishwa au ni toleo la Store\n");
        }
    }
    out.push_str("\n✅ Malizo — thibitisha: license status + license office-status\n⚠️ Keys ni za mmiliki pekee; usishiriki (key moja = PC moja kwa kawaida).\n");
    Ok(out)
}

// ---------- ASSIST AGENTIC (intake: agent anaongoza mteja mwanzo hadi mwisho) ----------

pub fn assist() -> Result<()> {
    let _ = crate::govagent::speak_sw("Karibu kwenye activation assistant. Nakagua hali ya Windows na Office kwanza.");
    println!("\n🔓 ACTIVATION ASSISTANT — agent inakuongoza (rasmi tu; hakuna cracks)");
    println!("-------------------------------------------------------------");
    match status() {
        Ok(s) => println!("{s}"),
        Err(e) => println!("  Windows status: {e}"),
    }
    match office_status() {
        Ok(s) => println!("{s}"),
        Err(e) => println!("  Office status: {e}"),
    }
    print!("\n  Una product key GENUINE unayotaka ku-install sasa? (y/ndiyo): ");
    use std::io::Write;
    let _ = std::io::stdout().flush();
    let mut l = String::new();
    let _ = std::io::stdin().read_line(&mut l);
    if matches!(l.trim().to_lowercase().as_str(), "y" | "yes" | "ndiyo") {
        print!("  Windows key (Enter kama hakuna): ");
        let _ = std::io::stdout().flush();
        let mut wk = String::new();
        let _ = std::io::stdin().read_line(&mut wk);
        print!("  Office key (Enter kama hakuna): ");
        let _ = std::io::stdout().flush();
        let mut ok = String::new();
        let _ = std::io::stdin().read_line(&mut ok);
        let wk = wk.trim();
        let okk = ok.trim();
        let res = apply(
            if wk.is_empty() { None } else { Some(wk) },
            if okk.is_empty() { None } else { Some(okk) },
            true, // tayari amethibitisha kwenye intake
        )?;
        println!("{res}");
    } else {
        println!("{}", genuine());
        let _ = crate::govagent::speak_sw("Nunua leseni halali kutoka Microsoft Store au maduka rasmi. Cracks ni hatari na ni kinyume cha sheria.");
    }
    Ok(())
}
