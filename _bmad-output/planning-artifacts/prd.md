---
stepsCompleted: ["step-01-init.md", "step-02-discovery.md", "step-02b-vision.md", "step-02c-executive-summary.md", "step-03-success.md", "step-04-journeys.md", "step-05-features.md", "step-06-nfr.md", "step-07-ux.md", "step-08-tech.md", "step-09-risks.md", "step-10-compliance.md", "step-11-review.md", "step-12-complete.md"]
inputDocuments: ["brainstorm_openness.md", "project-context.md", "architecture.md", "epics.md"]
workflowType: 'prd'
classification:
  projectType: developer_tool
  domain: AI / Developer Tool
  complexity: medium
  projectContext: brownfield
---

# Product Requirements Document — TraitClaw v0.2.0 "The Openness Update"

**Author:** Bangvu
**Date:** 2026-03-25
**Version:** 0.2.0

---

## Executive Summary

TraitClaw v0.2.0 ("The Openness Update") evolves the existing trait-driven Rust AI Agent framework from a functional but closed-loop system into a fully extensible platform. The update introduces three new core traits — `AgentStrategy`, `AgentHook`, and `Router` — alongside two smart implementations (`CompressedMemory`, `SmartToolRegistry`) to unlock advanced reasoning architectures (MCTS, ReAct), enterprise-grade observability (OpenTelemetry, DataDog), and non-linear multi-agent orchestration. The fundamental insight driving v0.2.0 is that an Agent is simply `f(context) → action`; every extension point is merely a way to organize how that function is invoked, observed, or composed. This keeps the API surface minimal while maximizing power for advanced users.

### What Makes This Special

Unlike Python/TypeScript AI frameworks that force developers to subclass monolithic `Agent` classes or learn proprietary DSLs, TraitClaw leverages Rust's trait system to provide **compile-time-verified extensibility** with zero runtime overhead on the default path. Key architectural decisions — dynamic dispatch for Strategy (LLM latency >> vtable overhead), async hooks (Rust 1.75+ native support), simple `Router` trait (no forced graph dependency), and decorator pattern for Memory — ensure that newcomers get a working Agent in 5 lines while power users can swap every component without forking the framework.

## Project Classification

- **Project Type:** Developer Tool (Rust Framework/Library, published as 12 crates on crates.io)
- **Domain:** AI / Developer Tooling
- **Complexity:** Medium (requires precise trait abstraction design)
- **Project Context:** Brownfield (building on TraitClaw v0.1.0 with 12 published crates)

---

## Success Criteria

### User Success

- **Newcomer onboarding:** A Rust developer unfamiliar with TraitClaw can build a working Agent with custom Strategy in under 30 minutes using the provided examples.
- **Zero-disruption upgrade:** Existing v0.1.0 code compiles and runs on v0.2.0 without modification. All new traits have Noop/Default implementations.
- **Extensibility clarity:** A developer can identify which trait to implement for their use case (observability → Hook, custom loop → Strategy, team routing → Router) from the README alone.

### Business Success

- **Ecosystem adoption:** 500+ downloads/month on crates.io within 3 months of v0.2.0 release.
- **Community engagement:** At least 5 community-contributed implementations (custom Strategies, Hooks, or Routers) within 6 months.
- **Competitive positioning:** Listed in Rust AI framework comparisons as the most extensible Rust alternative.

### Technical Success

- **Zero regression:** All existing tests pass. No performance degradation on the default path.
- **Minimal API surface:** No more than 3 new traits added to `traitclaw-core`.
- **No new mandatory dependencies:** All new functionality is feature-gated or implemented as optional crates.

### Measurable Outcomes

| Metric | Target | Measurement |
|--------|--------|-------------|
| Compile time (default features) | < 5% increase | `cargo build --timings` |
| Binary size (default features) | < 3% increase | `cargo bloat` |
| API surface (trait count) | ≤ 3 new traits in core | Manual count |
| Example coverage | 1 example per new trait | Directory listing |
| Doc coverage | 100% public items documented | `cargo doc --no-deps` |

---

## Product Scope

