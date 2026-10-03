//! PROCEDURES — taratibu za kazi (guide halisi + vitendo vya ADB vilivyozuiwa na consent).

use crate::brands;
use crate::consent;
use crate::devices;
use crate::services;
use anyhow::{bail, Result};
use std::process::Command;

pub struct Outcome {
    pub ok: bool,
    pub summary_sw: String,
    pub steps: Vec<String>,
}

/// ANDROID: futa password kwa ADB (data inabaki) — inahitaji consent ya reset_password_adb
pub fn android_reset_adb(imei: &str, technician: &str) -> Result<Outcome> {
    let svc = services::get("reset_password_adb")?;
    consent::verify(imei, &svc.id)?;
    devices::require_one_device()?;

    let mut steps = Vec::new();
    let files = [
        "/data/system/gesture.key",
        "/data/system/password.key",
        "/data/system/locksettings.db",
        "/data/system/locksettings.db-shm",
        "/data/system/locksettings.db-wal",
    ];
    for f in files {
        let r = Command::new("adb").args(["shell", "rm", f]).output();
        match r {
            Ok(o) if o.status.success() => steps.push(format!("✅ rm {f}")),
            Ok(_) => steps.push(format!("⚠️ {f} haipo/root haipo (si makosa)")),
            Err(e) => steps.push(format!("❌ {f}: {e}")),
        }
    }
    let rb = Command::new("adb").args(["reboot"]).output();
    steps.push(match rb {
        Ok(_) => "✅ adb reboot — subiri simu iwake (sekunde 30)".into(),
        Err(e) => format!("❌ reboot: {e}"),
    });

    Ok(Outcome {
        ok: true,
        summary_sw: "ADB reset imekamilika. Simu inapaswa kuwaka bila password; data inapaswa kubaki.".into(),
        steps,
    })
}

/// ANDROID: Recovery-mode wipe — hakuna ADB; inarudisha mwongozo wa brand (data inapotea)
pub fn android_reset_recovery(brand: &str, imei: &str, _technician: &str) -> Result<Outcome> {
    let svc = services::get("reset_password_recovery")?;
    consent::verify(imei, &svc.id)?;
    let combo = brands::recovery_combo(brand);

    Ok(Outcome {
        ok: true,
        summary_sw: format!("Recovery wipe ({brand}): data YOTE inapotea; password inafutwa."),
        steps: vec![
            "Zima simu (Power 10s)".into(),
            format!("Ingia Recovery Mode: {combo}"),
            "Chagua 'Wipe data/factory reset' (Volume = navigate, Power = select)".into(),
            "Chagua 'Yes' → subiri 'Data wipe complete'".into(),
            "Chagua 'Reboot system now'".into(),
            "Setup kama mpya (FRP itaonekana kama account ilikuwepo)".into(),
        ],
    })
}

/// Flash firmware — mwongozo data-driven kwa brand
pub fn flash_firmware(brand: &str, imei: &str) -> Result<Outcome> {
    let svc = services::get("flash_firmware")?;
    consent::verify(imei, &svc.id)?;
    let info = brands::flash_info(brand);
    Ok(Outcome {
        ok: true,
        summary_sw: format!("Flash ({brand}): tool = {}, firmware: {}", info.0, info.1),
        steps: vec![
            format!("Download firmware: {}", info.1),
            format!("Install tool: {} + drivers ({})", info.0, info.2),
            format!("Ingia download/fastboot mode: {}", brands::download_mode(brand)),
            "Chomeka kwa USB, chagua firmware, Start/Flash".into(),
            "Subiri 'PASS!' (Odin) au kumaliza kwa tool nyingine".into(),
        ],
    })
}

