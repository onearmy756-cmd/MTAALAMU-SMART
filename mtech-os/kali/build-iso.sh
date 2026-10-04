#!/usr/bin/env bash
# MTECH OS — jenga ISO ya OS HALISI ya Kali Linux yenye MTAALAMU SMART kama app
#
#   sudo apt install -y live-build rsync        # mara moja tu (kwenye Kali/Debian)
#   sudo ./mtech-os/kali/build-iso.sh           # ISO → mtech-os/build/kali-work/images/
#   sudo ./mtech-os/kali/build-iso.sh --everything   # zana ZOTE za Kali (~1800, ISO kubwa)
#
# Msingi: files rasmi za Kali (upstream/kali-live — gitlab.com/kalilinux/build-scripts/kali-live)
# Muonekano: desktop halisi ya Kali (XFCE + dragon wallpaper + menyu za zana) —
#            MTAALAMU SMART ni APP tu ndani yake (menyu ya Applications + icon ya Desktop).
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$HERE/../.." && pwd)"
KALI_SRC="${KALI_SRC:-$REPO_ROOT/upstream/kali-live}"
WORK="$REPO_ROOT/mtech-os/build/kali-work"
INC="kali-config/variant-mtech/includes.chroot"
EVERYTHING=0

for arg in "$@"; do
  case "$arg" in
    --everything) EVERYTHING=1 ;;
    *) echo "hoja haijulikani: $arg" >&2; exit 2 ;;
  esac
done

[ -d "$KALI_SRC" ] || { echo "Hakuna $KALI_SRC — clone: git clone --depth 1 https://gitlab.com/kalilinux/build-scripts/kali-live.git upstream/kali-live" >&2; exit 1; }
[ "$(id -u)" -eq 0 ] || { echo "Endesha kama root (sudo) — live-build inahitaji." >&2; exit 1; }
command -v lb >/dev/null || { echo "saki: sudo apt install -y live-build rsync" >&2; exit 1; }

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

echo "[MTECH] 3/4 — pandisha MTAALAMU SMART (app + akili) kwenye ISO"
mkdir -p "$WORK/$INC/opt/mtech" "$WORK/$INC/opt/mtaalamu/hermes-agent"
rsync -a "$REPO_ROOT/mtech-os/agent/"    "$WORK/$INC/opt/mtech/agent/"
rsync -a "$REPO_ROOT/mtech-os/gui/"      "$WORK/$INC/opt/mtech/gui/"
rsync -a "$REPO_ROOT/mtech-os/services/" "$WORK/$INC/opt/mtech/services/"
rsync -a "$REPO_ROOT/mtech-os/kernel/"   "$WORK/$INC/opt/mtech/kernel/"
rsync -a --exclude 'target/' --exclude '__pycache__/' \
      "$REPO_ROOT/hermes-agent/data/"        "$WORK/$INC/opt/mtaalamu/hermes-agent/data/"
rsync -a --exclude 'target/' --exclude '__pycache__/' \
      "$REPO_ROOT/hermes-agent/engine-rust/" "$WORK/$INC/opt/mtaalamu/hermes-agent/engine-rust/"

# --- App ya MTAALAMU SMART: menyu ya Applications + icon ya Desktop (kama Nmap/Firefox) ---
install -D -m 755 "$REPO_ROOT/mtech-os/desktop/mtaalamu-app"    "$WORK/$INC/usr/local/bin/mtaalamu-app"
install -D -m 644 "$REPO_ROOT/mtech-os/desktop/mtaalamu.desktop" "$WORK/$INC/usr/share/applications/mtaalamu.desktop"
install -D -m 644 "$REPO_ROOT/mtech-os/desktop/mtaalamu.svg"     "$WORK/$INC/usr/share/icons/hicolor/scalable/apps/mtaalamu.svg"
# Icon ya Desktop (XFCE inaonyesha /usr/share/applications kwenye Desktop ya Kali;
# tunaweka pia .desktop kwenye ~/Desktop ya user wa kawaida kupitia skel)
install -D -m 644 "$REPO_ROOT/mtech-os/desktop/mtaalamu.desktop" "$WORK/$INC/etc/skel/Desktop/mtaalamu.desktop"

# --- Dirisha "Kali Tools" (kama picha): search + categories 11 za zana halisi ---
install -D -m 755 "$REPO_ROOT/mtech-os/desktop/kali-tools"           "$WORK/$INC/usr/local/bin/kali-tools"
install -D -m 644 "$REPO_ROOT/mtech-os/desktop/kali-tools.desktop"   "$WORK/$INC/usr/share/applications/kali-tools.desktop"
install -D -m 644 "$REPO_ROOT/mtech-os/desktop/kali-tools-window.py" "$WORK/$INC/opt/mtech/desktop/kali-tools-window.py"

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

echo "[MTECH] 4/4 — live-build (OS halisi ya Kali; inachukua muda; ISO → $WORK/images/)"
cd "$WORK"
./build.sh --variant mtech --verbose
