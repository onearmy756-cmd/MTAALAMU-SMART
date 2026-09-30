#!/usr/bin/env python3
"""Fundi image manager — orodha, ongeza, thibitisha, futa images za OS.

Commands:
  list                     — orodha (path, bytes)
  add <file> [name]        — nakili image kwenye FUNDI_IMAGES + sha256
  verify [name]            — sha256 + magic bytes (ISO9660/WIM)
  remove <name>            — futa image
"""
import argparse
import hashlib
import json
import os
import shutil
import sys
from pathlib import Path

ROOT = Path(os.environ.get("FUNDI_IMAGES", "images"))
EXTS = {".iso", ".wim", ".img", ".gz"}


def sha256_file(p: Path, buf=1024 * 1024) -> str:
    h = hashlib.sha256()
    with p.open("rb") as f:
        while chunk := f.read(buf):
            h.update(chunk)
    return h.hexdigest()


def magic_kind(p: Path) -> str:
    try:
        with p.open("rb") as f:
            head = f.read(8)
            f.seek(0x8001)
            iso = f.read(5)
    except OSError:
        return "unknown"
    if iso == b"CD001":
        return "iso"
    if head.startswith(b"MSWIM\x00\x00\x00") or head.startswith(b"WLPWM\x00\x00\x00"):
        return "wim"
    return "unknown"


def all_images():
    ROOT.mkdir(parents=True, exist_ok=True)
    return sorted(p for p in ROOT.rglob("*") if p.is_file() and p.suffix.lower() in EXTS)


def list_images():
    out = []
    for p in all_images():
        out.append({
            "path": str(p.relative_to(ROOT)),
            "bytes": p.stat().st_size,
            "kind": magic_kind(p),
        })
    print(json.dumps({"images": out, "root": str(ROOT)}, indent=2))


def add_image(src: str, name: str | None):
    s = Path(src)
    if not s.is_file():
        sys.exit(f"Hakuna faili: {src}")
    dst = ROOT / (name or s.name)
    ROOT.mkdir(parents=True, exist_ok=True)
    print(f"[fundi] nakili {s} → {dst} ...")
    shutil.copy2(s, dst)
    digest = sha256_file(dst)
    (dst.with_suffix(dst.suffix + ".sha256")).write_text(f"{digest}  {dst.name}\n")
    print(json.dumps({"added": dst.name, "bytes": dst.stat().st_size, "sha256": digest}, indent=2))


def verify_image(name: str | None):
    targets = [ROOT / name] if name else all_images()
    out = []
    for p in targets:
        if not p.is_file():
            continue
        sidecar = p.with_suffix(p.suffix + ".sha256")
        expected = sidecar.read_text().split()[0] if sidecar.exists() else None
        digest = sha256_file(p)
        out.append({
            "name": p.name,
            "kind": magic_kind(p),
            "sha256": digest,
            "matches_sidecar": (expected == digest) if expected else None,
        })
    print(json.dumps({"verified": out}, indent=2))


def remove_image(name: str):
    p = ROOT / name
    for f in (p, p.with_suffix(p.suffix + ".sha256")):
        if f.exists():
            f.unlink()
            print(f"[fundi] ulifuta {f}")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("cmd", choices=["list", "add", "verify", "remove"])
    ap.add_argument("target", nargs="?", help="file (add) au name (verify/remove)")
    ap.add_argument("name", nargs="?", help="jina mbadala (add)")
    args = ap.parse_args()
    if args.cmd == "list":
        list_images()
    elif args.cmd == "add":
        if not args.target:
            sys.exit("add <file> [name]")
        add_image(args.target, args.name)
    elif args.cmd == "verify":
        verify_image(args.target)
    elif args.cmd == "remove":
        if not args.target:
            sys.exit("remove <name>")
        remove_image(args.target)


if __name__ == "__main__":
    main()
