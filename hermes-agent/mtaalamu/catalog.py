"""MTAALAMU SMART — Unified Solver: catalog ya operesheni zote HALISI.

Kila op ina: OS zinazoungwa mkono, amri HALISI za OS husika, risk (HITL),
kama inahitaji mtandao, bei (credits), na maelezo sw/en.
Kanuni ya MTAALAMU: LLM haiendeshi amri — catalog hii ndiyo inayoendesha;
LLM inachagua op na kueleza (hakuna hallucination ya amri).
"""
from dataclasses import dataclass, field
from typing import Callable


@dataclass
class Op:
    id: str
    group: str                      # storage|network|apps|users|office|system|security|hardware
    name_sw: str
    name_en: str
    os: tuple                       # ("linux","windows","macos")
    commands: dict                  # family -> list ya amri HALISI (str au callable)
    risk: str                       # LOW | MEDIUM | HIGH
    offline: bool                   # True = inafanya kazi bila mtandao
    credits: int                    # gharama (kwa metering ya license)
    ask: tuple = field(default_factory=tuple)   # maswali ya HITL (mteja anachagua)
    notes_sw: str = ""
    notes_en: str = ""


# ------------------------------------------------------------------ helpers
def _ps(script: str) -> str:
    return f"powershell -NoProfile -Command \"{script}\""


