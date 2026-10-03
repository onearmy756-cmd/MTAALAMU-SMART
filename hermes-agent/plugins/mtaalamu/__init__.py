"""MTAALAMU SMART — Hermes Agent plugin.

Bridges the MTAALAMU SMART Rust engine (engine-rust, `mtaalamu <cmd>`) and the
FUNDI services into Hermes as first-class agent tools, per R-1: LLM never
calculates — the engine does all math/diagnostics; the LLM explains.

Tools (registered via ctx.register_tool in register()):
  mtaalamu_solve(msg, approve)      — grounded solve (rules + knowledge)
  mtaalamu_agentic(msg, approve)    — full agentic run (vision→solve→report)
  mtaalamu_scan(top)                — live system probe (sysprobe/deep)
  mtaalamu_remediate(action, ok)    — remediation catalog / gated execution
  mtaalamu_deploy(sub, ...)         — Fundi Deploy: discover/start/approve/cancel/jobs
  mtaalamu_send_whatsapp(sms...)    — channel adapters (real HTTP, no fake sends)
  mtaalamu_send_sms(to, msg)
  mtaalamu_send_email(to, subject, body)
  mtaalamu_maintenance(horizon)     — predictive maintenance forecast
  mtaalamu_scanner(interval)        — background scanner status/start (av-watch)
  mtaalamu_job(kind, msg)           — remote job submission (background work)

RBAC: every tool takes `role` in {admin, specialist, user}; HITL-gated actions
(deploy start, remediate execute, agentic approve) require admin or specialist
and the user's explicit approval flag — Hermes relays the customer's consent.
"""

from __future__ import annotations

import json
import os
import shutil
import subprocess
from pathlib import Path
from typing import Any, Callable, Optional

log = __import__("logging").getLogger(__name__)

ROLES = ("admin", "specialist", "user")
# Actions the customer must approve before execution (HITL).
HITL_ACTIONS = {"deploy_start", "remediate_run", "agentic_approve", "job_submit"}


def _repo_root() -> Optional[Path]:
    """MTAALAMU repo root (has data/) — env override first, then relative guess."""
    env = os.environ.get("MTAALAMU_ROOT", "").strip()
    if env and (Path(env) / "data").exists():
        return Path(env)
    # plugins/mtaalamu/__init__.py → parents[2] = repo root in-tree
    guess = Path(__file__).resolve().parents[2]
    return guess if (guess / "data").exists() else None


def _engine_bin() -> Optional[str]:
    """Locate the mtaalamu engine binary (release first, then debug)."""
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


def _engine(args: list[str], timeout: int = 60) -> dict[str, Any]:
    """Run the Rust engine and return parsed JSON. Never fake data: errors propagate."""
    binp = _engine_bin()
    if not binp:
        return {"error": "mtaalamu engine haipo — jenga kwa: cd engine-rust && cargo build --release"}
    env = {**os.environ}
    root = _repo_root()
    try:
        out = subprocess.run(
            [binp, *args], capture_output=True, text=True, timeout=timeout, env=env,
            cwd=str(root) if root else None,
        )
    except subprocess.TimeoutExpired:
        return {"error": f"engine timeout after {timeout}s", "args": args}
    txt = (out.stdout or "").strip()
    if out.returncode != 0:
        return {"error": (out.stderr or txt or "engine failed").strip()[:800], "args": args}
    try:
        return json.loads(txt)
    except Exception:
        return {"raw": txt[:2000]}


def _check_role(role: str, action: str, approved: bool) -> Optional[str]:
    if role not in ROLES:
        return f"role '{role}' haipatikani (admin|specialist|user)"
    if action in HITL_ACTIONS and not approved:
        return (
            f"HITL: '{action}' inahitaji idhini ya mteja — uliza mteja kwanza, "
            "kisha ita tena na approved=true (kanuni ya HERMES: hakuna actuation bila ruhusa)."
        )
    if action in HITL_ACTIONS and role == "user":
        return "HITL: 'user' hawezi kuidhinisha — mteja mwenyewe (admin/specialist) anapaswa kuidhinisha."
    return None


