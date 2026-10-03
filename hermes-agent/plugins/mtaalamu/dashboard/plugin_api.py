"""MTAALAMU dashboard plugin API — mounted under /api/plugins/mtaalamu/.

Bridges the dashboard tab to the MTAALAMU Rust engine (subprocess, real data
only) and to the channel adapters. Enforces RBAC + HITL server-side: HITL
actions refuse to run without `approved=true` AND a role of admin/specialist.
"""

from __future__ import annotations

import json
import os
import shutil
import subprocess
import threading
import time
from pathlib import Path
from typing import Any

from fastapi import APIRouter, Request
from fastapi.responses import JSONResponse

router = APIRouter()

ROLES = ("admin", "specialist", "user")
HITL = {"deploy_start", "remediate_run", "agentic_approve", "scanner_run_once", "job_submit"}
_STATE_DIR = Path(__file__).resolve().parent.parent / "state"
_JOBS_FILE = _STATE_DIR / "remote_jobs.json"
_LOCK = threading.Lock()


def _repo_root() -> Path | None:
    env = os.environ.get("MTAALAMU_ROOT", "").strip()
    if env and (Path(env) / "data").exists():
        return Path(env)
    guess = Path(__file__).resolve().parents[3]  # dashboard/ → mtaalamu → plugins → hermes-agent → repo
    return guess if (guess / "data").exists() else None


def _engine_bin() -> str | None:
    root = _repo_root()
    if root:
        for rel in (
            "engine-rust/target/release/mtaalamu.exe",
            "engine-rust/target/release/mtaalamu",
            "engine-rust/target/debug/mtaalamu.exe",
            "engine-rust/target/debug/mtaalamu",
        ):
            p = root / rel
            if p.exists():
                return str(p)
    return shutil.which("mtaalamu")


def _engine(args: list[str], timeout: int = 90) -> dict[str, Any]:
    binp = _engine_bin()
    if not binp:
        return {"error": "mtaalamu engine haipo — jenga: cd engine-rust && cargo build --release"}
    root = _repo_root()
    try:
        out = subprocess.run([binp, *args], capture_output=True, text=True, timeout=timeout,
                             cwd=str(root) if root else None)
    except subprocess.TimeoutExpired:
        return {"error": f"engine timeout ({timeout}s)"}
    txt = (out.stdout or "").strip()
    if out.returncode != 0:
        return {"error": (out.stderr or txt or "engine failed").strip()[:800]}
    try:
        return json.loads(txt)
    except Exception:
        return {"raw": txt[:2000]}


def _rbac(body: dict, action: str) -> JSONResponse | None:
    role = str(body.get("role", "user"))
    if role not in ROLES:
        return JSONResponse({"error": f"role '{role}' haipatikani"}, status_code=403)
    if action in HITL and not body.get("approved"):
        return JSONResponse(
            {"error": "HITL: idhini ya mteja inahitajika (approved=true)", "hitl": action},
            status_code=403,
        )
    if action in HITL and role == "user":
        return JSONResponse({"error": "HITL: 'user' hawezi kuidhinisha"}, status_code=403)
    return None


async def _body(request: Request) -> dict:
    try:
        v = await request.json()
        return v if isinstance(v, dict) else {}
    except Exception:
        return {}


# ---------------------------------------------------------------- health/solve

@router.get("/health")
async def health():
    return {
        "plugin": "mtaalamu",
        "engine": bool(_engine_bin()),
        "engine_path": _engine_bin(),
        "provider_hint": "ollama-cloud (OLLAMA_API_KEY) kwa LLM; hesabu ni Rust",
    }


@router.post("/solve")
async def solve(request: Request):
    b = await _body(request)
    if (e := _rbac(b, "solve")):
        return e
    msg = str(b.get("msg", "")).strip()
    if not msg:
        return JSONResponse({"error": "msg inahitajika"}, status_code=400)
    args = ["solve", "--msg", msg] + (["--approve"] if b.get("approve") else [])
    return await _run_async(_engine, args, 60)


@router.post("/agentic")
async def agentic(request: Request):
    b = await _body(request)
    if b.get("approve"):
        if (e := _rbac(b, "agentic_approve")):
            return e
    msg = str(b.get("msg", "")).strip()
    if not msg:
        return JSONResponse({"error": "msg inahitajika"}, status_code=400)
    args = ["agent", "--msg", msg] + (["--approve"] if b.get("approve") else [])
    return await _run_async(_engine, args, 150)


async def _run_async(fn, *a):
    import asyncio
    return await asyncio.get_event_loop().run_in_executor(None, lambda: fn(*a))


# ---------------------------------------------------------------- scan/remediate

@router.get("/scan")
async def scan(top: int = 10, deep: bool = False):
    return await _run_async(_engine, ["deep" if deep else "sysprobe", "--top", str(max(1, min(top, 25)))], 90)


