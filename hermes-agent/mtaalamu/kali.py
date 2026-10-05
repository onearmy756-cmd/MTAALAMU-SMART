"""MTAALAMU SMART — KALI BRIDGE (Kali Linux HALISI imeunganishwa na MTECH OS).

Kila kitu cha OS kinaonekana na kinafanya kazi HALISI (hakuna kubuni):
  • ZANA za Kali zote — kutoka menyu/desktop entries halisi za Kali
    (Information Gathering, Vulnerability Analysis, Web Application,
    Password Attacks, Exploitation, Sniffing, Forensics, Wireless…) —
    kila zana inazinduliwa kwenye terminal halisi
  • TERMINALS ZOTE (command prompts) — xfce4-terminal, qterminal, konsole,
    gnome-terminal, xterm, alacritty, kitty, tilix, terminator, guake…
    fungua lolote kwa amri yoyote
  • DRIVERS — lspci/lsusb/lsmod (Linux) · pnputil (Windows) · macOS
  • PARTITIONS — lsblk (Linux) · Get-Disk (Windows) · diskutil (macOS)
  • SERVICES — systemctl / Get-Service / launchctl

Tumia:
  python3 -m mtaalamu.kali                 # ramani KAMILI ya Kali/OS (JSON)
  python3 -m mtaalamu.kali tools           # zana zote za Kali
  python3 -m mtaalamu.kali open nmap       # fungua zana kwenye terminal
  python3 -m mtaalamu.kali term            # fungua terminal mpya

API (mtaalamu serve):
  GET  /api/kali                       → integrate()
  POST /api/kali/terminal {"cmd"?}     → fungua terminal mpya
  POST /api/kali/tool {"exec": "nmap"} → fungua zana kwenye terminal
"""
import json
import os
import platform
import shlex
import shutil
import subprocess
from pathlib import Path

FAMILY = "windows" if os.name == "nt" else ("macos" if platform.system() == "Darwin" else "linux")

# ---------------------------------------------------------------- amri halisi
def _run(cmd: str, timeout: int = 10) -> tuple[int, str]:
    try:
        p = subprocess.run(cmd, shell=True, capture_output=True, text=True, timeout=timeout)
        return p.returncode, (p.stdout or p.stderr or "").strip()
    except (OSError, subprocess.TimeoutExpired) as e:
        return -1, str(e)[:200]


# ---------------------------------------------------------------- Kali/OS info
def os_release() -> dict:
    out = {}
    f = Path("/etc/os-release")
    if f.exists():
        for line in f.read_text().splitlines():
            if "=" in line:
                k, v = line.split("=", 1)
                out[k.strip()] = v.strip().strip('"')
    return out


def is_kali() -> bool:
    r = os_release()
    return r.get("ID", "") == "kali" or "kali" in r.get("PRETTY_NAME", "").lower()


# ---------------------------------------------------------------- desktop entries
def _parse_desktop(path: Path) -> dict:
    """Parser nyepesi ya .desktop (stdlib tu, hakuna configparser quirks)."""
    entry: dict = {}
    in_main = False
    try:
        for line in path.read_text(errors="replace").splitlines():
            s = line.strip()
            if s.startswith("["):
                in_main = s == "[Desktop Entry]"
                continue
            if not in_main or "=" not in s:
                continue
            k, v = s.split("=", 1)
            entry.setdefault(k.strip(), v.strip())
    except OSError:
        pass
    return entry


# Kali categories (ids kutoka desktop-directories za Kali) → majina mazuri
KALI_CATS = {
    "01-info-gathering": "Information Gathering",
    "02-vulnerability-analysis": "Vulnerability Analysis",
    "03-web-application-analysis": "Web Application Analysis",
    "04-database-assessment": "Database Assessment",
    "05-password-attacks": "Password Attacks",
    "06-wireless-attacks": "Wireless Attacks",
    "07-reverse-engineering": "Reverse Engineering",
    "08-exploitation-tools": "Exploitation Tools",
    "09-sniffing-spoofing": "Sniffing & Spoofing",
    "10-post-exploitation": "Post Exploitation",
    "11-forensics": "Forensics",
    "12-reporting": "Reporting Tools",
    "13-social-engineering": "Social Engineering Tools",
    "14-hardware": "Hardware Attacks",
    "15-hardware": "Hardware Attacks",
}


