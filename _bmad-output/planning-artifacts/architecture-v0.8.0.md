---
stepsCompleted: ["step-01-init", "step-02-context", "step-03-starter", "step-04-decisions", "step-05-patterns", "step-06-structure", "step-07-validation"]
inputDocuments: ["prd-v0.8.0.md", "architecture.md", "epics-v0.8.0.md", "project-context.md"]
workflowType: 'architecture'
project_name: 'traitclaw'
user_name: 'Bangvu'
date: '2026-03-28'
---

# Architecture Decision Document — v0.8.0 "Quality Foundation"

_This document describes the architectural decisions for TraitClaw v0.8.0, focused on testing infrastructure, CI/CD automation, and runtime observability. It supplements (not replaces) the v0.7.0 architecture document._

## Project Context Analysis

### Requirements Overview

**Functional Requirements (28 FRs across 7 capability areas):**

- **Shared Test Infrastructure (FR1-5, FR24):** New `traitclaw-test-utils` crate with `MockProvider`, `MockMemory`, `EchoTool`, `FailTool`, `make_runtime()` helper, comprehensive doc comments
- **CI Pipeline (FR6-11):** GitHub Actions workflow for fmt, clippy, test, doc, plus `cargo-llvm-cov` coverage reports
- **Tracing Instrumentation (FR12-15):** Structured tracing spans on LLM calls, tool executions, guard checks, with component-level filtering
- **Event System (FR16-19):** `AgentEvent` enum, `on_event()` callback on `AgentBuilder`, token usage in events, custom workflow support
- **Cost Tracking (FR20-22):** `RunUsage` cost estimation with configurable model pricing table
- **Documentation (FR23-25):** Migration guide, test-utils rustdoc, observability example
- **Backward Compatibility (FR26-28):** Zero breaking changes, `#[non_exhaustive]` on AgentEvent

**Non-Functional Requirements (10 NFRs):**

- NFR1-3 (Performance): Zero tracing overhead without subscriber, ≤ 1μs event callback, negligible cost calculation
- NFR4-5 (Build): CI ≤ 10 mins, test-utils zero compile cost for non-test builds
- NFR6-10 (Integration): Any tracing subscriber, standard Fn signatures, tokio::test compatible, stable Rust only, Debug+Clone+Send+Sync on all new types

### Scale & Complexity

- **Complexity level:** Medium
- **Primary domain:** Library / Framework (Rust crate) — infrastructure-only release
- **Estimated architectural components:** 4 (test-utils crate, CI config, tracing instrumentation, event/cost system)

### Technical Constraints & Dependencies

1. `traitclaw-test-utils` is a `dev-dependency` only — must not pollute user dependency trees
2. All new types in `traitclaw-core` — no new crates except test-utils
3. `tracing` already in workspace dependencies — no new runtime deps
4. `AgentEvent` must be `#[non_exhaustive]` for forward compatibility
5. Zero modifications to existing `AgentStrategy`, `Provider`, `Tool`, `Memory` trait signatures
6. CI uses GitHub Actions with stable Rust only (no nightly)

### Cross-Cutting Concerns

| Concern | Impact |
|---------|--------|
| Zero-cost abstractions | Tracing spans compiled to no-ops without subscriber. Event callbacks are `Option<Arc<dyn Fn>>` — single branch check per event |
| Backward compatibility | All v0.7.0 public APIs unchanged. New types/methods are purely additive |
| Testing consistency | Shared mocks eliminate 6 duplicate MockProvider definitions across workspace |
| Feature flags | No new feature flags needed — tracing is always available, events are opt-in via builder |
| Error handling | No new error types added. Cost tracking returns `0.0` for unknown models (with tracing warning) |

## Starter Template Evaluation

### Primary Technology Domain

Rust Library / Framework (Cargo workspace) — brownfield project with 14+ existing crates.

### Starter Options

