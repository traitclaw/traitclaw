---
stepsCompleted: ["step-01-init.md", "step-02-discovery.md", "step-02b-vision.md", "step-02c-executive-summary.md", "step-03-success.md", "step-04-journeys.md", "step-05-domain.md", "step-06-innovation.md", "step-07-project-type.md", "step-08-scoping.md", "step-09-functional.md", "step-10-nonfunctional.md", "step-11-polish.md", "step-12-complete.md"]
inputDocuments: ["brainstorming/v080-brainstorming.md", "planning-artifacts/prd.md", "project-context.md", "product-brief-traitclaw-2026-03-26.md"]
workflowType: 'prd'
classification:
  projectType: developer_tool
  domain: AI / Developer Tooling
  complexity: medium
  projectContext: brownfield
---

# Product Requirements Document - traitclaw v0.8.0 "Quality Foundation"

**Author:** Bangvu
**Date:** 2026-03-28

## Executive Summary

TraitClaw v0.8.0 "Quality Foundation" shifts the framework's development focus from feature delivery to quality infrastructure. Building on v0.7.0's reasoning strategies (ReAct, MCTS, CoT), this release invests in two pillars: **testing infrastructure** and **runtime observability** — the prerequisites for a confident v1.0 API freeze.

Part A delivers a shared `traitclaw-test-utils` crate that consolidates duplicated mock providers, mock memory, and test helpers currently copy-pasted across `traitclaw-core` and `traitclaw-strategies`. It introduces a GitHub Actions CI pipeline (`cargo fmt`, `clippy`, `test --workspace`, `doc`) and establishes coverage baselines via `cargo-llvm-cov`.

Part B adds structured observability: `tracing::instrument` spans on all LLM calls, tool executions, and guard checks; a typed `AgentEvent` enum (`LlmStart`, `LlmEnd`, `ToolCall`, `GuardBlock`, `HintTriggered`); an `on_event` callback on `AgentBuilder`; and cost estimation in `RunUsage`. These tools let agent developers see exactly what their agents are doing, where they're spending tokens, and why they get stuck.

The release targets the same Rust developers using TraitClaw v0.7.0, who now need to debug, monitor, and trust their agent behavior before v1.0 stabilization.

### What Makes This Special

1. **No new agent features — all investment in quality.** Every other version added capabilities (providers, tools, memory, strategies). v0.8.0 breaks the pattern deliberately: it protects what's already built and earns the confidence needed for API freeze.
2. **Shared test crate eliminates cross-crate duplication.** `traitclaw-test-utils` provides `MockProvider`, `MockMemory`, `MockTools`, and `make_runtime()` — one source of truth for deterministic testing across all 13+ crates.
3. **CI pipeline catches regressions automatically.** No more "works on my machine" — every push runs fmt, clippy, full workspace tests, and doc verification.
4. **Observability as a first-class primitive.** `AgentEvent` + tracing spans make agent debugging as natural as application debugging. Cost tracking in `RunUsage` prevents token budget surprises.

## Project Classification

| Attribute | Value |
|-----------|-------|
| **Project Type** | Developer Tool (Rust Framework/Library) |
| **Domain** | AI / Developer Tooling |
| **Complexity** | Medium |
| **Project Context** | Brownfield (building on TraitClaw v0.7.0) |
| **New Crate** | `traitclaw-test-utils` |
| **Breaking Changes** | Zero — purely additive |

## Success Criteria

### User Success

| Metric | Target | Persona |
|--------|--------|---------|
| **Mock migration effort** | Existing crate tests migrate to `traitclaw-test-utils` in ≤ 5 minutes per test file | Framework contributor |
| **CI feedback loop** | PR gets fmt+clippy+test results within 10 minutes of push | Framework contributor |
| **Agent debugging time** | Developer can identify why an agent loops/fails in ≤ 2 minutes using tracing output | Agent developer (Minh) |
| **Event observation setup** | Add `on_event` callback to existing agent in ≤ 3 lines of code | Agent developer (Minh) |
| **Cost visibility** | After agent run, `RunUsage` shows estimated cost in USD without manual calculation | Agent developer (Linh) |

### Business Success

| Metric | Target (6 months post-release) |
|--------|-------------------------------|
| **CI stability** | Zero regressions merged to main — all caught by CI |
| **Coverage baseline** | ≥ 70% line coverage measured across workspace |
| **Example coverage** | 29+ runnable examples (current ~28) |
| **Docs coverage** | 100% new public API documented with examples |

### Technical Success

