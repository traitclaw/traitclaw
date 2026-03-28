# Story 3.3: Implement `on_event()` Callback on `AgentBuilder`

Status: ready-for-dev

## Story

As an agent developer,
I want to register an event callback via `AgentBuilder::on_event()`,
so that I receive typed events during agent execution for logging, metrics, or debugging.

## Acceptance Criteria

1. `AgentBuilder::on_event(callback)` method accepts `impl Fn(&AgentEvent) + Send + Sync + 'static`
2. `Agent` struct has new field: `event_callback: Option<Arc<dyn Fn(&AgentEvent) + Send + Sync>>`
3. `Agent` has private `emit_event(&self, event: &AgentEvent)` method
4. `DefaultStrategy::execute()` emits events at correct lifecycle points:
   - `LlmStart` before `provider.complete()`
   - `LlmEnd` after `provider.complete()` with token usage and duration
   - `ToolCall` before each tool execution
   - `ToolResult` after each tool execution
   - `GuardBlock` when a guard denies an action
   - `HintTriggered` when a hint fires
5. When no callback is registered (`None`), `emit_event` is a no-op (≤ 1μs, NFR2)
6. `event_callback` is threaded through `AgentRuntime` so strategies can emit events
7. All existing tests pass
8. New integration test: agent with callback collects events and verifies sequence

## Tasks / Subtasks

- [ ] Task 1: Add callback to `Agent` and `AgentBuilder` (AC: #1, #2)
  - [ ] Add `event_callback: Option<Arc<dyn Fn(&AgentEvent) + Send + Sync>>` to `Agent` struct fields
  - [ ] Add `event_callback: Option<Arc<dyn Fn(&AgentEvent) + Send + Sync>>` to `AgentBuilder` struct fields
  - [ ] Add `on_event()` builder method
  - [ ] Thread callback through `AgentBuilder::build()` → `Agent::new()`
- [ ] Task 2: Add callback to `AgentRuntime` (AC: #6)
  - [ ] Add `event_callback: Option<Arc<dyn Fn(&AgentEvent) + Send + Sync>>` to `AgentRuntime`
  - [ ] Update `Agent::to_runtime()` to clone the callback
  - [ ] Update `Agent::stream_with_session()` to include callback in runtime
- [ ] Task 3: Implement `emit_event` helper (AC: #3, #5)
  - [ ] Add helper function (standalone or on `AgentRuntime`): `fn emit_event(cb: &Option<Arc<dyn Fn(&AgentEvent) + Send + Sync>>, event: &AgentEvent)`
  - [ ] Single branch check — if `None`, return immediately
- [ ] Task 4: Wire up events in `DefaultStrategy` (AC: #4)
  - [ ] Before `provider.complete()` → emit `LlmStart { model }`
  - [ ] After `provider.complete()` → emit `LlmEnd { model, prompt_tokens, completion_tokens, duration_ms }`
  - [ ] Before tool execution in `process_tool_calls()` → emit `ToolCall { tool_name, args }`
  - [ ] After tool execution → emit `ToolResult { tool_name, success, duration_ms }`
  - [ ] When guard blocks in execution strategy → emit `GuardBlock { guard_name, reason }`
  - [ ] When hint triggers in `inject_hints()` → emit `HintTriggered { hint_name }`
- [ ] Task 5: Tests (AC: #7, #8)
  - [ ] Existing tests pass: `cargo test --workspace`
  - [ ] New test: build agent with `.on_event()`, run it, verify events collected
  - [ ] New test: agent without `.on_event()` still works (no callback = no-op)

## Dev Notes

### AgentRuntime Modification

The `AgentRuntime` struct needs a new field. This is **NOT a breaking change** because `AgentRuntime` is constructed internally by `Agent::to_runtime()` — it's not `#[non_exhaustive]` but users don't construct it directly.

```rust
// In traits/strategy.rs — add to AgentRuntime
pub struct AgentRuntime {
    // ... existing fields ...
    /// Optional event callback for lifecycle event emission.
    pub event_callback: Option<Arc<dyn Fn(&AgentEvent) + Send + Sync>>,
}
```

### emit_event Helper

```rust
// Standalone function (can live in default_strategy.rs or a shared module)
fn emit_event(
    callback: &Option<Arc<dyn Fn(&AgentEvent) + Send + Sync>>,
    event: &AgentEvent,
) {
    if let Some(ref cb) = callback {
        cb(event);
    }
}

// Usage in DefaultStrategy:
emit_event(&runtime.event_callback, &AgentEvent::LlmStart {
    model: model_info.name.clone(),
});
```

### Event Emission Points in DefaultStrategy

```
default_strategy.rs:
  Line ~88:  BEFORE provider.complete()   → LlmStart
  Line ~100: AFTER  provider.complete()   → LlmEnd (with response.usage tokens + provider_duration)
  Line ~216: inject_hints() hint.trigger  → HintTriggered
  Line ~266: BEFORE tool execution        → ToolCall
  Line ~275: AFTER  tool execution        → ToolResult

  Guard blocking happens in execution_strategy.rs execute_batch() → GuardBlock
```

### Token Usage Mapping

```rust
// After provider.complete():
AgentEvent::LlmEnd {
    model: model_info.name.clone(),
    prompt_tokens: response.usage.prompt_tokens as u32,
    completion_tokens: response.usage.completion_tokens as u32,
    duration_ms: provider_duration.as_millis() as u64,
}
```

### Integration Test Pattern

```rust
#[tokio::test]
async fn test_on_event_callback() {
    use std::sync::{Arc, Mutex};
    let events: Arc<Mutex<Vec<AgentEvent>>> = Arc::new(Mutex::new(vec![]));
    let events_clone = events.clone();

    let agent = Agent::builder()
        .provider(MockProvider::text("hello"))
        .system("test")
        .on_event(move |event| {
            events_clone.lock().unwrap().push(event.clone());
        })
        .build()
        .unwrap();

    agent.run("test input").await.unwrap();

    let captured = events.lock().unwrap();
    assert!(captured.iter().any(|e| matches!(e, AgentEvent::LlmStart { .. })));
    assert!(captured.iter().any(|e| matches!(e, AgentEvent::LlmEnd { .. })));
}
```

### References

- [crates/traitclaw-core/src/agent.rs](file:///Users/admin/Desktop/Projects/traitclaw/crates/traitclaw-core/src/agent.rs) — Agent struct (line 127), to_runtime() (line 244)
- [crates/traitclaw-core/src/agent_builder.rs](file:///Users/admin/Desktop/Projects/traitclaw/crates/traitclaw-core/src/agent_builder.rs) — AgentBuilder struct (line 47)
- [crates/traitclaw-core/src/default_strategy.rs](file:///Users/admin/Desktop/Projects/traitclaw/crates/traitclaw-core/src/default_strategy.rs) — event emission targets
- [crates/traitclaw-core/src/traits/strategy.rs](file:///Users/admin/Desktop/Projects/traitclaw/crates/traitclaw-core/src/traits/strategy.rs) — AgentRuntime struct
- [_bmad-output/planning-artifacts/architecture-v0.8.0.md](file:///Users/admin/Desktop/Projects/traitclaw/_bmad-output/planning-artifacts/architecture-v0.8.0.md) — Decision 2: Event Callback Mechanism
- FR16-FR19, NFR2, NFR7 in PRD v0.8.0

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
