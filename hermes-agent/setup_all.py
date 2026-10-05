#!/usr/bin/env python3
"""MTAALAMU SMART + MTECH OS — ONE-CLICK INSTALL (kila kitu kwa amri moja).

    python3 setup_all.py                 # BINARY-FIRST: binary (Python NDANI yake),
                                         #   account ya kiotomatiki, ADMIN, server + dashboard
                                         #   — HAKUNA Python inayosakinishwa (venv ni hiari)
    python3 setup_all.py --check         # angalia tu — usisakinise chochote
    python3 setup_all.py --with-agent    # PIA: venv ya Python (Pillow/pyautogui/PySide6)
                                         #   — kwa MTECH agent (vision/control) + GUI ya desktop
    python3 setup_all.py --email me@mail.com   # tumia barua pepe yako (hiari)
    python3 setup_all.py --skip-rust     # ruka Rust
    python3 setup_all.py --skip-r        # ruka R
    python3 setup_all.py --skip-office   # ruka LibreOffice
    python3 setup_all.py --no-model      # usivute qwen2.5vl:3b (baadaye: ollama pull)
    python3 setup_all.py --no-start      # usianze server/dashboard kiotomatiki

INAYOSAKINISHWA (kwa mpangilio, OS zote 3):
  1. Zana za C:        gcc/clang + make (+ build-essential / xcode CLT / VS Build Tools hint)
  2. Rust:             rustup.rs (rasmi) → cargo + rustc  [--skip-rust kwa kuruka]
  3. R:                R + Rscript (apt/brew/winget)      [--skip-r kwa kuruka]
  4. LibreOffice:      ofisi kamili (apt/brew/winget)     [--skip-office kwa kuruka]
  5. BINARY (onefile): dist/mtaalamu-linux | MTAALAMU-Setup.exe — Python NDANI yake
                       (PyInstaller); kama haipo, inajengwa yenyewe — account/admin/serve
                       ZOTE zinatumia binary hii, si python -m
  6. Ollama + Qwen:    ollama (rasmi) + ollama pull qwen2.5vl:3b  [--no-model kwa kuruka]
  7. KIOTOMATIKI:      account (auto-register DIAMOND) + ADMIN (zana BURE)
                       + server inaanza + dashboard inafunguka browser — HAKUNA configuration

PYTHON? CHAGUOMSINGI: HAKUNA kusakinisha Python.
  • mtaalamu (CLI/API/brain) = binary yenye Python ndani (PyInstaller onefile)
  • venv ya Python (Pillow/pyautogui/PySide6) ni kwa MTECH agent (macho/mikono)
    na GUI ya desktop TU — weka --with-agent ikiwa unataka hiyo
"""
import os
import platform
import shutil
import subprocess
import sys

FAMILY = "windows" if os.name == "nt" else ("macos" if sys.platform == "darwin" else "linux")
PKG = {"linux": {"apt": "sudo apt-get update && sudo apt-get install -y", "dnf": "sudo dnf install -y", "pacman": "sudo pacman -S --noconfirm"}}
HERE = os.path.dirname(os.path.abspath(__file__))


def run(cmd: str, timeout: int = 900) -> tuple[int, str]:
    print(f"  ▸ $ {cmd[:120]}")
    try:
        p = subprocess.run(cmd, shell=True, capture_output=True, text=True, timeout=timeout)
        out = (p.stdout or "") + (p.stderr or "")
        return p.returncode, out.strip()[-500:]
    except (OSError, subprocess.TimeoutExpired) as e:
        return -1, str(e)[:200]


def have(tool: str) -> bool:
    return shutil.which(tool) is not None


def _linux_pkg() -> str:
    """Tambua package manager ya distro (apt/dnf/pacman)."""
    for pm in ("apt-get", "dnf", "pacman"):
        if shutil.which(pm):
            return pm
    return "apt-get"


