# Story 5.2: Migration Guide and Backward Compatibility Verification

Status: ready-for-dev

## Story

As an existing TraitClaw user,
I want a migration guide and verified backward compatibility,
so that I can upgrade to v0.8.0 with confidence.

## Acceptance Criteria

1. `docs/migration-v0.7-to-v0.8.md` exists with complete migration documentation
2. Document lists all new public types: `AgentEvent`, `ModelPricing`, `default_pricing()`
3. Document confirms zero breaking changes with specific examples
4. Document provides test migration examples (before/after using `traitclaw-test-utils`)
5. All 25 existing examples compile: `cargo build --examples` or per-example builds
6. `cargo test --workspace` passes with zero failures
7. Document is linked from main README or CHANGELOG

## Tasks / Subtasks

- [ ] Task 1: Create migration guide (AC: #1, #2, #3)
  - [ ] Create `docs/migration-v0.7-to-v0.8.md`
  - [ ] Section: "What's New in v0.8.0" — list all additions
  - [ ] Section: "Zero Breaking Changes" — explicit statement + evidence
  - [ ] Section: "New Types Reference" — AgentEvent, ModelPricing, RunUsage changes
- [ ] Task 2: Test migration examples (AC: #4)
  - [ ] Section: "Migrating Tests to traitclaw-test-utils" — before/after code
  - [ ] Section: "Adding Observability" — how to add tracing + events
  - [ ] Section: "Adding Cost Tracking" — how to use pricing
- [ ] Task 3: Backward compatibility verification (AC: #5, #6)
  - [ ] Compile all existing examples: loop through `examples/*/Cargo.toml`
  - [ ] `cargo test --workspace` — verify all pass
  - [ ] Document any deprecation warnings (should be none new)
- [ ] Task 4: Link from project docs (AC: #7)
  - [ ] Add link in README.md changelog section
  - [ ] Or create/update CHANGELOG.md

## Dev Notes

### Migration Guide Structure

```markdown
# Migrating from v0.7.0 to v0.8.0

## Overview

v0.8.0 is a **fully backward-compatible** infrastructure release.
No breaking changes. All existing code compiles without modification.

## What's New

### Shared Test Utilities (`traitclaw-test-utils`)
- `MockProvider` — deterministic LLM mock
- `MockMemory` — in-memory session storage
- `EchoTool`, `FailTool` — tool testing helpers
- `make_runtime()` — one-call test runtime setup

### Runtime Observability
- `AgentEvent` — typed lifecycle events
- `AgentBuilder::on_event()` — event callback
- Structured `tracing` spans on LLM, tool, guard operations

### Cost Tracking
- `RunUsage.estimated_cost_usd` — per-run cost estimate
- `ModelPricing` — configurable pricing table
- `default_pricing()` — built-in model prices

### CI/CD
- GitHub Actions workflow for automated quality checks

## Zero Breaking Changes

The following are explicitly unchanged:
- All trait signatures: `Provider`, `Tool`, `Memory`, `Guard`, `Hint`, `Tracker`
- `Agent::builder()` API — all existing methods preserved
- `AgentOutput` structure — new field `estimated_cost_usd` defaults to `0.0`
- `AgentStrategy` trait — no signature changes

## Migrating Tests (Optional)

### Before (v0.7.0 — inline mocks)
```rust
// Each test file defines its own MockProvider...
struct MockProvider { ... }
impl Provider for MockProvider { ... }
```

### After (v0.8.0 — shared utils)
```rust
// In Cargo.toml: [dev-dependencies] traitclaw-test-utils = { path = "..." }
use traitclaw_test_utils::provider::MockProvider;
use traitclaw_test_utils::runtime::make_runtime;

let rt = make_runtime(MockProvider::text("hello"), vec![]);
```

## Adding Observability (Optional)

```rust
let agent = Agent::builder()
    .provider(my_provider)
    .system("...")
    .on_event(|event| println!("{event:?}"))
    .with_pricing(traitclaw::default_pricing())
    .build()?;
```
```

### Verification Commands

```bash
# All examples compile
for dir in examples/*/; do
  cargo check --manifest-path "${dir}Cargo.toml" 2>&1 | tail -1
done

# All tests pass
cargo test --workspace --all-features

# No new deprecation warnings
cargo clippy --workspace --all-targets -- -D warnings
```

### References

- [_bmad-output/planning-artifacts/prd-v0.8.0.md](file:///Users/admin/Desktop/Projects/traitclaw/_bmad-output/planning-artifacts/prd-v0.8.0.md) — FR23, FR26, FR27
- [_bmad-output/planning-artifacts/architecture-v0.8.0.md](file:///Users/admin/Desktop/Projects/traitclaw/_bmad-output/planning-artifacts/architecture-v0.8.0.md) — all new types
- All existing examples in `examples/` directory

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
