//! shield.rs — FUNDI SHIELD: antivirus + security audit ya FUNDI (offline, local).
//!
//! KANUNI:
//!   1. Scan ni HALISI: signature scan (EICAR + patterns halisi), audit ya
//!      startup entries, audit ya security settings (Defender state), temp files.
//!   2. USIFUTE kitu BILA consent (HITL) — kazi ya kufuta ni hatari.
//!   3. Hakuna uongo: kama Defender haipo (Linux), tunasema hivyo.
//!
//! Command: fundi-mobile shield <audit|scan|clean|report>

use crate::consent;
use anyhow::{bail, Result};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

fn run(cmd: &str, args: &[&str]) -> Option<String> {
    std::process::Command::new(cmd)
        .args(args)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
}

// ---------- SECURITY AUDIT (halisi) ----------

#[cfg(target_os = "windows")]
fn defender_state() -> Value {
    // PowerShell Get-MpComputerStatus (halisi)
    let out = run("powershell", &["-NoProfile", "-Command",
        "Get-MpComputerStatus | Select-Object AMServiceEnabled,AntivirusEnabled,RealTimeProtectionEnabled,AntivirusSignatureLastUpdated | ConvertTo-Json"]);
    match out {
        Some(o) => {
            let v: Value = serde_json::from_str(&o).unwrap_or(json!({}));
            json!({
                "product": "Windows Defender",
                "antivirus_enabled": v["AntivirusEnabled"].as_bool().unwrap_or(false),
                "realtime_enabled": v["RealTimeProtectionEnabled"].as_bool().unwrap_or(false),
                "signature_last_updated": v["AntivirusSignatureLastUpdated"].as_str().unwrap_or("?"),
            })
        }
        None => json!({ "product": "Windows Defender", "error": "haipatikani (Get-MpComputerStatus imeshindikana)" }),
    }
}

#[cfg(not(target_os = "windows"))]
fn defender_state() -> Value {
    // Linux: hakuna Defender; onyesha uwazi
    let clamav = run("clamscan", &["--version"]).is_some();
    json!({
        "product": if clamav { "ClamAV" } else { "hakuna AV iliyopatikana" },
        "antivirus_enabled": clamav,
        "realtime_enabled": false,
        "note": "Linux: weka ClamAV (sudo apt install clamav) au tumia FUNDI SHIELD scan",
    })
}

/// Audit kamili ya usalama (halisi, hakuna kubadilisha chochote)
pub fn audit() -> Result<Value> {
    let defender = defender_state();

    // Startup entries (halisi)
    #[cfg(target_os = "windows")]
    let startup = {
        let out = run("wmic", &["startup", "get", "Caption,Command,Location", "/format:csv"]);
        let n = out.map(|o| o.lines().skip(1).filter(|l| !l.trim().is_empty()).count()).unwrap_or(0);
        json!({ "entries": n, "source": "wmic startup" })
    };
    #[cfg(not(target_os = "windows"))]
    let startup = {
        let dirs = ["~/.config/autostart", "/etc/xdg/autostart"];
        let mut n = 0;
        for d in dirs {
            if let Ok(rd) = std::fs::read_dir(shellexpand_dir(d)) {
                n += rd.filter_map(|e| e.ok()).filter(|e| e.path().extension().map(|x| x == "desktop").unwrap_or(false)).count();
            }
        }
        json!({ "entries": n, "source": "autostart .desktop" })
    };

    // Firewall state
    #[cfg(target_os = "windows")]
    let firewall = run("netsh", &["advfirewall", "show", "allprofiles", "state"])
        .map(|o| o.to_lowercase().contains("on"))
        .unwrap_or(false);
    #[cfg(not(target_os = "windows"))]
    let firewall = run("ufw", &["status"]).map(|o| o.contains("active")).unwrap_or(false);

    // UAC / sudo (alama tu)
    #[cfg(target_os = "windows")]
    let uac_note = "UAC: angalia Control Panel → User Accounts (Keep notification ON)";
    #[cfg(not(target_os = "windows"))]
    let uac_note = "sudo: tumia sudoers safi — hakuna NOPASSWD kwa watumia wote";

    // Temp files size
    let temp = temp_dir_stats();

    // Recommendations (halisi kulingana na matokeo)
    let mut recs = Vec::new();
    if !defender["antivirus_enabled"].as_bool().unwrap_or(false) {
        recs.push("❌ Antivirus HAIWASHI — washa Windows Defender au weka ClamAV (Linux)".into());
    }
    if !defender["realtime_enabled"].as_bool().unwrap_or(true) && defender["antivirus_enabled"].as_bool().unwrap_or(false) {
        recs.push("⚠️ Real-time protection imezimwa — washa mara moja".into());
    }
    if !firewall {
        recs.push("❌ Firewall HAIWASHI — washa (netsh advfirewall set allprofiles state on / ufw enable)".into());
    }
    if temp["size_mb"].as_f64().unwrap_or(0.0) > 500.0 {
        recs.push(format!("⚠️ Temp files: {} MB — endesha 'shield clean' (baada ya consent)", temp["size_mb"]));
    }
    let sig = defender["signature_last_updated"].as_str().unwrap_or("");
    if !sig.is_empty() && sig != "?" {
        recs.push(format!("ℹ️ AV signatures: last update {sig} — hakikisha ni za hivi karibuni"));
    }
    if recs.is_empty() {
        recs.push("✅ Audit safi: AV ON, firewall ON — endesha 'shield scan' kwa scan kamili".into());
    }

    Ok(json!({
        "kind": "fundi-shield-audit",
        "timestamp": chrono::Local::now().to_rfc3339(),
        "antivirus": defender,
        "firewall_on": firewall,
        "startup": startup,
        "temp": temp,
        "notes": [uac_note],
        "recommendations": recs,
    }))
}

