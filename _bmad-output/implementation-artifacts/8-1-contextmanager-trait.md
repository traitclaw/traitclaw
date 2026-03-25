# Story 1.1: ContextManager Trait Definition

Status: review

## Story

As a framework developer,
I want the async `ContextManager` trait defined in `traitclaw-core`,
so that context management is pluggable and supports LLM-powered compression.

## Acceptance Criteria

1. ✅ `traitclaw-core/src/traits/context_manager.rs` exists with async `ContextManager` trait
2. ✅ `prepare()` is async, accepts `&mut Vec<Message>`, `context_window: usize`, `&mut AgentState`
3. ✅ `estimate_tokens()` has a default impl using 4-chars ≈ 1-token approximation
4. ✅ Blanket impl `impl<T: ContextStrategy + 'static> ContextManager for T` exists
5. ✅ `ContextStrategy` marked `#[deprecated(since = "0.3.0", note = "Use ContextManager")]`
6. ✅ Trait requires `Send + Sync`
7. ✅ Rustdoc with usage example
8. ✅ Unit test confirms trait is object-safe (`Arc<dyn ContextManager>` compiles)

## Tasks / Subtasks

- [x] Task 1: Create `context_manager.rs` with trait definition (AC: #1, #2, #3, #6)
  - [x] Define async trait with `prepare()` and `estimate_tokens()` methods
  - [x] Add `Send + Sync` bounds
  - [x] Add default `estimate_tokens()` implementation (4-chars ≈ 1-token)
- [x] Task 2: Implement blanket impl for `ContextStrategy` (AC: #4)
  - [x] `impl<T: ContextStrategy + 'static> ContextManager for T` wrapping sync `prepare()`
- [x] Task 3: Mark `ContextStrategy` as deprecated (AC: #5)
  - [x] Add `#[deprecated]` attribute to `ContextStrategy` trait
  - [x] Suppress deprecation warnings in existing impls with `#[allow(deprecated)]`
- [x] Task 4: Register module and add re-exports (AC: #1)
  - [x] Add `pub mod context_manager;` to `traits.rs`
  - [x] Add re-exports to `lib.rs` and `prelude`
- [x] Task 5: Add rustdoc with examples (AC: #7)
- [x] Task 6: Add unit tests (AC: #8)
  - [x] Object-safety test (`Arc<dyn ContextManager>`)
  - [x] Blanket impl test (ContextStrategy → ContextManager)
  - [x] Default `estimate_tokens()` test
- [x] Task 7: Run full test suite (regression check)

## Dev Agent Record

### Agent Model Used

Gemini 2.5 Pro

### Completion Notes List

- Used `#[async_trait]` instead of native `async fn in trait` for object safety (`Arc<dyn ContextManager>`)
- Added `#[allow(deprecated)]` to 7 locations: `Agent` struct/impl, `AgentBuilder` struct/impl, `AgentRuntime` struct, `DefaultStrategy` impl, `context_strategy.rs` impls/tests
- Blanket impl correctly delegates sync `ContextStrategy::prepare()` within async wrapper
- All 3 new tests pass, all 130+ existing tests pass, no regressions

### File List

- `crates/traitclaw-core/src/traits/context_manager.rs` — NEW
- `crates/traitclaw-core/src/traits/context_strategy.rs` — MODIFIED (deprecated)
- `crates/traitclaw-core/src/traits.rs` — MODIFIED (module registration)
- `crates/traitclaw-core/src/lib.rs` — MODIFIED (re-exports)
- `crates/traitclaw-core/src/agent.rs` — MODIFIED (#[allow(deprecated)])
- `crates/traitclaw-core/src/agent_builder.rs` — MODIFIED (#[allow(deprecated)])
- `crates/traitclaw-core/src/traits/strategy.rs` — MODIFIED (#[allow(deprecated)])
- `crates/traitclaw-core/src/default_strategy.rs` — MODIFIED (#[allow(deprecated)])

### Change Log

- 2026-03-25: Story 1.1 implemented — ContextManager async trait with blanket impl and deprecation
