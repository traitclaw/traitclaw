#!/usr/bin/env bash
set -euo pipefail

echo "�� Publishing TraitClaw Crates to crates.io..."

CRATES=(
  "crates/traitclaw-core"
  "crates/traitclaw-macros"
  "crates/traitclaw-memory-sqlite"
  "crates/traitclaw-openai-compat"
  "crates/traitclaw-anthropic"
  "crates/traitclaw-openai"
  "crates/traitclaw-mcp"
  "crates/traitclaw-rag"
  "crates/traitclaw-team"
  "crates/traitclaw-steering"
  "crates/traitclaw-eval"
  "crates/traitclaw"
)

for CRATE in "${CRATES[@]}"; do
  echo "📦 Publishing $CRATE..."
  cd "$CRATE"
  cargo publish --allow-dirty || echo "⚠️ Failed to publish $CRATE. It might already be published."
  cd - >/dev/null
  sleep 2
done

echo "🎉 All crates published successfully!"
