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
check test -f web-r/R/iot.R
check test -f data/agents/agents_10.json || check test -f web-r/data/agents/agents_10.json

# IoT + HERMES (afya · usalama · nyumbani · kilimo)
check test -f data/iot/verticals.json
check test -f data/iot/hermes.json
check test -f data/iot/devices.json
check test -f data/iot/agents_oss.json
check test -f data/iot/telemetry_rules.json
check test -f data/iot/voice_fst.json
check test -f data/iot/registry/afya.json
check test -f data/iot/registry/usalama.json
check test -f data/iot/registry/nyumbani.json
check test -f data/iot/registry/kilimo.json
check test -f scripts/clone_iot_repos.sh
check test -f services/hermes_gateway/mix.exs
check test -f services/hermes_agents/worker.py

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