@router.get("/remediate")
async def remediate_catalog():
    return await _run_async(_engine, ["remediate"], 30)


@router.post("/remediate")
async def remediate_run(request: Request):
    b = await _body(request)
    if (e := _rbac(b, "remediate_run")):
        return e
    action = str(b.get("action", "")).strip()
    if not action:
        return JSONResponse({"error": "action inahitajika"}, status_code=400)
    return await _run_async(_engine, ["remediate", "--action", action, "--approve"], 120)


# ---------------------------------------------------------------- deploy

@router.get("/deploy/{sub}")
async def deploy_get(sub: str):
    if sub not in ("hosts", "discover", "jobs", "images", "summary", "cloud"):
        return JSONResponse({"error": f"sub '{sub}' haipatikani"}, status_code=400)
    return await _run_async(_engine, ["deploy", sub], 60)


@router.post("/deploy/start")
async def deploy_start(request: Request):
    b = await _body(request)
    if (e := _rbac(b, "deploy_start")):
        return e
    macs = str(b.get("macs", "")).strip()
    if not macs:
        return JSONResponse({"error": "macs zinahitajika (kutoka /deploy/hosts)"}, status_code=400)
    args = ["deploy", "start", "--macs", macs, "--os", str(b.get("os", "auto")), "--need", str(b.get("need", "office"))]
    return await _run_async(_engine, args, 120)


@router.post("/deploy/approve/{job_id}")
async def deploy_approve(job_id: str, request: Request):
    b = await _body(request)
    if (e := _rbac(b, "deploy_start")):
        return e
    return await _run_async(_engine, ["deploy", "approve", "--id", job_id], 60)


@router.post("/deploy/cancel/{job_id}")
async def deploy_cancel(job_id: str, request: Request):
    b = await _body(request)
    if (e := _rbac(b, "deploy_start")):
        return e
    return await _run_async(_engine, ["deploy", "cancel", "--id", job_id], 60)


# ---------------------------------------------------------------- channels

@router.post("/channels/whatsapp")
async def ch_whatsapp(request: Request):
    b = await _body(request)
    to, msg = str(b.get("to", "")).strip(), str(b.get("msg", "")).strip()
    if not (to and msg):
        return JSONResponse({"error": "to na msg zinahitajika"}, status_code=400)
    import urllib.request

    gw = os.environ.get("HERMES_GATEWAY_URL", "http://127.0.0.1:8088")
    try:
        req = urllib.request.Request(
            f"{gw}/api/send",
            data=json.dumps({"channel": "whatsapp", "to": to, "message": msg}).encode(),
            headers={"Content-Type": "application/json"}, method="POST",
        )
        with urllib.request.urlopen(req, timeout=15) as r:
            return {"ok": True, "channel": "whatsapp", "status": r.status}
    except Exception as exc:
        return JSONResponse({"ok": False, "error": str(exc), "hint": "sanidi channels.whatsapp kwenye cli-config.yaml na anzisha gateway"}, status_code=502)


@router.post("/channels/sms")
async def ch_sms(request: Request):
    b = await _body(request)
    to, msg = str(b.get("to", "")).strip(), str(b.get("msg", "")).strip()
    url = os.environ.get("FUNDI_SMS_URL", "").strip()
    if not (to and msg):
        return JSONResponse({"error": "to na msg zinahitajika"}, status_code=400)
    if not url:
        return JSONResponse({"ok": False, "error": "FUNDI_SMS_URL haijawekwa (Infisical)"}, status_code=503)
    import urllib.request

    try:
        req = urllib.request.Request(url, data=json.dumps({"to": to, "message": msg}).encode(),
                                     headers={"Content-Type": "application/json"}, method="POST")
        with urllib.request.urlopen(req, timeout=20) as r:
            return {"ok": True, "channel": "sms", "status": r.status}
    except Exception as exc:
        return JSONResponse({"ok": False, "error": str(exc)}, status_code=502)


@router.post("/channels/email")
async def ch_email(request: Request):
    b = await _body(request)
    import smtplib
    from email.mime.text import MIMEText

    to, subject, body = str(b.get("to", "")).strip(), str(b.get("subject", "MTAALAMU")).strip(), str(b.get("msg", "")).strip()
    host, user, pwd = os.environ.get("FUNDI_SMTP_HOST", "").strip(), os.environ.get("FUNDI_SMTP_USER", "").strip(), os.environ.get("FUNDI_SMTP_PASS", "")
    if not (to and host and user and pwd):
        return JSONResponse({"ok": False, "error": "FUNDI_SMTP_HOST/USER/PASS hazijawekwa (Infisical)"}, status_code=503)
    try:
        s = smtplib.SMTP(host, int(os.environ.get("FUNDI_SMTP_PORT", "587")), timeout=20)
        s.starttls()
        s.login(user, pwd)
        m = MIMEText(body, _charset="utf-8")
        m["Subject"], m["From"], m["To"] = subject, user, to
        s.send_message(m)
        s.quit()
        return {"ok": True, "channel": "email", "to": to}
    except Exception as exc:
        return JSONResponse({"ok": False, "error": str(exc)}, status_code=502)


