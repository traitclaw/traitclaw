# Story 1.8: Streaming Support

Status: review

## Story

As a developer,
I want `agent.stream("Hello")` to return an async stream of response chunks,
So that I can display responses incrementally.

## Acceptance Criteria

1. **Given** an Agent with a provider that supports streaming **When** I call `agent.stream("Hello")` **Then** it returns `Result<AgentStream>`
2. **And** AgentStream implements `Stream<Item = Result<StreamEvent>>`
3. **And** StreamEvent has variants: TextDelta, ToolCallStart, ToolCallDelta, Done
4. **And** the stream properly handles tool calls mid-stream (pause stream → execute → resume)

## Tasks / Subtasks

- [x] Task 1: `StreamEvent` enum (AC: 3) — pre-existed in `stream.rs`
- [x] Task 2: `AgentStream` type (AC: 2) — pre-existed in `streaming.rs`
- [x] Task 3: `agent.stream()` + `streaming.rs` (AC: 1, 4) — pre-existed
- [x] Task 4: Tests (AC: all)
  - [x] AC-2/3: text-only stream yields TextDelta + Done
  - [x] AC-3: Done always last event
  - [x] AC-1: memory saved after streaming completes

## Dev Notes

### Architecture Requirements
- Uses `tokio-stream` and `async-stream` crates
- `AgentStream` wraps a pinned boxed stream for ergonomics
- Tool calls mid-stream: accumulate tool call deltas → when complete → pause stream → execute tool → add result → resume with new LLM call
- Same Guard/Hint/Tracker hooks as runtime loop

### Critical Patterns
- Use `async_stream::stream!` macro for creating streams
- SSE parsing will be done in provider crates, not here — core just defines the stream types
- `streaming.rs` is separate from `runtime.rs` but shares context assembly logic

### References
- [Source: _bmad-output/architecture.md#3.5 Agent - stream method]
- [Source: _bmad-output/project-context.md#Performance Rules - Streaming]

## Dev Agent Record

### Agent Model Used
gemini-2.5-pro (antigravity)

### Debug Log References
- `cargo test` (workspace) → all passed
- `cargo clippy --all-targets` → clean

### Completion Notes List
- All streaming code pre-existed. Added 3 tests with `StreamingMockProvider`.
- Fixed unused import and `redundant_pattern_matching` clippy lint.

### File List
- `crates/traitclaw-core/src/streaming.rs` (3 new tests)

### Change Log
- 2026-03-24: All tasks complete.
