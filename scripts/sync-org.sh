#!/usr/bin/env bash
# Sync to traitclaw/traitclaw org repo, excluding _bmad-output/
# Usage: ./scripts/sync-org.sh

set -euo pipefail

ORG_REMOTE="org"
BRANCH="main"
EXCLUDE_PATHS=("_bmad-output/")

echo "🔄 Syncing to org repo (excluding private content)..."

# Create a temporary filtered branch
TEMP_BRANCH="__org_sync_temp__"
git checkout -b "$TEMP_BRANCH" 2>/dev/null || git checkout "$TEMP_BRANCH"

# Remove excluded paths from index only (keep local files)
for path in "${EXCLUDE_PATHS[@]}"; do
  if git ls-files --error-unmatch "$path" &>/dev/null; then
    git rm -r --cached --quiet "$path"
    echo "  ✂️  Excluded: $path"
  fi
done

# Create filtered commit
git commit --allow-empty -m "sync: filtered push to org" --quiet

# Force push to org
git push "$ORG_REMOTE" "$TEMP_BRANCH:$BRANCH" --force
echo "✅ Pushed to $ORG_REMOTE/$BRANCH"

# Switch back and clean up
git checkout "$BRANCH"
git branch -D "$TEMP_BRANCH"

echo "🎉 Sync complete!"
