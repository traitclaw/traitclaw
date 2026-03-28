---
stepsCompleted: ["step-01-init", "step-02-context", "step-03-starter", "step-04-decisions", "step-05-patterns", "step-06-structure", "step-07-validation", "step-08-complete"]
inputDocuments: ["prd.md", "project-context.md", "product-brief-traitclaw-2026-03-26.md"]
workflowType: 'architecture'
project_name: 'traitclaw'
user_name: 'Bangvu'
date: '2026-03-28'
status: 'complete'
completedAt: '2026-03-28'
---

# Architecture Decision Document

_This document builds collaboratively through step-by-step discovery. Sections are appended as we work through each architectural decision together._

## Project Context Analysis

### Requirements Overview

**Functional Requirements (27 FRs across 7 capability areas):**

- **Reasoning Strategy Core (FR1-7):** Three strategy structs (`ReActStrategy`, `MctsStrategy`, `ChainOfThoughtStrategy`), each implementing existing `AgentStrategy` trait
- **Strategy Interchangeability (FR8-10):** Trait-based polymorphism enabling zero-change strategy swap
- **Streaming Output (FR11-13):** New `StreamingOutputTransformer` trait in `traitclaw-core`
- **Crate & Feature Management (FR14-17):** New `traitclaw-strategies` crate with per-strategy feature flags, meta-crate re-export
- **Thought Step Observability (FR18-20):** `ThoughtStep` enum with typed events, post-execution inspection
- **Backward Compatibility (FR21-23):** Zero breaking changes — all v0.6.0 APIs unchanged
- **Documentation (FR24-27):** Per-strategy examples, migration guide, complete rustdoc

**Non-Functional Requirements (12 NFRs):**

- NFR1-6 (Performance): Compile time < 2% increase, binary < 5% increase, runtime < 1ms/cycle, MCTS spawn < 100μs, streaming latency < 10ms
- NFR7-12 (Integration): MSRV 1.75, tokio-only async, zero new deps, builder pattern, existing error types, trait object safety

### Scale & Complexity

- **Complexity level:** Medium-High
- **Primary domain:** Library / Framework (Rust crate)
- **Estimated architectural components:** 5 (strategies crate scaffold, ReAct module, MCTS module, CoT module, streaming transformer in core)

### Technical Constraints & Dependencies

1. `traitclaw-strategies` depends on `traitclaw-core` — no circular dependencies
2. All strategies must implement `AgentStrategy` trait unchanged from v0.2.0
3. `StreamingOutputTransformer` is additive — must not modify existing `OutputTransformer` chain
4. Feature flags: `react`, `mcts`, `cot` — default all-on, independently compilable
5. MCTS parallelism uses existing `tokio::spawn` — no new async runtime

### Cross-Cutting Concerns

| Concern | Impact |
|---------|--------|
| Error handling | All strategies use existing `TraitClawError` — no new error types (NFR11) |
| Async patterns | All strategies async, using `tokio` runtime already in workspace |
| Testing | Mock providers for offline deterministic tests, `cargo test` compatible |
| Feature flags | Each strategy gated independently; meta-crate re-export via `features = ["strategies"]` |
| Object safety | All strategies must be `dyn AgentStrategy` compatible for dynamic dispatch |

## Starter Template Evaluation

### Primary Technology Domain

Rust Library / Framework (Cargo workspace) — brownfield project with 14 existing crates

### Starter Options

N/A — brownfield project. No new project scaffolding needed. `traitclaw-strategies` follows identical patterns to existing crates (`traitclaw-steering`, `traitclaw-rag`, etc.).

### Existing Architectural Foundation

**Language & Runtime:**
- Rust edition 2021, MSRV 1.75+
- Async via `tokio 1.x` (features: full)

**Workspace Structure:**
```
crates/
├── traitclaw/              # meta-crate (re-exports + feature flags)
├── traitclaw-core/         # core traits, types, runtime
├── traitclaw-macros/       # proc macros
├── traitclaw-strategies/   # NEW in v0.7.0
├── traitclaw-openai/       # OpenAI provider
├── traitclaw-anthropic/    # Anthropic provider
├── traitclaw-ollama/       # Ollama provider
├── traitclaw-team/         # multi-agent orchestration
├── traitclaw-workflow/     # graph-based workflows
└── ... (5+ more crates)
```

**Build Tooling:** `cargo` with workspace-level `Cargo.toml` and shared dependency versions

**Testing:** `cargo test`, `tokio::test` for async, `mockall` for mocks

**Code Organization:** Module-per-crate, trait-based abstractions, feature-gated optional functionality

**v0.7.0 Addition:** New `traitclaw-strategies` crate. First implementation story should scaffold the crate with `Cargo.toml`, `lib.rs`, feature flags, and workspace registration.