# ---------------------------------------------------------------------------
# Tool implementations
# ---------------------------------------------------------------------------

def _t_solve(args: dict) -> dict:
    role = args.get("role", "user")
    err = _check_role(role, "solve", True)
    if err:
        return {"error": err}
    msg = args.get("msg", "").strip()
    if not msg:
        return {"error": "msg inahitajika"}
    approve = bool(args.get("approve", False))
    return _engine(["solve", "--msg", msg] + (["--approve"] if approve else []))


def _t_agentic(args: dict) -> dict:
    approve = bool(args.get("approve", False))
    err = _check_role(args.get("role", "user"), "agentic_approve" if approve else "solve", True)
    if err:
        return {"error": err}
    msg = args.get("msg", "").strip()
    if not msg:
        return {"error": "msg inahitajika"}
    return _engine(["agent", "--msg", msg] + (["--approve"] if approve else []), timeout=120)


def _t_scan(args: dict) -> dict:
    top = int(args.get("top", 10))
    deep = bool(args.get("deep", False))
    return _engine(["deep" if deep else "sysprobe", "--top", str(top)], timeout=90)


def _t_remediate(args: dict) -> dict:
    action = args.get("action", "").strip()
    if not action:
        return _engine(["remediate"])  # catalog
    approved = bool(args.get("approved", False))
    err = _check_role(args.get("role", "user"), "remediate_run", approved)
    if err:
        return {"error": err, "hint": "pata idhini ya mteja kwanza (HITL)"}
    return _engine(["remediate", "--action", action, "--approve"])


def _t_deploy(args: dict) -> dict:
    sub = args.get("sub", "summary").strip()
    role = args.get("role", "user")
    if sub == "start":
        macs = args.get("macs", "").strip()
        if not macs:
            return {"error": "macs inahitajika (aa:bb:cc:dd:ee:01,aa:bb:...)"}
        err = _check_role(role, "deploy_start", bool(args.get("approved", False)))
        if err:
            return {"error": err}
        a = ["deploy", "start", "--macs", macs]
        if args.get("os"):
            a += ["--os", str(args["os"])]
        if args.get("need"):
            a += ["--need", str(args["need"])]
        return _engine(a, timeout=120)
    if sub == "approve":
        return _engine(["deploy", "approve", "--id", args.get("id", "")], timeout=60)
    if sub == "cancel":
        return _engine(["deploy", "cancel", "--id", args.get("id", "")], timeout=60)
    if sub in ("hosts", "discover", "jobs", "images", "summary", "cloud"):
        return _engine(["deploy", sub], timeout=60)
    return {"error": f"deploy sub '{sub}' haipo (hosts|jobs|images|summary|cloud|os|start|approve|cancel)"}


def _t_maintenance(args: dict) -> dict:
    """Predictive maintenance: engine probes + rules forecast wear/failure risk."""
    top = int(args.get("top", 8))
    probe = _engine(["deep", "--top", str(top)], timeout=90)
    if probe.get("error"):
        return probe
    return {
        "kind": "predictive_maintenance",
        "note_sw": "Ubashiri wa matengenezo kutoka probe halisi + rules (LLM hauhesabu).",
        "probe": probe,
        "horizon_days": int(args.get("horizon", 30)),
    }


def _t_scanner(args: dict) -> dict:
    """Background scanner: status or one supervised auto-work cycle (av-auto)."""
    mode = args.get("mode", "status")
    if mode == "run_once":
        approved = bool(args.get("approved", False))
        err = _check_role(args.get("role", "user"), "agentic_approve", approved)
        if err:
            return {"error": err}
        rep = _engine(["av-auto"] + (["--approve"] if approved else []), timeout=180)
        return rep
    return _engine(["av-sessions"], timeout=30)