# ------------------------------------------------------------------ CATALOG
CATALOG: list[Op] = [
    # ============ STORAGE / DISK ============
    Op("disk.cleanup", "storage", "Disk cleanup (temp, cache, recycle)",
       "Disk cleanup (temp, cache, recycle bin)",
       ("linux", "windows", "macos"),
       {
        "linux": ["rm -rf /tmp/* 2>/dev/null; apt-get clean 2>/dev/null; journalctl --vacuum-time=3d 2>/dev/null; echo OK"],
        "windows": [_ps("Remove-Item $env:TEMP\\* -Recurse -Force -ErrorAction SilentlyContinue; Remove-Item 'C:\\Windows\\Temp\\*' -Recurse -Force -ErrorAction SilentlyContinue; Clear-RecycleBin -Force -ErrorAction SilentlyContinue; Write-Output OK")],
        "macos": ["rm -rf /private/var/folders/* 2>/dev/null; echo OK"],
       },
       "MEDIUM", True, 1,
       ask=("Je, nifute temp + cache + recycle bin zote?", "Delete all temp/cache/recycle? (ndio/hapana)"),
       notes_sw="Inaondoa files za muda tu; haimgusi documents.", notes_en="Removes temp files only; documents untouched."),
    Op("disk.partition.list", "storage", "Onyesha partitions zote",
       "List all partitions",
       ("linux", "windows", "macos"),
       {
        "linux": ["lsblk -o NAME,SIZE,TYPE,FSTYPE,MOUNTPOINT"],
        "windows": [_ps("Get-Partition | Format-Table -AutoSize")],
        "macos": ["diskutil list"],
       }, "LOW", True, 1),
    Op("disk.partition.create", "storage", "Tengeneza partition mpya",
       "Create a new partition",
       ("linux", "windows", "macos"),
       {
        "linux": [],   # inaundwa runtime: parted /dev/sdX mkpart ... (disk + size kutoka maswali)
        "windows": [], # New-Partition -DiskNumber N -Size ... -AssignDriveLetter
        "macos": [],   # diskutil apfs addPartition disk0 <size> APFS MTECH
       }, "HIGH", True, 5,
       ask=("Disk ipi? (mf: 0 au /dev/sda) na size? (mf: 50GB)",
            "Which disk? (e.g. 0 or /dev/sda) and size? (e.g. 50GB)")),
    Op("disk.health", "storage", "Afya ya disk (SMART)",
       "Disk health (SMART)",
       ("linux", "windows", "macos"),
       {
        "linux": ["smartctl -H /dev/sda 2>/dev/null || cat /sys/block/sda/device/smart_status 2>/dev/null || echo 'smartmontools haipo: sudo apt install smartmontools'"],
        "windows": [_ps("Get-PhysicalDisk | Select FriendlyName,MediaType,HealthStatus,OperationalStatus | Format-Table -AutoSize")],
        "macos": ["diskutil info disk0 | grep -i smart"],
       }, "LOW", True, 1),
    Op("disk.defrag", "storage", "Defragment (SSD: TRIM)",
       "Defragment (SSD: TRIM)",
       ("windows", "linux"),
       {
        "windows": [_ps("Optimize-Volume -DriveLetter C -Defrag -Verbose")],
        "linux": ["fstrim -av 2>/dev/null || echo 'fstrim haipo'"],
       }, "MEDIUM", True, 2),

    # ============ NETWORK ============
    Op("net.diagnose", "network", "Gundua matatizo ya network (ip, dns, gateway, ping)",
       "Diagnose network problems (ip, dns, gateway, ping)",
       ("linux", "windows", "macos"),
       {
        "linux": ["ip addr; ip route; cat /etc/resolv.conf; ping -c 3 8.8.8.8; ping -c 3 1.1.1.1"],
        "windows": [_ps("ipconfig /all; Get-NetRoute -DestinationPrefix 0.0.0.0/0 | Format-Table; Test-Connection 8.8.8.8 -Count 3")],
        "macos": ["ifconfig; netstat -rn | head -12; ping -c 3 8.8.8.8"],
       }, "LOW", False, 1),
    Op("net.dns.reset", "network", "Rekebisha DNS (flush + public DNS)",
       "Fix DNS (flush + public DNS)",
       ("linux", "windows", "macos"),
       {
        "windows": [_ps("ipconfig /flushdns; Set-DnsClientServerAddress -InterfaceAlias (Get-NetAdapter | Where Status -eq Up | Select -First 1).Name -ServerAddresses 1.1.1.1,8.8.8.8; Write-Output DNS-OK")],
        "linux": ["systemd-resolve --flush-caches 2>/dev/null; echo 'nameserver 1.1.1.1' | tee /etc/resolv.conf.bak >/dev/null 2>&1 || echo OK"],
        "macos": ["sudo dscacheutil -flushcache; sudo killall -HUP mDNSResponder"],
       }, "MEDIUM", False, 2),
    Op("net.wifi.reconnect", "network", "Ongesha upya WiFi",
       "Reconnect WiFi",
       ("linux", "windows", "macos"),
       {
        "windows": [_ps("netsh wlan disconnect; netsh wlan connect name=(netsh wlan show profiles | Select-String 'All User Profile' | Select-Object -First 1) -ErrorAction SilentlyContinue")],
        "linux": ["nmcli networking off && sleep 1 && nmcli networking on"],
        "macos": ["networksetup -setairportpower en0 off && networksetup -setairportpower en0 on"],
       }, "MEDIUM", True, 1),
    Op("net.reset.stack", "network", "Reset stack ya network (winsock/nic)",
       "Reset network stack (winsock/nic)",
       ("windows", "linux"),
       {
        "windows": [_ps("netsh winsock reset; netsh int ip reset; ipconfig /release; ipconfig /renew; Write-Output RESET-OK")],
        "linux": ["nmcli networking off; sleep 2; nmcli networking on"],
       }, "HIGH", True, 2,
       ask=("Reset inavunja connection kwa muda — endelea?", "Reset drops the connection briefly — continue?")),

    # ============ APPS / UPDATES ============
    Op("app.install", "apps", "Sakinisha app (package halisi ya OS)",
       "Install app (native OS package)",
       ("linux", "windows", "macos"),
       {
        "linux": [],   # apt install <jina> (jina kutoka maswali)
        "windows": [], # winget install <id>
        "macos": [],   # brew install <jina>
       }, "MEDIUM", False, 2,
       ask=("Jina la app? (mf: vlc / Spotify / gimp)", "App name? (e.g. vlc / Spotify / gimp)")),
    Op("app.update.all", "apps", "Update apps zote",
       "Update all apps",
       ("linux", "windows", "macos"),
       {
        "linux": ["apt-get update -qq && apt-get upgrade -y -qq"],
        "windows": [_ps("winget upgrade --all --silent")],
        "macos": ["brew update && brew upgrade"],
       }, "MEDIUM", False, 2),
    Op("app.uninstall", "apps", "Ondoa app",
       "Uninstall app",
       ("linux", "windows", "macos"),
       {
        "linux": [], "windows": [], "macos": [],
       }, "HIGH", True, 2,
       ask=("Jina la app ya kuondoa?", "App to remove?")),

    # ============ USERS ============
    Op("user.create", "users", "Tengeneza user account",
       "Create a user account",
       ("linux", "windows", "macos"),
       {
        "linux": [], "windows": [], "macos": [],
       }, "HIGH", True, 3,
       ask=("Jina la user mpya? (na admin? ndio/hapana)", "New username? (admin? yes/no)")),
    Op("user.password.change", "users", "Badilisha password ya user",
       "Change a user's password",
       ("linux", "windows", "macos"),
       {
        "linux": [], "windows": [], "macos": [],
       }, "HIGH", True, 3,
       ask=("User gani? (password itaulizwa kwa siri)", "Which user? (password asked securely)")),
    Op("user.list", "users", "Onyesha users wote",
       "List all users",
       ("linux", "windows", "macos"),
       {
        "linux": ["cut -d: -f1,3 /etc/passwd | awk -F: '$2>=1000 || $1==\"root\"'"],
        "windows": [_ps("Get-LocalUser | Select Name,Enabled,LastLogon | Format-Table -AutoSize")],
        "macos": ["dscl . -list /Users | grep -v '^_'"],
       }, "LOW", True, 1),

    # ============ SYSTEM (Task Manager / Device Manager / Disk Manager) ============
    Op("sys.taskmanager", "system", "Task Manager (processes zote live)",
       "Task Manager (all processes live)",
       ("linux", "windows", "macos"),
       {
        "linux": ["top -b -n 1 | head -25"],
        "windows": [_ps("Get-Process | Sort CPU -Descending | Select -First 20 Name,Id,CPU,WorkingSet | Format-Table -AutoSize")],
        "macos": ["ps aux | sort -nrk 3 | head -20"],
       }, "LOW", True, 1),
    Op("sys.process.kill", "system", "Maliza process (PID au jina)",
       "Kill a process (PID or name)",
       ("linux", "windows", "macos"),
       {
        "linux": [], "windows": [], "macos": [],
       }, "HIGH", True, 1,
       ask=("PID au jina la process?", "PID or process name?")),
    Op("sys.devmgr", "system", "Device Manager (hardware + drivers + issues)",
       "Device Manager (hardware + drivers + problems)",
       ("windows", "linux"),
       {
        "windows": [_ps("Get-PnpDevice | Where Status -ne OK | Format-Table FriendlyName,Class,Status -AutoSize; Write-Output '--- zote:'; Get-PnpDevice -PresentOnly | Select -First 25 FriendlyName,Class | Format-Table -AutoSize")],
        "linux": ["lspci -k 2>/dev/null | head -30; lsusb 2>/dev/null"],
       }, "LOW", True, 1),
    Op("sys.diskmgr", "system", "Disk Manager (volumes + spaces)",
       "Disk Manager (volumes + spaces)",
       ("windows", "linux", "macos"),
       {
        "windows": [_ps("Get-Volume | Format-Table DriveLetter,FileSystemLabel,SizeRemaining,Size -AutoSize")],
        "linux": ["df -h; echo ---; lsblk"],
        "macos": ["df -h"],
       }, "LOW", True, 1),
    Op("sys.startup.apps", "system", "Apps zinazoanza na OS (startup)",
       "Apps that start with the OS (startup)",
       ("windows", "linux", "macos"),
       {
        "windows": [_ps("Get-CimInstance Win32_StartupCommand | Select Name,Command,Location | Format-Table -AutoSize")],
        "linux": ["ls ~/.config/autostart/ /etc/xdg/autostart/ 2>/dev/null"],
        "macos": ["osascript -e 'tell application \"System Events\" to get the name of every login item' 2>/dev/null || echo '(osascript inahitaji ruhusa)'"],
       }, "LOW", True, 1),

    # ============ OFFICE ============
    Op("office.diagnose", "office", "Gundua matatizo ya Microsoft Office",
       "Diagnose Microsoft Office problems",
       ("windows", "macos"),
       {
        "windows": [_ps("$p=Get-Process WINWORD,EXCEL,POWERPNT,OUTLOOK -ErrorAction SilentlyContinue; if($p){$p|Select Name,Id,Responding|Format-Table}else{Write-Output 'Office haifanyi kazi sasa'}; Test-Path 'C:\\Program Files\\Microsoft Office'")],
        "macos": ["ls /Applications | grep -i -E 'word|excel|powerpoint|outlook' || echo 'Office haijasakinishwa'"],
       }, "LOW", True, 1),
    Op("office.repair", "office", "Rekebisha Office (Quick/Online repair)",
       "Repair Office (Quick/Online repair)",
       ("windows",),
       {
        "windows": [_ps("Write-Output 'Office repair: Appwiz.cpl -> Microsoft Office -> Change -> Quick Repair (GUI inafunguliwa)'; Start-Process appwiz.cpl")],
       }, "MEDIUM", True, 3,
       ask=("Quick Repair (bila mtandao) au Online Repair?", "Quick Repair (offline) or Online Repair?")),
    Op("office.license.check", "office", "Hali ya leseni ya Office",
       "Office license status",
       ("windows", "macos"),
       {
        "windows": [_ps("cscript \"C:\\Program Files\\Microsoft Office\\Office16\\OSPP.VBS\" /dstatus 2>$null | Select-String -Pattern 'LICENSE|ERROR'")],
        "macos": ["ls /Applications/Microsoft\\ Office\\ SP2/ 2>/dev/null || echo 'angalia kwenye Word → Account'"],
       }, "LOW", True, 1),

    # ============ SECURITY / ANTIVIRUS ============
    Op("av.scan", "security", "Antivirus scan (Defender/clamav halisi)",
       "Antivirus scan (real Defender/clamav)",
       ("windows", "linux", "macos"),
       {
        "windows": [_ps("Start-MpScan -ScanType QuickScan; Get-MpThreatDetection | Select -First 10 | Format-Table -AutoSize")],
        "linux": ["clamscan -ri --max-filesize=200M /home 2>/dev/null || echo 'clamav haipo: sudo apt install clamav'"],
        "macos": ["clamscan -ri ~/Downloads 2>/dev/null || echo 'clamav haipo: brew install clamav'"],
       }, "MEDIUM", True, 3,
       ask=("Scan ya haraka (Downloads/Temp) au nzima?", "Quick scan (Downloads/Temp) or full?")),
    Op("av.status", "security", "Hali ya antivirus",
       "Antivirus status",
       ("windows", "linux"),
       {
        "windows": [_ps("Get-MpComputerStatus | Select AntivirusEnabled,RealTimeProtectionEnabled,AntivirusSignatureLastUpdated | Format-List")],
        "linux": ["systemctl is-active clamav-freshclam 2>/dev/null || echo 'clamav haifanyi kazi'"],
       }, "LOW", True, 1),

    # ============ DRIVERS ============
    Op("driver.update", "system", "Update drivers (Windows Update / apt firmware)",
       "Update drivers (Windows Update / apt firmware)",
       ("windows", "linux"),
       {
        "windows": [_ps("Write-Output 'Driver update kwa Windows Update:'; (New-Object -ComObject Microsoft.Update.Session).CreateUpdateSearcher().Search(\"IsInstalled=0 and Type='Driver'\").Updates | Select Title")],
        "linux": ["apt-get install -y linux-firmware 2>/dev/null && update-initramfs -u 2>/dev/null; echo OK"],
       }, "MEDIUM", False, 3),
    Op("driver.list", "system", "Orodha ya drivers + toleo",
       "List drivers + versions",
       ("windows", "linux"),
       {
        "windows": [_ps("Get-CimInstance Win32_PnPSignedDriver | Select DeviceName,DriverVersion | Select -First 30 | Format-Table -AutoSize")],
        "linux": ["lspci -k | grep -A2 -i 'kernel driver'"],
       }, "LOW", True, 1),

    # ============ FILES (kama binadamu anavyofanya) ============
    Op("file.create.folder", "files", "Tengeneza folder",
       "Create a folder", ("linux", "windows", "macos"),
       {"linux": [], "windows": [], "macos": []}, "LOW", True, 1,
       ask=("Njia ya folder mpya?", "Path of the new folder?")),
    Op("file.delete", "files", "Futa file/folder (kwa kibali)",
       "Delete file/folder (with consent)", ("linux", "windows", "macos"),
       {"linux": [], "windows": [], "macos": []}, "HIGH", True, 1,
       ask=("Njia kamili ya kufuta?", "Full path to delete?")),

    # ============ HARDWARE DIAGNOSIS ============
    Op("hw.scan", "hardware", "Scan hardware yote (CPU/RAM/disk/battery/thermal)",
       "Scan all hardware (CPU/RAM/disk/battery/thermal)",
       ("linux", "windows", "macos"),
       {
        "linux": ["lscpu | head -12; free -h; smartctl -H /dev/sda 2>/dev/null; cat /sys/class/power_supply/BAT0/capacity 2>/dev/null; sensors 2>/dev/null | head -8 || echo 'lm-sensors haipo'"],
        "windows": [_ps("Get-CimInstance Win32_Processor | Select Name,LoadPercentage; Get-CimInstance Win32_PhysicalMemory | Select Capacity; Get-PhysicalDisk | Select FriendlyName,HealthStatus; Get-CimInstance Win32_Battery | Select EstimatedChargeRemaining")],
        "macos": ["system_profiler SPHardwareDataType | head -20"],
       }, "LOW", True, 1),
    Op("hw.battery.report", "hardware", "Ripoti kamili ya betri (health, cycles)",
       "Full battery report (health, cycles)",
       ("windows", "linux", "macos"),
       {
        "windows": [_ps("powercfg /batteryreport /output $env:TEMP\\battery.html; Write-Output ('ripoti: '+$env:TEMP+'\\battery.html')")],
        "linux": ["cat /sys/class/power_supply/BAT*/capacity 2>/dev/null; cat /sys/class/power_supply/BAT*/status 2>/dev/null"],
        "macos": ["system_profiler SPPowerDataType | grep -E 'Cycle|Condition|Charge'"],
       }, "LOW", True, 1),
]

