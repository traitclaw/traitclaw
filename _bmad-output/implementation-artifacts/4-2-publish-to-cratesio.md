# Story 4.2: Publish to crates.io and Post-Publish Verification

Status: ready-for-dev

## Story

As a **framework maintainer**,
I want all 14 crates published to crates.io in correct dependency order,
so that users can install TraitClaw via `cargo add traitclaw`.

## Acceptance Criteria

1. All 14 crates are live on crates.io
2. `cargo add traitclaw` works from a fresh project
3. `cargo doc` renders correctly after adding the dependency
4. README renders correctly on each crate's crates.io page
5. docs.rs builds succeed for all crates

## Tasks / Subtasks

- [ ] Task 1 (AC: #1): Publish all crates in dependency order
  - [ ] `cargo publish -p traitclaw-core` → wait 30s
  - [ ] `cargo publish -p traitclaw-test-utils` → wait 30s
  - [ ] `cargo publish -p traitclaw-macros` → wait 30s
  - [ ] `cargo publish -p traitclaw-openai-compat` → wait 30s
  - [ ] `cargo publish -p traitclaw-anthropic` → wait 30s
  - [ ] `cargo publish -p traitclaw-eval` → wait 30s
  - [ ] `cargo publish -p traitclaw-mcp` → wait 30s
  - [ ] `cargo publish -p traitclaw-memory-sqlite` → wait 30s
  - [ ] `cargo publish -p traitclaw-rag` → wait 30s
  - [ ] `cargo publish -p traitclaw-steering` → wait 30s
  - [ ] `cargo publish -p traitclaw-team` → wait 30s
  - [ ] `cargo publish -p traitclaw-openai` → wait 30s
  - [ ] `cargo publish -p traitclaw-strategies` → wait 30s
  - [ ] `cargo publish -p traitclaw` → done
- [ ] Task 2 (AC: #2-3): Verify from fresh project
  - [ ] `cargo init /tmp/test-traitclaw && cd /tmp/test-traitclaw && cargo add traitclaw && cargo build`
- [ ] Task 3 (AC: #4-5): Verify crates.io and docs.rs
  - [ ] Check https://crates.io/crates/traitclaw — README renders
  - [ ] Check https://docs.rs/traitclaw — docs build

## Post-Publish Checklist

- [ ] Create git tag: `git tag -a v1.0.0 -m "TraitClaw v1.0.0 — Production Ready"`
- [ ] Push tag: `git push origin v1.0.0`
- [ ] Create GitHub Release from tag with CHANGELOG entry as body
- [ ] Verify docs.rs builds complete for all 14 crates
- [ ] Announce release (if applicable)

## Dev Notes

- **REQUIRES** crates.io authentication: `cargo login <token>`
- **REQUIRES** ownership of all 14 crate names on crates.io
- Check name availability first: `cargo search traitclaw`
- If a name is taken, need to choose alternative name
- 30-second delay between publishes allows crates.io index to propagate
- If publish fails mid-sequence, resume from the failed crate — already-published crates don't need re-publish
- Create `scripts/publish.sh` for repeatability in future releases

### Publish Script (save as scripts/publish.sh)

```bash
#!/bin/bash
set -euo pipefail

CRATES=(
  traitclaw-core
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
  traitclaw-openai
  traitclaw-strategies
  traitclaw
)

for crate in "${CRATES[@]}"; do
  echo "Publishing $crate..."
  cargo publish -p "$crate"
  echo "Waiting for index propagation..."
  sleep 30
done

echo "All crates published!"
```

### References

- [Source: architecture-v1.0.0.md#AD2 — Publish Strategy]
- [Source: architecture-v1.0.0.md#P3 — Publish Script Pattern]
- [Source: prd-v1.0.0.md#FR13-FR14]
