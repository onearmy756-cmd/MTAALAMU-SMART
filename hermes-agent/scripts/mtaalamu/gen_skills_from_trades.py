#!/usr/bin/env python3
"""Gen data/skills/skills.json — skills ZOTE kutoka data/trades.json + AI/web/PDF skills.

Kanuni: data-driven (KANUNI 2/5). Generator hii inasoma trades.json (source of truth)
na kuunganisha na AI skills zilizoainishwa hapa. Endesha tena ukibadilisha trades.json:

    python3 scripts/gen_skills_from_trades.py
"""
import json
import re
import unicodedata
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
TRADES = ROOT / "data" / "trades.json"
OUT = ROOT / "data" / "skills" / "skills.json"


def slug(s: str) -> str:
    s = unicodedata.normalize("NFKD", s)
    s = s.encode("ascii", "ignore").decode("ascii")
    s = re.sub(r"[^a-zA-Z0-9]+", "_", s).strip("_").lower()
    return s or "skill"


def extract_inputs(formula: str):
    """Variable names kutoka formula (kabla ya '=') — default 1 kila moja."""
    if not formula or "=" not in formula:
        return []
    rhs = formula.split("=", 1)[1]
    names = []
    for tok in re.findall(r"[A-Za-z_][A-Za-z0-9_]*", rhs):
        if tok in ("muda", "x"):
            continue
        if tok not in names:
            names.append(tok)
    return [{"name": n, "label": n, "en": n, "default": 1} for n in names[:6]]


AI_SKILLS = [
    {
        "id": "ai.pytorch",
        "trade": "ai",
        "name_sw": "PyTorch — mifano ya ML (train/infer)",
        "name_en": "PyTorch — ML models (train/infer)",
        "difficulty": 4,
        "formula": "muda = 30 + (epochs * 2)",
        "inputs": [{"name": "epochs", "label": "Epochs", "en": "Epochs", "default": 10}],
        "tools": ["python3 + torch", "GPU (hiari)", "data/"],
        "action": "pytorch_train",
        "runtime": "python3",
        "script": "scripts/ai_pytorch.py",
    },
    {
        "id": "ai.tensorflow",
        "trade": "ai",
        "name_sw": "TensorFlow — mifano ya ML (train/infer)",
        "name_en": "TensorFlow — ML models (train/infer)",
        "difficulty": 4,
        "formula": "muda = 30 + (epochs * 2)",
        "inputs": [{"name": "epochs", "label": "Epochs", "en": "Epochs", "default": 10}],
        "tools": ["python3 + tensorflow", "GPU (hiari)", "data/"],
        "action": "tensorflow_train",
        "runtime": "python3",
        "script": "scripts/ai_tensorflow.py",
    },
    {
        "id": "ai.pdf_search",
        "trade": "ai",
        "name_sw": "Kutafuta PDF nyingi kwa wakati mmoja (parallel)",
        "name_en": "Parallel PDF search",
        "difficulty": 2,
        "formula": "muda = 10 + (files * 1)",
        "inputs": [{"name": "files", "label": "Faili", "en": "Files", "default": 20}],
        "tools": ["Rust pdf engine (pdf_extract.rs)", "data/pdfs/"],
        "action": "pdf_search",
        "runtime": "rust",
    },
    {
        "id": "ai.searxng",
        "trade": "ai",
        "name_sw": "Meta-search ya SearXNG (injini nyingi pamoja)",
        "name_en": "SearXNG meta-search",
        "difficulty": 2,
        "formula": "muda = 5 + (engines * 2)",
        "inputs": [{"name": "engines", "label": "Engines", "en": "Engines", "default": 4}],
        "tools": ["SearXNG API", "Rust HTTP"],
        "action": "searxng_search",
        "runtime": "rust",
    },
    {
        "id": "ai.duckduckgo",
        "trade": "ai",
        "name_sw": "Tafuta mtandaoni kwa DuckDuckGo",
        "name_en": "DuckDuckGo web search",
        "difficulty": 1,
        "formula": "muda = 5 + (pages * 2)",
        "inputs": [{"name": "pages", "label": "Pages", "en": "Pages", "default": 5}],
        "tools": ["DDG lite HTML", "Rust HTTP"],
        "action": "duckduckgo_search",
        "runtime": "rust",
    },
    {
        "id": "ai.ollama_llm",
        "trade": "ai",
        "name_sw": "LLM ya bure (Ollama cloud/local — tinyllama, gpt-oss)",
        "name_en": "Free LLM (Ollama cloud/local — tinyllama, gpt-oss)",
        "difficulty": 2,
        "formula": "muda = 5 + (tokens / 200)",
        "inputs": [{"name": "tokens", "label": "Tokens", "en": "Tokens", "default": 300}],
        "tools": ["OLLAMA_API_KEY", "data/ai/models.json"],
        "action": "ollama_ask",
        "runtime": "rust",
    },
    {
        "id": "ai.huggingface",
        "trade": "ai",
        "name_sw": "Hugging Face — modeli wazi za bure (search + inference)",
        "name_en": "Hugging Face — free open models (search + inference)",
        "difficulty": 3,
        "formula": "muda = 10 + (downloads / 50)",
        "inputs": [{"name": "downloads", "label": "Downloads", "en": "Downloads", "default": 100}],
        "tools": ["HF Hub API", "HF Inference API", "Rust HTTP"],
        "action": "hf_search",
        "runtime": "rust",
    },
    {
        "id": "ai.skill_plan",
        "trade": "ai",
        "name_sw": "Mpango wa kazi kwa skills (auto-plan + hesabu)",
        "name_en": "Skill-based work plan (auto-plan + math)",
        "difficulty": 2,
        "formula": "muda = 5 + (steps * 3)",
        "inputs": [{"name": "steps", "label": "Hatua", "en": "Steps", "default": 4}],
        "tools": ["skills.json", "expr engine"],
        "action": "skill_plan",
        "runtime": "rust",
    },
]


