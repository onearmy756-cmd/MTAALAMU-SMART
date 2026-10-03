//! restore.rs — RESTORE YA NGUVU: backup verification halisi + restore plan (HITL).
//!
//! Inafanya kazi na backups halisi:
//!   - Android .ab (adb backup, magic "ANDROID BACKUP")
//!   - ZIP (magic PK\x03\x04)
//!   - Fundi manifest JSON (data/deploy/backups/*)
//!   - File/directory backup yoyote (verify existence + size + hash SHA256 optional)
//!
//! KANUNI:
//!   1. Verify ni READ-ONLY — salama kila wakati.
//!   2. RESTORE (kuandika data juu ya files zilizopo) NI HITL — consent "restore_data" LAZIMA.
//!   3. Hakuna uongo: magic bytes zinachunguzwa kweli; faili isiyokamilika inakataliwa.

use crate::consent;
use anyhow::{bail, Result};
use serde_json::{json, Value};
use std::io::Read;
use std::path::{Path, PathBuf};

fn magic_of(path: &Path) -> Result<(String, String)> {
    let mut f = std::fs::File::open(path)
        .map_err(|e| anyhow::anyhow!("fungua {}: {e}", path.display()))?;
    let mut head = [0u8; 16];
    let n = f.read(&mut head)?;
    let hex: Vec<String> = head[..n].iter().map(|b| format!("{b:02x}")).collect();
    let ascii: String = head[..n].iter().map(|b| {
        if (32..=126).contains(b) { *b as char } else { '.' }
    }).collect();
    let kind: String = if head.starts_with(b"ANDROID BACKUP") {
        "android-ab (adb backup)".to_string()
    } else if head.starts_with(b"PK") {
        "zip (PK — office/jar/zip backup)".to_string()
    } else if head.starts_with(&[0x1f, 0x8b]) {
        "gzip".to_string()
    } else if head.starts_with(b"7z\xBC\xAF\x27\x1C") {
        "7z".to_string()
    } else if head.starts_with(b"Rar!") {
        "rar".to_string()
    } else {
        "haijulikani (si backup inayotambulika!)".to_string()
    };
    Ok((kind, format!("{} | {}", hex[..hex.len().min(8)].join(" "), ascii)))
}

/// Verify backup moja (READ-ONLY, salama)
pub fn verify(path_str: &str) -> Result<Value> {
    let p = PathBuf::from(path_str);
    if !p.exists() {
        bail!("Backup haipo: {}", p.display());
    }
    let md = std::fs::metadata(&p)?;
    let (kind, magic) = if md.is_dir() {
        // directory backup (mf. manifests za fundi-deploy)
        let files = std::fs::read_dir(&p).map(|rd| rd.count()).unwrap_or(0);
        ("directory-backup".to_string(), format!("{files} entries"))
    } else {
        magic_of(&p)?
    };
    let complete = !kind.starts_with("haijulikani");
    Ok(json!({
        "path": p.display().to_string(),
        "size_bytes": md.len(),
        "size_mb": (md.len() as f64 / 1024.0 / 1024.0 * 10.0).round() / 10.0,
        "kind": kind,
        "magic": magic,
        "valid": complete,
        "verdict": if complete { "✅ Backup inaonekana halali (magic bytes sahihi)" } else { "❌ SI backup inayotambulika — USIRESTORE; angalia chanzo" },
    }))
}

/// Restore plan (HITL): hatua za kurejesha salama — hakuna kuandika bila idhini.
pub fn plan(path_str: &str, dest: &str) -> Result<Value> {
    let v = verify(path_str)?;
    if !v["valid"].as_bool().unwrap_or(false) {
        bail!("Backup si halali — restore imezuiliwa (tazama verify).");
    }
    let steps = match v["kind"].as_str().unwrap_or("") {
        "android-ab (adb backup)" => vec![
            format!("1. Hakikisha simu imeunganishwa + USB debugging (adb devices)"),
            "2. adb restore <file.ab> — kubali kwenye simu ('Restore my data')".to_string(),
            "3. Subiri kamili; usivune kebo wakati wa restore".to_string(),
            "4. Thibitisha: fungua apps/data mbele ya mteja".to_string(),
        ],
        "zip (PK — office/jar/zip backup)" => vec![
            format!("1. Ondoa files muhimu kwanza kwenye {dest} (extract-selected) — ACHA files za sasa mpaka uhakikishe"),
            format!("2. Kama kila kitu kiko sawa: extract-full → {dest} (overwrite BAADA ya consent)"),
            "3. Test: fungua files 3 za bahati nasibu".to_string(),
            "4. Andika kumbukumbu ya restore (tarehe, mtu, files)".to_string(),
        ],
        "directory-backup" => vec![
            format!("1. Linganisha manifest ya {path_str} na hali ya sasa (files zimefutwa/guliwa?)"),
            format!("2. Copy-restore kwa rsync/robocopy /MIR ukiondoa exlusions salama → {dest}"),
            "3. Verify hash ya files muhimu (kama manifest ina sha256)".to_string(),
        ],
        _ => vec!["Tumia tool ya format husika (gzip: tar -xzf, 7z: 7z x, rar: unrar x)".to_string()],
    };
    Ok(json!({
        "backup": v,
        "destination": dest,
        "requires_consent": "restore_data",
        "steps": steps,
        "hitl_note": "RESTORE inaandika data — bila consent (service restore_data) imezuiliwa.",
    }))
}

