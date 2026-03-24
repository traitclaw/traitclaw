# Story 3.1: Guard-Hint-Track Trait Integration

Status: review

## Story

As a developer,
I want Guard, Hint, and Tracker traits wired into the runtime,
So that the runtime supports model steering when steering feature is enabled.

## Acceptance Criteria

1. **Given** the Guard/Hint/Tracker traits are defined in core **When** no steering is configured **Then** NoopGuard/NoopHint/NoopTracker are used (zero overhead)
2. **And** when Guards are configured, every Action goes through `guard.check()` before execution
3. **And** when Hints are configured, `hint.should_trigger()` is checked each iteration
4. **And** when Tracker is configured, state is updated after each LLM call/tool call
5. **And** `AgentState` struct tracks: iteration_count, token_usage, context_utilization

## Tasks / Subtasks

- [x] Task 1: Define Guard trait in `traits/guard.rs` (AC: 1, 2)
  - [x] `fn name(&self) -> &str`
  - [x] `fn check(&self, action: &Action) -> GuardResult` — sync, must be fast
  - [x] `NoopGuard` always returns Allow
- [x] Task 2: Define Hint trait in `traits/hint.rs` (AC: 1, 3)
  - [x] `fn name(&self) -> &str`
  - [x] `fn should_trigger(&self, state: &AgentState) -> bool`
  - [x] `fn generate(&self, state: &AgentState) -> HintMessage`
  - [x] `fn injection_point(&self) -> InjectionPoint`
  - [x] `NoopHint` never triggers
- [x] Task 3: Define Tracker trait in `traits/tracker.rs` (AC: 1, 4)
  - [x] `fn on_iteration(&self, state: &mut AgentState)`
  - [x] `fn on_tool_call(&self, name: &str, args: &Value, state: &mut AgentState)`
  - [x] `fn on_llm_response(&self, response: &CompletionResponse, state: &mut AgentState)`
  - [x] `fn recommended_concurrency(&self, state: &AgentState) -> usize`
  - [x] `NoopTracker` — all no-ops, recommended_concurrency returns usize::MAX
- [x] Task 4: Wire into runtime (AC: 2, 3, 4, 5)
  - [x] Guard.check() before each tool execution
  - [x] Hint.should_trigger() + generate() each iteration
  - [x] Tracker callbacks at appropriate points
  - [x] Maintain AgentState throughout loop
- [x] Task 5: Add builder methods (AC: 1)
  - [x] `.guard(impl Guard)`, `.hint(impl Hint)`, `.tracker(impl Tracker)`
- [x] Task 6: Write tests (AC: all)
  - [x] Test Noop impls have zero overhead
  - [x] Test Guard deny prevents tool execution
  - [x] Test Hint message injected when triggered
  - [x] Test Tracker callbacks called at correct points

## Dev Notes

### Architecture Requirements
- Traits are in core (AD-2) — implementations in traitclaw-steering (Epic 3)
- Guard is SYNC — must be fast (regex/rules, 0 tokens)
- Hint costs 1-2 iterations but saves 20+
- Tracker is silent monitoring (0 tokens)
- Multiple guards/hints supported, single tracker

### References
- [Source: _bmad-output/architecture.md#3.4 Guard/Hint/Tracker]
- [Source: _bmad-output/architecture.md#5 Guard-Hint-Track Steering System]

## Dev Agent Record

### Agent Model Used
gemini-2.5-pro (antigravity)

### Debug Log References
- `cargo test -p traitclaw-core -- traits::guard traits::hint traits::tracker` → 7 passed
- `cargo clippy --all-targets` → clean

### Completion Notes List
- All traits, Noop impls, runtime wiring, and builder methods were already implemented.
- Added 4 Hint tests, 2 Tracker tests. Combined with 1 existing Guard test = 7 trait tests.
- Runtime guard deny already tested in runtime.rs (test_guard_deny_blocks_tool_ac7).

### File List
- `crates/traitclaw-core/src/traits/hint.rs` (tests added)
- `crates/traitclaw-core/src/traits/tracker.rs` (tests added)

### Change Log
- 2026-03-24: Added Hint/Tracker tests. Story verified complete.
