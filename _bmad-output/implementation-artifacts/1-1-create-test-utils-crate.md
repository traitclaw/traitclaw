# Story 1.1: Create `traitclaw-test-utils` Crate Scaffold

Status: review

## Story

As a framework contributor,
I want a dedicated `traitclaw-test-utils` crate in the workspace,
so that I have a central place for all shared test utilities.

## Acceptance Criteria

1. `crates/traitclaw-test-utils/Cargo.toml` exists with workspace-inherited metadata (version, edition, rust-version, license)
2. `crates/traitclaw-test-utils/src/lib.rs` compiles with `cargo build -p traitclaw-test-utils`
3. Root `Cargo.toml` includes `"crates/traitclaw-test-utils"` in workspace members list
4. Crate has proper `[dependencies]` on `traitclaw-core` (workspace path), `async-trait`, `tokio`, `serde_json`
5. `lib.rs` has module structure with public re-exports: `pub mod provider;`, `pub mod memory;`, `pub mod tools;`, `pub mod runtime;`
6. Crate compiles with `cargo check --workspace` (no workspace-level breakage)
7. `cargo doc -p traitclaw-test-utils --no-deps` succeeds with crate-level doc comments

## Tasks / Subtasks

- [x] Task 1: Create crate directory and Cargo.toml (AC: #1, #4)
  - [x] Create `crates/traitclaw-test-utils/` directory
  - [x] Create `Cargo.toml` with workspace-inherited fields
  - [x] Add dependencies: `traitclaw-core = { path = "../traitclaw-core" }`, `async-trait`, `tokio`, `serde_json`
- [x] Task 2: Add to workspace members (AC: #3)
  - [x] Add `"crates/traitclaw-test-utils"` to root `Cargo.toml` members array
- [x] Task 3: Create module structure (AC: #5, #7)
  - [x] Create `src/lib.rs` with crate-level doc comment and module declarations
  - [x] Create `src/provider.rs` — empty module for MockProvider (Story 1.2)
  - [x] Create `src/memory.rs` — empty module for MockMemory (Story 1.3)
  - [x] Create `src/tools.rs` — empty module for EchoTool/FailTool (Story 1.3)
  - [x] Create `src/runtime.rs` — empty module for make_runtime (Story 1.4)
- [x] Task 4: Verify compilation (AC: #2, #6)
  - [x] Run `cargo build -p traitclaw-test-utils`
  - [x] Run `cargo check --workspace`
  - [x] Run `cargo doc -p traitclaw-test-utils --no-deps`

## Dev Notes

### Workspace Crate Pattern (MUST FOLLOW)

All crates in this workspace follow an identical pattern. Reference `crates/traitclaw-core/Cargo.toml` as the canonical example:

```toml
[package]
name = "traitclaw-test-utils"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
description = "Shared test utilities for the TraitClaw AI Agent Framework"

[dependencies]
traitclaw-core = { path = "../traitclaw-core" }
async-trait = { workspace = true }
tokio = { workspace = true }
serde_json = { workspace = true }
```

### Workspace Metadata (from root Cargo.toml)

- `version = "0.6.0"` (will bump to 0.8.0 in release)
- `edition = "2021"`
- `rust-version = "1.75"`

### Root Cargo.toml Members Array

The new crate must be added to the existing workspace members. Current members list ends around line 14. Add `"crates/traitclaw-test-utils"` after `"crates/traitclaw-strategies"`.

### Module Structure Intent

This crate is a scaffold — each module will be populated by subsequent stories:
- `provider.rs` → Story 1.2 (MockProvider)
- `memory.rs` + `tools.rs` → Story 1.3 (MockMemory, EchoTool, FailTool)
- `runtime.rs` → Story 1.4 (make_runtime helper)

For now, each module can be an empty file or contain a placeholder doc comment.

### Existing Mock Locations (Context Only)

These are the duplicate mocks that Story 1.5 will migrate. Do NOT touch these files in this story:
- `crates/traitclaw-strategies/src/test_utils.rs` — MockProvider, MockMemory, make_runtime (224 lines)
- `crates/traitclaw-core/src/agent.rs` (line 579) — inline MockProvider in test module
- `crates/traitclaw-core/src/transformers.rs` (line 442) — inline MockProvider in test module
- `crates/traitclaw-core/src/traits/provider.rs` (line 62) — inline MockProvider in test module
- `crates/traitclaw-core/tests/integration.rs` (line 18) — MockProvider

### Key Dependencies from `traitclaw-core`

The test-utils crate will use these types (populated in later stories):
- `Provider` trait from `traitclaw_core::traits::provider`
- `Memory` trait from `traitclaw_core::Memory`
- `AgentRuntime` struct from `traitclaw_core::traits::strategy`
- `CompletionRequest`, `CompletionResponse`, `Usage` from `traitclaw_core::types::completion`
- `Message` from `traitclaw_core::types::message`
- `ErasedTool` from `traitclaw_core::traits::tool`

### Project Structure Notes

- Crate lives at: `crates/traitclaw-test-utils/`
- Follows naming convention: `traitclaw-{component}`
- Package name: `traitclaw-test-utils`
- Rust crate name (for imports): `traitclaw_test_utils`
- No conflicts with existing crates detected

### References

- [Source: Cargo.toml — workspace members and package metadata]
- [Source: crates/traitclaw-core/Cargo.toml — crate pattern reference]
- [Source: crates/traitclaw-strategies/src/test_utils.rs — existing mock patterns]
- [Source: _bmad-output/planning-artifacts/epics-v0.8.0.md — Epic 1 requirements]
- [Source: _bmad-output/planning-artifacts/prd-v0.8.0.md — FR1-FR5 definitions]

## Dev Agent Record

### Agent Model Used
Antigravity (Google DeepMind)

### Debug Log References
- Doc examples use `ignore` annotation since types are scaffolded (not yet populated — Stories 1.2-1.4)
- Added `serde` and `schemars` deps proactively (tool mocks in Story 1.3 will need them)

### Completion Notes List
- ✅ Created `crates/traitclaw-test-utils/Cargo.toml` with workspace-inherited metadata
- ✅ Added crate to workspace members in root `Cargo.toml` (after `traitclaw-strategies`)
- ✅ Created `src/lib.rs` with 4 public modules and comprehensive crate-level docs
- ✅ Created placeholder modules: `provider.rs`, `memory.rs`, `tools.rs`, `runtime.rs` — each with module-level docs
- ✅ `cargo build -p traitclaw-test-utils` — success
- ✅ `cargo check --workspace` — success (no breakage)
- ✅ `cargo doc -p traitclaw-test-utils --no-deps` — success (doc link warnings expected for future types)
- ✅ `cargo test --workspace` — all tests pass, zero regressions

### File List
- NEW: `crates/traitclaw-test-utils/Cargo.toml`
- NEW: `crates/traitclaw-test-utils/src/lib.rs`
- NEW: `crates/traitclaw-test-utils/src/provider.rs`
- NEW: `crates/traitclaw-test-utils/src/memory.rs`
- NEW: `crates/traitclaw-test-utils/src/tools.rs`
- NEW: `crates/traitclaw-test-utils/src/runtime.rs`
- MODIFIED: `Cargo.toml` (added `crates/traitclaw-test-utils` to workspace members)
