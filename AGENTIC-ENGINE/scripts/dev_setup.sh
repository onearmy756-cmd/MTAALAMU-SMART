#!/usr/bin/env bash
# Huweka mazingira ya MTECH OS / MBILINYI AKILI kwenye Debian/Kali.
#
# Hufanya: Rust (rustup), Python pkgs (numpy/safetensors/pillow/transformers/torch),
# Go, Elixir, na hupakua modeli ikiwa haipo.
#
# Matumizi:
#   scripts/dev_setup.sh [model_repo] [model_dir]
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REPO="${1:-Qwen/Qwen2.5-VL-3B-Instruct}"
MODEL_DIR="${2:-${ROOT}/models/Qwen2.5-VL-3B-Instruct}"

export PATH="${HOME}/.cargo/bin:/opt/cargo/bin:${HOME}/.local/bin:${PATH}"
export CARGO_HOME="${CARGO_HOME:-/opt/cargo}"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-/opt/target}"
if ! mkdir -p "${CARGO_HOME}" "${CARGO_TARGET_DIR}" 2>/dev/null; then
  sudo mkdir -p "${CARGO_HOME}" "${CARGO_TARGET_DIR}" && sudo chown -R "$(id -u):$(id -g)" /opt
fi

step() { printf '\n=== %s ===\n' "$*"; }

step "Rust"
if ! command -v cargo >/dev/null 2>&1; then
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs |
    sh -s -- -y --profile minimal --default-toolchain stable --no-modify-path
else
  cargo --version
fi

step "Python"
if ! python3 -c "import numpy, safetensors, PIL, transformers, torch" 2>/dev/null; then
  pip install --quiet --no-input numpy safetensors pillow accelerate transformers
  pip install --quiet --no-input torch torchvision --index-url https://download.pytorch.org/whl/cpu
fi
python3 - <<'PY'
import numpy, PIL, safetensors, torch, transformers
print("numpy", numpy.__version__, "torch", torch.__version__, "transformers", transformers.__version__)
PY

step "Go"
if ! command -v go >/dev/null 2>&1; then
  sudo apt-get update -qq
  sudo DEBIAN_FRONTEND=noninteractive apt-get install -y -qq golang-go
fi
go version 2>/dev/null || echo "go haipo"

step "Elixir"
if ! command -v elixir >/dev/null 2>&1; then
  sudo apt-get update -qq
  sudo DEBIAN_FRONTEND=noninteractive apt-get install -y -qq elixir erlang-base
fi
elixir --version 2>/dev/null | tail -1 || echo "elixir haipo"

step "Modeli"
if [[ -s "${MODEL_DIR}/config.json" ]] && compgen -G "${MODEL_DIR}/*.safetensors" >/dev/null; then
  echo "ipo: ${MODEL_DIR}"
  du -sh "${MODEL_DIR}"
else
  "${ROOT}/scripts/fetch_model.sh" "${REPO}" "${MODEL_DIR}"
fi

step "Rust build"
(cd "${ROOT}/rust" && cargo build -p akilid 2>&1 | tail -3)

step "Go build"
(cd "${ROOT}/bridge/go" && go build ./... && echo "go build ok")

printf '\n[dev_setup] imekamilika\n'
printf '  AKILI_MODEL=%s\n' "${MODEL_DIR}"
printf '  akilid=%s\n' "${CARGO_TARGET_DIR}/debug/akilid"