def _t_job(args: dict) -> dict:
    """Remote job: kind in {solve, scan, deploy_summary, agentic} — runs now, returns result."""
    kind = args.get("kind", "solve")
    err = _check_role(args.get("role", "user"), "job_submit", True)
    if err:
        return {"error": err}
    if kind == "scan":
        return _engine(["sysprobe", "--top", "10"], timeout=90)
    if kind == "deploy_summary":
        return _engine(["deploy", "summary"], timeout=60)
    if kind == "agentic":
        return _t_agentic(args)
    return _t_solve(args)


def _t_send_whatsapp(args: dict) -> dict:
    """Send via WhatsApp gateway channel. Real send requires HERMES WhatsApp channel
    configured (cli-config.yaml channels); otherwise returns the exact setup error."""
    to, msg = args.get("to", "").strip(), args.get("msg", "").strip()
    if not to or not msg:
        return {"error": "to na msg zinahitajika"}
    gw = os.environ.get("HERMES_GATEWAY_URL", "http://127.0.0.1:8088")
    try:
        import urllib.request

        req = urllib.request.Request(
            f"{gw}/api/send",
            data=json.dumps({"channel": "whatsapp", "to": to, "message": msg}).encode(),
            headers={"Content-Type": "application/json"},
            method="POST",
        )
        with urllib.request.urlopen(req, timeout=15) as r:
            return {"ok": True, "channel": "whatsapp", "status": r.status, "to": to}
    except Exception as exc:
        return {
            "ok": False,
            "error": f"WhatsApp gateway haipatikani ({exc})",
            "hint": "Sanidi WhatsApp channel kwenye Hermes (cli-config.yaml: channels.whatsapp) kisha anzisha gateway :8088",
        }


def _t_send_sms(args: dict) -> dict:
    """SMS via SMS provider env (FUNDI_SMS_URL) — real HTTP POST, no fake success."""
    to, msg = args.get("to", "").strip(), args.get("msg", "").strip()
    if not to or not msg:
        return {"error": "to na msg zinahitajika"}
    url = os.environ.get("FUNDI_SMS_URL", "").strip()
    if not url:
        return {
            "ok": False,
            "error": "FUNDI_SMS_URL haijawekwa (Infisical)",
            "hint": "Weka FUNDI_SMS_URL (HTTP SMS provider) ndani ya Infisical kisha anzisha upya.",
        }
    try:
        import urllib.request

        req = urllib.request.Request(
            url,
            data=json.dumps({"to": to, "message": msg}).encode(),
            headers={"Content-Type": "application/json"},
            method="POST",
        )
        with urllib.request.urlopen(req, timeout=20) as r:
            return {"ok": True, "channel": "sms", "status": r.status, "to": to}
    except Exception as exc:
        return {"ok": False, "error": str(exc)}


def _t_send_email(args: dict) -> dict:
    """Email via SMTP env (FUNDI_SMTP_HOST/USER/PASS) — real SMTP, TLS, no fake success."""
    import smtplib
    from email.mime.text import MIMEText

    to = args.get("to", "").strip()
    subject = args.get("subject", "MTAALAMU").strip()
    body = args.get("msg", "").strip()
    host = os.environ.get("FUNDI_SMTP_HOST", "").strip()
    user = os.environ.get("FUNDI_SMTP_USER", "").strip()
    pwd = os.environ.get("FUNDI_SMTP_PASS", "")
    if not (to and host and user and pwd):
        return {
            "ok": False,
            "error": "FUNDI_SMTP_HOST/USER/PASS hazijawekwa (Infisical)",
            "hint": "Weka SMTP credentials ndani ya Infisical kisha anzisha upya.",
        }
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
        return {"ok": False, "error": str(exc)}


# ---------------------------------------------------------------------------
# Tool schemas (JSON-schema subset Hermes accepts via ctx.register_tool)
# ---------------------------------------------------------------------------

_ROLE_PROP = {
    "role": {"type": "string", "enum": list(ROLES), "description": "RBAC role: admin|specialist|user"}
}


