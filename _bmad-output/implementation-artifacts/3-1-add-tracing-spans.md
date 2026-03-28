# Story 3.1: Add Tracing Spans to Core Runtime Operations

Status: ready-for-dev

## Story

As an agent developer,
I want structured tracing spans on all LLM calls, tool executions, and guard checks,
so that I can observe agent behavior using standard `tracing` tooling.

## Acceptance Criteria

1. `provider.complete()` is wrapped in `info_span!(target: "traitclaw::llm", "llm.complete", model = %model)` 
2. Tool execution is wrapped in `info_span!(target: "traitclaw::tool", "tool.call", tool_name = %name)`
3. Guard check is wrapped in `info_span!(target: "traitclaw::guard", "guard.check", guard_name = %name)`
4. Hint injection emits `debug_span!(target: "traitclaw::hint", "hint.trigger", hint_name = %name)`
5. All spans have zero overhead when no `tracing` subscriber is registered (NFR1)
6. `RUST_LOG=traitclaw::llm=debug` filters to only LLM spans (FR15)
7. Existing `#[tracing::instrument]` on `DefaultStrategy::execute()` is preserved/enhanced
8. All existing tests pass
9. New tests verify span emission using `tracing-test` or `tracing_subscriber` capture

## Tasks / Subtasks

- [ ] Task 1: Add tracing spans to `default_strategy.rs` (AC: #1, #7)
  - [ ] Wrap `provider.complete()` call (line ~91) with `info_span!(target: "traitclaw::llm", "llm.complete", model = ...)`
  - [ ] Existing `#[tracing::instrument]` on `execute()` already provides agent-level span — keep it
  - [ ] Add timing field to LLM span: record duration after completion
- [ ] Task 2: Add tracing spans to tool execution (AC: #2)
  - [ ] In `process_tool_calls()` — wrap each tool's execution in `info_span!(target: "traitclaw::tool", "tool.call", tool_name = %tc.name)`
  - [ ] Record `duration_ms` and `success` after tool completes
- [ ] Task 3: Add tracing spans to guard checks (AC: #3)
  - [ ] In `execution_strategy.execute_batch()` — or where guards are checked in runtime
  - [ ] `info_span!(target: "traitclaw::guard", "guard.check", guard_name = %name, blocked = %result)`
- [ ] Task 4: Add tracing spans to hint injection (AC: #4)
  - [ ] In `inject_hints()` — existing `tracing::debug!` can be enhanced with a span
  - [ ] `debug_span!(target: "traitclaw::hint", "hint.trigger", hint_name = %name)`
- [ ] Task 5: Verify zero-cost (AC: #5, #6)
  - [ ] Compile and benchmark without subscriber — verify no measurable overhead
  - [ ] Test component-level filtering with `RUST_LOG=traitclaw::llm=debug`
- [ ] Task 6: Tests (AC: #8, #9)
  - [ ] All existing tests pass: `cargo test --workspace`
  - [ ] Add at least 1 test verifying span emission (use `tracing-test` crate if available, or capture spans manually)

## Dev Notes

### Instrumentation Locations

```
crates/traitclaw-core/src/default_strategy.rs:
  Line 32:  #[tracing::instrument(...)]  ← existing agent-level span, KEEP
  Line 91:  runtime.provider.complete(request).await  ← ADD llm.complete span
  Line 223: process_tool_calls()  ← ADD tool.call span per tool
  Line 207: inject_hints()  ← ADD hint.trigger span
```

### Span Pattern (from Architecture v0.8.0)

```rust
use tracing::{info_span, Instrument};

// Pattern 1: Sync span guard
let span = info_span!(
    target: "traitclaw::llm",
    "llm.complete",
    model = %model_info.name,
);
let _guard = span.enter();
let response = runtime.provider.complete(request).await;
drop(_guard);

// Pattern 2: Async instrumentation (preferred for async calls)
let response = runtime.provider.complete(request)
    .instrument(info_span!(
        target: "traitclaw::llm",
        "llm.complete",
        model = %model_info.name,
    ))
    .await;
```

### Guard Check Location

Guards are checked in `crates/traitclaw-core/src/traits/execution_strategy.rs` within `execute_batch()`. The guard span should be added there:

```rust
for guard in guards {
    let span = info_span!(
        target: "traitclaw::guard",
        "guard.check",
        guard_name = guard.name(),
    );
    let _g = span.enter();
    let result = guard.check(&action);
    // ...
}
```

### Zero-Cost Guarantee

`tracing` crate guarantees zero overhead when no subscriber is registered — the macros expand to no-ops. This is a compile-time optimization, not runtime branching.

### tracing Dependency

`tracing` is already in workspace dependencies. May need `tracing = { workspace = true }` in `traitclaw-core/Cargo.toml` if not already present. Check:
- `Instrument` trait requires `tracing` `futures` feature or just the base crate

### References

- [crates/traitclaw-core/src/default_strategy.rs](file:///Users/admin/Desktop/Projects/traitclaw/crates/traitclaw-core/src/default_strategy.rs) — all instrumentation targets
- [_bmad-output/planning-artifacts/architecture-v0.8.0.md](file:///Users/admin/Desktop/Projects/traitclaw/_bmad-output/planning-artifacts/architecture-v0.8.0.md) — Decision 4: Tracing Span Naming
- FR12-FR15 in PRD v0.8.0

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
