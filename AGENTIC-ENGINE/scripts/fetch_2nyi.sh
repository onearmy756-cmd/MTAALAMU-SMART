#!/usr/bin/env bash
# Hupakua na kuunganisha faili za modeli 2NYI kutoka GitHub Release.
#
# Faili za safetensors (>2GB) zimegawanywa vipande chini ya 2GiB kila kimo
# (kikomo cha GitHub release asset) na kupakiwa kwenye release tag "2NYI".
# Script hii hupakua vipande, hiviunganisha, na kuthibitisha SHA256.
#
# Repo ni PRIVATE, hivyo assets hupakuliwa kwa API ya GitHub + token.
# Weka GITHUB_PAT kwenye mazingira kabla ya kuiita.
#
# Matumizi:
#   GITHUB_PAT=ghp_... scripts/fetch_2nyi.sh [dest_dir]
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEST="${1:-$ROOT/models/2NYI}"
REPO="onearmy756-cmd/AGENT-ENGINE-"
TAG="2NYI"
TOK="${GITHUB_PAT:?weka GITHUB_PAT (repo ni private)}"
API="https://api.github.com/repos/${REPO}"

mkdir -p "$DEST"
cd "$DEST"

echo "== kutafuta assets za release '$TAG' =="
REL_ID=$(curl -sf -H "Authorization: token $TOK" "$API/releases/tags/$TAG" \
  | python3 -c "import json,sys;print(json.load(sys.stdin)['id'])")

curl -sf -H "Authorization: token $TOK" "$API/releases/$REL_ID/assets?per_page=100" \
  | python3 -c "
import json,sys
for a in sorted(json.load(sys.stdin), key=lambda x: x['name']):
    print(a['name'], a['id'])" > /tmp/_2nyi_assets.txt

echo "== kupakua =="
while read -r name id; do
  echo "  $name"
  curl -fL --retry 3 -H "Authorization: token $TOK" \
    -H "Accept: application/octet-stream" -o "$name" \
    "$API/releases/assets/$id"
done < /tmp/_2nyi_assets.txt

echo "== kuunganisha vipande =="
for p0 in *.part00; do
  target="${p0%.part00}"
  echo "  $target"
  cat "${target}".part* > "$target"
  rm -f "${target}".part*
done

echo "== kuthibitisha SHA256 =="
if [ -f MANIFEST.sha256 ]; then
  sha256sum -c MANIFEST.sha256
else
  echo "MANIFEST.sha256 haipo; ruka uthibitisho."
fi

echo "TAYARI: $DEST"
