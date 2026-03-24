# Story 1.14: Provider Retry with Backoff

Status: review

## Story

As a developer,
I want automatic retry with exponential backoff for transient provider errors,
So that my agent is resilient to temporary API issues.

## Acceptance Criteria

1. **Given** a `RetryProvider` wrapper in `traitclaw-core` **When** the inner provider returns a transient error (429, 500, 502, 503, 504, timeout) **Then** it retries up to `max_retries` times (default: 3)
2. **And** uses exponential backoff starting at `initial_delay` (default: 500ms)
3. **And** caps delay at `max_delay` (default: 30s)
4. **And** non-transient errors are propagated immediately without retry
5. **And** `RetryProvider` implements `Provider` trait (decorator pattern)
6. **And** `AgentBuilder::with_retry(config)` is a convenience method

## Tasks / Subtasks

- [x] Task 1: `RetryConfig` (AC: 1, 2, 3)
  - [x] max_retries=3, initial_delay=500ms, max_delay=30s
- [x] Task 2: `RetryProvider` decorator (AC: 1-5)
  - [x] Wraps `Arc<dyn Provider>`, implements Provider
  - [x] Exponential backoff with cap
  - [x] Retries both complete() and stream()
- [x] Task 3: Builder convenience (AC: 6)
  - [x] `.with_retry(config)` wraps provider
- [x] Task 4: Error classification (AC: 4)
  - [x] Added `status_code` to `Error::Provider`
  - [x] `is_retryable()` helper
- [x] Task 5: Tests (AC: all)
  - [x] Retry succeeds on second attempt
  - [x] Max retries exhausted
  - [x] Non-retryable error propagated immediately
  - [x] Exponential backoff timing
  - [x] Max delay cap

## Dev Notes

### Architecture Requirements
- Decorator pattern: RetryProvider wraps any Provider
- Must classify errors as transient vs permanent
- Error enum needs HTTP status code support for classification
- `tokio::time::sleep` for delays between retries

### Critical Patterns
- Jitter: consider adding random jitter to prevent thundering herd
- Log retries using `tracing::warn!`
- Don't retry on auth errors (401, 403)

### References
- [Source: _bmad-output/epics.md#Story 1.14]

## Dev Agent Record

### Agent Model Used
gemini-2.5-pro (antigravity)

### Debug Log References
- `cargo test` (workspace) → all passed
- `cargo clippy --all-targets` → clean

### Completion Notes List
- Created `RetryProvider` decorator and `RetryConfig` from scratch.
- Extended `Error::Provider` with optional `status_code` field.
- Added `is_retryable()` helper method to Error enum.
- Added `.with_retry()` convenience method to AgentBuilder.

### File List
- `crates/traitclaw-core/src/retry.rs` (NEW)
- `crates/traitclaw-core/src/error.rs` (status_code + is_retryable)
- `crates/traitclaw-core/src/lib.rs` (module + exports)
- `crates/traitclaw-core/src/agent_builder.rs` (with_retry convenience)

### Change Log
- 2026-03-24: All tasks complete.
