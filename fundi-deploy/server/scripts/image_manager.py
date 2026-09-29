#!/usr/bin/env python3
"""Fundi image manager — orodha ISO/WIM."""
import argparse, os, json
from pathlib import Path

ROOT = Path(os.environ.get("FUNDI_IMAGES", "images"))

def list_images():
    ROOT.mkdir(parents=True, exist_ok=True)
    out = []
    for p in ROOT.rglob("*"):
        if p.is_file() and p.suffix.lower() in {".iso", ".wim", ".img", ".gz"}:
            out.append({"path": str(p.relative_to(ROOT)), "bytes": p.stat().st_size})
    print(json.dumps({"images": out, "root": str(ROOT)}, indent=2))

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("cmd", choices=["list"])
    args = ap.parse_args()
    if args.cmd == "list":
        list_images()

if __name__ == "__main__":
    main()
