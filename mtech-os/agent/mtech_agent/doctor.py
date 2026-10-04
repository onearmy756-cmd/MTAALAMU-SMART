"""MTECH OS — doctor: ukaguzi halisi wa mashine (amri moja).

    mtech doctor

Inakimbia kwenye computer HALISI unayoweka MTECH na inakuambia kila kitu:
kile kilichofanya kazi, kile kisichofanya kazi, na JINSI ya kurekebisha.
"""
import shutil
import subprocess
import sys
from importlib import import_module

from .config import CONFIG
from .platform import FAMILY, events_source_hint


def _ok(name: str, detail: str = "") -> tuple:
    return (True, name, detail)


def _fail(name: str, fix: str) -> tuple:
    return (False, name, fix)


def check_python() -> tuple:
    v = sys.version_info
    if v >= (3, 9):
        return _ok(f"Python {v.major}.{v.minor}.{v.micro}")
    return _fail(f"Python {v.major}.{v.minor}", "saki Python 3.9+ (python.org au distro)")


def check_pillow() -> tuple:
    try:
        import PIL  # noqa: F401
        from PIL import ImageGrab  # noqa: F401
        return _ok("Pillow (vision)")
    except ImportError:
        return _fail("Pillow", "pip install Pillow")


def check_screen() -> tuple:
    """Full screen halisi — display ipo?"""
    try:
        from .platform import screenshot_path
        p = screenshot_path()
        size = p.stat().st_size
        if size > 1000:
            return _ok(f"Full screen vision ({size//1024} KB)")
        return _fail("Screen", "screenshot tupu — angalia display/session")
    except Exception as e:  # noqa: BLE001
        return _fail("Screen", f"hakuna display inayopatikana ({type(e).__name__}) — endesha kutoka ndani ya desktop session, si SSH")


def check_control() -> tuple:
    """Full control — ALLOW ipo? zana ipo?"""
    if not CONFIG.allow_control:
        return _fail("Full control", "IMEZIMWA — fungua: mtech allow on (au install.py --allow-control)")
    if FAMILY == "linux" and shutil.which("xdotool"):
        return _ok("Full control (xdotool)")
    if FAMILY in ("windows", "macos"):
        try:
            import_module("pyautogui")
            return _ok("Full control (pyautogui)")
        except ImportError:
            return _fail("Full control", "pip install pyautogui")
    return _fail("Full control", f"Linux: sudo apt install xdotool")


def check_ollama() -> tuple:
    try:
        from .llm import installed_models
        models = installed_models()
        return _ok(f"Ollama ({len(models)} models)", ", ".join(models[:3]))
    except Exception as e:  # noqa: BLE001
        return _fail("Ollama", f"haifanyi kazi ({CONFIG.ollama_url}) — saki: https://ollama.com au endesha 'ollama serve'")


def check_model() -> tuple:
    try:
        from .llm import model_ready
        if model_ready():
            return _ok(f"Model: {CONFIG.model}")
        return _fail(f"Model {CONFIG.model}", f"ollama pull {CONFIG.model}  (au install.py)")
    except Exception:
        return _fail(f"Model {CONFIG.model}", "ollama haipo — angalia hatua ya Ollama")


def check_kernel_feed() -> tuple:
    from .kernel_events import start_feed
    import time
    feed = start_feed()
    time.sleep(1.2)
    n = len(feed.snapshot(500))
    return _ok(f"Kernel feed: {feed.source} ({n} events)")


def check_skills() -> tuple:
    from .skills_bridge import skills_summary
    s = skills_summary()
    if s["total"] >= 100:
        return _ok(f"Skills: {s['total']} / trades {len(s['trades'])}")
    return _fail("Skills", f"zimepungua ({s['total']}) — angalia {s['source']}")


def check_boot() -> tuple:
    """Autostart integration ya OS husika."""
    from pathlib import Path
    if FAMILY == "linux":
        for p in (Path.home() / ".config" / "autostart" / "mtech-mtaalamu.desktop",
                  Path("/etc/xdg/autostart/mtech-shell.desktop"),
                  Path("/etc/xdg/autostart/mtech-mtaalamu.desktop")):
            if p.exists():
                return _ok("Boot: inaload na OS (autostart)", str(p))
        return _fail("Boot", "autostart haipo — endesha: python3 install.py --boot")
    if FAMILY == "windows":
        from pathlib import Path as P
        startup = P.home() / "AppData" / "Roaming" / "Microsoft" / "Windows" / "Start Menu" / "Programs" / "Startup"
        if any(startup.glob("MTAALAMU*.lnk")):
            return _ok("Boot: Startup folder ✓ (Sysmon: ps1 kama Admin)")
        return _fail("Boot", "Startup shortcut haipo — powershell -File boot/mtech-boot.ps1 (Admin)")
    plist = Path.home() / "Library" / "LaunchAgents" / "com.mtech.mtaalamu.plist"
    if plist.exists():
        return _ok("Boot: LaunchAgent ✓")
    return _fail("Boot", "plist haipo — python3 install.py --boot")


CHECKS = (check_python, check_pillow, check_screen, check_control,
          check_ollama, check_model, check_kernel_feed, check_skills, check_boot)


def run_all() -> int:
    print(f"═══ MTECH doctor — {FAMILY} ═══")
    failed = 0
    results = []
    for fn in CHECKS:
        try:
            ok, name, detail = fn()
        except Exception as e:  # noqa: BLE001 — doctor isife kwa kosa la check moja
            ok, name, detail = False, fn.__name__, f"{type(e).__name__}: {e}"
        results.append((ok, name, detail))
        if not ok:
            failed += 1
    for ok, name, detail in results:
        mark = "✓" if ok else "✗"
        print(f"  [{mark}] {name}" + (f" — {detail}" if detail else ""))
    print()
    if failed == 0:
        print("✅ KILA KITU KINAFA YA KAZI — MTECH iko tayari kwenye computer hii!")
        return 0
    print(f"⚠️  {failed} hazifanyi kazi — REKEBISHO (fanya kama zimeonyeshwa hapo juu):")
    print("   (baada ya kurekebisha, endesha tena: mtech doctor)")
    return 1