def _register_tools(ctx) -> None:
    tools: list[tuple[str, str, dict, Callable, str]] = [
        (
            "mtaalamu_solve",
            "Run the MTAALAMU grounded solver on a customer problem (rules + knowledge; math is Rust, not LLM).",
            {"type": "object", "properties": {"msg": {"type": "string"}, **_ROLE_PROP, "approve": {"type": "boolean"}}, "required": ["msg"]},
            _t_solve, "🛠️",
        ),
        (
            "mtaalamu_agentic",
            "Full agentic run: scan → diagnose → plan → HITL → solve → report (Kiswahili narration).",
            {"type": "object", "properties": {"msg": {"type": "string"}, **_ROLE_PROP, "approve": {"type": "boolean"}}, "required": ["msg"]},
            _t_agentic, "👁️",
        ),
        (
            "mtaalamu_scan",
            "Live system probe (CPU/RAM/disk/services) for diagnostics and predictive maintenance.",
            {"type": "object", "properties": {"top": {"type": "integer"}, "deep": {"type": "boolean"}}},
            _t_scan, "📡",
        ),
        (
            "mtaalamu_remediate",
            "List or (HITL-gated) run system remediation actions.",
            {"type": "object", "properties": {"action": {"type": "string"}, "approved": {"type": "boolean"}, **_ROLE_PROP}},
            _t_remediate, "🧰",
        ),
        (
            "mtaalamu_deploy",
            "Fundi Deploy: discover hosts, start OS deployment (HITL), approve/cancel jobs, cloud/summary.",
            {"type": "object", "properties": {"sub": {"type": "string"}, "macs": {"type": "string"}, "id": {"type": "string"}, "os": {"type": "string"}, "need": {"type": "string"}, "approved": {"type": "boolean"}, **_ROLE_PROP}},
            _t_deploy, "💻",
        ),
        (
            "mtaalamu_send_whatsapp",
            "Send a WhatsApp message via the Hermes gateway WhatsApp channel.",
            {"type": "object", "properties": {"to": {"type": "string"}, "msg": {"type": "string"}}, "required": ["to", "msg"]},
            _t_send_whatsapp, "💬",
        ),
        (
            "mtaalamu_send_sms",
            "Send an SMS via the configured FUNDI_SMS_URL provider (real HTTP).",
            {"type": "object", "properties": {"to": {"type": "string"}, "msg": {"type": "string"}}, "required": ["to", "msg"]},
            _t_send_sms, "📱",
        ),
        (
            "mtaalamu_send_email",
            "Send an email via configured SMTP (FUNDI_SMTP_* env, real TLS send).",
            {"type": "object", "properties": {"to": {"type": "string"}, "subject": {"type": "string"}, "msg": {"type": "string"}}, "required": ["to", "subject", "msg"]},
            _t_send_email, "✉️",
        ),
        (
            "mtaalamu_maintenance",
            "Predictive maintenance forecast from real probes + rules (wear/failure risk).",
            {"type": "object", "properties": {"top": {"type": "integer"}, "horizon": {"type": "integer"}}},
            _t_maintenance, "🔮",
        ),
        (
            "mtaalamu_scanner",
            "Background scanner: status (av-sessions) or one supervised auto-work cycle (mode=run_once, HITL).",
            {"type": "object", "properties": {"mode": {"type": "string"}, "approved": {"type": "boolean"}, **_ROLE_PROP}},
            _t_scanner, "🔎",
        ),
        (
            "mtaalamu_job",
            "Remote job submission: kind in {solve, scan, deploy_summary, agentic}.",
            {"type": "object", "properties": {"kind": {"type": "string"}, "msg": {"type": "string"}, **_ROLE_PROP}},
            _t_job, "🗂️",
        ),
    ]
    for name, desc, schema, handler, emoji in tools:
        try:
            ctx.register_tool(
                name=name,
                toolset="mtaalamu",
                schema=schema,
                handler=handler,
                description=desc,
                emoji=emoji,
            )
            log.info("MTAALAMU tool registered: %s", name)
        except Exception:
            log.exception("Failed to register MTAALAMU tool %s", name)


def register(ctx) -> None:
    """Plugin entry point (Hermes PluginManager general plugin contract)."""
    _register_tools(ctx)
