# Story 14.1: ProgressiveTransformer Implementation

Status: done

## Story

As a developer whose tools return large outputs,
I want outputs summarized first with full content available on demand,
so that most interactions save significant tokens.

## Acceptance Criteria

1. `ProgressiveTransformer` is implemented in `traitclaw-core` and implements the `OutputTransformer` trait
2. Phase 1: LLM-generated summary (configurable max length) is returned to the agent
3. Phase 2: full output is cached and available via virtual `__get_full_output` tool
4. `ProgressiveTransformer::new(provider, max_length)` accepts any `Provider`
5. `.with_summary_prompt("...")` allows custom summarization prompt
6. If LLM summarization fails, falls back to character truncation
7. If output is shorter than `max_summary_length`, output is passed through unchanged
8. Unit test: large output → summary returned, `__get_full_output` returns original
9. Unit test: short output → passed through without LLM call
10. Unit test: LLM failure → graceful fallback to truncation
11. Re-exported via `traitclaw::prelude::*`

## Tasks / Subtasks

- [x] Task 1: `ProgressiveTransformer` struct with `Arc<RwLock<HashMap>>` cache (AC: #1, #4)
- [x] Task 2: `OutputTransformer` impl with passthrough, LLM summarize, fallback (AC: #2, #5, #6, #7)
- [x] Task 3: `FullOutputRetriever` virtual tool via `retriever_tool()` (AC: #3)
- [x] Task 4: 5 unit tests (AC: #8, #9, #10)
- [x] Task 5: Re-exports (AC: #11)

## Dev Notes

- File: `crates/traitclaw-core/src/transformers.rs` (existing file with BudgetAwareTransformer, etc.)
- Follows `LlmCompressor` pattern from `context_managers.rs` for provider usage
- Cache is per-transformer instance — one `RwLock<HashMap>` stores recent outputs
- The `__get_full_output` virtual tool design needs careful consideration:
  - Option A: standalone tool that user registers manually
  - Option B: method on `ProgressiveTransformer` that returns the tool
  - Recommend Option B for cleaner API
- Default summary prompt: "Summarize the following tool output concisely, preserving key data points: {output}"

### References

- [Source: epics-v0.4.0.md#Epic 14, Story 14.1]
- [Source: transformers.rs] — existing OutputTransformer implementations
- [Source: context_managers.rs] — LlmCompressor pattern with Arc<dyn Provider>

## Dev Agent Record

### Agent Model Used
Gemini 2.5 Flash

### Completion Notes List
- All 12 transformer tests pass (5 new progressive + 7 existing)
- 2 doc-tests pass
- `ProgressiveTransformer` + `FullOutputRetriever` re-exported via `prelude::*`
- Discovered actual `CompletionRequest`/`CompletionResponse` API shapes during implementation

### File List
- `crates/traitclaw-core/src/transformers.rs` — ProgressiveTransformer + FullOutputRetriever
- `crates/traitclaw-core/src/lib.rs` — re-exports

### Change Log
- 2026-03-25: Initial implementation complete
