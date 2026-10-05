"""MTAALAMU SMART — ENV SETUP (mazingira HALISI ndani ya OS yako).

Ukishakuwa ndani ya MTECH OS (au OS yoyote), hii ndiyo sehemu ya KUSAKINISHA
vifaa vya akili na kasi — bila kutoka kwenye mfumo:

    mtaalamu env                      # hali ya mazingira yote (checks halisi)
    mtaalamu env install ollama       # Ollama (rasmi: curl/brew/winget)
    mtaalamu env install qwen         # qwen2.5vl:3b (huanzisha ollama serve kwanza)
    mtaalamu env install rust         # rustup rasmi → cargo + rustc
    mtaalamu env install engine       # jenga engine ya Rust (cargo build --release)
    mtaalamu env install r            # R (apt/brew/winget)
    mtaalamu env install r-packages   # shiny + jsonlite (user-lib, bila maswali)
    mtaalamu env install libreoffice  # ofisi kamili
    mtaalamu env install node         # Node.js (npm — kwa website/desktop)
    mtaalamu env install all          # yote kwa mpangilio salama

ENVIRONMENT NZURI:
  • process_env()  — PATH kamili (~/.cargo/bin, /usr/local/bin) kwa kila
                     process tunayoianzisha (cargo inafanya kazi BAADA ya rustup!)
  • ensure_dirs()  — ~/.mtaalamu/{logs,models,data} zinaundwa waziwazi
  • Ollama serve   — inaanzishwa yenyewe kama API haipatikani, kisha model inavutwa

API (mtaalamu serve):
  GET  /api/env                  → status()
  POST /api/env/install          → {"target": "qwen"} (mf: ollama|qwen|rust|engine|r|r-packages|libreoffice|node|all)
"""
import json
import os
import platform
import shutil
import subprocess
import time
import urllib.request
from pathlib import Path

FAMILY = "windows" if os.name == "nt" else ("macos" if platform.system() == "Darwin" else "linux")
HOME = Path.home()
ROOT = Path(__file__).resolve().parents[1]   # hermes-agent/

# ---------------------------------------------------------------- environment nzuri
def ensure_dirs() -> dict:
    """Folda za mfumo wa MTAALAMU — zinapatikana kwa kila kipengele."""
    made = []
    for d in (HOME / ".mtaalamu" / "logs", HOME / ".mtaalamu" / "models",
              HOME / ".mtaalamu" / "data", HOME / ".mtaalamu" / "apps"):
        if not d.exists():
            d.mkdir(parents=True, exist_ok=True)
            made.append(str(d))
    return {"ok": True, "created": made, "base": str(HOME / ".mtaalamu")}


def process_env(extra: dict | None = None) -> dict:
    """ENV nzuri ya processes: PATH ina ~/.cargo/bin + /usr/local/bin (cargo HUUFA baada ya rustup)."""
    env = dict(os.environ)
    add = [str(HOME / ".cargo" / "bin"), "/usr/local/bin", "/usr/bin", "/bin"]
    cur = env.get("PATH", "")
    for p in reversed(add):
        if p not in cur.split(os.pathsep):
            cur = p + os.pathsep + cur
    env["PATH"] = cur
    env.setdefault("R_LIBS_USER", str(HOME / "R" / "library"))
    if extra:
        env.update(extra)
    return env


def _run(cmd: str, timeout: int = 30, env: dict | None = None) -> tuple[int, str]:
    e = env or process_env()
    try:
        p = subprocess.run(cmd, shell=True, capture_output=True, text=True, timeout=timeout, env=e)
        return p.returncode, (p.stdout or p.stderr or "").strip()
    except (OSError, subprocess.TimeoutExpired) as ex:
        return -1, str(ex)[:200]


def _pkg_manager() -> str:
    for pm in ("apt-get", "dnf", "pacman"):
        if shutil.which(pm):
            return pm
    return "apt-get"


# ---------------------------------------------------------------- Ollama + Qwen
OLLAMA_API = "http://127.0.0.1:11434"


