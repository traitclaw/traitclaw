---
stepsCompleted: ["step-01-validate-prerequisites.md", "step-02-design-epics.md", "step-03-create-stories.md", "step-04-final-validation.md"]
inputDocuments: ["planning-artifacts/prd-v0.8.0.md", "architecture.md"]
---

# TraitClaw v0.8.0 "Quality Foundation" - Epic Breakdown

## Overview

This document provides the complete epic and story breakdown for TraitClaw v0.8.0, decomposing the requirements from the PRD and Architecture into implementable stories.

## Requirements Inventory

### Functional Requirements

- FR1: Framework contributor can import shared mock types (`MockProvider`, `MockMemory`) from a single `traitclaw-test-utils` crate
- FR2: Framework contributor can create deterministic mock providers with pre-defined response sequences
- FR3: Framework contributor can construct a complete `AgentRuntime` for testing via `make_runtime()` helper
- FR4: Framework contributor can use shared mock tools (EchoTool, FailTool) for tool-calling test scenarios
- FR5: All existing crate-local test mocks are consolidated into `traitclaw-test-utils` with zero duplicates remaining
- FR6: CI pipeline can verify code formatting across the entire workspace on every push/PR
- FR7: CI pipeline can run `clippy` lint checks across the entire workspace on every push/PR
- FR8: CI pipeline can run the full workspace test suite on every push/PR
- FR9: CI pipeline can verify that all public API types have documentation
- FR10: Framework contributor can generate code coverage reports locally via `cargo-llvm-cov`
- FR11: CI pipeline can generate and archive coverage reports for each run
- FR12: Agent developer can observe structured tracing spans for all LLM provider calls
- FR13: Agent developer can observe structured tracing spans for all tool executions
- FR14: Agent developer can observe structured tracing spans for guard checks
- FR15: Agent developer can filter trace output by component using standard `tracing` levels and targets
- FR16: Agent developer can register an event callback on `AgentBuilder` via `on_event()`
- FR17: `AgentEvent` enum can represent the full agent lifecycle: `LlmStart`, `LlmEnd`, `ToolCall`, `ToolResult`, `GuardBlock`, `HintTriggered`
- FR18: `AgentEvent::LlmEnd` variant includes token usage information (prompt_tokens, completion_tokens)
- FR19: Agent developer can use `AgentEvent` to build custom logging, metrics, or debugging workflows
- FR20: `RunUsage` can report estimated cost in USD after an agent run
- FR21: Cost estimation uses a configurable model pricing table
- FR22: Agent developer can access per-invocation token counts and cumulative costs
- FR23: A migration guide (`v0.7-to-v0.8.md`) documents all new types, zero breaking changes, and test migration path
- FR24: `traitclaw-test-utils` public API has comprehensive doc comments with usage examples
- FR25: An observability example (`26-observability`) demonstrates tracing + event callback end-to-end
- FR26: All v0.7.0 public APIs remain functional without changes
- FR27: All v0.7.0 examples compile and run on v0.8.0 without modification
- FR28: `AgentEvent` is `#[non_exhaustive]` to allow future variant additions without breaking changes

### NonFunctional Requirements

- NFR1: Tracing spans have zero runtime overhead when no `tracing` subscriber is registered
- NFR2: `AgentEvent` callback invocation adds ≤ 1μs per event when callback is registered
- NFR3: `RunUsage` cost calculation adds negligible overhead
- NFR4: CI pipeline completes full workspace check in ≤ 10 minutes on GitHub Actions standard runner
- NFR5: `traitclaw-test-utils` mock types add zero compile-time cost to non-test builds
- NFR6: Tracing spans are compatible with any `tracing::Subscriber` implementation
- NFR7: `AgentEvent` callback signature is `Fn(&AgentEvent) + Send + Sync + 'static`
- NFR8: `traitclaw-test-utils` works with `#[tokio::test]` and standard `#[test]` contexts
- NFR9: CI pipeline uses stable Rust toolchain only — no nightly features required
- NFR10: All new public types implement standard Rust traits: `Debug`, `Clone`, `Send + Sync`

### Additional Requirements

