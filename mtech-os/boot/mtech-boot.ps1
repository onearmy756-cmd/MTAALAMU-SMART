# MTECH OS — Windows boot integration (HALISI)
#   Inafanywa na install.py --boot (Run as Administrator):
#     1) Sysmon (Microsoft Sysinternals, bila malipo) — inasakinishwa KIOTOMATIKI
#        na config ya MTECH: kila process inayoanza/kufa inaonekana kwenye Event Log
#        (ikiwa na kernel-level visibility, kama /dev/mtech ya Linux)
#     2) MTAALAMU SMART + Kali Tools → Startup folder (inaload inapowaka OS)
#     3) Wallpaper + branding ya MTECH (logo yake kwenye desktop)
# Kisha: boot manager ya Windows inaonyesha Windows kawaida; MTAALAMU inaanza
# pamoja na OS — hakuna dual-boot, hakuna sandbox.

$ErrorActionPreference = "Stop"
$isAdmin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()
           ).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $isAdmin) { Write-Host "ENDESHA KAMA ADMINISTRATOR (kwa --boot)" -ForegroundColor Red; exit 1 }

Write-Host "=== MTECH — Windows boot integration ===" -ForegroundColor Green

# ---------- 1) Sysmon (HALISI, bila malipo) ----------
$sysmon = "$env:ProgramFiles\Sysmon\Sysmon64.exe"
if (-not (Test-Path $sysmon)) {
    Write-Host "[1/3] Kupakua Sysmon (Microsoft Sysinternals)…"
    $zip = "$env:TEMP\Sysmon.zip"
    Invoke-WebRequest -Uri "https://download.sysinternals.com/files/Sysmon.zip" -OutFile $zip
    Expand-Archive -Path $zip -DestinationPath "$env:TEMP\Sysmon" -Force
    $conf = Join-Path $PSScriptRoot "sysmon-config.xml"
    & "$env:TEMP\Sysmon\Sysmon64.exe" -accepteula -i $conf | Write-Host
} else {
    Write-Host "[1/3] Sysmon ipo tayari — napakia config ya MTECH…"
    & $sysmon -c (Join-Path $PSScriptRoot "sysmon-config.xml") | Write-Host
}
Write-Host "  ✓ Sysmon inaona KILA process (kernel-level) — Event Log: 'Microsoft-Windows-Sysmon/Operational'"

# ---------- 2) Startup: MTAALAMU inaload inapowaka OS ----------
Write-Host "[2/3] Startup shortcuts (inaload na OS)…"
$startup = [Environment]::GetFolderPath("Startup")
$py = "$env:LOCALAPPDATA\MTECH\venv\Scripts\python.exe"
$ws = New-Object -ComObject WScript.Shell
$s1 = $ws.CreateShortcut("$startup\MTAALAMU SMART.lnk")
$s1.TargetPath = $py
$s1.Arguments  = "$env:LOCALAPPDATA\MTECH\gui\mtech_shell.py"
$s1.IconLocation = "$env:LOCALAPPDATA\MTECH\desktop\mtaalamu.svg"
$s1.Save()
$s2 = $ws.CreateShortcut("$startup\Kali Tools.lnk")
$s2.TargetPath = $py
$s2.Arguments  = "$env:LOCALAPPDATA\MTECH\desktop\kali-tools-window.py"
$s2.Save()
Write-Host "  ✓ Startup → $startup"

# ---------- 3) Branding: logo + wallpaper ya MTECH ----------
Write-Host "[3/3] Branding ya MTECH (logo/desktop)…"
$wall = "$env:LOCALAPPDATA\MTECH\desktop\mtech-wallpaper.png"
if (Test-Path $wall) {
    Set-ItemProperty -Path "HKCU:\Control Panel\Desktop" -Name Wallpaper -Value $wall
    rundll32.exe user32.dll,UpdatePerUserSystemParameters | Out-Null
    Write-Host "  ✓ Wallpaper ya MTECH imewekwa"
}
Write-Host ""
Write-Host "✅ IMEKAMILIKA — restart: MTAALAMU inaanza na Windows, Sysmon inaona kila kitu." -ForegroundColor Green