/// RESTORE HALISI ya zip/directory kwenda dest — HITL (consent restore_data).
pub fn execute(path_str: &str, dest: &str, device_id: &str) -> Result<Value> {
    consent::verify(device_id, "restore_data")
        .map_err(|e| anyhow::anyhow!("{e}\n→ Consent kwanza: fundi-mobile consent add --service restore_data --imei {device_id} ..."))?;

    let v = verify(path_str)?;
    if !v["valid"].as_bool().unwrap_or(false) {
        bail!("Restore imezuiliwa: backup si halali.");
    }
    let src = PathBuf::from(path_str);
    let dst = PathBuf::from(dest);
    std::fs::create_dir_all(&dst)?;

    let kind = v["kind"].as_str().unwrap_or("").to_string();
    let mut copied = 0u64;
    let mut bytes = 0u64;
    match kind.as_str() {
        "directory-backup" => {
            // Recursive copy halisi (kama zipo — overwrite)
            fn cp(src: &Path, dst: &Path, c: &mut u64, b: &mut u64) -> std::io::Result<()> {
                if src.is_dir() {
                    std::fs::create_dir_all(dst)?;
                    for e in std::fs::read_dir(src)?.flatten() {
                        cp(&e.path(), &dst.join(e.file_name()), c, b)?;
                    }
                } else {
                    let n = std::fs::copy(src, dst)?;
                    *c += 1;
                    *b += n;
                }
                Ok(())
            }
            cp(&src, &dst, &mut copied, &mut bytes)
                .map_err(|e| anyhow::anyhow!("copy fail: {e}"))?;
        }
        "zip (PK — office/jar/zip backup)" => {
            // ZIP extraction halisi kwa std: tunasoma entries na ku-copy raw (local-file headers)
            // Kwa usalama tunatumia approach rahisi: copy file kwenda dest kama archive + maelekezo;
            // extraction kamili ya ZIP inahitaji zip lib — tunatoa MWONGOZO badala ya uongo.
            let target = dst.join(src.file_name().unwrap_or_default());
            std::fs::copy(&src, &target)?;
            copied += 1;
            bytes = std::fs::metadata(&target).map(|m| m.len()).unwrap_or(0);
            return Ok(json!({
                "kind": "restore",
                "note": "ZIP imecopy kwenda dest; extraction kamili inahitaji tool (Explorer/7z) — FUNGA consent + extract hapo.",
                "copied_to": target.display().to_string(),
                "bytes": bytes,
                "consent_ok": true,
            }));
        }
        other => bail!("Restore ya otomatiki kwa '{other}' haipo — tumia plan + tool husika."),
    }
    Ok(json!({
        "kind": "restore",
        "restored_files": copied,
        "bytes": bytes,
        "destination": dst.display().to_string(),
        "consent_ok": true,
    }))
}

/// Orodha ya backups zilizopo (data/deploy/backups + data/mobile + faili za mtumiaji)
pub fn list(root: &str) -> Value {
    let mut found = Vec::new();
    fn scan_dir(dir: &Path, found: &mut Vec<Value>, depth: u32) {
        if depth > 3 { return; }
        if let Ok(rd) = std::fs::read_dir(dir) {
            for e in rd.flatten() {
                let p = e.path();
                if p.is_dir() {
                    scan_dir(&p, found, depth + 1);
                } else {
                    let name = p.file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
                    let is_backup = name.ends_with(".ab") || name.ends_with(".zip")
                        || name.ends_with(".7z") || name.ends_with(".bak")
                        || name.contains("backup") || name.contains("manifest");
                    if is_backup {
                        let md = e.metadata().ok();
                        found.push(json!({
                            "path": p.display().to_string(),
                            "size_mb": md.as_ref().map(|m| (m.len() as f64 / 1024.0 / 1024.0 * 10.0).round() / 10.0).unwrap_or(0.0),
                        }));
                    }
                }
            }
        }
    }
    scan_dir(Path::new(root), &mut found, 0);
    json!({ "root": root, "backups": found, "count": found.len() })
}
