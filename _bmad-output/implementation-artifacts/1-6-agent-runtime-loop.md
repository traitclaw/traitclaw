# Story 1.6: Agent Runtime Loop

Status: review

## Story

As a developer,
I want the agent runtime loop that orchestrates LLM calls and tool execution,
So that `agent.run("Hello")` produces a complete response.

## Acceptance Criteria

1. **Given** an Agent with a provider and system prompt **When** I call `agent.run("Hello")` **Then** it assembles context (system prompt + memory + user message)
2. **And** sends CompletionRequest to provider
3. **And** if response is text → returns AgentOutput::Text
4. **And** if response is tool_calls → executes tools → feeds results back → loops
5. **And** loop terminates when LLM returns text (not tool calls)
6. **And** conversation is saved to memory after completion
7. **And** Guard/Hint/Tracker hook points exist (Noop by default)

## Tasks / Subtasks

- [x] Task 1: `types/agent_state.rs` (AC: 7) — pre-existed
- [x] Task 2: `types/action.rs` (AC: 7) — pre-existed
- [x] Task 3: `runtime.rs` (AC: 1-7) — pre-existed with full loop
- [x] Task 4: `AgentOutput` enum (AC: 3) — pre-existed in `agent.rs`
- [x] Task 5: Tests (AC: all)
  - [x] AC-3: simple text response
  - [x] AC-4/5: tool call → result → text loop
  - [x] AC-5: max iterations error
  - [x] AC-6: memory saved after completion
  - [x] AC-7: guard deny blocks tool execution

## Dev Notes

### Architecture Requirements
- The runtime loop is the HEART of the framework — see architecture.md diagram
- Flow: Input → Context → LLM → Parse → (Text? return) || (ToolCall? guard → execute → loop)
- Guard/Hint/Tracker hooks must exist but Noop by default
- Tool execution currently inline — Story 1.12 extracts to ExecutionStrategy

### Critical Patterns
- `agent.run()` calls into runtime module, not inline
- Use `for i in 0..config.max_iterations` loop with guard
- Each iteration: check Hints → build request → call LLM → parse response → if tool calls: check Guards → execute tools → track
- Tool execution: match by name from `self.tools`, deserialize args, call execute
- Error from tool → return error message to LLM as tool result (don't crash)
- `catch_unwind` not needed yet (Story 7.4)

### Dependencies on Other Stories
- Story 1.2: Message, CompletionRequest/Response types
- Story 1.3: Provider trait
- Story 1.4: Agent struct, AgentConfig
- Story 1.5: Memory trait
- Story 1.7: Error types

### References
- [Source: _bmad-output/architecture.md#4 Agent Runtime — The Loop]
- [Source: _bmad-output/architecture.md#3.4 Guard/Hint/Tracker]

## Dev Agent Record

### Agent Model Used
gemini-2.5-pro (antigravity)

### Debug Log References
- `cargo test` (workspace) → all passed
- `cargo clippy --all-targets` → clean
- `cargo fmt --all --check` → clean

### Completion Notes List
- All code pre-existed. Only gap was Task 5 (tests): added 5 tests with `SequenceProvider` (configurable response sequence) and `EchoTool` mocks.
- Fixed clippy `unnecessary_literal_bound` on mock Tool impls.

### File List
- `crates/traitclaw-core/src/runtime.rs` (5 new tests)

### Change Log
- 2026-03-24: All tasks complete.