## Core Architectural Decisions

### Decision Priority Analysis

**Critical Decisions (Block Implementation):**
1. Strategy module structure within `traitclaw-strategies`
2. `ThoughtStep` type design (shared across all strategies)
3. `StreamingOutputTransformer` trait surface (lives in `traitclaw-core`)

**Important Decisions (Shape Architecture):**
4. MCTS branch execution pattern
5. Feature flag naming and gating approach

**Deferred Decisions (Post-MVP):**
- DAG execution engine internals (v0.8.0)
- Config-driven agent spawning format (v0.8.0)
- `OrchestrationStrategy` trait design (v0.8.0)

### Decision 1: Strategy Module Structure

```
traitclaw-strategies/
├── Cargo.toml              # feature flags: react, mcts, cot
├── src/
│   ├── lib.rs              # public re-exports, feature gates
│   ├── common/
│   │   ├── mod.rs
│   │   └── thought_step.rs # ThoughtStep enum (shared)
│   ├── react/
│   │   ├── mod.rs
│   │   └── strategy.rs     # ReActStrategy impl
│   ├── mcts/
│   │   ├── mod.rs
│   │   ├── strategy.rs     # MctsStrategy impl
│   │   ├── config.rs       # MctsConfig
│   │   └── tree.rs         # Tree search internals
│   └── cot/
│       ├── mod.rs
│       └── strategy.rs     # ChainOfThoughtStrategy impl
```

**Rationale:** One module per strategy. `common/` holds shared types (`ThoughtStep`). Feature flags gate at module level in `lib.rs`.

### Decision 2: ThoughtStep Type Design

**Decision:** Enum (not trait)

```rust
#[derive(Debug, Clone, Serialize)]
pub enum ThoughtStep {
    Think { content: String },
    Act { tool_name: String, tool_input: serde_json::Value },
    Observe { tool_output: String },
    Answer { content: String },
}
```

**Rationale:**
- Exhaustive pattern matching ensures all step types handled
- Serializable for logging/streaming
- No need for extensibility — step types are fixed per strategy
- Avoids `dyn Trait` allocation overhead

### Decision 3: MCTS Branch Execution

**Decision:** `tokio::spawn` per branch, `JoinSet` to collect results

```rust
pub struct MctsConfig {
    pub branches: usize,        // default: 5
    pub max_depth: usize,       // default: 3
    pub scoring: ScoringFn,     // default: LLM self-score
}

type ScoringFn = Arc<dyn Fn(&str) -> f64 + Send + Sync>;
```

**Rationale:** `JoinSet` provides structured concurrency with automatic cleanup. Default scoring = LLM self-evaluation. Custom `ScoringFn` via `Arc<dyn Fn>` for user extensibility without trait overhead.

### Decision 4: StreamingOutputTransformer Trait

**Decision:** Separate trait from `OutputTransformer` (additive, never modifies existing chain)

```rust
#[async_trait]
pub trait StreamingOutputTransformer: Send + Sync {
    async fn transform_chunk(&self, chunk: &str) -> Result<String>;
    async fn on_thought_step(&self, step: &ThoughtStep) -> Result<()> {
        Ok(()) // default no-op
    }
}
```

**Rationale:**
- `transform_chunk` (required): per-token/chunk transformation
- `on_thought_step` (optional): hook for thought step events
- Lives in `traitclaw-core` alongside existing `OutputTransformer`
- Default no-op on `on_thought_step` minimizes adoption friction

### Decision 5: Feature Flag Design

**Decision:** Additive flags, no inter-flag dependencies

```toml
# traitclaw-strategies/Cargo.toml
[features]
default = ["react", "mcts", "cot"]
react = []
mcts = []
cot = []
```

```toml
# traitclaw/Cargo.toml (meta-crate)
[features]
strategies = ["dep:traitclaw-strategies"]
```

**Rationale:** `common` module always compiled (shared types). Meta-crate gates entire strategies crate, individual strategy selection via `traitclaw-strategies` feature flags.

### Decision Impact Analysis

**Implementation Sequence:**
1. Scaffold crate (Cargo.toml, lib.rs, common/) → enables all other work
2. `ThoughtStep` enum in common/ → shared dependency for all strategies
3. `StreamingOutputTransformer` trait in core → enables streaming stories
4. `ReActStrategy` → highest value, simplest strategy
5. `ChainOfThoughtStrategy` → similar pattern to ReAct
6. `MctsStrategy` → most complex, depends on ThoughtStep + async patterns

