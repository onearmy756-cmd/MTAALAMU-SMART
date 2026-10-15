#!/usr/bin/env python3
"""Hubadilisha `.npy` za rejeleo kuwa raw f32 + manifest.json (kwa uthibitisho wa Rust).

Matumizi:
    python3 scripts/ref_convert.py <ref_dir>
"""
import json
import os
import sys

import numpy as np

REF = sys.argv[1] if len(sys.argv) > 1 else "/tmp/ref"
man = {}
for name in sorted(os.listdir(REF)):
    if not name.endswith(".npy"):
        continue
    arr = np.load(os.path.join(REF, name)).astype(np.float32)
    base = name[:-4]
    arr.tofile(os.path.join(REF, base + ".bin"))
    man[base] = {"shape": list(arr.shape), "bytes": int(arr.size * 4)}
json.dump(man, open(os.path.join(REF, "manifest.json"), "w"), indent=1)
print("manifest:", json.dumps({k: v["shape"] for k, v in man.items()}, indent=1))
