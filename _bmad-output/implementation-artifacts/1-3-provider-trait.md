# Story 1.3: Provider Trait

Status: review

## Story

As a developer,
I want a `Provider` trait that abstracts LLM communication,
So that any LLM can be plugged in behind a common interface.

## Acceptance Criteria

1. **Given** the Provider trait is defined **When** I implement it for a mock provider **Then** `complete()` accepts `CompletionRequest` and returns `Result<CompletionResponse>`
2. **And** `stream()` returns `Result<CompletionStream>`
3. **And** `model_info()` returns `ModelInfo` with name, tier, context_window, capabilities
4. **And** `ModelTier` enum has Small, Medium, Large variants
5. **And** the trait requires `Send + Sync + 'static`

## Tasks / Subtasks

- [x] Task 1: Create `traits/provider.rs` (AC: 1, 2, 3, 5)
  - [x] Define `Provider` trait with `async fn complete()`, `async fn stream()`, `fn model_info()`
  - [x] Use `#[async_trait]` for async trait methods
  - [x] Add `Send + Sync + 'static` bounds
  - [x] Add doc comments with usage example
- [x] Task 2: Create `types/model_info.rs` (AC: 3, 4)
  - [x] Define `ModelInfo` struct (name, tier, context_window, supports_tools, supports_vision, supports_structured)
  - [x] Define `ModelTier` enum (Small, Medium, Large)
  - [x] Implement `Clone, Debug` for both
- [x] Task 3: Create `types/stream.rs` (AC: 2)
  - [x] Define `CompletionStream` as type alias for `Pin<Box<dyn Stream<Item = Result<StreamEvent>> + Send>>`
  - [x] Define `StreamEvent` enum for incremental chunks
- [x] Task 4: Create mock provider for testing (AC: 1, 5)
  - [x] `MockProvider` in test module implementing Provider trait
  - [x] Test that mock provider returns expected responses
  - [x] Test `Send + Sync + 'static` bounds compile correctly
- [x] Task 5: Wire up modules (AC: all)
  - [x] Add `model_info` to `types.rs`, re-export from `lib.rs` and `prelude`
  - [x] `cargo test` and `cargo clippy` pass workspace-wide

## Dev Notes

### Architecture Requirements
- Provider trait is THE core abstraction — trait-based for zero-cost (AD-3)
- `ModelInfo` includes `tier` field crucial for Guard-Hint-Track auto-config (AD-6)
- No third-party LLM SDK — providers call HTTP API directly via reqwest
- `CompletionStream` uses `tokio-stream` + `async-stream`

### Critical Patterns
- `#[async_trait]` required for async methods in traits
- `fn model_info(&self) -> &ModelInfo` — returns reference, not owned
- Mock provider should return configurable responses for testing runtime later
- `Arc<dyn Provider>` will be stored in Agent struct

### Dependencies to Add
- `async-trait` = "0.1" (workspace dep)
- `tokio-stream` = "0.1" (workspace dep)
- `futures-core` = "0.3" for Stream trait
- `pin-project-lite` = "0.2" (optional, for stream impls)

### References
- [Source: _bmad-output/architecture.md#3.1 Provider]
- [Source: _bmad-output/project-context.md#Rust-Specific Rules]

## Dev Agent Record

### Agent Model Used
gemini-2.5-pro (antigravity)

### Debug Log References
- `cargo test` (workspace) → 44 passed, 0 failed
- `cargo clippy --all-targets` → clean
- `cargo fmt --all --check` → clean

### Completion Notes List
- `traits/provider.rs` already had Provider trait + ModelInfo/ModelTier; moved ModelInfo/ModelTier to canonical `types/model_info.rs`
- `types/stream.rs` already had `CompletionStream` and `StreamEvent` enum (richer than `StreamDelta` spec)
- Added MockProvider to `traits/provider.rs` test module with 4 tests covering all 5 ACs
- `ModelTier` got `#[non_exhaustive]` + `Serialize/Deserialize` (consistent with project patterns)
- Downstream crates (`traitclaw-steering`, `traitclaw-anthropic`, `traitclaw-openai-compat`) updated with wildcard arms

### File List
- `crates/traitclaw-core/src/types/model_info.rs` [NEW]
- `crates/traitclaw-core/src/types.rs` (added model_info module)
- `crates/traitclaw-core/src/types/agent_state.rs` (updated import)
- `crates/traitclaw-core/src/traits/provider.rs` (imports from model_info, adds MockProvider tests)
- `crates/traitclaw-core/src/lib.rs` (re-exports updated)
- `crates/traitclaw-steering/src/trackers/adaptive.rs` (wildcard arm added)

### Change Log
- 2026-03-24: All tasks complete; 44 tests pass across workspace.

---

## Senior Developer Review (AI)

**Review Date:** 2026-03-24  
**Outcome:** Approved  
**Reviewer:** gemini-2.5-pro (code-review workflow)

### Action Items

#### Bad Spec
- [x] [Med] BS-1: Story spec updated — `StreamEvent` enum documented in Dev Notes.

#### Patches
- [x] [High] P-1: Added `#[non_exhaustive]` to `ModelInfo`; added `ModelInfo::new()` constructor for external crates; updated openai-compat and anthropic to use constructor.
- [x] [Med] P-2: Added `tracing::warn!` in `TierConfig::for_tier` wildcard arm.
- [x] [Med] P-3: Added `#[non_exhaustive]` to `StreamEvent`; documented `Done` contract in doc comment.

#### Deferred (no action this story)
- [ ] [Low] D-1: No error-path test for `complete()` — add configurable error mode to `MockProvider` in a future story.
- [ ] [Low] D-2: `StreamEvent::Done` contract undocumented — clarify if required-to-emit.
- [ ] [Low] D-3: `ModelInfo::context_window: usize` is platform-dependent — consider `u32`/`u64`.

