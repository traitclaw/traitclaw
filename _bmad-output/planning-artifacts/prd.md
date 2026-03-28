---
stepsCompleted: ["step-01-init.md", "step-02-discovery.md", "step-02b-vision.md", "step-02c-executive-summary.md", "step-03-success.md", "step-04-journeys.md", "step-05-domain.md", "step-06-innovation.md", "step-07-project-type.md", "step-08-scoping.md", "step-09-functional.md", "step-10-nonfunctional.md", "step-11-polish.md", "step-12-complete.md"]
inputDocuments: ["product-brief-traitclaw-2026-03-26.md", "prd-v0.6.0.md", "brainstorming/v070-reasoning-strategies.md", "project-context.md"]
workflowType: 'prd'
classification:
  projectType: developer_tool
  domain: AI / Developer Tool
  complexity: high
  projectContext: brownfield
---

# Product Requirements Document - traitclaw v0.7.0 "Reasoning"

**Author:** Bangvu
**Date:** 2026-03-28

## Executive Summary

TraitClaw v0.7.0 "Reasoning" extends the framework from agent composition into agent intelligence. Building on v0.6.0's `AgentFactory`, `AgentPool`, and `RoundRobinGroupChat`, this release delivers production-ready reasoning strategies — `ReActStrategy`, `MctsStrategy`, and `ChainOfThoughtStrategy` — in a new `traitclaw-strategies` crate. Users import and apply strategies in ≤3 lines; no custom reasoning loop design required.

Additionally, v0.7.0 introduces `StreamingOutputTransformer` in `traitclaw-core`, enabling real-time streaming transformation of agent output — critical for ReAct loops where users need to see each Think→Act→Observe step as it happens.

The release targets Rust developers who already use TraitClaw for multi-agent systems and need their agents to reason better without building reasoning infrastructure from scratch. No Rust AI framework currently ships built-in reasoning strategies — TraitClaw v0.7.0 closes this gap.

### What Makes This Special

1. **Batteries-included reasoning** — Three production-ready strategies (ReAct, MCTS, CoT) ship out of the box. Feature flags default all-on; power users opt-out per strategy for minimal binary size.
2. **New dedicated crate** — `traitclaw-strategies` keeps `traitclaw-core` minimal while providing zero-cost access to advanced reasoning patterns.
3. **Streaming-native design** — `StreamingOutputTransformer` enables real-time thought-step streaming, transforming the UX of reasoning-heavy agents from blocking to progressive.
4. **Consistent with existing patterns** — All strategies implement the existing `AgentStrategy` trait from v0.2.0; users already familiar with custom strategies can inspect, extend, or replace any built-in.

## Project Classification

| Attribute | Value |
|-----------|-------|
| **Project Type** | Developer Tool (Rust Framework/Library) |
| **Domain** | AI / Developer Tooling |
| **Complexity** | High |
| **Project Context** | Brownfield (building on TraitClaw v0.6.0) |
| **New Crate** | `traitclaw-strategies` |
| **Breaking Changes** | Zero — purely additive |

## Success Criteria

### User Success

| Metric | Target | Persona |
|--------|--------|---------|
| **Time to first reasoning agent** | ≤ 10 minutes from `cargo add` → ReAct agent running | Minh (Rust dev) |
| **Strategy swap effort** | Switch strategy (ReAct ↔ MCTS ↔ CoT) in ≤ 3 lines of code | Minh |
| **Custom strategy migration** | Users with custom `AgentStrategy` upgrade v0.6→v0.7 with zero changes | Minh, Linh |
| **Streaming thought UX** | ReAct thought steps stream progressively, not blocking | Linh |

### Business Success

| Metric | Target (6 months post-release) |
|--------|-------------------------------|
| **Example coverage** | 28+ runnable examples (current ~24) |
| **Docs coverage** | 100% public API of `traitclaw-strategies` documented |
| **Adoption signal** | ≥ 3 community examples/blog posts using built-in strategies |

### Technical Success

| Metric | Target |
|--------|--------|
| **Backward compatibility** | All v0.6.0 examples compile on v0.7.0 — zero breaking changes |
| **Compile time** | < 2% increase with `traitclaw-strategies` default features |
| **MSRV** | Rust 1.75+ maintained |
| **Test coverage** | ≥ 80% line coverage for `traitclaw-strategies` |
| **No new required deps** | Core strategies work with existing deps; MCTS parallelism uses `tokio` (already required) |

### Measurable Outcomes

- A new user can run a ReAct agent from the example in ≤ 10 minutes
- Swapping from `ReActStrategy` to `MctsStrategy` requires changing only the strategy constructor — all other agent code unchanged
- `StreamingOutputTransformer` produces visible incremental output within 500ms of first token
- `cargo build --timings` shows < 2% regression vs v0.6.0 baseline

