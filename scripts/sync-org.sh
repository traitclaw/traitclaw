#!/usr/bin/env bash
# Sync to traitclaw/traitclaw org repo, excluding _bmad-output/
# Usage: ./scripts/sync-org.sh

set -euo pipefail

ORG_REMOTE="org"
BRANCH="main"

echo "🔄 Syncing to org repo (excluding private content)..."

# Use a temp index to build filtered tree without switching branches
export GIT_INDEX_FILE="$(git rev-parse --git-dir)/index.sync-org"
trap 'rm -f "$GIT_INDEX_FILE"' EXIT

# Read current HEAD into temp index
git read-tree HEAD

# Remove excluded paths from temp index
git rm -r --cached --quiet _bmad-output/ 2>/dev/null || true

# Write filtered tree
TREE=$(git write-tree)

# Create commit
MSG="sync: mirror from private repo (excludes _bmad-output)"
COMMIT=$(echo "$MSG" | git commit-tree "$TREE")

# Force push
git push "$ORG_REMOTE" "$COMMIT:refs/heads/$BRANCH" --force

echo "✅ Pushed to $ORG_REMOTE/$BRANCH (without _bmad-output)"
echo "🎉 Sync complete!"