N/A — brownfield project. No new project scaffolding needed. `traitclaw-test-utils` follows identical patterns to existing crates (`traitclaw-steering`, `traitclaw-rag`, etc.).

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
├── traitclaw-strategies/   # v0.7.0 reasoning strategies
├── traitclaw-steering/     # Guard-Hint-Track
├── traitclaw-openai/       # OpenAI provider
├── traitclaw-anthropic/    # Anthropic provider
├── traitclaw-test-utils/   # NEW in v0.8.0 (dev-dependency only)
└── ... (5+ more crates)
```

**Build Tooling:** `cargo` with workspace-level `Cargo.toml` and shared dependency versions

**Testing:** `cargo test`, `tokio::test` for async, manual mocks (no mockall in practice)

**v0.8.0 Addition:** New `traitclaw-test-utils` crate as dev-dependency. New types in `traitclaw-core`. CI configuration at `.github/workflows/ci.yml`.

## Core Architectural Decisions

### Decision Priority Analysis

**Critical Decisions (Block Implementation):**
1. `AgentEvent` type design and placement
2. Event callback mechanism in `AgentBuilder` / `Agent`
3. `traitclaw-test-utils` dependency graph

**Important Decisions (Shape Architecture):**
4. Tracing span naming convention
5. `RunUsage` cost estimation approach
6. CI pipeline job structure

**Deferred Decisions (v0.9.0+):**
- Distributed tracing correlation IDs
- OpenTelemetry export integration
- Cost alerting / budgets

### Decision 1: AgentEvent Enum Design

**Decision:** Enum with `#[non_exhaustive]` (not trait)

```rust
// New file: traitclaw-core/src/types/event.rs

/// Lifecycle events emitted by the agent runtime.
///
/// Use with [`AgentBuilder::on_event()`] to observe agent behavior
/// for logging, metrics, debugging, or cost tracking.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum AgentEvent {
    /// LLM completion request started.
    LlmStart { model: String },
    /// LLM completion request completed.
    LlmEnd {
        model: String,
        prompt_tokens: u32,
        completion_tokens: u32,
        duration_ms: u64,
    },
    /// Tool invocation started.
    ToolCall {
        tool_name: String,
        args: serde_json::Value,
    },
    /// Tool invocation completed.
    ToolResult {
        tool_name: String,
        success: bool,
        duration_ms: u64,
    },
    /// Guard blocked an action.
    GuardBlock {
        guard_name: String,
        reason: String,
    },
    /// Hint was triggered.
    HintTriggered { hint_name: String },
}
```

**Rationale:**
- Enum pattern matches `ThoughtStep` from v0.7.0 — consistent codebase style
- `#[non_exhaustive]` allows adding `StrategyStep`, `MemoryRecall` etc. in v0.9.0+ without breaking changes
- `Debug + Clone` satisfies NFR10 — `Send + Sync` is automatic for this enum
- Duration fields provide free timing data at zero extra cost

### Decision 2: Event Callback Mechanism

**Decision:** `Option<Arc<dyn Fn>>` field on `Agent`, set via `AgentBuilder::on_event()`

```rust
// In Agent struct — new field alongside existing fields
pub(crate) event_callback: Option<Arc<dyn Fn(&AgentEvent) + Send + Sync>>,

// In AgentBuilder — new method
impl AgentBuilder {
    /// Register a callback for agent lifecycle events.
    ///
    /// The callback receives a reference to each [`AgentEvent`]
    /// emitted during the agent run.
    pub fn on_event(
        mut self,
        callback: impl Fn(&AgentEvent) + Send + Sync + 'static,
    ) -> Self {
        self.event_callback = Some(Arc::new(callback));
        self
    }
}

// In runtime code — emit pattern
impl Agent {
    fn emit_event(&self, event: &AgentEvent) {
        if let Some(ref cb) = self.event_callback {
            cb(event);
        }
    }
}
```

**Rationale:**
- `Option<Arc>` — when `None`, event emission is a single branch (NFR2: ≤ 1μs)
- `Arc` enables `Clone` on `Agent` (existing pattern — `hooks: Vec<Arc<dyn AgentHook>>`)
- `Fn(&AgentEvent)` — immutable reference, caller can clone if needed
- `Send + Sync + 'static` — matches existing trait object bounds in `Agent` struct
- Single callback (not `Vec`) — simpler API; users can multiplex internally

