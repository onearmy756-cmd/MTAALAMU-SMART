"""MTAALAMU SMART — SYSTEM APPS (apps za hermes-agent zimeunganishwa na MTECH OS).

Hizi ndiyo apps ZA MFUMO — zinaonekana watu wanaposakinisha apps nyingine,
na UI zake NI ZILE za waundaji (halisi, hakuna kubuni):

  • OpenMRS          — EHR ya hospitali (Java/Tomcat, UI ya OpenMRS halisi)
  • Home Assistant   — nyumba mahiri (Python, UI rasmi ya HA)
  • Web R            — dashboard ya Shiny (R) ya analytics
  • Hermes Desktop   — app ya Electron (UI rasmi ya desktop)
  • MTAALAMU Website — Docusaurus docs site
  • MTAALAMU Engine  — Rust engine (reason/knowledge/solve — CLI halisi)
  • Analytics R      — Rscript analytics.R (riporti + chati)

Kila app ina:
  status()  — HALISI: uwepo wa code, uwepo wa runtime, uwepo wa data
  launch()  — inazindua UI halisi (docker compose up / Rscript shiny / electron)
  stop()    — inasimamisha (docker compose down / pkill)

Tumia:
  python3 -m mtaalamu.system_apps            # hali ya apps zote
  python3 -m mtaalamu.system_apps launch web-r
  python3 -m mtaalamu.system_apps stop openmrs

API (mtaalamu serve):
  GET  /api/apps              → hali zote
  POST /api/apps/launch       {"app": "web-r"}
  POST /api/apps/stop         {"app": "openmrs"}
"""
import os
import platform
import shutil
import subprocess
import urllib.request
from pathlib import Path

FAMILY = "windows" if os.name == "nt" else ("macos" if platform.system() == "Darwin" else "linux")
ROOT = Path(__file__).resolve().parents[1]        # hermes-agent/
WEB_DIR = ROOT / "web-html"
# MTECH OS inapoendesha kwenye ISO (/opt/mtaalamu/hermes-agent), ROOT ni hapa hapa.

OPEN = {}   # ports zilizofunguliwa na app hii module (session hii)

# ---------------------------------------------------------------- helpers
def _run(cmd: str, timeout: int = 12) -> tuple[int, str]:
    try:
        p = subprocess.run(cmd, shell=True, capture_output=True, text=True, timeout=timeout)
        return p.returncode, (p.stdout or p.stderr or "").strip()
    except (OSError, subprocess.TimeoutExpired) as e:
        return -1, str(e)[:200]


def _port_up(port: int, path: str = "/") -> bool:
    try:
        with urllib.request.urlopen(f"http://127.0.0.1:{port}{path}", timeout=1.5) as r:
            return r.status < 500
    except Exception:  # noqa: BLE001
        return False


def _spawn(args: list[str], name: str, port: int | None = None, log: str = "", cwd: str = "") -> dict:
    kwargs: dict = {}
    if FAMILY != "windows":
        kwargs["start_new_session"] = True
    try:
        lf = open(log, "ab") if log else subprocess.DEVNULL
        subprocess.Popen(args, cwd=cwd or str(ROOT), stdin=subprocess.DEVNULL,
                         stdout=lf, stderr=subprocess.STDOUT, **kwargs)
    except OSError as e:
        return {"ok": False, "error": f"{type(e).__name__}: {e}"}
    if port:
        OPEN[name] = port
    return {"ok": True}


def _docker() -> str | None:
    return shutil.which("docker") or shutil.which("podman")


def _open_browser(url: str) -> None:
    try:
        import webbrowser
        webbrowser.open(url)
    except Exception:  # noqa: BLE001
        pass


# ---------------------------------------------------------------- app definitions
def _app_dir(name: str) -> Path:
    return ROOT / name


