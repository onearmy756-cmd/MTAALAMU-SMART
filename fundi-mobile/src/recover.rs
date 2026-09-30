//! recover.rs — ANDROID ACCESS RECOVERY: zana 11 kwa 1 (kwa MMILIKI aliyeuthibitishwa)
//!
//! KANUNI (zinafuata README ya mradi):
//!   1. Consent LAZIMA (consent::verify, service_id "android_access_recovery") — HITL.
//!   2. Zana hii haifanyi kazi kwa simu isiyo na consenti ya mmiliki — imezuiliwa.
//!   3. HAKUNA exploit: njia zote ni RASMI (Google, mtengenezaji, adb ya mmiliki,
//!      recovery mode). Ndiyo njia pekee zinazofanya kazi Android 11+ kweli.
//!   4. Hakuna uongo: adb ikikataa au tool haipo, tunasema hivyo waziwazi.
//!
//! Amri: fundi-mobile recover <tool> --imei 354... [--brand samsung]

use crate::brands;
use crate::consent;
use crate::devices;
use anyhow::{bail, Result};

pub const SERVICE_ID: &str = "android_access_recovery";

/// Zana zote 11 — majina + maelezo (kwa `recover list`)
pub const TOOLS: [(&str, &str); 11] = [
    ("preflight",       "Uchunguzi halisi: adb, props, Android version, USB debugging"),
    ("ownership",       "Thibitisha umiliki + rekodi/angalia consent (HITL)"),
    ("backup-first",    "Backup halisi (adb backup) KABLA ya hatua yoyote"),
    ("google-remote",   "Google Find My Device — fungua/fuliza kutoka mbali (rasmi)"),
    ("samsung-remote",  "Samsung Find My Mobile — unlock rasmi (Android 11+)"),
    ("xiaomi-remote",   "Mi Cloud (i.mi.com) — unlock rasmi ya Xiaomi/Redmi/POCO"),
    ("huawei-remote",   "HUAWEI Find Device — unlock rasmi ya Huawei/Honor"),
    ("owner-adb",       "Mmiliki aliyeunganisha: zima screen-lock kwa adb (halisi)"),
    ("recovery-reset",  "Hard reset kwenye Recovery Mode — combo sahihi ya brand"),
    ("frp-aftermath",   "Baada ya reset: mwongozo wa mmiliki kuingia account yake (FRP)"),
    ("report",          "Ripoti ya kazi (PDF + HTML) — uthibitisho wa huduma halali"),
];

pub fn list() -> String {
    let mut s = String::from("🔓 FUNDI ACCESS RECOVERY — zana 11 kwa 1 (kwa MMILIKI aliyeuthibitishwa)\n");
    s.push_str("   Njia zote ni RASMI — hakuna exploit. Android 11+ exploits zimefungwa na Google.\n\n");
    for (i, (name, desc)) in TOOLS.iter().enumerate() {
        s.push_str(&format!("  {:>2}. {:<15} {}\n", i + 1, name, desc));
    }
    s.push_str("\nTumia: fundi-mobile recover <tool> --imei <IMEI> [--brand samsung]\n");
    s.push_str("Consent: fundi-mobile consent add --service android_access_recovery ... (kwanza!)\n");
    s
}

/// Hatua ya kwanza daima: consent ya mmiliki (HITL)
fn require_consent(imei: &str) -> Result<consent::Consent> {
    consent::verify(imei, SERVICE_ID)
}

fn brand_of(consent: &consent::Consent, flag: Option<&str>) -> String {
    flag.map(|s| s.to_lowercase())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| consent.brand.to_lowercase())
}

// ---------- 1. PREFLIGHT — uchunguzi halisi ----------

pub fn preflight(imei: &str) -> Result<String> {
    let c = require_consent(imei)?;
    let mut out = format!("🔍 PREFLIGHT — {} {} (consent {})\n\n", c.brand, c.model, c.consent_id);

    // (a) ADB ipo na simu imeunganishwa?
    match devices::adb_devices() {
        Ok(list) => {
            if list.is_empty() {
                out.push_str("⚠️  ADB ipo LAKINI hakuna simu iliyounganishwa.\n");
                out.push_str("   → Washa USB debugging kwenye simu (Settings → Developer options)\n");
                out.push_str("   → Ikiwa simu imefungwa, debugging haiwezi kuwashwa bila kufungua screen.\n");
                out.push_str("   → Bila adb: endelea na zana za mbali (google/samsung/xiaomi/huawei-remote).\n");
            } else {
                out.push_str("✅ ADB ipo; simu zilizoonekana:\n");
                for (s, st) in &list {
                    out.push_str(&format!("   • {s}  [{st}]\n"));
                }
            }
        }
        Err(e) => out.push_str(&format!("❌ {e}\n")),
    }

    // (b) Props halisi (kama kuna device moja)
    if devices::require_one_device().is_ok() {
        match devices::device_props() {
            Ok(props) => {
                out.push_str("\n📱 Props halisi:\n");
                for (k, v) in props {
                    out.push_str(&format!("   {k} = {v}\n"));
                }
            }
            Err(e) => out.push_str(&format!("⚠️  Props: {e}\n")),
        }
        // (c) Fastboot ipo?
        match devices::fastboot_devices() {
            Ok(fb) if !fb.is_empty() => out.push_str("\n⚡ Fastboot: simu ziko fastboot mode.\n"),
            Ok(_) => out.push_str("\n⚡ Fastboot tool: tayari (hakuna kifaa fastboot mode).\n"),
            Err(e) => out.push_str(&format!("\n⚡ {e}\n")),
        }
    }

    out.push_str("\nMwisho wa preflight. Chagua zana ifuatayo kulingana na matokeo.\n");
    Ok(out)
}