def main():
    trades_doc = json.loads(TRADES.read_text(encoding="utf-8"))
    skills = []
    seen = set()
    for trade in trades_doc.get("trades", []):
        tid = trade.get("id", "trade")
        for s in trade.get("skills", []):
            name = s.get("name", "").strip()
            sid = f"{tid}.{slug(name)}"
            if sid in seen:
                sid = f"{sid}_{len(seen)}"
            seen.add(sid)
            skills.append(
                {
                    "id": sid,
                    "trade": tid,
                    "name_sw": name,
                    "name_en": s.get("name_en") or name,
                    "difficulty": s.get("difficulty", 2),
                    "formula": s.get("formula", "muda = 30"),
                    "inputs": s.get("inputs") or extract_inputs(s.get("formula", "")),
                    "tools": s.get("tools", []),
                    "action": "guide",
                }
            )

    for a in AI_SKILLS:
        if a["id"] not in seen:
            seen.add(a["id"])
            skills.append(a)

    OUT.parent.mkdir(parents=True, exist_ok=True)
    doc = {
        "version": "1.1.0",
        "note": "Skills ZOTE za trades zote (kutoka data/trades.json) + skills za AI/web/PDF. Rust (skills.rs) inasoma file hii pekee. Formula = muda/dakika (expr engine). Rejenerisha: python3 scripts/gen_skills_from_trades.py",
        "trades_source": "data/trades.json",
        "skills": skills,
    }
    OUT.write_text(json.dumps(doc, ensure_ascii=False, indent=1) + "\n", encoding="utf-8")
    by_trade = {}
    for s in skills:
        by_trade[s["trade"]] = by_trade.get(s["trade"], 0) + 1
    print(f"OK: {len(skills)} skills -> {OUT.relative_to(ROOT)}")
    print("  kwa trade:", json.dumps(by_trade, ensure_ascii=False))


if __name__ == "__main__":
    main()
