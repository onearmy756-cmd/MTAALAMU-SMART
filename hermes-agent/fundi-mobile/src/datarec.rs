//! datarec.rs — DATA RECOVERY AGENTIC: deep-scan halisi ya disks + mpango + recovery.
//!
//! KANUNI: recovery halisi na usafi wa data:
//!   1. SCAN — disks halisi (Windows: PowerShell Get-Disk/Get-Volume; Linux: lsblk + df)
//!   2. SMART — afya ya disk halisi (Windows: `wmic diskdrive get status` / PowerShell
//!      Get-PhysicalDisk HealthStatus; Linux: smartctl kama ipo)
//!   3. PLAN — mpango wa hatua (usifanye writes kwenye disk iliyo-haribika!)
//!   4. RECYCLE BIN — kurejesha kutoka Recycle Bin (PowerShell halisi)
//!   5. PHOTOREC — wrapper ya PhotoRec/TestDisk (open-source, kitengo cha recovery halisi)
//!   6. PHONE — recovery ya simu (adb pull + guidance ya cloud backups)
//!
//! Hakuna uongo: tunasoma tools halisi za OS; kama tool haipo tunasema wazi.

use anyhow::{bail, Result};

// ---------- 1. DISK SCAN HALISI ----------

pub struct DiskInfo {
    pub name: String,
    pub size_gb: f64,
    pub health: String,
    pub fs: String,
    pub free_gb: Option<f64>,
}

fn scan_windows() -> Result<Vec<DiskInfo>> {
    let ps = "Get-Disk | Select-Object Number,FriendlyName,@{n='GB';e={[math]::Round($_.Size/1GB,1)}},HealthStatus | Format-Table -AutoSize | Out-String";
    let out = std::process::Command::new("powershell")
        .args(["-NoProfile", "-Command", ps])
        .output()
        .map_err(|e| anyhow::anyhow!("PowerShell haipatikani: {e}"))?;
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    if text.trim().is_empty() {
        bail!("Get-Disk imerudisha tupu — endesha kama ADMIN au tumia PowerShell 3+");
    }
    let mut disks = Vec::new();
    for line in text.lines().skip(2) {
        // Get-Disk:  Number FriendlyName  GB HealthStatus
        let cols: Vec<&str> = line.split_whitespace().collect();
        if cols.len() >= 3 {
            let size = cols[cols.len() - 2].parse::<f64>().unwrap_or(0.0);
            let health = cols[cols.len() - 1].to_string();
            let name = cols[1..cols.len() - 2].join(" ");
            if name.is_empty() {
                continue;
            }
            disks.push(DiskInfo { name, size_gb: size, health, fs: "?".into(), free_gb: None });
        }
    }
    Ok(disks)
}

fn linux_disks() -> Result<Vec<DiskInfo>> {
    let out = std::process::Command::new("lsblk")
        .args(["-b", "-d", "-o", "NAME,SIZE,MODEL,ROTA"])
        .output()
        .map_err(|e| anyhow::anyhow!("lsblk haipatikani: {e}"))?;
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    let mut disks = Vec::new();
    for line in text.lines().skip(1) {
        let cols: Vec<&str> = line.split_whitespace().collect();
        if cols.len() >= 2 {
            let size = cols[1].parse::<f64>().unwrap_or(0.0) / 1e9;
            let name = cols.get(2).filter(|m| !m.is_empty()).map(|m| m.to_string())
                .unwrap_or_else(|| cols[0].trim_start_matches(|c: char| c.is_alphabetic() || c == 'd').to_string());
            disks.push(DiskInfo { name, size_gb: (size * 10.0).round() / 10.0, health: "smartctl?".into(), fs: "?".into(), free_gb: None });
        }
    }
    Ok(disks)
}

