#!/usr/bin/env python3
"""Merge data/devices/*.json → data/devices_catalog.json (100%)."""
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DEV = ROOT / "data" / "devices"
OUT = ROOT / "data" / "devices_catalog.json"

def main():
    devices = []
    for p in sorted(DEV.glob("*.json")):
        if p.name in ("manifest.json", "kanuni_10.json"):
            continue
        doc = json.loads(p.read_text(encoding="utf-8"))
        for d in doc.get("devices") or []:
            devices.append(d)
    man = {}
    mp = DEV / "manifest.json"
    if mp.exists():
        man = json.loads(mp.read_text(encoding="utf-8"))
    kanuni = None
    kp = DEV / "kanuni_10.json"
    if kp.exists():
        kanuni = json.loads(kp.read_text(encoding="utf-8"))
    cat = {
        "version": 1,
        "jina": man.get("jina") or "Electronic Devices Solver",
        "jumla_vifaa_catalog": len(devices),
        "jumla_matatizo_indexed": sum(len(d.get("matatizo") or []) for d in devices),
        "devices": devices,
        "suluhisho_kanuni_10": kanuni,
        "makundi_20": man.get("makundi_20") or [],
        "hitimisho": man.get("hitimisho"),
    }
    OUT.write_text(json.dumps(cat, ensure_ascii=False, indent=2), encoding="utf-8")
    print(f"Wrote {OUT}: {cat['jumla_vifaa_catalog']} devices, {cat['jumla_matatizo_indexed']} problems")

if __name__ == "__main__":
    main()