**Cross-Component Dependencies:**
- `traitclaw-strategies` → depends on `traitclaw-core` (for `AgentStrategy`, `AgentRuntime`)
- `traitclaw` meta-crate → optional dep on `traitclaw-strategies`
- All strategies → depend on `common::ThoughtStep`
- MCTS → additionally depends on `tokio::task::JoinSet`

## Implementation Patterns & Consistency Rules

### Naming Patterns

| Element | Convention | Example |
|---------|-----------|--------|
| Crate names | kebab-case | `traitclaw-strategies` |
| Module names | snake_case | `thought_step.rs` |
| Structs | PascalCase | `ReActStrategy`, `MctsConfig` |
| Traits | PascalCase | `StreamingOutputTransformer` |
| Functions | snake_case | `transform_chunk`, `on_thought_step` |
| Feature flags | lowercase, short | `react`, `mcts`, `cot` |
| Constants | SCREAMING_SNAKE | `DEFAULT_MAX_DEPTH` |

### Structure Patterns

**Module Organization:**
- `mod.rs` = public re-exports only, no logic
- `strategy.rs` = `impl AgentStrategy for XxxStrategy`
- `config.rs` = builder pattern for strategy configuration
- Tests: inline `#[cfg(test)] mod tests` in same file (co-located)
- Integration tests: `tests/` directory at crate root

**Public API Surface:**
- `lib.rs` gates modules via `#[cfg(feature = "xxx")]`
- All public types re-exported from `lib.rs`
- Internal types marked `pub(crate)` — never `pub`

### Error Handling Patterns

All strategies use existing `TraitClawError`. New error enums are forbidden. Map internal errors via `?` + `From` impls. Strategy-specific context via `TraitClawError::Strategy(String)` variant.

### Builder Pattern

```rust
let strategy = ReActStrategy::builder()
    .max_iterations(10)
    .system_prompt("You are a ReAct agent")
    .build()?;
```

- Builder methods return `&mut Self`
- `.build()` returns `Result<Strategy, TraitClawError>`
- Required fields: compile-time enforcement via typestate OR runtime validation

### Testing Patterns

- Test names: descriptive, snake_case, describe behavior (`react_strategy_completes_think_act_observe_cycle`)
- Mock LLM providers for deterministic offline tests
- Assert on `ThoughtStep` sequence, not internal state
- `#[tokio::test]` for all async tests

### Documentation Patterns

- All public items: `///` doc comments with `# Examples` section
- All modules: `//!` module-level docs
- Examples must compile (`cargo test --doc`)

### Anti-Patterns

| ❌ Don't | ✅ Do |
|----------|------|
| `pub` for internal types | `pub(crate)` |
| New error types | Use `TraitClawError` |
| `Box<dyn Error>` | `TraitClawError` with `From` impls |
| `unwrap()` / `expect()` in library code | `?` propagation |
| Hardcoded strategy params | Builder pattern with defaults |
| `#[allow(dead_code)]` | Remove unused code |

## Project Structure & Boundaries

### v0.7.0 Project Structure Changes

```
traitclaw/
├── Cargo.toml                          # add traitclaw-strategies dep
├── crates/
│   ├── traitclaw/Cargo.toml           # add features = ["strategies"]
│   ├── traitclaw-core/
│   │   └── src/
│   │       ├── streaming.rs           # NEW: StreamingOutputTransformer trait
│   │       └── lib.rs                 # re-export StreamingOutputTransformer
│   └── traitclaw-strategies/          # NEW CRATE
│       ├── Cargo.toml                 # features: react, mcts, cot
│       ├── src/
│       │   ├── lib.rs                 # feature-gated re-exports
│       │   ├── common/
│       │   │   ├── mod.rs
│       │   │   └── thought_step.rs    # ThoughtStep enum
│       │   ├── react/
│       │   │   ├── mod.rs
│       │   │   └── strategy.rs        # ReActStrategy
│       │   ├── mcts/
│       │   │   ├── mod.rs
│       │   │   ├── strategy.rs        # MctsStrategy
│       │   │   ├── config.rs          # MctsConfig
│       │   │   └── tree.rs            # Tree search
│       │   └── cot/
│       │       ├── mod.rs
│       │       └── strategy.rs        # ChainOfThoughtStrategy
│       └── tests/
│           ├── react_integration.rs
│           ├── mcts_integration.rs
│           └── cot_integration.rs
├── examples/
│   ├── 11-custom-strategy/            # UPGRADE: add strategy comparison
│   ├── XX-react-strategy/             # NEW
│   ├── XX-mcts-strategy/              # NEW
│   └── XX-cot-strategy/               # NEW
└── docs/
    └── migration-v0.6-to-v0.7.md      # NEW
```

### FR-to-Module Mapping

