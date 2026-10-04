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
"""
import json
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

from . import billing, ipc, models, scope, unified
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
        elif p == "/api/shortcuts":
            self._send(200, {"shortcuts": ALL_SHORTCUTS})
        elif p == "/api/doctor":
            from .doctor_lite import run_checks
            self._send(200, run_checks())
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
