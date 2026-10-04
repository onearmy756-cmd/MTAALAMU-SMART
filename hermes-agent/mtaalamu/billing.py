"""MTAALAMU SMART — billing, licenses, na metering ya zana.

MAZUNGUMZO:
  • Mteja anajisajili kwa EMAIL halisi (validated) + aina: INDIVIDUAL,
    CORPORATION (watu 10), ORGANIZATION (watu N — anachagua kikomo).
  • Makundi (kwa kila aina): BASIC, BRONZE, GOLD, PLATINUM, DIAMOND.
  • Baada ya kodi/confirm → LICENSE KEY inatengenezwa (deterministic,
    HMAC-signed — haitengenezwi upya kwa mabadiliko).
  • Kila zana/op ina credits; metering inahifadhiwa (audit).
"""
import hashlib
import hmac
import json
import os
import re
import time
from pathlib import Path

EMAIL_RE = re.compile(r"^[\w.+-]+@[\w-]+\.[\w.-]{2,}$")

PLANS = ("INDIVIDUAL", "CORPORATION", "ORGANIZATION")
SEATS = {"INDIVIDUAL": 1, "CORPORATION": 10, "ORGANIZATION": None}  # None = anachagua N
TIERS = ("BASIC", "BRONZE", "GOLD", "PLATINUM", "DIAMOND")

# credits za mwezi kwa tier (zana zote zina credits kutoka catalog)
TIER_CREDITS = {
    "BASIC": 50,
    "BRONZE": 200,
    "GOLD": 600,
    "PLATINUM": 1500,
    "DIAMOND": 5000,
}
TIER_SW = {
    "BASIC": "BASIC — kuanza (credits 50/mwezi)",
    "BRONZE": "BRONZE — nyumbani (200/mwezi)",
    "GOLD": "GOLD — biashara ndogo (600/mwezi)",
    "PLATINUM": "PLATINUM — kampuni (1500/mwezi)",
    "DIAMOND": "DIAMOND — kikamilifu (5000/mwezi)",
}

# ------------------------------------------------------------- MALIPO YA KILA ZANA (TZS)
# Kila zana ina bei kulingana na ukubwa wa kazi: TZS 5,000 → milioni.
# Risk ya op inaamua bei-msingi; tier ina punguzo (discount) kubwa zaidi juu.
BASE_PRICE_TZS = {"LOW": 5_000, "MEDIUM": 15_000, "HIGH": 50_000}
TIER_DISCOUNT = {"BASIC": 0.0, "BRONZE": 0.10, "GOLD": 0.20, "PLATINUM": 0.30, "DIAMOND": 0.40}

# Ops maalum zenye bei yake (kazi kubwa — hadi milioni)
PRICE_OVERRIDES_TZS = {
    "disk.partition.create": 100_000,     # tengeneza partition — kazi hatari kubwa
    "driver.update": 30_000,              # driver update/installation
    "office.repair": 25_000,              # Office repair
    "av.scan": 10_000,                    # antivirus scan
    "user.create": 20_000,
    "user.password.change": 20_000,
    "hw.scan": 7_000,                     # hardware diagnosis kamili
    "net.reset.stack": 15_000,
}


def price_tzs(op_id: str, risk: str, tier: str | None = None) -> int:
    """Bei ya zana moja (TZS) — overrides kwanza, vinginevyo kwa risk; tier inapunguza."""
    base = PRICE_OVERRIDES_TZS.get(op_id, BASE_PRICE_TZS.get(risk, 5_000))
    if tier:
        base = int(base * (1 - TIER_DISCOUNT.get(tier.upper(), 0.0)))
    return base


def fmt_tzs(n: int) -> str:
    return f"TZS {n:,}"

STORE_DIR = Path(os.environ.get("MTECH_STORE", Path.home() / ".mtaalamu"))
LICENSE_FILE = STORE_DIR / "license.json"
USAGE_FILE = STORE_DIR / "usage.json"

_SIGN_SECRET = os.environ.get("MTECH_LICENSE_SECRET", "mtech-mtaalamu-license-v1")


def _sign(payload: str) -> str:
    return hmac.new(_SIGN_SECRET.encode(), payload.encode(), hashlib.sha256).hexdigest()[:16].upper()


def gen_license_key(email: str, plan: str, tier: str, seats: int) -> str:
    """MST-XXXX-XXXX-SIG1-SIG2 — SIG ni HMAC ya body; inathibitishwa bila mtandao."""
    body = f"{email}|{plan}|{tier}|{seats}"
    sig = _sign(body)  # 16 hex uppercase
    raw = hashlib.sha256(body.encode()).hexdigest().upper()[:8]
    return f"MST-{raw[:4]}-{raw[4:8]}-{sig[:4]}-{sig[4:8]}"