def ollama_api_up() -> bool:
    try:
        with urllib.request.urlopen(f"{OLLAMA_API}/api/tags", timeout=2):
            return True
    except Exception:  # noqa: BLE001
        return False


def ollama_models() -> list[str]:
    try:
        with urllib.request.urlopen(f"{OLLAMA_API}/api/tags", timeout=3) as r:
            data = json.loads(r.read().decode())
        return [m.get("name", "") for m in data.get("models", [])]
    except Exception:  # noqa: BLE001
        return []


def qwen_pulled() -> bool:
    return any("qwen2.5vl" in m for m in ollama_models())


def ollama_serve_start() -> dict:
    """Anzisha `ollama serve` detached (env nzuri) + subiri API."""
    if ollama_api_up():
        return {"ok": True, "note": "API tayari inaendesha"}
    exe = shutil.which("ollama") or "/usr/local/bin/ollama"
    if not (Path(exe).exists() or shutil.which("ollama")):
        return {"ok": False, "error": "ollama haipo — endesha: mtaalamu env install ollama"}
    log_dir = HOME / ".mtaalamu" / "logs"
    log_dir.mkdir(parents=True, exist_ok=True)
    kwargs: dict = {}
    if FAMILY != "windows":
        kwargs["start_new_session"] = True
    try:
        with open(log_dir / "ollama-serve.log", "ab") as lf:
            subprocess.Popen([exe, "serve"], stdin=subprocess.DEVNULL,
                             stdout=lf, stderr=subprocess.STDOUT, env=process_env(), **kwargs)
    except OSError as e:
        return {"ok": False, "error": f"{type(e).__name__}: {e}"}
    for _ in range(24):  # hadi ~12s
        if ollama_api_up():
            return {"ok": True, "note": "ollama serve imeanza (11434)"}
        time.sleep(0.5)
    return {"ok": False, "error": "ollama serve haikufika — ona ~/.mtaalamu/logs/ollama-serve.log"}


def _install_ollama() -> dict:
    if shutil.which("ollama") or Path("/usr/local/bin/ollama").exists():
        return {"ok": True, "note": "Ollama tayari ipo — " + (shutil.which("ollama") or "/usr/local/bin/ollama")}
    if FAMILY == "linux":
        code, out = _run("curl -fsSL https://ollama.com/install.sh | sh", timeout=1800)
    elif FAMILY == "macos":
        code, out = _run("brew install ollama", timeout=1800)
    else:
        code, out = _run("winget install -e --id Ollama.Ollama --accept-source-agreements --accept-package-agreements", timeout=1800)
    ok = code == 0 and (shutil.which("ollama") or Path("/usr/local/bin/ollama").exists())
    return {"ok": ok, "output": out[-500:], "note": "kama imeshindikana: https://ollama.com/download"}


def _install_qwen() -> dict:
    serve = ollama_serve_start()
    if not serve.get("ok"):
        return serve
    if qwen_pulled():
        return {"ok": True, "note": "qwen2.5vl:3b tayari ipo ✔"}
    code, out = _run("ollama pull qwen2.5vl:3b", timeout=3600)
    return {"ok": code == 0, "output": out[-500:],
            "note": "~2.3GB — mtandao unahitajika" if code != 0 else "qwen2.5vl:3b iko tayari ✔"}


# ---------------------------------------------------------------- Rust + engine
def _install_rust() -> dict:
    if shutil.which("cargo") and shutil.which("rustc"):
        return {"ok": True, "note": "Rust tayari ipo: " + subprocess.getoutput("rustc --version")}
    if FAMILY == "windows":
        code, out = _run("winget install -e --id Rustlang.Rustup --accept-source-agreements --accept-package-agreements", timeout=1800)
    else:
        code, out = _run("curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable --profile minimal", timeout=1800)
    # PATH mpya inaingia kwenye process_env() — hii ndiyo 'cargo ifanye kazi' ya kweli
    ok = shutil.which("cargo", path=process_env()["PATH"]) is not None
    return {"ok": ok, "output": out[-400:],
            "note": "cargo imeongezwa kwenye PATH ya mtaalamu (fungua terminal mpya kwa shell zingine)"}


