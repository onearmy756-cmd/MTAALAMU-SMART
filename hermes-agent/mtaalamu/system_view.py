"""MTAALAMU SMART — SYSTEM VIEW (mfumo mzima, kuanzia ukiwaka).

Hii ndiyo inayoonyesha FULL SYSTEM kwenye tabs zote:
  • boot_timeline()   — mfululizo wa UKIWAKA: kernel → session/autostart →
                        Sysmon/LaunchAgent → Ollama+model → agent/API → GUI,
                        kila hatua ina hali HALISI (checks za kweli za OS)
  • mtech_components()— MTECH OS na kila kilichomo (module, GUI, kali ISO,
                        boot files, agent, doctor...)
  • system_map()      — ramani ya OS: CPU/mem/disk, top processes, network
  • book_recent()     — Kitabu Kidigitali (kesi + maarifa)
  • payments_recent() — malipo yote (audit)

Yote ni checks HALISI za OS husika (Linux/Windows/macOS) — hakuna kubuni.
"""
import json
import os
import platform
import shutil
import subprocess
import sys as _sys
from pathlib import Path

from . import admin, billing, clickpesa

FAMILY = "windows" if os.name == "nt" else ("macos" if _sys.platform == "darwin" else "linux")
HOME = Path.home()


def _run(cmd: str, timeout: int = 8) -> tuple[int, str]:
    """Amri halisi (stdout mafupi). code -1 = haikupatikana/ikafeli."""
    try:
        p = subprocess.run(cmd, shell=True, capture_output=True, text=True, timeout=timeout)
        return p.returncode, (p.stdout or p.stderr or "").strip()[:400]
    except (OSError, subprocess.TimeoutExpired) as e:
        return -1, str(e)[:200]


def _ollama() -> dict:
    """Ollama + modeli (HALISI: /api/tags ya ollama)."""
    import urllib.request
    try:
        with urllib.request.urlopen("http://127.0.0.1:11434/api/tags", timeout=3) as r:
            data = json.loads(r.read().decode())
        models = [m.get("name", "") for m in data.get("models", [])]
        qwen = any("qwen2.5vl" in m for m in models)
        return {"ok": True, "models": models[:12], "qwen2_5vl": qwen}
    except Exception as e:  # noqa: BLE001
        return {"ok": False, "models": [], "qwen2_5vl": False, "error": str(e)[:120]}


