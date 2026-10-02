#!/usr/bin/env bash
# ============================================================
# clone_iot_repos.sh — MTAALAMU SMART
# Inaclone (shallow) miradi YOTE ya bure ya GitHub iliyoandikwa kwenye:
#   data/iot/registry/afya.json      (hospitali)
#   data/iot/registry/usalama.json   (security)
#   data/iot/registry/nyumbani.json  (smart home)
#   data/iot/registry/kilimo.json    (agriculture)
#   data/iot/hermes.json             (HERMES — msimamizi mkuu)
#   data/iot/agents_oss.json         (agents za OSS — kwa --agents pekee)
#
# Kila repo inakwenda upstream/<id> (haipo kwenye git — tazama .gitignore).
# Kanuni: JSON ndio chanzo pekee; script hii haitoi repos kwa mkono.
#
# Matumizi:
#   bash scripts/clone_iot_repos.sh                 # clone zote za verticals + hermes-agent
#   bash scripts/clone_iot_repos.sh --list          # orodhesha tu (bila clone)
#   bash scripts/clone_iot_repos.sh --only kilimo   # vertical moja
#   bash scripts/clone_iot_repos.sh --id openmrs    # repo moja
#   bash scripts/clone_iot_repos.sh --agents        # + agents zote za OSS (kubwa!)
#   bash scripts/clone_iot_repos.sh --force         # clone upya zilizopo
#   bash scripts/clone_iot_repos.sh --limit 3       # repo 3 za kwanza (test/mtandao hafifu)
# ============================================================
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

UPSTREAM_DIR="upstream"
MODE="clone"        # clone | list
ONLY=""             # afya | usalama | nyumbani | kilimo | hermes
ONLY_ID=""          # id ya project moja
WITH_AGENTS=0
FORCE=0
LIMIT=0

usage() {
  sed -n '2,20p' "$0" | sed 's/^# \{0,1\}//'
  exit 0
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --list)   MODE="list"; shift ;;
    --only)   ONLY="${2:?--only inahitaji vertical}"; shift 2 ;;
    --id)     ONLY_ID="${2:?--id inahitaji project id}"; shift 2 ;;
    --agents) WITH_AGENTS=1; shift ;;
    --force)  FORCE=1; shift ;;
    --limit)  LIMIT="${2:?--limit inahitaji namba}"; shift 2 ;;
    -h|--help) usage ;;
    *) echo "Flag haijulikani: $1 (tumia --help)" >&2; exit 2 ;;
  esac
done

# --- 1. Soma registry zote (JSON = chanzo pekee) ----------------------------
extract_rows() {
  if command -v python3 >/dev/null 2>&1; then
    python3 - "$ROOT" "$WITH_AGENTS" <<'PY'
import glob, json, os, sys

root, with_agents = sys.argv[1], sys.argv[2] == "1"

def load(rel):
    with open(os.path.join(root, rel), encoding="utf-8") as fh:
        return json.load(fh)

for path in sorted(glob.glob(os.path.join(root, "data/iot/registry/*.json"))):
    vertical = os.path.basename(path)[:-5]
    doc = load(os.path.relpath(path, root))
    for p in doc.get("projects", []):
        print(f"{vertical}|{p['id']}|{p['repo']}")

hermes = load("data/iot/hermes.json").get("hermes", {})
up = hermes.get("upstream", {}).get("repo")
if up:
    print(f"hermes|hermes-agent|{up}")

if with_agents:
    agents = load("data/iot/agents_oss.json").get("agents", [])
    for a in agents:
        print(f"agents|{a['id']}|{a['repo']}")
PY
  elif command -v node >/dev/null 2>&1; then
    node -e '
const fs = require("fs"), path = require("path");
const root = process.argv[1], withAgents = process.argv[2] === "1";
const load = (rel) => JSON.parse(fs.readFileSync(path.join(root, rel), "utf8"));
for (const f of fs.readdirSync(path.join(root, "data/iot/registry")).sort()) {
  if (!f.endsWith(".json")) continue;
  const v = f.slice(0, -5);
  for (const p of (load(`data/iot/registry/${f}`).projects || [])) console.log(`${v}|${p.id}|${p.repo}`);
}
const up = (load("data/iot/hermes.json").hermes || {}).upstream || {};
if (up.repo) console.log(`hermes|hermes-agent|${up.repo}`);
if (withAgents) for (const a of (load("data/iot/agents_oss.json").agents || [])) console.log(`agents|${a.id}|${a.repo}`);
' "$ROOT" "$WITH_AGENTS"
  else
    echo "[FAIL] python3 au node inahitajika kusoma JSON registries" >&2
    exit 1
  fi
}

