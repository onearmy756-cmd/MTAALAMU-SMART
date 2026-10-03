#!/usr/bin/env python3
"""ai_tensorflow.py — Bridge ya TensorFlow kwa MTAALAMU SMART SkillsEngine.

Inaitwa na Rust (skills-run --skill ai.tensorflow --approve) au moja kwa moja:
    python3 scripts/ai_tensorflow.py --task "demo"
    python3 scripts/ai_tensorflow.py --task "train" --epochs 3

Inafanya kazi HALISI na tensorflow (CPU) — hakuna data ya uongo:
  - demo:  neural network ndogo inayojifunza square function
  - train: regression + report (loss before/after)
Output: JSON kwenye mstari wa mwisho (Rust inasoma).
"""
import argparse
import json
import sys
import time

OUT = {"bridge": "tensorflow", "task": "demo", "ok": False}


def main():
    p = argparse.ArgumentParser()
    p.add_argument("--task", default="demo")
    p.add_argument("--epochs", type=int, default=20)
    args = p.parse_args()
    OUT["task"] = args.task

    t0 = time.time()
    try:
        import numpy as np
        import tensorflow as tf
    except ImportError:
        OUT["error"] = "tensorflow haipo. Sakinisha: pip3 install tensorflow-cpu"
        print(json.dumps(OUT))
        sys.exit(1)

    OUT["tf_version"] = tf.__version__
    tf.random.set_seed(7)

    # Data halisi: kujifunza muda = 30 + 2*x (formula ya skills!)
    X = np.linspace(1, 20, 40, dtype=np.float32).reshape(-1, 1)
    y = 30.0 + 2.0 * X + 0.5 * np.random.randn(40, 1).astype(np.float32)

    model = tf.keras.Sequential(
        [
            tf.keras.layers.Input(shape=(1,)),
            tf.keras.layers.Dense(16, activation="relu"),
            tf.keras.layers.Dense(1),
        ]
    )
    model.compile(optimizer=tf.keras.optimizers.Adam(0.05), loss="mse")

    hist = model.fit(X, y, epochs=max(1, args.epochs), verbose=0)
    losses = hist.history["loss"]
    first_loss = float(losses[0])
    final_loss = float(losses[-1])
    pred = float(model.predict(np.array([[10.0]], dtype=np.float32), verbose=0)[0][0])
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
            "summary_sw": f"TensorFlow: loss imepungua {improved}% kwa epochs {args.epochs}",
        }
    )
    print(json.dumps(OUT))


if __name__ == "__main__":
    main()
