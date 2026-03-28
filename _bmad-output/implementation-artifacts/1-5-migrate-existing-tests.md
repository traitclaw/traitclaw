# Story 1.5: Migrate Existing Crate Tests to Shared Utils

Status: ready-for-dev

## Story

As a framework contributor,
I want all existing crate-local mock implementations replaced with `traitclaw-test-utils` imports,
so that there are zero duplicate mock types across the workspace.

## Acceptance Criteria

1. Zero local `MockProvider` / `SequenceProvider` definitions remain in any crate's test code
2. Zero local `MockMemory` definitions remain in any crate
3. Zero local `EchoTool` / `DangerousTool` definitions remain in `traitclaw-core` test code (keep DangerousTool if semantically different from FailTool)
4. All crates with test code use `traitclaw-test-utils` as `[dev-dependencies]`
5. All existing tests pass: `cargo test --workspace` (580+ tests green)
6. `make_runtime()` from test-utils replaces local runtime builder boilerplate in `traitclaw-strategies`

## Tasks / Subtasks

- [ ] Task 1: Add dev-dependency to all test-using crates (AC: #4)
  - [ ] `crates/traitclaw-core/Cargo.toml` — add `traitclaw-test-utils = { path = "../traitclaw-test-utils" }` under `[dev-dependencies]`
  - [ ] `crates/traitclaw-strategies/Cargo.toml` — add same
  - [ ] `crates/traitclaw-team/Cargo.toml` — add same (if it has test mocks)
  - [ ] Scan any other crates with inline mocks
- [ ] Task 2: Migrate `traitclaw-strategies` (AC: #1, #6)
  - [ ] Replace `use crate::test_utils::MockProvider` → `use traitclaw_test_utils::provider::MockProvider`
  - [ ] Replace `use crate::test_utils::MockMemory` → `use traitclaw_test_utils::memory::MockMemory`
  - [ ] Replace `use crate::test_utils::make_runtime` → `use traitclaw_test_utils::runtime::make_runtime`
  - [ ] Delete `crates/traitclaw-strategies/src/test_utils.rs`
  - [ ] Remove `pub(crate) mod test_utils;` from strategies `lib.rs`
  - [ ] Run `cargo test -p traitclaw-strategies`
- [ ] Task 3: Migrate `traitclaw-core` test_utils.rs (AC: #1, #2, #3)
  - [ ] Replace `SequenceProvider` usage with `MockProvider` from test-utils
  - [ ] Replace local `EchoTool` usage with test-utils `EchoTool`
  - [ ] Keep `DenyGuard` in core (guard mocking is core-specific) OR move if general enough
  - [ ] Delete `crates/traitclaw-core/src/test_utils.rs`
  - [ ] Remove `pub(crate) mod test_utils;` from core `lib.rs` (the `#[cfg(test)]` one)
- [ ] Task 4: Migrate inline test mocks (AC: #1)
  - [ ] `agent.rs` (line ~579) — inline `MockProvider` in test module → use test-utils
  - [ ] `agent_builder.rs` (line ~420) — `FakeProvider` → use test-utils MockProvider
  - [ ] `transformers.rs` — any inline mocks → use test-utils
  - [ ] `traits/provider.rs` — any inline mocks → use test-utils
  - [ ] `tests/integration.rs` — any inline mocks → use test-utils
- [ ] Task 5: Full workspace verification (AC: #5)
  - [ ] `cargo test --workspace` — all tests pass
  - [ ] `cargo clippy --workspace --all-targets -- -D warnings` — no warnings
  - [ ] Grep workspace for remaining `struct MockProvider` / `struct SequenceProvider` — zero results

## Dev Notes

### Files to Modify

| File | Action | Mocks to Remove |
|------|--------|-----------------|
| `crates/traitclaw-strategies/src/test_utils.rs` | DELETE entire file | MockProvider, MockMemory, NoopTracker, NoopContextManager, NoopOutputTransformer, make_runtime |
| `crates/traitclaw-core/src/test_utils.rs` | DELETE entire file | SequenceProvider, EchoTool, DangerousTool, DenyGuard |
| `crates/traitclaw-core/src/agent.rs` | MODIFY test module | Inline MockProvider → import |
| `crates/traitclaw-core/src/agent_builder.rs` | MODIFY test module | FakeProvider → import |
| `crates/traitclaw-core/src/lib.rs` | MODIFY | Remove `#[cfg(test)] pub(crate) mod test_utils;` |

### Migration Pattern

```rust
// BEFORE (inline mock)
#[cfg(test)]
mod tests {
    struct MockProvider { ... }
    impl Provider for MockProvider { ... }

    #[test]
    fn test_something() {
        let p = MockProvider::new();
        // ...
    }
}

// AFTER (shared import)
#[cfg(test)]
mod tests {
    use traitclaw_test_utils::provider::MockProvider;

    #[test]
    fn test_something() {
        let p = MockProvider::text("ok");
        // ...
    }
}
```

### SequenceProvider → MockProvider Mapping

| SequenceProvider method | MockProvider equivalent |
|------------------------|----------------------|
| `SequenceProvider::text("x")` | `MockProvider::text("x")` |
| `SequenceProvider::with_responses(v)` | `MockProvider::sequence(v)` |

### DangerousTool Decision

`DangerousTool` in core tests is used specifically for hook interception tests. Options:
1. Add `DangerousTool` to test-utils (if reusable across crates)
2. Keep inline in core (if only core uses it)
→ Recommend: Move to test-utils as `pub struct DangerousTool` since hook testing is general

### References

- [crates/traitclaw-strategies/src/test_utils.rs](file:///Users/admin/Desktop/Projects/traitclaw/crates/traitclaw-strategies/src/test_utils.rs) — DELETE target
- [crates/traitclaw-core/src/test_utils.rs](file:///Users/admin/Desktop/Projects/traitclaw/crates/traitclaw-core/src/test_utils.rs) — DELETE target
- [crates/traitclaw-core/src/agent.rs](file:///Users/admin/Desktop/Projects/traitclaw/crates/traitclaw-core/src/agent.rs) — inline mock at test module
- [crates/traitclaw-core/src/agent_builder.rs](file:///Users/admin/Desktop/Projects/traitclaw/crates/traitclaw-core/src/agent_builder.rs) — inline FakeProvider
- FR5 in PRD v0.8.0

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