## Product Scope

### MVP — v0.7.0 Release

1. `ReActStrategy` — Think→Act→Observe loop with tool calling
2. `MctsStrategy` — Monte Carlo Tree Search with parallel branch evaluation
3. `ChainOfThoughtStrategy` — Step-by-step reasoning injection
4. `StreamingOutputTransformer` trait in `traitclaw-core`
5. New crate `traitclaw-strategies` with feature flags (default all-on)
6. Upgraded `examples/11-custom-strategy` + new per-strategy examples
7. Migration guide `docs/migration-v0.6-to-v0.7.md`
8. README update with "Reasoning Strategies" section

> See **Project Scoping & Phased Development** for MVP strategy, simplifications, post-MVP roadmap (v0.8.0, v0.9.0+), and risk mitigation.

## User Journeys

### Journey 1: Minh — "ReAct Agent from Zero" (Primary, Success Path)

**Minh** — Senior Rust dev, fintech. Needs an agent that autonomously calls tools (stock price lookup, risk calculation) instead of hardcoded if/else logic.

**Opening:** Minh has a v0.6.0 agent using `DefaultStrategy` — agent only chats, no reasoning loop. Tool calling is manual.
**Rising Action:** `cargo add traitclaw-strategies`, import `ReActStrategy`, swap `.strategy(ReActStrategy::new())` into the builder. 3 lines of code.
**Climax:** Agent autonomously reasons "I need stock prices" → calls `StockPriceTool` → receives result → reasons "Now I need to calculate risk" → calls `RiskCalcTool` → synthesizes final answer. **No loop logic coded by hand.**
**Resolution:** Production-ready reasoning agent in 10 minutes. Minh continues using custom tools — only the strategy changed.

### Journey 2: Linh — "Streaming Thoughts for Users" (Primary, Edge Case)

**Linh** — AI engineer from Python, familiar with LangChain streaming. Wants ChatGPT-like UX — users see the agent think step-by-step.

**Opening:** Linh builds a chatbot with TraitClaw. ReAct works but users wait 15-20s for the full response.
**Rising Action:** Linh adds `StreamingOutputTransformer` — transforms each ReAct step into a streamed chunk. Attaches to agent builder.
**Climax:** Users see: "🤔 Thinking: need to check weather..." → "🔧 Calling: WeatherTool..." → "📊 Observed: 25°C, clear sky" → "✅ The weather is..."
**Resolution:** UX improves drastically — users engage more, feel in control. Linh didn't redesign the agent, just added a transformer.

### Journey 3: Minh — "MCTS for Complex Decisions" (Primary, Advanced)

**Minh** — Needs an agent that evaluates multiple investment strategies in parallel then selects the best option.

**Opening:** ReAct follows a single reasoning path. Minh needs the agent to explore 5-10 different approaches and score each path.
**Rising Action:** Swap `ReActStrategy` → `MctsStrategy::new().with_branches(5).with_depth(3)`. Same tools, same system prompt.
**Climax:** Agent creates 5 parallel reasoning branches, each exploring a different investment approach, self-scores, backpropagates scores, selects the best-scoring path.
**Resolution:** Portfolio recommendation quality improves thanks to tree search instead of single reasoning thread.

### Journey 4: Power User — "Custom Strategy Coexistence" (Edge Case)

A user already has a custom `AgentStrategy` from v0.2.0. Upgrades to v0.7.0.

**Opening:** `cargo update traitclaw` — compile, zero errors. Custom strategy still works.
**Rising Action:** User wants to try ReAct for one agent. Imports `ReActStrategy`, uses it for a new agent. Old agent keeps custom strategy.
**Climax:** Both custom strategy and built-in ReAct run in the same pipeline. Zero conflict.
**Resolution:** Progressive adoption — user gradually replaces custom strategies with built-in ones as appropriate.

### Journey Requirements Summary

| Journey | Capabilities Revealed |
|---------|----------------------|
| Journey 1 | `ReActStrategy`, tool-calling loop, strategy swap API |
| Journey 2 | `StreamingOutputTransformer`, per-step streaming, transformer composition |
| Journey 3 | `MctsStrategy`, parallel branching, scoring, configurable depth/width |
| Journey 4 | Backward compatibility, strategy coexistence, zero breaking changes |

## Innovation & Novel Patterns

### Detected Innovation Areas

1. **First Rust AI framework with built-in reasoning strategies** — No existing Rust framework (Rig, Swarms-RS) ships ReAct, MCTS, or CoT built-in. Python frameworks (LangChain, CrewAI) provide these, but Rust has zero options. TraitClaw v0.7.0 establishes a new paradigm for the Rust AI ecosystem.