| Metric | Target |
|--------|--------|
| **Backward compatibility** | All v0.7.0 examples compile on v0.8.0 — zero breaking changes |
| **Test deduplication** | Zero duplicate mock implementations across crates |
| **CI pipeline** | GitHub Actions: fmt, clippy, test, doc — all green on main |
| **Coverage tool** | `cargo-llvm-cov` generates HTML report locally and in CI |
| **Tracing integration** | All `Provider::complete`, `Tool::call`, `Guard::check` have tracing spans |
| **Event completeness** | `AgentEvent` covers full agent lifecycle (LLM, tool, guard, hint) |
| **No new required deps** | `tracing` already in deps; `traitclaw-test-utils` is dev-dependency only |
| **MSRV** | Rust 1.75+ maintained |

### Measurable Outcomes

- CI pipeline runs full workspace test suite in ≤ 10 minutes
- `traitclaw-test-utils` replaces all duplicated mocks — single source of truth
- A new contributor can run `cargo llvm-cov --workspace --html` and see coverage report
- Observability example demonstrates full agent lifecycle tracing with `tracing-subscriber`
- `AgentEvent::LlmEnd` includes token usage; `RunUsage` includes `estimated_cost_usd`

## Product Scope

### MVP — v0.8.0 Release

**Must-Have (Go/No-Go):**

1. `traitclaw-test-utils` crate with shared `MockProvider`, `MockMemory`, `MockTools`, `make_runtime()`
2. All existing crate tests migrated to use shared test utils (zero duplicate mocks)
3. GitHub Actions CI pipeline: `cargo fmt --check`, `cargo clippy --workspace`, `cargo test --workspace`, `cargo doc --workspace --no-deps`
4. `cargo-llvm-cov` integration with baseline coverage report
5. `tracing::instrument` spans on core runtime operations (LLM calls, tool calls, guard checks)
6. `AgentEvent` enum with lifecycle events
7. `AgentBuilder::on_event(callback)` for event observation
8. `RunUsage` extended with `estimated_cost_usd`
9. Observability example (`examples/26-observability`)
10. Zero breaking changes verified

### Growth Features (Post-MVP — v0.9.0)

- Property-based testing with `proptest` (fuzz guards, strategies)
- Snapshot testing with `insta` (lock output formats)
- Performance benchmarks with `criterion`
- Coverage gap filling to reach 80%+ on core

### Vision (Future — v1.0+)

- OpenTelemetry exporter for production agent monitoring
- Distributed tracing across multi-agent teams
- Real-time agent dashboard (separate project)

## User Journeys

### Journey 1: Contributor — "Fix a Bug Without Breaking Everything" (Primary, Success Path)

**Contributor** — A Rust developer (could be Bangvu or future contributor) working on TraitClaw internals. Needs confidence that changes don't break other crates.

**Opening:** Contributor finds a bug in `traitclaw-steering` guard logic. Writes a fix, but the guard tests use a local `MockProvider` copy-pasted from `traitclaw-core`. Changing the mock means editing two files identically.

**Rising Action:** After v0.8.0, contributor writes the fix and tests using `traitclaw-test-utils::MockProvider` — one import, one mock, shared across all crates. Pushes to GitHub.

**Climax:** CI pipeline runs automatically: `cargo fmt --check` ✓, `cargo clippy` ✓, `cargo test --workspace` ✓ (all 580+ tests pass), `cargo doc` ✓. Coverage report shows the fix increased coverage in `traitclaw-steering` by 2%.

**Resolution:** Contributor merges with confidence. No manual "did I break traitclaw-strategies?" checks needed — CI caught everything.

### Journey 2: Minh — "Why Is My Agent Stuck in a Loop?" (Primary, Edge Case)

**Minh** — Senior Rust dev, building a fintech agent with ReAct strategy. Agent sometimes loops endlessly calling the same tool.

**Opening:** Minh's agent runs for 30 seconds and hits the max iteration limit. Output is just the final answer — no visibility into what happened during reasoning.

**Rising Action:** Minh adds `.on_event(|event| { tracing::info!(?event); })` to his agent builder — 1 line of code. Runs with `RUST_LOG=info`.

**Climax:** Console shows: `LlmStart { model: "gpt-4o" }` → `ToolCall { name: "stock_price", args: "AAPL" }` → `ToolResult { ... }` → `LlmStart` → `ToolCall { name: "stock_price", args: "AAPL" }` — **the agent calls the same tool with the same args repeatedly.** Minh immediately sees the issue: the tool result isn't being included in the next LLM prompt.

**Resolution:** Bug found in 2 minutes instead of 20. Minh fixes his tool output formatting. The `AgentEvent` stream made the invisible visible.

### Journey 3: Linh — "How Much Is This Agent Costing Me?" (Primary, Advanced)

**Linh** — AI engineer, running a multi-agent team for content generation. Needs to track token costs across agents.

