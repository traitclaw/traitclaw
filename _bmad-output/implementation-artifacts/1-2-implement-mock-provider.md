# Story 1.2: Implement `MockProvider` with Response Sequences

Status: review

## Story

As a framework contributor,
I want a `MockProvider` that returns pre-defined responses in sequence,
so that I can write deterministic tests without hitting real LLM APIs.

## Acceptance Criteria

1. `MockProvider::text("hello")` creates a provider returning a single text response
2. `MockProvider::sequence(vec![r1, r2])` creates a provider returning `r1` then `r2` in order
3. `MockProvider::tool_then_text(tool_calls, "final")` returns tool calls first, then text
4. Calling `complete()` beyond the sequence returns the **last** response (wrap-around to last)
5. `MockProvider` implements `Provider + Send + Sync`
6. All factory methods and the struct have doc comments with `/// # Example` blocks
7. `cargo test -p traitclaw-test-utils` passes with unit tests covering all factory methods

## Tasks / Subtasks

- [x] Task 1: Implement `MockProvider` struct (AC: #1, #2, #5)
  - [x] Define `pub struct MockProvider` with fields: `info: ModelInfo`, `responses: Vec<CompletionResponse>`, `call_idx: AtomicUsize`
  - [x] Implement `MockProvider::text(text: &str) -> Self` — single text response
  - [x] Implement `MockProvider::sequence(responses: Vec<CompletionResponse>) -> Self`
  - [x] Implement `Provider` trait: `complete()` uses `AtomicUsize` for lock-free indexing, clamps to last response
  - [x] `stream()` → `unimplemented!()` (consistent with existing pattern)
  - [x] `model_info()` returns `ModelInfo::new("mock-model", ModelTier::Small, 4096, false, false, false)`
- [x] Task 2: Add convenience factory methods (AC: #3)
  - [x] `MockProvider::tool_then_text(tool_calls: Vec<ToolCall>, final_text: &str) -> Self`
  - [x] `MockProvider::always_tool_calls(tool_calls: Vec<ToolCall>) -> Self`
  - [x] `MockProvider::error(msg: &str) -> Self` — returns `Err(Error::Runtime(msg))`
- [x] Task 3: Add doc comments and examples (AC: #6)
  - [x] Module-level doc comment on `provider.rs`
  - [x] `/// # Example` on `text()`, `sequence()`, `tool_then_text()`
- [x] Task 4: Write unit tests (AC: #4, #7)
  - [x] Test `text()` returns correct response
  - [x] Test `sequence()` returns in order
  - [x] Test calling beyond sequence length returns last response
  - [x] Test `tool_then_text()` returns tool calls then text
  - [x] Test `Send + Sync` static assertions

## Dev Notes

### Existing Pattern (copy from, then delete in Story 1.5)

The canonical implementation is at `crates/traitclaw-strategies/src/test_utils.rs` lines 30-116. The v0.8.0 version should be **public** (`pub`) instead of `pub(crate)`:

```rust
// Key imports needed
use std::sync::atomic::{AtomicUsize, Ordering};
use async_trait::async_trait;
use traitclaw_core::traits::provider::Provider;
use traitclaw_core::types::completion::*;
use traitclaw_core::types::model_info::{ModelInfo, ModelTier};
use traitclaw_core::types::stream::CompletionStream;
use traitclaw_core::types::tool_call::ToolCall;
```

### Key Differences from Existing Mock

| Aspect | Existing (strategies) | New (test-utils) |
|--------|----------------------|------------------|
| Visibility | `pub(crate)` | `pub` |
| Name | `MockProvider` | `MockProvider` (same) |
| Error factory | None | Add `error()` |
| Doc comments | Minimal | Full with examples |
| Tests | None (inline use) | Dedicated module |

### Send + Sync Static Assertion Pattern

```rust
#[test]
fn mock_provider_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<MockProvider>();
}
```

### References

- [crates/traitclaw-strategies/src/test_utils.rs](file:///Users/admin/Desktop/Projects/traitclaw/crates/traitclaw-strategies/src/test_utils.rs) — existing MockProvider (lines 30-116)
- [crates/traitclaw-core/src/traits/provider.rs](file:///Users/admin/Desktop/Projects/traitclaw/crates/traitclaw-core/src/traits/provider.rs) — Provider trait definition
- [_bmad-output/planning-artifacts/prd-v0.8.0.md](file:///Users/admin/Desktop/Projects/traitclaw/_bmad-output/planning-artifacts/prd-v0.8.0.md) — FR1, FR2

## Dev Agent Record

### Agent Model Used
Antigravity (Google DeepMind)

### Debug Log References
- Doc examples compile and pass as doc-tests (6 doc-tests pass)
- Added `call_count()` method and `error_message` field beyond story spec — useful utilities
- Added `default_usage()` helper to DRY usage construction

### Completion Notes List
- ✅ `MockProvider` struct with `AtomicUsize` lock-free indexing — `Send + Sync` verified
- ✅ Factory methods: `text()`, `sequence()`, `tool_then_text()`, `always_tool_calls()`, `error()`
- ✅ `Provider` trait impl with clamp-to-last behavior when sequence exhausted
- ✅ Full doc comments with `# Example` blocks on all public items
- ✅ 10 unit tests: text, sequence ordering, clamp behavior, tool_then_text, error, always_tool_calls, send+sync, call_count, model_info
- ✅ 6 doc-tests compile and pass
- ✅ `cargo test --workspace` — all pass, zero regressions

### Change Log
- Implemented MockProvider in `crates/traitclaw-test-utils/src/provider.rs` (Date: 2026-03-28)

### File List
- MODIFIED: `crates/traitclaw-test-utils/src/provider.rs`