2. **Strategy-as-trait composability** — Users swap strategies the same way they swap providers — same interface, different implementation. Built-in strategies implement the same `AgentStrategy` trait users already use for custom strategies. Zero learning curve for strategy adoption.

3. **Streaming-integrated reasoning** — `StreamingOutputTransformer` combined with reasoning loops enables streaming at thought-step granularity. No AI framework (Rust or Python) provides this level of streaming integration out of the box.

### Validation Approach

- **Built-in strategies** validated via runnable examples using mock providers (offline, deterministic)
- **Performance** validated: `cargo build --timings` must show < 2% regression vs v0.6.0
- **Strategy correctness** validated: unit tests verify reasoning loop behavior (Think→Act→Observe cycle count, tool call sequences, termination conditions)
- **MCTS tree search** validated: tests verify branching factor, depth limits, and score backpropagation

> See **Project Scoping & Phased Development → Risk Mitigation Strategy** for consolidated risk analysis.

## Developer Tool Specific Requirements

### Language & Platform

- **Language:** Rust (edition 2021, MSRV 1.75+)
- **Package manager:** Cargo / crates.io
- **IDE integration:** Standard Rust tooling (rust-analyzer, cargo doc) — no IDE-specific plugins

### Installation Methods

```toml
# Full batteries-included (default)
traitclaw-strategies = "0.7.0"

# Selective strategies
traitclaw-strategies = { version = "0.7.0", default-features = false, features = ["react"] }

# Via meta-crate re-export
traitclaw = { version = "0.7.0", features = ["strategies"] }
```

### API Surface (New in v0.7.0)

| Type | Crate | Kind |
|------|-------|------|
| `ReActStrategy` | `traitclaw-strategies` | struct impl `AgentStrategy` |
| `MctsStrategy` | `traitclaw-strategies` | struct impl `AgentStrategy` |
| `MctsConfig` | `traitclaw-strategies` | config struct (branches, depth, scoring) |
| `ChainOfThoughtStrategy` | `traitclaw-strategies` | struct impl `AgentStrategy` |
| `StreamingOutputTransformer` | `traitclaw-core` | trait |
| `ThoughtStep` | `traitclaw-strategies` | enum (Think, Act, Observe, Answer) |

### Code Examples (New/Updated)

| Example | Content |
|---------|--------|
| `11-custom-strategy` (upgrade) | Updated to v0.7.0 patterns, mock provider, working demo |
| `25-react-strategy` (new) | ReAct loop with tool calling |
| `26-mcts-strategy` (new) | MCTS tree search with branch configuration |
| `27-chain-of-thought` (new) | CoT reasoning with streaming output |
| `28-streaming-transformer` (new) | StreamingOutputTransformer usage |

### Migration Guide

- `docs/migration-v0.6-to-v0.7.md` — Documents all new types, zero breaking changes, adoption examples

## Project Scoping & Phased Development

### MVP Strategy & Philosophy

**MVP Approach:** Platform MVP — Deliver complete reasoning strategy infrastructure that enables users to immediately build smarter agents. Not a minimal demo, but a complete toolkit.

### MVP Feature Set (Phase 1 — v0.7.0)

**Must-Have (Go/No-Go):**

1. `ReActStrategy` — Core reasoning loop (supports Journey 1, 2)
2. `MctsStrategy` — Tree search reasoning (supports Journey 3)
3. `ChainOfThoughtStrategy` — Step reasoning (completes strategy trio)
4. `StreamingOutputTransformer` trait — Streaming support (supports Journey 2)
5. `traitclaw-strategies` crate with feature flags
6. Per-strategy working examples (25–28)
7. Migration guide v0.6→v0.7
8. Zero breaking changes verified

**Simplifications Accepted for MVP:**

- MCTS scoring: ship with default scoring function, custom `ScoringFn` in v0.7.1
- CoT step count: fixed max steps, adaptive termination in v0.7.1

### Post-MVP Features (Phase 2 — v0.8.0)

- DAG execution engine
- Config-driven agent spawn (YAML/TOML)
- `OrchestrationStrategy` trait
- Retry/checkpoint/fallback agents

### Expansion (Phase 3 — v0.9.0+)

- Distributed agent execution
- WASM deployment target
- Inter-agent typed contracts
- OpenTelemetry integration

### Risk Mitigation Strategy

| Risk Type | Risk | Mitigation |
|-----------|------|------------|
| **Technical** | MCTS tree search complexity may delay release | Feature-gated; ship ReAct + CoT first, MCTS can land as v0.7.1 patch if needed |
| **Technical** | `StreamingOutputTransformer` trait design may conflict with existing `OutputTransformer` | Additive-only design — new trait, does not modify existing chain |
| **Technical** | Strategy API stability | Implement existing `AgentStrategy` trait — no new API surface to stabilize |
| **Technical** | Feature flag overhead | Default all-on for batteries-included DX; `default-features = false` for minimal builds |
| **Market** | Users may not need MCTS for typical agentic workloads | Feature-gated opt-in; no binary cost if unused |
| **Resource** | Solo developer; large scope | Priority order: ReAct (highest value) → CoT → Streaming → MCTS |

