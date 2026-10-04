#!/usr/bin/env bash
# MTECH OS — install MTAALAMU SMART kama APP kwenye OS iliyopo (Kali/Debian/Ubuntu)
#
#   sudo ./install.sh                  # app + huduma + Qwen 2.5 VL 3B
#   sudo ./install.sh --no-gui         # bila app ya GUI (CLI tu: mtech …)
#   sudo ./install.sh --no-model       # usivute modeli sasa (baadaye: ollama pull)
#   sudo ./install.sh --with-kernel    # build kernel ya MTECH kutoka upstream/linux
#   sudo ./install.sh --kali-look      # + panel/dock/conky/desktop icons kama Kali halisi
#
# Desktop yako haiguswi — MTAALAMU SMART inaonekana kama app tu:
#   menyu ya Applications → MTAALAMU SMART (na icon kwenye Desktop).
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$HERE/.." && pwd)"
DEST=/opt/mtech
DATA_DEST=/opt/mtaalamu
WITH_GUI=1
WITH_MODEL=1
WITH_KERNEL=0
KALI_LOOK=0

for arg in "$@"; do
  case "$arg" in
    --no-gui)      WITH_GUI=0 ;;
    --no-model)    WITH_MODEL=0 ;;
    --with-kernel) WITH_KERNEL=1 ;;
    --kali-look)   KALI_LOOK=1 ;;
    *) echo "hoja haijulikani: $arg" >&2; exit 2 ;;
  esac
done

[ "$(id -u)" -eq 0 ] || { echo "Endesha kwa sudo — inasakinisha kwenye /opt na systemd." >&2; exit 1; }

echo "════════════════════════════════════════"
echo " MTAALAMU SMART — install (app ndani ya OS yako)"
echo "════════════════════════════════════════"

# --- 1) Vitega vya mfumo ---
export DEBIAN_FRONTEND=noninteractive
apt-get update -qq
apt-get install -y -qq python3 python3-venv python3-pip python3-dev \
    curl xdotool wmctrl scrot xclip imagemagick rsync desktop-file-utils >/dev/null

# --- 2) Ollama (binary rasmi) ---
if [ ! -x /usr/local/bin/ollama ]; then
    echo "[1/6] kupakua Ollama…"
    curl -fsSL -o /tmp/ollama.tgz https://ollama.com/download/ollama-linux-amd64.tgz
    tar -xzf /tmp/ollama.tgz -C /usr/local && rm -f /tmp/ollama.tgz
else
    echo "[1/6] Ollama ipo tayari"
fi

# --- 3) Nakili app + akili ya MTAALAMU ---
echo "[2/6] nakili MTAALAMU SMART → $DEST (akili: $DATA_DEST)"
mkdir -p "$DEST" "$DATA_DEST/hermes-agent"
rsync -a "$HERE/agent/"    "$DEST/agent/"
rsync -a "$HERE/gui/"      "$DEST/gui/"
rsync -a "$HERE/services/" "$DEST/services/"
rsync -a "$HERE/kernel/"   "$DEST/kernel/"
rsync -a "$HERE/desktop/"  "$DEST/desktop/"
rsync -a --exclude 'target/' --exclude '__pycache__/' \
      "$REPO_ROOT/hermes-agent/data/"        "$DATA_DEST/hermes-agent/data/"
rsync -a --exclude 'target/' --exclude '__pycache__/' \
      "$REPO_ROOT/hermes-agent/engine-rust/" "$DATA_DEST/hermes-agent/engine-rust/"

# --- 4) venv + deps ---
echo "[3/6] venv ya app…"
python3 -m venv "$DEST/venv"
"$DEST/venv/bin/pip" install -q -r "$DEST/agent/requirements.txt"
if [ "$WITH_GUI" -eq 1 ]; then
    "$DEST/venv/bin/pip" install -q -r "$DEST/gui/requirements-gui.txt" || echo "  (WARNING: GUI deps zimeshindikana — endesha tena baadaye)"
fi

# --- 5) App ya desktop + CLI + systemd ---
echo "[4/6] app ya desktop (menyu + icon) na huduma…"
install -D -m 755 "$HERE/desktop/mtaalamu-app"     /usr/local/bin/mtaalamu-app
install -D -m 644 "$HERE/desktop/mtaalamu.desktop" /usr/share/applications/mtaalamu.desktop
install -D -m 644 "$HERE/desktop/mtaalamu.svg"     /usr/share/icons/hicolor/scalable/apps/mtaalamu.svg

