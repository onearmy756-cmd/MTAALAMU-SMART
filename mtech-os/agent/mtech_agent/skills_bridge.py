"""MTECH OS — daraja la akili ya MTAALAMU SMART (skills 146).

Inasoma data/skills/skills.json (KANUNI 2/5: content yote kutoka JSON).
Kama engine ya Rust (mtaalamu) ipo, inaitumia; la, inahesabu formula
kwa evaluator salama (ast — hakuna eval).
"""
import ast
import json
import operator
import subprocess
from pathlib import Path

from .config import CONFIG

_OPS = {
    ast.Add: operator.add, ast.Sub: operator.sub, ast.Mult: operator.mul,
    ast.Div: operator.truediv, ast.Mod: operator.mod, ast.Pow: operator.pow,
    ast.USub: operator.neg, ast.UAdd: operator.pos,
    ast.FloorDiv: operator.floordiv,
}
_FUNCS = {
    "sqrt": lambda x: x ** 0.5, "abs": abs, "min": min, "max": max,
    "ceil": lambda x: float(int(x) + (1 if x > int(x) else 0)),
    "floor": lambda x: float(int(x)),
    "round": lambda x: float(round(x, 2)), "pow": pow,
}

_SKILLS: list = None


def load_skills() -> list:
    global _SKILLS
    if _SKILLS is not None:
        return _SKILLS
    try:
        raw = json.loads(CONFIG.skills_file.read_text())
        _SKILLS = raw.get("skills", raw if isinstance(raw, list) else [])
    except (OSError, json.JSONDecodeError):
        _SKILLS = []
    return _SKILLS


def trades() -> dict:
    try:
        raw = json.loads((CONFIG.data_dir / "trades.json").read_text())
        items = raw.get("trades", raw) if isinstance(raw, dict) else raw
        return {t.get("id", t.get("name", "?")): t for t in items} if isinstance(items, list) else items
    except (OSError, json.JSONDecodeError, AttributeError):
        return {}


def _brief(s: dict) -> dict:
    return {
        "id": s.get("id"),
        "name": s.get("name_sw") or s.get("name_en"),
        "trade": s.get("trade"),
        "action": s.get("action"),
        "formula": s.get("formula", ""),
        "inputs": [i.get("name") for i in s.get("inputs", [])],
        "tools": s.get("tools", []),
    }


def search_skills(q: str) -> list:
    skills = load_skills()
    if not q:
        return [_brief(s) for s in skills]
    q = q.lower().strip()
    hits = []
    for s in skills:
        hay = " ".join(
            str(s.get(k, "")).lower() for k in ("id", "name_sw", "name_en", "trade", "action")
        ) + " " + " ".join(str(t).lower() for t in s.get("tools", []))
        if q in hay:
            hits.append(_brief(s))
    return hits


def plan_skills(msg: str) -> list:
    """Ulinganifu wa maneno mengi — 'skrini ya simu imevunjika' → skills zinazohusiana."""
    words = {w for w in msg.lower().split() if len(w) > 2}
    scored = []
    for s in load_skills():
        hay = (str(s.get("name_sw", "")) + " " + str(s.get("name_en", "")) + " " + str(s.get("id", ""))).lower()
        score = sum(1 for w in words if w in hay)
        if score:
            scored.append((score, s))
    scored.sort(key=lambda x: -x[0])
    return [_brief(s) for _, s in scored[:10]]


# --------------------------------------------------------------- formula eval
def _eval_node(node, env):
    if isinstance(node, ast.Expression):
        return _eval_node(node.body, env)
    if isinstance(node, ast.Constant) and isinstance(node.value, (int, float)):
        return float(node.value)
    if isinstance(node, ast.Name):
        if node.id in env:
            return float(env[node.id])
        raise ValueError(f"input haijulikani: {node.id}")
    if isinstance(node, ast.BinOp) and type(node.op) in _OPS:
        return _OPS[type(node.op)](_eval_node(node.left, env), _eval_node(node.right, env))
    if isinstance(node, ast.UnaryOp) and type(node.op) in _OPS:
        return _OPS[type(node.op)](_eval_node(node.operand, env))
    if isinstance(node, ast.Call) and isinstance(node.func, ast.Name) and node.func.id in _FUNCS:
        return _FUNCS[node.func.id](*[_eval_node(a, env) for a in node.args])
    raise ValueError(f"lugha hairuhusiwi: {ast.dump(node)[:60]}")


def eval_formula(formula: str, inputs: dict):
    """'muda = 45 + (num_programs * 5)' → hesabu salama (R-1: LLM haihesabu, code inahesabu)."""
    if "=" not in formula:
        raise ValueError("formula haina '='")
    lhs, rhs = formula.split("=", 1)
    lhs, rhs = lhs.strip(), rhs.strip()
    env = {k: v for k, v in inputs.items()}
    tree = ast.parse(rhs, mode="eval")
    return lhs, _eval_node(tree, env)


def run_skill(skill_id: str, inputs: dict) -> dict:
    skills = {s.get("id"): s for s in load_skills()}
    skill = skills.get(skill_id)
    if not skill:
        return {"ok": False, "error": f"skill haiipo: {skill_id}"}

    # 1) Engine ya Rust ikiwepo — tumia (hesabu deterministic za MTAALAMU)
    engine = Path(CONFIG.engine_bin)
    if engine.exists():
        try:
            p = subprocess.run(
                [str(engine), "skill", "--id", skill_id, "--inputs", json.dumps(inputs)],
                capture_output=True, text=True, timeout=60,
            )
            if p.returncode == 0:
                return {"ok": True, "via": "mtaalamu-engine", "result": p.stdout.strip()[:2000]}
        except (OSError, subprocess.TimeoutExpired):
            pass

    # 2) Fallback: evaluator salama ya formula (offline, bila engine)
    formula = skill.get("formula", "")
    if not formula:
        return {
            "ok": True, "via": "guide",
            "name": skill.get("name_sw"), "trade": skill.get("trade"),
            "action": skill.get("action"), "tools": skill.get("tools", []),
            "note": "Skill hii ni maelekezo (guide) — hakuna formula ya kuhesabu.",
        }
    try:
        filled = {i["name"]: inputs.get(i["name"], i.get("default", 0)) for i in skill.get("inputs", [])}
        lhs, value = eval_formula(formula, filled)
        return {"ok": True, "via": "formula-eval", "skill": skill_id, "result": {lhs: round(value, 4)}, "inputs_used": filled}
    except (ValueError, ZeroDivisionError) as e:
        return {"ok": False, "error": f"formula imeshindikana: {e}"}


def skills_summary() -> dict:
    skills = load_skills()
    by_trade = {}
    for s in skills:
        by_trade.setdefault(s.get("trade", "?"), 0)
        by_trade[s.get("trade", "?")] += 1
    return {"total": len(skills), "trades": by_trade, "source": str(CONFIG.skills_file)}