def app_openmrs() -> dict:
    """OpenMRS — EHR halisi (docker compose up → UI: http://localhost:8080/openmrs)."""
    d = _app_dir("openmrs")
    has_src = (d / "pom.xml").exists()
    docker = _docker()
    up = _port_up(8080, "/openmrs")
    return {
        "id": "openmrs", "name": "OpenMRS (EHR ya Hospitali)",
        "desc": "Rekodi za wagonjwa — UI rasmi ya OpenMRS (Java/Tomcat)",
        "ui": "http://localhost:8080/openmrs",
        "source": str(d.relative_to(ROOT.parent)) if has_src else "",
        "installed": has_src,
        "runtime": docker or "java/mvn (docker inapendekezwa)",
        "runtime_ok": bool(docker),
        "running": up,
        "launch_hint": "docker compose up -d (openmrs/docker-compose.yml)",
    }


def app_home_assistant() -> dict:
    """Home Assistant — nyumba mahiri (UI rasmi ya HA: http://localhost:8123)."""
    d = _app_dir("home-assistant")
    has_src = (d / "pyproject.toml").exists() and (d / "homeassistant").is_dir()
    up = _port_up(8123)
    return {
        "id": "home-assistant", "name": "Home Assistant (Nyumba Mahiri)",
        "desc": "UI rasmi ya Home Assistant — dashboard ya vifaa vya nyumba",
        "ui": "http://localhost:8123",
        "source": str(d.relative_to(ROOT.parent)) if has_src else "",
        "installed": has_src,
        "runtime": "docker au python homeassistant",
        "runtime_ok": bool(_docker()),
        "running": up,
        "launch_hint": "docker run -d --network host -v ~/.ha:/config ghcr.io/home-assistant/home-assistant",
    }


def app_web_r() -> dict:
    """Web R — dashboard ya Shiny (R halisi) — port 3838 (README ya web-r).

    Tabu za ndani zilizoshughulikiwa:
      • port 3838 (README) — si 8089, si default 8100
      • inahitaji shiny + jsonlite (auto-install kwenye launch)
      • CWD lazima iwe web-r/ (app.R inasoma R/ + data kutoka ../data)
      • data/ haiipo ndani ya web-r — p_root("data") = hermes-agent/data ✔
    """
    d = _app_dir("web-r")
    app = d / "app.R"
    has_src = app.exists()
    rscript = shutil.which("Rscript")
    up = _port_up(3838)
    libs_ok = False
    if rscript:
        code, out = _run(f"'{rscript}' -e 'cat(requireNamespace(\"shiny\",quietly=TRUE), requireNamespace(\"jsonlite\",quietly=TRUE))'", timeout=60)
        libs_ok = code == 0 and "1 1" in out
    return {
        "id": "web-r", "name": "Web R (Dashboard ya Analytics)",
        "desc": "App ya Shiny (R) — chati, ramani ya TZ (Leaflet), agentic vision; port 3838",
        "ui": "http://localhost:3838",
        "source": str(d.relative_to(ROOT.parent)) if has_src else "",
        "installed": has_src,
        "runtime": "Rscript + shiny + jsonlite",
        "runtime_ok": bool(rscript) and libs_ok,
        "running": up,
        "launch_hint": "mtaalamu apps launch web-r (au: cd web-r && Rscript -e 'shiny::runApp(\"app.R\", port=3838)')",
        "fix": "" if libs_ok else "mtaalamu env install r-packages (shiny + jsonlite)",
    }


def app_desktop() -> dict:
    """Hermes Desktop — app ya Electron (UI rasmi ya desktop ya hermes-agent)."""
    d = ROOT / "apps" / "desktop"
    has_src = (d / "package.json").exists()
    node = shutil.which("node")
    npm = shutil.which("npm")
    return {
        "id": "desktop", "name": "Hermes Desktop (App ya Electron)",
        "desc": "UI rasmi ya desktop ya hermes-agent (Electron + Vite)",
        "ui": "electron window (npm start ndani ya apps/desktop)",
        "source": str(d.relative_to(ROOT.parent)) if has_src else "",
        "installed": has_src,
        "runtime": "node + npm (electron-builder kwa binary)",
        "runtime_ok": bool(node and npm),
        "running": False,
        "launch_hint": "cd apps/desktop && npm install && npm start",
    }


