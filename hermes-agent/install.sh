#!/usr/bin/env sh
# MTAALAMU SMART — SETU (bonyeza install: kila kitu kwa amri moja)
#
#   sh ./install.sh                 # kila kitu: C, Rust, R, Ollama+Qwen, venv, register+admin
#   sh ./install.sh --check         # angalia tu — usisakinise chochote
#   sh ./install.sh --skip-rust --skip-r --no-model
#
# Kumbuka: hosting/Node images hazina rust/R — hii ni kwa computer yako halisi.
set -eu

HERE="$(cd "$(dirname "$0")" && pwd)"
PY=python3
command -v python3 >/dev/null 2>&1 || PY=python

cd "$HERE"   # python -m mtaalamu inahitaji kuwa hermes-agent ndani ya cwd

echo "══ MTAALAMU SMART — SETU (one-click) ══"
"$PY" "$HERE/setup_all.py" "$@"

# Baada ya setup: admin unlock (owner-mode) — register inaulizwa na setup_all
if [ "$1" != "--check" ] 2>/dev/null; then
  "$PY" -m mtaalamu admin unlock 2>/dev/null && echo "👑 ADMIN umefunguliwa (owner-mode) — zana zote BURE." || true
fi
echo "Imekamilika. Endesha: $PY -m mtaalamu serve  kisha fungua web-html/mtaalamu-unified.html"
