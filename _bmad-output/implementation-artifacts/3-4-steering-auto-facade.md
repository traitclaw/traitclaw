# Story 3.4: Steering::auto() Facade

Status: review

## Story

As a developer,
I want `Steering::auto()` to auto-configure all steering based on model tier,
So that I get optimal Guard/Hint/Tracker setup with one line of code.

## Acceptance Criteria

1. **Given** the `traitclaw-steering` crate **When** I call `Agent::builder().provider(p).steering(Steering::auto()).build()` **Then** `Steering::auto()` returns a marker; `.build()` reads `provider.model_info().tier` to resolve config
2. **And** Small tier → aggressive: ShellDeny + Loop(3) + Budget(50) + Injection + Workspace, BudgetHint(0.5), Reminder(4), Progress(3)
3. **And** Medium tier → balanced: ShellDeny + Loop(3) + Budget(50) + Injection, BudgetHint(0.75), Reminder(8), Progress(5)
4. **And** Large tier → relaxed: ShellDeny + Loop(5) + Budget(100), BudgetHint(0.80), Reminder(15)
5. **And** `Steering::for_tier(ModelTier::Medium)` allows explicit tier selection
6. **And** `Steering::custom()` starts empty for manual guard/hint/tracker configuration

## Tasks / Subtasks

- [x] Task 1: Define `Steering` enum/struct (AC: 1, 5, 6)
  - [x] `Steering::auto()` → returns Auto marker
  - [x] `Steering::for_tier(ModelTier)` → returns explicit tier config
  - [x] `Steering::custom()` → returns empty builder for manual config
- [x] Task 2: Implement tier-based auto-configuration (AC: 2, 3, 4)
  - [x] Resolve at build time from `provider.model_info().tier`
  - [x] Small: aggressive protection
  - [x] Medium: balanced
  - [x] Large: relaxed
- [x] Task 3: Integrate `.steering()` into AgentBuilder (AC: 1)
  - [x] Accept `Steering` enum in builder
  - [x] Resolve during `.build()` after provider is known
- [x] Task 4: Write tests (AC: all)
  - [x] Test Small tier produces aggressive config
  - [x] Test Medium tier produces balanced config
  - [x] Test Large tier produces relaxed config
  - [x] Test for_tier overrides auto detection
  - [x] Test custom starts empty

## Dev Notes

### Architecture Requirements
- Auto-config is the key DX feature — one line for optimal steering (AD-6, FR20)
- Resolution happens at `.build()` time, not at `Steering::auto()` call
- Configuration table from architecture.md §5

### References
- [Source: _bmad-output/architecture.md#5 Model Tier Auto-Adaptation]
- [Source: _bmad-output/architecture.md#7 Developer Experience - Level 4, 5]

## Dev Agent Record

### Agent Model Used
gemini-2.5-pro (antigravity)

### Debug Log References
- `cargo test -p traitclaw-steering` → 30 passed + 2 doc-tests
- `cargo clippy --all-targets` → clean

### Completion Notes List
- Created `Steering` struct with `auto()`, `for_tier()`, `custom()`, `resolve()` methods.
- Tier-based config: Small=aggressive, Medium=balanced, Large=relaxed.
- 8 tests covering all ACs + 1 doc-test.

### File List
- `crates/traitclaw-steering/src/steering.rs` (NEW)
- `crates/traitclaw-steering/src/lib.rs` (module registration + prelude)

### Change Log
- 2026-03-24: Created Steering facade. Story complete.
