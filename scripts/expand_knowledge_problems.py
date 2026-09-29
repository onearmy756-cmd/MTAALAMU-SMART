#!/usr/bin/env python3
"""Expand data/knowledge/problems_parts.b64.{a,b}.txt → data/knowledge/problems/*.json"""
import base64, gzip, json
from pathlib import Path
ROOT = Path(__file__).resolve().parents[1]
DIR = ROOT / "data" / "knowledge"
OUT = DIR / "problems"
def main():
    OUT.mkdir(parents=True, exist_ok=True)
    parts = []
    for name in ("problems_parts.b64.a.txt", "problems_parts.b64.b.txt", "problems_parts.b64.txt"):
        p = DIR / name
        if p.exists():
            parts.append(p.read_text(encoding="utf-8").strip())
    if not parts:
        raise SystemExit("missing problems_parts.b64*.txt — run split_problems_to_knowledge.py instead")
    b64 = "".join(parts)
    data = json.loads(gzip.decompress(base64.b64decode(b64)).decode("utf-8"))
    for name, content in sorted(data.items()):
        (OUT / name).write_text(content, encoding="utf-8")
        print("wrote", name, len(content))
    print("done", len(data), "files →", OUT)
if __name__ == "__main__":
    main()
