#!/usr/bin/env bash
# MTECH OS — jenga ISO HALISI inayofanya kazi (Kali Linux kamili + MTAALAMU SMART)
#
# NJIA TATU (zote zinatengeneza ISO halisi, bootable):
#
#   1) Kali halisi (rahisi zaidi):
#        sudo apt install -y live-build rsync
#        sudo ./mtech-os/kali/build-iso.sh
#
#   2) Docker (kutoka OS yoyote: Ubuntu/Debian/Arch/Mac/Windows+WSL2):
#        ./mtech-os/kali/build-iso.sh --container
#      (inatumia build-in-container.sh RASMI ya Kali — config + privileges
#       zote halisi za kalilinux/build-scripts/kali-live)
#
#   3) Podman:  CONTAINER=podman ./mtech-os/kali/build-iso.sh --container
#
# CHAGUO:
#   --everything    zana ZOTE za Kali (~1800) badala ya kali-linux-large (~400+)
#   --arch ARCH     usanifu wa ISO: amd64 (chaguomsingi), arm64 (simu/ Raspberry Pi), armhf
#                   — ARM64: ISO inafanya kazi kwenye simu/PC zote za 64-bit ARM
#   --container     jenga ndani ya container rasmi ya Kali (docker/podman)
#   --check         hakikishi packages zote dhidi ya repo halisi ya Kali (bila kujenga)
#   --output DIR    mahali pa ISO (chaguomsingi: mtech-os/build/images)
#
# DOCKER (ISO haijajengwa? — MTECH OS kama container, OS zote):
#   docker build -t mtech-os -f mtech-os/kali/Dockerfile.mtech .
#   ./mtech-os/kali/docker-run.sh            # container ya MTECH OS + API 8795
# Chanzo: upstream/kali-live (gitlab.com/kalilinux/build-scripts/kali-live RASMI)
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$HERE/../.." && pwd)"
KALI_SRC="${KALI_SRC:-$REPO_ROOT/upstream/kali-live}"
WORK="${WORK:-$REPO_ROOT/mtech-os/build/kali-work}"
OUT_DIR=""
INC="kali-config/variant-mtech/includes.chroot"
EVERYTHING=0
USE_CONTAINER=0
CHECK_ONLY=0
ARCH="amd64"

while [ $# -gt 0 ]; do
  case "$1" in
    --everything) EVERYTHING=1; shift ;;
    --container)  USE_CONTAINER=1; shift ;;
    --check)      CHECK_ONLY=1; shift ;;
    --arch)       ARCH="$2"; shift 2 ;;
    --output)     OUT_DIR="$(realpath -m "$2")"; shift 2 ;;
    *) echo "hoja haijulikani: $1" >&2; exit 2 ;;
  esac
done

case "$ARCH" in
  amd64|x86_64) ARCH=amd64 ;;
  arm64|aarch64) ARCH=arm64 ;;
  armhf) ARCH=armhf ;;
  *) echo "--arch: tumia amd64, arm64 au armhf (ulitoa: $ARCH)" >&2; exit 2 ;;
esac

[ -d "$KALI_SRC" ] || {
  echo "Hakuna $KALI_SRC — clone kwanza:" >&2
  echo "  git clone --depth 1 https://gitlab.com/kalilinux/build-scripts/kali-live.git upstream/kali-live" >&2
  exit 1
}

# --- ukaguzi wa mapema wa vitega (ujumbe sahihi badala ya kufeli kati ya kazi) ---
for dep in rsync curl; do
  command -v "$dep" >/dev/null 2>&1 || { echo "saki: sudo apt install -y $dep" >&2; exit 1; }
done
if [ "$USE_CONTAINER" -eq 1 ]; then
  command -v docker >/dev/null 2>&1 || command -v podman >/dev/null 2>&1 || {
    echo "saki: docker au podman (kwa --container)" >&2; exit 1
  }
else
  [ "$(id -u)" -eq 0 ] || { echo "Endesha kama root (sudo) au tumia --container." >&2; exit 1; }
  command -v lb >/dev/null 2>&1 || { echo "saki: sudo apt install -y live-build" >&2; exit 1; }
fi

