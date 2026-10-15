#!/usr/bin/env bash
# Hupakua modeli kutoka Hugging Face (faili za config + tokenizer + safetensors).
#
# Matumizi:
#   scripts/fetch_model.sh <repo> <katalogi-lengo> [revision]
# Mfano:
#   scripts/fetch_model.sh Qwen/Qwen2.5-VL-7B-Instruct models/2NYI
set -euo pipefail

REPO="${1:?tumia: fetch_model.sh <repo> <katalogi> [revision]}"
DEST="${2:?tumia: fetch_model.sh <repo> <katalogi> [revision]}"
REV="${3:-main}"
BASE="https://huggingface.co/${REPO}/resolve/${REV}"

mkdir -p "$DEST"
FILES=(
  config.json
  generation_config.json
  preprocessor_config.json
  chat_template.json
  tokenizer.json
  tokenizer_config.json
  vocab.json
  merges.txt
  model.safetensors.index.json
)

echo "[fetch] ${REPO}@${REV} -> ${DEST}"
for f in "${FILES[@]}"; do
  if [[ -s "${DEST}/${f}" ]]; then
    echo "  [skip] ${f}"
    continue
  fi
  if curl -fsSL --retry 3 -o "${DEST}/${f}.part" "${BASE}/${f}"; then
    mv "${DEST}/${f}.part" "${DEST}/${f}"
    echo "  [ok]   ${f}"
  else
    rm -f "${DEST}/${f}.part"
    echo "  [--]   ${f} (haipo kwenye repo)"
  fi
done

# Orodha ya shards: kutoka index.json, au faili moja ya safetensors.
if [[ -s "${DEST}/model.safetensors.index.json" ]]; then
  SHARDS=$(python3 -c "
import json,sys
idx=json.load(open('${DEST}/model.safetensors.index.json'))['weight_map']
print('\n'.join(sorted(set(idx.values()))))
")
else
  SHARDS="model.safetensors"
fi

for shard in $SHARDS; do
  if [[ -s "${DEST}/${shard}" ]]; then
    echo "  [skip] ${shard}"
    continue
  fi
  echo "  [get]  ${shard}"
  curl -fL --retry 3 --retry-delay 2 -C - -o "${DEST}/${shard}.part" "${BASE}/${shard}"
  mv "${DEST}/${shard}.part" "${DEST}/${shard}"
  echo "  [done] ${shard} ($(stat -c%s "${DEST}/${shard}") bytes)"
done

echo "[fetch] imekamilika: ${DEST}"
du -sh "${DEST}"
