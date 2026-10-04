#!/usr/bin/env bash
# MTECH OS — build kernel kutoka upstream/linux (torvalds) + module ya /dev/mtech
#
#   ./build-kernel.sh                  # kernel kamili + module (dakika ~10-40)
#   ./build-kernel.sh --module-only    # module tu dhidi ya kernel inayoendesha
#   ./build-kernel.sh --install        # ongeza: sudo make modules_install install
#
# Msingi: https://github.com/torvalds/linux (imepakwa upstream/linux)
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$HERE/../.." && pwd)"
LINUX="${LINUX:-$REPO_ROOT/upstream/linux}"
MODULE_ONLY=0
DO_INSTALL=0

for arg in "$@"; do
  case "$arg" in
    --module-only) MODULE_ONLY=1 ;;
    --install)     DO_INSTALL=1 ;;
    *) echo "hoja haijulikani: $arg" >&2; exit 2 ;;
  esac
done

echo "[MTECH] kernel tree: $LINUX"
[ -d "$LINUX" ] || { echo "Hakuna $LINUX — paka kernel: git clone --depth 1 https://github.com/torvalds/linux.git upstream/linux" >&2; exit 1; }

# --- Vitega vya ujenzi (Kali/Debian) ---
NEED=(make gcc flex bison bc libelf-dev libssl-dev)
MISSING=()
for p in "${NEED[@]}"; do
  case "$p" in
    libelf-dev|libssl-dev) dpkg -s "$p" >/dev/null 2>&1 || MISSING+=("$p") ;;
    *) command -v "$p" >/dev/null 2>&1 || MISSING+=("$p") ;;
  esac
done
if [ "${#MISSING[@]}" -gt 0 ]; then
  echo "[MTECH] vinahitajika: sudo apt install -y ${MISSING[*]}"
  echo "        Endesha hiyo kisha jaribu tena." >&2
  exit 1
fi

build_module() {
  local kdir="$1"
  echo "[MTECH] building module: mtech_dev.ko (KDIR=$kdir)"
  make -C "$kdir" M="$HERE" modules
  echo "[MTECH] OK → $HERE/mtech_dev.ko"
  echo "        Load:  sudo insmod $HERE/mtech_dev.ko"
  echo "        Ona:   cat /proc/mtech_status && cat /dev/mtech"
}

if [ "$MODULE_ONLY" -eq 1 ]; then
  build_module "/lib/modules/$(uname -r)/build"
  exit 0
fi

# --- 1) Config: kali/defconfig + fragment ya MTECH ---
cd "$LINUX"
echo "[MTECH] config: x86_64 defconfig + mtech.config"
make defconfig
"$LINUX/scripts/kconfig/merge_config.sh" -m .config "$HERE/mtech.config"
make olddefconfig

# --- 2) Kernel + modules ---
JOBS="$(nproc)"
echo "[MTECH] make -j$JOBS bzImage modules (subiri...)"
make -j"$JOBS" bzImage modules

if [ "$DO_INSTALL" -eq 1 ]; then
  echo "[MTECH] sudo make modules_install install ..."
  sudo make modules_install install
  echo "[MTECH] Kernel imefungwa — restart ili kuingia MTECH kernel."
fi

# --- 3) Module ya /dev/mtech dhidi ya kernel hii mpya ---
build_module "$LINUX"
echo "[MTECH] kernel tayari: $LINUX/arch/x86/boot/bzImage"