# ------------------------------------------------------------------ steps
def step_c(check: bool) -> bool:
    ok = have("gcc") or have("clang")
    mk = have("make")
    print(f"[1/9] Zana za C: gcc/clang {'✔' if ok else '✗'} · make {'✔' if mk else '✗'}")
    if (ok and mk) or check:
        return ok and mk
    if FAMILY == "linux":
        pm = _linux_pkg()
        if pm == "apt-get":
            run(f"{PKG['linux']['apt']} build-essential")
        elif pm == "dnf":
            run("sudo dnf groupinstall -y \"Development Tools\"")
        else:
            run("sudo pacman -S --noconfirm base-devel")
    elif FAMILY == "macos":
        run("xcode-select --install", timeout=60)  # inauliza GUI — mkondo wake
        print("  (kama dialog ya CLT ilifunguka — kamilisha kwenye dirisha lililofunguka)")
    else:
        print("  ⚠ Windows: sakinisha 'Visual Studio Build Tools' (C) — winget install Microsoft.VisualStudio.2022.BuildTools")
    return (have("gcc") or have("clang")) and have("make")


def step_rust(check: bool, skip: bool) -> bool:
    if skip:
        print("[2/9] Rust: IMERUKIWA (--skip-rust)")
        return have("cargo")
    ok = have("cargo") and have("rustc")
    print(f"[2/9] Rust: {'✔ ' + subprocess.getoutput('rustc --version') if ok else '✗'}")
    if ok or check:
        return ok
    if FAMILY == "windows":
        run("winget install -e --id Rustlang.Rustup --accept-source-agreements --accept-package-agreements", timeout=1200)
        if not have("cargo"):
            print("  ⚠ Fungua terminal MPYA kisha rudia: setup_all.py (PATH imebadilika)")
    else:
        run("curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable --profile minimal", timeout=1200)
        #PATH ya session hii
        os.environ["PATH"] = os.path.expanduser("~/.cargo/bin") + os.pathsep + os.environ.get("PATH", "")
    return have("cargo") and have("rustc")


def step_r(check: bool, skip: bool) -> bool:
    if skip:
        print("[3/9] R: IMERUKIWA (--skip-r)")
        return have("Rscript")
    ok = have("Rscript")
    print(f"[3/9] R: {'✔ ' + subprocess.getoutput('Rscript --version 2>&1').strip() if ok else '✗'}")
    if ok or check:
        return ok
    if FAMILY == "linux":
        pm = _linux_pkg()
        pkg = {"apt-get": "r-base", "dnf": "R", "pacman": "r"}[pm]
        run(f"{PKG['linux'][pm]} {pkg}", timeout=1200)
    elif FAMILY == "macos":
        run("brew install r", timeout=1200)
    else:
        run("winget install -e --id RProject.R --accept-source-agreements --accept-package-agreements", timeout=1200)
    return have("Rscript")


def step_office(check: bool, skip: bool) -> bool:
    """LibreOffice iliyoreshwa — kazi zote za ofisi ndani ya mfumo (Word/Excel/PPT/PDF)."""
    if skip:
        print("[4/9] LibreOffice: IMERUKIWA (--skip-office)")
        return have("libreoffice") or have("soffice")
    ok = have("libreoffice") or have("soffice")
    print(f"[4/9] LibreOffice: {'✔' if ok else '✗'}")
    if ok or check:
        return ok
    if FAMILY == "linux":
        pm = _linux_pkg()
        if pm == "apt-get":
            run(f"{PKG['linux']['apt']} libreoffice libreoffice-gtk3 libreoffice-l10n-sw", timeout=1800)
        elif pm == "dnf":
            run("sudo dnf install -y libreoffice", timeout=1800)
        else:
            run("sudo pacman -S --noconfirm libreoffice-fresh", timeout=1800)
    elif FAMILY == "macos":
        run("brew install --cask libreoffice", timeout=1800)
    else:
        run("winget install -e --id TheDocumentFoundation.LibreOffice --accept-source-agreements --accept-package-agreements", timeout=1800)
    return have("libreoffice") or have("soffice")


def binary_name() -> str:
    return {"windows": "MTAALAMU-Setup.exe", "darwin": "MTAALAMU-macos",
            "linux": "mtaalamu-linux"}.get(FAMILY, "mtaalamu")


def binary_path() -> str:
    p = os.path.join(HERE, "mtaalamu", "dist", binary_name())
    return p if os.path.exists(p) else ""


