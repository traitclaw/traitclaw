# Story 1.1: Workspace & Crate Scaffolding

Status: review

## Story

As a developer,
I want the Cargo workspace and crate structure set up,
So that I can start implementing core types with proper module organization.

## Acceptance Criteria

1. **Given** a fresh clone of the repository **When** I run `cargo build` **Then** all crates compile successfully with no warnings
2. **And** the workspace structure matches project-context.md specifications
3. **And** `crates/traitclaw-core/src/lib.rs` exists with module declarations
4. **And** `crates/traitclaw/src/lib.rs` re-exports traitclaw-core

## Tasks / Subtasks

- [x] Task 1: Create root `Cargo.toml` workspace manifest (AC: 1, 2)
  - [x] Define `[workspace]` with members list for all crates
  - [x] Set resolver = "2"
  - [x] Add workspace-level `[workspace.dependencies]` for shared deps (tokio, serde, serde_json, thiserror, tracing, async-trait)
- [x] Task 2: Create `traitclaw-core` crate (AC: 3)
  - [x] `crates/traitclaw-core/Cargo.toml` with workspace deps
  - [x] `crates/traitclaw-core/src/lib.rs` with module declarations (`pub mod traits;`, `pub mod types;`, `pub mod error;`, `pub mod memory;`)
  - [x] Create empty module stub files: `traits/mod.rs` → NO! Use `traits.rs` with sub-modules
  - [x] Create placeholder module files: `error.rs`, `traits.rs`, `types.rs`
  - [x] Add `#![deny(warnings)]`, `#![deny(missing_docs)]`, `#![warn(clippy::pedantic)]`
- [x] Task 3: Create `traitclaw-macros` crate (AC: 2)
  - [x] `crates/traitclaw-macros/Cargo.toml` with proc-macro = true
  - [x] `crates/traitclaw-macros/src/lib.rs` placeholder
- [x] Task 4: Create `traitclaw` meta-crate (AC: 4)
  - [x] `crates/traitclaw/Cargo.toml` with dependency on traitclaw-core
  - [x] `crates/traitclaw/src/lib.rs` with `pub use traitclaw_core::*;`
  - [x] Add `#![deny(warnings)]`, `#![deny(missing_docs)]`
- [x] Task 5: Verify `cargo build` compiles cleanly (AC: 1)
  - [x] Run `cargo build --workspace`
  - [x] Run `cargo clippy --all-targets`
  - [x] Run `cargo fmt --all --check`

## Dev Notes

### Architecture Requirements
- Workspace uses Rust 2021 edition, MSRV 1.75+
- **No `mod.rs` files** — use `module_name.rs` + `module_name/` directory pattern (Rust 2018+)
- `lib.rs` only contains `pub mod` declarations and `pub use` re-exports (no logic)
- All public items must have doc comments
- Use `#![deny(warnings)]`, `#![deny(missing_docs)]` in core crates

### Workspace Dependencies (shared versions)
- `tokio` = { version = "1", features = ["full"] }
- `serde` = { version = "1", features = ["derive"] }
- `serde_json` = "1"
- `thiserror` = "1"
- `tracing` = "0.1"
- `async-trait` = "0.1"
- `schemars` = "0.8"

### Project Structure Notes
- Follow exact structure from `project-context.md` → Workspace Structure section
- Only create Phase 1 crates now: `traitclaw-core`, `traitclaw-macros`, `traitclaw`
- Other crates will be added in later stories

