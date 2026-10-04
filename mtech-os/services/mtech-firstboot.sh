#!/bin/sh
# MTECH OS firstboot: ollama pull qwen2.5vl:3b (+ module ya kernel ikiwa headers zipo)
set -u
LOG=/var/log/mtech-firstboot.log
MARKER=/var/lib/mtech/.bootstrapped
exec >>"$LOG" 2>&1
mkdir -p /var/lib/mtech
[ -f "$MARKER" ] && exit 0
echo "[MTECH-firstboot] $(date) kuanza"

if [ ! -x /usr/local/bin/ollama ]; then
    curl -fsSL -o /tmp/ollama.tgz https://ollama.com/download/ollama-linux-amd64.tgz \
        && tar -xzf /tmp/ollama.tgz -C /usr/local && rm -f /tmp/ollama.tgz
fi

# subiri ollama api
for i in $(seq 1 30); do
    curl -fsS http://127.0.0.1:11434/ >/dev/null 2>&1 && break
    sleep 2
done

/usr/local/bin/ollama pull qwen2.5vl:3b && touch "$MARKER" \
    && echo "[MTECH-firstboot] modeli tayari" \
    || echo "[MTECH-firstboot] WARNING: modeli haijavutwa — endesha: ollama pull qwen2.5vl:3b"