CATALOG_BY_ID = {op.id: op for op in CATALOG}
GROUPS = sorted({op.group for op in CATALOG})


def ops_for_os(family: str) -> list:
    return [op for op in CATALOG if family in op.os]


def build_command(op: Op, family: str, answers: dict) -> list:
    """Tunga amri HALISI kwa kutumia majibu ya mteja (HITL) — hakuna hallucination."""
    cmds = list(op.commands.get(family, []))
    aid = op.id
    if aid == "disk.partition.create":
        disk = answers.get("disk", "0"); size = answers.get("size", "50GB")
        if family == "linux":
            cmds = [f"parted -s {disk} mkpart primary 0% {size} 2>/dev/null || echo 'tumia: sudo parted {disk} mkpart primary 0% {size}'"]
        elif family == "windows":
            cmds = [_ps(f"New-Partition -DiskNumber {disk} -Size {size} -AssignDriveLetter | Format-Table -AutoSize")]
        else:
            cmds = [f"diskutil apfs addPartition disk0 {size} APFS MTECH 2>/dev/null || echo 'angalia: diskutil list'"]
    elif aid == "app.install":
        app = answers.get("app", "")
        if app:
            if family == "linux": cmds = [f"apt-get install -y {app} 2>/dev/null || echo 'package haipo: {app}'"]
            elif family == "windows": cmds = [_ps(f"winget install --id {app} --silent --accept-package-agreements --accept-source-agreements")]
            else: cmds = [f"brew install {app}"]
    elif aid == "app.uninstall":
        app = answers.get("app", "")
        if app:
            if family == "linux": cmds = [f"apt-get remove -y {app}"]
            elif family == "windows": cmds = [_ps(f"winget uninstall --id {app} --silent")]
            else: cmds = [f"brew uninstall {app}"]
    elif aid == "user.create":
        u = answers.get("user", "mtechuser"); admin = str(answers.get("admin", "ndio")).lower() in ("ndio","yes","y","1")
        if family == "linux":
            cmds = [f"useradd -m {u} && echo '{u} IMETENGENEZWA'"] + ([f"usermod -aG sudo {u}"] if admin else [])
        elif family == "windows":
            cmds = [_ps(f"New-LocalUser -Name {u} -NoPassword -AccountNeverExpires | Format-List")] + ([_ps(f"Add-LocalGroupMember -Group Administrators -Member {u}")] if admin else [])
        else:
            cmds = [f"sysadminctl -addUser {u}"]
    elif aid == "user.password.change":
        u = answers.get("user", "")
        if u:
            if family == "linux": cmds = [f"passwd {u}"]
            elif family == "windows": cmds = [_ps(f"net user {u} *")]
            else: cmds = [f"dscl . -passwd /Users/{u}"]
    elif aid == "sys.process.kill":
        t = answers.get("target", "")
        if t:
            if t.isdigit():
                if family == "windows": cmds = [_ps(f"Stop-Process -Id {t} -Force")]
                else: cmds = [f"kill {t}"]
            else:
                if family == "windows": cmds = [_ps(f"Stop-Process -Name {t} -Force -ErrorAction SilentlyContinue")]
                else: cmds = [f"pkill -f {t} 2>/dev/null || echo 'haipo'"]
    elif aid == "file.create.folder":
        p = answers.get("path", "")
        if p:
            if family == "windows": cmds = [_ps(f"New-Item -ItemType Directory -Force -Path '{p}' | Select FullName")]
            else: cmds = [f"mkdir -p '{p}' && echo OK"]
    elif aid == "file.delete":
        p = answers.get("path", "")
        if p:
            if family == "windows": cmds = [_ps(f"Remove-Item -Recurse -Force -Path '{p}'")]
            else: cmds = [f"rm -rf '{p}' && echo OK"]
    return cmds
