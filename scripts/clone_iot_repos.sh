#!/usr/bin/env bash
# ============================================================
# MTAALAMU SMART — clone repos za IoT kutoka data/iot/*.json
# JSON ndio chanzo (KANUNI 2/5): miradi yenye "focus": true
# kwenye data/iot/registry/<vertical>.json, data/iot/hermes.json
# na data/iot/agents_oss.json ndiyo seti ya FOCUS.
#
# Matumizi:
#   bash scripts/clone_iot_repos.sh                 # FOCUS 4 (registry tu)
#   bash scripts/clone_iot_repos.sh --agents        # FOCUS + agents za focus (8)
#   bash scripts/clone_iot_repos.sh --all           # miradi YOTE ya registry (4 verticals)
#   bash scripts/clone_iot_repos.sh --all --agents  # ZOTE (registry + agents wote)
#   bash scripts/clone_iot_repos.sh --only nyumbani # vertical moja tu
#   bash scripts/clone_iot_repos.sh --list          # onyesha bila ku-clone (dry-run)
# ============================================================
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

UPSTREAM="$ROOT/upstream"
MODE_ALL=0
WITH_AGENTS=0
ONLY_VERTICAL=""
DO_LIST=0

for arg in "$@"; do
  case "$arg" in
    --all)    MODE_ALL=1 ;;
    --agents) WITH_AGENTS=1 ;;
    --list)   DO_LIST=1 ;;
    --only)   ONLY_VERTICAL="__NEXT__" ;;
    *)
      if [ "$ONLY_VERTICAL" = "__NEXT__" ]; then ONLY_VERTICAL="$arg"; fi
      ;;
  esac
done

PYTHON_BIN="$(command -v python3 || command -v python)"

# Kila mstari: TAB id  DIR  REPO  (focus=true inaonyeshwa kwenye --list)
list_registry() { # $1 = vertical id au "" (zote)
  "$PYTHON_BIN" - "$1" <<'PY'
import json, sys, os
want = sys.argv[1]
reg = "data/iot/registry"
for f in sorted(os.listdir(reg)):
    if not f.endswith(".json"):
        continue
    vert = f[:-5]
    if want and vert != want:
        continue
    d = json.load(open(os.path.join(reg, f), encoding="utf-8"))
    for p in d.get("projects", []):
        focus = "FOCUS" if p.get("focus") else "-----"
        print(f"{p['id']}\tupstream/{p['id']}\t{p['repo']}\t{focus}")
PY
}

list_hermes() {
  "$PYTHON_BIN" <<'PY'
import json
d = json.load(open("data/iot/hermes.json", encoding="utf-8"))["hermes"]
u = d.get("upstream", {})
if u.get("repo"):
    focus = "FOCUS" if u.get("focus") else "-----"
    print(f"hermes-agent\tupstream/hermes-agent\t{u['repo']}\t{focus}")
PY
}

list_agents() {
  "$PYTHON_BIN" <<'PY'
import json
d = json.load(open("data/iot/agents_oss.json", encoding="utf-8"))
for a in d.get("agents", []):
    if a.get("status") == "adapter-planned":
        continue
    focus = "FOCUS" if a.get("focus") else "-----"
    print(f"{a['id']}\tupstream/{a['id']}\t{a['repo']}\t{focus}")
PY
}

# Chagua seti: registry (focus au all, na hiari vertical moja)
if [ -n "$ONLY_VERTICAL" ]; then
  ROWS="$(list_registry "$ONLY_VERTICAL")"
else
  ROWS="$(list_registry "")"
  if [ "$MODE_ALL" -ne 1 ]; then
    ROWS="$(printf '%s\n' "$ROWS" | awk -F'\t' '$4=="FOCUS"')"
  fi
fi

if [ "$WITH_AGENTS" -eq 1 ]; then
  AGENT_ROWS="$(list_agents)"
  if [ "$MODE_ALL" -ne 1 ]; then
    AGENT_ROWS="$(printf '%s\n' "$AGENT_ROWS" | awk -F'\t' '$4=="FOCUS"')"
  fi
  HERMES_ROW="$(list_hermes)"
  ROWS="$(printf '%s\n%s\n%s\n%s\n' "$ROWS" "$HERMES_ROW" "$AGENT_ROWS" | awk 'NF' | sort -u -t$'\t' -k1,1)"
fi

if [ "$DO_LIST" -eq 1 ]; then
  echo "id	dir	repo	focus"
  printf '%s\n' "$ROWS" | awk 'NF'
  echo "---"
  echo "jumla: $(printf '%s\n' "$ROWS" | awk 'NF' | wc -l) (focus: $(printf '%s\n' "$ROWS" | awk -F'\t' '$4=="FOCUS"' | wc -l))"
  exit 0
fi

mkdir -p "$UPSTREAM"
FAIL=0
while IFS=$'\t' read -r id dir repo _focus; do
  [ -z "${id:-}" ] && continue
  dest="$ROOT/$dir"
  if [ -d "$dest/.git" ]; then
    echo "SKIP   $id (ipo)"
    continue
  fi
  echo "CLONED $id -> $dir"
  if ! git clone --depth 1 --single-branch "$repo" "$dest" >/dev/null 2>&1; then
    echo "WARN   $id imeshindikana ($repo)" >&2
    FAIL=$((FAIL+1))
  fi
done <<EOF
$ROWS
EOF

if [ "$FAIL" -gt 0 ]; then
  echo "Imekamilika na makosa $FAIL." >&2
  exit 1
fi
echo "Imekamilika."
