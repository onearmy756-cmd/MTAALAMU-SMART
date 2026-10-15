#!/usr/bin/env python3
"""Rejeleo la Qwen2.5-VL (transformers) kwa uthibitisho wa injini ya Rust.

Hutoa: input_ids, pixel_values, image_grid_thw, vision embeds, hidden states,
logits, na tokeni za greedy. Huandika `.npy` (na `manifest.json` baada ya
`ref_convert.py`).

Matumizi:
    python3 scripts/ref_dump.py <model_dir> <out_dir>
"""
import json
import os
import sys

import numpy as np
import torch
from PIL import Image, ImageDraw

MODEL = sys.argv[1] if len(sys.argv) > 1 else "/tmp/ref-model"
OUT = sys.argv[2] if len(sys.argv) > 2 else "/tmp/ref"
DTYPE = (sys.argv[3] if len(sys.argv) > 3 else "bf16").lower()
DTYPES = {"bf16": torch.bfloat16, "f32": torch.float32, "f16": torch.float16}
TORCH_DTYPE = DTYPES[DTYPE]
os.makedirs(OUT, exist_ok=True)
torch.manual_seed(0)
torch.set_grad_enabled(False)

from transformers import AutoProcessor, Qwen2_5_VLForConditionalGeneration

print(f"kupakia processor + model (dtype={DTYPE}) ...", flush=True)
proc = AutoProcessor.from_pretrained(MODEL)
model = Qwen2_5_VLForConditionalGeneration.from_pretrained(MODEL, dtype=TORCH_DTYPE)
model.eval()
cfg = model.config
print(
    "cfg:",
    cfg.text_config.num_hidden_layers if hasattr(cfg, "text_config") else cfg.num_hidden_layers,
    "layers;",
    cfg.vision_config.depth,
    "vision blocks;",
    flush=True,
)

# ---- picha ya majaribio (dhahania, ya kurudia) ----
img = Image.new("RGB", (640, 480), (20, 40, 80))
d = ImageDraw.Draw(img)
for i in range(480):
    d.line([(0, i), (640, i)], fill=(i % 256, (i * 2) % 256, 255 - (i % 256)))
d.rectangle([40, 40, 240, 200], fill=(220, 30, 30))
d.ellipse([300, 120, 560, 380], fill=(30, 220, 90))
d.polygon([(80, 400), (300, 400), (190, 260)], fill=(250, 220, 20))
img.save(f"{OUT}/test.png")
print("picha imehifadhiwa", img.size, flush=True)

TEXT_ONLY = "What is the capital of Tanzania? Answer in one short sentence."
IMAGE_Q = "Describe the main shapes and colors in this image."

# ---- maandishi pekee ----
msgs = [{"role": "user", "content": [{"type": "text", "text": TEXT_ONLY}]}]
text = proc.apply_chat_template(msgs, tokenize=False, add_generation_prompt=True)
enc = proc(text=text, return_tensors="pt")
input_ids = enc["input_ids"]
print("text tokens:", input_ids.shape, flush=True)
out = model(input_ids=input_ids, output_hidden_states=True)
np.save(f"{OUT}/text_logits.npy", out.logits[0, -1].float().numpy())
for li in (0, 1, 17):
    np.save(f"{OUT}/text_hidden_{li}.npy", out.hidden_states[li][0].float().numpy())
np.save(f"{OUT}/text_input_ids.npy", input_ids[0].numpy())
json.dump({"text": text}, open(f"{OUT}/text_prompt.json", "w"))
gen = model.generate(**enc, max_new_tokens=24, do_sample=False)
np.save(f"{OUT}/text_gen_ids.npy", gen[0].numpy())
print("text gen:", proc.tokenizer.decode(gen[0][input_ids.shape[1]:]), flush=True)
del out, gen, enc

# ---- picha ----
msgs = [{"role": "user", "content": [
    {"type": "image", "image": img},
    {"type": "text", "text": IMAGE_Q},
]}]
text = proc.apply_chat_template(msgs, tokenize=False, add_generation_prompt=True)
enc = proc(text=[text], images=[img], return_tensors="pt")
print("pixel_values:", enc["pixel_values"].shape, "grid:", enc["image_grid_thw"], flush=True)
np.save(f"{OUT}/img_input_ids.npy", enc["input_ids"][0].numpy())
np.save(f"{OUT}/pixel_values.npy", enc["pixel_values"].float().numpy())
np.save(f"{OUT}/image_grid_thw.npy", enc["image_grid_thw"].numpy())
json.dump({"text": text, "image_q": IMAGE_Q}, open(f"{OUT}/img_prompt.json", "w"))

pixel_values = enc["pixel_values"].to(TORCH_DTYPE)
grid_thw = enc["image_grid_thw"]
ve = model.model.visual(pixel_values, grid_thw=grid_thw)
ve = ve.pooler_output if hasattr(ve, "pooler_output") else ve
np.save(f"{OUT}/vision_embeds.npy", ve.float().numpy())
print("vision embeds:", ve.shape, flush=True)

out = model(**enc, output_hidden_states=True)
np.save(f"{OUT}/img_logits.npy", out.logits[0, -1].float().numpy())
np.save(f"{OUT}/img_hidden_0.npy", out.hidden_states[0][0].float().numpy())
del out

gen = model.generate(**enc, max_new_tokens=24, do_sample=False)
np.save(f"{OUT}/img_gen_ids.npy", gen[0].numpy())
print("img gen:", proc.tokenizer.decode(gen[0][enc["input_ids"].shape[1]:]), flush=True)

print("REJELEO_TAYARI", flush=True)