def _build_engine() -> dict:
    cargo = shutil.which("cargo", path=process_env()["PATH"])
    d = ROOT / "engine-rust"
    if not (d / "Cargo.toml").exists():
        return {"ok": False, "error": "engine-rust haipo"}
    if not cargo:
        return {"ok": False, "error": "cargo haipo — endesha: mtaalamu env install rust"}
    code, out = _run(f"cd '{d}' && cargo build --release", timeout=1800)
    binary = d / "target" / "release" / ("mtaalamu.exe" if FAMILY == "windows" else "mtaalamu")
    if code == 0 and binary.exists():
        try:
            link = Path("/usr/local/bin/mtaalamu-engine")
            if link.parent.exists() and FAMILY == "linux":
                if link.exists() or link.is_symlink():
                    link.unlink()
                link.symlink_to(binary)
        except OSError:
            pass
        return {"ok": True, "binary": str(binary), "note": "engine ya Rust imejengwa ✔ (mtaalamu-engine)"}
    return {"ok": False, "output": out[-600:]}


# ---------------------------------------------------------------- R + packages
def _install_r() -> dict:
    if shutil.which("Rscript"):
        return {"ok": True, "note": "R tayari ipo"}
    if FAMILY == "linux":
        pm = _pkg_manager()
        pkg = {"apt-get": "r-base r-base-dev", "dnf": "R", "pacman": "r"}[pm]
        code, out = _run(("sudo " if pm == "apt-get" else "") + f"{pm} install -y {pkg}", timeout=1800)
    elif FAMILY == "macos":
        code, out = _run("brew install r", timeout=1800)
    else:
        code, out = _run("winget install -e --id RProject.R --accept-source-agreements --accept-package-agreements", timeout=1800)
    return {"ok": bool(shutil.which("Rscript")), "output": out[-400:]}


def _install_r_packages() -> dict:
    """shiny + jsonlite kwenye user-library — BILA maswali (non-interactive)."""
    rs = shutil.which("Rscript")
    if not rs:
        return {"ok": False, "error": "Rscript haipo — endesha: mtaalamu env install r"}
    expr = ('dir.create(Sys.getenv("R_LIBS_USER"), recursive=TRUE, showWarnings=FALSE); '
            'install.packages(c("shiny","jsonlite","curl"), lib=Sys.getenv("R_LIBS_USER"), '
            'repos="https://cloud.r-project.org", quiet=TRUE); '
            'cat(requireNamespace("shiny",quietly=TRUE), requireNamespace("jsonlite",quietly=TRUE))')
    code, out = _run(f"'{rs}' -e '{expr}'", timeout=1800)
    ok = out.strip().endswith("1 1") or (code == 0 and "1 1" in out)
    return {"ok": ok, "output": out[-300:], "lib": os.environ.get("R_LIBS_USER", str(HOME / "R" / "library"))}


# ---------------------------------------------------------------- LibreOffice / Node
def _install_libreoffice() -> dict:
    if shutil.which("libreoffice") or shutil.which("soffice"):
        return {"ok": True, "note": "LibreOffice tayari ipo"}
    if FAMILY == "linux":
        pm = _pkg_manager()
        pkg = {"apt-get": "libreoffice libreoffice-gtk3", "dnf": "libreoffice", "pacman": "libreoffice-fresh"}[pm]
        code, out = _run(("sudo " if pm == "apt-get" else "") + f"{pm} install -y {pkg}", timeout=2400)
    elif FAMILY == "macos":
        code, out = _run("brew install --cask libreoffice", timeout=2400)
    else:
        code, out = _run("winget install -e --id TheDocumentFoundation.LibreOffice --accept-source-agreements --accept-package-agreements", timeout=2400)
    return {"ok": bool(shutil.which("libreoffice") or shutil.which("soffice")), "output": out[-400:]}


