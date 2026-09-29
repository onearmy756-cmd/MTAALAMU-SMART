#!/usr/bin/env python3
"""Split data/problems.json → data/knowledge/problems/{trade}.json (21 trades × 20 = 420)."""
from __future__ import annotations
import json
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SRC = ROOT / "data" / "problems.json"
OUT = ROOT / "data" / "knowledge" / "problems"

def main() -> None:
    OUT.mkdir(parents=True, exist_ok=True)
    doc = json.loads(SRC.read_text(encoding="utf-8"))
    problems = doc.get("problems") or []
    by: dict[str, list] = defaultdict(list)
    for p in problems:
        by[p.get("trade") or "general"].append(p)
    trades_meta = {}
    for trade, items in sorted(by.items()):
        compact = []
        for p in items:
            compact.append({
                "id": p.get("id"),
                "trade": p.get("trade"),
                "description": p.get("description"),
                "symptoms": p.get("symptoms"),
                "causes": p.get("causes"),
                "solution": p.get("solution"),
                "time_min": p.get("time_min"),
                "cost_tzs": p.get("cost_tzs"),
                "success_rate": p.get("success_rate"),
                "severity": p.get("severity"),
            })
        body = {"trade": trade, "count": len(compact), "problems": compact}
        path = OUT / f"{trade}.json"
        path.write_text(json.dumps(body, ensure_ascii=False, separators=(",", ":")), encoding="utf-8")
        trades_meta[trade] = {"file": path.name, "count": len(compact)}
        print(f"  {trade}: {len(compact)} → {path}")
    man = {"total": len(problems), "trades": trades_meta}
    (OUT / "manifest.json").write_text(json.dumps(man, ensure_ascii=False, indent=2), encoding="utf-8")
    print(f"Done: {len(problems)} problems → {len(by)} trades in {OUT}")

if __name__ == "__main__":
    main()
