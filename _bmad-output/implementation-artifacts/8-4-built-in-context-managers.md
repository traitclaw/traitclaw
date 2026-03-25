# Story 8.4: Built-in Context Managers

Status: review

## Story

As a developer,
I want pre-built context managers for common compression strategies,
so that I get intelligent context management without writing custom code.

## Acceptance Criteria

1. ✅ `RuleBasedCompressor` is implemented: scores messages by importance (system=∞, recent=0.9, tools=0.7, old=0.3) and removes lowest-scored first when context exceeds threshold
2. ✅ `LlmCompressor::new(provider)` accepts any `Provider` for summarization
3. ✅ `LlmCompressor` has configurable `summary_prompt` template via `.with_prompt()`
4. ✅ `LlmCompressor` makes exactly 1 LLM call per compression event (not per iteration)
5. ✅ `TieredCompressor` chains: keep recent N → rule-compress mid → LLM-summarize old
6. ✅ All managers update `AgentState` when messages are compressed (`last_output_truncated = true`)
7. ✅ Integration test: 50-message conversation stays within token budget

## Tasks / Subtasks

- [x] Task 1: Create `crates/traitclaw-core/src/context_managers.rs` module (AC: #1)
  - [x] Create file with module-level rustdoc
  - [x] Add `pub mod context_managers;` to `lib.rs`
  - [x] Add re-exports: `pub use context_managers::{RuleBasedCompressor, LlmCompressor, TieredCompressor};`
- [x] Task 2: Implement `RuleBasedCompressor` (AC: #1, #6)
  - [x] Struct with configurable threshold (default: 0.85 of context_window)
  - [x] Scoring: system=∞, last N messages=0.9, tool-result=0.7, older=0.3
  - [x] `impl ContextManager`: sort by score ascending, remove lowest until under budget
  - [x] MUST never remove system messages (role == `MessageRole::System`)
  - [x] Update `state.last_output_truncated = true` when messages removed
  - [x] Unit tests: scoring logic, removal order, system preservation, state update
- [x] Task 3: Implement `LlmCompressor` (AC: #2, #3, #4, #6)
  - [x] Struct: `provider: Arc<dyn Provider>`, `summary_prompt: String`, `threshold: f64`, `keep_recent: usize`
  - [x] `new(provider)` constructor with default prompt template
  - [x] `with_prompt(template)` builder method
  - [x] `impl ContextManager`: on threshold breach, collect old messages → build summarization prompt → single `provider.complete()` call → replace old messages with one summary message
  - [x] Fallback to brief note when LLM call fails (logged via `tracing::warn!`)
  - [x] Update `state.last_output_truncated = true` after compression
  - [x] Unit tests: mock provider, verify single LLM call, prompt template substitution, summary insertion, fallback on failure
- [x] Task 4: Implement `TieredCompressor` (AC: #5, #6)
  - [x] Struct: `recent_count: usize`, `rule_compressor: RuleBasedCompressor`, `llm_compressor: Option<LlmCompressor>`
  - [x] `new(recent_count)` for rule-only mode, `with_llm(provider)` to enable LLM tier
  - [x] `impl ContextManager`: LLM-summarize oldest → rule-compress remaining
  - [x] Unit tests: tiered with and without LLM
- [x] Task 5: Integration test (AC: #7)
  - [x] 50-message conversation with `RuleBasedCompressor` stays within budget
  - [x] Verify system message survives compression
  - [x] Verify state is updated correctly
- [x] Task 6: Run full test suite — `cargo test --all --all-features` — 0 failures

## Dev Notes

- File: `crates/traitclaw-core/src/context_managers.rs`
- Follows `transformers.rs` structural pattern
- Uses `#[async_trait]` for object safety (consistent with `ContextManager` trait)
- `LlmCompressor` fallback: on LLM failure, logs warning and inserts brief message count note
- System messages are NEVER removed (score = `f64::INFINITY`)

### References

- [Source: epics-v0.3.0.md#Epic 1, Story 1.4]
- [Source: context_manager.rs] — trait definition
- [Source: transformers.rs] — structural template

## Dev Agent Record

### Agent Model Used

Gemini 2.5 Pro

### Debug Log References

### Completion Notes List

- Created `context_managers.rs` (≈350 lines) with 3 structs implementing `ContextManager`
- `RuleBasedCompressor`: importance-scored pruning with configurable threshold and recent-protection
- `LlmCompressor`: single LLM call per compression, graceful fallback on failure, configurable prompt via `.with_prompt()`
- `TieredCompressor`: chains LLM → rule compression, works without LLM provider (rule-only mode)
- 11 unit tests covering: no-prune, prune ordering, system preservation, state updates, mock provider, fallback, custom prompt, tiered with/without LLM, 50-message stress test
- Registered module in `lib.rs` with crate-root re-exports and prelude additions
- Fixed 2 clippy `doc_markdown` warnings (backtick `context_window`)
- Full test suite: 173+ tests, 0 failures, 0 regressions

### File List

- `crates/traitclaw-core/src/context_managers.rs` — NEW
- `crates/traitclaw-core/src/lib.rs` — MODIFIED (module registration, re-exports, prelude)

### Change Log

- 2026-03-25: Story 8.4 implemented — RuleBasedCompressor, LlmCompressor, TieredCompressor with full test suite