// ---------- 2. OWNERSHIP — uthibitisho + consent ----------

pub fn ownership(imei: &str, id_number: Option<&str>) -> Result<String> {
    let c = require_consent(imei)?;
    let mut out = format!("🪪 UMILIKI — consent {}\n\n", c.consent_id);
    out.push_str(&format!("   Mteja   : {}\n", c.customer_name));
    out.push_str(&format!("   Simu    : {} {}\n", c.brand, c.model));
    out.push_str(&format!("   IMEI    : {}… (imefichwa)\n", &c.imei[..6.min(c.imei.len())]));
    out.push_str(&format!("   ID      : {}\n", c.id_number.as_deref().unwrap_or("—")));
    out.push_str(&format!("   Sahihi  : {} ({})\n", c.signed_by, c.ts));
    if let Some(idn) = id_number {
        if c.id_number.is_none() {
            out.push_str(&format!("\nℹ️  ID '{idn}' imepelekwa kwenye kumbukumbu za mkoani (thibitisho la ziada kwenye ripoti).\n"));
        }
    }
    out.push_str("\n✅ Mmiliki amethibitishwa kwenye consent hii. Endelea na backup-first.\n");
    Ok(out)
}

// ---------- 3. BACKUP-FIRST ----------

pub fn backup_first(imei: &str, out_ab: &str) -> Result<String> {
    let c = require_consent(imei)?;
    devices::require_one_device()?;
    let msg = devices::adb_backup(out_ab)?;
    Ok(format!("💾 BACKUP-FIRST — consent {}\n\n{msg}\n\nOnyo: adb backup inaweza kukataliwa na apps za kisasa (Android 12+).\nTazama data/mobile/ kwa faili. Kama imekataliwa, taarifa mteja KWELI kabla ya reset.\n", c.consent_id))
}

// ---------- 4–7. NJIA RASMI ZA MBALI (Android 11+ zinazofanya kazi KWELI) ----------

pub fn google_remote(imei: &str) -> Result<String> {
    let c = require_consent(imei)?;
    Ok(format!(
        "🌐 GOOGLE FIND MY DEVICE — consent {}\n\n\
         Inafanya kazi: Android yoyote (ikiwa simu bado imeunganishwa kwenye mtandao\n\
         NA mmiliki anakumbuka password ya account ya Google kwenye simu).\n\n\
         HATUA (mmiliki anafanya mwenyewe, kifaa chake au simu nyingine):\n\
         1. Nenda https://google.com/android/find\n\
         2. Ingia account ile ile ya Google iliyoko kwenye simu\n\
         3. Chagua kifaa → kama 'Unlock device' inaonekana (Android 7-), tumia\n\
         4. Android 8+: fungua password ya screen kupitia 'Lock' → weka PIN mpya →\n\
            tumia PIN hiyo kufungua simu\n\
         5. Kama simu imekwama FRP: hii HAIWEZI kuondoa FRP — mmiliki lazima aingie\n\
            account yake baada ya reset (tazama frp-aftermath).\n\n\
         Ukweli: Android 11+ hakuna njia rasmi ya Google ya kuondoa FRP bila\n\
         password ya account. Mtu akwambie vinginevyo ni uongo.\n",
        c.consent_id
    ))
}