# ---------------------------------------------------------------- maintenance/scanner

@router.get("/maintenance")
async def maintenance(horizon: int = 30, top: int = 8):
    probe = await _run_async(_engine, ["deep", "--top", str(max(1, min(top, 20)))], 90)
    if isinstance(probe, dict) and probe.get("error"):
        return probe
    return {
        "kind": "predictive_maintenance",
        "note_sw": "Ubashiri kutoka probe halisi + rules (kanuni R-1).",
        "horizon_days": horizon,
        "probe": probe,
    }


@router.post("/scanner")
async def scanner(request: Request):
    b = await _body(request)
    mode = str(b.get("mode", "status"))
    if mode == "run_once":
        if (e := _rbac(b, "scanner_run_once")):
            return e
        args = ["av-auto"] + (["--approve"] if b.get("approved") else [])
        return await _run_async(_engine, args, 180)
    return await _run_async(_engine, ["av-sessions"], 30)


# ---------------------------------------------------------------- home assistant (real UI data, our theme)

def _ha_cfg():
    import urllib.request as _u
    base = os.environ.get("HASS_URL", "http://127.0.0.1:8123").rstrip("/")
    tok = os.environ.get("HASS_TOKEN", "").strip()
    return base, tok, _u


@router.get("/ha/state")
async def ha_state():
    base, tok, u = _ha_cfg()
    if not tok:
        return JSONResponse({"ok": False, "error": "HASS_TOKEN haijawekwa (Infisical)", "hint": "HA → Profile → Security → Long-Lived Access Token"}, status_code=503)
    try:
        req = u.Request(f"{base}/api/states", headers={"Authorization": f"Bearer {tok}"})
        with u.urlopen(req, timeout=8) as r:
            entities = json.loads(r.read().decode())
        keep = [e for e in entities if e["entity_id"].startswith(("light.", "switch.", "sensor.", "climate.", "binary_sensor."))]
        out = [{"id": e["entity_id"], "state": e["state"],
                "name": e["attributes"].get("friendly_name", e["entity_id"]),
                "unit": e["attributes"].get("unit_of_measurement", ""),
                "bri": e["attributes"].get("brightness")} for e in keep]
        return {"ok": True, "base": base, "count": len(out), "entities": out[:100]}
    except Exception as exc:
        return JSONResponse({"ok": False, "error": f"Home Assistant :8123 haipatikani: {exc}", "hint": "anzisha hass (Python) kisha weka HASS_TOKEN Infisical"}, status_code=502)


@router.post("/ha/service")
async def ha_service(request: Request):
    b = await _body(request)
    if (e := _rbac(b, "remediate_run")):
        return e  # HITL: kuwasha/kuzima vifaa ni hatua inayobadilisha nyumbani
    base, tok, u = _ha_cfg()
    domain, action = str(b.get("domain", "light")), str(b.get("action", "turn_on"))
    entity = str(b.get("entity", "")).strip()
    if not entity or action not in ("turn_on", "turn_off", "toggle"):
        return JSONResponse({"error": "entity + action (turn_on|turn_off|toggle) zinahitajika"}, status_code=400)
    if not tok:
        return JSONResponse({"ok": False, "error": "HASS_TOKEN haijawekwa (Infisical)"}, status_code=503)
    try:
        req = u.Request(f"{base}/api/services/{domain}/{action}",
                        data=json.dumps({"entity_id": entity}).encode(),
                        headers={"Authorization": f"Bearer {tok}", "Content-Type": "application/json"}, method="POST")
        with u.urlopen(req, timeout=8) as r:
            return {"ok": True, "entity": entity, "action": action, "status": r.status}
    except Exception as exc:
        return JSONResponse({"ok": False, "error": str(exc)}, status_code=502)


# ---------------------------------------------------------------- openmrs (real UI data, our theme)

def _omrs_cfg():
    import base64
    import urllib.request as _u
    base = os.environ.get("OPENMRS_URL", "http://127.0.0.1:8080/openmrs").rstrip("/")
    user = os.environ.get("OPENMRS_USER", "").strip()
    pwd = os.environ.get("OPENMRS_PASS", "")
    hdr = {"Content-Type": "application/json"}
    if user and pwd:
        hdr["Authorization"] = "Basic " + base64.b64encode(f"{user}:{pwd}".encode()).decode()
    return base, hdr, _u