def _install_node() -> dict:
    if shutil.which("node") and shutil.which("npm"):
        return {"ok": True, "note": "Node tayari ipo: " + subprocess.getoutput("node --version")}
    if FAMILY == "linux":
        pm = _pkg_manager()
        pkg = {"apt-get": "nodejs npm", "dnf": "nodejs npm", "pacman": "nodejs npm"}[pm]
        code, out = _run(("sudo " if pm == "apt-get" else "") + f"{pm} install -y {pkg}", timeout=1800)
    elif FAMILY == "macos":
        code, out = _run("brew install node", timeout=1800)
    else:
        code, out = _run("winget install -e --id OpenJS.NodeJS.LTS --accept-source-agreements --accept-package-agreements", timeout=1800)
    return {"ok": bool(shutil.which("node")), "output": out[-400:]}


TARGETS = {
    "ollama": _install_ollama,
    "qwen": _install_qwen,
    "rust": _install_rust,
    "engine": _build_engine,
    "r": _install_r,
    "r-packages": _install_r_packages,
    "libreoffice": _install_libreoffice,
    "node": _install_node,
}

ORDER = ("rust", "engine", "r", "r-packages", "ollama", "qwen", "node", "libreoffice")


# ---------------------------------------------------------------- STATUS
def status() -> dict:
    env = process_env()
    cargo = shutil.which("cargo", path=env["PATH"])
    rustc = shutil.which("rustc", path=env["PATH"])
    engine = (ROOT / "engine-rust" / "target" / "release" / ("mtaalamu.exe" if FAMILY == "windows" else "mtaalamu"))
    rs = shutil.which("Rscript")
    r_pkgs = "—"
    if rs:
        code, out = _run(f"'{rs}' -e 'cat(requireNamespace(\"shiny\",quietly=TRUE), requireNamespace(\"jsonlite\",quietly=TRUE))'", timeout=60)
        r_pkgs = "shiny✔ jsonlite✔" if (code == 0 and "1 1" in out) else ("shiny/jsonlite ✗ — env install r-packages")
    models = ollama_models() if ollama_api_up() else []
    return {
        "family": FAMILY,
        "dirs": ensure_dirs(),
        "python": platform.python_version(),
        "cargo": {"ok": bool(cargo and rustc), "version": subprocess.getoutput("rustc --version") if rustc else "—",
                  "fix": "mtaalamu env install rust"},
        "engine": {"ok": engine.exists(), "path": str(engine) if engine.exists() else "—",
                   "fix": "mtaalamu env install engine"},
        "r": {"ok": bool(rs), "packages": r_pkgs, "fix": "mtaalamu env install r-packages"},
        "ollama": {"ok": shutil.which("ollama") is not None or Path("/usr/local/bin/ollama").exists(),
                   "api_up": ollama_api_up(), "models": models[:10],
                   "qwen2_5vl": any("qwen2.5vl" in m for m in models),
                   "fix": "mtaalamu env install ollama && mtaalamu env install qwen"},
        "node": {"ok": bool(shutil.which("node")), "fix": "mtaalamu env install node"},
        "libreoffice": {"ok": bool(shutil.which("libreoffice") or shutil.which("soffice")),
                        "fix": "mtaalamu env install libreoffice"},
        "docker": {"ok": bool(shutil.which("docker") or shutil.which("podman"))},
        "path_preview": env["PATH"][:300],
    }


def install(target: str) -> dict:
    target = (target or "").strip().lower()
    ensure_dirs()
    if target == "all":
        results = {}
        for t in ORDER:
            results[t] = TARGETS[t]()
        good = sum(1 for v in results.values() if v.get("ok"))
        return {"ok": good == len(results), "installed": good, "total": len(results), "results": results}
    if target not in TARGETS:
        return {"ok": False, "error": f"target haijulikani: {target} (zipo: {', '.join(TARGETS)})"}
    return TARGETS[target]()


# ---------------------------------------------------------------- CLI
def main() -> None:
    import sys
    args = sys.argv[1:]
    if args and args[0] == "install" and len(args) > 1:
        print(json.dumps(install(args[1]), ensure_ascii=False, indent=1))
    else:
        print(json.dumps(status(), ensure_ascii=False, indent=1, default=str))


if __name__ == "__main__":
    main()