# ------------------------------------------------------------------ prepare
prepare_variant() {
  echo "[MTECH] 1/4 — nakili kali-live → $WORK"
  mkdir -p "$(dirname "$WORK")"
  rm -rf "$WORK"
  rsync -a --exclude .git "$KALI_SRC/" "$WORK/"

  echo "[MTECH] 2/4 — weka variant-mtech (OS halisi ya Kali + app ya MTAALAMU)"
  rm -rf "$WORK/kali-config/variant-mtech"
  mkdir -p "$WORK/kali-config/variant-mtech"
  cp -r "$HERE/variant-mtech/." "$WORK/kali-config/variant-mtech/"

  if [ "$EVERYTHING" -eq 1 ]; then
    echo "[MTECH]    --everything: kali-linux-large → kali-linux-everything"
    sed -i 's/^kali-linux-large$/kali-linux-everything/' \
      "$WORK/kali-config/variant-mtech/package-lists/mtech.list.chroot"
  fi

  echo "[MTECH] 3/4 — pandisha MTAALAMU SMART (app + akili + apps za mfumo) kwenye ISO"
  mkdir -p "$WORK/$INC/opt/mtech" "$WORK/$INC/opt/mtaalamu/hermes-agent" \
           "$WORK/$INC/opt/mtech/desktop"
  rsync -a "$REPO_ROOT/mtech-os/agent/"    "$WORK/$INC/opt/mtech/agent/"
  rsync -a "$REPO_ROOT/mtech-os/gui/"      "$WORK/$INC/opt/mtech/gui/"
  rsync -a "$REPO_ROOT/mtech-os/services/" "$WORK/$INC/opt/mtech/services/"
  rsync -a "$REPO_ROOT/mtech-os/kernel/"   "$WORK/$INC/opt/mtech/kernel/"
  rsync -a --exclude 'target/' --exclude '__pycache__/' \
        "$REPO_ROOT/hermes-agent/data/"        "$WORK/$INC/opt/mtaalamu/hermes-agent/data/"
  rsync -a --exclude 'target/' --exclude '__pycache__/' \
        "$REPO_ROOT/hermes-agent/engine-rust/" "$WORK/$INC/opt/mtaalamu/hermes-agent/engine-rust/"
  rsync -a --exclude '__pycache__/' \
        "$REPO_ROOT/hermes-agent/mtaalamu/"    "$WORK/$INC/opt/mtaalamu/hermes-agent/mtaalamu/"
  rsync -a --exclude 'node_modules/' \
        "$REPO_ROOT/hermes-agent/web-html/"    "$WORK/$INC/opt/mtaalamu/hermes-agent/web-html/"
  rsync -a "$REPO_ROOT/hermes-agent/analytics-r/" "$WORK/$INC/opt/mtaalamu/hermes-agent/analytics-r/"
  rsync -a "$REPO_ROOT/hermes-agent/web-r/"       "$WORK/$INC/opt/mtaalamu/hermes-agent/web-r/"
  # Apps za mfumo (OpenMRS, Home Assistant, Website, Desktop) — kama folda zipo
  # (branch ya slim imezifuta; ISO inajengwa bila hizo na /api/apps inaonyesha haipo)
  for app in openmrs home-assistant website; do
    [ -d "$REPO_ROOT/hermes-agent/$app" ] && rsync -a "$REPO_ROOT/hermes-agent/$app/" \
      "$WORK/$INC/opt/mtaalamu/hermes-agent/$app/"
  done
  if [ -d "$REPO_ROOT/hermes-agent/apps" ]; then
    rsync -a --exclude 'node_modules/' --exclude 'dist/' --exclude 'build/' \
      "$REPO_ROOT/hermes-agent/apps/" "$WORK/$INC/opt/mtaalamu/hermes-agent/apps/"
  fi

  # --- Engine ya Rust: kama cargo ipo kwenye HOST, jenga + nakili binary halisi ---
  if command -v cargo >/dev/null 2>&1; then
    echo "[MTECH]    Rust engine: cargo build --release…"
    (cd "$REPO_ROOT/hermes-agent/engine-rust" && cargo build --release --quiet) \
      && install -D -m 755 "$REPO_ROOT/hermes-agent/engine-rust/target/release/mtaalamu" \
           "$WORK/$INC/usr/local/bin/mtaalamu-engine" \
      && echo "[MTECH]    ✔ engine ya Rust (binary) iko ndani ya ISO" \
      || echo "[MTECH]    ⚠ engine haikujengwa — hook ya ISO itaijenga chroot"
  else
    echo "[MTECH]    (cargo haipo kwenye host — hook ya ISO itajenga engine chroot)"
  fi

  # --- App ya MTAALAMU SMART: menyu ya Applications + icon ya Desktop ---
  install -D -m 755 "$REPO_ROOT/mtech-os/desktop/mtaalamu-app"     "$WORK/$INC/usr/local/bin/mtaalamu-app"
  install -D -m 644 "$REPO_ROOT/mtech-os/desktop/mtaalamu.desktop" "$WORK/$INC/usr/share/applications/mtaalamu.desktop"
  install -D -m 644 "$REPO_ROOT/mtech-os/desktop/mtaalamu.svg"     "$WORK/$INC/usr/share/icons/hicolor/scalable/apps/mtaalamu.svg"
  install -D -m 644 "$REPO_ROOT/mtech-os/desktop/mtaalamu.desktop" "$WORK/$INC/etc/skel/Desktop/mtaalamu.desktop"

  # --- Dirisha "Kali Tools" (search + categories 11 za zana halisi) ---
  install -D -m 755 "$REPO_ROOT/mtech-os/desktop/kali-tools"           "$WORK/$INC/usr/local/bin/kali-tools"
  install -D -m 644 "$REPO_ROOT/mtech-os/desktop/kali-tools.desktop"   "$WORK/$INC/usr/share/applications/kali-tools.desktop"
  install -D -m 644 "$REPO_ROOT/mtech-os/desktop/kali-tools-window.py" "$WORK/$INC/opt/mtech/desktop/kali-tools-window.py"

  # --- Dirisha "System Apps" (OpenMRS, Home Assistant, Web R, Desktop, Website…) ---
  install -D -m 755 "$REPO_ROOT/mtech-os/desktop/system-apps"              "$WORK/$INC/usr/local/bin/system-apps"
  install -D -m 644 "$REPO_ROOT/mtech-os/desktop/system-apps.desktop"      "$WORK/$INC/usr/share/applications/system-apps.desktop"
  install -D -m 644 "$REPO_ROOT/mtech-os/desktop/system-apps-window.py"    "$WORK/$INC/opt/mtech/desktop/system-apps-window.py"

  # --- wrapper ya CLI: mtech ---
  install -d -m 755 "$WORK/$INC/usr/local/bin"
  cat > "$WORK/$INC/usr/local/bin/mtech" <<'EOF'
#!/bin/sh
export MTECH_ROOT=/opt/mtech
export MTAALAMU_ROOT=/opt/mtaalamu/hermes-agent
export PYTHONPATH=/opt/mtech/agent
exec /opt/mtech/venv/bin/python -m mtech_agent "$@"
EOF
  chmod +x "$WORK/$INC/usr/local/bin/mtech"

  # --- release tag ---
  install -d -m 755 "$WORK/$INC/etc"
  echo 'PRETTY_NAME="MTECH OS (Kali Rolling) — MTAALAMU SMART ndani"' \
      > "$WORK/$INC/etc/mtech-os-release"
}

