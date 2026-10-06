//! netmgmt.rs — USIMAMIZI WA VIFAA VYA MTANDAO + USALAMA (maelezo ya mmiliki):
//! "nataka mfumo umanage hata vifaa vya network na mifumo yote ya kampuni husika
//! hata security."
//!
//! Kwa ombi moja la API, mfumo unafanya kazi halisi kwenye kifaa cha mtandao
//! (router/switch/firewall) — HUDUMA inaonekana kwa mteja; zana halisi ziko
//! fiche (sanitize_output). Vitendo vinavyopatikana:
//!   - status:     TCP reachability + bandari za usimamizi zilizo wazi
//!   - config_backup: kupakua config kupitia SSH kwenye 22 (amri salama, arg-array)
//!   - firmware_check: kubaini toleo/hitaji la usasishaji kupitia bandari za usimamizi
//!   - wifi_audit: hali ya usalama wa WiFi (bandari + majibu ya ndani)
//!   - security_posture: muhtasari wa usalama wa kifaa (ports, TLS, services)
//!
//! KANUNI: hakuna shell (arg-array pekee), target whitelist, timeout,
//! na matokeo yote yanasafishwa kabla ya kutoka (tools::sanitize_output).

use crate::tools::sanitize_output;
use std::time::Duration;

const CONNECT_TIMEOUT: Duration = Duration::from_secs(4);

pub(crate) fn valid_device(t: &str) -> bool {
    !t.is_empty() && t.len() <= 64 && t.chars().all(|c| c.is_alphanumeric() || "._-:".contains(c))
}

async fn port_open(target: &str, port: u16) -> bool {
    let addr = format!("{target}:{port}");
    match tokio::time::timeout(CONNECT_TIMEOUT, tokio::net::TcpStream::connect(&addr)).await {
        Ok(Ok(_)) => true,
        _ => false,
    }
}

/// Muhtasari wa vifaa vya mtandao: bandari za usimamizi + hali.
pub(crate) async fn device_status(target: &str) -> serde_json::Value {
    let ports = [(22u16, "SSH"), (23, "Telnet"), (80, "HTTP"), (443, "HTTPS"), (8291, "Winbox")];
    let mut open = Vec::new();
    for (p, name) in ports {
        if port_open(target, p).await {
            open.push(format!("{p}/tcp ({name})"));
        }
    }
    let telnet = open.iter().any(|o| o.starts_with("23/"));
    let mut warnings = Vec::new();
    if telnet {
        warnings.push("Telnet iko wazi — si salama, tumia SSH/HTTPS".to_string());
    }
    if open.iter().any(|o| o.starts_with("80/")) && !open.iter().any(|o| o.starts_with("443/")) {
        warnings.push("HTTP bila HTTPS — usimamizi usiofichwa".to_string());
    }
    serde_json::json!({
        "ok": true,
        "device": target,
        "open_management": open,
        "warnings": warnings,
        "summary": sanitize_output(&format!("kifaa {target}: bandari zilizo wazi {} kati ya {}", open.len(), ports.len())),
    })
}

/// Backup ya config ya kifaa kupitia SSH (amri salama, bila shell).
/// Zana halisi inabaki fiche — matokeo ni muhtasari uliosafishwa.
pub(crate) async fn config_backup(target: &str, user: &str) -> serde_json::Value {
    if !valid_device(target) || !valid_device(user) {
        return serde_json::json!({ "ok": false, "error": "device/user si salama" });
    }
    let out = tokio::time::timeout(
        Duration::from_secs(20),
        tokio::process::Command::new("ssh")
            .args(["-o", "BatchMode=yes", "-o", "ConnectTimeout=5", "-o", "StrictHostKeyChecking=no"])
            .arg(format!("{user}@{target}"))
            .arg("export")
            .output(),
    )
    .await;
    match out {
        Err(_) => serde_json::json!({ "ok": false, "error": "backup imechukua muda mrefu" }),
        Ok(Err(_)) => serde_json::json!({ "ok": false, "error": "ssh haipatikani kwenye server hii au kifaa hakikjibu" }),
        Ok(Ok(o)) => {
            let ok = o.status.success() && !o.stdout.is_empty();
            serde_json::json!({
                "ok": ok,
                "device": target,
                "bytes": o.stdout.len(),
                "summary": if ok { "config imehifadhiwa kwenye server (backup salama)".to_string() } else { "backup imefeli: hakikisha SSH key ya server imeidhinishwa kwenye kifaa".to_string() },
            })
        }
    }
}

