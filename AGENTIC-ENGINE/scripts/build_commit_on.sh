#!/usr/bin/env bash
# Huunda commit tayari ya kusukuma: faili ZETU juu ya FETCH_HEAD (historia ya mbali).
# Matumizi: bash scripts/build_commit_on.sh <dest_dir_au_-_> <branch> <msg_file>
#   dest_dir "-" = weka faili kwenye mzizi wa repo
set -u
unset GIT_INDEX_FILE
cd /workspace/project

DEST="$1"
BRANCH="$2"
MSG="$3"

# Orodha ya faili ZETU (index ya kawaida) — kabla ya kubadilisha index.
git ls-files > /tmp/proj_files.txt
echo "faili zetu: $(wc -l < /tmp/proj_files.txt)"

export GIT_INDEX_FILE=/tmp/msidx
rm -f "$GIT_INDEX_FILE"
git read-tree FETCH_HEAD || exit 1

n=0
while IFS= read -r f; do
  sha=$(git hash-object -w "$f") || { echo "hash imeshindikana: $f"; exit 1; }
  if [ "$DEST" = "-" ]; then path="$f"; else path="$DEST/$f"; fi
  git update-index --add --cacheinfo "100644,$sha,$path" || exit 1
  n=$((n + 1))
done < /tmp/proj_files.txt
echo "zilizoongezwa: $n"

tree=$(git write-tree) || exit 1
commit=$(git commit-tree "$tree" -p FETCH_HEAD -F "$MSG") || exit 1
git update-ref "refs/heads/$BRANCH" "$commit" || exit 1
unset GIT_INDEX_FILE

echo "tree=$tree"
echo "commit=$commit"
git --no-pager log --oneline -1 "$BRANCH"
git --no-pager show --stat "$BRANCH" | tail -3
