# Story 3.2: Define `AgentEvent` Enum and Lifecycle Events

Status: ready-for-dev

## Story

As an agent developer,
I want a typed `AgentEvent` enum that represents the full agent lifecycle,
so that I can observe agent behavior programmatically without parsing logs.

## Acceptance Criteria

1. `AgentEvent` enum defined in `traitclaw-core/src/types/event.rs`
2. Variants: `LlmStart { model }`, `LlmEnd { model, prompt_tokens, completion_tokens, duration_ms }`, `ToolCall { tool_name, args }`, `ToolResult { tool_name, success, duration_ms }`, `GuardBlock { guard_name, reason }`, `HintTriggered { hint_name }`
3. `AgentEvent` derives `Debug`, `Clone`
4. `AgentEvent` is `#[non_exhaustive]` (FR28)
5. `AgentEvent` is `Send + Sync` (automatic for this enum)
6. `AgentEvent` is re-exported from `traitclaw-core` root and `prelude` module
7. All variants and fields have doc comments
8. Module registered in `types.rs` and `lib.rs`
9. Compilation passes: `cargo check --workspace`

## Tasks / Subtasks

- [ ] Task 1: Create event module (AC: #1, #2, #3, #4)
  - [ ] Create `crates/traitclaw-core/src/types/event.rs`
  - [ ] Define `AgentEvent` enum with all 6 variants
  - [ ] Add `#[derive(Debug, Clone)]` and `#[non_exhaustive]`
  - [ ] Add doc comment on enum and each variant
- [ ] Task 2: Register module (AC: #6, #8)
  - [ ] Add `pub mod event;` to `crates/traitclaw-core/src/types.rs`
  - [ ] Add `pub use types::event::AgentEvent;` to `crates/traitclaw-core/src/lib.rs`
  - [ ] Add `pub use crate::types::event::AgentEvent;` to `prelude` module
- [ ] Task 3: Tests (AC: #5, #9)
  - [ ] Send + Sync static assertion test
  - [ ] Debug format test for each variant
  - [ ] Clone test
  - [ ] `cargo check --workspace`

## Dev Notes

### Complete Type Definition (from Architecture v0.8.0)

```rust
//! Agent lifecycle events for runtime observability.
//!
//! [`AgentEvent`] represents discrete lifecycle moments during agent execution.
//! Register a callback via [`AgentBuilder::on_event()`] to receive events.
//!
//! # Example
//!
//! ```rust,no_run
//! use traitclaw_core::AgentEvent;
//!
//! fn handle_event(event: &AgentEvent) {
//!     match event {
//!         AgentEvent::LlmStart { model } => println!("LLM call to {model}"),
//!         AgentEvent::LlmEnd { prompt_tokens, completion_tokens, .. } => {
//!             println!("Used {prompt_tokens} + {completion_tokens} tokens");
//!         }
//!         _ => {} // #[non_exhaustive] — always include wildcard
//!     }
//! }
//! ```

/// Lifecycle events emitted by the agent runtime.
///
/// Use with [`AgentBuilder::on_event()`] to observe agent behavior
/// for logging, metrics, debugging, or cost tracking.
///
/// This enum is `#[non_exhaustive]` — new variants may be added in
/// future releases. Always include a wildcard arm when matching.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum AgentEvent {
    /// An LLM completion request is about to be sent.
    LlmStart {
        /// The model being called (e.g., "gpt-4o").
        model: String,
    },
    /// An LLM completion request has completed.
    LlmEnd {
        /// The model that was called.
        model: String,
        /// Number of tokens in the prompt.
        prompt_tokens: u32,
        /// Number of tokens in the completion.
        completion_tokens: u32,
        /// Duration of the LLM call in milliseconds.
        duration_ms: u64,
    },
    /// A tool is about to be called.
    ToolCall {
        /// Name of the tool being invoked.
        tool_name: String,
        /// Arguments passed to the tool.
        args: serde_json::Value,
    },
    /// A tool call has completed.
    ToolResult {
        /// Name of the tool that was called.
        tool_name: String,
        /// Whether the tool execution succeeded.
        success: bool,
        /// Duration of the tool call in milliseconds.
        duration_ms: u64,
    },
    /// A guard blocked an action.
    GuardBlock {
        /// Name of the guard that blocked.
        guard_name: String,
        /// Reason the action was blocked.
        reason: String,
    },
    /// A hint was triggered and injected.
    HintTriggered {
        /// Name of the hint that fired.
        hint_name: String,
    },
}
```

### Module Registration Pattern

```rust
// In crates/traitclaw-core/src/types.rs
pub mod event;  // ADD this line

// In crates/traitclaw-core/src/lib.rs — re-exports section
pub use types::event::AgentEvent;  // ADD after existing type re-exports

// In prelude module
pub use crate::types::event::AgentEvent;  // ADD
```

### serde_json Dependency

`serde_json` is already a dependency of `traitclaw-core` — `ToolCall.args` field in the enum uses `serde_json::Value`.

### References

- [_bmad-output/planning-artifacts/architecture-v0.8.0.md](file:///Users/admin/Desktop/Projects/traitclaw/_bmad-output/planning-artifacts/architecture-v0.8.0.md) — Decision 1: AgentEvent Design
- [crates/traitclaw-core/src/types.rs](file:///Users/admin/Desktop/Projects/traitclaw/crates/traitclaw-core/src/types.rs) — module registry
- [crates/traitclaw-core/src/lib.rs](file:///Users/admin/Desktop/Projects/traitclaw/crates/traitclaw-core/src/lib.rs) — re-export pattern
- FR16-FR19, FR28 in PRD v0.8.0

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