### Decision 3: `traitclaw-test-utils` Dependency Graph

**Decision:** dev-dependency only, path dependency for tight integration

```
traitclaw-test-utils
  └── [dependencies]
      ├── traitclaw-core = { path = "../traitclaw-core" }
      ├── async-trait (workspace)
      ├── tokio (workspace)
      └── serde_json (workspace)

traitclaw-core         [dev-dependencies] → traitclaw-test-utils
traitclaw-strategies   [dev-dependencies] → traitclaw-test-utils
traitclaw-team         [dev-dependencies] → traitclaw-test-utils
traitclaw-steering     [dev-dependencies] → traitclaw-test-utils
```

**Rationale:**
- `dev-dependency` only — never in user dependency tree (NFR5)
- Path dependency — workspace-native, no version mismatch
- No circular dependency: test-utils depends on core (forward), test code in core depends on test-utils (dev-only, not compiled into library)

### Decision 4: Tracing Span Naming Convention

**Decision:** Hierarchical targets with dot-separated span names

| Operation | Span Target | Span Name | Key Fields |
|-----------|------------|-----------|------------|
| LLM call | `traitclaw::llm` | `llm.complete` | `model`, `provider` |
| Tool execution | `traitclaw::tool` | `tool.call` | `tool_name` |
| Guard check | `traitclaw::guard` | `guard.check` | `guard_name`, `blocked` |
| Strategy iteration | `traitclaw::strategy` | `strategy.iterate` | `iteration`, `strategy_name` |
| Agent run | `traitclaw::agent` | `agent.run` | `session_id` |

**Rationale:**
- Hierarchical targets enable `RUST_LOG=traitclaw::llm=debug` component-level filtering (FR15)
- Dot-separated names follow OpenTelemetry semantic conventions
- Fields as structured data — queryable in Jaeger, Grafana, etc.
- `tracing::info_span!` at info level — zero cost without subscriber (NFR1)

**Implementation pattern:**

```rust
use tracing::info_span;

// In agent.rs — wrap strategy.execute()
let span = info_span!(
    target: "traitclaw::agent",
    "agent.run",
    session_id = %session_id,
);
let _guard = span.enter();
```

### Decision 5: RunUsage Cost Estimation

**Decision:** Extend existing `RunUsage` with optional cost field, configurable pricing table

```rust
// Extend existing RunUsage (in agent.rs)
#[derive(Debug, Clone, Default)]
pub struct RunUsage {
    pub tokens: usize,
    pub iterations: usize,
    pub duration: std::time::Duration,
    /// Estimated cost in USD. `0.0` if no pricing configured.
    pub estimated_cost_usd: f64,
}

// New type in types/pricing.rs
pub struct ModelPricing {
    pub prompt_per_1m_tokens: f64,
    pub completion_per_1m_tokens: f64,
}

// On AgentBuilder
impl AgentBuilder {
    pub fn with_pricing(mut self, table: HashMap<String, ModelPricing>) -> Self {
        self.pricing_table = Some(table);
        self
    }
}
```

**Rationale:**
- Extends existing `RunUsage` (already `#[non_exhaustive]`) — no breaking change
- `estimated_cost_usd` defaults to `0.0` — zero impact on existing code
- Per-1M-tokens pricing matches industry standard (OpenAI, Anthropic pricing pages)
- `HashMap` lookup — negligible overhead (NFR3)
- Unknown models return `0.0` + `tracing::warn!` — no panics

### Decision 6: CI Pipeline Structure

**Decision:** 4 parallel GitHub Actions jobs

```yaml
# .github/workflows/ci.yml
name: CI
on: [push, pull_request]

jobs:
  fmt:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with: { components: rustfmt }
      - run: cargo fmt --all --check

  clippy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with: { components: clippy }
      - uses: Swatinem/rust-cache@v2
      - run: cargo clippy --all-targets --all-features -- -D warnings

  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo test --workspace --all-features

  docs:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo doc --workspace --no-deps
        env: { RUSTDOCFLAGS: "-D warnings" }
```