### MVP — Minimum Viable Product (v0.2.0)

1. **`AgentStrategy` trait** — Dynamic dispatch (`Box<dyn AgentStrategy>`) with `DefaultStrategy` preserving current loop behavior.
2. **`AgentHook` trait** — Async lifecycle hooks (`on_provider_start`, `on_provider_end`, `before_tool_execute`, `after_tool_execute`, `on_stream_chunk`). NoopHook default.
3. **`Router` trait** — Simple `fn route(msg, state) → AgentId` for `traitclaw-team`. `SequentialRouter` and `LeaderRouter` defaults.
4. **`CompressedMemory`** — Decorator wrapping any `Memory` implementation with automatic context summarization.
5. **Examples** — Dedicated examples for Strategy, Hook, Router, and CompressedMemory.
6. **Documentation** — Updated README, API docs, and migration guide.

### Growth Features (Post-MVP — v0.3.0)

- `SmartToolRegistry` — Semantic search-based dynamic tool selection.
- `OutputParser` trait — Custom format parsing for local LLMs without JSON mode.
- Built-in `MctsStrategy` and `ReActStrategy` implementations.
- OpenTelemetry Hook implementation crate (`traitclaw-otel`).

### Vision (Future — v1.0)

- Visual workflow builder (graph-based UI for Agent orchestration).
- Cloud-native deployment with horizontal scaling support.
- Marketplace for community Strategies, Hooks, and Tools.

---

## User Journeys

### Journey 1: "I want to customize the Agent loop"

**Persona:** AI Researcher building a tree-search reasoning system.

```
Discovery → Reads README → Sees `AgentStrategy` trait
→ Reads `examples/10-custom-strategy/` → Implements `MctsStrategy`
→ Plugs into existing Agent: `agent.set_strategy(MctsStrategy::new())`
→ Agent now uses MCTS instead of default sequential loop
→ Shares implementation with community
```

### Journey 2: "I need observability for production"

**Persona:** Platform Engineer deploying TraitClaw agents at scale.

```
Requirement → Need latency tracking + token cost monitoring
→ Implements `AgentHook` with OpenTelemetry spans
→ `before_tool_execute` starts span, `after_tool_execute` ends span
→ `on_provider_end` records token usage metrics
→ Plugs in: `agent.hook(OtelHook::new(exporter))`
→ Full traces visible in Jaeger/DataDog without modifying agent logic
```

### Journey 3: "I want agents to collaborate intelligently"

**Persona:** Developer building a code review pipeline with multiple specialized agents.

```
Design → CodeAgent writes code, ReviewAgent reviews, DeployAgent deploys
→ Implements `Router`: if review fails → route back to CodeAgent
→ Uses `LeaderRouter` as starting point, modifies routing logic
→ `traitclaw-team` handles message passing, Router handles "who's next"
→ Non-linear multi-agent workflow without external graph libraries
```

### Journey 4: "My agent runs out of context window"

**Persona:** Developer running long-lived chat agents that hit token limits.

```
Problem → Agent crashes after 50+ messages (exceeds context window)
→ Wraps existing memory: `CompressedMemory::wrap(SqliteMemory::new(...))`
→ CompressedMemory auto-summarizes old messages when approaching 80% capacity
→ Agent receives: [Summary of first 45 messages] + [Last 5 messages]
→ Runs indefinitely without hitting context limits or excessive API costs
```

---

## Feature Requirements

### F1: `AgentStrategy` Trait

**Priority:** P0 (Must-have)
**Crate:** `traitclaw-core`

```rust
#[async_trait]
pub trait AgentStrategy: Send + Sync + 'static {
    async fn execute(
        &self,
        agent: &AgentRuntime,
        input: &str,
    ) -> Result<AgentOutput>;
}
```

| Requirement | Detail |
|-------------|--------|
| Dispatch | `Box<dyn AgentStrategy>` (dynamic) |
| Default | `DefaultStrategy` — preserves current v0.1.0 loop behavior |
| Builder integration | `Agent::builder().strategy(MctsStrategy::new())` |
| Backward compat | If no strategy set, uses `DefaultStrategy` automatically |