def boot_timeline() -> list:
    """Mfululizo wa UKIWAKA — kila hatua na hali yake HALISI sasa hivi."""
    stages = []

    # [1] KERNEL + MTECH MODULE
    if FAMILY == "linux":
        code, uname = _run("uname -r")
        module = Path("/dev/mtech").exists()
        stages.append({"n": 1, "stage": "KERNEL", "sw": "Kernel + moduli ya MTECH (/dev/mtech)",
                       "ok": code == 0 and module,
                       "detail": f"kernel {uname}" + ("" if module else " · /dev/mtech haipo (kmod haikupakiwa — sudo ./mtech-os/kernel/build-kernel.sh --module-only)")})
    elif FAMILY == "windows":
        code, out = _run("sc query Sysmon64")
        stages.append({"n": 1, "stage": "KERNEL FEED", "sw": "Sysmon service (kernel events)",
                       "ok": code == 0 and "RUNNING" in out.upper(),
                       "detail": "RUNNING" if code == 0 else "Sysmon haijaanza — Admin: boot/mtech-boot.ps1"})
    else:
        code, out = _run("launchctl list | grep -i mtech")
        stages.append({"n": 1, "stage": "LAUNCHAGENT", "sw": "LaunchAgent ya MTAALAMU",
                       "ok": code == 0, "detail": out[:120] or "com.mtech.mtaalamu haipo — install.py --boot"})

    # [2] SESSION / AUTOSTART (inapowaka OS)
    if FAMILY == "linux":
        session = Path("/usr/share/xsessions/mtech.desktop").exists()
        auto_sys = Path("/etc/xdg/autostart/mtech-mtaalamu.desktop").exists()
        auto_usr = (HOME / ".config/autostart/mtech-mtaalamu.desktop").exists()
        stages.append({"n": 2, "stage": "SESSION", "sw": "Session 'MTECH OS' + autostart",
                       "ok": session or auto_sys or auto_usr,
                       "detail": f"xsessions: {'✔' if session else '✗'} · autostart-sys: {'✔' if auto_sys else '✗'} · autostart-user: {'✔' if auto_usr else '✗'}"})
    elif FAMILY == "windows":
        startup = Path(os.environ.get("APPDATA", "")) / "Microsoft/Windows/Start Menu/Programs/Startup"
        lnks = list(startup.glob("*.lnk")) if startup.exists() else []
        stages.append({"n": 2, "stage": "STARTUP", "sw": "Startup folder (inapowaka Windows)",
                       "ok": len(lnks) > 0, "detail": f"shortcuts: {len(lnks)}"})
    else:
        plist = HOME / "Library/LaunchAgents/com.mtech.mtaalamu.plist"
        stages.append({"n": 2, "stage": "LAUNCHAGENT", "sw": "LaunchAgent (inapowaka macOS)",
                       "ok": plist.exists(), "detail": str(plist)})

    # [3] AKILI: Ollama + modeli
    ol = _ollama()
    stages.append({"n": 3, "stage": "AKILI", "sw": "Ollama + Qwen 2.5 VL 3B + TinyLlama",
                   "ok": ol["ok"],
                   "detail": (f"models: {', '.join(ol['models'][:4]) or '—'}"
                              + (" · qwen2.5vl ✔" if ol["qwen2_5vl"] else " · qwen2.5vl ✗ (ollama pull qwen2.5vl:3b)"))
                              if ol["ok"] else f"Ollama haifiki: {ol.get('error','')}"})

    # [4] AGENT/API + IPC
    api_port = os.environ.get("MTECH_API_PORT", "8795")
    code, out = _run(f"curl -s -o /dev/null -w '%{{http_code}}' http://127.0.0.1:{api_port}/api/status --max-time 2"
                     if FAMILY != "windows" else
                     f"powershell -NoProfile -Command \"(Invoke-WebRequest -UseBasicParsing http://127.0.0.1:{api_port}/api/status -TimeoutSec 2).StatusCode\"")
    api_ok = (out.strip().strip("'") == "200")
    sock = Path("/tmp/mtaalamu.sock").exists() if FAMILY != "windows" else True
    stages.append({"n": 4, "stage": "AGENT", "sw": "MTAALAMU agent (API + IPC)",
                   "ok": api_ok,
                   "detail": f"API :{api_port} {'✔ 200' if api_ok else '✗ (mtaalamu serve)'} · IPC {'✔' if sock else '—'}"})

    # [5] GUI (MTECH Shell / PySide6)
    try:
        import PySide6  # noqa: F401
        gui = {"ok": True, "ver": PySide6.__version__ if hasattr(PySide6, "__version__") else "✓"}
    except Exception:
        gui = {"ok": False, "ver": None}
    stages.append({"n": 5, "stage": "GUI", "sw": "MTECH Shell (PySide6 GUI)",
                   "ok": gui["ok"],
                   "detail": f"PySide6 {gui['ver']}" if gui["ok"] else "PySide6 haipo — pip install PySide6 (au tumia UI ya browser)"})

    # [6] HIFADHI: leseni + wallet + kitabu
    book = Path(os.environ.get("MTECH_BOOK", HOME / ".mtaalamu" / "digital_book.jsonl"))
    know = Path(os.environ.get("MTECH_KNOW", HOME / ".mtaalamu" / "knowledge.jsonl"))
    stages.append({"n": 6, "stage": "HIFADHI", "sw": "Leseni + Wallet + Kitabu Kidigitali",
                   "ok": billing.is_active(),
                   "detail": f"leseni {'✔ ' + (billing.current_license() or {}).get('tier', '') if billing.is_active() else '✗ (mtaalamu register)'}"
                             f" · wallet {billing.fmt_tzs(billing.wallet_balance())}"
                             f" · kesi {sum(1 for _ in book.open()) if book.exists() else 0}"
                             f" · maarifa {sum(1 for _ in know.open()) if know.exists() else 0}"})
    return stages


# ------------------------------------------------------------------ MTECH OS components
def _repo_root() -> Path:
    return Path(__file__).resolve().parents[2]


def mtech_components() -> list:
    """MTECH OS na kila kilichomo (paths halisi + uwepo wao)."""
    root = _repo_root()
    items = [
        ("KERNEL", "mtech-os/kernel", "Kernel bridge (config + build-kernel.sh) — torvalds/linux"),
        ("MODULE", "mtech-os/kernel/mtech_dev.c", "Moduli ya kernel: /dev/mtech (kprobes)"),
        ("AGENT", "mtech-os/agent/mtech_agent", "Agent: platform.py (Linux/Win/macOS), tools, vision, HITL"),
        ("GUI", "mtech-os/gui/mtech_shell.py", "MTECH Shell (PySide6) + Kali Tools window"),
        ("KALI-LOOK", "mtech-os/kali", "Muonekano wa Kali (XFCE panel, conky, wallpaper, icons)"),
        ("KALI-ISO", "mtech-os/kali/build-iso.sh", "ISO halisi ya Kali + MTAALAMU (--check/--container)"),
        ("BOOT", "mtech-os/boot", "Boot: xsessions, autostart, Sysmon config, LaunchAgent, wallpaper"),
        ("INSTALL", "mtech-os/install.py", "Installer moja ya OS zote 3 (--boot --allow-control)"),
        ("DOCTOR", "mtech-os/agent/mtech_agent/doctor.py", "mtech doctor (ukaguzi wa computer halisi)"),
        ("MTAALAMU", "hermes-agent/mtaalamu", "Unified Solver: catalog 31 ops, HITL, ClickPesa, admin"),
        ("UI", "hermes-agent/web-html/mtaalamu-unified.html", "Dashboard ya tabs (offline, single-file)"),
        ("SERVER", "hermes-agent/mtaalamu/server.py", "HTTP API + IPC + R bridge"),
    ]
    out = []
    for name, rel, desc in items:
        p = root / rel
        out.append({"name": name, "path": rel, "desc": desc,
                    "ok": p.exists(), "detail": "ipo" if p.exists() else "haipo"})
    return out