**Rationale:**
- 4 parallel jobs maximize speed (NFR4: ≤ 10 min)
- `Swatinem/rust-cache@v2` caches cargo registry + build artifacts
- `dtolnay/rust-toolchain@stable` — stable only, no nightly (NFR9)
- Matches `project-context.md` CI checks exactly
- Coverage (`cargo-llvm-cov`) added as optional 5th job — non-blocking

### Decision Impact Analysis

**Implementation Sequence:**
1. `traitclaw-test-utils` crate scaffold → enables all test work
2. Mock types (MockProvider, MockMemory, tools) → enables test migration
3. CI pipeline → catches regressions from day one
4. `AgentEvent` enum + `event.rs` → type foundation for events
5. Tracing spans in runtime → instrumentation layer
6. `on_event()` callback → event delivery mechanism
7. `RunUsage` cost estimation → economics
8. Observability example + migration guide → documentation

**Cross-Component Dependencies:**
- `traitclaw-test-utils` → depends on `traitclaw-core` (path)
- `AgentEvent` → used by `on_event()` callback and tracing spans
- `RunUsage` extension → depends on `AgentEvent::LlmEnd` for token accumulation
- CI → independent, can be done in parallel with any crate work

## Implementation Patterns & Consistency Rules

### v0.8.0–Specific Patterns

All v0.8.0 patterns inherit from `project-context.md` (naming, structure, error handling, testing). The following are **additional** patterns specific to this release.

### Naming Patterns (Additions)

| Element | Convention | Example |
|---------|-----------|---------|
| Event variants | PascalCase, verb-noun | `LlmStart`, `ToolResult`, `GuardBlock` |
| Tracing targets | `traitclaw::{component}` | `traitclaw::llm`, `traitclaw::tool` |
| Tracing span names | `{component}.{action}` | `llm.complete`, `tool.call` |
| Span fields | snake_case | `tool_name`, `prompt_tokens` |
| Pricing keys | Model name strings | `"gpt-4o"`, `"claude-3-5-sonnet"` |

### Event Emission Pattern

All event emissions follow this exact pattern:

```rust
// CORRECT: emit helper method
fn emit_event(&self, event: &AgentEvent) {
    if let Some(ref cb) = self.event_callback {
        cb(event);
    }
}

// Usage — always construct inline, never store
self.emit_event(&AgentEvent::LlmStart {
    model: model_name.clone(),
});
```

**Anti-pattern:**
```rust
// ❌ WRONG: Don't store events, emit immediately
let event = AgentEvent::LlmStart { model: "gpt-4o".into() };
events.push(event); // ❌ No event buffering
```

### Tracing Instrumentation Pattern

```rust
use tracing::{info_span, Instrument};

// Pattern 1: Sync span guard
let span = info_span!(target: "traitclaw::llm", "llm.complete", model = %model);
let _guard = span.enter();

// Pattern 2: Async instrumentation
async fn call_provider(&self, req: CompletionRequest) -> Result<CompletionResponse> {
    self.provider
        .complete(req)
        .instrument(info_span!(
            target: "traitclaw::llm",
            "llm.complete",
            model = %self.provider.model_info().name,
        ))
        .await
}
```

**Anti-patterns:**

| ❌ Don't | ✅ Do |
|----------|------|
| `tracing::info!("calling LLM")` for instrumentation | `info_span!` for structured spans |
| `#[instrument]` attribute on hot paths | Manual `info_span!` for precise control |
| `debug_span!` for production instrumentation | `info_span!` — visible at default level |

### Test-Utils Module Pattern

Each module in `traitclaw-test-utils` follows:

```rust
//! Module-level doc comment explaining purpose.
//!
//! # Example
//! ```rust
//! use traitclaw_test_utils::provider::MockProvider;
//! let p = MockProvider::text("hello");
//! ```

