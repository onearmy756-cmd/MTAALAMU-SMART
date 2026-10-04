#!/usr/bin/env python3
"""MTECH OS — installer moja kwa OS ZOTE (Linux, Windows, macOS).

Hakuna ISO, hakuna Docker, hakuna dual-boot: MTECH inafanya kazi NDANI YA
OS iliyopo kwa uhalisia kamili.

    python3 install.py                 # weka kila kitu (Ollama + model + apps)
    python3 install.py --no-model      # usivute qwen2.5vl:3b sasa
    python3 install.py --allow-control # fungua full control (mouse/keyboard) mara moja
    python3 install.py --kernel        # (Linux tu) build module ya /dev/mtech

Inafanya:
  1) Python venv + deps (Pillow, PySide6, pyautogui Win/macOS)
  2) Ollama halisi (installer rasmi ya OS husika) + pull qwen2.5vl:3b
  3) Apps: MTAALAMU SMART + Kali Tools (desktop launcher kwa kila OS)
  4) (Linux, hiari) kernel module /dev/mtech — ukunganisho na kernel
  5) config ya mtumiaji: ~/.mtech/config.json (allow_control)
"""
import argparse
import os
import platform
import shutil
import subprocess
import sys
import urllib.request
from pathlib import Path

HERE = Path(__file__).resolve().parent
FAMILY = "windows" if sys.platform.startswith("win") else ("macos" if sys.platform == "darwin" else "linux")
DEST = {"windows": Path(os.environ.get("LOCALAPPDATA", Path.home())) / "MTECH",
        "macos": Path.home() / "Library" / "MTECH",
        "linux": Path("/opt/mtech") if os.geteuid() == 0 else Path.home() / ".local" / "share" / "mtech"}[FAMILY]


def run(cmd, **kw):
    print("  $", " ".join(str(c) for c in cmd))
    return subprocess.run([str(c) for c in cmd], **kw)


def step(n, msg):
    print(f"\n[{n}] {msg}")


def have(bin_name):
    return shutil.which(bin_name) is not None


# ----------------------------------------------------------------- 1) venv
def setup_venv(with_gui: bool) -> Path:
    step(1, "Python venv + dependencies…")
    venv = DEST / "venv"
    if not venv.exists():
        run([sys.executable, "-m", "venv", str(venv)], check=True)
    pip = venv / ("Scripts" / "python.exe" if FAMILY == "windows" else "bin/python")
    run([pip, "-q", "install", "-r", str(HERE / "agent" / "requirements.txt")], check=True)
    if with_gui:
        run([pip, "-q", "install", "-r", str(HERE / "gui" / "requirements-gui.txt")])
        # Full control native kwa Win/macOS (Linux ina xdotool)
        if FAMILY in ("windows", "macos"):
            run([pip, "-q", "install", "pyautogui"])
    # nakili code
    shutil.copytree(HERE / "agent" / "mtech_agent", DEST / "agent" / "mtech_agent", dirs_exist_ok=True)
    shutil.copytree(HERE / "gui", DEST / "gui", dirs_exist_ok=True)
    shutil.copytree(HERE / "desktop", DEST / "desktop", dirs_exist_ok=True)
    shutil.copytree(HERE / "services", DEST / "services", dirs_exist_ok=True)
    return pip


# ----------------------------------------------------------------- 2) ollama
def install_ollama() -> bool:
    step(2, "Ollama (LLM engine halisi)…")
    if FAMILY == "linux":
        if not have("ollama"):
            run(["bash", "-c", "curl -fsSL https://ollama.com/install.sh | sh"])
        return have("ollama")
    if FAMILY == "macos":
        if not have("ollama"):
            if have("brew"):
                run(["brew", "install", "ollama"])
            else:
                run(["bash", "-c",
                     "curl -fsSL https://ollama.com/download/Ollama-darwin.zip -o /tmp/ollama.zip && "
                     "unzip -o /tmp/ollama.zip -d /Applications && "
                     "open /Applications/Ollama.app"])
                print("  (fungua Ollama.app mara moja ili daemon ianze)")
        return True
    # windows
    if not have("ollama"):
        exe = DEST / "OllamaSetup.exe"
        urllib.request.urlretrieve("https://ollama.com/download/OllamaSetup.exe", exe)
        run([exe, "/silent"], check=False)
        print("  (OllamaSetup imezinduliwa — subiri isakinishike, kisha endesha tena install.py)")
        return False
    return True