## Functional Requirements

### Reasoning Strategy Core

- **FR1:** Developer can instantiate a `ReActStrategy` with default configuration and assign it to an agent via builder
- **FR2:** Developer can configure `ReActStrategy` max iterations (loop limit) and tool-calling behavior
- **FR3:** `ReActStrategy` can autonomously execute Think→Act→Observe loops until answering or hitting max iterations
- **FR4:** Developer can instantiate a `MctsStrategy` with configurable branch count and search depth
- **FR5:** `MctsStrategy` can spawn parallel reasoning branches, score each path, and select the highest-scoring result
- **FR6:** Developer can instantiate a `ChainOfThoughtStrategy` with configurable max steps
- **FR7:** `ChainOfThoughtStrategy` can inject step-by-step reasoning into the agent's prompt and return structured thought steps

### Strategy Interchangeability

- **FR8:** Developer can swap any built-in strategy for another by changing only the strategy constructor — no other code changes required
- **FR9:** Developer can use built-in strategies alongside custom `AgentStrategy` implementations in the same application
- **FR10:** All built-in strategies implement the existing `AgentStrategy` trait without modifications to the trait interface

### Streaming Output

- **FR11:** Developer can implement `StreamingOutputTransformer` trait to transform agent output as it streams
- **FR12:** `StreamingOutputTransformer` can be composed into the agent builder pipeline alongside existing `OutputTransformer`
- **FR13:** Developer can use `StreamingOutputTransformer` with `ReActStrategy` to stream individual thought steps in real-time

### Crate & Feature Management

- **FR14:** Developer can add `traitclaw-strategies` as a standalone dependency via `cargo add`
- **FR15:** Developer can selectively enable/disable individual strategies via Cargo feature flags
- **FR16:** All strategies are enabled by default (batteries-included); developer can opt-out with `default-features = false`
- **FR17:** `traitclaw-strategies` re-exports through the `traitclaw` meta-crate via `features = ["strategies"]`

### Thought Step Observability

- **FR18:** `ReActStrategy` emits typed `ThoughtStep` events (Think, Act, Observe, Answer) during execution
- **FR19:** Developer can inspect `ThoughtStep` sequence after strategy execution for debugging/logging
- **FR20:** `MctsStrategy` exposes branch scores and selected path for post-execution analysis

### Backward Compatibility

- **FR21:** All v0.6.0 public APIs remain unchanged and functional in v0.7.0
- **FR22:** All existing examples (1–24) compile and run without modification on v0.7.0
- **FR23:** Custom `AgentStrategy` implementations created against v0.2.0+ trait work without changes

### Documentation & Examples

- **FR24:** Each built-in strategy has a dedicated runnable example demonstrating core usage
- **FR25:** `StreamingOutputTransformer` has a dedicated example demonstrating streaming thought steps
- **FR26:** Migration guide documents all new types and adoption path from v0.6.0
- **FR27:** All new public types have complete rustdoc documentation with code examples

## Non-Functional Requirements

### Performance

| NFR | Metric | Target |
|-----|--------|--------|
| **NFR1: Compile time** | `cargo build --timings` delta vs v0.6.0 | < 2% increase with default features |
| **NFR2: Binary size** | `cargo bloat` delta vs v0.6.0 | < 5% increase with default features |
| **NFR3: Runtime overhead** | ReAct loop latency per iteration (excluding LLM call) | < 1ms per Think→Act→Observe cycle |
| **NFR4: MCTS parallelism** | Tokio task spawn overhead per branch | < 100μs per branch spawn |
| **NFR5: Streaming latency** | Time from first token to first `StreamingOutputTransformer` emission | < 10ms |
| **NFR6: Feature-gated build** | Build time with `default-features = false` + single strategy | < 50% of full build time |

### Integration

| NFR | Requirement |
|-----|-------------|
| **NFR7: MSRV** | Rust 1.75+ — no nightly features required |
| **NFR8: Async runtime** | `tokio` only (consistent with existing crate ecosystem) |
| **NFR9: Dependency budget** | Zero new required dependencies for `traitclaw-strategies` beyond `traitclaw-core` and `tokio` |
| **NFR10: API ergonomics** | All strategy constructors follow builder pattern consistent with `AgentBuilder` |
| **NFR11: Error types** | Strategy errors use existing `TraitClawError` enum — no new top-level error types |
| **NFR12: Trait object safety** | Built-in strategies must be object-safe (`dyn AgentStrategy`) for dynamic dispatch |