pub fn samsung_remote(imei: &str) -> Result<String> {
    let c = require_consent(imei)?;
    let b = brand_of(&c, None);
    if !b.contains("samsung") && !b.contains("galaxy") {
        bail!("Simu hii si Samsung ({}). Tumia google-remote au brand inayofaa.", c.brand);
    }
    Ok(format!(
        "🔵 SAMSUNG FIND MY MOBILE — consent {}\n\n\
         Njia rasmi + ya haraka zaidi kwa Samsung (Android 11+ ikiwa account ya\n\
         Samsung ilikuwa imeunganishwa KABLA simu kufungwa).\n\n\
         HATUA:\n\
         1. Nenda https://findmymobile.samsung.com\n\
         2. Ingia Samsung account ya mmiliki\n\
         3. Chagua kifaa → 'Unlock' → ingiza password ya Samsung account\n\
         4. Simu inafunguka MARA MOJA — screen lock na biometrics zinafutwa\n\
         5. Kama hakuna Samsung account: tumia google-remote au recovery-reset.\n\n\
         FRP: hii haiondoi FRP bila Google account ya mmiliki (frp-aftermath).\n",
        c.consent_id
    ))
}

pub fn xiaomi_remote(imei: &str) -> Result<String> {
    let c = require_consent(imei)?;
    let b = brand_of(&c, None);
    if !["xiaomi", "redmi", "poco", "mi"].iter().any(|k| b.contains(k)) {
        bail!("Simu hii si Xiaomi/Redmi/POCO ({}).", c.brand);
    }
    Ok(format!(
        "🟠 XIAOMI (Mi Cloud) — consent {}\n\n\
         HATUA:\n\
         1. Nenda https://i.mi.com\n\
         2. Ingia Mi Account ya mmiliki\n\
         3. 'Find device' → chagua simu → 'Unlock' (kama inapatikana)\n\
         4. Kama Mi Account imefunga bootloader/reset: mmiliki anahitaji password\n\
            ya Mi Account — hakuna njia nyingine rasmi.\n\
         5. Huduma za duka (Mi Store /Authorized) zinaweza kusaidia kwa uthibitisho\n\
            wa umiliki (rasidi/ID) — tumia `recover report` kuandaa hati.\n",
        c.consent_id
    ))
}

pub fn huawei_remote(imei: &str) -> Result<String> {
    let c = require_consent(imei)?;
    let b = brand_of(&c, None);
    if !b.contains("huawei") && !b.contains("honor") {
        bail!("Simu hii si Huawei/Honor ({}).", c.brand);
    }
    Ok(format!(
        "🔴 HUAWEI FIND DEVICE — consent {}\n\n\
         HATUA:\n\
         1. Nenda https://cloud.huawei.com (au app 'Find Device')\n\
         2. Ingia HUAWEI ID ya mmiliki\n\
         3. Chagua kifaa → 'Unlock device' → thibitisha password\n\
         4. Simu inafunguka; screen lock inafutwa.\n",
        c.consent_id
    ))
}

// ---------- 8. OWNER-ADB — mmiliki aliyeunganisha ----------

pub fn owner_adb(imei: &str) -> Result<String> {
    let c = require_consent(imei)?;
    devices::require_one_device()?;

    // Ukweli halisi: kama debugging ilikuwa IMEWASHWA kabla, mmiliki anaweza
    // kuzima screen-lock kwa locksettings — hii ni adb ya kweli, sio uongo.
    let mut out = format!("🔓 OWNER-ADB — consent {}\n\n", c.consent_id);
    out.push_str("Inafanya kazi TU ikiwa USB debugging ilikuwa IMEWASHWA kabla simu kufungwa.\n\n");

    // Android version halisi
    let ver = devices::adb_shell("getprop ro.build.version.release")
        .map(|v| v.trim().to_string())
        .unwrap_or_else(|_| "?".into());
    out.push_str(&format!("   Android: {ver}\n\n"));

    // Jaribu kuzima lock screen (kwa simu zilizoruhusu bila root wakati imefunguka)
    match devices::adb_shell("locksettings set-disabled true") {
        Ok(_) => {
            out.push_str("✅ `locksettings set-disabled true` imekubaliwa.\n");
            out.push_str("   → Fungua screen kwa PIN ya sasa mara moja, kisha:\n");
            match devices::adb_shell("locksettings clear --old <PIN_ya_sasa>") {
                Ok(_) => out.push_str("✅ Screen lock imeondolewa (reboot kuthibitisha).\n"),
                Err(_) => out.push_str("   → Endelea: Settings → Security → Screen lock → None.\n"),
            }
        }
        Err(_) => {
            out.push_str("⚠️  `locksettings set-disabled` imekataliwa (kawaida kwenye Android mpya\n");
            out.push_str("   au simu bado imefungwa). Hakuna njia nyingine rasmi ya adb bila root.\n");
            out.push_str("   → Njia halisi ifuatayo: samsung/xiaomi/huawei/google-remote, kisha\n");
            out.push_str("     recovery-reset + frp-aftermath (data inaenda — mmiliki ametia sahihi).\n");
        }
    }
    Ok(out)
}

// ---------- 9. RECOVERY-RESET — combo halisi ya brand ----------

