---
stepsCompleted: ["step-01-init.md", "step-02-discovery.md", "step-02b-vision.md", "step-02c-executive-summary.md", "step-03-success.md", "step-04-journeys.md", "step-05-domain.md", "step-06-innovation.md", "step-07-project-type.md", "step-08-scoping.md", "step-09-risks.md", "step-10-compliance.md", "step-11-review.md", "step-12-complete.md"]
inputDocuments: ["prd-v0.5.0.md", "prd-v0.4.0.md", "docs/migration-v0.4-to-v0.5.md"]
workflowType: 'prd'
classification:
  projectType: developer_tool
  domain: AI / Developer Tool
  complexity: high
  projectContext: brownfield
---

# Product Requirements Document — TraitClaw v0.6.0 "Composition"

**Author:** Bangvu
**Date:** 2026-03-26
**Version:** 0.6.0
**Depends on:** v0.5.0 (Ecosystem)

---

## Executive Summary

TraitClaw v0.6.0 ("Composition") completes the ergonomics arc of the framework. Where v0.5.0 made every crate *executable*, v0.6.0 makes composing multiple agents *effortless*. The single largest source of friction in TraitClaw's multi-agent API — spinning up N agents from the same provider with different system prompts — is solved with `AgentFactory`, `AgentPool`, and `Agent::with_system()`. A content pipeline that required 30 lines of builder boilerplate now takes 5.

This release draws a deliberate parallel to AutoGen's `AssistantAgent` ergonomics and CrewAI's declarative agent definition, while preserving what makes TraitClaw unique: compile-time verified composition, zero-cost abstractions, and no runtime YAML magic.

### What Makes This Special

Every other Rust AI framework requires developers to repeat provider configuration for every agent instance. TraitClaw v0.6.0 introduces a shared-provider model: one `AgentFactory` holds the provider config, and `factory.spawn("Your role here")` creates fully-configured agents. Combined with `AgentPool::from_team()`, an entire team can be instantiated from its `Team` definition in a single call — bridging the gap between TraitClaw's declarative team model and its runtime execution engine.

This is the ergonomics unlock that makes TraitClaw competitive with Python frameworks for multi-agent use cases, without sacrificing Rust's correctness guarantees.

## Project Classification

- **Project Type:** Developer Tool (Rust Framework/Library)
- **Domain:** AI / Developer Tooling
- **Complexity:** High (multi-crate coordination, trait design, backward compatibility)
- **Project Context:** Brownfield (building on TraitClaw v0.5.0 "Ecosystem")

---

## Success Criteria

### User Success

- **Multi-agent in < 10 lines:** Developer creates 5 specialized agents from one provider in fewer than 10 lines of code (down from 50+ with the current builder pattern).
- **Zero-disruption upgrade:** All v0.5.0 code compiles and runs on v0.6.0 without modification — zero breaking changes.
- **Discovery clarity:** Developer can find the right API (`AgentFactory` vs `AgentPool` vs `Agent::with_system()`) from the README alone.
- **Team from roles:** `AgentPool::from_team(&team, provider)` works correctly with all existing `Process` types.

### Business Success

- **crates.io growth:** 20% increase in monthly downloads within 2 months of v0.6.0 release.
- **Community adoption:** At least 3 community-contributed examples or blog posts using `AgentFactory`.
- **Competitive positioning:** TraitClaw cited as having the most ergonomic multi-agent API among Rust AI frameworks.

### Technical Success

- **Zero overhead:** `AgentFactory::spawn()` overhead < 1μs vs direct builder (thin wrapper, not a runtime abstraction).
- **Backward compat:** All 20 existing examples compile and run unchanged on v0.6.0.
- **No new required deps:** Implemented purely in `traitclaw-core` and `traitclaw-team`.
- **MSRV:** Rust 1.75+ maintained.

### Measurable Outcomes

| Metric | Target | Measurement |
|--------|--------|-------------|
| Lines to create 5-agent team | ≤ 10 | Example comparison |
| Compile time increase | < 2% | `cargo build --timings` |
| Breaking changes | 0 | v0.5.0 example suite |
| New public API surface | ≤ 5 new types/methods | Manual count |

---

## Product Scope

### MVP — v0.6.0 "Composition"

1. **`Agent::with_system(provider, system_prompt)`** — Single-line agent creation shorthand
2. **`AgentFactory`** — Shared-provider factory with `spawn(system_prompt)` method
3. **`AgentPool::from_team(team, provider)`** — Batch-spawn all agents from a `Team` definition
4. **`RoundRobinGroupChat`** — Array-of-agents team preset (AutoGen-inspired)
5. **Example `24-agent-factory/`** — Demonstrates all three APIs in one runnable example
6. **Migration guide** — `docs/migration-v0.5-to-v0.6.md`