### References
- [Source: _bmad-output/project-context.md#Workspace Structure]
- [Source: _bmad-output/project-context.md#Structural Rules]
- [Source: _bmad-output/architecture.md#Internal Crate Architecture]

## Dev Agent Record

### Agent Model Used
gemini-2.5-pro (antigravity)

### Debug Log References
- `cargo build --workspace` → Success, exit code 0
- `cargo clippy --all-targets` → Success, no warnings
- `cargo fmt --all --check` → Formatting issues found in agent_builder.rs, in_memory.rs, runtime.rs, streaming.rs, types/completion.rs, macros/lib.rs, macros/tests/derive_tool.rs, openai/lib.rs, openai-compat/convert.rs, openai-compat/provider.rs, steering guards
- `cargo fmt --all` → Applied formatting fixes
- `cargo fmt --all --check` + `cargo clippy --all-targets` → Both pass after formatting

### Completion Notes List
- Workspace Cargo.toml already had correct structure: resolver = "2", all workspace deps, workspace.package settings
- `traitclaw-core` already had full module structure: lib.rs with `pub mod traits`, `pub mod types`, `pub mod error`, `pub mod memory`, plus `#![deny(warnings)]`, `#![deny(missing_docs)]`, `#![warn(clippy::pedantic)]`
- `traitclaw-macros` already had `proc-macro = true` in Cargo.toml and lib.rs implementation
- `traitclaw` meta-crate already had `pub use traitclaw_core::*;` and `pub use traitclaw_macros::Tool;` with deny attributes
- Applied `cargo fmt --all` to fix formatting issues across 10+ files (line wrapping style differences)
- All 4 acceptance criteria verified as satisfied

### File List
- `Cargo.toml` (formatting applied via `cargo fmt`)
- `crates/traitclaw-core/Cargo.toml`
- `crates/traitclaw-core/src/lib.rs`
- `crates/traitclaw-core/src/traits.rs`
- `crates/traitclaw-core/src/types.rs`
- `crates/traitclaw-core/src/error.rs`
- `crates/traitclaw-core/src/memory.rs`
- `crates/traitclaw-core/src/agent_builder.rs` (fmt fixed)
- `crates/traitclaw-core/src/runtime.rs` (fmt fixed)
- `crates/traitclaw-core/src/streaming.rs` (fmt fixed)
- `crates/traitclaw-core/src/types/completion.rs` (fmt fixed)
- `crates/traitclaw-core/src/memory/in_memory.rs` (fmt fixed)
- `crates/traitclaw-macros/Cargo.toml`
- `crates/traitclaw-macros/src/lib.rs` (fmt fixed)
- `crates/traitclaw-macros/tests/derive_tool.rs` (fmt fixed)
- `crates/traitclaw/Cargo.toml`
- `crates/traitclaw/src/lib.rs`
- `crates/traitclaw-openai/src/lib.rs` (fmt fixed)
- `crates/traitclaw-openai-compat/src/convert.rs` (fmt fixed)
- `crates/traitclaw-openai-compat/src/provider.rs` (fmt fixed)
- `crates/traitclaw-steering/src/guards/shell_deny.rs` (fmt fixed)
- `crates/traitclaw-steering/src/guards/workspace_boundary.rs` (fmt fixed)


### Change Log
- 2026-03-24: Verified workspace scaffold meets all ACs; applied `cargo fmt --all` to fix formatting inconsistencies across 10+ source files. All CI checks (build, clippy, fmt) now pass cleanly.

---

## Senior Developer Review (AI)

**Review Date:** 2026-03-24  
**Outcome:** Approved (patches resolved 2026-03-24)  
**Reviewer:** gemini-2.5-pro (code-review workflow)

### Action Items

#### Intent Gaps
- [ ] [High] IG-1: Story scope undermatch — repo was pre-seeded with full runtime/traits/streaming, not just "placeholder" modules. Document in project-context.md or story notes.
- [x] [Med] IG-2: Session ID hardcoded as `"default"` in runtime.rs:20 — concurrent agents will share memory. Add explicit single-session constraint comment and a test.

#### Patches
- [x] [Med] P-1: `AgentOutput` missing `#[non_exhaustive]` — future variant additions will silently break `.text()` callers. (`agent.rs:18`)
- [x] [High] P-2: `run_structured` needs FIXME comment clarifying async upgrade path in Story 3.3. (`agent.rs:118`)
- [x] [Low] P-3: `InMemoryMemory::recall("")` matches all entries, truncation is silent — doc comment added. (`in_memory.rs:77`)
- [x] [Med] P-4: `process_tool_calls` swallows errors — design comment added explaining intentional LLM-self-correct pattern. (`runtime.rs:110`)
- [x] [Med] P-5: Added `tool_arc()` and `tools_arc()` overloads for shared `Arc<dyn ErasedTool>`. (`agent_builder.rs`)
- [x] [Low] P-6: Added `#[must_use]` to `Error::provider()` and `Error::tool_execution()`. (`error.rs:50-62`)

#### Deferred (no action this story)
- [ ] [Low] D-1: Only 1 unit test for `AgentBuilder` — below 80% coverage target.
- [ ] [Low] D-2: Hint injection deduplication missing — same hint injects every iteration.
- [ ] [Low] D-3: `MemoryEntry` lacks timestamp — ordering undefined for long-term recall.
- [ ] [Low] D-4: `Cargo.toml` workspace members mix Phase 1 and future-story crates without grouping comments.

