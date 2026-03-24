# Story 1.4: Agent Builder & Config

Status: review

## Story

As a developer,
I want an `AgentBuilder` with a fluent API to configure agents,
So that creating an agent is intuitive and discoverable.

## Acceptance Criteria

1. **Given** I use the builder pattern **When** I call `Agent::builder().model(provider).system("...").build()` **Then** it returns a configured `Agent` instance
2. **And** `.build()` returns `Result<Agent>` (error if no provider set)
3. **And** builder methods accept `impl Into<String>` for string parameters
4. **And** optional settings have sensible defaults (max_tokens, temperature, etc.)

## Tasks / Subtasks

- [x] Task 1: Create `config.rs` (AC: 4)
  - [x] `AgentConfig` with system_prompt, max_tokens, temperature, max_iterations, token_budget
  - [x] Defaults: max_tokens=4096, temperature=0.7, max_iterations=20
  - [x] `Default` trait implemented
- [x] Task 2: Create `agent.rs` with Agent struct (AC: 1)
  - [x] `Agent` struct with all Arc<dyn Trait> fields
  - [x] `Agent::builder()` associated fn
  - [x] `run()`, `stream()`, `run_structured()` stubs
- [x] Task 3: Create `agent_builder.rs` (AC: 1, 2, 3)
  - [x] `.model()` alias (AC-1) and `.provider()` method
  - [x] `.system(impl Into<String>)`, `.max_tokens()`, `.temperature()`, `.max_iterations()`
  - [x] `.build() -> Result<Agent>` — error if provider is None
  - [x] NoopGuard, NoopHint, NoopTracker, InMemoryMemory as defaults
- [x] Task 4: Noop steering implementations (AC: 4)
  - [x] `NoopGuard` — always Allow
  - [x] `NoopHint` — never triggers
  - [x] `NoopTracker` — no-ops
- [x] Task 5: Tests (AC: all)
  - [x] AC-1: `.model()` alias builds successfully
  - [x] AC-2: `.build()` without provider returns error
  - [x] AC-3: `system()` accepts both `&str` and `String`
  - [x] AC-4: defaults match spec

## Dev Notes

### Architecture Requirements
- Builder pattern for Agent (AD-7) — progressive complexity
- Return `Result<Agent>` from `.build()`, not panic
- Accept `impl Into<String>` for ergonomic API
- NoopGuard/NoopHint/NoopTracker in core for zero-cost when unused (AD-2)
- Depends on Story 1.2 (types), 1.3 (Provider), 1.5 (Memory)

### Critical Patterns
- `Agent::builder()` returns `AgentBuilder` (not `Builder<Agent>`)
- Provider stored as `Arc<dyn Provider>` — must be Send + Sync + 'static
- Tools stored as `Vec<Arc<dyn ErasedTool>>` — will be populated in Epic 2
- Guards/Hints as `Vec<Arc<dyn Guard>>` / `Vec<Arc<dyn Hint>>`
- Tracker as `Arc<dyn Tracker>` (single tracker, but it can compose internally)

### Dependencies on Other Stories
- Story 1.2: Message types (for CompletionRequest)
- Story 1.3: Provider trait (required for builder)
- Story 1.5: Memory trait + InMemoryMemory (default memory)
- Story 1.7: Error types (for Result<Agent>)

### References
- [Source: _bmad-output/architecture.md#3.5 Agent]
- [Source: _bmad-output/architecture.md#7 Developer Experience]
- [Source: _bmad-output/project-context.md#Code Style - Builder pattern]

## Dev Agent Record

### Agent Model Used
gemini-2.5-pro (antigravity)

### Debug Log References
- `cargo test` (workspace) → all passed
- `cargo clippy --all-targets` → clean
- `cargo fmt --all --check` → clean

### Completion Notes List
- Core files already existed. Completed the gaps: added `.model()` alias (AC-1), fixed defaults (AC-4), added 3 tests.
- `AgentConfig` defaults updated from `None` to `Some(4096)` / `Some(0.7)` per spec.
- `FakeProvider` in test module uses `OnceLock<ModelInfo>` for static lifetime.

### File List
- `crates/traitclaw-core/src/config.rs` (defaults fixed)
- `crates/traitclaw-core/src/agent_builder.rs` (.model() alias + 3 new tests)

### Change Log
- 2026-03-24: All tasks complete.

---

## Senior Developer Review (AI)

**Review Date:** 2026-03-24  
**Outcome:** Approved  
**Reviewer:** gemini-2.5-pro (code-review workflow)

### Action Items

#### Patches
- [x] [High] P-1: Added `#[non_exhaustive]` to `AgentConfig`.
- [x] [Med] P-2: `FakeProvider` simplified to use `info: ModelInfo` struct field.
- [x] [Low] BS-1: `AgentBuilder::provider()` doc updated to clearly prefer `.model()`.

#### Deferred
- [ ] [Med] D-1: No temperature range validation — silent out-of-range values cause LLM API errors downstream.
- [ ] [Low] D-2: Missing tests for `.tool()`, `.guard()`, `.hint()`, `.tracker()` builder chains.
- [ ] [Low] D-3: `token_budget: None` default intent not documented in test.