def pull_model() -> None:
    step(3, "Ollama pull qwen2.5vl:3b (GB ~2.3, mtandao)…")
    ollama = shutil.which("ollama") or (DEST / "ollama")
    if FAMILY == "windows" and not have("ollama"):
        ollama = Path(os.environ.get("LOCALAPPDATA", "")) / "Programs" / "Ollama" / "ollama.exe"
    run([ollama, "pull", "qwen2.5vl:3b"])


# ----------------------------------------------------------------- 4) launchers
# ----------------------------------------------------------------- 4b) BOOT integration
def setup_boot(pip: Path) -> None:
    """Inapowaka OS: MTAALAMU inaload KIOTOMATIKI, logo yake inaonekana,
    na (Windows) Sysmon inasakinishwa kiotomatiki — kernel feed HALISI."""
    step("4b", "BOOT integration — inaload na OS (autostart + logo + kernel feed)…")
    boot = HERE / "boot"
    if FAMILY == "linux":
        # autostart ya kila login (XDG — inafanya kazi GNOME/KDE/XFCE zote)
        auto = Path.home() / ".config" / "autostart"
        auto.mkdir(parents=True, exist_ok=True)
        src = boot / "mtech-mtaalamu-autostart.desktop"
        dst = auto / "mtech-mtaalamu.desktop"
        dst.write_text(src.read_text().replace("/usr/local/bin/mtaalamu-app", str(DEST / "desktop" / "mtaalamu-app") if (DEST / "desktop" / "mtaalamu-app").exists() else f"{pip} {DEST / 'gui' / 'mtech_shell.py'}"))
        # icon kwenye hicolor (logo inaonekana kila mahali)
        icons = Path.home() / ".local" / "share" / "icons" / "hicolor" / "scalable" / "apps"
        icons.mkdir(parents=True, exist_ok=True)
        shutil.copy(boot.parent / "desktop" / "mtaalamu.svg", icons / "mtaalamu.svg")
        # Session ya MTECH kwenye login manager (kama distro inaruhusu per-user)
        xs = Path.home() / ".local" / "share" / "xsessions"
        xs.mkdir(parents=True, exist_ok=True)
        shutil.copy(boot / "mtech-session.desktop", xs / "mtech.desktop")
        print(f"  ✓ autostart → {dst}")
        print(f"  ✓ icon → {icons / 'mtaalamu.svg'}")
        print(f"  ✓ session 'MTECH OS' → {xs / 'mtech.desktop'} (chagua kwenye login)")
    elif FAMILY == "windows":
        print("  → Windows: endesha (Admin) → powershell -File boot/mtech-boot.ps1")
        print("     (inasakinisha Sysmon KIOTOMATIKI + Startup shortcuts + branding)")
    else:
        plist_src = boot / "com.mtech.mtaalamu.plist"
        lib = Path.home() / "Library" / "LaunchAgents"
        lib.mkdir(parents=True, exist_ok=True)
        plist = lib / "com.mtech.mtaalamu.plist"
        plist.write_text(plist_src.read_text()
                         .replace("__MTAALAMU_PYTHON__", str(pip))
                         .replace("__MTAALAMU_GUI__", str(DEST / "gui" / "mtech_shell.py")))
        run(["launchctl", "load", str(plist)], check=False)
        print(f"  ✓ LaunchAgent → {plist} (inaload na login)")


def make_launchers(pip: Path) -> None:
    step(4, "Desktop launchers (MTAALAMU SMART + Kali Tools)…")
    if FAMILY == "linux":
        apps = Path.home() / ".local" / "share" / "applications"
        apps.mkdir(parents=True, exist_ok=True)
        for name, script in (("mtaalamu", "gui/mtech_shell.py"), ("kali-tools", "desktop/kali-tools-window.py")):
            desktop = apps / f"mtech-{name}.desktop"
            desktop.write_text(f"""[Desktop Entry]
Type=Application
Name={name}
Exec={pip} {DEST / script}
Icon={DEST / 'desktop' / 'mtaalamu.svg' if name == 'mtaalamu' else 'utilities-terminal'}
Terminal=false
Categories=System;Utility;
""")
        print(f"  ✓ launchers → {apps}")
    elif FAMILY == "windows":
        ps = f"""$ws=New-Object -ComObject WScript.Shell;
$s=$ws.CreateShortcut('{Path.home()/ "Desktop" / "MTAALAMU SMART.lnk"}');
$s.TargetPath='{pip}'; $s.Arguments='{DEST / "gui" / "mtech_shell.py"}'; $s.Save();
$k=$ws.CreateShortcut('{Path.home()/ "Desktop" / "Kali Tools.lnk"}');
$k.TargetPath='{pip}'; $k.Arguments='{DEST / "desktop" / "kali-tools-window.py"}'; $k.Save()"""
        run(["powershell", "-NoProfile", "-Command", ps])
        print("  ✓ shortcuts → Desktop")
    else:
        for name, script in (("MTAALAMU SMART", "gui/mtech_shell.py"), ("Kali Tools", "desktop/kali-tools-window.py")):
            cmd = Path.home() / "Desktop" / f"{name}.command"
            cmd.write_text(f'#!/bin/sh\nexec "{pip}" "{DEST / script}"\n')
            cmd.chmod(0o755)
        print("  ✓ .command launchers → Desktop")


