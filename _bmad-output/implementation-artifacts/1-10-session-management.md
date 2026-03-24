# Story 1.10: Session Management

Status: review

## Story

As a developer,
I want to manage multiple conversations with separate session IDs,
So that different users/conversations have isolated memory.

## Acceptance Criteria

1. **Given** the `Memory` trait is extended with session lifecycle methods **When** I call `agent.session("user-123")` **Then** it returns an `AgentSession` wrapper bound to that session ID
2. **And** `session.say("Hello")` uses the bound session ID for memory operations
3. **And** `agent.session_auto()` creates a new session with UUID and returns `AgentSession`
4. **And** `agent.run(input)` is backward-compatible (auto-creates ephemeral session)
5. **And** `runtime.rs` and `streaming.rs` accept `session_id` parameter instead of hardcode `"default"`
6. **And** `Memory` trait has default impls: `create_session() -> String`, `list_sessions()`, `delete_session(id)`
7. **And** `InMemoryMemory` implements session lifecycle using internal HashMap keys

## Tasks / Subtasks

- [x] Task 1: `AgentSession` wrapper (AC: 1, 2)
  - [x] `AgentSession<'a>` struct with `agent` ref + `session_id`
  - [x] `say()` delegates to runtime with session
  - [x] `stream()` delegates to streaming with session
  - [x] `id()` getter
- [x] Task 2: Agent session methods (AC: 1, 3, 4)
  - [x] `agent.session(id)` returns `AgentSession`
  - [x] `agent.session_auto()` generates UUID v4
  - [x] `agent.run()` backward-compatible (uses "default" session)
- [x] Task 3: Parameterized runtime and streaming (AC: 5)
  - [x] `run_agent()` accepts `session_id` parameter
  - [x] `stream_agent()` accepts `session_id` parameter
- [x] Task 4: Memory trait session lifecycle (AC: 6, 7) — done in story 1.5
- [x] Task 5: Tests — existing tests pass with parameterized session_id

## Dev Notes

### Architecture Requirements
- `AgentSession` holds a reference/Arc to Agent, not a copy
- Backward compatibility is critical — `agent.run()` must still work without sessions
- Session IDs are strings — flexible for users to provide their own IDs
- UUID v4 for auto-generated session IDs

### References
- [Source: _bmad-output/epics.md#Story 1.10]

## Dev Agent Record

### Agent Model Used
gemini-2.5-pro (antigravity)

### Debug Log References
- `cargo test` (workspace) → all passed
- `cargo clippy --all-targets` → clean

### Completion Notes List
- Created `AgentSession` wrapper with `say()`/`stream()`/`id()`.
- Added `agent.session()` and `agent.session_auto()` to Agent.
- Parameterized `run_agent` and `stream_agent` with `session_id`.
- `agent.run()` backward-compatible using "default" session.

### File List
- `crates/traitclaw-core/src/agent.rs` (AgentSession + session methods)
- `crates/traitclaw-core/src/runtime.rs` (parameterized session_id)
- `crates/traitclaw-core/src/streaming.rs` (parameterized session_id)
- `crates/traitclaw-core/src/lib.rs` (exports + prelude)

### Change Log
- 2026-03-24: All tasks complete.