# --- 2. Chuja (only / id) ----------------------------------------------------
ROWS="$(extract_rows)"
if [[ -n "$ONLY" ]]; then
  ROWS="$(printf '%s\n' "$ROWS" | awk -F'|' -v v="$ONLY" '$1 == v')"
fi
if [[ -n "$ONLY_ID" ]]; then
  ROWS="$(printf '%s\n' "$ROWS" | awk -F'|' -v id="$ONLY_ID" '$2 == id')"
fi
if [[ "$LIMIT" -gt 0 ]]; then
  ROWS="$(printf '%s\n' "$ROWS" | head -n "$LIMIT")"
fi

if [[ -z "$ROWS" ]]; then
  echo "[FAIL] Hakuna project inayolingana (--only '$ONLY' --id '$ONLY_ID')." >&2
  exit 1
fi

TOTAL="$(printf '%s\n' "$ROWS" | wc -l | tr -d ' ')"

# --- 3. List au clone --------------------------------------------------------
if [[ "$MODE" == "list" ]]; then
  printf '%-9s %-18s %-9s %s\n' "VERTICAL" "ID" "HALI" "REPO"
  while IFS='|' read -r vertical id url; do
    if [[ -d "$UPSTREAM_DIR/$id/.git" ]]; then state="CLONED"; else state="-"; fi
    printf '%-9s %-18s %-9s %s\n' "$vertical" "$id" "$state" "$url"
  done <<< "$ROWS"
  echo ""
  echo "Jumla: $TOTAL project(s). Clone: bash scripts/clone_iot_repos.sh [--only V | --id ID] [--agents]"
  exit 0
fi

mkdir -p "$UPSTREAM_DIR"
OK=0; SKIP=0; FAIL=0
FAILED_IDS=()

echo "=== MTAALAMU SMART — clone miradi ya IoT + HERMES ($TOTAL) ==="
echo "Dest: $ROOT/$UPSTREAM_DIR/"
echo ""

while IFS='|' read -r vertical id url; do
  dest="$UPSTREAM_DIR/$id"
  if [[ -d "$dest" && "$FORCE" -eq 0 ]]; then
    echo "[SKIP] $id ($vertical) — tayari ipo ($dest)"
    SKIP=$((SKIP + 1))
    continue
  fi
  if [[ "$FORCE" -eq 1 && -d "$dest" ]]; then
    rm -rf "$dest"
  fi
  echo "[...]  $id ($vertical) <- $url"
  if git clone --depth 1 --single-branch --quiet "$url" "$dest"; then
    echo "[OK]   $id -> $dest"
    OK=$((OK + 1))
  else
    echo "[FAIL] $id — clone imeshindikana ($url)"
    FAIL=$((FAIL + 1))
    FAILED_IDS+=("$id")
  fi
done <<< "$ROWS"

echo ""
echo "=== SUMMARY: OK $OK · SKIP $SKIP · FAIL $FAIL ==="
if [[ "$FAIL" -gt 0 ]]; then
  echo "Zilizoshindikana: ${FAILED_IDS[*]}"
  echo "Kumbuka: thibitisha LICENSE ya kila repo kabla ya uzalishaji (data/iot/SCHEMA.md)."
  exit 1
fi
echo "Kumbuka: thibitisha LICENSE ya kila repo kabla ya uzalishaji (data/iot/SCHEMA.md)."
