#!/usr/bin/env bash
# MTECH OS — endesha container HALISI ya Kali + MTAALAMU (OS yoyote yenye Docker)
#
#   ./mtech-os/kali/docker-run.sh              # shell ya MTECH OS (arm64 AMA amd64 — Docker inachagua)
#   ./mtech-os/kali/docker-run.sh serve        # anzisha API ya moja kwa moja (port 8795)
#   ./mtech-os/kali/docker-run.sh build        # jenga image kwanza
#
# Ikiwa image haijajengwa bado: inajenga yenyewe (Dockerfile.mtech).
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$HERE/../.." && pwd)"
IMAGE="${MTECH_IMAGE:-mtech-os}"

command -v docker >/dev/null 2>&1 || { echo "saki: docker (au podman — badilisha docker→podman hapa chini)" >&2; exit 1; }

# --- build kama hakuna ---
if ! docker image inspect "$IMAGE" >/dev/null 2>&1; then
  echo "[MTECH] image '$IMAGE' haipo — inajenga (dakika chache)…"
  docker build -t "$IMAGE" -f "$HERE/Dockerfile.mtech" "$REPO_ROOT"
fi

# --- run ---
MODE="${1:-shell}"
case "$MODE" in
  build)
    docker build -t "$IMAGE" -f "$HERE/Dockerfile.mtech" "$REPO_ROOT" ;;
  serve)
    exec docker run -it --rm --name mtech-os -p 8795:8795 \
      -v "$HOME/.mtaalamu:/root/.mtaalamu" "$IMAGE" ;;
  shell|*)
    exec docker run -it --rm --name mtech-os -p 8795:8795 \
      -v "$HOME/.mtaalamu:/root/.mtaalamu" "$IMAGE" /bin/bash ;;
esac
