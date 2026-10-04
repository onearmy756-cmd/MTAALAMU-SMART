"""MTAALAMU SMART — HTTP API (UI ya hermes-agent inaunganishwa hapa).

    python3 -m mtaalamu.server          # http://127.0.0.1:8795

Endpoints (zote JSON):
  GET  /api/status            hali (model, license, usage, os, ipc)
  GET  /api/catalog           ops zote (grouped, sw/en)
  POST /api/solve             {"text": "..."}                       → plan + questions
  POST /api/solve/answer      {"text": "...", "answers": {...}}     → execute
  POST /api/register          {"email","plan","tier","seats"?}      → license key
  GET  /api/usage             metering (credits)
  POST /api/lang              {"lang": "sw"|"en"}
  GET  /api/shortcuts         shortcuts zote (win/mac/ubuntu/linux)
  POST /api/shortcuts/run     {"os": "...", "key": "..."}           → HITL ya keyboard
  POST /api/models            {"mode","model","api_url","api_key_env"}
  GET  /api/doctor            kama mtech doctor

Malipo (ClickPesa — pay-before-use):
  GET  /api/wallet            salio + hali ya gateway
  POST /api/pay               {"amount_tzs"|"op_id", "phone"} → USSD push + poll
  GET  /api/pay/status?ref=MST...   hali ya malipo kwa reference
  POST /api/webhook/clickpesa (ClickPesa) → verify HMAC → wallet_topup

Admin (FULL SYSTEM — inahitaji mtaalamu admin unlock):
  POST /api/admin/unlock      {"key"?}  → admin (MTECH_ADMIN_KEY ama owner-mode)
  POST /api/admin/lock        zima admin
  GET  /api/admin/dashboard   FULL SYSTEM: OS + leseni + wallet + catalog (bei/tier)
  POST /api/admin/price       {"op_id", "amount_tzs"} (0 = BURE)
  POST /api/admin/tier        {"op_id", "tier"} (ANY/BASIC/.../DIAMOND)
"""
import json
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

from . import admin, billing, clickpesa, ipc, models, scope, unified
from .catalog import CATALOG, GROUPS
from .shortcuts import ALL_SHORTCUTS, run_shortcut


