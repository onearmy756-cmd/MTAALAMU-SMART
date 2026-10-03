//! DEVICES — kuzungumza na simu halisi (adb / fastboot / idevice).
//! Hakuna uongo: kama tool haipo au simu haipo, tunasema hivyo.

use anyhow::{bail, Result};
use std::process::Command;

pub fn adb_devices() -> Result<Vec<(String, String)>> {
    let out = Command::new("adb").args(["devices"]).output();
    let Ok(o) = out else {
        bail!("ADB haipo — install Android platform-tools (choco install adb)")
    };
    let stdout = String::from_utf8_lossy(&o.stdout);
    let mut list = Vec::new();
    for line in stdout.lines().skip(1) {
        let mut it = line.split_whitespace();
        if let (Some(serial), Some(state)) = (it.next(), it.next()) {
            list.push((serial.to_string(), state.to_string()));
        }
    }
    Ok(list)
}

pub fn require_one_device() -> Result<String> {
    let devs = adb_devices()?;
    let online: Vec<_> = devs.iter().filter(|(_, s)| s == "device").collect();
    if online.is_empty() {
        bail!(
            "Hakuna simu iliyounganishwa (state 'device'). Chomeka simu, washa USB debugging, ruhusu 'Allow'."
        );
    }
    if online.len() > 1 {
        bail!("Simu zaidi ya moja zimeunganishwa — chagua moja (ANDROID_SERIAL).");
    }
    Ok(online[0].0.clone())
}

pub fn adb_shell(cmd: &str) -> Result<String> {
    let o = Command::new("adb")
        .args(["shell", cmd])
        .output()
        .map_err(|e| anyhow::anyhow!("adb shell fail: {e}"))?;
    let s = String::from_utf8_lossy(&o.stdout).to_string();
    if o.status.success() {
        Ok(s)
    } else {
        bail!("adb shell '{}' fail: {}", cmd, String::from_utf8_lossy(&o.stderr))
    }
}

pub fn fastboot_devices() -> Result<Vec<String>> {
    let out = Command::new("fastboot").args(["devices"]).output();
    let Ok(o) = out else {
        bail!("Fastboot haipo — install platform-tools")
    };
    let stdout = String::from_utf8_lossy(&o.stdout);
    Ok(stdout
        .lines()
        .filter_map(|l| l.split_whitespace().next().map(|s| s.to_string()))
        .collect())
}

pub fn idevice_list() -> Vec<String> {
    Command::new("idevice_id")
        .args(["-l"])
        .output()
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .filter(|l| !l.trim().is_empty())
                .map(|s| s.to_string())
                .collect()
        })
        .unwrap_or_default()
}

// ---------- Diagnostics halisi ----------

pub fn battery_info() -> Result<String> {
    adb_shell("dumpsys battery")
}

pub fn storage_info() -> Result<String> {
    adb_shell("df /data")
}

pub fn device_props() -> Result<Vec<(String, String)>> {
    let mut out = Vec::new();
    for key in ["ro.product.brand", "ro.product.model", "ro.build.version.release", "ro.serialno"] {
        match adb_shell(&format!("getprop {key}")) {
            Ok(v) => out.push((key.to_string(), v.trim().to_string())),
            Err(e) => out.push((key.to_string(), format!("? ({e})"))),
        }
    }
    Ok(out)
}

/// Backup halisi kwa adb backup (kama simu inaruhusu; inahitaji kukubali kwenye simu)
pub fn adb_backup(dest_ab: &str) -> Result<String> {
    let o = Command::new("adb")
        .args(["backup", "-apk", "-shared", "-all", "-f", dest_ab])
        .output()
        .map_err(|e| anyhow::anyhow!("adb backup fail: {e}"))?;
    if o.status.success() {
        Ok(format!("Backup imeanza — kubali kwenye simu ('Backup my data'). Faili: {dest_ab}"))
    } else {
        bail!("adb backup imeshindikana: {}", String::from_utf8_lossy(&o.stderr))
    }
}