### F2: `AgentHook` Trait

**Priority:** P0 (Must-have)
**Crate:** `traitclaw-core`

```rust
pub trait AgentHook: Send + Sync + 'static {
    async fn on_provider_start(&self, request: &CompletionRequest) {}
    async fn on_provider_end(&self, response: &CompletionResponse, duration: Duration) {}
    async fn before_tool_execute(&self, name: &str, args: &Value) -> HookAction { HookAction::Continue }
    async fn after_tool_execute(&self, name: &str, result: &Value) {}
    async fn on_stream_chunk(&self, chunk: &str) {}
    async fn on_error(&self, error: &AgentError) {}
}
```

| Requirement | Detail |
|-------------|--------|
| Concurrency | All methods are `async fn` (Rust 1.75+ native) |
| Default impl | All methods have empty default implementations (NoopHook) |
| Interception | `before_tool_execute` returns `HookAction::Continue` or `HookAction::Block(reason)` |
| Multiple hooks | `Vec<Box<dyn AgentHook>>` — multiple hooks can be registered |
| Builder integration | `Agent::builder().hook(OtelHook::new()).hook(SecurityHook::new())` |

### F3: `Router` Trait

**Priority:** P0 (Must-have)
**Crate:** `traitclaw-team`

```rust
pub trait Router: Send + Sync + 'static {
    fn route(&self, message: &TeamMessage, state: &TeamState) -> RoutingDecision;
}

pub enum RoutingDecision {
    SendTo(AgentId),
    Broadcast,
    Complete(String),
}
```

| Requirement | Detail |
|-------------|--------|
| Design | Simple trait — no graph engine dependency |
| Defaults | `SequentialRouter` (round-robin), `LeaderRouter` (coordinator pattern) |
| No new deps | No `petgraph` or similar — users bring their own if needed |
| Builder integration | `Team::builder().router(CustomRouter::new())` |

### F4: `CompressedMemory` Decorator

**Priority:** P1 (Should-have)
**Crate:** `traitclaw-core` or new `traitclaw-memory-compressed`

```rust
pub struct CompressedMemory<M: Memory> {
    inner: M,
    compressor: Box<dyn Provider>,  // cheap LLM for summarization
    threshold: f32,                  // 0.8 = compress at 80% capacity
}

impl<M: Memory> Memory for CompressedMemory<M> { /* delegates + compresses */ }
```

| Requirement | Detail |
|-------------|--------|
| Pattern | Decorator — wraps any `Memory` implementation |
| Stackable | `CompressedMemory::wrap(CachedMemory::wrap(SqliteMemory::new(...)))` |
| Trigger | Auto-compresses when message count exceeds threshold % of context window |
| Transparency | Downstream code sees a normal `Memory` — no API changes |

### F5: Examples

**Priority:** P0 (Must-have)

| Example | Demonstrates |
|---------|-------------|
| `10-custom-strategy/` | Implementing a simple ReAct-style strategy |
| `11-lifecycle-hooks/` | Logging hook, timing hook, security interception hook |
| `12-custom-router/` | State-machine router for a code review pipeline |
| `13-compressed-memory/` | Long-running agent with automatic context management |

---

## Non-Functional Requirements

### Performance

- Zero overhead on default path: When using `DefaultStrategy` + `NoopHook`, performance must be identical to v0.1.0.
- Hook overhead budget: Each hook call adds no more than 1μs when hooks are registered but have empty implementations.
- Dynamic dispatch: `Box<dyn AgentStrategy>` vtable lookup is acceptable (nanoseconds vs. LLM milliseconds).

### Compatibility

- **Backward compatibility:** v0.1.0 code compiles without changes on v0.2.0.
- **MSRV:** Rust 1.75+ (required for `async fn` in traits).
- **Semver:** This is a minor version bump (0.1.0 → 0.2.0). No breaking changes.

### Documentation

- All public traits and methods must have rustdoc with examples.
- Migration guide from v0.1.0 to v0.2.0.
- Architecture Decision Records (ADR) published in `/docs/adr/`.

### Testing