// All types are `pub` (this is a test library)
pub struct MockProvider { ... }

impl MockProvider {
    /// Factory method doc comment.
    pub fn text(response: &str) -> Self { ... }
}

// Trait implementations
#[async_trait]
impl Provider for MockProvider { ... }
```

### CI Configuration Pattern

- All CI job steps use pinned action versions (`@v4`, `@v2`)
- Rust toolchain always `@stable` — never `@nightly`
- `rust-cache` used for all jobs except `fmt` (formatting is fast)
- Environment variables for flags: `RUSTDOCFLAGS`, `CARGO_TERM_COLOR`

## Project Structure & Boundaries

### v0.8.0 Project Structure Changes

```
traitclaw/
├── .github/
│   └── workflows/
│       └── ci.yml                     # NEW: CI pipeline (FR6-11)
├── crates/
│   ├── traitclaw-core/
│   │   └── src/
│   │       ├── types/
│   │       │   ├── event.rs           # NEW: AgentEvent enum (FR16-19)
│   │       │   ├── pricing.rs         # NEW: ModelPricing (FR20-22)
│   │       │   └── ... (existing)
│   │       ├── agent.rs               # MOD: add event_callback field, emit_event(), RunUsage.estimated_cost_usd
│   │       ├── agent_builder.rs       # MOD: add on_event(), with_pricing()
│   │       └── lib.rs                 # MOD: re-export AgentEvent, ModelPricing
│   │
│   └── traitclaw-test-utils/          # NEW CRATE (dev-dependency only)
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs                 # pub mod re-exports
│           ├── provider.rs            # MockProvider
│           ├── memory.rs              # MockMemory
│           ├── tools.rs               # EchoTool, FailTool
│           └── runtime.rs             # make_runtime() helper
│
├── examples/
│   └── 26-observability/              # NEW: tracing + events demo (FR25)
│       ├── Cargo.toml
│       └── src/main.rs
│
└── docs/
    └── migration-v0.7-to-v0.8.md      # NEW: migration guide (FR23)
```

### FR-to-Module Mapping

| FR Group | Module Location |
|----------|----------------|
| FR1-5 (Test Utils) | `crates/traitclaw-test-utils/src/` |
| FR6-11 (CI) | `.github/workflows/ci.yml` |
| FR12-15 (Tracing) | `crates/traitclaw-core/src/agent.rs` (span instrumentation) |
| FR16-19 (Events) | `crates/traitclaw-core/src/types/event.rs` + `agent.rs` + `agent_builder.rs` |
| FR20-22 (Cost) | `crates/traitclaw-core/src/types/pricing.rs` + `agent.rs` (RunUsage extension) |
| FR23-25 (Docs) | `docs/migration-v0.7-to-v0.8.md` + `examples/26-observability/` |
| FR26-28 (Compat) | Verified by existing test suite |

### Architectural Boundaries

**Crate Boundaries (uni-directional):**
- `traitclaw-core` → owns `AgentEvent`, `ModelPricing`, tracing spans, event callback
- `traitclaw-test-utils` → depends on core, provides mock types for dev-time only
- `traitclaw` (meta) → re-exports `AgentEvent`, `ModelPricing` from core
- NO changes to `traitclaw-strategies`, `traitclaw-team`, or other crates' public APIs

**Data Flow — Event System:**
```
Agent::run() → strategy.execute()
    ↓
  emit_event(LlmStart) → callback (if registered)
    ↓
  provider.complete()   → tracing span [traitclaw::llm / llm.complete]
    ↓
  emit_event(LlmEnd)   → callback + accumulate RunUsage tokens/cost
    ↓
  tool.call()           → tracing span [traitclaw::tool / tool.call]
    ↓
  emit_event(ToolResult) → callback
    ↓
  AgentOutput { usage: RunUsage { estimated_cost_usd } }