def step_geo(check: bool) -> bool:
    """Ramani data (geo.json + hierarchy index) — generators za stdlib, hakuna mtandao."""
    geojson = os.path.join(HERE, "web-r", "data", "geo.json")
    ok = os.path.exists(geojson) and os.path.getsize(geojson) > 50000
    print(f"[5/9] Ramani data (geo.json): {'✔ web-r/data/geo.json tayari' if ok else '– itatengenezwa'}")
    if ok or check:
        return ok
    try:
        run(f"{shlex_quote(sys.executable)} web-r/scripts/build_geo.py", timeout=120)
        run(f"{shlex_quote(sys.executable)} scripts/mtaalamu/gen_geo_index.py", timeout=120)
    except Exception as e:  # noqa: BLE001
        print(f"      ⚠ geo generators zimeshindikana ({e})")
    ok = os.path.exists(geojson)
    if not ok:
        print("      ⚠ geo.json haipo — RAMANI itatumia fallback (dashboard bado inafanya kazi)")
    return ok


def step_binary(check: bool) -> bool:
    """Binary (onefile) — Python NDANI yake. Account/admin/serve zote hii itatumika."""
    binp = binary_path()
    ok = bool(binp)
    print(f"[6/9] Binary (onefile, Python ndani): {'✔ ' + binp if ok else '✗ (itajengwa)'}")
    if ok or check:
        return ok
    # jenga yenyewe (PyInstaller inasakinishwa kiotomatiki ndani ya build_onefile)
    from mtaalamu.setup_builder import build_onefile
    rc = build_onefile()
    return rc == 0 and bool(binary_path())


def step_python_venv(check: bool, with_agent: bool) -> bool:
    """VENV NI HIARI — binary inatosha. --with-agent kwa MTECH agent (vision/control) + GUI."""
    venv = os.path.join(HERE, ".venv")
    pybin = os.path.join(venv, "Scripts", "python.exe") if FAMILY == "windows" else os.path.join(venv, "bin", "python")
    ok = os.path.exists(pybin)
    if not with_agent:
        print(f"[7/9] MTECH Agent venv (Python): {'✔ ' + venv + ' (tayari ipo)' if ok else '– IMERUKIWA (chaguomsingi: binary inatosha)'}")
        print("      weka --with-agent kama unataka agent ya vision/control + GUI (PySide6)")
        return True
    print(f"[7/9] Python venv + deps (agent/GUI): {'✔ ' + venv if ok else '✗'}")
    if ok or check:
        return ok
    run(f"{sys.executable} -m venv {shlex_quote(venv)}")
    if not os.path.exists(pybin):
        return False
    req = os.path.join(HERE, "..", "mtech-os", "agent", "requirements.txt")
    if os.path.exists(req):
        run(f"{shlex_quote(pybin)} -m pip install -q --upgrade pip")
        run(f"{shlex_quote(pybin)} -m pip install -q -r {shlex_quote(req)}", timeout=1200)
    return os.path.exists(pybin)


def shlex_quote(s: str) -> str:
    return "'" + s.replace("'", "'\"'\"'") + "'" if " " in s or "'" in s else s


def step_ollama(check: bool, no_model: bool) -> bool:
    ok = have("ollama")
    print(f"[8/9] Ollama: {'✔' if ok else '✗'}")
    if not ok and not check:
        if FAMILY == "linux":
            run("curl -fsSL https://ollama.com/install.sh | sh", timeout=1800)
        elif FAMILY == "macos":
            run("brew install ollama", timeout=1200)
        else:
            run("winget install -e --id Ollama.Ollama --accept-source-agreements --accept-package-agreements", timeout=1800)
        ok = have("ollama")
    if not ok:
        print("  ⚠ Ollama haijasakinishwa — pakua kutoka https://ollama.com/download kisha rudia")
        return False
    if no_model:
        print("      modeli: IMERUKIWA (--no-model) — baadaye: ollama pull qwen2.5vl:3b")
        return True
    # modeli: kama tayari ipo, ruka
    out = run("ollama list", timeout=30)[1]
    if "qwen2.5vl" in out:
        print("      modeli: qwen2.5vl ✔ (tayari ipo)")
        return True
    print("      inavuta qwen2.5vl:3b (~2.3GB — intaneti inahitajika, subiri)…")
    rc, _ = run("ollama pull qwen2.5vl:3b", timeout=3600)
    return rc == 0


