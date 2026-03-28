# Story 1.1: Scaffold `traitclaw-strategies` Crate

Status: review

## Story

As a developer,
I want to add `traitclaw-strategies` as a workspace crate,
so that I have a properly configured crate to implement reasoning strategies.

## Acceptance Criteria

1. New crate at `crates/traitclaw-strategies/` with `Cargo.toml` depending on `traitclaw-core`
2. Feature flags: `react`, `mcts`, `cot` (default = all on)
3. `src/lib.rs` with feature-gated module declarations
4. `src/common/mod.rs` with public re-exports
5. `cargo check -p traitclaw-strategies` succeeds
6. `cargo check -p traitclaw-strategies --no-default-features` succeeds
7. Internal types use `pub(crate)`, public API re-exported from `lib.rs`
8. Crate registered in workspace `Cargo.toml` members

## Tasks / Subtasks

- [x] Task 1: Create crate directory structure (AC: #1, #4)
  - [x] Create `crates/traitclaw-strategies/Cargo.toml`
  - [x] Create `crates/traitclaw-strategies/src/lib.rs`
  - [x] Create `crates/traitclaw-strategies/src/common/mod.rs`
  - [x] Create empty module dirs: `src/react/`, `src/mcts/`, `src/cot/`
- [x] Task 2: Configure Cargo.toml (AC: #1, #2, #8)
  - [x] Add `traitclaw-core` dependency
  - [x] Add `serde`, `serde_json`, `tokio`, `async-trait` dependencies
  - [x] Define feature flags: `default = ["react", "mcts", "cot"]`
  - [x] Register in workspace `Cargo.toml` members list
- [x] Task 3: Implement `lib.rs` with feature gates (AC: #3, #5, #6, #7)
  - [x] `#[cfg(feature = "react")] pub mod react;`
  - [x] `#[cfg(feature = "mcts")] pub mod mcts;`
  - [x] `#[cfg(feature = "cot")] pub mod cot;`
  - [x] `pub mod common;` (always compiled)
  - [x] Public re-exports from `lib.rs`
- [x] Task 4: Verify compilation (AC: #5, #6)
  - [x] `cargo check -p traitclaw-strategies`
  - [x] `cargo check -p traitclaw-strategies --no-default-features`
  - [x] `cargo test -p traitclaw-strategies` (6 unit + 1 doc = 7 tests pass)

## Dev Notes

- Followed existing crate patterns from `traitclaw-steering`
- Used `edition = "2021"` and workspace MSRV 1.75+ via `workspace = true`
- Feature flags are additive, no inter-flag dependencies (AR5)
- ThoughtStep enum also implemented (Story 1.2 content included for convenience)
- `#[serde(tag = "type")]` for clean JSON serialization

### Project Structure Notes

- Path: `crates/traitclaw-strategies/` (consistent with all other crates)
- Workspace Cargo.toml: registered in members + workspace.dependencies

### References

- [Source: architecture.md#Decision-1-Strategy-Module-Structure]
- [Source: architecture.md#Decision-5-Feature-Flag-Design]

## Dev Agent Record

### Agent Model Used
Gemini 2.5 Pro

### Debug Log References
- `cargo check -p traitclaw-strategies` → OK (3.31s)
- `cargo check --no-default-features` → OK (0.17s)
- `cargo test -p traitclaw-strategies` → 6 unit + 1 doc = 7 pass (0.00s + 0.63s)

### Completion Notes List
- Created traitclaw-strategies crate with full scaffold
- ThoughtStep enum with 4 variants + tagged JSON serialization
- 6 unit tests (serialize each variant + clone + debug) + 1 doc test
- Feature flags: react, mcts, cot (default all-on)
- Stub modules for react/, mcts/, cot/

### File List
- crates/traitclaw-strategies/Cargo.toml (NEW)
- crates/traitclaw-strategies/src/lib.rs (NEW)
- crates/traitclaw-strategies/src/common/mod.rs (NEW)
- crates/traitclaw-strategies/src/common/thought_step.rs (NEW)
- crates/traitclaw-strategies/src/react/mod.rs (NEW)
- crates/traitclaw-strategies/src/mcts/mod.rs (NEW)
- crates/traitclaw-strategies/src/cot/mod.rs (NEW)
- Cargo.toml (MODIFIED - added workspace member + dependency)