class API(BaseHTTPRequestHandler):
    def _send(self, code: int, payload: dict):
        body = json.dumps(payload, ensure_ascii=False).encode()
        self.send_response(code)
        self.send_header("Content-Type", "application/json; charset=utf-8")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def _body(self) -> dict:
        n = int(self.headers.get("Content-Length", 0) or 0)
        try:
            return json.loads(self.rfile.read(n).decode() or "{}")
        except json.JSONDecodeError:
            return {}

    def do_GET(self):  # noqa: N802
        p = self.path.split("?")[0]
        if p == "/api/status":
            self._send(200, {
                "os": unified.FAMILY, "lang": scope.LANG,
                "model": models.load_choice(), "license": billing.usage_summary(),
                "catalog_ops": len(CATALOG), "groups": GROUPS,
            })
        elif p == "/api/catalog":
            tier = (billing.current_license() or {}).get("tier")
            self._send(200, {"groups": GROUPS, "ops": [
                {"id": o.id, "group": o.group, "sw": o.name_sw, "en": o.name_en,
                 "risk": o.risk, "offline": o.offline, "credits": o.credits, "os": o.os,
                 "price_tzs": billing.price_tzs(o.id, o.risk, tier),
                 "price": billing.fmt_tzs(billing.price_tzs(o.id, o.risk, tier))}
                for o in CATALOG]})
        elif p == "/api/usage":
            self._send(200, billing.usage_summary())
        elif p == "/api/wallet":
            self._send(200, {"wallet_tzs": billing.wallet_balance(),
                             "gateway": clickpesa.status_mode()})
        elif p == "/api/pay/status":
            ref = (self.path.split("?", 1)[1] if "?" in self.path else "")
            ref = dict(kv.split("=", 1) for kv in ref.split("&") if "=" in kv).get("ref", "")
            if not ref:
                self._send(400, {"ok": False, "error": "ref inahitajika"})
            else:
                self._send(200, {"ok": True, "ref": ref, "payments": clickpesa.payment_status(ref)})
        elif p == "/api/shortcuts":
            self._send(200, {"shortcuts": ALL_SHORTCUTS})
        elif p == "/api/doctor":
            from .doctor_lite import run_checks
            self._send(200, run_checks())
        elif p == "/api/admin/dashboard":
            try:
                self._send(200, admin.dashboard(probe=unified.probe_system()))
            except PermissionError as e:
                self._send(401, {"ok": False, "error": str(e)})
        else:
            self._send(404, {"error": "hakuna"})

    def do_POST(self):  # noqa: N802
        p = self.path.split("?")[0]
        body = self._body()
        try:
            if p == "/api/solve":
                self._send(200, unified.solve(body.get("text", "")))
            elif p == "/api/solve/answer":
                self._send(200, unified.solve(body.get("text", ""),
                                              answers=body.get("answers") or {},
                                              auto_confirm=bool(body.get("auto"))))
            elif p == "/api/register":
                lic = billing.register(body.get("email", ""), body.get("plan", ""),
                                       body.get("tier", ""), body.get("seats"))
                self._send(200, {"ok": True, "license": lic})
            elif p == "/api/lang":
                scope.set_lang(body.get("lang", "sw"))
                self._send(200, {"ok": True, "lang": scope.LANG})
            elif p == "/api/shortcuts/run":
                self._send(200, run_shortcut(body.get("os", ""), body.get("key", "")))
            elif p == "/api/models":
                models.save_choice(body.get("mode", "local"), body.get("model", models.DEFAULT_LOCAL),
                                   body.get("api_url", ""), body.get("api_key_env", ""))
                self._send(200, {"ok": True, "choice": models.load_choice()})
            elif p == "/api/pay":
                if not clickpesa.available():
                    self._send(503, {"ok": False, "error":
                                     "ClickPesa haijawekwa — weka CLICKPESA_CLIENT_ID + CLICKPESA_API_KEY"})
                    return
                amount = int(body.get("amount_tzs") or 0)
                op_id = body.get("op_id", "")
                if op_id and not amount:
                    op = __import__(".catalog", fromlist=["CATALOG_BY_ID"]).CATALOG_BY_ID.get(op_id)
                    if not op:
                        self._send(400, {"ok": False, "error": f"op haijulikani: {op_id}"})
                        return
                    tier = (billing.current_license() or {}).get("tier")
                    amount = billing.price_tzs(op.id, op.risk, tier)
                phone = body.get("phone", "")
                out = clickpesa.collect(amount, phone)
                if out.get("ok"):
                    billing.wallet_topup(out["amount_tzs"], source="clickpesa",
                                         ref=out.get("ref", ""), tx=out.get("tx", ""),
                                         channel=out.get("channel", ""))
                    out["wallet_tzs"] = billing.wallet_balance()
                self._send(200, out)
            elif p == "/api/admin/unlock":
                info = admin.unlock(body.get("key"))
                self._send(200, {"ok": True, "admin": info,
                                 "note": "owner-mode: weka MTECH_ADMIN_KEY kwenye Environment kwa usalama zaidi"
                                         if info["mode"] == "owner" else "key mode"})
            elif p == "/api/admin/lock":
                self._send(200, {"ok": True, "admin": admin.lock()})
            elif p == "/api/admin/price":
                try:
                    self._send(200, {"ok": True, **admin.set_price(body.get("op_id", ""),
                                                                   int(body.get("amount_tzs", 0)))})
                except PermissionError as e:
                    self._send(401, {"ok": False, "error": str(e)})
            elif p == "/api/admin/tier":
                try:
                    self._send(200, {"ok": True, **admin.set_tool_tier(body.get("op_id", ""),
                                                                       body.get("tier", "ANY"))})
                except PermissionError as e:
                    self._send(401, {"ok": False, "error": str(e)})
            elif p == "/api/webhook/clickpesa":
                sig = self.headers.get("X-ClickPesa-Signature", "")
                if not clickpesa.verify_webhook(body, sig):
                    self._send(401, {"ok": False, "error": "signature si halali — webhook imekataliwa"})
                    return
                st = str(body.get("status", "")).upper()
                if st in ("SUCCESS", "SETTLED"):
                    amt = int(float(body.get("collectedAmount") or body.get("amount") or 0))
                    if amt > 0:
                        w = billing.wallet_topup(amt, source="clickpesa-webhook",
                                                 ref=body.get("orderReference", ""),
                                                 tx=body.get("id", ""), channel=body.get("channel", ""))
                        self._send(200, {"ok": True, "credited_tzs": amt, "wallet_tzs": w["balance_tzs"]})
                        return
                self._send(200, {"ok": True, "note": f"event '{st or '?'}' — hakuna credit"})
            else:
                self._send(404, {"error": "hakuna"})
        except ValueError as e:
            self._send(400, {"ok": False, "error": str(e)})
        except Exception as e:  # noqa: BLE001
            self._send(500, {"ok": False, "error": f"{type(e).__name__}: {e}"})

    def log_message(self, *a):  # kimya
        pass


def main() -> None:
    ipc.start_ipc(handler=lambda req: unified.solve(req.get("text", ""),
                                                    answers=req.get("answers"),
                                                    auto_confirm=req.get("auto", False)))
    port = int(__import__("os").environ.get("MTECH_API_PORT", "8795"))
    print(f"[MTAALAMU] API: http://127.0.0.1:{port}  (IPC: {ipc.SOCK_PATH})")
    ThreadingHTTPServer(("127.0.0.1", port), API).serve_forever()


if __name__ == "__main__":
    main()
