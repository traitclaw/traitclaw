---
stepsCompleted:
  - step-01-init
  - step-02-context
  - step-03-starter
  - step-04-decisions
  - step-05-patterns
  - step-06-structure
  - step-07-validation
  - step-08-complete
inputDocuments:
  - prd-v0.9.0.md
  - architecture-v0.8.0.md
  - prd-v0.8.0.md
  - v080-brainstorming.md
workflowType: 'architecture'
project_name: 'traitclaw'
user_name: 'Bangvu'
date: '2026-03-28'
---

# Architecture Decision Document — v0.9.0 "Hardening"

_This document describes the architectural decisions for TraitClaw v0.9.0, focused on API cleanup and hardening before the v1.0 semver freeze. It supplements (not replaces) the v0.8.0 architecture document._

## Project Context Analysis

### Requirements Overview

**Functional Requirements (23 FRs across 5 capability areas):**

- **Deprecated Type Removal (FR1-7):** Delete `ContextStrategy` trait, `OutputProcessor` trait, all blanket impls, all implementations, all `#[allow(deprecated)]` annotations, source files
- **API Surface Audit (FR8-12):** Audit all `pub` items, clean prelude, remove deprecated re-exports
- **Error Message Improvement (FR13-14):** Actionable builder validation errors
- **Documentation Updates (FR15-20):** Update doc comments, create migration guide
- **Backward Compatibility Verification (FR21-23):** All examples compile, tests pass, CI green

**Non-Functional Requirements (5 NFRs):**

- NFR1 (Build): Build time same or better
- NFR2 (Stability): Zero deprecated items signals v1.0 readiness
- NFR3 (Docs): All public items documented
- NFR4 (Tests): No regressions, 658+ tests pass
- NFR5 (Migration): < 5 minutes for typical user

### Scale & Complexity

- **Complexity level:** Low-Medium (destructive refactoring — removing code is simpler than adding)
- **Primary domain:** Library / Framework (Rust crate) — cleanup-only release
- **Architectural components affected:** 4 (traits module, AgentRuntime, AgentBuilder, prelude)
- **Net LoC change:** Approximately -400 lines

### Technical Constraints & Dependencies

1. All changes are within `traitclaw-core` — no other crates have deprecated types
2. No new dependencies — this is a removal-only release
3. Breaking changes are allowed (final chance before v1.0 freeze)
4. `AgentRuntime` struct is public — field removal is a breaking change
5. All 26 examples must compile after changes
6. External crate users with custom `AgentStrategy` implementations will need to update

### Cross-Cutting Concerns

| Concern | Impact |
|---------|--------|
| Breaking changes | Removed traits cause compile errors — migration guide required |
| Trait object safety | No impact — `ContextManager` and `OutputTransformer` remain object-safe |
| Test suites | Tests for removed types are deleted with the types. Blanket impl tests removed. |
| Documentation | All doc references to removed types updated |
| Feature flags | No feature flag changes |
| Error handling | No error type changes — improved error messages only |

## Starter Template Evaluation

### Primary Technology Domain

Rust Library / Framework (Cargo workspace) — brownfield project with 14 existing crates.

### Starter Options

N/A — brownfield project. No new crates or scaffolding needed. This release only removes code and modifies existing files.

### Existing Architectural Foundation

Unchanged from v0.8.0. Key elements:

- **Language & Runtime:** Rust edition 2021, async via `tokio 1.x`
- **Workspace:** 14 crates in `crates/` + 26 examples in `examples/`
- **Testing:** 658+ tests, `traitclaw-test-utils` for shared mocks
- **CI:** 4 parallel GitHub Actions jobs (fmt, clippy, test, docs)
- **Observability:** `tracing` crate with OpenTelemetry GenAI conventions (added in v0.8.0)

## Core Architectural Decisions

### Decision Priority Analysis

**Critical Decisions (Block Implementation):**
1. Deletion order for deprecated types (dependency graph matters)
2. `AgentRuntime` field removal strategy
3. Blanket impl removal handling

**Important Decisions (Shape Quality):**
4. Prelude composition after cleanup
5. Error message format standardization

**No Deferred Decisions** — v0.9.0 is the final cleanup before freeze.

---

### Decision 1: Deletion Order (Dependency-First)

**Decision:** Delete in reverse-dependency order to prevent cascading compile errors during development.

**Order:**

```
Step 1: Remove blanket impls (context_manager.rs, output_transformer.rs)
        → breaks: nothing (blanket impls are additive)

Step 2: Remove AgentRuntime deprecated fields (strategy.rs)
        → breaks: custom AgentStrategy impls accessing old fields

Step 3: Remove deprecated imports & re-exports (lib.rs, traits mod, prelude)
        → breaks: users importing deprecated types

Step 4: Update AgentBuilder to remove deprecated builder methods
        → breaks: users calling .context_strategy() or .output_processor()

Step 5: Remove default_strategy.rs references to deprecated types
        → breaks: nothing (internal)

Step 6: Delete source files (context_strategy.rs, output_processor.rs)
        → final cleanup

Step 7: Remove #[allow(deprecated)] annotations everywhere
        → must be last — annotations are needed until types are gone
```