# ------------------------------------------------------------------ system map
def system_map() -> dict:
    """Ramani HALISI ya OS: vifaa, processes, network, services."""
    if FAMILY == "linux":
        _, cpu = _run("nproc; grep -m1 'model name' /proc/cpuinfo | cut -d: -f2")
        _, mem = _run("free -m | awk 'NR==2{print $3\"/\"$2\" MB used\"}'")
        _, disk = _run("df -h / | tail -1")
        _, procs = _run("ps aux --sort=-%cpu | head -7")
        _, net = _run("ip -br addr 2>/dev/null | head -6; ip route | head -3")
        _, svc = _run("systemctl list-units --type=service --state=running --no-pager 2>/dev/null | head -8")
    elif FAMILY == "windows":
        _, cpu = _run("powershell -NoProfile -Command \"(Get-CimInstance Win32_Processor).Name; (Get-CimInstance Win32_ComputerSystem).NumberOfLogicalProcessors\"")
        _, mem = _run("powershell -NoProfile -Command \"(Get-CimInstance Win32_OperatingSystem | ForEach-Object { [math]::Round($_.TotalVisibleMemorySize/1KB) })\"")
        _, disk = _run("powershell -NoProfile -Command \"Get-PSDrive C | Select-Object Used,Free | Format-List\"")
        _, procs = _run("powershell -NoProfile -Command \"Get-Process | Sort-Object CPU -Descending | Select-Object -First 6 Name,CPU | Format-Table\"")
        _, net = _run("powershell -NoProfile -Command \"Get-NetIPAddress -AddressFamily IPv4 | Select-Object -First 4 InterfaceAlias,IPAddress | Format-Table\"")
        _, svc = _run("powershell -NoProfile -Command \"Get-Service | Where-Object Status -eq Running | Select-Object -First 8 Name,DisplayName | Format-Table\"")
    else:
        _, cpu = _run("sysctl -n hw.ncpu; sysctl -n machdep.cpu.brand_string")
        _, mem = _run("vm_stat | head -4")
        _, disk = _run("df -h / | tail -1")
        _, procs = _run("ps aux -r | head -7")
        _, net = _run("ifconfig en0 | grep 'inet '")
        _, svc = _run("launchctl list | head -8")
    return {"family": FAMILY, "release": platform.release(), "machine": platform.machine(),
            "node": platform.node(), "python": _sys.version.split()[0],
            "cpu": cpu, "memory": mem, "disk": disk, "top_processes": procs,
            "network": net, "services_running": svc}


# ------------------------------------------------------------------ kitabu + payments
def _jsonl(path: Path, n: int) -> list:
    if not path.exists():
        return []
    lines = path.read_text().splitlines()[-n:]
    out = []
    for l in reversed(lines):  # mpya kwanza
        try:
            out.append(json.loads(l))
        except json.JSONDecodeError:
            continue
    return out


def book_recent(n: int = 25) -> dict:
    home = Path(os.environ.get("MTECH_BOOK", HOME / ".mtaalamu" / "digital_book.jsonl")).parent
    return {"cases": _jsonl(home / "digital_book.jsonl", n),
            "knowledge": _jsonl(home / "knowledge.jsonl", n),
            "total_cases": sum(1 for _ in (home / "digital_book.jsonl").open()) if (home / "digital_book.jsonl").exists() else 0}


def payments_recent(n: int = 30) -> dict:
    f = billing.PAYMENTS_FILE
    rows = _jsonl(f, n) if f.exists() else []
    for r in rows:
        r["amount"] = billing.fmt_tzs(r.get("amount_tzs", 0))
    return {"payments": rows, "total_tzs": billing.payments_total(),
            "wallet_tzs": billing.wallet_balance(),
            "gateway": clickpesa.status_mode()}


# ------------------------------------------------------------------ FULL SYSTEM (admin)
def full_system() -> dict:
    """KILA KITU: boot + components + map + kitabu + malipo + leseni + admin."""
    return {
        "boot": boot_timeline(),
        "components": mtech_components(),
        "map": system_map(),
        "license": billing.usage_summary(),
        "payments": payments_recent(),
        "book": book_recent(),
        "gateway": clickpesa.status_mode(),
        "admin": admin.info(),
    }