```

**Data Flow — Test Utils:**
```
crate tests (#[cfg(test)])
    ↓
  use traitclaw_test_utils::{MockProvider, MockMemory, make_runtime};
    ↓
  construct AgentRuntime with deterministic mocks
    ↓
  assert on AgentOutput
```

## Architecture Validation Results

### Coherence Validation ✅

**Decision Compatibility:** All 6 decisions are mutually compatible:
- `AgentEvent` is emitted by `emit_event()` which is gated by `Option<Arc<dyn Fn>>`
- Tracing spans are independent of event callbacks — both can fire simultaneously
- `RunUsage` accumulates tokens from `AgentEvent::LlmEnd` — natural data flow
- Test-utils crate has no knowledge of events/tracing — clean separation
- CI pipeline validates all code changes — catches integration issues

**Pattern Consistency:** v0.8.0 patterns extend (never replace) `project-context.md` rules. Naming, testing, error handling all align with existing codebase.

**Structure Alignment:** All new files follow existing workspace conventions. No new directories outside established patterns.

### Requirements Coverage ✅

| Area | Coverage | Notes |
|------|----------|-------|
| FR1-5 (Test Utils) | ✅ 100% | `traitclaw-test-utils` crate with all mock types |
| FR6-11 (CI) | ✅ 100% | 4-job GitHub Actions pipeline |
| FR12-15 (Tracing) | ✅ 100% | Structured spans with component filtering |
| FR16-19 (Events) | ✅ 100% | `AgentEvent` enum + `on_event()` callback |
| FR20-22 (Cost) | ✅ 100% | `RunUsage` + `ModelPricing` |
| FR23-25 (Docs) | ✅ 100% | Migration guide + observability example |
| FR26-28 (Compat) | ✅ 100% | All additions are non-breaking |
| NFR1-10 | ✅ 100% | Zero-cost tracing, Option callback, stable Rust |

### Gap Analysis

**Critical Gaps:** None ✅

**Minor Gaps (non-blocking):**
1. `AgentEvent` may benefit from a `timestamp` field — defer to implementation (can add without breaking due to `#[non_exhaustive]`)
2. Cost pricing table needs initial values for popular models — implementation detail
3. Coverage threshold for CI not yet defined — add as configurable parameter

### Architecture Completeness Checklist

- [x] Project context analyzed — 28 FRs, 10 NFRs fully mapped
- [x] Scale and complexity assessed — Medium (infrastructure-only)
- [x] Technical constraints identified — 6 constraints documented
- [x] Cross-cutting concerns mapped — 5 concerns addressed
- [x] Critical decisions documented with code examples — 6 decisions
- [x] Implementation patterns defined — event emission, tracing, test-utils
- [x] Project structure with FR mapping complete — every FR → module
- [x] Crate boundaries and data flow documented — event + test-utils flows
- [x] Validation passed — zero critical gaps

### Architecture Readiness Assessment

**Overall Status:** ✅ READY FOR IMPLEMENTATION

**Confidence Level:** High

**Key Strengths:**
- Zero new public traits — all additions are types and builder methods
- Uni-directional crate dependency graph preserved
- `#[non_exhaustive]` on `AgentEvent` future-proofs for v0.9.0+
- All patterns extend (never replace) existing `project-context.md` rules
- CI pipeline exactly matches existing local quality checks

**Areas for Future Enhancement:**
- OpenTelemetry exporter integration (v0.9.0)
- Distributed tracing correlation IDs (v0.9.0)
- Cost alerting / budget enforcement (v1.0+)
- `AgentEvent::StrategyStep` variant linking to `ThoughtStep` (v0.9.0)

### Implementation Handoff

**AI Agent Guidelines:**
- Follow all architectural decisions exactly as documented
- Use implementation patterns consistently — especially event emission and tracing span patterns
- Respect crate boundaries — events and pricing types live in `traitclaw-core` only
- `traitclaw-test-utils` is dev-dependency only — never in regular dependencies
- Refer to this document for all v0.8.0 architectural questions

**First Implementation Priority:** Scaffold `traitclaw-test-utils` crate with `Cargo.toml`, `lib.rs`, empty module files, and workspace registration.