- New `traitclaw-test-utils` crate follows workspace crate pattern (Cargo.toml + src/lib.rs)
- Must be added to workspace `Cargo.toml` members list
- GitHub Actions CI config at `.github/workflows/ci.yml`
- Zero breaking changes — all additions via new types/methods, no existing signature changes
- `AgentEvent` uses `#[non_exhaustive]` for forward compatibility
- Priority order: test-utils → CI → tracing → events → cost tracking

### UX Design Requirements

N/A — Library project, no UI components.

### FR Coverage Map

- FR1: Epic 1 - MockProvider import from test-utils
- FR2: Epic 1 - Deterministic mock providers
- FR3: Epic 1 - make_runtime() helper
- FR4: Epic 1 - Shared mock tools (EchoTool, FailTool)
- FR5: Epic 1 - Consolidate all duplicate mocks
- FR6: Epic 2 - CI fmt check
- FR7: Epic 2 - CI clippy check
- FR8: Epic 2 - CI workspace test
- FR9: Epic 2 - CI doc verification
- FR10: Epic 2 - Local coverage via cargo-llvm-cov
- FR11: Epic 2 - CI coverage report
- FR12: Epic 3 - Tracing spans for LLM calls
- FR13: Epic 3 - Tracing spans for tool executions
- FR14: Epic 3 - Tracing spans for guard checks
- FR15: Epic 3 - Trace filtering by component
- FR16: Epic 3 - on_event() callback registration
- FR17: Epic 3 - AgentEvent enum lifecycle
- FR18: Epic 3 - Token usage in LlmEnd
- FR19: Epic 3 - Custom event workflows
- FR20: Epic 4 - RunUsage estimated_cost_usd
- FR21: Epic 4 - Configurable pricing table
- FR22: Epic 4 - Per-invocation token counts
- FR23: Epic 5 - Migration guide
- FR24: Epic 1 - Test-utils doc comments
- FR25: Epic 5 - Observability example
- FR26: Epic 5 - Backward compatibility
- FR27: Epic 5 - Examples compile on v0.8.0
- FR28: Epic 5 - #[non_exhaustive] AgentEvent

## Epic List

### Epic 1: Shared Test Infrastructure
Framework contributor has a single source of truth for test mocks — import one crate, write tests faster, zero duplication across workspace.
**FRs covered:** FR1, FR2, FR3, FR4, FR5, FR24

### Epic 2: Continuous Integration Pipeline
Every push/PR gets automated quality feedback — contributor knows code passes or fails within minutes, no manual verification needed.
**FRs covered:** FR6, FR7, FR8, FR9, FR10, FR11

### Epic 3: Runtime Tracing & Observability
Agent developer can see every LLM call, tool execution, and guard check via structured tracing and typed events — debug agent behavior with confidence.
**FRs covered:** FR12, FR13, FR14, FR15, FR16, FR17, FR18, FR19

### Epic 4: Cost Tracking & Usage Reporting
Agent developer knows exactly how much an agent costs per run in tokens and USD — optimize cost before production.
**FRs covered:** FR20, FR21, FR22

### Epic 5: Documentation, Examples & Backward Compatibility
New contributors onboard quickly, existing users migrate painlessly, observability example runs end-to-end.
**FRs covered:** FR23, FR25, FR26, FR27, FR28

## Epic 1: Shared Test Infrastructure

Framework contributor has a single source of truth for test mocks — import one crate, write tests faster, zero duplication across workspace.

### Story 1.1: Create `traitclaw-test-utils` Crate Scaffold

As a framework contributor,
I want a dedicated `traitclaw-test-utils` crate in the workspace,
So that I have a central place for all shared test utilities.

**Acceptance Criteria:**

**Given** the TraitClaw workspace
**When** I add `traitclaw-test-utils` to `crates/`
**Then** `crates/traitclaw-test-utils/Cargo.toml` exists with proper metadata
**And** `crates/traitclaw-test-utils/src/lib.rs` compiles
**And** root `Cargo.toml` includes `traitclaw-test-utils` in workspace members
**And** `cargo build -p traitclaw-test-utils` succeeds

### Story 1.2: Implement `MockProvider` with Response Sequences