@router.get("/openmrs/patients")
async def omrs_patients(q: str = "", limit: int = 10):
    base, hdr, u = _omrs_cfg()
    if "Authorization" not in hdr:
        return JSONResponse({"ok": False, "error": "OPENMRS_USER/PASS hazijawekwa (Infisical)", "hint": "demo ya mtandaopo: o3.openmrs.org (admin/Admin123)"}, status_code=503)
    try:
        req = u.Request(f"{base}/ws/rest/v1/patient?q={q}&limit={max(1, min(limit, 25))}&v=default", headers=hdr)
        with u.urlopen(req, timeout=8) as r:
            data = json.loads(r.read().decode())
        out = [{"uuid": p["uuid"], "name": p["person"].get("display", ""),
                "age": p["person"].get("age"), "gender": p["person"].get("gender"),
                "ids": [i.get("display") for i in p.get("identifiers", [])]} for p in data.get("results", [])]
        return {"ok": True, "base": base, "total": data.get("totalCount", len(out)), "patients": out}
    except Exception as exc:
        return JSONResponse({"ok": False, "error": f"OpenMRS :8080 haipatikani: {exc}", "hint": "anzisha openmrs (Java+Tomcat) au tumia demo o3.openmrs.org"}, status_code=502)


@router.post("/openmrs/patient")
async def omrs_create_patient(request: Request):
    b = await _body(request)
    if (e := _rbac(b, "remediate_run")):
        return e  # HITL: kuunda mgonjwa kwenye EHR ni hatua ya kliniki
    base, hdr, u = _omrs_cfg()
    names = b.get("names") or {}
    person = b.get("person") or {}
    if not (names.get("given") and names.get("family") and person.get("gender")):
        return JSONResponse({"error": "names.given, names.family na person.gender zinahitajika"}, status_code=400)
    if "Authorization" not in hdr:
        return JSONResponse({"ok": False, "error": "OPENMRS_USER/PASS hazijawekwa (Infisical)"}, status_code=503)
    payload = {"names": [{"givenName": names["given"], "familyName": names["family"]}],
               "person": {"gender": person["gender"], **({"age": person["age"]} if person.get("age") else {}),
                          **({"birthdate": person["birthdate"]} if person.get("birthdate") else {})}}
    try:
        req = u.Request(f"{base}/ws/rest/v1/patient", data=json.dumps(payload).encode(), headers=hdr, method="POST")
        with u.urlopen(req, timeout=10) as r:
            created = json.loads(r.read().decode())
        return {"ok": True, "uuid": created.get("uuid"), "display": created.get("display")}
    except Exception as exc:
        return JSONResponse({"ok": False, "error": str(exc)}, status_code=502)


# ---------------------------------------------------------------- remote jobs

def _load_jobs() -> list:
    try:
        return json.loads(_JOBS_FILE.read_text(encoding="utf-8"))
    except Exception:
        return []


def _save_jobs(jobs: list) -> None:
    _STATE_DIR.mkdir(parents=True, exist_ok=True)
    _JOBS_FILE.write_text(json.dumps(jobs[-500:], ensure_ascii=False, indent=1), encoding="utf-8")


@router.get("/jobs")
async def jobs_list():
    return {"jobs": _load_jobs()}


@router.post("/jobs")
async def jobs_create(request: Request):
    b = await _body(request)
    if (e := _rbac(b, "job_submit")):
        return e
    kind = str(b.get("kind", "solve"))
    msg = str(b.get("msg", "")).strip()
    job = {
        "id": f"RJ-{int(time.time()*1000)}",
        "kind": kind,
        "msg": msg,
        "role": str(b.get("role", "user")),
        "status": "queued",
        "created_at": time.strftime("%Y-%m-%dT%H:%M:%S"),
        "result": None,
    }
    with _LOCK:
        jobs = _load_jobs()
        jobs.append(job)
        _save_jobs(jobs)

    def _run() -> None:
        job["status"] = "running"
        try:
            if kind == "scan":
                job["result"] = _engine(["sysprobe", "--top", "10"])
            elif kind == "deploy_summary":
                job["result"] = _engine(["deploy", "summary"])
            elif kind == "agentic" and msg:
                job["result"] = _engine(["agent", "--msg", msg])
            elif msg:
                job["result"] = _engine(["solve", "--msg", msg])
            else:
                job["result"] = {"error": "msg inahitajika"}
            job["status"] = "done" if not (isinstance(job["result"], dict) and job["result"].get("error")) else "error"
        except Exception as exc:
            job["status"], job["result"] = "error", {"error": str(exc)}
        with _LOCK:
            allj = _load_jobs()
            for i, j in enumerate(allj):
                if j.get("id") == job["id"]:
                    allj[i] = job
            _save_jobs(allj)

    threading.Thread(target=_run, name=f"mtaalamu-job-{job['id']}", daemon=True).start()
    return job
