#!/usr/bin/env python3
"""Sukuma kazi ya /workspace/project kwenda repo ya GitHub kupitia API.

Hutengeneza branch mpya kwa kutumia Git data API (blobs -> tree -> commit -> ref)
bila kuhitaji clone nzito. Matumizi:

    python3 scripts/push_github.py <owner/repo> <branch> <dest_dir>
"""
import base64
import json
import os
import subprocess
import sys
import urllib.error
import urllib.request

REPO = sys.argv[1] if len(sys.argv) > 1 else "onearmy756-cmd/MTAALAMU-SMART"
BRANCH = sys.argv[2] if len(sys.argv) > 2 else "agentic-enginr-ai-engine"
DEST = sys.argv[3].strip("/") if len(sys.argv) > 3 else "ai-engine"
BASE = sys.argv[4] if len(sys.argv) > 4 else "main"
SRC = "/workspace/project"

TOKEN = os.environ["GITHUB_TOKEN"]
API = "https://api.github.com"
H = {
    "Authorization": f"token {TOKEN}",
    "Accept": "application/vnd.github+json",
    "X-GitHub-Api-Version": "2022-11-28",
    "User-Agent": "openhands-agent",
}


def req(method, path, payload=None):
    data = json.dumps(payload).encode() if payload is not None else None
    r = urllib.request.Request(f"{API}{path}", data=data, headers=H, method=method)
    try:
        with urllib.request.urlopen(r) as resp:
            body = resp.read()
            return json.loads(body) if body else {}
    except urllib.error.HTTPError as e:
        raise SystemExit(f"{method} {path} -> {e.code}: {e.read().decode()[:400]}")


files = subprocess.run(
    ["git", "-C", SRC, "ls-files"], capture_output=True, text=True, check=True
).stdout.split()
print(f"faili {len(files)}")

base_ref = req("GET", f"/repos/{REPO}/git/ref/heads/{BASE}")
base_sha = base_ref["object"]["sha"]
base_tree = req("GET", f"/repos/{REPO}/git/commits/{base_sha}")["tree"]["sha"]
print(f"base {BASE}={base_sha[:8]} tree={base_tree[:8]}")

entries = []
for i, rel in enumerate(files, 1):
    raw = open(os.path.join(SRC, rel), "rb").read()
    blob = req(
        "POST",
        f"/repos/{REPO}/git/blobs",
        {"content": base64.b64encode(raw).decode(), "encoding": "base64"},
    )
    entries.append({"path": f"{DEST}/{rel}", "mode": "100644", "type": "blob", "sha": blob["sha"]})
    if i % 10 == 0 or i == len(files):
        print(f"  blobs {i}/{len(files)}")

tree = req("POST", f"/repos/{REPO}/git/trees", {"base_tree": base_tree, "tree": entries})
msg = (
    f"{DEST}: AGENTIC-ENGINR — injini ya akili ya Rust (Qwen2.5-VL) + madaraja ya Go/Elixir\n\n"
    "Rust (akili-core + akilid): upakiaji wa safetensors kwa mmap (bf16/f16/f32),\n"
    "kichwa cha macho (ViT), decoder ya maandishi (GQA, M-RoPE, KV cache),\n"
    "uthibitisho wa namba dhidi ya rejeleo la transformers, kumbukumbu, sampler.\n"
    "Go (akilibridge) + Elixir wrappers. Scripts za rejeleo na mazingira.\n\n"
    "Co-authored-by: openhands <openhands@all-hands.dev>"
)
commit = req(
    "POST",
    f"/repos/{REPO}/git/commits",
    {"message": msg, "tree": tree["sha"], "parents": [base_sha]},
)
print(f"commit {commit['sha'][:8]}")

try:
    req("POST", f"/repos/{REPO}/git/refs", {"ref": f"refs/heads/{BRANCH}", "sha": commit["sha"]})
    print(f"branch mpya: {BRANCH}")
except SystemExit as e:
    if "422" not in str(e):
        raise
    req("PATCH", f"/repos/{REPO}/git/refs/heads/{BRANCH}", {"sha": commit["sha"], "force": False})
    print(f"branch ilisasishwa: {BRANCH}")

print("TAYARI", commit["sha"])