#[cfg(not(target_os = "windows"))]
fn shellexpand_dir(p: &str) -> PathBuf {
    if let Some(rest) = p.strip_prefix("~/") {
        if let Some(home) = std::env::var_os("HOME") {
            return PathBuf::from(home).join(rest);
        }
    }
    PathBuf::from(p)
}

// ---------- TEMP FILES (halisi) ----------

fn temp_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        std::env::var("TEMP").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from("C:\\Windows\\Temp"))
    }
    #[cfg(not(target_os = "windows"))]
    {
        std::env::var("TMPDIR").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from("/tmp"))
    }
}

fn dir_size(path: &Path) -> (u64, u64) {
    let mut files = 0u64;
    let mut bytes = 0u64;
    if let Ok(rd) = std::fs::read_dir(path) {
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                let (f, b) = dir_size(&p);
                files += f;
                bytes += b;
            } else if let Ok(md) = e.metadata() {
                files += 1;
                bytes += md.len();
            }
        }
    }
    (files, bytes)
}

fn temp_dir_stats() -> Value {
    let d = temp_dir();
    let (files, bytes) = dir_size(&d);
    json!({
        "path": d.display().to_string(),
        "files": files,
        "size_mb": (bytes as f64 / 1024.0 / 1024.0 * 10.0).round() / 10.0,
    })
}

// ---------- SIGNATURE SCAN (halisi) ----------

/// EICAR test file (kiwango cha dunia cha kupima antivirus — si malware)
const EICAR: &[u8] = b"X5O!P%@AP[4\\PZX54(P^)7CC)7}$EICAR-STANDARD-ANTIVIRUS-TEST-FILE!$H+H*";

/// Patterns halisi (zinaonekana kwenye files za scan — kwa ajili ya DETECTION ya test/known)
const SIGNATURES: &[(&str, &str)] = &[
    ("EICAR-STANDARD-ANTIVIRUS-TEST-FILE", "EICAR-Test-File"),
    ("X5O!P%@AP[4\\PZX54(P^)7CC)7}", "EICAR-Test-File"),
];

fn scan_file(path: &Path, hits: &mut Vec<Value>) -> std::io::Result<()> {
    // Soma files ndogo tu (skip binaries kubwa — hii ni scanner ya haraka)
    let md = std::fs::metadata(path)?;
    if md.len() > 2 * 1024 * 1024 {
        return Ok(());
    }
    let bytes = std::fs::read(path)?;
    let head = String::from_utf8_lossy(&bytes[..bytes.len().min(4096)]);
    for (sig, name) in SIGNATURES {
        if head.contains(sig) {
            hits.push(json!({
                "file": path.display().to_string(),
                "detection": name,
                "action": "quarantine-pending (HITL)",
            }));
            break;
        }
    }
    Ok(())
}

fn walk(dir: &Path, depth: u32, hits: &mut Vec<Value>, scanned: &mut u64) {
    if depth > 6 || hits.len() >= 50 {
        return;
    }
    if let Ok(rd) = std::fs::read_dir(dir) {
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                walk(&p, depth + 1, hits, scanned);
            } else {
                *scanned += 1;
                let _ = scan_file(&p, hits);
            }
        }
    }
}

