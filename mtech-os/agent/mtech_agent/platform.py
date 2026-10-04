"""MTECH OS — tabaka la mfumo kwa OS ZOTE (Linux, Windows, macOS).

Kanuni: kila kitu HALISI kwenye OS husika:
  • screenshot  → full screen ya kweli (Pillow ImageGrab Win/macOS; scrot Linux X11)
  • input       → pyautogui (Win/macOS native); xdotool (Linux X11)
  • probe       → /proc (Linux); PowerShell/CIM (Windows); sysctl/vm_stat (macOS)
  • events      → /dev/mtech (Linux kernel module); Sysmon Event Log (Windows);
                  Unified log (macOS); fallback: uchunguzi wa processes
  • shell       → bash/zsh (Linux/macOS); PowerShell (Windows)

Full control ya kompyuta inahitaji ALLOW ya mtumiaji (MTECH_ALLOW_CONTROL=1
au kibali kupitia HITL gate).
"""
import os
import platform as _platform
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

def os_family() -> str:
    s = sys.platform
    if s.startswith("win"):
        return "windows"
    if s == "darwin":
        return "macos"
    return "linux"

FAMILY = os_family()
IS_LINUX = FAMILY == "linux"
IS_WIN = FAMILY == "windows"
IS_MAC = FAMILY == "macos"

# ------------------------------------------------------------------ screenshot
def screenshot_path() -> Path:
    """Full screen halisi — OS yoyote. Inarudisha njia ya PNG."""
    out = Path(tempfile.gettempdir()) / "mtech_screen.png"

    # 1) Pillow ImageGrab — native kwa Windows na macOS (na X11 mpya)
    try:
        from PIL import ImageGrab  # type: ignore
        img = ImageGrab.grab(all_screens=True)
        img.save(out)
        if out.stat().st_size > 0:
            return out
    except Exception:
        pass

    # 2) Linux tools halisi
    if IS_LINUX:
        for cmd in (["scrot", "-o", str(out)], ["gnome-screenshot", "-f", str(out)]):
            if shutil.which(cmd[0]):
                try:
                    subprocess.run(cmd, capture_output=True, timeout=30)
                    if out.exists() and out.stat().st_size > 0:
                        return out
                except (OSError, subprocess.TimeoutExpired):
                    continue

    # 3) macOS native
    if IS_MAC and shutil.which("screencapture"):
        subprocess.run(["screencapture", "-x", str(out)], capture_output=True, timeout=30)
        if out.exists() and out.stat().st_size > 0:
            return out

    # 4) Windows PowerShell fallback
    if IS_WIN:
        ps = (
            "Add-Type -AssemblyName System.Windows.Forms;"
            "$b=[System.Windows.Forms.Screen]::PrimaryScreen.Bounds;"
            "$bmp=New-Object System.Drawing.Bitmap $b.Width,$b.Height;"
            "$g=[System.Drawing.Graphics]::FromImage($bmp);"
            "$g.CopyFromScreen($b.Location,[System.Drawing.Point]::Empty,$b.Size);"
            f"$bmp.Save('{out}');"
        )
        subprocess.run(["powershell", "-NoProfile", "-Command", ps], capture_output=True, timeout=30)
        if out.exists() and out.stat().st_size > 0:
            return out

    raise RuntimeError(f"Screenshot imeshindikana kwenye {FAMILY} — saki Pillow (pip install Pillow)")


# ------------------------------------------------------------------ input (mouse/keyboard)
def input_control(action: str, args: dict) -> dict:
    """Udhibiti halisi wa kipanya/kibodi — OS yoyote. Full control inahitaji allow."""
    if IS_LINUX and shutil.which("xdotool"):
        return _input_xdotool(action, args)

    # pyautogui — native Windows/macOS (na Linux X11)
    try:
        import pyautogui  # type: ignore
        pyautogui.FAILSAFE = True  # hamisha kipanya kona 0,0 kusimamisha
        if action == "move":
            pyautogui.moveTo(int(args.get("x", 0)), int(args.get("y", 0)), duration=0.2)
            return {"ok": True}
        if action == "click":
            pyautogui.click(button=str(args.get("button", "left") or "left"))
            return {"ok": True}
        if action == "type":
            pyautogui.typewrite(str(args.get("text", "")), interval=0.04)
            return {"ok": True}
        if action == "key":
            pyautogui.press(str(args.get("key", "enter")))
            return {"ok": True}
        return {"ok": False, "error": f"action haijulikani: {action}"}
    except ImportError:
        return {"ok": False, "error": "saki: pip install pyautogui (au xdotool kwa Linux)"}