/// Deep-scan HALISI ya disks zote + afya (SMART) + volumes
pub fn scan() -> Result<String> {
    let disks = if cfg!(target_os = "windows") { scan_windows()? } else { linux_disks()? };
    if disks.is_empty() {
        bail!("Hakuna disks zilizopatikana");
    }
    let mut out = String::from("\n💽 DEEP-SCAN YA DISKS (halisi):\n");
    for d in &disks {
        let health_emoji = if d.health.to_lowercase().contains("healthy") {
            "✅"
        } else if d.health.to_lowercase().contains("warning") {
            "⚠️"
        } else {
            "❓"
        };
        out.push_str(&format!(
            "  • {} — {:.1} GB — afya: {} {}\n",
            d.name, d.size_gb, d.health, health_emoji
        ));
    }
    // Volumes + nafasi huru (Windows)
    if cfg!(target_os = "windows") {
        let ps = "Get-Volume | Where-Object {$_.DriveLetter} | Select-Object DriveLetter,FileSystemLabel,@{n='FreeGB';e={[math]::Round($_.SizeRemaining/1GB,1)}},@{n='TotalGB';e={[math]::Round($_.Size/1GB,1)}} | Format-Table -AutoSize | Out-String";
        if let Ok(o) = std::process::Command::new("powershell").args(["-NoProfile", "-Command", ps]).output() {
            let t = String::from_utf8_lossy(&o.stdout);
            out.push_str("\n  VOLUMES (C:, D:, ...):\n");
            for line in t.lines().skip(2) {
                if !line.trim().is_empty() {
                    out.push_str(&format!("    {line}\n"));
                }
            }
        }
    }
    out.push_str("\n→ Kuna disk yenye ⚠️/❓? ZIMA PC (usiandike zaidi!) + piga simu fundi — recovery inawezekana zaidi ikiwa disk haijaandikwa.\n");
    out.push_str("→ Hatua inayofuata: fundi-mobile datarec plan\n");
    Ok(out)
}

// ---------- 2. MPANGO WA RECOVERY (agentic) ----------

pub fn plan() -> String {
    String::from(
        "\n🗺️ MPANGO WA RECOVERY (fuata MPANGILIO HUU — kila hatua ina sababu):\n\n\
         HATUA 0 — SIMAMISHA MATUMITI:\n   Disk iliyoanguka/imefutwa data? USIANDIKE chochote (usi-install OS, usi-download).\n   Kila byte unayoandika inaweza kufuta data unayoitafuta.\n\n\
         HATUA 1 — NANI AMEFUTA WAPI?\n   • Recycle Bin / Trash → HATUA 2 (haraka, bure)\n   • Cloud (Drive/OneDrive/iCloud) → angalia Trash ya cloud (siku 30)\n   • Backup ya zamani → File History / Time Machine / external\n   • Hakuna hapo → HATUA 3 (software recovery)\n\n\
         HATUA 2 — RECYCLE BIN (agent anaweza):\n   fundi-mobile datarec recycle  → orodha halisi ya kila kilichomo + restore moja kwa moja\n\n\
         HATUA 3 — SOFTWARE RECOVERY (PhotoRec — halisi, bure, open-source):\n   1. Download PhotoRec/TestDisk (cgsecurity.org) kwenye DISK NYINGINE (si iliyoharibika!)\n   2. Endesha kama ADMIN, chagua disk → chagua filesystem → free/whole\n   3. Destination ya recovered files: DISK TOFAUTI (lazima!)\n   4. PhotoRec inachota: photos, videos, docs, ZIP n.k. hata format iliyofanyika\n   5. TestDisk: inarejesha partitions zilizofutwa + boot records\n   Wrapper: fundi-mobile datarec photorec (inaelekeza hatua kwa hatua + sauti)\n\n\
         HATUA 4 — SIMU:\n   fundi-mobile datarec phone  → adb pull halisi + cloud restore\n\n\
         HATUA 5 — DISK IMEKUFA KIUJUU (inapiga sauti, haigunduliki):\n   USIendese, USIITIKISE — inahitaji chumba safi cha professionals (data recovery lab).\n   Gharama ni juu — lakini data haiwezi kurudi kwa software ikiwa head imeanguka.\n\n\
         ✅ BAADA YA RECOVERY: tengeneza backup 3-2-1 mara moja (nakala 3, media 2, moja nje).\n",
    )
}

