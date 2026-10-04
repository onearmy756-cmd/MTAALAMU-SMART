"""MTECH OS — CLI + agent loop + HTTP API.

    mtech probe                     # uchunguzi wa mfumo (kernel/proc)
    mtech skills [--q NENO]         # skills 146 za MTAALAMU
    mtech skill --id X --inputs {}  # tekeleza skill
    mtech ask "tatizo..."           # agent loop kamili (Qwen VL + zana)
    mtech watch                     # matukio ya kernel live
    mtech serve                     # HTTP API kwa MTECH Shell (GUI)
"""
import argparse
import json
import re
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

from .config import CONFIG, SYSTEM_PROMPT
from .kernel_events import start_feed
from .safety import Gate
from .tools import TOOL_HELP, TOOLS

# ----------------------------------------------------------------- JSON utils
def parse_action(text: str) -> dict:
    text = text.strip()
    text = re.sub(r"^```(json)?|```$", "", text, flags=re.M).strip()
    try:
        return json.loads(text)
    except json.JSONDecodeError:
        m = re.search(r"\{.*\}", text, flags=re.S)
        if m:
            try:
                return json.loads(m.group(0))
            except json.JSONDecodeError:
                pass
    return {"answer": text or "Sijaelewa jibu la modeli."}


# ----------------------------------------------------------------- agent loop
class Agent:
    def __init__(self, approve: bool = False, hitl_cb=None):
        self.gate = Gate(auto_approve=approve or CONFIG.approve_all, callback=hitl_cb)
        self.history: list = []
        self.events = start_feed()
        self.last_screenshot = None

    def _tool(self, name: str, args: dict) -> dict:
        if name == "see_screen":
            from .vision import see_screen
            return see_screen(self, args)
        if name == "kernel_events":
            return {"ok": True, "source": self.events.source, "events": self.events.snapshot(int(args.get("limit", 20)))}
        fn = TOOLS.get(name)
        if fn is None:
            return {"ok": False, "error": f"zana haijulikani: {name}"}
        try:
            return fn(args, self.gate)
        except Exception as e:  # noqa: BLE001 — agent isirudishe crash
            return {"ok": False, "error": f"{type(e).__name__}: {e}"}

    def run(self, task: str, verbose: bool = True) -> dict:
        self.history = [
            {"role": "system", "content": SYSTEM_PROMPT},
            {"role": "user", "content": f"KAZI: {task}\nKonteksti: skills={json.dumps(_skills_ctx())}"},
        ]
        trace = []
        from .llm import chat, OllamaError
        for step in range(CONFIG.max_steps):
            try:
                raw = chat(self.history)
            except OllamaError as e:
                return {"ok": False, "error": str(e), "trace": trace}
            action = parse_action(raw)
            trace.append({"step": step + 1, "action": action})
            if verbose:
                print(f"  ▸ hatua {step + 1}: {action.get('thought', '')[:100]}")
            if "answer" in action:
                return {"ok": True, "answer": action["answer"], "trace": trace, "gate_log": self.gate.log}
            tool = action.get("tool", "")
            args = action.get("args") or {}
            result = self._tool(tool, args)
            trace[-1]["result"] = {k: (str(v)[:200]) for k, v in list(result.items())[:6]}
            self.history.append({"role": "assistant", "content": raw})
            self.history.append({"role": "user", "content": "MATOKEO ya zana " + tool + ": " + json.dumps(result, ensure_ascii=False)[:3000]})
        return {"ok": False, "answer": "Nimefika mwisho wa hatua bila jibu la mwisho.", "trace": trace}


def _skills_ctx() -> list:
    from .skills_bridge import skills_summary
    s = skills_summary()
    return [{"total": s["total"], "trades": list(s["trades"].items())[:8]}]


