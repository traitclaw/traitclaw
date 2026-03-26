# Story 1.1: Implement `Agent::with_system()` Shorthand

Status: review

## Story

As a **Rust developer building AI agents**,
I want to create an agent with `Agent::with_system(provider, "system prompt")` in a single line,
so that I can skip the builder pattern for simple single-agent use cases.

## Acceptance Criteria

1. Given a valid `Provider` instance and a system prompt string, when `Agent::with_system(provider, "You are a helpful assistant.")` is called, then an `Agent` is returned with the provider and system prompt configured, and the agent behaves identically to one created via `Agent::builder().provider(p).system(s).build()?`.

2. Given any type implementing `Into<String>` (e.g., `&str`, `String`), when passed as the `system` parameter, then it is accepted without explicit conversion.

3. Given `Agent::with_system()` is called, when compared to the existing `Agent::builder()` API, then the builder API is unchanged — no existing methods are modified or removed (purely additive).

4. Given a unit test with a mock provider, when `Agent::with_system(mock_provider, "test prompt")` is called, then the resulting agent has the correct system prompt and provider configured.

## Tasks / Subtasks

- [x] Task 1: Add `with_system()` associated function to `Agent` struct (AC: #1, #2)
  - [x] Add method signature: `pub fn with_system(provider: impl Provider, system: impl Into<String>) -> Self`
  - [x] Implement by delegating to `Agent::builder().provider(provider).system(system).build().expect("with_system is infallible")`
  - [x] Verify `build()` cannot fail when only provider + system are set (FR1: infallible)
- [x] Task 2: Unit test (AC: #3, #4)
  - [x] Test with mock provider and `&str` system prompt
  - [x] Test with `String` system prompt
  - [x] Test that builder API is unchanged (compile-time check via existing tests)

## Dev Notes

- **Crate:** `traitclaw-core` → `crates/traitclaw-core/src/agent.rs`
- **Pattern:** This is a convenience constructor, NOT a new struct. Just add an associated function to existing `Agent`.
- **Infallible:** Per FR1, `with_system` must be infallible. The only way `build()` can fail is if provider is missing — but we're passing it. Use `expect()` internally or restructure to avoid `Result`.
- **ADR compliance:** No new traits needed (ADR-21: no factory trait).
- **MSRV:** Rust 1.75+ — no special features needed for this.
- **Provider type:** Uses `impl Provider` to accept concrete provider types.

### Project Structure Notes

- File: `crates/traitclaw-core/src/agent.rs` — add method to existing `impl Agent` block
- Tests: `crates/traitclaw-core/src/agent.rs` — inline test module
- No new files needed

### References

- [Source: _bmad-output/planning-artifacts/epics-v0.6.0.md#Story 1.1]
- [Source: _bmad-output/planning-artifacts/prd-v0.6.0.md#FR1]
- [Source: _bmad-output/project-context.md#Agent API patterns]

## Dev Agent Record

### Agent Model Used

Gemini 2.5 Pro (Antigravity)

### Debug Log References

### Completion Notes List

- ✅ Added `Agent::with_system(provider, system)` as an infallible convenience constructor
- ✅ Delegates to `Agent::builder().provider(p).system(s).build().expect(...)` internally
- ✅ Full rustdoc with `# Example` and `# Panics` sections
- ✅ 4 unit tests: `&str` prompt, `String` prompt, builder compatibility, provider verification
- ✅ All 201 existing tests pass (zero regressions)
- ✅ Doc-test for `with_system()` compiles successfully

### File List

- `crates/traitclaw-core/src/agent.rs` (modified: added `with_system()` method + 4 unit tests)