def _app_dirs() -> list[Path]:
    dirs = [Path("/usr/local/share/applications"), Path("/usr/share/applications"),
            Path.home() / ".local/share/applications"]
    if FAMILY == "windows":
        dirs.append(Path(os.environ.get("APPDATA", "")) / "Microsoft/Windows/Start Menu/Programs")
        dirs.append(Path(os.environ.get("ProgramData", "")) / "Microsoft/Windows/Start Menu/Programs")
    elif FAMILY == "macos":
        dirs = [Path("/Applications"), Path.home() / "Applications"]
    return [d for d in dirs if d.is_dir()]


def tools() -> list[dict]:
    """Zana ZOTE za Kali zilizosakinishwa (kutoka menyu halisi za Kali)."""
    out: dict[str, dict] = {}
    for d in _app_dirs():
        for f in sorted(d.glob("*.desktop")):
            e = _parse_desktop(f)
            if not e or e.get("NoDisplay", "").lower() in ("true", "1"):
                continue
            cats = [c.strip().lower() for c in e.get("Categories", "").split(";") if c.strip()]
            kali_cat = next((c for c in cats if c.startswith("kali-")), None)
            if not kali_cat:
                continue
            cat_id = kali_cat.replace("kali-", "", 1)
            pretty = KALI_CATS.get(cat_id, cat_id.replace("-", " ").title())
            exe = e.get("Exec", "").replace(" %", "").strip()
            exe0 = exe.split()[0] if exe else ""
            out[f.name] = {
                "name": e.get("Name", f.stem), "exec": exe0 or exe,
                "cmd": exe, "category": pretty, "cat_id": cat_id,
                "icon": e.get("Icon", ""),
                "installed": bool(exe0) and (shutil.which(exe0) is not None or exe0 in ("msfconsole", "burpsuite", "autopsy", "cherrytree", "ghidra", "wireshark")),
            }
    return sorted(out.values(), key=lambda x: (x["cat_id"], x["name"].lower()))


def kali_packages() -> dict:
    """Metapackages + packages za Kali zilizosakinishwa (zana level ya package)."""
    code, out = _run("dpkg -l 2>/dev/null | grep -c '^ii'")
    total = out.strip() or "0" if code == 0 else "?"
    code, kt = _run("dpkg -l 2>/dev/null | grep '^ii' | grep -cE 'kali-|nmap|metasploit|wireshark|burpsuite|hashcat|aircrack|sqlmap|john|hydra'")
    return {"packages_total": total, "kali_security_packages": kt.strip() if code == 0 else "?"}


# ---------------------------------------------------------------- TERMINALS ZOTE
# kila terminal inayopatikana kwenye mfumo (command prompts zote zinaonekana)
TERMINALS = [
    ("xfce4-terminal", "Xfce Terminal"), ("qterminal", "QTerminal"),
    ("konsole", "Konsole"), ("gnome-terminal", "GNOME Terminal"),
    ("mate-terminal", "MATE Terminal"), ("tilix", "Tilix"),
    ("terminator", "Terminator"), ("guake", "Guake"),
    ("alacritty", "Alacritty"), ("kitty", "Kitty"),
    ("st", "st"), ("stterm", "st"), ("foot", "Foot"),
    ("sakura", "Sakura"), ("lxterminal", "LXTerminal"),
    ("deepin-terminal", "Deepin Terminal"), ("urxvt", "rxvt-unicode"),
    ("xterm", "Xterm"), ("uxterm", "Uxterm"),
    ("wt", "Windows Terminal"), ("powershell", "PowerShell"), ("cmd", "cmd.exe"),
]


def terminals() -> list[dict]:
    out = []
    for tid, name in TERMINALS:
        path = shutil.which(tid)
        if path:
            out.append({"id": tid, "name": name, "path": path})
    if FAMILY == "macos":
        out.append({"id": "macos-terminal", "name": "Terminal.app", "path": "/System/Applications/Utilities/Terminal.app"})
    return out


