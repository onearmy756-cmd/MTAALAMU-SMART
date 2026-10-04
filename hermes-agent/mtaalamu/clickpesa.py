"""MTAALAMU SMART — ClickPesa PSP (malipo HALISI kabla ya zana).

JINSI CLICKPESA INAVYOFANYA KAZI (docs.clickpesa.com — HALISI):
  1) generate-token:   POST /third-parties/generate-token
     headers: client-id + api-key  →  {"token": "<JWT incl. 'Bearer '>"}
     Token inaisha ndani ya saa 1 — inahifadhiwa kwenye cache (55 min).
  2) initiate-ussd-push-request:  POST /third-parties/payments/initiate-ussd-push-request
     body: {amount (string), currency:"TZS", orderReference (alnum, max 20),
            phoneNumber (mf 255712345678 — bila +), checksum?}
     → {id, status: PROCESSING|SUCCESS|FAILED|SETTLED, channel: M-PESA|
        TIGO-PESA|AIRTEL-MONEY, collectedAmount, ...}
     Simu ya mteja inapokea USSD prompt ya PIN — malipo yanaidhinishwa hapo.
  3) status:  GET /third-parties/payments/{orderReference}  → list ya payments
  4) checksum: HMAC-SHA256 juu ya JSON (keys zimepangwa kwa alfabeti
     recursively, separators kamili) kwa CHECKSUM_KEY — inawekwa kwenye body.
  5) webhook: header "X-ClickPesa-Signature" = HMAC hiyo hiyo ya body —
     inathibitishwa kwa hmac.compare_digest (constant-time).

USALAMA (kulinda kutoibiwa):
  • Pesa ZINAINGIA kwenye wallet tu baada ya SUCCESS HALISI ya ClickPesa
    (webhook iliyo-verify AMA poll ya status iliyothibitishwa).
  • Hakuna njia ya kuongeza wallet bila uthibitisho wa ClickPesa.

KEYS (Settings → Environment):
  CLICKPESA_CLIENT_ID     (lazima)
  CLICKPESA_API_KEY       (lazima)
  CLICKPESA_CHECKSUM_KEY  (hiari — inawasha checksum + webhook verify)
  CLICKPESA_SANDBOX=1     (api-sandbox.clickpesa.com — kwa majaribio)
"""
import base64
import hashlib
import hmac
import json
import os
import time
import urllib.error
import urllib.request
from pathlib import Path

PROD_BASE = "https://api.clickpesa.com"
SANDBOX_BASE = "https://api-sandbox.clickpesa.com"
_TOKEN_CACHE: dict = {}


def _base_url() -> str:
    if os.environ.get("CLICKPESA_BASE_URL"):          # override kwa majaribio
        return os.environ["CLICKPESA_BASE_URL"].rstrip("/")
    return SANDBOX_BASE if os.environ.get("CLICKPESA_SANDBOX") == "1" else PROD_BASE


def client_id() -> str:
    return os.environ.get("CLICKPESA_CLIENT_ID", "").strip()


def api_key() -> str:
    return os.environ.get("CLICKPESA_API_KEY", "").strip()


def checksum_key() -> str:
    return os.environ.get("CLICKPESA_CHECKSUM_KEY", "").strip()


def available() -> bool:
    """TRUE ikiwa keys za ClickPesa zimeweka (endpoint halisi inaweza kuitwa)."""
    return bool(client_id() and api_key())


def status_mode() -> str:
    if not available():
        return "OFF (weka CLICKPESA_CLIENT_ID + CLICKPESA_API_KEY)"
    return "SANDBOX" if os.environ.get("CLICKPESA_SANDBOX") == "1" else "PRODUCTION"


# ------------------------------------------------------------------ crypto
def _canonical(obj) -> str:
    """Panga keys kwa alfabeti kila kiwango → compact JSON (kama ClickPesa)."""
    if isinstance(obj, list):
        return json.dumps([_canonical(i) for i in obj], separators=(",", ":"), ensure_ascii=False)
    if isinstance(obj, dict):
        return json.dumps({k: _canonical(obj[k]) for k in sorted(obj)},
                          separators=(",", ":"), ensure_ascii=False)
    return obj


def create_checksum(payload: dict) -> str:
    """HMAC-SHA256 hex ya payload (bila 'checksum' ndani yake)."""
    key = checksum_key()
    if not key:
        return ""
    return hmac.new(key.encode(), _canonical(payload).encode(), hashlib.sha256).hexdigest()


def verify_webhook(payload: dict, signature: str) -> bool:
    """Thibitisha webhook HALISI: X-ClickPesa-Signature == HMAC ya body."""
    if not signature or not checksum_key():
        return False
    return hmac.compare_digest(create_checksum(payload), signature)


# ------------------------------------------------------------------ phone / refs
def normalize_phone(phone: str) -> str:
    """255712345678 — anza na country code, hakuna + wala nafasi."""
    p = (phone or "").replace(" ", "").replace("+", "").replace("-", "")
    if p.startswith("0") and len(p) == 10:
        p = "255" + p[1:]
    if not (p.startswith("255") and len(p) == 12 and p[3:].isdigit()):
        raise ValueError("SIM SI HALALI — tumia mfano: 255712345678 (au 0712345678)")
    return p