def app_website() -> dict:
    """MTAALAMU Website — Docusaurus (UI rasmi ya docs)."""
    d = _app_dir("website")
    has_src = (d / "package.json").exists() and (d / "docusaurus.config.ts").exists()
    node = shutil.which("node")
    return {
        "id": "website", "name": "MTAALAMU Website (Docusaurus)",
        "desc": "Tovuti rasmi ya docs (Docusaurus) — UI ya waundaji (baseUrl: /docs/)",
        "ui": "http://localhost:3000/docs/",
        "source": str(d.relative_to(ROOT.parent)) if has_src else "",
        "installed": has_src,
        "runtime": "node + npm (docusaurus start)",
        "runtime_ok": bool(node),
        "running": _port_up(3000),
        "launch_hint": "cd website && npm install && npm start",
    }


def app_engine() -> dict:
    """MTAALAMU Engine — Rust (kasi + usalama wa OS)."""
    d = _app_dir("engine-rust")
    has_src = (d / "Cargo.toml").exists()
    cargo = shutil.which("cargo")
    binary = d / "target" / "release" / ("mtaalamu.exe" if FAMILY == "windows" else "mtaalamu")
    built = binary.exists()
    return {
        "id": "engine-rust", "name": "MTAALAMU Engine (Rust)",
        "desc": "Engine ya kasi na usalama: reason, knowledge, solve, agentic (CLI halisi)",
        "ui": "mtaalamu reason 'swali' (terminal) — CLI halisi ya Rust",
        "source": str(d.relative_to(ROOT.parent)) if has_src else "",
        "installed": has_src,
        "runtime": "cargo (rustup) — au binary ya release",
        "runtime_ok": bool(cargo or built),
        "running": False,
        "launch_hint": "cargo run --release -- reason 'swali'",
    }


def app_analytics_r() -> dict:
    """Analytics R — ripoti + chati za OS (R halisi)."""
    d = _app_dir("analytics-r")
    has_src = (d / "analytics.R").exists()
    rscript = shutil.which("Rscript")
    return {
        "id": "analytics-r", "name": "Analytics R (Ripoti + Chati)",
        "desc": "R halisi: muhtasari, Bayesian cross-check, charti za matatizo (JSON/PNG)",
        "ui": "riporti za JSON/chati — Rscript analytics.R results.json",
        "source": str(d.relative_to(ROOT.parent)) if has_src else "",
        "installed": has_src,
        "runtime": "Rscript (jsonlite haihitajiki — ina fallback)",
        "runtime_ok": bool(rscript),
        "running": False,
        "launch_hint": "Rscript analytics-r/analytics.R",
    }


APPS = {
    "openmrs": app_openmrs,
    "home-assistant": app_home_assistant,
    "web-r": app_web_r,
    "desktop": app_desktop,
    "website": app_website,
    "engine-rust": app_engine,
    "analytics-r": app_analytics_r,
}