# ----------------------------------------------------------------- HTTP (GUI)
class _API(BaseHTTPRequestHandler):
    agent: Agent = None

    def _json(self, code: int, payload: dict) -> None:
        body = json.dumps(payload, ensure_ascii=False).encode()
        self.send_response(code)
        self.send_header("Content-Type", "application/json; charset=utf-8")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def do_GET(self):  # noqa: N802
        if self.path.startswith("/api/status"):
            from .skills_bridge import skills_summary
            self._json(200, {
                "model": CONFIG.model, "ollama": CONFIG.ollama_url,
                "events_source": self.agent.events.source,
                "skills": skills_summary(),
            })
        elif self.path.startswith("/api/events"):
            limit = 30
            if "limit=" in self.path:
                try:
                    limit = int(self.path.split("limit=")[1].split("&")[0])
                except ValueError:
                    pass
            self._json(200, {"events": self.agent.events.snapshot(limit)})
        elif self.path.startswith("/api/gate"):
            pending = [e for e in self.agent.gate.log if e["risk"] == "HIGH"]
            self._json(200, {"pending": pending[-5:]})
        else:
            self._json(404, {"error": "hakuna"})

    def do_POST(self):  # noqa: N802
        n = int(self.headers.get("Content-Length", 0))
        try:
            payload = json.loads(self.rfile.read(n).decode() or "{}")
        except json.JSONDecodeError:
            return self._json(400, {"error": "JSON mbaya"})
        if self.path.startswith("/api/ask"):
            res = self.agent.run(payload.get("msg", ""))
            return self._json(200, res)
        if self.path.startswith("/api/skills"):
            from .skills_bridge import search_skills, plan_skills
            q = payload.get("q", "")
            hits = search_skills(q) or plan_skills(q)
            return self._json(200, {"count": len(hits), "skills": hits[:15]})
        if self.path.startswith("/api/skill/run"):
            from .skills_bridge import run_skill
            return self._json(200, run_skill(payload.get("id", ""), payload.get("inputs") or {}))
        self._json(404, {"error": "hakuna"})

    def log_message(self, *a):  # kimya
        pass


def serve(port: int, approve: bool) -> None:
    _API.agent = Agent(approve=approve)
    httpd = ThreadingHTTPServer(("127.0.0.1", port), _API)
    print(f"[MTECH] API: http://127.0.0.1:{port}  (model: {CONFIG.model})")
    httpd.serve_forever()


# ----------------------------------------------------------------- CLI
def main() -> None:
    ap = argparse.ArgumentParser(prog="mtech", description="MTECH OS — akili ya mfumo")
    sub = ap.add_subparsers(dest="cmd")
    sub.add_parser("probe")
    p_skills = sub.add_parser("skills")
    p_skills.add_argument("--q", default="")
    p_skill = sub.add_parser("skill")
    p_skill.add_argument("--id", required=True)
    p_skill.add_argument("--inputs", default="{}")
    p_ask = sub.add_parser("ask")
    p_ask.add_argument("msg")
    p_ask.add_argument("--approve", action="store_true")
    p_ask.add_argument("--quiet", action="store_true")
    sub.add_parser("watch")
    p_serve = sub.add_parser("serve")
    p_serve.add_argument("--port", type=int, default=CONFIG.serve_port)
    p_serve.add_argument("--approve", action="store_true")
    args = ap.parse_args()

    if args.cmd == "probe":
        print(json.dumps(TOOLS["sys_probe"]({}, Gate(auto_approve=True))["system"], ensure_ascii=False, indent=1)[:4000])
    elif args.cmd == "skills":
        from .skills_bridge import search_skills, skills_summary
        s = skills_summary()
        print(f"MTAALAMU SMART — skills {s['total']} kwenye {len(s['trades'])} trades ({s['source']})")
        for sk in search_skills(args.q)[:30]:
            print(f"  • {sk.get('id')}  [{sk.get('trade')}] {sk.get('name')}")
    elif args.cmd == "skill":
        from .skills_bridge import run_skill
        print(json.dumps(run_skill(args.id, json.loads(args.inputs)), ensure_ascii=False, indent=1))
    elif args.cmd == "ask":
        agent = Agent(approve=args.approve)
        res = agent.run(args.msg, verbose=not args.quiet)
        print("\n" + ("◆ " + res.get("answer", res.get("error", "")) if res["ok"] else "✖ " + str(res.get("error"))))
    elif args.cmd == "watch":
        feed = start_feed()
        print(f"[MTECH] kernel events kutoka: {feed.source}  (Ctrl-C kuacha)")
        seen = 0
        try:
            while True:
                for ev in feed.snapshot(500)[seen:]:
                    print(f"  {ev['type']:<5} pid={ev['pid']:<7} {ev['detail'][:60]}")
                    seen += 1
        except KeyboardInterrupt:
            pass
    elif args.cmd == "serve":
        serve(args.port, getattr(args, "approve", False))
    else:
        ap.print_help()


if __name__ == "__main__":
    main()