**Opening:** Linh's 3-agent team generates content but she has no idea how much each run costs. She manually checks OpenAI dashboard after each test.

**Rising Action:** After v0.8.0, Linh checks `run_result.usage()` which now includes `estimated_cost_usd`. She adds `.on_event(...)` to log per-agent costs.

**Climax:** Console: `Agent "researcher": $0.03 (2,100 tokens)` → `Agent "writer": $0.12 (8,400 tokens)` → `Agent "editor": $0.02 (1,200 tokens)` → **Total: $0.17/run.** Linh realizes the "writer" agent is 4x more expensive — she optimizes its system prompt to reduce token usage.

**Resolution:** Cost per run drops from $0.17 to $0.09. Linh has full cost visibility without leaving her Rust code.

### Journey 4: New Contributor — "Setting Up Dev Environment" (Edge Case, Onboarding)

**New Contributor** — wants to contribute to TraitClaw for the first time.

**Opening:** Clones the repo, runs `cargo test --workspace` — all tests pass. But how do they write new tests? Which mocks to use? Where are the test patterns?

**Rising Action:** They find `traitclaw-test-utils` in `crates/`. Reads the doc comments: `MockProvider::sequence(vec![...])`, `MockMemory::new()`, `make_runtime(provider, tools)`. Clear, composable test helpers.

**Climax:** New contributor writes their first test using shared utils. Pushes to a branch. CI runs automatically — they get feedback in minutes, not after a reviewer manually runs tests.

**Resolution:** First PR merged with confidence. The test infrastructure made contributing feel professional and safe.

### Journey Requirements Summary

| Journey | Capabilities Revealed |
|---------|----------------------|
| Journey 1 | `traitclaw-test-utils` shared crate, CI pipeline, coverage reporting |
| Journey 2 | `AgentEvent` enum, `on_event` callback, tracing spans, debugging workflow |
| Journey 3 | `RunUsage.estimated_cost_usd`, per-event token tracking, cost optimization |
| Journey 4 | Test utils documentation, CI for external contributors, onboarding DX |

## Developer Tool Specific Requirements

### Language & Platform

- **Language:** Rust (edition 2021, MSRV 1.75+)
- **Package manager:** Cargo / crates.io
- **IDE integration:** Standard Rust tooling (rust-analyzer, cargo doc) — no IDE-specific plugins

### Installation Methods

```toml
# New test utils crate (dev-dependency only)
[dev-dependencies]
traitclaw-test-utils = { path = "../traitclaw-test-utils" }

# Observability via existing traitclaw meta-crate (no new dep needed)
traitclaw = "0.8.0"  # tracing already included
```

### API Surface (New in v0.8.0)

| Type | Crate | Kind |
|------|-------|------|
| `MockProvider` | `traitclaw-test-utils` | struct (dev-dependency) |
| `MockMemory` | `traitclaw-test-utils` | struct (dev-dependency) |
| `MockTools` (EchoTool, etc.) | `traitclaw-test-utils` | structs (dev-dependency) |
| `make_runtime()` | `traitclaw-test-utils` | helper fn (dev-dependency) |
| `AgentEvent` | `traitclaw-core` | enum |
| `AgentBuilder::on_event()` | `traitclaw-core` | method |
| `RunUsage::estimated_cost_usd` | `traitclaw-core` | field |

### Code Examples (New/Updated)

| Example | Content |
|---------|--------|
| `26-observability` (new) | Full tracing + event callback demo with `tracing-subscriber` |

### Migration Guide

- `docs/migration-v0.7-to-v0.8.md` — Documents new types, zero breaking changes, test migration path

## Project Scoping & Phased Development

### MVP Strategy & Philosophy

**MVP Approach:** Platform MVP — Deliver complete quality infrastructure (test utils + CI + coverage + observability) as a foundation for v0.9.0 Hardening. Not a partial release, but a complete toolkit.

**Simplifications Accepted for MVP:**

- Coverage targets measured but not enforced in CI (blocking enforcement deferred to v0.9.0)
- Cost estimation uses hardcoded model pricing table (dynamic pricing API deferred)
- `AgentEvent` covers core lifecycle only — strategy-specific events (ThoughtStep) deferred

### Risk Mitigation Strategy

| Risk Type | Risk | Mitigation |
|-----------|------|------------|
| **Technical** | `cargo-llvm-cov` may not work on all CI runners | Fallback: coverage runs as optional CI step; local-only initially |
| **Technical** | `AgentEvent` trait design may need revision before v1.0 | Design as additive enum — new variants are non-breaking; `#[non_exhaustive]` |
| **Technical** | Migrating all crate tests to shared utils may surface hidden test coupling | Migrate one crate at a time; core → strategies → team → others |
| **Technical** | Tracing spans may add measurable overhead to hot paths | Use `tracing` level filtering; spans are zero-cost when subscriber is absent |
| **Resource** | Solo developer; combined testing + observability scope | Priority order: test-utils (highest) → CI → tracing → events → cost tracking |