### Growth Features (Post-MVP — v0.7.0)

- `ReActStrategy` and `MctsStrategy` built-in implementations
- `StreamingOutputTransformer` trait
- Graph-based routing (DAG execution engine)
- Config-driven agent spawn from YAML/TOML

### Vision (v1.0)

- Visual workflow builder for agent orchestration
- Cloud-native agent fleet deployment
- Marketplace for community Strategies, Hooks, Tools

---

## User Journeys

### Journey 1: "I need a content pipeline with 3 specialized agents"

**Persona:** Linh, a backend developer building an automated content system.

Linh is maintaining a content generation service where a single agent keeps hallucinating facts and producing inconsistent tone. She knows she needs specialized agents — a researcher, a writer, and a reviewer — but every attempt with TraitClaw required 10 nearly-identical lines of builder boilerplate per agent.

She finds TraitClaw v0.6.0's release notes. In 3 minutes:

```rust
let factory = AgentFactory::new(provider);
let researcher = factory.spawn("You are a thorough researcher.");
let writer     = factory.spawn("You are a precise technical writer.");
let reviewer   = factory.spawn("You are a quality reviewer.");
let pool = AgentPool::new(vec![researcher, writer, reviewer]);
let result = pool.run_sequential("Write about Rust async").await?;
```

"This is exactly what I needed" — 7 lines instead of 30+, same type safety.

### Journey 2: "I'm upgrading from v0.5.0 and don't want to break anything"

**Persona:** Minh, a developer maintaining a production agent deployed for a client.

Minh sees the v0.6.0 announcement. His production code uses the v0.5.0 builder API extensively. He bumps the version in `Cargo.toml`, runs `cargo build` — ✅ Finished, zero errors. He then optionally adopts `Agent::with_system()` in new code while legacy code stays untouched. Smooth upgrade, earned trust.

### Journey 3: "I already have AgentRole definitions and want to spawn the team"

**Persona:** Duc, building a multi-agent code review pipeline.

Duc already has a `Team` with `AgentRole` definitions for `"linter"`, `"reviewer"`, and `"security_auditor"`. Previously he had to manually bind each role to an agent instance. With v0.6.0:

```rust
let pool = AgentPool::from_team(&team, provider)?;
pool.run("Review this PR diff").await?;
```

All roles automatically mapped to agents — the bridge between the declarative `Team` model and runtime execution that was missing.

### Journey 4: "I want to write a blog comparing TraitClaw to CrewAI"

**Persona:** Tran, a technical content creator writing a Rust AI framework comparison.

Tran forks the repo, runs `cargo run -p agent-factory`. The example spins up a 3-agent team, runs a task, and prints results — all in under 10 lines of Rust. She writes: *"TraitClaw v0.6 closes the ergonomics gap with Python frameworks while keeping Rust's compile-time safety."* Published — drives community adoption.

### Journey Requirements Summary

| Journey | Capabilities Required |
|---------|----------------------|
| Content pipeline | `AgentFactory`, `AgentPool::run_sequential()`, `Agent::with_system()` |
| Zero-disruption upgrade | 100% backward compat, no forced migration |
| Team from roles | `AgentPool::from_team()`, `Team`→`Agent` binding |
| Blog/demo | `24-agent-factory` example, clean runnable output |

---

## Developer Tool Specific Requirements

### API Surface

The new APIs must follow TraitClaw's existing ergonomic conventions:

```rust
// Tier 1: Single agent shorthand
let agent = Agent::with_system(provider, "You are a researcher.");

// Tier 2: Factory for same-provider multiple agents
let factory = AgentFactory::new(provider);
let agent_a = factory.spawn("You are a researcher.");
let agent_b = factory.spawn("You are a writer.");

// Tier 3: Pool from existing Team definition
let pool = AgentPool::from_team(&team, provider)?;
```

### Language & Ecosystem Support

- **Language:** Rust 1.75+ (stable only, no nightly features)
- **Async runtime:** `tokio` (existing dependency, no changes)
- **Re-exports:** All new types available via `traitclaw::prelude::*`
- **Feature flags:** No new feature flags for core composition APIs

### Installation & Distribution

- Added to existing `traitclaw` meta-crate — no new crate to add to `Cargo.toml`
- `AgentFactory` and `AgentPool` in `traitclaw-core`
- `RoundRobinGroupChat` in `traitclaw-team`

### Documentation Requirements