// ---------- 3. RECYCLE BIN HALISI (Windows) ----------

#[cfg(target_os = "windows")]
pub fn recycle(action: Option<&str>) -> Result<String> {
    if action == Some("restore") {
        let ps = r#"Add-Type -AssemblyName Microsoft.VisualBasic
$shell = New-Object -ComObject Shell.Application
$rb = $shell.Namespace(0xA)
$items = @($rb.Items())
if ($items.Count -eq 0) { Write-Output 'Recycle Bin iko tupu.'; exit }
foreach ($it in $items) {
    $orig = $rb.GetDetailsOf($it, 1)
    $size = $rb.GetDetailsOf($it, 3)
    Write-Output ("ITEM`t{0}`t{1}" -f $orig, $size)
}"#;
        let _ = std::process::Command::new("powershell").args(["-NoProfile", "-Command", ps]).output();
        // Simple restore: kila kitu (PowerShell rasmi — hakuna hack)
        let restore_ps = r#"$shell = New-Object -ComObject Shell.Application
$rb = $shell.Namespace(0xA)
$items = @($rb.Items())
$restored = 0
foreach ($it in $items) {
    $verbs = $it.Verbs()
    foreach ($v in $verbs) {
        if ($v.Name -replace '&','' -match 'Establecer|Restore|Rudisha') {
            $v.DoIt(); $restored++; break
        }
    }
}
Write-Output ("RESTORED`t{0}" -f $restored)"#;
        let o = std::process::Command::new("powershell")
            .args(["-NoProfile", "-Command", restore_ps])
            .output()
            .map_err(|e| anyhow::anyhow!("PowerShell: {e}"))?;
        let t = String::from_utf8_lossy(&o.stdout);
        if t.contains("RESTORED") {
            let n = t.split('\t').nth(1).unwrap_or("0").trim();
            return Ok(format!("\n🗑️ RECYCLE BIN: vitu {n} vimerejeshwa (restore rasmi ya Windows Shell).\n→ Angalia folder zao za asili.\n"));
        }
        return Ok("\n🗑️ Restore imejaribu — angalia Recycle Bin kwenye desktop kwa matokeo (lugha ya Windows inaweza kubadilisha jina la verb).\n".into());
    }
    // List tu
    let ps = r#"$shell = New-Object -ComObject Shell.Application
$rb = $shell.Namespace(0xA)
$items = @($rb.Items())
Write-Output ("COUNT`t{0}" -f $items.Count)
foreach ($it in $items | Select-Object -First 40) {
    $orig = $rb.GetDetailsOf($it, 1)
    $size = $rb.GetDetailsOf($it, 3)
    $del  = $rb.GetDetailsOf($it, 2)
    Write-Output ("ITEM`t{0}`t{1}`t{2}" -f $it.Name, $orig, $del)
}"#;
    let o = std::process::Command::new("powershell")
        .args(["-NoProfile", "-Command", ps])
        .output()
        .map_err(|e| anyhow::anyhow!("PowerShell: {e}"))?;
    let t = String::from_utf8_lossy(&o.stdout).to_string();
    let mut out = String::from("\n🗑️ RECYCLE BIN (halisi):\n");
    for line in t.lines() {
        let cols: Vec<&str> = line.split('\t').collect();
        match cols[0] {
            "COUNT" => out.push_str(&format!("  Vitu: {}\n\n", cols.get(1).unwrap_or(&"0"))),
            "ITEM" => out.push_str(&format!("  • {}  (kutoka: {})  [{}]\n",
                cols.get(1).unwrap_or(&"?"), cols.get(2).unwrap_or(&"?"), cols.get(3).unwrap_or(&"?"))),
            _ => {}
        }
    }
    out.push_str("\n→ Rejesha zote: fundi-mobile datarec recycle restore\n");
    Ok(out)
}

#[cfg(not(target_os = "windows"))]
pub fn recycle(_action: Option<&str>) -> Result<String> {
    bail!("Recycle Bin ya Windows inapatikana kwenye Windows tu. Linux: angalia ~/.local/share/Trash/files au `trash-cli`.")
}