- Unit tests for all default implementations (DefaultStrategy, NoopHook, SequentialRouter, LeaderRouter).
- Integration tests demonstrating custom Strategy/Hook/Router.
- Backward compatibility test: v0.1.0 example code compiles unmodified.

---

## Technical Constraints

### Architecture Decisions (from ADR Session)

| ADR | Decision | Rationale |
|-----|----------|-----------|
| ADR-1 | `Box<dyn AgentStrategy>` (dynamic dispatch) | LLM latency (200-2000ms) >> vtable overhead (ns). Simpler API. |
| ADR-2 | `async fn` hooks (Rust 1.75+ native) | No `#[async_trait]` needed. Non-blocking observability. |
| ADR-3 | Simple `trait Router` (no graph dependency) | Minimal deps. Users bring `petgraph` if needed. |
| ADR-4 | Decorator pattern for `CompressedMemory` | Open/Closed Principle. Stackable wrappers. No API changes. |

### Dependency Policy

- **No new required dependencies** for core functionality.
- `CompressedMemory` may live in a feature-gated crate to avoid coupling core to a specific LLM provider for summarization.
- All new traits go into `traitclaw-core`. All implementations that require additional dependencies go into feature-gated crates.

### Crate Impact Map

| Crate | Changes |
|-------|---------|
| `traitclaw-core` | Add `AgentStrategy`, `AgentHook` traits + defaults. Modify `Agent` struct to accept Strategy and Hook. |
| `traitclaw-team` | Add `Router` trait + `SequentialRouter`, `LeaderRouter`. Refactor orchestration to use Router. |
| `traitclaw` (meta) | Re-export new traits. Update feature flags if needed. |
| New examples | `10-custom-strategy/`, `11-lifecycle-hooks/`, `12-custom-router/`, `13-compressed-memory/` |

---

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Breaking backward compatibility | Medium | High | Comprehensive test suite. All new traits have default impls. Run v0.1.0 examples against v0.2.0. |
| Over-engineering Strategy trait | Medium | Medium | Start with simplest possible interface. Add complexity only when real use cases demand it (YAGNI). |
| Async Hook performance overhead | Low | Medium | Benchmark with empty hooks. Budget: < 1μs per call. Use `tokio::spawn` for expensive hooks if needed. |
| CompressedMemory summarization quality | Medium | Medium | Ship as opt-in feature. Provide clear docs on choosing the summarization LLM. Allow users to implement their own compression strategy. |
| Community adoption friction | Medium | Medium | Comprehensive examples + migration guide. Announce on r/rust and Rust AI communities. |

---

## Release Plan

### Phase 1: Core Traits (Week 1-2)

- [ ] Design and implement `AgentStrategy` trait in `traitclaw-core`
- [ ] Implement `DefaultStrategy` (extract current loop logic)
- [ ] Design and implement `AgentHook` trait in `traitclaw-core`
- [ ] Implement `NoopHook` default
- [ ] Modify `Agent` struct and `AgentBuilder` to accept Strategy and Hook
- [ ] Backward compatibility tests

### Phase 2: Team & Memory (Week 3-4)

- [ ] Design and implement `Router` trait in `traitclaw-team`
- [ ] Implement `SequentialRouter` and `LeaderRouter`
- [ ] Refactor existing team orchestration to use Router
- [ ] Implement `CompressedMemory` decorator
- [ ] Integration tests for Router and CompressedMemory

### Phase 3: Examples & Docs (Week 5-6)

- [ ] Create `examples/10-custom-strategy/`
- [ ] Create `examples/11-lifecycle-hooks/`
- [ ] Create `examples/12-custom-router/`
- [ ] Create `examples/13-compressed-memory/`
- [ ] Write migration guide (v0.1.0 → v0.2.0)
- [ ] Publish ADRs to `/docs/adr/`
- [ ] Update README with v0.2.0 features

### Phase 4: Release (Week 7)

- [ ] Version bump all crates to 0.2.0
- [ ] Final CI/CD checks
- [ ] Publish to crates.io
- [ ] Announce on r/rust, Rust AI communities