**Rationale:**
- Blanket impls go first because they depend on the deprecated traits but nothing depends on them
- Source files go last because tests and impls reference them
- Annotations go absolute last because clippy would error during intermediate steps

---

### Decision 2: AgentRuntime Field Removal

**Decision:** Remove 2 deprecated fields, keep all other fields as `pub`.

**Before (v0.8.0):**
```rust
#[allow(deprecated)]
pub struct AgentRuntime {
    pub provider: Arc<dyn Provider>,
    pub tools: Vec<Arc<dyn ErasedTool>>,
    pub memory: Arc<dyn Memory>,
    pub guards: Vec<Arc<dyn Guard>>,
    pub hints: Vec<Arc<dyn Hint>>,
    pub tracker: Arc<dyn Tracker>,
    pub context_manager: Arc<dyn ContextManager>,
    pub context_strategy: Arc<dyn ContextStrategy>,   // ❌ REMOVE
    pub execution_strategy: Arc<dyn ExecutionStrategy>,
    pub output_transformer: Arc<dyn OutputTransformer>,
    pub output_processor: Arc<dyn OutputProcessor>,     // ❌ REMOVE
    pub tool_registry: Arc<dyn ToolRegistry>,
    pub config: AgentConfig,
    pub hooks: Vec<Arc<dyn super::hook::AgentHook>>,
}
```

**After (v0.9.0):**
```rust
pub struct AgentRuntime {
    pub provider: Arc<dyn Provider>,
    pub tools: Vec<Arc<dyn ErasedTool>>,
    pub memory: Arc<dyn Memory>,
    pub guards: Vec<Arc<dyn Guard>>,
    pub hints: Vec<Arc<dyn Hint>>,
    pub tracker: Arc<dyn Tracker>,
    pub context_manager: Arc<dyn ContextManager>,
    pub execution_strategy: Arc<dyn ExecutionStrategy>,
    pub output_transformer: Arc<dyn OutputTransformer>,
    pub tool_registry: Arc<dyn ToolRegistry>,
    pub config: AgentConfig,
    pub hooks: Vec<Arc<dyn super::hook::AgentHook>>,
}
```

**Rationale:**
- Fields go from 14 to 12 — cleaner struct
- No more `#[allow(deprecated)]` on struct definition
- No imports needed for deprecated trait types
- The `#[allow(deprecated)]` import of `ContextStrategy` and `OutputProcessor` in strategy.rs is also removed

**Migration Impact:**
- Custom `AgentStrategy` implementations that access `runtime.context_strategy` → use `runtime.context_manager` instead
- Custom `AgentStrategy` implementations that access `runtime.output_processor` → use `runtime.output_transformer` instead
- Method signatures are nearly identical: sync `prepare()` → async `prepare()`, sync `process()` → async `transform()`

---

### Decision 3: Blanket Impl Removal

**Decision:** Delete both blanket impls entirely.

**Removed from `context_manager.rs`:**
```rust
// DELETED
#[allow(deprecated)]
#[async_trait]
impl<T: ContextStrategy + 'static> ContextManager for T {
    async fn prepare(&self, messages: &mut Vec<Message>, context_window: usize, state: &mut AgentState) {
        ContextStrategy::prepare(self, messages, context_window, state);
    }
}
```

**Removed from `output_transformer.rs`:**
```rust
// DELETED
#[allow(deprecated)]
#[async_trait]
impl<T: OutputProcessor + 'static> OutputTransformer for T {
    async fn transform(&self, output: String, _tool_name: &str, _state: &AgentState) -> String {
        OutputProcessor::process(self, output)
    }
}
```

**Rationale:**
- These blanket impls only existed as a migration bridge from v0.3.0 → v0.9.0
- Without the deprecated traits, the blanket impls have nothing to bridge
- All built-in implementations (`SlidingWindowStrategy`, `TruncateProcessor`) were already available through `ContextManager` / `OutputTransformer` alternatives

**Tests Affected:**
- `test_blanket_impl_delegates_to_context_strategy` — DELETE
- `test_blanket_impl_delegates_to_output_processor` — DELETE

---

### Decision 4: Prelude Composition

**Decision:** Clean prelude of deprecated items, add commonly-used items.

**Removed from prelude:**
```rust
// DELETED from prelude
pub use crate::traits::context_strategy::{ContextStrategy, NoopContextStrategy, SlidingWindowStrategy};
pub use crate::traits::output_processor::{OutputProcessor, TruncateProcessor};
```

**Added to prelude:**
```rust
// ADDED to prelude
pub use crate::memory::compressed::CompressedMemory;
pub use crate::retry::{RetryConfig, RetryProvider};
pub use crate::registries::DynamicRegistry;
```