> See **Product Scope → MVP** (Step 3) for full feature list, **Growth Features** for v0.9.0, and **Vision** for v1.0+.

## Functional Requirements

### Testing Infrastructure

- **FR1:** Framework contributor can import shared mock types (`MockProvider`, `MockMemory`) from a single `traitclaw-test-utils` crate
- **FR2:** Framework contributor can create deterministic mock providers with pre-defined response sequences
- **FR3:** Framework contributor can construct a complete `AgentRuntime` for testing via `make_runtime()` helper
- **FR4:** Framework contributor can use shared mock tools (EchoTool, FailTool) for tool-calling test scenarios
- **FR5:** All existing crate-local test mocks are consolidated into `traitclaw-test-utils` with zero duplicates remaining

### Continuous Integration

- **FR6:** CI pipeline can verify code formatting across the entire workspace on every push/PR
- **FR7:** CI pipeline can run `clippy` lint checks across the entire workspace on every push/PR
- **FR8:** CI pipeline can run the full workspace test suite on every push/PR
- **FR9:** CI pipeline can verify that all public API types have documentation
- **FR10:** Framework contributor can generate code coverage reports locally via `cargo-llvm-cov`
- **FR11:** CI pipeline can generate and archive coverage reports for each run

### Runtime Observability — Tracing

- **FR12:** Agent developer can observe structured tracing spans for all LLM provider calls
- **FR13:** Agent developer can observe structured tracing spans for all tool executions
- **FR14:** Agent developer can observe structured tracing spans for guard checks
- **FR15:** Agent developer can filter trace output by component using standard `tracing` levels and targets

### Runtime Observability — Events

- **FR16:** Agent developer can register an event callback on `AgentBuilder` via `on_event()`
- **FR17:** `AgentEvent` enum can represent the full agent lifecycle: `LlmStart`, `LlmEnd`, `ToolCall`, `ToolResult`, `GuardBlock`, `HintTriggered`
- **FR18:** `AgentEvent::LlmEnd` variant includes token usage information (prompt_tokens, completion_tokens)
- **FR19:** Agent developer can use `AgentEvent` to build custom logging, metrics, or debugging workflows

### Cost Tracking

- **FR20:** `RunUsage` can report estimated cost in USD after an agent run
- **FR21:** Cost estimation uses a configurable model pricing table
- **FR22:** Agent developer can access per-invocation token counts and cumulative costs

### Documentation & Migration

- **FR23:** A migration guide (`v0.7-to-v0.8.md`) documents all new types, zero breaking changes, and test migration path
- **FR24:** `traitclaw-test-utils` public API has comprehensive doc comments with usage examples
- **FR25:** An observability example (`26-observability`) demonstrates tracing + event callback end-to-end

### Backward Compatibility

- **FR26:** All v0.7.0 public APIs remain functional without changes
- **FR27:** All v0.7.0 examples compile and run on v0.8.0 without modification
- **FR28:** `AgentEvent` is `#[non_exhaustive]` to allow future variant additions without breaking changes

## Non-Functional Requirements

### Performance

- **NFR1:** Tracing spans have zero runtime overhead when no `tracing` subscriber is registered (compile-time optimization via `tracing` crate's no-subscriber fast path)
- **NFR2:** `AgentEvent` callback invocation adds ≤ 1μs per event when callback is registered
- **NFR3:** `RunUsage` cost calculation adds negligible overhead (simple arithmetic on cached pricing table)
- **NFR4:** CI pipeline completes full workspace check (fmt + clippy + test + doc) in ≤ 10 minutes on GitHub Actions standard runner
- **NFR5:** `traitclaw-test-utils` mock types add zero compile-time cost to non-test builds (dev-dependency isolation)

### Integration & Compatibility

- **NFR6:** Tracing spans are compatible with any `tracing::Subscriber` implementation (tracing-subscriber, opentelemetry-tracing, custom subscribers)
- **NFR7:** `AgentEvent` callback signature is `Fn(&AgentEvent) + Send + Sync + 'static` — compatible with standard Rust closure patterns
- **NFR8:** `traitclaw-test-utils` works with `#[tokio::test]` and standard `#[test]` contexts
- **NFR9:** CI pipeline uses stable Rust toolchain only — no nightly features required
- **NFR10:** All new public types implement standard Rust traits: `Debug`, `Clone` (where applicable), `Send + Sync`