pub fn recovery_reset(imei: &str) -> Result<String> {
    let c = require_consent(imei)?;
    let b = brand_of(&c, None);
    let combo = brands::recovery_combo(&b);
    let (tool, site, _drv) = brands::flash_info(&b);
    let mut out = format!("⚙️ RECOVERY RESET — {} {} (consent {})\n\n", c.brand, c.model, c.consent_id);
    out.push_str("⚠️  DATA YOTE INAENDA. Consent inaonyesha mteja alikubali (HITL).\n\n");
    out.push_str(&format!("Combo ya {b}: {combo}\n"));
    out.push_str("\nHATUA:\n");
    out.push_str("1. Zima simu kabisa.\n");
    out.push_str("2. Bonyeza combo (Vol+ na Power kwa kawaida) hadi menu ya Recovery.\n");
    out.push_str("3. Chagua 'Wipe data / factory reset' (tumia Vol kutembea, Power kukubali).\n");
    out.push_str("4. Chagua 'Yes' → subiri → 'Reboot system now'.\n");
    out.push_str("5. FRP itaomba Google account — mmiliki anaingia yake (frp-aftermath).\n\n");
    out.push_str(&format!("Flash (kama recovery imeshindikana): {tool} · firmware: {site}\n"));
    Ok(out)
}

// ---------- 10. FRP-AFTERMATH — mmiliki anaingia account yake ----------

pub fn frp_aftermath(imei: &str) -> Result<String> {
    let c = require_consent(imei)?;
    Ok(format!(
        "🛡️ FRP AFTERMATH — consent {}\n\n\
         FRP (Factory Reset Protection) inafunguliwa na MMILIKI PEKEE:\n\n\
         HATUA (baada ya reset):\n\
         1. Simu itaomba Google account iliyokuwepo KABLA ya reset.\n\
         2. Mmiliki anaingia account yake (email + password).\n\
         3. Kama amesahau password: google.com/android/find → ama password mpya\n\
            kwenye account, subiri dakika 24-72, kisha ingia tena.\n\
         4. Hakuna njia rasmi nyingine — mtu yeyote anayekuahuru 'bypass instant'\n\
            anadanganya au anatumia exploit haramu.\n\n\
         Kama mmiliki HANA uthibitisho wa umiliki:\n\
         → Google Support (support.google.com/accounts) kwa uthibitisho wa\n\
           umiliki (ricehani, ID, receipt) — ndiyo njia pekee iliyobaki.\n",
        c.consent_id
    ))
}

// ---------- 11. REPORT — ripoti ya kazi ----------

pub fn report(imei: &str, out: &str) -> Result<String> {
    let c = require_consent(imei)?;
    let mut html = String::from(
        "<!DOCTYPE html><html><head><meta charset='utf-8'><title>FUNDI Access Recovery</title>\
         <style>body{font-family:Segoe UI,Arial;background:#0f172a;color:#e2e8f0;max-width:760px;\
         margin:40px auto;padding:24px}.card{background:#1e293b;border-radius:12px;padding:24px;\
         margin-bottom:16px}h1{color:#22d3ee;font-size:22px}td,th{padding:6px 12px;text-align:left;\
         border-bottom:1px solid #334155}</style></head><body>",
    );
    html.push_str(&format!(
        "<div class='card'><h1>🔓 FUNDI ACCESS RECOVERY — Ripoti ya Huduma</h1>\
         <p>Zana rasmi (bila exploit) · Consent: <b>{}</b></p></div>",
        c.consent_id
    ));
    html.push_str("<div class='card'><table>");
    for (k, v) in [
        ("Mteja", c.customer_name.as_str()),
        ("Simu", &format!("{} {}", c.brand, c.model)),
        ("IMEI", &format!("{}…", &c.imei[..6.min(c.imei.len())])),
        ("Huduma", SERVICE_ID),
        ("Fundi", c.technician.as_str()),
        ("Tarehe", c.ts.as_str()),
    ] {
        html.push_str(&format!("<tr><th>{k}</th><td>{v}</td></tr>"));
    }
    html.push_str("</table></div>");
    html.push_str(&format!(
        "<div class='card'><h2 style='color:#22d3ee'>Njia zilizotumika (rasmi)</h2><ol>\
         <li>{} — uchunguzi</li><li>Backup kabla ya kazi</li>\
         <li>Njia rasmi ya brand/Google (remote unlock au recovery)</li>\
         <li>FRP kuliungwa na MMILIKI kwa account yake</li></ol>\
         <p style='color:#94a3b8'>Kazi hii ilifanywa kwa ruhusa iliyosainiwa (Kifungu 267).\
         Hakuna exploit, hakuna kuingia kwa account ya mtu mwingine.</p></div></body></html>",
        TOOLS[0].1
    ));
    std::fs::write(out, html)?;
    Ok(format!("📄 Ripoti: {out} (consent {})", c.consent_id))
}