def open_terminal(cmd: str = "", terminal: str = "") -> dict:
    """Fungua command prompt halisi (na amri hiari — dirisha halisi linafunguka)."""
    terms = terminals()
    if not terms:
        return {"ok": False, "error": "Hakuna terminal iliyopatikana kwenye OS hii"}
    tid = terminal or os.environ.get("MTECH_TERMINAL", "") or terms[0]["id"]
    if not any(t["id"] == tid for t in terms):
        tid = terms[0]["id"]

    if FAMILY == "macos":
        script = f'tell application "Terminal" to do script "{cmd}"' if cmd else None
        argv = (["osascript", "-e", script] if script else ["open", "-a", "Terminal"])
    elif tid in ("wt", "powershell", "cmd"):
        if tid == "wt":
            argv = ["wt"] + ([cmd] if cmd else [])
        elif tid == "powershell":
            argv = ["powershell", "-NoExit", "-Command", cmd or "$PSVersionTable"]
        else:
            argv = ["cmd", "/k"] + ([cmd] if cmd else [])
    else:
        keep = f"{cmd}; exec bash" if cmd else "exec bash"
        argv_map = {
            "xfce4-terminal": ["xfce4-terminal", "-x", "bash", "-c", keep],
            "qterminal": ["qterminal", "-e", "bash", "-c", keep],
            "konsole": ["konsole", "-e", "bash", "-c", keep],
            "gnome-terminal": ["gnome-terminal", "--", "bash", "-c", keep],
            "mate-terminal": ["mate-terminal", "-e", "bash", "-c", keep],
            "tilix": ["tilix", "-e", "bash", "-c", keep],
            "terminator": ["terminator", "-e", "bash", "-c", keep],
            "guake": ["guake", "-e", keep],
            "lxterminal": ["lxterminal", "-e", "bash", "-c", keep],
            "sakura": ["sakura", "-e", "bash", "-c", keep],
            "deepin-terminal": ["deepin-terminal", "-e", "bash", "-c", keep],
        }
        argv = argv_map.get(tid, [tid, "-e", "bash", "-c", keep])
    try:
        kwargs: dict = {}
        if FAMILY != "windows":
            kwargs["start_new_session"] = True
        subprocess.Popen(argv, stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL,
                         stderr=subprocess.DEVNULL, **kwargs)
        return {"ok": True, "terminal": tid, "cmd": cmd or "(shell mpya)", "argv": argv[:2]}
    except OSError as e:
        return {"ok": False, "error": f"{type(e).__name__}: {e}", "terminal": tid}


def launch_tool(exec_cmd: str, terminal: str = "") -> dict:
    """Zindua zana ya Kali kwenye terminal halisi (mf: nmap, john, msfconsole)."""
    exe = (exec_cmd or "").strip()
    if not exe:
        return {"ok": False, "error": "exec inahitajika"}
    exe0 = shlex.split(exe)[0] if exe else ""
    gui = {"wireshark", "burpsuite", "autopsy", "cherrytree", "ghidra", "zaproxy"}
    if exe0 in gui:  # GUI apps zinafungua zenyewe — bila terminal
        try:
            kwargs: dict = {"start_new_session": True} if FAMILY != "windows" else {}
            subprocess.Popen(shlex.split(exe), **kwargs)
            return {"ok": True, "gui": True, "cmd": exe}
        except OSError as e:
            return {"ok": False, "error": str(e)}
    return open_terminal(exe, terminal)


# ---------------------------------------------------------------- DRIVERS
def drivers() -> dict:
    """Drivers halisi za vifaa (pci/usb/moduli/firmware)."""
    if FAMILY == "linux":
        _, pci = _run("lspci -k 2>/dev/null | grep -A2 -E '^[0-9a-f]{2}:' | head -60")
        _, usb = _run("lsusb 2>/dev/null | head -30")
        _, mods = _run("lsmod 2>/dev/null | head -30")
        _, fw = _run("dmesg 2>/dev/null | grep -ciE 'firmware' || true")
        return {"pci": pci[:1500] or "(lspci haipo)", "usb": usb[:800] or "(lsusb haipo)",
                "modules": mods[:800] or "(lsmod haipo)", "firmware_events": fw.strip()}
    if FAMILY == "windows":
        _, drv = _run("pnputil /enum-drivers 2>nul | findstr /i \"PublishedName ClassName\" | head -40")
        _, pci = _run("powershell -NoProfile -Command \"Get-PnpDevice | Where-Object Status -eq 'OK' | Select-Object -First 25 Class,FriendlyName | Format-Table\"")
        return {"drivers": drv[:1500], "devices": pci[:1500]}
    _, ext = _run("system_profiler SPExtensionsDataType 2>/dev/null | grep -E 'Identifier|Loaded' | head -40")
    _, usb = _run("system_profiler SPUSBDataType 2>/dev/null | grep -E 'Product ID|Manufacturer' | head -20")
    return {"extensions": ext[:1500], "usb": usb[:800]}