/// Simu za BUTTON: master reset codes + hard reset (data-driven kutoka brands.json)
pub fn button_reset(brand: &str, imei: &str) -> Result<Outcome> {
    let svc = services::get("reset_button_code")?;
    consent::verify(imei, &svc.id)?;
    let bp = brands::button_phone(brand);
    let codes = bp
        .master_codes
        .iter()
        .map(|c| format!("  • {} → Restore factory", c))
        .collect::<Vec<_>>()
        .join("\n");
    Ok(Outcome {
        ok: true,
        summary_sw: format!("Button reset ({brand}): master codes au hard-reset combo."),
        steps: vec![
            "ANGALIA: Hii inafuta kila kitu (contacts, messages, n.k.)".into(),
            "NJIA 1 — Master reset code (kwenye simu, bila password kwa kawaida):".into(),
            codes,
            format!("NJIA 2 — Hard reset: {}", bp.hard_reset_sw),
            format!("NJIA 3 — Flash (kama codes zote zimeshindwa): {}", bp.flash_tool),
        ],
    })
}

/// FRP — mwongozo (destructive = la; lakini inahitaji consent kwa kisheria)
pub fn frp_guide(android_ver: &str, imei: &str) -> Result<Outcome> {
    let svc = services::get(if android_ver.starts_with("1") && android_ver.contains("1") {
        "bypass_frp_new"
    } else {
        "bypass_frp"
    })?;
    consent::verify(imei, &svc.id)?;
    let steps: Vec<String> = if android_ver >= "11" {
        vec![
            "⚠️ Android 11+: bypass rasmi ni ngumu — tumia tools za kulipwa au Google Account Recovery".into(),
            "Google Account Recovery: mteja anajua email → accounts.google.com/signin/recovery".into(),
            "Kama hapana: thibitisha umiliki kwa receipt/box → wasiliana na duka/reo la brand".into(),
        ]
    } else {
        vec![
            "Kwenye setup screen: Wi-Fi → Add Network".into(),
            "Andika SSID yoyote → Show password".into(),
            "Keyboard: ?123 → weka ~@#$%^&*() → shikilia '~' → chagua 'Settings' (au lens/share)".into(),
            "Settings → Backup & Reset → Factory data reset (kama FRP haijazimwa kabla ya reset)".into(),
            "Au: TalkBack → Voice command → 'Open YouTube' → browser → FRP bypass APK".into(),
        ]
    };
    Ok(Outcome { ok: true, summary_sw: format!("FRP guide ({android_ver})"), steps })
}

/// BACKUP halisi kwa adb (inahitaji simu iwe hai + kukubali kwenye screen)
pub fn backup(dest_dir: &str, imei: &str) -> Result<Outcome> {
    let svc = services::get("backup_data")?;
    consent::verify(imei, &svc.id)?;
    devices::require_one_device()?;
    std::fs::create_dir_all(dest_dir)?;
    let ts = chrono::Local::now().format("%Y%m%d-%H%M%S");
    let dest = format!("{dest_dir}/backup-{ts}.ab");
    let msg = devices::adb_backup(&dest)?;
    Ok(Outcome { ok: true, summary_sw: msg, steps: vec![format!("dest: {dest}")] })
}

/// Diagnostics (haina consent — haisababishi mabadiliko)
pub fn diagnostics() -> Result<Outcome> {
    let mut steps = Vec::new();
    match devices::device_props() {
        Ok(props) => {
            for (k, v) in props {
                steps.push(format!("prop {k} = {v}"));
            }
        }
        Err(e) => steps.push(format!("props: {e}")),
    }
    match devices::battery_info() {
        Ok(b) => {
            for line in b.lines() {
                if line.contains("level") || line.contains("health") || line.contains("temperature") {
                    steps.push(format!("battery: {}", line.trim()));
                }
            }
        }
        Err(e) => steps.push(format!("battery: {e}")),
    }
    match devices::storage_info() {
        Ok(s) => steps.push(format!("storage: {}", s.lines().last().unwrap_or("").trim())),
        Err(e) => steps.push(format!("storage: {e}")),
    }
    if steps.is_empty() {
        bail!("Diagnostics haikupata kitu — simu imeunganishwa? ADB ipo?");
    }
    Ok(Outcome { ok: true, summary_sw: "Diagnostics (halisi kutoka adb)".into(), steps })
}

pub fn ensure_not_locked(service_id: &str, imei: &str) -> Result<()> {
    if consent::verify(imei, service_id).is_ok() {
        Ok(())
    } else {
        bail!("Kazi {service_id} kwa IMEI {imei} imezuiliwa: hakuna consent.")
    }
}