- Rustdoc with examples for every new public type and method
- `docs/migration-v0.5-to-v0.6.md` migration guide
- Example `24-agent-factory/` runnable with `cargo run -p agent-factory`
- README updated with v0.6.0 "Quick Start: Multi-Agent" section

### Code Examples (Required)

```rust
// Minimum viable example — must fit in README code block
let factory = AgentFactory::new(provider);
let agents = vec![
    factory.spawn("You are a researcher."),
    factory.spawn("You are a writer."),
];
let team = RoundRobinGroupChat::new(agents);
let result = team.run("Write about async Rust").await?;
```

---

## Feature Requirements

### F1: `Agent::with_system()`

**Priority:** P0 | **Crate:** `traitclaw-core`

```rust
impl Agent {
    pub fn with_system(
        provider: impl Provider + 'static,
        system: impl Into<String>,
    ) -> Self;
}
```

| Requirement | Detail |
|-------------|--------|
| Behavior | Equivalent to `Agent::builder().provider(p).system(s).build()?` |
| Error handling | Infallible — no configuration that can fail at this scope |
| Backward compat | Purely additive — existing `Agent::builder()` unchanged |
| Re-export | Available via `traitclaw::prelude::*` |

---

### F2: `AgentFactory`

**Priority:** P0 | **Crate:** `traitclaw-core`

```rust
pub struct AgentFactory<P: Provider + Clone> {
    provider: P,
    default_model: Option<String>,
}

impl<P: Provider + Clone> AgentFactory<P> {
    pub fn new(provider: P) -> Self;
    pub fn spawn(&self, system: impl Into<String>) -> Agent;
    pub fn spawn_with<F>(&self, f: F) -> Agent
    where F: FnOnce(AgentBuilder) -> AgentBuilder;
}
```

| Requirement | Detail |
|-------------|--------|
| Provider cloning | `Provider: Clone` bound — factory clones provider per agent |
| `spawn()` | Creates agent with system prompt, uses factory's provider |
| `spawn_with()` | Escape hatch for full builder customization |
| Thread safety | `AgentFactory: Send + Sync` if `P: Send + Sync` |
| Overhead | `spawn()` = one `provider.clone()` + builder call. No allocation beyond that. |

---

### F3: `AgentPool`

**Priority:** P0 | **Crate:** `traitclaw-core`

```rust
pub struct AgentPool {
    agents: Vec<Agent>,
}

impl AgentPool {
    pub fn new(agents: Vec<Agent>) -> Self;
    pub fn from_team<P>(team: &Team, provider: P) -> Result<Self>
    where P: Provider + Clone + 'static;
    pub async fn run_sequential(&self, input: &str) -> Result<AgentOutput>;
    pub fn get(&self, index: usize) -> Option<&Agent>;
    pub fn len(&self) -> usize;
}
```

| Requirement | Detail |
|-------------|--------|
| `from_team()` | Maps each `AgentRole` → `Agent` using role's system_prompt |
| `run_sequential()` | Runs agents in order, each receives previous output |
| Owned agents | `AgentPool` takes ownership of `Vec<Agent>` |
| Error | Returns `Err` if team has roles with no system_prompt set |

---

### F4: `RoundRobinGroupChat`

**Priority:** P1 | **Crate:** `traitclaw-team`

```rust
pub struct RoundRobinGroupChat {
    agents: Vec<Agent>,
    max_rounds: usize,
    termination: Box<dyn TerminationCondition>,
}

impl RoundRobinGroupChat {
    pub fn new(agents: Vec<Agent>) -> Self;
    pub fn with_max_rounds(mut self, n: usize) -> Self;
    pub async fn run(&mut self, task: &str) -> Result<GroupChatResult>;
}
```

| Requirement | Detail |
|-------------|--------|
| Round-robin | Agents take turns in order, each sees full conversation history |
| Termination | Default: `MaxRoundsTermination(agents.len() * 3)` |
| Broadcast | Each agent response added to shared `Vec<Message>` |
| Output | `GroupChatResult` contains full transcript + final message |

---

### F5: Example `24-agent-factory/`

**Priority:** P0 | **Crate:** new example

Demonstrates all three composition APIs in one progressive example:
1. `Agent::with_system()` — simplest case
2. `AgentFactory::spawn()` — same-provider batch
3. `AgentPool::from_team()` — from existing Team definition
4. `RoundRobinGroupChat` — multi-turn collaboration

---

## Non-Functional Requirements

### Performance

- `AgentFactory::spawn()` overhead: zero heap allocation beyond `provider.clone()` — verified by benchmark.
- `AgentPool::from_team()`: O(n) where n = number of roles. No async work at construction time.
- `RoundRobinGroupChat`: conversation history stored as `Vec<Message>` — same as existing agent conversation tracking.