# ---------------------------------------------------------------- launch / stop
def launch(app_id: str) -> dict:
    """Zindua UI HALISI ya app ya mfumo (inayoonekana kwa mtumiaji mara moja)."""
    app_id = (app_id or "").strip().lower()
    if app_id not in APPS:
        return {"ok": False, "error": f"app haijulikani: {app_id} (zipo: {', '.join(APPS)})"}

    if app_id == "openmrs":
        docker = _docker()
        if not docker:
            return {"ok": False, "error": "docker haipo — sakinisha docker kisha rudia (OpenMRS ina Dockerfile rasmi)"}
        return {**_spawn([docker, "compose", "up", "-d"], app_id,
                         log=str(ROOT / ".openmrs.log")),
                "note": "OpenMRS inajengwa mara ya kwanza (dakika chache) — UI: http://localhost:8080/openmrs",
                "ui": "http://localhost:8080/openmrs"}

    if app_id == "home-assistant":
        docker = _docker()
        if docker:
            ha_cfg = Path.home() / ".mtaalamu" / "home-assistant"
            ha_cfg.mkdir(parents=True, exist_ok=True)
            return {**_spawn([docker, "run", "-d", "--name", "mtech-home-assistant",
                              "--restart", "unless-stopped", "--network", "host",
                              "-v", f"{ha_cfg}:/config",
                              "ghcr.io/home-assistant/home-assistant:stable"], app_id,
                             log=str(ROOT / ".homeassistant.log")),
                    "ui": "http://localhost:8123",
                    "note": f"config: {ha_cfg}"}
        # bila docker: python venv ya HA (source ipo)
        return {"ok": False, "error": "docker haipo — Home Assistant inaendesha vizuri zaidi ndani ya container (ghcr.io/home-assistant/home-assistant)"}

    if app_id == "web-r":
        rscript = shutil.which("Rscript")
        if not rscript:
            return {"ok": False, "error": "Rscript haipo — endesha: mtaalamu env install r", "fix": "mtaalamu env install r"}
        app = ROOT / "web-r" / "app.R"
        if not app.exists():
            return {"ok": False, "error": "web-r/app.R haipo"}
        # Tabu za ndani: shiny/jsonlite zinaweza kukosekana → install kiotomatiki
        # (user-lib, bila maswali); CWD=web-r; port 3838 (README ya web-r).
        code, out = _run(f"'{rscript}' -e 'cat(requireNamespace(\"shiny\",quietly=TRUE))'", timeout=60)
        if "1" not in out:
            print("[web-r] shiny haipo — inasakinishwa (dakika 1-3, CRAN)…")
            from . import envsetup
            envsetup.ensure_dirs()
            envsetup._install_r_packages()
        # CWD lazima iwe web-r/ (app.R inasoma R/*.R kutoka APP_DIR na data/ kutoka p_root)
        return {**_spawn([rscript, "-e", 'shiny::runApp("app.R", port=3838, launch.browser=FALSE)'],
                         app_id, port=3838, cwd=str(ROOT / "web-r"),
                         log=str(ROOT / ".webr.log")),
                "ui": "http://localhost:3838",
                "note": "Shiny inafunguka — subiri sekunde 5 kisha fungua UI (port 3838)"}

    if app_id == "desktop":
        npm = shutil.which("npm")
        if not npm:
            return {"ok": False, "error": "npm haipo — sakinisha Node.js kisha rudia"}
        d = ROOT / "apps" / "desktop"
        if not (d / "node_modules").exists():
            _run(f"cd {shlex_q(str(d))} && npm install --no-audit --no-fund", timeout=600)
        return {**_spawn([npm, "start"], app_id, cwd=str(d), log=str(ROOT / ".desktop.log")),
                "note": "Dirisha la Hermes Desktop litafunguka (Electron)"}

    if app_id == "website":
        npm = shutil.which("npm")
        if not npm:
            return {"ok": False, "error": "npm haipo — sakinisha Node.js kisha rudia"}
        d = ROOT / "website"
        if not (d / "node_modules").exists():
            _run(f"cd {shlex_q(str(d))} && npm install --no-audit --no-fund", timeout=600)
        return {**_spawn([npm, "start"], app_id, cwd=str(d), port=3000, log=str(ROOT / ".website.log")),
                "ui": "http://localhost:3000"}

    if app_id == "engine-rust":
        cargo = shutil.which("cargo")
        binary = ROOT / "engine-rust" / "target" / "release" / ("mtaalamu.exe" if FAMILY == "windows" else "mtaalamu")
        if binary.exists():
            code, out = _run(f"{shlex_q(str(binary))} help", timeout=30)
            return {"ok": code == 0, "output": out[:1200], "ui": str(binary),
                    "note": "CLI halisi ya Rust — mfano: mtaalamu reason 'swali'"}
        if not cargo:
            return {"ok": False, "error": "cargo haipo — rustup.rs kisha rudia"}
        code, out = _run(f"cd {shlex_q(str(ROOT / 'engine-rust'))} && cargo build --release", timeout=600)
        return {"ok": code == 0, "output": out[-800:], "ui": str(binary),
                "note": "Engine imejengwa (release) — endesha: mtaalamu reason 'swali'"}

    if app_id == "analytics-r":
        rscript = shutil.which("Rscript")
        if not rscript:
            return {"ok": False, "error": "Rscript haipo — sakinisha R kisha rudia"}
        code, out = _run(f"{shlex_q(rscript)} {shlex_q(str(ROOT / 'analytics-r' / 'analytics.R'))}", timeout=120)
        return {"ok": code == 0, "output": out[:1500],
                "note": "Ripoti ya R imekamilika (JSON stdout)"}

    return {"ok": False, "error": f"app haijulikani: {app_id}"}