| FR Group | Module Location |
|----------|----------------|
| FR1-3 (ReAct) | `crates/traitclaw-strategies/src/react/` |
| FR4-5 (MCTS) | `crates/traitclaw-strategies/src/mcts/` |
| FR6-7 (CoT) | `crates/traitclaw-strategies/src/cot/` |
| FR8-10 (Interchangeability) | All strategy `impl AgentStrategy` blocks |
| FR11-13 (Streaming) | `crates/traitclaw-core/src/streaming.rs` |
| FR14-17 (Crate/Features) | `Cargo.toml` files + `lib.rs` |
| FR18-20 (Observability) | `crates/traitclaw-strategies/src/common/thought_step.rs` |
| FR21-23 (Backward Compat) | Verified via existing test suite |
| FR24-27 (Documentation) | `examples/`, `docs/`, inline rustdoc |

### Architectural Boundaries

**Crate Boundaries (uni-directional):**
- `traitclaw-core` → owns traits (`AgentStrategy`, `StreamingOutputTransformer`), runtime, types
- `traitclaw-strategies` → owns strategy implementations, depends on core only
- `traitclaw` (meta) → re-exports both via feature flags

**Data Flow:**
```
User Code → AgentBuilder → Agent(strategy: Box<dyn AgentStrategy>) → AgentRuntime
                                    ↓
                          ReActStrategy / MctsStrategy / CoTStrategy
                                    ↓
                          AgentRuntime.provider.chat() → ThoughtStep events
                                    ↓
                          StreamingOutputTransformer.transform_chunk()
```

## Architecture Validation Results

### Coherence Validation ✅

**Decision Compatibility:** All 5 decisions are mutually compatible. Module structure supports feature flags. `ThoughtStep` enum is shared by all strategies and used by `StreamingOutputTransformer`. No circular dependencies in crate graph.

**Pattern Consistency:** Naming, testing, error handling, and builder patterns align with `project-context.md` rules. No conflicts.

**Structure Alignment:** File tree maps 1:1 to decisions. Each directory has clear ownership.

### Requirements Coverage ✅

| Area | Coverage | Notes |
|------|----------|-------|
| FR1-7 (Strategy Core) | ✅ 100% | Each strategy has dedicated module + `impl AgentStrategy` |
| FR8-10 (Interchangeability) | ✅ 100% | Trait-based polymorphism via `Box<dyn AgentStrategy>` |
| FR11-13 (Streaming) | ✅ 100% | `StreamingOutputTransformer` in `traitclaw-core` |
| FR14-17 (Crate/Features) | ✅ 100% | Feature flag design in Decision 5 |
| FR18-20 (Observability) | ✅ 100% | `ThoughtStep` enum in Decision 2 |
| FR21-23 (Backward Compat) | ✅ 100% | Existing `AgentStrategy` unchanged |
| FR24-27 (Documentation) | ✅ 100% | Examples, migration guide, rustdoc in file tree |
| NFR1-12 | ✅ 100% | All performance and integration NFRs addressed |

### Gap Analysis

**Critical Gaps:** None ✅

**Minor Gaps (non-blocking):**
1. `ThoughtStep` may benefit from a `timestamp` field — defer to implementation
2. MCTS `ScoringFn` may need `async` support if scoring involves LLM calls — can add later
3. Example numbering needs assignment based on existing examples

### Architecture Completeness Checklist

- [x] Project context analyzed
- [x] Scale and complexity assessed
- [x] Technical constraints identified
- [x] Cross-cutting concerns mapped
- [x] Critical decisions documented with code examples
- [x] Technology stack specified
- [x] Implementation patterns defined
- [x] Project structure with FR mapping complete
- [x] Crate boundaries and data flow documented
- [x] Validation passed — zero critical gaps

### Architecture Readiness Assessment

**Overall Status:** ✅ READY FOR IMPLEMENTATION

**Confidence Level:** High

**Key Strengths:**
- Zero new traits needed — strategies use existing `AgentStrategy`
- Uni-directional crate dependency graph
- All patterns match existing codebase conventions
- Feature flags enable granular compilation

**Areas for Future Enhancement:**
- MCTS async scoring function (v0.7.1)
- Strategy composition patterns (v0.8.0)
- Telemetry hooks into ThoughtStep events (v0.9.0+)

### Implementation Handoff

**AI Agent Guidelines:**
- Follow all architectural decisions exactly as documented
- Use implementation patterns consistently across all components
- Respect crate boundaries (uni-directional dependencies only)
- Refer to this document for all architectural questions

**First Implementation Priority:** Scaffold `traitclaw-strategies` crate with `Cargo.toml`, `lib.rs`, `common/thought_step.rs`, feature flags, and workspace registration.