### Compatibility

- **Backward compatibility:** v0.5.0 code compiles and runs on v0.6.0 without modification. Verified by running all 20 existing examples in CI.
- **MSRV:** Rust 1.75+ maintained (existing requirement, no change).
- **Semver:** Minor version bump (0.5.0 → 0.6.0). No breaking changes.
- **New traits:** None. New types only — `AgentFactory`, `AgentPool`, `RoundRobinGroupChat`.

### Documentation

- All new public types and methods: rustdoc with at least one `# Example` block.
- Migration guide: `docs/migration-v0.5-to-v0.6.md`.
- README: "Multi-Agent Quickstart" section added showing the 3-tier API.

### Testing

- Unit tests for `AgentFactory::spawn()` using mock provider
- Unit tests for `AgentPool::from_team()` — verifies role→agent mapping
- Integration test for `RoundRobinGroupChat` with 2 mock agents, 2 rounds
- Backward-compat test: all v0.5.0 examples compile in CI

---

## Technical Constraints

### Architecture Decisions

| ADR | Decision | Rationale |
|-----|----------|-----------|
| ADR-18 | `Provider: Clone` bound on `AgentFactory` | Factory must give each agent its own provider instance. Clone is the simplest contract — Arc<dyn Provider> already impls Clone. |
| ADR-19 | `AgentPool` takes `Vec<Agent>` (owned) | Consistent with existing `Team::bind()` ownership model. Prevents shared mutable state. |
| ADR-20 | `RoundRobinGroupChat` in `traitclaw-team` (not `traitclaw-core`) | Multi-agent coordination is a team concern. Core stays minimal. |
| ADR-21 | No `AgentFactory` trait, only concrete struct | Factory behavior doesn't need polymorphism. YAGNI — add trait later if real use case emerges. |

### Dependency Policy

- **No new dependencies for any of the four features.**
- `AgentFactory` and `AgentPool`: pure Rust, no new crates.
- `RoundRobinGroupChat`: uses existing `traitclaw-team` internals.

### Crate Impact Map

| Crate | Changes |
|-------|---------|
| `traitclaw-core` | Add `Agent::with_system()`, `AgentFactory`, `AgentPool` |
| `traitclaw-team` | Add `RoundRobinGroupChat`, `GroupChatResult`, `TerminationCondition` trait |
| `traitclaw` (meta) | Re-export all new types in `prelude` |
| New examples | `examples/24-agent-factory/` |
| New docs | `docs/migration-v0.5-to-v0.6.md` |

---

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| `Provider: Clone` not implemented by user's custom provider | Medium | High | Document requirement clearly; provide `Arc<dyn Provider>` which blanket-impls Clone. Add compile error message guiding users. |
| `AgentPool::from_team()` fails silently when roles have no system_prompt | Medium | Medium | Return `Result<AgentPool>` with clear error message listing which roles are missing system_prompt. |
| Naming conflicts with future official TraitClaw types | Low | Medium | Use conservative names: prefer `AgentPool` over `AgentTeam` to avoid collision with existing `Team` struct. |
| Scope creep: adding streaming, config-driven spawn to v0.6.0 | High | Medium | Hard cutoff: PRD lists exactly 4 features for MVP. Anything else is v0.7.0. |
| Community misuse: using `RoundRobinGroupChat` with too many rounds | Low | Low | Default max_rounds = n_agents × 3. Document recommended values. |

---

## Release Plan

### Phase 1: Core API (Week 1)

- [ ] `Agent::with_system()` in `traitclaw-core`
- [ ] `AgentFactory` struct with `spawn()` and `spawn_with()`
- [ ] Unit tests for both
- [ ] `traitclaw::prelude::*` re-exports updated

### Phase 2: Pool & GroupChat (Week 2)

- [ ] `AgentPool` with `new()`, `from_team()`, `run_sequential()`
- [ ] `TerminationCondition` trait in `traitclaw-team`
- [ ] `RoundRobinGroupChat` with `new()`, `with_max_rounds()`, `run()`
- [ ] Integration tests

### Phase 3: Example & Docs (Week 3)

- [ ] `examples/24-agent-factory/` (progressive 4-part demo)
- [ ] `docs/migration-v0.5-to-v0.6.md`
- [ ] README "Multi-Agent Quickstart" section
- [ ] Backward-compat CI check: all 20 v0.5.0 examples

### Phase 4: Release (Week 4)

- [ ] Version bump all crates to 0.6.0
- [ ] `cargo publish` dry-run check
- [ ] Tag `v0.6.0` in git
- [ ] Announce on r/rust, Rust AI communities