def _input_xdotool(action: str, args: dict) -> dict:
    xdo = {
        "move": lambda: ["mousemove", str(args.get("x", 0)), str(args.get("y", 0))],
        "click": lambda: ["click", "1" if str(args.get("button", "left")) == "left" else "3"],
        "type": lambda: ["type", "--delay", "40", str(args.get("text", ""))],
        "key": lambda: ["key", str(args.get("key", "Return"))],
    }
    if action not in xdo:
        return {"ok": False, "error": f"action haijulikani: {action}"}
    p = subprocess.run(["xdotool", *xdo[action]()], capture_output=True, text=True, timeout=30)
    return {"ok": p.returncode == 0, "err": p.stderr[:300]}


# ------------------------------------------------------------------ probe
def probe_extra(system: dict) -> dict:
    """Taarifa za ziada za OS husika (zote HALISI)."""
    try:
        if IS_WIN:
            out = subprocess.run(
                ["powershell", "-NoProfile", "-Command",
                 "Get-CimInstance Win32_OperatingSystem | Select-Object Caption,Version | ConvertTo-Json"],
                capture_output=True, text=True, timeout=20)
            system["os_info"] = out.stdout.strip()[:500] or "Windows"
        elif IS_MAC:
            out = subprocess.run(["sw_vers"], capture_output=True, text=True, timeout=10)
            system["os_info"] = out.stdout.strip()
        else:
            try:
                system["os_info"] = Path("/etc/os-release").read_text().splitlines()[0]
            except OSError:
                pass
    except (OSError, subprocess.TimeoutExpired):
        pass
    system["platform"] = FAMILY
    return system


# ------------------------------------------------------------------ events source
def events_source_hint() -> str:
    """Chanzo halisi cha matukio ya chini ya mfumo, kwa kila OS."""
    if IS_LINUX:
        return "/dev/mtech (kernel module) au /proc"
    if IS_WIN:
        return "Sysmon Event Log (Microsoft-Windows-Sysmon/Operational) — saki Sysmon bila malipo"
    return "macOS Unified Log (log stream)"


def tail_events(callback, stop_flag) -> str:
    """Anzisha mtiririko halisi wa matukio (Windows: Sysmon; macOS: log stream).
    Linux inatumia /dev/mtech au /proc (angalia kernel_events.py)."""
    try:
        if IS_WIN:
            cmd = ["powershell", "-NoProfile", "-Command",
                   "Get-WinEvent -LogName 'Microsoft-Windows-Sysmon/Operational' -MaxEvents 20"]
            p = subprocess.Popen(cmd, stdout=subprocess.PIPE, text=True, errors="replace")
            for line in p.stdout or []:
                if stop_flag.is_set():
                    break
                line = line.strip()
                if line:
                    callback({"type": "event", "detail": line[:120]})
            return "sysmon"
        if IS_MAC:
            p = subprocess.Popen(
                ["log", "stream", "--style", "compact", "--predicate",
                 'processImagePath CONTAINS "/"'],
                stdout=subprocess.PIPE, text=True, errors="replace")
            for line in p.stdout or []:
                if stop_flag.is_set():
                    break
                callback({"type": "log", "detail": line.strip()[:120]})
            return "unified-log"
    except (OSError, subprocess.TimeoutExpired):
        pass
    return "proc-fallback"


# ------------------------------------------------------------------ shell
SHELL_NAME = "powershell" if IS_WIN else ("zsh" if IS_MAC and shutil.which("zsh") else "bash")