# ----------------------------------------------------------------- 5) kernel (linux)
def build_kernel_module() -> None:
    step(5, "Kernel bridge /dev/mtech (Linux tu)…")
    if FAMILY != "linux":
        print("  (Windows/macOS: kernel feed inatumia Sysmon / Unified Log — hakuna module inahitajika)")
        return
    if not (Path("/lib/modules") .exists() and (Path("/lib/modules") / platform.release() / "build").exists()):
        print("  ✖ kernel headers hazipo — saki: sudo apt install -y linux-headers-$(uname -r)")
        return
    run(["make", "-C", str(HERE / "kernel")], check=False)
    run(["sudo", "insmod", str(HERE / "kernel" / "mtech_dev.ko")], check=False)
    print("  ✓ /dev/mtech — angalia: cat /proc/mtech_status")


# ----------------------------------------------------------------- main
def main() -> None:
    ap = argparse.ArgumentParser(description="MTECH installer — OS yoyote, hakuna ISO/Docker/dual-boot")
    ap.add_argument("--no-model", action="store_true", help="usivute qwen2.5vl:3b sasa")
    ap.add_argument("--no-gui", action="store_true", help="bila apps za GUI")
    ap.add_argument("--allow-control", action="store_true", help="fungua full control (mouse/keyboard) — ALLOW ya mtumiaji")
    ap.add_argument("--kernel", action="store_true", help="(Linux) build + load module ya /dev/mtech")
    ap.add_argument("--boot", action="store_true", help="inaload kiotomatiki inapowaka OS (autostart + logo + Sysmon kwa Windows)")
    args = ap.parse_args()

    print("═══════════════════════════════════════════════")
    print(f" MTECH — installer ({FAMILY}, {platform.release()})")
    print("═══════════════════════════════════════════════")

    DEST.mkdir(parents=True, exist_ok=True)
    pip = setup_venv(with_gui=not args.no_gui)
    ok_ollama = install_ollama()
    if ok_ollama and not args.no_model:
        pull_model()

    # config ya mtumiaji
    cfg = Path.home() / ".mtech" / "config.json"
    cfg.parent.mkdir(parents=True, exist_ok=True)
    import json
    data = json.loads(cfg.read_text()) if cfg.exists() else {}
    if args.allow_control:
        data["allow_control"] = True
    cfg.write_text(json.dumps(data, indent=1))

    make_launchers(pip)
    if args.boot:
        setup_boot(pip)
    if args.kernel:
        build_kernel_module()

    print("\n✅ IMEKAMILIKA — MTECH inafanya kazi ndani ya OS yako:")
    print(f"   App:        MTAALAMU SMART (desktop launcher)")
    print(f"   CLI:        {pip} -m mtech_agent skills   (au: mtech ask \"...\")")
    print(f"   Control:    {'FUNGULIWA ✅' if args.allow_control else 'imezimwa — fungua: python3 install.py --allow-control (au: mtech allow on)'}")
    print(f"   Full screen: vision inaona screen yote (Pillow/native) ✓")
    if FAMILY != "linux":
        print(f"   Kernel feed: {('Windows → Sysmon (endesha --boot kama Admin; inasakinishwa kiotomatiki)' if FAMILY == 'windows' else 'macOS → Unified Log')} — halisi")
    if not args.boot:
        print("   Boot:       python3 install.py --boot  → inaload KIOTOMATIKI inapowaka OS (+ logo)")


if __name__ == "__main__":
    main()
