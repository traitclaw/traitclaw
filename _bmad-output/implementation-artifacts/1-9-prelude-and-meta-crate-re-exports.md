# Story 1.9: Prelude & Meta-crate Re-exports

Status: review

## Story

As a developer,
I want `use traitclaw::prelude::*` to import everything I need,
So that getting started requires minimal import knowledge.

## Acceptance Criteria

1. **Given** the traitclaw meta-crate exists **When** I add `traitclaw = "0.1"` to my Cargo.toml **Then** `use traitclaw::prelude::*` imports Agent, AgentBuilder, Provider, Tool, Memory, Message types
2. **And** the meta-crate feature-gates optional crate re-exports
3. **And** default features provide a working agent with `openai-compat` provider

## Tasks / Subtasks

- [x] Task 1: prelude in traitclaw-core (AC: 1) — pre-existed, added AgentConfig + AgentState
- [x] Task 2: meta-crate lib.rs (AC: 1, 2) — pre-existed, added feature-gated `openai_compat`
- [x] Task 3: Feature flags (AC: 2, 3)
  - [x] `default = ["openai-compat", "macros"]`
  - [x] `openai-compat = ["dep:traitclaw-openai-compat"]`
- [x] Task 4: Compile-time validation (AC: all) — verified by workspace CI

## Dev Notes

### Architecture Requirements
- AD-1: Single `traitclaw` dependency with feature flags
- Default features: `["openai-compat", "macros"]` — enough for a working agent
- `prelude.rs` should be curated — only the most commonly used types
- Feature-gated re-exports use `#[cfg(feature = "...")]`

### Critical Patterns
- Only re-export types that users typically need
- Don't re-export internal implementation details
- Feature flags defined in traitclaw meta-crate Cargo.toml, not in core

### References
- [Source: _bmad-output/architecture.md#2 Architecture Overview - Single Dependency]
- [Source: _bmad-output/architecture.md#6 Core vs Optional]
- [Source: _bmad-output/architecture.md#AD-1, AD-5]

## Dev Agent Record

### Agent Model Used
gemini-2.5-pro (antigravity)

### Debug Log References
- `cargo test` (workspace) → all passed
- `cargo clippy --all-targets` → clean

### Completion Notes List
- Added `AgentConfig`, `AgentState` to prelude.
- Added `openai-compat` feature flag to meta-crate Cargo.toml.
- Added conditional `pub use traitclaw_openai_compat as openai_compat`.

### File List
- `crates/traitclaw-core/src/lib.rs` (prelude additions)
- `crates/traitclaw/Cargo.toml` (feature flags)
- `crates/traitclaw/src/lib.rs` (openai-compat re-export)

### Change Log
- 2026-03-24: All tasks complete.
