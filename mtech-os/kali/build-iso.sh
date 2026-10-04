#!/usr/bin/env bash
# MTECH OS — jenga ISO ya Kali Linux yenye akili (Qwen 2.5 VL 3B + MTAALAMU SMART)
#
#   sudo apt install -y live-build rsync        # mara moja tu (kwenye Kali/Debian)
#   ./mtech-os/kali/build-iso.sh                # ISO → mtech-os/build/kali-work/images/
#
# Msingi: files rasmi za Kali (upstream/kali-live — gitlab.com/kalilinux/build-scripts/kali-live)
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$HERE/../.." && pwd)"
KALI_SRC="${KALI_SRC:-$REPO_ROOT/upstream/kali-live}"
WORK="$REPO_ROOT/mtech-os/build/kali-work"
INC="kali-config/variant-mtech/includes.chroot"

[ -d "$KALI_SRC" ] || { echo "Hakuna $KALI_SRC — clone: git clone --depth 1 https://gitlab.com/kalilinux/build-scripts/kali-live.git upstream/kali-live" >&2; exit 1; }
[ "$(id -u)" -eq 0 ] || { echo "Endesha kama root (sudo) — live-build inahitaji." >&2; exit 1; }
command -v lb >/dev/null || { echo "saki: sudo apt install -y live-build rsync" >&2; exit 1; }

echo "[MTECH] 1/4 — nakili kali-live → $WORK"
mkdir -p "$(dirname "$WORK")"
rm -rf "$WORK"
rsync -a --exclude .git "$KALI_SRC/" "$WORK/"

echo "[MTECH] 2/4 — weka variant-mtech"
rm -rf "$WORK/kali-config/variant-mtech"
mkdir -p "$WORK/kali-config/variant-mtech"
cp -r "$HERE/variant-mtech/." "$WORK/kali-config/variant-mtech/"

echo "[MTECH] 3/4 — pandisha MTECH layer + akili ya MTAALAMU kwenye ISO"
mkdir -p "$WORK/$INC/opt/mtech" "$WORK/$INC/opt/mtaalamu/hermes-agent"
rsync -a "$REPO_ROOT/mtech-os/agent/"   "$WORK/$INC/opt/mtech/agent/"
rsync -a "$REPO_ROOT/mtech-os/gui/"     "$WORK/$INC/opt/mtech/gui/"
rsync -a "$REPO_ROOT/mtech-os/services/" "$WORK/$INC/opt/mtech/services/"
rsync -a "$REPO_ROOT/mtech-os/kernel/"  "$WORK/$INC/opt/mtech/kernel/"
rsync -a --exclude 'target/' --exclude '__pycache__/' \
      "$REPO_ROOT/hermes-agent/data/"        "$WORK/$INC/opt/mtaalamu/hermes-agent/data/"
rsync -a --exclude 'target/' --exclude '__pycache__/' \
      "$REPO_ROOT/hermes-agent/engine-rust/" "$WORK/$INC/opt/mtaalamu/hermes-agent/engine-rust/"

# --- wrapper ya CLI: mtech ---
mkdir -p "$WORK/$INC/usr/local/bin" "$WORK/$INC/usr/local/sbin" "$WORK/$INC/etc/xdg/autostart"
cat > "$WORK/$INC/usr/local/bin/mtech" <<'EOF'
#!/bin/sh
export MTECH_ROOT=/opt/mtech
export MTAALAMU_ROOT=/opt/mtaalamu/hermes-agent
export PYTHONPATH=/opt/mtech/agent
exec /opt/mtech/venv/bin/python -m mtech_agent "$@"
EOF
chmod +x "$WORK/$INC/usr/local/bin/mtech"

# --- firstboot: vuta Qwen 2.5 VL 3B mara ya kwanza (mtandao unahitajika) ---
install -D -m 755 "$REPO_ROOT/mtech-os/services/mtech-firstboot.sh" \
    "$WORK/$INC/usr/local/sbin/mtech-firstboot"

# --- autostart ya GUI (MTECH Shell) kwenye kila session ya XFCE ---
cat > "$WORK/$INC/etc/xdg/autostart/mtech-shell.desktop" <<'EOF'
[Desktop Entry]
Type=Application
Name=MTECH Shell
Comment=GUI ya kipekee ya MTECH OS (wrapper ya desktop)
Exec=/opt/mtech/venv/bin/python /opt/mtech/gui/mtech_shell.py
Terminal=false
Categories=System;
X-GNOME-Autostart-enabled=true
EOF

# --- release tag ---
echo 'PRETTY_NAME="MTECH OS 0.1 (Kali) — Qwen 2.5 VL 3B + MTAALAMU SMART"' \
    > "$WORK/$INC/etc/mtech-os-release"

echo "[MTECH] 4/4 — live-build (hii inachukua muda; ISO → $WORK/images/)"
cd "$WORK"
./build.sh --variant mtech --verbose
