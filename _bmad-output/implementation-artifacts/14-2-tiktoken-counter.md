# Story 14.2: TikTokenCounter Implementation

Status: done

## Story

As a developer who needs accurate token counting,
I want exact OpenAI-compatible tokenization,
so that context budget decisions are precise.

## Acceptance Criteria

1. `TikTokenCounter` is implemented behind `"tiktoken"` feature flag in `traitclaw-core`
2. `TikTokenCounter::for_model("gpt-4o")` creates a counter with the correct encoding
3. `count_tokens(&[Message])` returns exact token count
4. Unknown models fall back to `cl100k_base` encoding
5. Can be used as the `estimate_tokens()` override in `ContextManager` implementations
6. `tiktoken-rs` is an optional dependency (not pulled without feature flag)
7. Without feature flag, `TikTokenCounter` type is not available
8. Accuracy test: CharApprox vs TikToken on 100 sample messages shows < 2% error for English
9. Re-exported via `traitclaw::prelude::*` when feature is enabled

## Tasks / Subtasks

- [x] Task 1: Add `tiktoken-rs = { version = "0.6", optional = true }` + `[features]` section
- [x] Task 2: `token_counter.rs` with `TikTokenCounter` gated by `#[cfg(feature = "tiktoken")]`
- [x] Task 3: `estimate_for_model(messages, model)` standalone helper (AC: #5)
- [x] Task 4: 9 unit tests (AC: #3, #4, #7, #8)
- [x] Task 5: Re-exports in `lib.rs` + `tiktoken` feature forwarding in meta-crate

## Dev Notes

- New file: `crates/traitclaw-core/src/token_counter.rs`
- Entire module gated behind `#[cfg(feature = "tiktoken")]`
- `tiktoken-rs` crate provides `CoreBPE` and encoding lookup by model name
- Token counting per message: `<|im_start|>role\n{content}<|im_end|>` ≈ content_tokens + 4
- `cl100k_base` is the default encoding for GPT-4, GPT-3.5-turbo, etc.
- CharApprox baseline: 4 chars ≈ 1 token (existing `estimate_tokens()` logic)

### References

- [Source: epics-v0.4.0.md#Epic 14, Story 14.2]
- [Source: context_managers.rs] — estimate_tokens() helper function
- [tiktoken-rs docs] — https://docs.rs/tiktoken-rs

## Dev Agent Record

### Agent Model Used
Gemini 2.5 Flash

### Completion Notes List
- 9/9 unit tests pass with `--features tiktoken`
- `cargo check` without feature passes cleanly
- o200k_base for gpt-4o/o1/o3/o4; cl100k_base for gpt-4/gpt-3.5; fallback for unknowns
- `tiktoken` feature forwarded in meta-crate `traitclaw`
- `MessageRole` used via match (no `as_str()` method)

### File List
- `crates/traitclaw-core/src/token_counter.rs` — new module
- `crates/traitclaw-core/Cargo.toml` — tiktoken-rs optional dep + features
- `crates/traitclaw-core/src/lib.rs` — conditional re-export
- `crates/traitclaw/Cargo.toml` — tiktoken feature forwarding

### Change Log
- 2026-03-26: Initial implementation complete