As a framework contributor,
I want a `MockProvider` that returns pre-defined responses in sequence,
So that I can write deterministic tests without hitting real LLM APIs.

**Acceptance Criteria:**

**Given** a `MockProvider` initialized with `MockProvider::sequence(vec![response1, response2])`
**When** I call `provider.complete()` twice
**Then** the first call returns `response1` and the second returns `response2`
**And** calling `complete()` beyond the sequence returns an error
**And** `MockProvider` implements `Provider + Send + Sync`
**And** doc comments with usage examples are included

### Story 1.3: Implement `MockMemory` and `MockTools`

As a framework contributor,
I want shared `MockMemory` and mock tool types (EchoTool, FailTool),
So that I can test memory and tool-calling scenarios without boilerplate.

**Acceptance Criteria:**

**Given** `MockMemory::new()`
**When** I use it in an `AgentRuntime`
**Then** it stores and retrieves messages correctly
**And** `EchoTool` returns its input as output
**And** `FailTool` always returns an error
**And** all mock tools implement `Tool + Send + Sync`
**And** doc comments with usage examples are included

### Story 1.4: Implement `make_runtime()` Helper

As a framework contributor,
I want a `make_runtime(provider, tools)` helper that constructs a ready-to-use `AgentRuntime`,
So that test setup is a single function call instead of multi-line builder boilerplate.

**Acceptance Criteria:**

**Given** a `MockProvider` and a list of tools
**When** I call `make_runtime(provider, tools)`
**Then** I get a fully configured `AgentRuntime` ready for `agent.run()`
**And** default memory and config are applied automatically
**And** doc comments with usage examples are included

### Story 1.5: Migrate Existing Crate Tests to Shared Utils

As a framework contributor,
I want all existing crate-local mock implementations replaced with `traitclaw-test-utils` imports,
So that there are zero duplicate mock types across the workspace.

**Acceptance Criteria:**

**Given** the `traitclaw-core`, `traitclaw-strategies`, and `traitclaw-team` crates
**When** I search for local `MockProvider`/`MockMemory` definitions
**Then** zero local mock definitions exist — all tests use `traitclaw-test-utils`
**And** all existing tests pass (580+ tests green)
**And** each crate's `Cargo.toml` lists `traitclaw-test-utils` as a `[dev-dependencies]`

## Epic 2: Continuous Integration Pipeline

Every push/PR gets automated quality feedback — contributor knows code passes or fails within minutes, no manual verification needed.

### Story 2.1: Create GitHub Actions CI Workflow

As a framework contributor,
I want a GitHub Actions CI workflow that runs fmt, clippy, test, and doc checks on every push and PR,
So that code quality is verified automatically before merge.

**Acceptance Criteria:**

**Given** a push or PR to any branch
**When** GitHub Actions triggers `.github/workflows/ci.yml`
**Then** `cargo fmt --check --all` runs and fails on formatting violations
**And** `cargo clippy --workspace --all-targets -- -D warnings` runs and fails on lint warnings
**And** `cargo test --workspace` runs the full test suite
**And** `cargo doc --workspace --no-deps` verifies documentation builds
**And** pipeline uses stable Rust toolchain only
**And** total pipeline completes in ≤ 10 minutes

### Story 2.2: Add Code Coverage with `cargo-llvm-cov`

As a framework contributor,
I want code coverage reports generated locally and in CI,
So that I can identify untested code and track coverage improvements.

**Acceptance Criteria:**

**Given** `cargo-llvm-cov` is installed
**When** I run `cargo llvm-cov --workspace --html`
**Then** an HTML coverage report is generated in `target/llvm-cov/html/`
**And** CI workflow includes a coverage job that generates and archives the report
**And** coverage baseline is measured and documented
**And** coverage job is optional (does not block PR merges)

## Epic 3: Runtime Tracing & Observability

Agent developer can see every LLM call, tool execution, and guard check via structured tracing and typed events — debug agent behavior with confidence.

### Story 3.1: Add Tracing Spans to Core Runtime Operations

As an agent developer,
I want structured tracing spans on all LLM calls, tool executions, and guard checks,
So that I can observe agent behavior using standard `tracing` tooling.