**Rationale:**
- `CompressedMemory` — used by most non-trivial agents
- `RetryConfig` / `RetryProvider` — common production pattern
- `DynamicRegistry` — frequently used for runtime tool registration
- These 4 types are used in >60% of examples

---

### Decision 5: Error Message Format

**Decision:** Standardized format: `"[Component]: [what happened]. Use .[method]() to fix."`

**Example — Missing Provider:**
```rust
// Before (v0.8.0)
Err(Error::Config("Provider is required".into()))

// After (v0.9.0)
Err(Error::Config(
    "AgentBuilder: no provider configured. Use .provider(my_provider) before .build()".into()
))
```

**Rationale:**
- Actionable errors reduce debugging time
- Component prefix identifies where the error originates
- Method suggestion tells the user exactly what to do
- Consistent format across all builder validation errors

---

## Implementation Patterns & Consistency Rules

### Pattern 1: File Deletion Protocol

When deleting a trait file:

1. Remove all imports of the file from other modules
2. Remove module declaration from `lib.rs` `pub mod traits { ... }`
3. Remove all re-exports from `lib.rs` root and prelude
4. Delete the file
5. Run `cargo check` after each deletion to catch cascading issues

### Pattern 2: Deprecated Annotation Cleanup

After all deprecated types are removed:

```bash
# Verify zero deprecated annotations remain
grep -rn "allow(deprecated)" crates/ --include="*.rs"
# Expected output: (empty)
```

Every `#[allow(deprecated)]` must be removed — leaving any behind is a bug.

### Pattern 3: Doc Comment Updates

When a doc comment references a removed type:

```rust
// Before
/// Use [`ContextManager`] instead of the deprecated [`ContextStrategy`].

// After
/// Use [`ContextManager`] for pluggable context window management.
```

Remove migration language — v0.9.0 is the target version, not a stepping stone.

## Project Structure & Boundaries

### Files Modified

```
crates/traitclaw-core/src/
├── traits/
│   ├── context_strategy.rs      ← DELETE
│   ├── output_processor.rs      ← DELETE
│   ├── context_manager.rs       ← MODIFY (remove blanket impl + tests)
│   ├── output_transformer.rs    ← MODIFY (remove blanket impl + tests)
│   ├── strategy.rs              ← MODIFY (remove AgentRuntime deprecated fields)
│   └── execution_strategy.rs    ← MODIFY (remove allow(deprecated))
├── lib.rs                       ← MODIFY (remove re-exports, clean prelude)
├── agent.rs                     ← MODIFY (remove deprecated imports)
├── agent_builder.rs             ← MODIFY (remove deprecated builder methods, improve errors)
└── default_strategy.rs          ← MODIFY (remove deprecated references)

crates/traitclaw-test-utils/src/
└── runtime.rs                   ← MODIFY (remove deprecated runtime construction)

docs/
└── migration-v0.8-to-v0.9.md   ← NEW
```

### Files NOT Modified

All other crates (`traitclaw-strategies`, `traitclaw-steering`, `traitclaw-mcp`, etc.) do not import deprecated types and require no changes.

### Dependency Impact

```
traitclaw-core (modified)
    ↓ depended on by
traitclaw (meta-crate — glob re-export, no changes needed)
    ↓ depended on by
examples/* (must compile — verification step)
```

## Architecture Validation

### Checklist

| Check | Status |
|-------|--------|
| All deprecated types have async equivalents | ✅ `ContextManager`, `OutputTransformer` |
| No functionality lost | ✅ `SlidingWindowStrategy` → `RuleBasedCompressor`, `TruncateProcessor` → `BudgetAwareTruncator` |
| `AgentRuntime` fields are all non-deprecated | ✅ 12 fields (down from 14) |
| Prelude is comprehensive | ✅ 4 new items added |
| Migration path is clear | ✅ Trait renames only, same method signatures |
| Deletion order prevents cascading errors | ✅ Reverse dependency order |
| All 26 examples will compile | ✅ No example uses deprecated types directly |
| All 658+ tests will pass | ✅ Only blanket-impl tests are deleted |

### Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| External users on `ContextStrategy` | Low (deprecated 3 versions ago) | Compile error | Migration guide |
| Custom `AgentStrategy` using `runtime.context_strategy` | Low | Compile error | Migration guide with before/after |
| Missed `allow(deprecated)` annotation | Medium | Clippy warning | Automated grep verification |
| Doc link breakage | Medium | Warning in cargo doc | `RUSTDOCFLAGS="-D warnings"` enforces |

### Verification Commands

```bash
# Full CI suite
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps

# Zero deprecated items
grep -rn "#\[deprecated" crates/ --include="*.rs" | wc -l   # Expected: 0
grep -rn "allow(deprecated)" crates/ --include="*.rs" | wc -l  # Expected: 0

# All examples compile
for dir in examples/*/; do cargo check --manifest-path "${dir}Cargo.toml"; done
```