def step_finish(check: bool, email: str | None, start: bool) -> None:
    py = "python" if FAMILY == "windows" else "python3"
    binp = binary_path()
    # BINARY-FIRST: kila kitu (account/admin/serve) kinatumia binary — Python NDANI yake
    mta = shlex_quote(binp) if binp else f"{py} -m mtaalamu"
    if check:
        print("[9/9] Mwisho (UKAGUZI TU — hakuna kinachosakinishwa)")
        return
    # AUTO-REGISTER: hakuna maswali — email ya default (au --email) + DIAMOND + admin
    mail = (email or "owner@mtaalamu.local").strip()
    print(f"[9/9] KUANZISHA KIOTOMATIKI (hakuna configuration — account: {mail})")
    print(f"      kipimo: {'BINARY (' + binary_name() + ') — hakuna Python inayohitajika' if binp else 'python -m mtaalamu'}")
    run(f"{mta} register {mail} INDIVIDUAL DIAMOND")
    run(f"{mta} admin unlock")   # owner-mode: zana ZOTE BURE
    if start:
        _start_server(py)


def _start_server(py: str) -> None:
    """Anzisha mtaalamu serve KIOTOMATIKI (detached) + fungua dashboard ya browser."""
    log = os.path.join(HERE, ".serve.log")
    if _api_up():
        print("      API tayari inaendesha (http://127.0.0.1:8795) ✔")
    else:
        kwargs: dict = {}
        if FAMILY == "windows":
            kwargs.update(creationflags=0x00000008)  # DETACHED_PROCESS
        else:
            kwargs.update(start_new_session=True)
        try:
            binp = binary_path()
            serve_cmd = [binp, "serve"] if binp else [py, "-m", "mtaalamu", "serve"]
            with open(log, "ab") as lf:
                subprocess.Popen(serve_cmd, cwd=HERE,
                                 stdout=lf, stderr=lf, stdin=subprocess.DEVNULL, **kwargs)
        except OSError as e:
            print(f"      ⚠ serve haikuanza ({e}) — endesha wewe: {py} -m mtaalamu serve")
            return
        import time as _t
        for _ in range(20):
            if _api_up():
                break
            _t.sleep(0.5)
        print("      API: http://127.0.0.1:8795 ✔ (log: .serve.log)" if _api_up()
              else "      ⚠ API haikufika kwa wakati — endesha: mtaalamu serve")
    # fungua dashboard (faili la ndani)
    dash = os.path.join(HERE, "web-html", "mtaalamu-unified.html")
    if os.path.exists(dash):
        import webbrowser
        webbrowser.open("file://" + dash.replace(os.sep, "/"))
        print("      Dashboard imefunguka kwenye browser ✔ (tabs 11)")


def _api_up() -> bool:
    try:
        import urllib.request
        with urllib.request.urlopen("http://127.0.0.1:8795/api/status", timeout=2):
            return True
    except Exception:  # noqa: BLE001
        return False


def main() -> None:
    args = sys.argv[1:]
    check = "--check" in args
    with_agent = "--with-agent" in args
    skip_rust = "--skip-rust" in args
    skip_r = "--skip-r" in args
    skip_office = "--skip-office" in args
    no_model = "--no-model" in args
    start = "--no-start" not in args
    email = None
    if "--email" in args:
        i = args.index("--email")
        email = args[i + 1] if i + 1 < len(args) else None
    print(f"══ MTAALAMU SMART — ONE-CLICK INSTALL ({FAMILY}, {'UKAGUZI TU' if check else 'KILA KITU — HAKUNA CONFIGURATION'}) ══")
    results = {
        "C (gcc/make)": step_c(check),
        "Rust (cargo)": step_rust(check, skip_rust),
        "R (Rscript)": step_r(check, skip_r),
        "LibreOffice": step_office(check, skip_office),
        "Ramani (geo.json)": step_geo(check),
        "Binary (onefile)": step_binary(check),
        "MTECH Agent venv": step_python_venv(check, with_agent),
        "Ollama + Qwen": step_ollama(check, no_model),
    }
    step_finish(check, email, start)
    print("══ MUHTASARI ══")
    for k, v in results.items():
        print(f"  {'✔' if v else '✗'} {k}")
    print("  ✔ Account (auto-register + ADMIN, zana BURE)" if not check else "  – account: (hakuna, ukaguzi tu)")
    if start and not check:
        print("  ✔ Server http://127.0.0.1:8795 + Dashboard (tabs 11)")
    bad = [k for k, v in results.items() if not v]
    print(f"\n{len(results)-len(bad)}/{len(results)} sawa." + (f" Zilizobaki: {', '.join(bad)} — endesha tena setup_all.py baada ya kuzirekebisha." if bad else " KILA KITU TAYARI — Dashboard iko browser yako!"))


if __name__ == "__main__":
    main()
