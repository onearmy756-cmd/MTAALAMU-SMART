#!/usr/bin/env python3
"""MTAALAMU SMART + MTECH OS — ONE-CLICK INSTALL (kila kitu kwa amri moja).

    python3 setup_all.py                 # kila kitu: C, Rust, R, Python, Ollama+Qwen, deps
    python3 setup_all.py --check         # angalia tu — usisakinise chochote
    python3 setup_all.py --skip-rust     # ruka Rust
    python3 setup_all.py --skip-r        # ruka R
    python3 setup_all.py --no-model      # usivute qwen2.5vl:3b (baadaye: ollama pull)

INAYOSAKINISHWA (kwa mpangilio, OS zote 3):
  1. Zana za C:        gcc/clang + make (+ build-essential / xcode CLT / VS Build Tools hint)
  2. Rust:             rustup.rs (rasmi) → cargo + rustc  [--skip-rust kwa kuruka]
  3. R:                R + Rscript (apt/brew/winget)      [--skip-r kwa kuruka]
  4. Python + venv:    venv ndani ya hermes-agent + deps zote (Pillow, pyautogui kwa win/mac)
  5. Ollama + Qwen:    ollama (rasmi) + ollama pull qwen2.5vl:3b  [--no-model kwa kuruka]
  6. Mwisho:           register + admin unlock + boot wiring + serve amri zinazoonyeshwa

HUDUMA ZA PYTHON (stdlib tu kwa mtaalamu; venv ina Pillow/pyautogui kwa MTECH agent):
  mtaalamu inafanya kazi BILA venv — venv ni kwa ajili ya MTECH OS agent + GUI.
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
    print(f"[1/6] Zana za C: gcc/clang {'✔' if ok else '✗'} · make {'✔' if mk else '✗'}")
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
        print("[2/6] Rust: IMERUKIWA (--skip-rust)")
        return have("cargo")
    ok = have("cargo") and have("rustc")
    print(f"[2/6] Rust: {'✔ ' + subprocess.getoutput('rustc --version') if ok else '✗'}")
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
        print("[3/6] R: IMERUKIWA (--skip-r)")
        return have("Rscript")
    ok = have("Rscript")
    print(f"[3/6] R: {'✔ ' + subprocess.getoutput('Rscript --version 2>&1').strip() if ok else '✗'}")
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


def step_python_venv(check: bool) -> bool:
    venv = os.path.join(HERE, ".venv")
    pybin = os.path.join(venv, "Scripts", "python.exe") if FAMILY == "windows" else os.path.join(venv, "bin", "python")
    ok = os.path.exists(pybin)
    print(f"[4/6] Python venv + deps: {'✔ ' + venv if ok else '✗'}")
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
    print(f"[5/6] Ollama: {'✔' if ok else '✗'}")
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


def step_finish(check: bool) -> None:
    print("[6/6] Mwisho — amri zako (nakili moja moja):")
    py = "python" if FAMILY == "windows" else "python3"
    print(f"""
  cd {HERE}
  {py} -m mtaalamu register BARUA-YAKO@mail.com INDIVIDUAL DIAMOND
  {py} -m mtaalamu admin unlock          # wewe mmiliki → zana ZOTE BURE
  {py} -m mtaalamu boot                  # ona kuanzia ukiwaka (checks halisi)
  {py} -m mtaalamu serve                 # kisha fungua web-html/mtaalamu-unified.html
""")
    if not check:
        try:
            reg = input("  Barua pepe ya kujisajili (Enter = ruka): ").strip()
        except (EOFError, KeyboardInterrupt):
            reg = ""  # bila terminal ya mwingiliano — ruka register (unaweza baadaye)
        if reg:
            run(f"{py} -m mtaalamu register {reg} INDIVIDUAL DIAMOND")
            run(f"{py} -m mtaalamu admin unlock")


def main() -> None:
    args = sys.argv[1:]
    check = "--check" in args
    skip_rust = "--skip-rust" in args
    skip_r = "--skip-r" in args
    no_model = "--no-model" in args
    print(f"══ MTAALAMU SMART — ONE-CLICK INSTALL ({FAMILY}, {'UKAGUZI TU' if check else 'SAKINISHA KILA KITU'}) ══")
    results = {
        "C (gcc/make)": step_c(check),
        "Rust (cargo)": step_rust(check, skip_rust),
        "R (Rscript)": step_r(check, skip_r),
        "Python venv": step_python_venv(check),
        "Ollama + Qwen": step_ollama(check, no_model),
    }
    step_finish(check)
    print("══ MUHTASARI ══")
    for k, v in results.items():
        print(f"  {'✔' if v else '✗'} {k}")
    bad = [k for k, v in results.items() if not v]
    print(f"\n{len(results)-len(bad)}/{len(results)} sawa." + (f" Zilizobaki: {', '.join(bad)} — endesha tena setup_all.py baada ya kuzirekebisha." if bad else " KILA KITU TAYARI!"))


if __name__ == "__main__":
    main()
