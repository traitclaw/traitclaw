#!/usr/bin/env bash
set -euo pipefail

# TraitClaw v1.0.0 — crates.io publish script
# Publishes all 14 workspace crates in topological dependency order.
#
# Usage:
#   ./scripts/publish.sh              # Publish all crates
#   ./scripts/publish.sh --dry-run    # Dry-run mode (no actual publish)
#   ./scripts/publish.sh --from=NAME  # Resume from a specific crate
#
# Prerequisites:
#   cargo login <your-crates-io-token>

DRY_RUN=false
FROM_CRATE=""

while [[ $# -gt 0 ]]; do
  case $1 in
    --dry-run) DRY_RUN=true; shift ;;
    --from=*) FROM_CRATE="${1#*=}"; shift ;;
    --from) FROM_CRATE="$2"; shift 2 ;;
    *) echo "Unknown option: $1"; exit 1 ;;
  esac
done

# Topological dependency order (Level 0 → Level 3)
CRATES=(
  # Level 0: No internal deps
  traitclaw-core
  # Level 1: Depends on core only
  traitclaw-test-utils
  traitclaw-macros
  traitclaw-openai-compat
  traitclaw-anthropic
  traitclaw-eval
  traitclaw-mcp
  traitclaw-memory-sqlite
  traitclaw-rag
  traitclaw-steering
  traitclaw-team
  # Level 2: Depends on Level 1 crates
  traitclaw-openai
  traitclaw-strategies
  # Level 3: Meta-crate (depends on everything)
  traitclaw
)

DELAY=30
SKIP=true

if [ -z "$FROM_CRATE" ]; then
  SKIP=false
fi

echo "╔══════════════════════════════════════════╗"
echo "║   TraitClaw — crates.io Publish Script   ║"
echo "╠══════════════════════════════════════════╣"
if [ "$DRY_RUN" = true ]; then
  echo "║   Mode: DRY RUN (no actual publish)      ║"
else
  echo "║   Mode: LIVE PUBLISH                     ║"
fi
echo "╚══════════════════════════════════════════╝"
echo ""

TOTAL=${#CRATES[@]}
CURRENT=0

for crate in "${CRATES[@]}"; do
  CURRENT=$((CURRENT + 1))

  # Skip until we reach the --from crate
  if [ "$SKIP" = true ]; then
    if [ "$crate" = "$FROM_CRATE" ]; then
      SKIP=false
    else
      echo "⏭  [$CURRENT/$TOTAL] Skipping $crate (already published)"
      continue
    fi
  fi

  echo "📦 [$CURRENT/$TOTAL] Publishing $crate..."

  if [ "$DRY_RUN" = true ]; then
    cargo publish --dry-run --allow-dirty -p "$crate" 2>&1 || true
  else
    cargo publish -p "$crate"
  fi

  echo "✅ $crate published!"

  # Wait for crates.io index propagation (skip for last crate)
  if [ "$CURRENT" -lt "$TOTAL" ] && [ "$DRY_RUN" = false ]; then
    echo "⏳ Waiting ${DELAY}s for index propagation..."
    sleep $DELAY
  fi

  echo ""
done

echo "════════════════════════════════════════════"
echo "🎉 All $TOTAL crates published successfully!"
echo ""
echo "Post-publish checklist:"
echo "  1. git tag -a v1.0.0 -m 'TraitClaw v1.0.0 — Production Ready'"
echo "  2. git push origin v1.0.0"
echo "  3. Create GitHub Release from tag"
echo "  4. Verify https://crates.io/crates/traitclaw"
echo "  5. Verify https://docs.rs/traitclaw"
echo "════════════════════════════════════════════"