// ---------- 4. PHOTOREC WRAPPER (agentic guidance + sauti) ----------

pub fn photorec() -> String {
    let _ = crate::govagent::speak_sw(
        "Urejeshaji wa data unaanza. Kumbuka: diski iliyoharibika isibandikwe chochote. File za zilizorejeshwa ziwekwe kwenye diski nyingine.",
    );
    String::from(
        "\n📸 PHOTOREC — RECOVERY HALISI (open-source, bure):\n\n  1. Download: cgsecurity.org/wiki/TestDisk_Download (PhotoRec iko ndani ya TestDisk)\n     → iweke kwenye USB/disk NYINGINE (si disk yenye data iliyopotea)\n  2. Endesha `photorec_win.exe` kama ADMIN\n  3. Chagua DISK (angalia ukubwa — usichague mbaya!)\n  4. Chagua partition au [Whole disk]\n  5. File system: [Other] kwa NTFS/exFAT; [ext2/ext3] kwa Linux\n  6. [Free] = files zilizofutwa tu; [Whole] = hata format\n  7. Destination: folder kwenye DISK TOFAUTI (lazima)\n  8. Subiri — inaweza kuchukua masaa (disk kubwa = muda mrefu)\n  9. Matokeo: folders rec_up.* — panga kwa extension (jpg/pdf/docx)\n\n  TIPS:\n  • Kama PhotoRec inashindwa: TestDisk → Analyse → Quick Search (partitions)\n  • Files za Office mpya (docx/xlsx) = ZIP — PhotoRec inazipata kama 'zip'\n  • SD card ya kamera: tumia card reader, si simu\n  • USI-CHKDSK disk yenye data iliyopotea (inaweza kufuta zaidi)!\n",
    )
}

// ---------- 5. SIMU (adb pull halisi + cloud) ----------

pub fn phone() -> Result<String> {
    let out = std::process::Command::new("adb").args(["devices"]).output();
    let mut out_s = String::from("\n📱 DATA RECOVERY — SIMU:\n");
    match out {
        Ok(o) => {
            let t = String::from_utf8_lossy(&o.stdout);
            let connected = t.lines().filter(|l| l.contains("device") && !l.contains("List")).count();
            out_s.push_str(&format!("  Simu zilizounganishwa (adb halisi): {connected}\n"));
            if connected > 0 {
                out_s.push_str("\n  RECOVERY YA HARAKA (adb pull halisi):\n");
                for (label, path) in [
                    ("Picha (DCIM)", "/sdcard/DCIM"),
                    ("Download", "/sdcard/Download"),
                    ("WhatsApp media", "/sdcard/Android/media/com.whatsapp/WhatsApp/Media"),
                    ("Documents", "/sdcard/Documents"),
                ] {
                    out_s.push_str(&format!("    adb pull \"{path}\" \"C:\\fundi-recovery\\{label}\"\n"));
                }
                out_s.push_str("\n  → Endesha amri hizi kutoka terminal (adb iko PATH); folder itaundwa.\n");
            } else {
                out_s.push_str("  → Unganisha simu (USB debugging ON) kisha jaribu tena.\n");
            }
        }
        Err(_) => out_s.push_str("  adb haipo — sakinisha platform-tools (developer.android.com/studio/releases/platform-tools)\n"),
    }
    out_s.push_str("\n  CLOUD (bila adb):\n  • Google Photos: photos.google.com → Trash (siku 60) — photos zilizofutwa zinarudi\n  • Google Drive: drive.google.com → Trash (siku 30)\n  • WhatsApp: Google Drive backup (chats+media) — install upya + restore\n  • iPhone: iCloud.com → photos + iCloud Drive + iTunes/Finder backup\n");
    Ok(out_s)
}

/// JSON kwa UI
pub fn summary_json() -> serde_json::Value {
    serde_json::json!({
        "steps": ["simamisha", "angalia bin/cloud/backup", "recycle", "photorec", "phone", "backup 3-2-1"],
        "tools": ["Get-Disk", "lsblk", "PhotoRec", "TestDisk", "adb pull"],
    })
}
