#!/usr/bin/env bash
# Huunda commit tayari ya kusukuma: folda AGENTIC-ENGINE/ juu ya main ya MTAALAMU-SMART.
# Matumizi: bash scripts/build_ms_commit.sh [dest_dir] [branch]
set -u
unset GIT_INDEX_FILE
cd /workspace/project

DEST="${1:-AGENTIC-ENGINE}"
BRANCH="${2:-agentic-engine}"

# 1. Orodha ya faili ZETU (index ya kawaida) — kabla ya kubadilisha index.
git ls-files > /tmp/proj_files.txt
echo "faili zetu: $(wc -l < /tmp/proj_files.txt)"

# 2. Index ya muda yenye main ya MTAALAMU-SMART.
export GIT_INDEX_FILE=/tmp/msidx
rm -f "$GIT_INDEX_FILE"
git read-tree FETCH_HEAD || exit 1

# 3. Ongeza faili zetu chini ya DEST/.
n=0
while IFS= read -r f; do
  sha=$(git hash-object -w "$f") || { echo "hash imeshindikana: $f"; exit 1; }
  git update-index --add --cacheinfo "100644,$sha,$DEST/$f" || exit 1
  n=$((n + 1))
done < /tmp/proj_files.txt
echo "zilizoongezwa: $n"

tree=$(git write-tree) || exit 1
echo "tree=$tree"

commit=$(git commit-tree "$tree" -p FETCH_HEAD -F /tmp/ms_msg.txt) || exit 1
git update-ref "refs/heads/$BRANCH" "$commit" || exit 1
unset GIT_INDEX_FILE

echo "commit=$commit"
git --no-pager log --oneline -1 "$BRANCH"
git --no-pager show --stat "$BRANCH" | tail -4