**Acceptance Criteria:**

**Given** an agent runtime with a `tracing` subscriber registered
**When** `provider.complete()` is called
**Then** a tracing span `llm.complete` is emitted with `model` and `provider` fields
**When** `tool.call()` is called
**Then** a tracing span `tool.call` is emitted with `tool_name` and `args` fields
**When** `guard.check()` is called
**Then** a tracing span `guard.check` is emitted with `guard_name` and `result` fields
**And** all spans use `tracing::instrument` or manual span creation
**And** spans have zero overhead when no subscriber is registered

### Story 3.2: Define `AgentEvent` Enum and Lifecycle Events

As an agent developer,
I want a typed `AgentEvent` enum that represents the full agent lifecycle,
So that I can observe agent behavior programmatically without parsing logs.

**Acceptance Criteria:**

**Given** the `AgentEvent` enum in `traitclaw-core`
**Then** it includes variants: `LlmStart`, `LlmEnd`, `ToolCall`, `ToolResult`, `GuardBlock`, `HintTriggered`
**And** `LlmEnd` includes `prompt_tokens: u32` and `completion_tokens: u32`
**And** `AgentEvent` derives `Debug`, `Clone` and is `#[non_exhaustive]`
**And** `AgentEvent` is `Send + Sync`

### Story 3.3: Implement `on_event()` Callback on `AgentBuilder`

As an agent developer,
I want to register an event callback via `AgentBuilder::on_event()`,
So that I receive typed events during agent execution for logging, metrics, or debugging.

**Acceptance Criteria:**

**Given** an agent built with `.on_event(|event| { /* handle */ })`
**When** the agent executes and calls an LLM
**Then** the callback receives `AgentEvent::LlmStart` before the call
**And** the callback receives `AgentEvent::LlmEnd` after the call with token usage
**When** the agent calls a tool
**Then** the callback receives `AgentEvent::ToolCall` and `AgentEvent::ToolResult`
**And** callback signature is `Fn(&AgentEvent) + Send + Sync + 'static`
**And** callback invocation adds ≤ 1μs per event

## Epic 4: Cost Tracking & Usage Reporting

Agent developer knows exactly how much an agent costs per run in tokens and USD — optimize cost before production.

### Story 4.1: Extend `RunUsage` with Cost Estimation

As an agent developer,
I want `RunUsage` to include `estimated_cost_usd` after an agent run,
So that I know exactly how much each agent invocation costs.

**Acceptance Criteria:**

**Given** a completed agent run with token usage data
**When** I access `run_result.usage()`
**Then** `usage.estimated_cost_usd` returns a cost estimate in USD
**And** `usage.prompt_tokens` and `usage.completion_tokens` are available
**And** cost estimation uses an internal model pricing table
**And** pricing table is configurable (can add/update model prices)
**And** if model is unknown, cost is `0.0` with a tracing warning

## Epic 5: Documentation, Examples & Backward Compatibility

New contributors onboard quickly, existing users migrate painlessly, observability example runs end-to-end.

### Story 5.1: Create Observability Example

As an agent developer,
I want an end-to-end example demonstrating tracing + event callbacks,
So that I can learn observability features by running working code.

**Acceptance Criteria:**

**Given** `examples/26-observability/`
**When** I run the example with `RUST_LOG=info cargo run -p example-26-observability`
**Then** console shows structured tracing output with LLM, tool, and guard spans
**And** event callback prints `AgentEvent` variants in sequence
**And** `RunUsage` cost is displayed at the end
**And** example includes comments explaining each observability feature

### Story 5.2: Migration Guide and Backward Compatibility Verification

As an existing TraitClaw user,
I want a migration guide and verified backward compatibility,
So that I can upgrade to v0.8.0 with confidence.

**Acceptance Criteria:**

**Given** `docs/migration-v0.7-to-v0.8.md`
**Then** it documents all new types (`AgentEvent`, `RunUsage` changes, test-utils)
**And** it confirms zero breaking changes
**And** it provides test migration examples (before/after using `traitclaw-test-utils`)
**And** all existing v0.7.0 examples compile and run on v0.8.0 without modification
**And** `cargo test --workspace` passes with zero failures