# ---------------------------------------------------------------- PARTITIONS
def partitions() -> dict:
    """Disks + partitions halisi (size, fstype, mountpoint)."""
    if FAMILY == "linux":
        code, out = _run("lsblk -o NAME,PATH,SIZE,TYPE,FSTYPE,MOUNTPOINT --json 2>/dev/null")
        if code == 0 and out:
            try:
                data = json.loads(out)
                devs = []
                for d in data.get("blockdevices", []):
                    devs.append({"name": d.get("name"), "size": d.get("size"),
                                 "type": d.get("type"), "fstype": d.get("fstype"),
                                 "mountpoint": d.get("mountpoint"),
                                 "children": [{"name": c.get("name"), "size": c.get("size"),
                                               "fstype": c.get("fstype"), "mountpoint": c.get("mountpoint")}
                                              for c in (d.get("children") or [])]})
                return {"ok": True, "devices": devs}
            except json.JSONDecodeError:
                pass
        _, out = _run("lsblk -o NAME,SIZE,TYPE,FSTYPE,MOUNTPOINT 2>/dev/null || df -h | head -12")
        return {"ok": code == 0, "text": out[:1500]}
    if FAMILY == "windows":
        _, out = _run("powershell -NoProfile -Command \"Get-Disk | Format-Table Number,FriendlyName,@{n='SizeGB';e={[math]::Round($_.Size/1GB,1)}}; Get-Partition | Format-Table DiskNumber,PartitionNumber,DriveLetter,@{n='SizeGB';e={[math]::Round($_.Size/1GB,1)}}\"")
        return {"ok": True, "text": out[:1500]}
    _, out = _run("diskutil list 2>/dev/null | head -40")
    return {"ok": True, "text": out[:1500]}


# ---------------------------------------------------------------- SERVICES
def services() -> dict:
    if FAMILY == "linux":
        _, out = _run("systemctl list-units --type=service --state=running --no-pager --no-legend 2>/dev/null | awk '{print $1}' | head -40")
        running = [s for s in out.splitlines() if s.strip()]
        _, mtech = _run("systemctl is-active mtech-agent.service mtech-ollama.service 2>/dev/null")
        return {"running": running, "mtech": mtech.replace("\n", " · ")}
    if FAMILY == "windows":
        _, out = _run("powershell -NoProfile -Command \"Get-Service | Where-Object Status -eq 'Running' | Select-Object -First 30 -ExpandProperty Name\"")
        return {"running": out.splitlines()[:30], "mtech": "Sysmon64: " + ("RUNNING" if "RUNNING" in _run("sc query Sysmon64")[1].upper() else "?")}
    _, out = _run("launchctl list 2>/dev/null | head -30")
    return {"running": out.splitlines()[:30], "mtech": ""}


def kernel() -> dict:
    code, rel = _run("uname -r") if FAMILY == "linux" else (0, platform.release())
    return {"release": rel, "arch": platform.machine(), "python": platform.python_version(),
            "kali": os_release().get("PRETTY_NAME", ""), "is_kali": is_kali(),
            "mtech_module": Path("/dev/mtech").exists() if FAMILY == "linux" else None}


# ---------------------------------------------------------------- RAMANI KAMILI
def integrate() -> dict:
    """Kila kitu cha Kali/OS kwa pamoja (API /api/kali + tab ya KALI)."""
    t = tools()
    terms = terminals()
    return {
        "family": FAMILY,
        "kernel": kernel(),
        "packages": kali_packages(),
        "tools": t,
        "tools_count": len(t),
        "tools_categories": sorted({x["category"] for x in t}),
        "terminals": terms,
        "terminals_count": len(terms),
        "drivers": drivers(),
        "partitions": partitions(),
        "services": services(),
    }


# ---------------------------------------------------------------- CLI
def main() -> None:
    import sys
    what = sys.argv[1] if len(sys.argv) > 1 else ""
    if what == "tools":
        for t in tools():
            mark = "✔" if t["installed"] else "✗"
            print(f"  {mark} [{t['category']:<28}] {t['name']:<28} {t['cmd']}")
        print(f"— zana {len(tools())}")
    elif what == "open" and len(sys.argv) > 2:
        import json as _j
        print(_j.dumps(launch_tool(" ".join(sys.argv[2:])), ensure_ascii=False, indent=1))
    elif what == "term":
        import json as _j
        print(_j.dumps(open_terminal(" ".join(sys.argv[2:])), ensure_ascii=False, indent=1))
    else:
        print(json.dumps(integrate(), ensure_ascii=False, indent=1, default=str)[:8000])


if __name__ == "__main__":
    main()
