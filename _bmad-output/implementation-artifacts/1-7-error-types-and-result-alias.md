# Story 1.7: Error Types & Result Alias

Status: review

## Story

As a developer,
I want well-defined error types for traitclaw-core,
So that errors are descriptive and actionable.

## Acceptance Criteria

1. **Given** the error module exists **When** an error occurs during agent execution **Then** it returns a typed `Error` enum variant (not a string)
2. **And** variants include: Provider, ToolExecution, Memory, Config, Runtime
3. **And** `type Result<T> = std::result::Result<T, Error>` alias exists
4. **And** errors implement `std::error::Error` via `thiserror`

## Tasks / Subtasks

- [x] Task 1: `error.rs` (AC: 1, 2, 3, 4) — pre-existed
  - [x] `Error` enum with thiserror derive
  - [x] All 5 variants: Provider, ToolExecution, Memory, Config, Runtime
  - [x] `#[from]` conversions for `io::Error`, `serde_json::Error`
  - [x] `type Result<T>` alias
- [x] Task 2: Tests (AC: all)
  - [x] Display tests for all 5 variants
  - [x] `#[from]` conversion tests (io::Error, serde_json::Error)
  - [x] Result alias with `?` operator
  - [x] `std::error::Error` trait implementation check

## Dev Notes

### Architecture Requirements
- Each crate has its own Error enum using `thiserror` — NO anyhow in library code
- Provider error variant will later have inner ProviderError type
- `String` error messages for MVP, can be refined to structured errors later
- `#[must_use]` on Result-returning functions

### Error Handling Pattern
```rust
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Provider error: {0}")]
    Provider(String),
    #[error("Tool execution failed: {tool_name}: {message}")]
    ToolExecution { tool_name: String, message: String },
    #[error("Memory error: {0}")]
    Memory(String),
    #[error("Configuration error: {0}")]
    Config(String),
    #[error("Runtime error: {0}")]
    Runtime(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}
```

### References
- [Source: _bmad-output/project-context.md#Error Handling Pattern]
- [Source: _bmad-output/project-context.md#Anti-Patterns to AVOID]

## Dev Agent Record

### Agent Model Used
gemini-2.5-pro (antigravity)

### Debug Log References
- `cargo test` (workspace) → all passed
- `cargo clippy --all-targets` → clean

### Completion Notes List
- error.rs pre-existed with 3 tests. Added 5 new tests covering remaining ACs.

### File List
- `crates/traitclaw-core/src/error.rs` (5 new tests)

### Change Log
- 2026-03-24: All tasks complete.
