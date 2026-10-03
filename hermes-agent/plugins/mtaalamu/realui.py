"""MTAALAMU real-UI routes — UI HALISI za source, chini ya Hermes (si iframe, si URL za nje).

Hermes inapitisha (reverse-proxy) UI halisi zinazotolewa na source iliyoko:
  - hermes-agent/home-assistant/  (Home Assistant core — UI: Lovelace)  → /ha/...
  - hermes-agent/openmrs/         (OpenMRS core — UI: o3 SPA)           → /openmrs/...
  - Hermes yenyewe                (hermes-agent/web/)                   → /hermes-ui/...

Mifumo hii inakimbia kwenye server ya mtumiaji (HASS_URL / OPENMRS_URL / HERMES_UI_URL).
Hermes inaipitisha UI yao HALISI byte-kwa-byte chini ya asili yake mwenyewe — session/cookies
zinapitishwa, `X-Frame-Options`/`frame-ancestors` za upstream hazipitishwi (tunajenga headers
mwenyewe). Matokeo: unafungua /ha/ ukurasa kamili → unapata Home Assistant HALISI (Lovelace
yake) chini ya Hermes. Hakuna iframe, hakuna kuiga UI, hakuna URL ya nje.

WebSocket (HA realtime): HTTP proxy hii haipitishi WS bado — HA UI inafanya kazi kwa vitendo
vya mwongozo; live-telemetry inakuja kupitia tools za Hermes (mtaalamu_ha) na dashboard.
"""

from __future__ import annotations

import base64
import os

from fastapi import APIRouter, Request
from fastapi.responses import JSONResponse, RedirectResponse, Response

realui_router = APIRouter()

_HOP = {"connection", "keep-alive", "proxy-authenticate", "proxy-authorization", "te",
        "trailers", "transfer-encoding", "upgrade", "host", "content-length",
        "accept-encoding", "cookie", "authorization"}

_ROUTES = {
    "ha": "/ha",
    "openmrs": "/openmrs",
    "hermes": "/hermes-ui",
}


def _base(kind: str) -> str | None:
    env = {"ha": "HASS_URL", "openmrs": "OPENMRS_URL", "hermes": "HERMES_UI_URL"}[kind]
    u = os.environ.get(env, "").strip().rstrip("/")
    return u or None


def _auth_headers(kind: str) -> dict:
    h: dict[str, str] = {}
    if kind == "ha":
        tok = os.environ.get("HASS_TOKEN", "").strip()
        if tok:
            h["Authorization"] = f"Bearer {tok}"
    elif kind == "openmrs":
        user, pwd = os.environ.get("OPENMRS_USER", "").strip(), os.environ.get("OPENMRS_PASS", "")
        if user and pwd:
            h["Authorization"] = "Basic " + base64.b64encode(f"{user}:{pwd}".encode()).decode()
    return h


def _pfx(kind: str) -> str:
    return _ROUTES[kind]


async def _proxy(kind: str, request: Request, path: str):  # noqa: C901
    import urllib.error as ue
    import urllib.request as ur

    base = _base(kind)
    if not base:
        env = {"ha": "HASS_URL", "openmrs": "OPENMRS_URL", "hermes": "HERMES_UI_URL"}[kind]
        src = {"ha": "hermes-agent/home-assistant/", "openmrs": "hermes-agent/openmrs/",
               "hermes": "hermes-agent/web/"}[kind]
        return JSONResponse({
            "ok": False,
            "error": f"UI halisi ya {kind} inahitaji server yako: weka {env} (Infisical).",
            "hint": f"Source halisi ipo: {src} — endesha kwenye server (ona hermes-agent/deploy/docker-compose.mtaalamu.yml).",
        }, status_code=503)

    url = f"{base}/{path}"
    if request.url.query:
        url += "?" + request.url.query
    data = await request.body() if request.method in ("POST", "PUT", "PATCH") else None

    fwd = {k: v for k, v in request.headers.items() if k.lower() not in _HOP}
    fwd.update(_auth_headers(kind))
    if data and "content-type" not in {k.lower() for k in fwd}:
        fwd["Content-Type"] = "application/json"

    req = ur.Request(url, data=data, headers=fwd, method=request.method)
    try:
        resp = ur.urlopen(req, timeout=30)
    except ue.HTTPError as e:  # pitisha na majibu ya upstream (login pages n.k.)
        resp = e
    except Exception as exc:
        return JSONResponse({"ok": False, "error": f"server ya {kind} haipatikani: {exc}",
                             "hint": f"angalia {base} + firewall"}, status_code=502)

    body = resp.read()
    ct = resp.headers.get("Content-Type", "application/octet-stream")
    status = getattr(resp, "status", getattr(resp, "code", 502))
    out: dict[str, str] = {}
    sc = resp.headers.get("Set-Cookie")
    if sc:
        out["Set-Cookie"] = sc  # session ya UI halisi inafanya kazi chini ya Hermes
    loc = resp.headers.get("Location")
    if loc:
        out["Location"] = (_pfx(kind) + "/" + loc[len(base):].lstrip("/")) if loc.startswith(base) else loc
    resp.close()

    if "text/html" in ct and kind != "hermes":
        # SPA za HA/OpenMRS zinaita assets zao kwa njia za root-absolute; zielekee proxy.
        txt = body.decode("utf-8", "replace")
        p = _pfx(kind)
        txt = (txt.replace('href="/', f'href="{p}/')
                  .replace('src="/', f'src="{p}/')
                  .replace('action="/', f'action="{p}/'))
        body = txt.encode()

    return Response(content=body, status_code=status, media_type=ct, headers=out)


@realui_router.get("/ha")
async def ha_root():
    return RedirectResponse("/ha/", status_code=307)


@realui_router.api_route("/ha/{path:path}", methods=["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS"])
async def ha_proxy(path: str, request: Request):
    return await _proxy("ha", request, path)


@realui_router.get("/openmrs")
async def openmrs_root():
    return RedirectResponse("/openmrs/", status_code=307)


@realui_router.api_route("/openmrs/{path:path}", methods=["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS"])
async def openmrs_proxy(path: str, request: Request):
    return await _proxy("openmrs", request, path)


@realui_router.get("/hermes-ui")
async def hermes_root():
    return RedirectResponse("/hermes-ui/", status_code=307)


@realui_router.api_route("/hermes-ui/{path:path}", methods=["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS"])
async def hermes_proxy(path: str, request: Request):
    return await _proxy("hermes", request, path)
