#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

echo "=== MTAALAMU SMART healthcheck ==="
FAIL=0

check() {
  if "$@"; then echo "[OK] $*"; else echo "[FAIL] $*"; FAIL=1; fi
}

check test -f engine-rust/Cargo.toml
check test -f web-r/app.R
check test -f web-r/R/sysprobe.R
check test -f web-r/R/agentic.R
check test -f data/agents/agents_10.json || check test -f web-r/data/agents/agents_10.json

if command -v cargo >/dev/null 2>&1; then
  (cd engine-rust && cargo build --release) || FAIL=1
  BIN="engine-rust/target/release/mtaalamu"
  if [[ -x "$BIN" ]] || [[ -f "$BIN.exe" ]]; then
    "$BIN" help >/dev/null || "$BIN.exe" help >/dev/null || FAIL=1
    echo "[OK] mtaalamu CLI"
    "$BIN" deep --top 3 >/dev/null 2>&1 || "$BIN.exe" deep --top 3 >/dev/null 2>&1 || echo "[WARN] deep probe runtime"
  fi
else
  echo "[WARN] cargo not installed — skip Rust build"
fi

if command -v Rscript >/dev/null 2>&1; then
  Rscript scripts/r_smoke.R || FAIL=1
else
  echo "[WARN] Rscript not installed — skip R smoke"
fi

if [[ "$FAIL" -eq 0 ]]; then
  echo "=== HEALTH PASS ==="
  exit 0
else
  echo "=== HEALTH FAIL ==="
  exit 1
fi