def stop(app_id: str) -> dict:
    app_id = (app_id or "").strip().lower()
    if app_id not in APPS:
        return {"ok": False, "error": f"app haijulikani: {app_id}"}
    if app_id == "openmrs":
        docker = _docker()
        if docker:
            return {**_run2ok(f"cd {shlex_q(str(ROOT / 'openmrs'))} && {docker} compose down --remove-orphans", timeout=90),
                    "note": "OpenMRS containers zimesimama"}
        return {"ok": False, "error": "docker haipo"}
    if app_id == "home-assistant":
        docker = _docker()
        if docker:
            return _run2ok(f"{docker} rm -f mtech-home-assistant")
        return {"ok": False, "error": "docker haipo"}
    if app_id in ("web-r", "website"):
        pat = "app.R|Rscript" if app_id == "web-r" else "docusaurus"
        if FAMILY == "windows":
            return _run2ok(f"taskkill /F /FI \"WINDOWTITLE eq {pat}\" 2>nul & exit 0")
        return _run2ok(f"pkill -f '{pat}' 2>/dev/null; exit 0")
    return {"ok": False, "error": f"app '{app_id}' haina background process (zindua/kata kutoka terminal)"}


def _run2ok(cmd: str, timeout: int = 60) -> dict:
    code, out = _run(cmd, timeout)
    return {"ok": code == 0, "output": out[:800]}


def shlex_q(s: str) -> str:
    return "'" + s.replace("'", "'\"'\"'") + "'" if (" " in s or "'" in s) else s


# ---------------------------------------------------------------- API
def status() -> dict:
    apps = [fn() for fn in APPS.values()]
    return {"apps": apps, "count": len(apps),
            "running": [a["id"] for a in apps if a.get("running")],
            "available": [a["id"] for a in apps if a.get("installed")],
            "runtimes": {
                "docker": bool(_docker()), "node": bool(shutil.which("node")),
                "npm": bool(shutil.which("npm")), "Rscript": bool(shutil.which("Rscript")),
                "cargo": bool(shutil.which("cargo")),
            }}


# ---------------------------------------------------------------- CLI
def main() -> None:
    import sys
    cmd = sys.argv[1] if len(sys.argv) > 1 else ""
    if cmd == "launch" and len(sys.argv) > 2:
        import json
        out = launch(sys.argv[2])
        if out.get("ok") and out.get("ui", "").startswith("http"):
            _open_browser(out["ui"])
        print(json.dumps(out, ensure_ascii=False, indent=1))
    elif cmd == "stop" and len(sys.argv) > 2:
        import json
        print(json.dumps(stop(sys.argv[2]), ensure_ascii=False, indent=1))
    else:
        import json
        print(json.dumps(status(), ensure_ascii=False, indent=1, default=str))


if __name__ == "__main__":
    main()