# --- Dirisha "Kali Tools" (search + categories 11 kama Kali) ---
install -D -m 755 "$HERE/desktop/kali-tools"         /usr/local/bin/kali-tools
install -D -m 644 "$HERE/desktop/kali-tools.desktop" /usr/share/applications/kali-tools.desktop
install -D -m 644 "$HERE/desktop/kali-tools-window.py" "$DEST/desktop/kali-tools-window.py"
# Icon kwenye Desktop ya kila user (desktop icon kama picha ya Kali)
for d in /home/*/ /root/; do
    [ -d "$d" ] || continue
    u="$(basename "$d")"
    mkdir -p "$d/Desktop"
    install -m 644 "$HERE/desktop/mtaalamu.desktop" "$d/Desktop/mtaalamu.desktop"
    chown -R "${u}:${u}" "$d/Desktop/mtaalamu.desktop" 2>/dev/null || true
done
update-desktop-database /usr/share/applications 2>/dev/null || true

cat > /usr/local/bin/mtech <<'EOF'
#!/bin/sh
export MTECH_ROOT=/opt/mtech
export MTAALAMU_ROOT=/opt/mtaalamu/hermes-agent
export PYTHONPATH=/opt/mtech/agent
exec /opt/mtech/venv/bin/python -m mtech_agent "$@"
EOF
chmod +x /usr/local/bin/mtech

install -m 644 "$DEST/services/mtech-ollama.service"    /etc/systemd/system/
install -m 644 "$DEST/services/mtech-agent.service"     /etc/systemd/system/
install -m 644 "$DEST/services/mtech-firstboot.service" /etc/systemd/system/
install -m 755 "$HERE/services/mtech-firstboot.sh"      /usr/local/sbin/mtech-firstboot

# BOOT: MTAALAMU inaload kiotomatiki inapowaka OS (autostart ya kila user + icon)
install -D -m 644 "$HERE/boot/mtech-mtaalamu-autostart.desktop" /etc/xdg/autostart/mtech-mtaalamu.desktop
install -D -m 644 "$HERE/desktop/mtaalamu.svg" /usr/share/icons/hicolor/scalable/apps/mtaalamu.svg
install -D -m 644 "$HERE/boot/mtech-session.desktop" /usr/share/xsessions/mtech.desktop

systemctl daemon-reload
systemctl enable --now mtech-ollama.service
systemctl enable --now mtech-agent.service
systemctl enable mtech-firstboot.service

# --- 6) Modeli ya Qwen 2.5 VL 3B ---
if [ "$WITH_MODEL" -eq 1 ]; then
    echo "[5/6] ollama pull qwen2.5vl:3b (subiri…)"
    mkdir -p /var/lib/mtech
    /usr/local/bin/ollama pull qwen2.5vl:3b && touch /var/lib/mtech/.bootstrapped
else
    echo "[5/6] --no-model: endesha baadaye → ollama pull qwen2.5vl:3b"
fi

# --- 7) Kernel (hiari) ---
if [ "$WITH_KERNEL" -eq 1 ]; then
    echo "[6/6] kernel ya MTECH (build + install — dakika 10-40)…"
    bash "$HERE/kernel/build-kernel.sh" --install
else
    echo "[6/6] kernel: ruka (endesha → make -C mtech-os kernel)"
fi

# --- 8) Kali look (hiari): panel/dock/conky/icons kama picha ---
if [ "$KALI_LOOK" -eq 1 ]; then
    echo "[+] Kali look: panel + dock + conky + desktop icons…"
    apt-get install -y -qq conky-all xfce4-panel >/dev/null || true
    mkdir -p /etc/xdg/xfce4/xfconf/xfce-perchannel-xml
    install -m 644 "$HERE/kali/variant-mtech/includes.chroot/etc/xdg/xfce4/xfconf/xfce-perchannel-xml/xfce4-panel.xml" \
        /etc/xdg/xfce4/xfconf/xfce-perchannel-xml/xfce4-panel.xml
    install -m 644 "$HERE/kali/variant-mtech/includes.chroot/etc/xdg/xfce4/xfconf/xfce-perchannel-xml/xfce4-desktop.xml" \
        /etc/xdg/xfce4/xfconf/xfce-perchannel-xml/xfce4-desktop.xml
    install -m 644 "$HERE/kali/variant-mtech/includes.chroot/etc/skel/.conkyrc" /etc/skel/.conkyrc
    install -D -m 644 "$HERE/kali/variant-mtech/includes.chroot/etc/xdg/autostart/mtech-conky.desktop" \
        /etc/xdg/autostart/mtech-conky.desktop
    for d in /home/*/ /root/; do
        [ -d "$d" ] || continue
        u="$(basename "$d")"
        install -m 644 "$HERE/kali/variant-mtech/includes.chroot/etc/skel/.conkyrc" "$d/.conkyrc"
        chown "${u}:${u}" "$d/.conkyrc" 2>/dev/null || true
        mkdir -p "$d/Desktop"
        for f in mtaalamu.desktop kali-tools.desktop; do
            install -m 644 "$HERE/desktop/$f" "$d/Desktop/$f" 2>/dev/null || true
            chown "${u}:${u}" "$d/Desktop/$f" 2>/dev/null || true
        done
    done
    echo "[+] Kali look imewekwa — login tena (au: xfce4-panel -r) ili panel mpya ionekane."
fi

echo ""
echo "✅ MTAALAMU SMART imewekwa kama APP ndani ya OS yako!"
echo "   GUI:      menyu ya Applications → MTAALAMU SMART (au icon ya Desktop)"
echo "   Boot:     inaload KIOTOMATIKI inapowaka OS (+ session 'MTECH OS' kwenye login)"
echo "   CLI:      mtech skills | mtech ask \"tatizo la kompyuta\" | mtech watch"
echo "   Doctor:   mtech doctor   ← hakiki kila kitu kwenye computer hii"
echo "   Huduma:   systemctl status mtech-agent mtech-ollama"
echo "   Kernel:   cat /proc/mtech_status  (baada ya insmod mtech_dev.ko)"
echo "   Control:  mtech allow on  (full control — mouse/keyboard, mtumiaji akiruhusu)"