def new_order_ref(prefix: str = "MST") -> str:
    """orderReference: alphanumeric TU, max 20 (mpaka wa mobile money)."""
    ts = base64.b32encode(int(time.time() * 1000).to_bytes(6, "big")).decode().rstrip("=")
    rnd = os.urandom(3).hex().upper()
    ref = f"{prefix}{ts}{rnd}"[:20]
    return "".join(c for c in ref if c.isalnum()) or "MST0001"


# ------------------------------------------------------------------ HTTP
def _request(method: str, path: str, body: dict | None = None, auth: bool = True,
             timeout: int = 25) -> dict | list:
    url = f"{_base_url()}{path}"
    headers = {"Content-Type": "application/json", "Accept": "application/json"}
    if auth:
        headers["Authorization"] = _token()
    data = json.dumps(body).encode() if body is not None else None
    req = urllib.request.Request(url, data=data, headers=headers, method=method)
    try:
        with urllib.request.urlopen(req, timeout=timeout) as resp:
            raw = resp.read().decode()
    except urllib.error.HTTPError as e:
        detail = ""
        try:
            detail = e.read().decode()[:300]
        except OSError:
            pass
        raise RuntimeError(f"ClickPesa {e.code} kwenye {path}: {detail}") from None
    except urllib.error.URLError as e:
        raise RuntimeError(f"ClickPesa HAIWEZI kufikiwa ({path}): {e.reason}") from None
    return json.loads(raw) if raw else {}


def _token() -> str:
    """Token ya JWT — inapewa mara moja, inatumika hadi 55 min."""
    tok = _TOKEN_CACHE.get("token")
    if tok and time.time() < _TOKEN_CACHE.get("exp", 0):
        return tok
    req = urllib.request.Request(
        f"{_base_url()}/third-parties/generate-token", data=b"{}",
        headers={"client-id": client_id(), "api-key": api_key(),
                 "Content-Type": "application/json"}, method="POST")
    try:
        with urllib.request.urlopen(req, timeout=25) as resp:
            out = json.loads(resp.read().decode() or "{}")
    except urllib.error.HTTPError as e:
        raise RuntimeError(f"ClickPesa token imekataliwa ({e.code}) — angalia CLICKPESA_CLIENT_ID/CLICKPESA_API_KEY") from None
    except urllib.error.URLError as e:
        raise RuntimeError(f"ClickPesa HAIWEZI kufikiwa: {e.reason}") from None
    token = (out.get("token") or "").strip()
    if not token:
        raise RuntimeError(f"ClickPesa token: jibu lisilotarajiwa: {str(out)[:200]}")
    _TOKEN_CACHE.update(token=token, exp=time.time() + 55 * 60)
    return token


# ------------------------------------------------------------------ collect
def initiate_ussd_push(amount_tzs: int, phone: str, order_ref: str | None = None) -> dict:
    """Tuma USSD prompt kwa simu ya mteja (M-Pesa/Tigo/Airtel/Halo...)."""
    if not available():
        raise RuntimeError("ClickPesa HAIJAWEKWA — weka CLICKPESA_CLIENT_ID + CLICKPESA_API_KEY kwenye Environment")
    if amount_tzs < 100:
        raise ValueError("Kiasi kidogo mno (chini: TZS 100)")
    phone = normalize_phone(phone)
    ref = (order_ref or new_order_ref())[:20]
    assert ref.isalnum(), "orderReference lazima iwe alphanumeric"
    payload = {"amount": str(int(amount_tzs)), "currency": "TZS",
               "orderReference": ref, "phoneNumber": phone}
    ck = create_checksum(payload)
    if ck:
        payload["checksum"] = ck
    out = _request("POST", "/third-parties/payments/initiate-ussd-push-request", payload)
    out.setdefault("orderReference", ref)
    return out


def payment_status(order_ref: str) -> list:
    """Hali ya malipo kwa reference: list ya payments (mtiririko wa ClickPesa)."""
    return _request("GET", f"/third-parties/payments/{order_ref}")


def collect(amount_tzs: int, phone: str, timeout_s: int = 90, poll_s: int = 3) -> dict:
    """Mzunguko KAMILI: initiate → poll mpaka SUCCESS/FAILED/timeout.

    Rejesha: {ok, status, ref, tx, channel, amount_tzs}
    ok=True tu kama malipo yamekamilika HALISI kwenye ClickPesa.
    """
    init = initiate_ussd_push(amount_tzs, phone)
    ref = init.get("orderReference") or ""
    deadline = time.time() + timeout_s
    last = {"status": init.get("status", "PROCESSING")}
    while time.time() < deadline:
        try:
            pays = payment_status(ref)
            entries = pays if isinstance(pays, list) else [pays]
            if entries:
                last = entries[-1]
                st = str(last.get("status", "")).upper()
                if st in ("SUCCESS", "SETTLED"):
                    return {"ok": True, "status": st, "ref": ref,
                            "tx": last.get("id") or init.get("id", ""),
                            "channel": last.get("channel") or init.get("channel", ""),
                            "amount_tzs": amount_tzs}
                if st == "FAILED":
                    return {"ok": False, "status": "FAILED", "ref": ref,
                            "channel": last.get("channel", ""), "amount_tzs": amount_tzs}
        except RuntimeError as e:
            last = {"status": f"poll-error: {str(e)[:120]}"}
        time.sleep(poll_s)
    return {"ok": False, "status": "TIMEOUT", "ref": ref, "channel": last.get("channel", ""),
            "amount_tzs": amount_tzs}
