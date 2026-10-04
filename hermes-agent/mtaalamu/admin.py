"""MTAALAMU SMART — ADMIN (mmiliki wa mfumo).

MAMLAKA YA ADMIN:
  • Anaona FULL SYSTEM dashboard (OS nzima: probe, leseni, wallet, malipo, catalog)
  • Zana ZOTE ni BURE kwake (hakuna wallet/malipo)
  • Anabadilisha BEI ya zana yoyote (prices.json — huwaweka nguvu juu ya defaults)
  • Anaweka TIER kwa huduma yoyote (mf. disk.partition.create → GOLD+ tu)

UNGANISHO (usimamizi salama):
  • MTECH_ADMIN_KEY imeweka kwenye Environment → unlock inahitaji key hiyo
    (mfumo wa multi-user/watelii).
  • MTECH_ADMIN_KEY HAIPO → "owner-mode": mmiliki wa kompyuta anajifungua
    mara moja (mtaalamu admin unlock) — onyo linaonyeshwa.
  • mtaalamu admin lock → inazima admin.
"""
import hmac
import json
import os
import time
from pathlib import Path

from . import billing, clickpesa

STORE_DIR = billing.STORE_DIR
ADMIN_FILE = STORE_DIR / "admin.json"


def admin_key() -> str:
    return os.environ.get("MTECH_ADMIN_KEY", "").strip()


def is_admin() -> bool:
    try:
        return bool(json.loads(ADMIN_FILE.read_text()).get("admin"))
    except (OSError, json.JSONDecodeError):
        return False


def assert_admin() -> None:
    if not is_admin():
        raise PermissionError("INAHITAJI ADMIN — fungua: mtaalamu admin unlock")


def unlock(key: str | None = None) -> dict:
    """Fungua admin: key mode (MTECH_ADMIN_KEY) ama owner-mode (bila key env)."""
    STORE_DIR.mkdir(parents=True, exist_ok=True)
    env_key = admin_key()
    if env_key:
        if not key or not hmac.compare_digest(key.strip(), env_key):
            raise PermissionError("KEY YA ADMIN SI SAHIHI (MTECH_ADMIN_KEY)")
        mode = "key"
    else:
        mode = "owner"  # mmiliki wa kompyuta yake (onyo kwenye CLI)
    info = {"admin": True, "mode": mode, "unlocked_at": int(time.time())}
    ADMIN_FILE.write_text(json.dumps(info, indent=1))
    return info


def lock() -> dict:
    """Zima admin kwenye kompyuta hii."""
    STORE_DIR.mkdir(parents=True, exist_ok=True)
    info = {"admin": False, "locked_at": int(time.time())}
    ADMIN_FILE.write_text(json.dumps(info, indent=1))
    return info


# ------------------------------------------------------------------ bei (admin overrides)
def set_price(op_id: str, amount_tzs: int) -> dict:
    """Weka bei maalum ya zana (TZS). 0 = BURE kwa wote."""
    assert_admin()
    op = _op(op_id)
    if op is None:
        raise ValueError(f"op haijulikani: {op_id}")
    amount_tzs = int(amount_tzs)
    if amount_tzs < 0:
        raise ValueError("Bei lazima iwe >= 0")
    p = _prices()
    p[op_id] = amount_tzs
    (STORE_DIR / "prices.json").write_text(json.dumps(p, indent=1))
    return {"op": op_id, "new_price_tzs": amount_tzs,
            "price": billing.fmt_tzs(amount_tzs), "free": amount_tzs == 0}


def clear_price(op_id: str) -> dict:
    assert_admin()
    p = _prices()
    p.pop(op_id, None)
    (STORE_DIR / "prices.json").write_text(json.dumps(p, indent=1))
    return {"op": op_id, "cleared": True}


# ------------------------------------------------------------------ tier kwa huduma
TIERS_ALL = ("ANY",) + billing.TIERS


def set_tool_tier(op_id: str, tier: str) -> dict:
    """Weka TIER ndogo inayoruhusiwa kwa huduma (mf. GOLD → GOLD/PLATINUM/DIAMOND tu)."""
    assert_admin()
    op = _op(op_id)
    if op is None:
        raise ValueError(f"op haijulikani: {op_id}")
    tier = (tier or "ANY").upper()
    if tier not in TIERS_ALL:
        raise ValueError(f"Tier lazima iwe: {', '.join(TIERS_ALL)}")
    t = _tiers()
    if tier == "ANY":
        t.pop(op_id, None)
    else:
        t[op_id] = tier
    (STORE_DIR / "tool_tiers.json").write_text(json.dumps(t, indent=1))
    return {"op": op_id, "min_tier": tier, "note": f"wanatumia: {tier}+" if tier != "ANY" else "wote"}


# ------------------------------------------------------------------ dashboard (FULL SYSTEM)
def dashboard(probe: dict | None = None) -> dict:
    """FULL SYSTEM: OS nzima + leseni + wallet + malipo + catalog (bei/tier halisi)."""
    assert_admin()
    lic = billing.current_license() or {}
    tier = lic.get("tier")
    catalog = []
    from .catalog import CATALOG
    for o in CATALOG:
        price = billing.price_tzs(o.id, o.risk, tier)
        catalog.append({"id": o.id, "group": o.group, "sw": o.name_sw, "en": o.name_en,
                        "risk": o.risk, "offline": o.offline, "credits": o.credits,
                        "min_tier": billing.tool_tier(o.id) or "ANY",
                        "price_tzs": price, "price": billing.fmt_tzs(price),
                        "free": price == 0, "os": o.os})
    free_count = sum(1 for c in catalog if c["free"])
    return {
        "admin": info(),
        "os": {"family": probe.get("os") if probe else None,
               "release": probe.get("release") if probe else None,
               "machine": probe.get("machine") if probe else None,
               "node": probe.get("node") if probe else None,
               "snapshot": probe.get("snapshot", "")[:800] if probe else ""},
        "license": billing.usage_summary(),
        "wallet_tzs": billing.wallet_balance(),
        "gateway": clickpesa.status_mode(),
        "payments_total_tzs": billing.payments_total(),
        "usage_month_credits": billing.credits_used(),
        "catalog_ops": len(catalog),
        "free_ops": free_count,
        "priced_ops": len(catalog) - free_count,
        "tier_overrides": _tiers(),
        "price_overrides": {k: {"tzs": v, "price": billing.fmt_tzs(v)} for k, v in _prices().items()},
        "catalog": catalog,
    }


def info() -> dict:
    try:
        d = json.loads(ADMIN_FILE.read_text())
    except (OSError, json.JSONDecodeError):
        return {"admin": False}
    return d


# ------------------------------------------------------------------ internals
def _op(op_id: str):
    from .catalog import CATALOG_BY_ID
    return CATALOG_BY_ID.get(op_id)


def _prices() -> dict:
    f = STORE_DIR / "prices.json"
    try:
        return json.loads(f.read_text())
    except (OSError, json.JSONDecodeError):
        return {}


def _tiers() -> dict:
    f = STORE_DIR / "tool_tiers.json"
    try:
        return json.loads(f.read_text())
    except (OSError, json.JSONDecodeError):
        return {}
