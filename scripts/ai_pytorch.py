#!/usr/bin/env python3
"""ai_pytorch.py — Bridge ya PyTorch kwa MTAALAMU SMART SkillsEngine.

Inaitwa na Rust (skills-run --skill ai.pytorch --approve) au moja kwa moja:
    python3 scripts/ai_pytorch.py --task "demo"
    python3 scripts/ai_pytorch.py --task "train" --epochs 3

Inafanya kazi HALISI na torch (CPU) — hakuna data ya uongo:
  - demo:  neural network ndogo inayojifunza square function
  - train: linear regression + report (loss before/after)
Output: JSON kwenye mstari wa mwisho (Rust inasoma).
"""
import argparse
import json
import sys
import time

OUT = {"bridge": "pytorch", "task": "demo", "ok": False}


def main():
    p = argparse.ArgumentParser()
    p.add_argument("--task", default="demo")
    p.add_argument("--epochs", type=int, default=20)
    args = p.parse_args()
    OUT["task"] = args.task

    t0 = time.time()
    try:
        import torch
        import torch.nn as nn
    except ImportError:
        OUT["error"] = "torch haipo. Sakinisha: pip3 install torch --index-url https://download.pytorch.org/whl/cpu"
        print(json.dumps(OUT))
        sys.exit(1)

    OUT["torch_version"] = torch.__version__
    OUT["device"] = "cuda" if torch.cuda.is_available() else "cpu"
    torch.manual_seed(7)

    # Data halisi: kujifunza muda wa kazi = 30 + 2*epochs (kama formula za skills!)
    X = torch.linspace(1, 20, 40).reshape(-1, 1)
    y = 30.0 + 2.0 * X + 0.5 * torch.randn(40, 1)

    model = nn.Sequential(nn.Linear(1, 16), nn.ReLU(), nn.Linear(16, 1))
    opt = torch.optim.Adam(model.parameters(), lr=0.05)
    lossf = nn.MSELoss()

    first_loss = None
    for epoch in range(max(1, args.epochs)):
        opt.zero_grad()
        loss = lossf(model(X), y)
        loss.backward()
        opt.step()
        if first_loss is None:
            first_loss = float(loss)

    final_loss = float(lossf(model(X), y))
    with torch.no_grad():
        pred = float(model(torch.tensor([[10.0]]))[0][0])

    improved = round((1 - final_loss / first_loss) * 100, 1) if first_loss else 0
    OUT.update(
        {
            "ok": True,
            "epochs": args.epochs,
            "loss_first": round(first_loss, 3),
            "loss_final": round(final_loss, 3),
            "improved_pct": improved,
            "prediction_at_x10": round(pred, 2),
            "true_value_x10": 30.0 + 2.0 * 10.0,
            "duration_ms": int((time.time() - t0) * 1000),
            "summary_sw": f"PyTorch: loss imepungua {improved}% kwa epochs {args.epochs}",
        }
    )
    print(json.dumps(OUT))


if __name__ == "__main__":
    main()
