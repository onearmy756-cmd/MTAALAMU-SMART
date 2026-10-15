#!/usr/bin/env python3
"""Rejeleo la kichwa cha macho (ViT) kwa F32 — kwa uthibitisho wa namba.

Hupakia kichwa cha macho PEKEE (bila decoder ya maandishi) ili kutoshea kwenye RAM.
Husoma `pixel_values.npy` na `image_grid_thw.npy` zilizoandikwa na `ref_dump.py`.

Matumizi:
    python3 scripts/ref_vision.py <model_dir> <ref_dir>
"""
import os
import sys

import numpy as np
import torch

MODEL = sys.argv[1] if len(sys.argv) > 1 else "/tmp/ref-model"
REF = sys.argv[2] if len(sys.argv) > 2 else "/tmp/ref"
torch.set_grad_enabled(False)

from transformers import AutoConfig
from transformers.models.qwen2_5_vl.modeling_qwen2_5_vl import (
    Qwen2_5_VisionTransformerPretrainedModel,
)
from safetensors.torch import load_file

cfg = AutoConfig.from_pretrained(MODEL)
vision = Qwen2_5_VisionTransformerPretrainedModel._from_config(cfg.vision_config)
vision = vision.to(torch.float32).eval()

# Pakia uzito wa `visual.*` pekee, kutoka shards zote.
state = {}
for name in sorted(os.listdir(MODEL)):
    if name.startswith("model-") and name.endswith(".safetensors"):
        sd = load_file(os.path.join(MODEL, name))
        for k, v in sd.items():
            if k.startswith("visual."):
                state[k[len("visual.") :]] = v.float()
        del sd
missing, unexpected = vision.load_state_dict(state, strict=False)
print("missing:", len(missing), "unexpected:", len(unexpected), flush=True)
del state

pixel_values = torch.from_numpy(np.load(f"{REF}/pixel_values.npy")).to(torch.float32)
grid_thw = torch.from_numpy(np.load(f"{REF}/image_grid_thw.npy"))
out = vision(pixel_values, grid_thw=grid_thw)
out = out.pooler_output if hasattr(out, "pooler_output") else out
arr = out.float().numpy()
np.save(f"{REF}/vision_embeds_f32.npy", arr)
print("vision_embeds_f32:", arr.shape, flush=True)

ref = np.load(f"{REF}/vision_embeds.npy")
a, b = arr.ravel(), ref.ravel()
cos = float(np.dot(a, b) / (np.linalg.norm(a) * np.linalg.norm(b)))
print(f"f32 vs bf16 rejeleo: max_abs={np.abs(a-b).max():.6f} cosine={cos:.6f}", flush=True)
print("TAYARI", flush=True)