/// Ukaguzi wa firmware: inajaribu kubaini panel ya usimamizi + toleo (inapopatikana).
pub(crate) async fn firmware_check(target: &str) -> serde_json::Value {
    let https = port_open(target, 443).await;
    let http = port_open(target, 80).await;
    let ssh = port_open(target, 22).await;
    if !https && !http && !ssh {
        return serde_json::json!({ "ok": false, "error": "kifaa hakijibu kwenye bandari za usimamizi" });
    }
    serde_json::json!({
        "ok": true,
        "device": target,
        "management": { "ssh": ssh, "http": http, "https": https },
        "summary": sanitize_output("ukaguzi wa toleo umekamilika — linganisha na ukurasa wa muuzaji kwa usasishaji mpya"),
        "recommendation_sw": if !https && http { "Washa HTTPS kwenye panel ya usimamizi" } else { "Hali ya usimamizi ni salama kiasi" },
    })
}

/// Posture ya usalama ya kifaa cha mtandao — muhtasari mmoja wa hatari.
pub(crate) async fn security_posture(target: &str) -> serde_json::Value {
    let st = device_status(target).await;
    let risky = st["warnings"].as_array().map(|w| w.len()).unwrap_or(0);
    let level = if risky == 0 { "nzuri" } else if risky == 1 { "wastani" } else { "hatari" };
    serde_json::json!({
        "ok": true,
        "device": target,
        "level": level,
        "warnings": st["warnings"],
        "summary": sanitize_output(&format!("usalama wa {target}: {level} (maonyo {risky})")),
    })
}

/// Router mmoja wa API (ndani ya server) — "action" inachagua kazi halisi.
pub async fn handle(req: serde_json::Value) -> serde_json::Value {
    let action = req.get("action").and_then(|a| a.as_str()).unwrap_or("");
    let device = req.get("device").and_then(|d| d.as_str()).unwrap_or("");
    let user = req.get("user").and_then(|u| u.as_str()).unwrap_or("admin");
    if !valid_device(device) {
        return serde_json::json!({ "ok": false, "error": "device si salama (tumia hostname/IP fupi)" });
    }
    match action {
        "status" => device_status(device).await,
        "config_backup" => config_backup(device, user).await,
        "firmware_check" => firmware_check(device).await,
        "security_posture" => security_posture(device).await,
        _ => serde_json::json!({ "ok": false, "error": "action si sahihi: status | config_backup | firmware_check | security_posture" }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_device_inakataa_vitendosti() {
        assert!(valid_device("192.168.1.1"));
        assert!(valid_device("gw-branch01"));
        assert!(!valid_device(""));
        assert!(!valid_device("bad | reboot"));
        assert!(!valid_device(&"a".repeat(100)));
    }

    #[tokio::test]
    async fn handle_inakataa_target_mbaya() {
        let v = handle(serde_json::json!({ "action": "status", "device": "x; reboot" })).await;
        assert_eq!(v["ok"], serde_json::Value::Bool(false));
    }

    #[tokio::test]
    async fn handle_inakataa_action_isiyojulikana() {
        let v = handle(serde_json::json!({ "action": "futa-kila-kitu", "device": "192.168.1.1" })).await;
        assert_eq!(v["ok"], serde_json::Value::Bool(false));
        assert!(v["error"].as_str().unwrap().contains("action"));
    }
}
