#!/usr/bin/env sh
# MTAALAMU SMART — SETU (bonyeza install: kila kitu, HAKUNA configuration)
#
#   sh ./install.sh                 # kila kitu: C, Rust, R, LibreOffice, BINARY (Python ndani),
#                                   #   Ollama+Qwen, account ya kiotomatiki, ADMIN, server + dashboard
#                                   #   — HAKUNA Python inayosakinishwa kwa chaguomsingi (binary-first)
#   sh ./install.sh --check         # angalia tu — usisakinise chochote
#   sh ./install.sh --with-agent    # PIA venv ya Python (MTECH agent vision/control + GUI)
#   sh ./install.sh --email me@mail.com --skip-rust --skip-r --no-model
#
# Kumbuka: hosting/Node images hazina rust/R — hii ni kwa computer yako halisi.
set -eu

HERE="$(cd "$(dirname "$0")" && pwd)"
PY=python3
command -v python3 >/dev/null 2>&1 || PY=python

cd "$HERE"   # python -m mtaalamu inahitaji kuwa hermes-agent ndani ya cwd

echo "══ MTAALAMU SMART — SETU (one-click) ══"
"$PY" "$HERE/setup_all.py" "$@"

# Baada ya setup: setup_all.py inafanya KILA KITU yenyewe (auto-register,
# admin unlock, server + dashboard) — hakuna configuration inayohitajika.
if [ "${1:-}" = "--check" ]; then
  echo "(UKAGUZI TU — hakuna kinachosakinishwa)"
else
  echo "Imekamilika! Dashboard iko browser yako (au: $PY -m mtaalamu serve)."
fi