/// Scan halisi: EICAR test + startup + audit kama muhtasari.
/// `with_eicar`: tengeneza EICAR test file kwanza (kupima uwezo wa scan YENYEWE).
pub fn scan(with_eicar: bool) -> Result<Value> {
    // (a) kama with_eicar: tandika EICAR kwenye temp (test file rasmi ya dunia)
    if with_eicar {
        let p = temp_dir().join("fundi_eicar_test.com");
        std::fs::write(&p, EICAR)?;
    }

    // (b) scan temp + current dir (haraka, salama)
    let mut hits = Vec::new();
    let mut scanned = 0u64;
    walk(&temp_dir(), 0, &mut hits, &mut scanned);
    walk(&std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")), 0, &mut hits, &mut scanned);

    // (c) Defender inaweza kufuta EICAR yenyewe (ni kazi yake halisi) — hilo ni BAY
    let eicar_removed = with_eicar
        && !temp_dir().join("fundi_eicar_test.com").exists();

    let audit = audit()?;
    Ok(json!({
        "kind": "fundi-shield-scan",
        "timestamp": chrono::Local::now().to_rfc3339(),
        "files_scanned": scanned,
        "detections": hits,
        "eicar_test": {
            "planted": with_eicar,
            "removed_by_real_av": eicar_removed,
            "meaning": if eicar_removed { "Antivirus HALISI ya mfumo ilifuta EICAR — protection inafanya kazi ✅" }
                       else if with_eicar { "EICAR bado ipo — FUNDI SHIELD inaweza kuiona (detections); Defender inaweza kuwa imezimwa" }
                       else { "haikupandikizwa" },
        },
        "audit_summary": audit["recommendations"].clone(),
    }))
}

// ---------- CLEAN (HITL — consent LAZIMA) ----------

/// Futa temp files — HITL: inahitaji consent "shield_clean".
pub fn clean(imei_or_device: &str) -> Result<Value> {
    // HITL: consent ya service "shield_clean" (mteja/fundi ameomba rasmi)
    consent::verify(imei_or_device, "shield_clean")
        .map_err(|e| anyhow::anyhow!("{e}\n→ Rekodi kwanza: fundi-mobile consent add --service shield_clean --imei {imei_or_device} ..."))?;

    let d = temp_dir();
    let (files_before, bytes_before) = dir_size(&d);
    let mut removed = 0u64;
    let mut freed_bytes = 0u64;
    let mut errors = 0u64;
    if let Ok(rd) = std::fs::read_dir(&d) {
        for e in rd.flatten() {
            let p = e.path();
            let size = e.metadata().map(|m| if m.is_dir() { dir_size(&p).1 } else { m.len() }).unwrap_or(0);
            let ok = if p.is_dir() { std::fs::remove_dir_all(&p).is_ok() } else { std::fs::remove_file(&p).is_ok() };
            if ok {
                removed += 1;
                freed_bytes += size;
            } else {
                errors += 1; // files zinazotumiwa na programs haziwezi kufutwa — ni kawaida
            }
        }
    }
    Ok(json!({
        "kind": "fundi-shield-clean",
        "path": d.display().to_string(),
        "removed_entries": removed,
        "freed_mb": (freed_bytes as f64 / 1024.0 / 1024.0 * 10.0).round() / 10.0,
        "size_before_mb": (bytes_before as f64 / 1024.0 / 1024.0 * 10.0).round() / 10.0,
        "locked_files_skipped": errors,
        "note": "Files zilizofungwa na programs hazikufutwa (salama). Reboot kisha clean tena kama unahitaji.",
    }))
}

/// Ripoti ya HTML (audit + scan summary)
pub fn report(out_path: &str) -> Result<String> {
    let a = audit()?;
    let mut h = String::from(
        "<!DOCTYPE html><html><head><meta charset='utf-8'><title>FUNDI SHIELD — Ripoti</title>\
         <style>body{font-family:Segoe UI,Arial;background:#0f172a;color:#e2e8f0;max-width:760px;margin:32px auto;padding:0 16px}\
         .c{background:#1e293b;border-radius:12px;padding:20px;margin:12px 0}h1{color:#22d3ee}\
         .good{color:#00e676}.bad{color:#ff5252}td,th{padding:6px 12px;border-bottom:1px solid #334155;text-align:left}</style></head><body>",
    );
    h.push_str("<div class='c'><h1>🛡️ FUNDI SHIELD — Ripoti ya Usalama</h1></div>");
    h.push_str("<div class='c'><table>");
    let av = &a["antivirus"];
    h.push_str(&format!(
        "<tr><th>Antivirus</th><td>{}</td></tr>\
         <tr><th>Real-time</th><td>{}</td></tr>\
         <tr><th>Firewall</th><td>{}</td></tr>\
         <tr><th>Startup entries</th><td>{}</td></tr>\
         <tr><th>Temp files</th><td>{}</td></tr>",
        av["product"].as_str().unwrap_or("?"),
        if av["realtime_enabled"].as_bool().unwrap_or(false) { "<span class='good'>ON</span>" } else { "<span class='bad'>OFF</span>" },
        if a["firewall_on"].as_bool().unwrap_or(false) { "<span class='good'>ON</span>" } else { "<span class='bad'>OFF</span>" },
        a["startup"]["entries"].as_i64().unwrap_or(0),
        format!("{} ({} files)", a["temp"]["size_mb"].as_f64().unwrap_or(0.0), a["temp"]["files"].as_i64().unwrap_or(0)),
    ));
    h.push_str("</table></div><div class='c'><h3>Mapendekezo</h3><ul>");
    if let Some(recs) = a["recommendations"].as_array() {
        for r in recs {
            h.push_str(&format!("<li>{}</li>", r.as_str().unwrap_or("")));
        }
    }
    h.push_str("</ul></div></body></html>");
    std::fs::write(out_path, h)?;
    Ok(out_path.to_string())
}

pub fn bail_help() -> Result<()> {
    bail!("shield: audit | scan [--eicar] | clean --device <id> (HITL) | report --out f.html | guard baseline | guard check | guard install | guard status")
}

// ---------- GUARD: kinga ya virusi kila siku (baseline + drift) ----------

fn guard_dir() -> PathBuf {
    let base = std::env::var("FUNDI_DATA").unwrap_or_else(|_| "../data".into());
    PathBuf::from(base).join("shield")
}

#[cfg(target_os = "windows")]
fn hosts_path() -> PathBuf {
    PathBuf::from("C:\\Windows\\System32\\drivers\\etc\\hosts")
}
#[cfg(not(target_os = "windows"))]
fn hosts_path() -> PathBuf {
    PathBuf::from("/etc/hosts")
}

#[cfg(target_os = "windows")]
fn startup_paths() -> Vec<PathBuf> {
    let mut v = Vec::new();
    if let Some(appdata) = std::env::var_os("APPDATA") {
        v.push(PathBuf::from(appdata).join("Microsoft\\Windows\\Start Menu\\Programs\\Startup"));
    }
    if let Some(pd) = std::env::var_os("ProgramData") {
        v.push(PathBuf::from(pd).join("Microsoft\\Windows\\Start Menu\\Programs\\StartUp"));
    }
    v
}
#[cfg(not(target_os = "windows"))]
fn startup_paths() -> Vec<PathBuf> {
    let mut v = Vec::new();
    if let Some(home) = std::env::var_os("HOME") {
        v.push(PathBuf::from(home).join(".config/autostart"));
    }
    v.push(PathBuf::from("/etc/xdg/autostart"));
    v
}

fn hash_drift_hex(bytes: &[u8]) -> String {
    // FNV-1a 64-bit — hash ya drift-detection (si crypto, inatosha kubadilisha-hali)
    let mut h: u64 = 0xcbf29ce484222325;
    for b in bytes {
        h ^= *b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    format!("{h:016x}")
}

/// GUARD BASELINE — pima hali YA SALAMA ya sasa (hosts + startup + temp count)
pub fn guard_baseline() -> Result<Value> {
    let hosts = std::fs::read(hosts_path()).unwrap_or_default();
    let mut startup = Vec::new();
    for d in startup_paths() {
        if let Ok(rd) = std::fs::read_dir(&d) {
            for e in rd.flatten() {
                startup.push(json!({ "name": e.file_name().to_string_lossy(), "path": e.path().display().to_string() }));
            }
        }
    }
    let temp = temp_dir_stats();
    let baseline = json!({
        "kind": "shield-guard-baseline",
        "ts": chrono::Local::now().to_rfc3339(),
        "hosts_hash": hash_drift_hex(&hosts),
        "hosts_size": hosts.len(),
        "startup": startup,
        "temp_files": temp["files"].clone(),
    });
    let dir = guard_dir();
    std::fs::create_dir_all(&dir)?;
    std::fs::write(dir.join("baseline.json"), serde_json::to_string_pretty(&baseline)?)?;
    Ok(baseline)
}

fn load_baseline() -> Option<Value> {
    let p = guard_dir().join("baseline.json");
    std::fs::read_to_string(&p).ok().and_then(|t| serde_json::from_str(&t).ok())
}

/// GUARD CHECK — linganisha na baseline; drift = dalili za malware (hosts imebadilika,
/// startup mpya isiyojulikana, temp imejaa kasi isiyo ya kawaida)
pub fn guard_check() -> Result<Value> {
    let base = load_baseline()
        .ok_or_else(|| anyhow::anyhow!("Hakuna baseline — endesha kwanza: fundi-mobile shield guard baseline"))?;
    let hosts = std::fs::read(hosts_path()).unwrap_or_default();
    let hosts_now = hash_drift_hex(&hosts);
    let hosts_changed = hosts_now != base["hosts_hash"].as_str().unwrap_or("");

    let mut startup_now: Vec<Value> = Vec::new();
    for d in startup_paths() {
        if let Ok(rd) = std::fs::read_dir(&d) {
            for e in rd.flatten() {
                startup_now.push(json!({
                    "name": e.file_name().to_string_lossy(),
                    "path": e.path().display().to_string(),
                }));
            }
        }
    }
    let base_names: Vec<String> = base["startup"].as_array()
        .map(|a| a.iter().filter_map(|x| x["name"].as_str().map(String::from)).collect())
        .unwrap_or_default();
    let new_startup: Vec<&Value> = startup_now
        .iter()
        .filter(|s| !base_names.contains(&s["name"].as_str().unwrap_or("").to_string()))
        .collect();

    let temp = temp_dir_stats();
    let temp_files = temp["files"].as_i64().unwrap_or(0);
    let temp_base = base["temp_files"].as_i64().unwrap_or(0);
    let temp_exploded = temp_base > 0 && temp_files > temp_base * 10;

    let mut alerts = Vec::new();
    if hosts_changed {
        alerts.push("🚨 hosts file imebadilika tangu baseline — MALWARE inayoweza kueneza huifanyia hivi (redirects/phishing). Angalia hosts + endesha scan.".into());
    }
    for s in &new_startup {
        alerts.push(format!(
            "⚠️ STARTUP MPYA: '{}' ({}) — hakikisha ni program unayoijua; malware hujiweka startup",
            s["name"].as_str().unwrap_or("?"), s["path"].as_str().unwrap_or("?")
        ));
    }
    if temp_exploded {
        alerts.push(format!("⚠️ Temp imeongezeka kasi isiyo ya kawaida: {temp_base} → {temp_files} files — malware miners huwacha junk huko"));
    }
    if alerts.is_empty() {
        alerts.push("✅ Hakuna drift — mfumo unaofanana na baseline salama".into());
    }
    Ok(json!({
        "kind": "shield-guard-check",
        "ts": chrono::Local::now().to_rfc3339(),
        "hosts_changed": hosts_changed,
        "new_startup": new_startup,
        "temp_files": temp_files,
        "temp_baseline": temp_base,
        "alerts": alerts,
        "verdict": if hosts_changed || !new_startup.is_empty() || temp_exploded { "🟡 DRIFT ILIPATIKANA — chunguza" } else { "🟢 SAFI" },
    }))
}

/// GUARD INSTALL — andika scheduled task ya kila siku (schtasks Windows / cron Linux)
pub fn guard_install() -> Result<String> {
    let exe = std::env::current_exe()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| "fundi-mobile".into());
    #[cfg(target_os = "windows")]
    {
        let out = std::process::Command::new("schtasks")
            .args([
                "/Create", "/F", "/SC", "DAILY", "/ST", "09:00",
                "/TN", "FundiShieldGuard",
                "/TR", &format!("\"{exe}\" shield guard check --quiet"),
            ])
            .output()
            .map_err(|e| anyhow::anyhow!("schtasks fail: {e}"))?;
        if out.status.success() {
            Ok("✅ GUARD imesakinishwa: kila siku 09:00 (schtasks /query /tn FundiShieldGuard)".into())
        } else {
            bail!("schtasks imekataliwa (run as admin): {}", String::from_utf8_lossy(&out.stderr))
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        let cron_line = format!("0 9 * * * {exe} shield guard check --quiet\n");
        let cron_file = "/etc/cron.d/fundi-shield";
        std::fs::write(cron_file, cron_line)
            .map_err(|e| anyhow::anyhow!("andika {cron_file} (run as root): {e}"))?;
        Ok(format!("✅ GUARD imesakinishwa: {cron_file} (kila siku 09:00)"))
    }
}

/// GUARD STATUS
pub fn guard_status() -> Value {
    let base = load_baseline();
    json!({
        "baseline_exists": base.is_some(),
        "baseline_ts": base.as_ref().map(|b| b["ts"].clone()).unwrap_or(json!(null)),
        "check_hint": "fundi-mobile shield guard check",
    })
}