# ------------------------------------------------------------------ check
# --check: hakikisha kila package ya mtech.list.chroot ipo kwenye repo halisi ya Kali.
# Inafanya kazi popote (inapakua indexes ZOTE 4 za kali-rolling: main, contrib,
# non-free, non-free-firmware — firmware zipo kwenye non-free-firmware).
check_packages() {
  echo "[MTECH] --check: kupakua indexes za kali-rolling (main/contrib/non-free/non-free-firmware)..."
  local idx; idx=$(mktemp)
  trap 'rm -f "${idx:-}"' EXIT
  local comp
  for comp in main contrib non-free non-free-firmware; do
    curl -fsSL "https://kali.download/kali/dists/kali-rolling/${comp}/binary-amd64/Packages.gz" | gzip -d >> "$idx" \
      || { echo "IMEKOSA: kuipakua index ya ${comp} (mtandao?)" >&2; exit 1; }
  done
  local missing=0 total=0
  local list="$HERE/variant-mtech/package-lists/mtech.list.chroot"
  while IFS= read -r pkg; do
    pkg="${pkg%%#*}"; pkg="$(echo "$pkg" | tr -d '[:space:]')"
    [ -z "$pkg" ] && continue
    case "$pkg" in \#*|'') continue ;; esac
    total=$((total+1))
    if ! grep -q "^Package: ${pkg}\$" "$idx"; then
      echo "  HAYAPO: $pkg"
      missing=$((missing+1))
    fi
  done < "$list"
  echo "[MTECH] --check: packages $total, zilizokosekana: $missing"
  [ "$missing" -eq 0 ] && echo "[OK] Packages zote za MTECH zipo kwenye Kali rolling" || exit 1
}

if [ "$CHECK_ONLY" -eq 1 ]; then
  check_packages
  exit 0
fi

prepare_variant

echo "[MTECH] 4/4 — BUILD HALISI ya ISO"
if [ "$USE_CONTAINER" -eq 1 ]; then
  echo "[MTECH] njia: container rasmi ya Kali (build-in-container.sh) — arch: $ARCH"
  cd "$WORK"
  # build-in-container.sh inapitisha hoja kwa build.sh (--variant, --arch n.k.)
  ./build-in-container.sh --variant mtech --arch "$ARCH" --verbose
  echo "[MTECH] ISO → $WORK/output/ (au $OUT_DIR kama uliweka --output)"
else
  cd "$WORK"
  ./build.sh --variant mtech --arch "$ARCH" --verbose
  echo "[MTECH] ISO ($ARCH) → $WORK/images/"
fi