def validate_license_key(key: str, email: str, plan: str, tier: str, seats: int) -> bool:
    """Rejesha HMAC kutoka body na kulinganisha na sehemu ya mwisho ya key."""
    if not key or not key.startswith("MST-"):
        return False
    body = f"{email}|{plan}|{tier}|{seats}"
    sig = _sign(body)
    # sehemu ya mwisho ya key: 'SIG1-SIG2' (9 chars na dash) → 'SIG1SIG2' (8) == sig[:8]
    return hmac.compare_digest(key[-9:].replace("-", ""), sig[:8])


def register(email: str, plan: str, tier: str, seats: int | None = None) -> dict:
    """Jisajili: email halisi + plan + tier → license key (+ hifadhi)."""
    email = (email or "").strip().lower()
    if not EMAIL_RE.match(email):
        raise ValueError("EMAIL SI HALALI — mfano: jina@gmail.com")
    plan = (plan or "").upper()
    if plan not in PLANS:
        raise ValueError(f"Aina lazima iwe mojawapo: {', '.join(PLANS)}")
    tier = (tier or "").upper()
    if tier not in TIERS:
        raise ValueError(f"Kundi lazima liwe: {', '.join(TIERS)}")
    if seats is None:
        seats = SEATS[plan] or 1
    seats = int(seats)
    if plan == "CORPORATION":
        seats = min(max(seats, 1), 10)  # CORPORATION = watu 10 tu
    key = gen_license_key(email, plan, tier, seats)
    lic = {
        "email": email, "plan": plan, "tier": tier, "seats": seats,
        "key": key, "issued_at": int(time.time()),
        "monthly_credits": TIER_CREDITS[tier] * (1 if plan == "INDIVIDUAL" else 2),
    }
    STORE_DIR.mkdir(parents=True, exist_ok=True)
    LICENSE_FILE.write_text(json.dumps(lic, indent=1))
    return lic


def current_license() -> dict | None:
    try:
        return json.loads(LICENSE_FILE.read_text())
    except (OSError, json.JSONDecodeError):
        return None


def is_active() -> bool:
    lic = current_license()
    if not lic:
        return False
    return validate_license_key(lic["key"], lic["email"], lic["plan"], lic["tier"], lic["seats"])


# ------------------------------------------------------------------ metering
def _month_key() -> str:
    return time.strftime("%Y-%m")


def credits_used() -> int:
    try:
        u = json.loads(USAGE_FILE.read_text())
    except (OSError, json.JSONDecodeError):
        return 0
    return int(u.get(_month_key(), 0))


def credits_left() -> int:
    lic = current_license() or {}
    return max(0, int(lic.get("monthly_credits", 0)) - credits_used())


def charge(op_id: str, credits: int) -> bool:
    """Kata credits za op; True ikiwa zipo — vinginevyo False (mteja apande tier)."""
    lic = current_license()
    if not lic or not is_active():
        return False
    left = credits_left()
    if credits > left:
        return False
    STORE_DIR.mkdir(parents=True, exist_ok=True)
    try:
        u = json.loads(USAGE_FILE.read_text())
    except (OSError, json.JSONDecodeError):
        u = {}
    k = _month_key()
    u[k] = int(u.get(k, 0)) + credits
    u.setdefault("log", []).append({"t": int(time.time()), "op": op_id, "credits": credits})
    USAGE_FILE.write_text(json.dumps(u, indent=1))
    return True


def usage_summary() -> dict:
    lic = current_license() or {}
    return {
        "email": lic.get("email"), "plan": lic.get("plan"), "tier": lic.get("tier"),
        "seats": lic.get("seats"), "key": lic.get("key"),
        "credits_used": credits_used(), "credits_left": credits_left(),
        "monthly_credits": lic.get("monthly_credits", 0),
        "total_paid_tzs": int(lic.get("total_paid_tzs", 0)),
    }


# ------------------------------------------------------------------ payments (kila zana)
PAYMENTS_FILE = STORE_DIR / "payments.jsonl"


def record_payment(op_id: str, amount_tzs: int, tier: str | None = None) -> dict:
    """Malipo ya zana moja (pay-per-use) — imehifadhiwa kwa audit."""
    STORE_DIR.mkdir(parents=True, exist_ok=True)
    rec = {"t": int(time.time()), "op": op_id, "amount_tzs": amount_tzs, "tier": tier}
    with PAYMENTS_FILE.open("a") as f:
        f.write(json.dumps(rec, ensure_ascii=False) + "\n")
    # jumla kwenye license
    lic = current_license() or {}
    lic["total_paid_tzs"] = int(lic.get("total_paid_tzs", 0)) + amount_tzs
    LICENSE_FILE.write_text(json.dumps(lic, indent=1))
    return rec


def payments_total() -> int:
    return sum(int(json.loads(l).get("amount_tzs", 0)) for l in PAYMENTS_FILE.read_text().splitlines() if l.strip()) if PAYMENTS_FILE.exists() else 0
