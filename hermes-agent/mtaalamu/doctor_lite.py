"""MTAALAMU — doctor (toleo la API/JSON, offline-safe)."""
import platform
import shutil
import sys

from . import billing, models


def run_checks() -> dict:
    checks = []

    def add(name, ok, detail=""):
        checks.append({"name": name, "ok": bool(ok), "detail": detail})

    v = sys.version_info
    add("python", v >= (3, 9), f"{v.major}.{v.minor}.{v.micro}")
    try:
        import PIL  # noqa: F401
        add("pillow", True)
    except ImportError:
        add("pillow", False, "pip install Pillow")
    try:
        from . import models as _m
        _m._ollama_chat(_m.DEFAULT_LOCAL, "ping", "ping")
        add("ollama-tinyllama", True)
    except Exception as e:  # noqa: BLE001
        add("ollama-tinyllama", False, str(e)[:120])
    try:
        from . import models as _m
        _m._ollama_chat(_m.DEFAULT_VISION, "ping", "ping")
        add("qwen-vl", True)
    except Exception as e:  # noqa: BLE001
        add("qwen-vl", False, str(e)[:120])
    try:
        from .skills_bridge import skills_summary
    except Exception:  # skills bridge ya mtech-os (si mtaalamu pkg)
        skills_summary = None
    if skills_summary:
        try:
            s = skills_summary()
            add("skills", s["total"] >= 100, f"{s['total']}")
        except Exception:  # noqa: BLE001
            add("skills", False, "skills.json haipatikani")
    else:
        add("skills", False, "bridge haipo")
    lic = billing.usage_summary()
    add("license", bool(lic.get("key")), f"{lic.get('plan','—')}/{lic.get('tier','—')}")
    add("os", True, f"{platform.system()} {platform.release()}")

    ok = all(c["ok"] for c in checks)
    return {"ok": ok, "os": platform.system(), "checks": checks}
