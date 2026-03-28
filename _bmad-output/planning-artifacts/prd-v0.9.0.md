---
stepsCompleted:
  - step-01-init
  - step-02-discovery
  - step-02b-vision
  - step-02c-executive-summary
  - step-03-success
  - step-04-journeys
  - step-05-domain
  - step-06-innovation
  - step-07-project-type
  - step-08-scoping
  - step-09-functional
  - step-10-nonfunctional
  - step-11-polish
  - step-12-complete
inputDocuments:
  - product-brief-traitclaw-2026-03-26.md
  - v080-brainstorming.md
  - prd-v0.8.0.md
  - architecture-v0.8.0.md
  - migration-v0.7-to-v0.8.md
workflowType: 'prd'
documentCounts:
  briefs: 1
  research: 0
  brainstorming: 1
  projectDocs: 6
classification:
  projectType: developer_tool
  domain: scientific
  complexity: medium
  projectContext: brownfield
---

# Product Requirements Document - TraitClaw v0.9.0 "Hardening"

**Author:** Bangvu
**Date:** 2026-03-28
**Version:** 0.9.0
**Status:** Draft

## Executive Summary

TraitClaw v0.9.0 "Hardening" is the **final pre-stable release** of the TraitClaw AI Agent Framework. Its singular purpose is to prepare the public API for the v1.0 semver freeze by removing deprecated traits, auditing the public API surface, and improving developer experience.

This is the **last version allowing breaking changes**. After v0.9.0, the public API is frozen until v2.0.

### Key Outcomes

1. **Clean API surface** — Remove all deprecated types (`ContextStrategy`, `OutputProcessor`) and their 35 `#[allow(deprecated)]` annotations
2. **Audited public surface** — Every `pub` item reviewed for necessity, visibility, and naming consistency
3. **Improved developer experience** — Actionable error messages, enriched prelude, clear migration path
4. **Zero technical debt** — No deprecated items, no compatibility shims, no dead code

### Context

| Metric | Current (v0.8.0) | Target (v0.9.0) |
|--------|-------------------|------------------|
| Codebase | ~39,200 LoC | ~38,000 LoC (net reduction) |
| Tests | 658 | 658+ (no regression) |
| Deprecated items | 2 traits, 35 `allow(deprecated)` | 0 |
| `#![deny(missing_docs)]` | All 13 crates | All crates ✅ |
| Crates | 14 | 14 (unchanged) |

## Product Vision

TraitClaw is a Rust AI Agent Framework — **simple by default, powerful when needed**. The framework provides composable traits for building production-grade AI agents with type-safe tool calling, multi-strategy reasoning, and pluggable observability.

v0.9.0 is not about new features. It is about **trust**: giving developers confidence that the API they build against today will remain stable. This version draws the line between "evolving framework" and "dependable foundation."

## Success Criteria

### SC1: Zero Deprecated Items
- No `#[deprecated]` attributes remain in the codebase
- No `#[allow(deprecated)]` annotations remain
- The deprecated trait files (`context_strategy.rs`, `output_processor.rs`) are deleted

### SC2: API Audit Complete
- Every `pub` item in `traitclaw-core` has been reviewed
- Prelude contains all commonly-used types and no dead re-exports
- `AgentRuntime` struct has no deprecated fields

### SC3: Improved Error Messages
- `AgentBuilder::build()` produces actionable error messages ("did you forget to .provider()?")
- Error messages follow the pattern: what happened + what to do

### SC4: Migration Guide Published
- `docs/migration-v0.8-to-v0.9.md` documents all breaking changes
- Before/after code examples for every breaking change
- Guide is linked from project README or CHANGELOG

### SC5: Zero Regression
- All 658+ existing tests pass
- All 26 examples compile
- CI pipeline (4 jobs) passes: fmt, clippy, test, docs

## User Journeys

### Journey 1: Existing User Upgrading from v0.8.0

1. Developer reads `migration-v0.8-to-v0.9.md`
2. Searches codebase for `ContextStrategy` → replaces with `ContextManager`
3. Searches for `OutputProcessor` → replaces with `OutputTransformer`
4. Runs `cargo build` — compiles cleanly
5. **Outcome:** Upgrade takes < 5 minutes for typical projects

### Journey 2: New User Discovering TraitClaw

1. Developer adds `traitclaw` to `Cargo.toml`
2. Uses `traitclaw::prelude::*` — gets all essential types in one import
3. Gets clear, actionable errors if they misconfigure the builder
4. **Outcome:** Clean API with no deprecated noise, excellent first impression

### Journey 3: Custom Strategy Author

1. Developer implements `AgentStrategy` using `AgentRuntime`
2. `AgentRuntime` struct only contains current, relevant fields
3. No deprecated fields to confuse or mislead
4. **Outcome:** Clear, minimal surface for extension points

## Project-Type Requirements (Developer Tool)

### DT1: API Surface Clarity
- Every public type serves a clear purpose
- No "zombie" re-exports from deprecated modules
- Prelude is the recommended import path for common usage

### DT2: Migration Support
- Migration guide with search-and-replace patterns
- Compiler errors from removed types point users to replacements
- No silent behavior changes — all changes are compile-time breaks

### DT3: Semver Readiness
- Public API is ready for v1.0 freeze
- All trait signatures finalized
- All type names finalized

## Scoping

### In Scope (v0.9.0)

| Category | Items |
|----------|-------|
| **Deprecated Removal** | Delete `ContextStrategy` trait, `OutputProcessor` trait, all blanket impls, all compat shims |
| **API Audit** | Review all `pub` items in `traitclaw-core`, fix visibility, update prelude |
| **Error Improvement** | Improve `AgentBuilder::build()` validation errors |
| **AgentRuntime Cleanup** | Remove deprecated fields (`context_strategy`, `output_processor`) |
| **Migration Guide** | `docs/migration-v0.8-to-v0.9.md` with breaking change documentation |
| **Doc References** | Update all doc comments referencing removed types |

### Out of Scope (Deferred)

| Category | Reason |
|----------|--------|
| Property-based testing (`proptest`) | 658 tests sufficient; defer to post-v1.0 |
| Snapshot testing (`insta`) | Agent outputs are non-deterministic |
| Benchmarks (`criterion`) | No current performance concerns |
| New features | v0.9.0 is cleanup-only |
| Code coverage tooling | CI already enforces quality gates |

## Functional Requirements

### Deprecated Type Removal

- FR1: The framework removes the `ContextStrategy` trait and all its implementations (`NoopContextStrategy`, `SlidingWindowStrategy`)
- FR2: The framework removes the `OutputProcessor` trait and all its implementations (`NoopProcessor`, `TruncateProcessor`, `ChainProcessor`)
- FR3: The framework removes the blanket impl `impl<T: ContextStrategy> ContextManager for T`
- FR4: The framework removes the blanket impl `impl<T: OutputProcessor> OutputTransformer for T`
- FR5: The framework removes `context_strategy` and `output_processor` fields from `AgentRuntime`
- FR6: The framework removes all `#[allow(deprecated)]` annotations from the codebase
- FR7: The framework deletes the source files `context_strategy.rs` and `output_processor.rs`

### API Surface Audit

- FR8: The framework audits all `pub` items in `traitclaw-core` for appropriate visibility
- FR9: The framework removes deprecated types from the prelude module
- FR10: The framework adds commonly-used types to the prelude: `CompressedMemory`, `RetryConfig`, `RetryProvider`, `DynamicRegistry`
- FR11: The framework removes the `pub mod context_strategy` and `pub mod output_processor` declarations from `traits/mod.rs`
- FR12: The framework removes all deprecated re-exports from `traitclaw-core/src/lib.rs`

### Error Message Improvement

- FR13: `AgentBuilder::build()` produces actionable error messages identifying which required field is missing
- FR14: Error messages follow the format: "[Context]: [what happened]. [suggestion to fix]"

### Documentation Updates

- FR15: All doc comments referencing `ContextStrategy` are updated to reference `ContextManager`
- FR16: All doc comments referencing `OutputProcessor` are updated to reference `OutputTransformer`
- FR17: Module-level documentation removed for deleted modules
- FR18: Migration guide `docs/migration-v0.8-to-v0.9.md` is created with breaking change documentation
- FR19: Migration guide includes search-and-replace patterns for each removed type
- FR20: Migration guide includes before/after code examples

### Backward Compatibility Verification

- FR21: All 26 existing examples compile without modification (except those using deprecated types)
- FR22: All 658+ existing tests pass
- FR23: CI pipeline (fmt, clippy, test, docs) passes with zero warnings

## Non-Functional Requirements

### NFR1: Build Performance
- Build time does not increase by more than 5% vs v0.8.0
- Removing deprecated code should result in net code reduction

### NFR2: API Stability Signal
- Zero deprecated items remaining signals readiness for v1.0 freeze
- All public types are at their final naming

### NFR3: Documentation Quality
- All public items have doc comments (enforced by `#![deny(missing_docs)]`)
- No broken doc links (enforced by `RUSTDOCFLAGS="-D warnings"`)

### NFR4: Test Coverage
- No test regressions — all 658+ tests continue to pass
- Tests for deprecated types are either migrated or removed with the deprecated code

### NFR5: Migration Effort
- Typical user migration from v0.8.0 should take less than 5 minutes
- Migration consists only of trait renames — method signatures are identical

## Technical Constraints

- **Language:** Rust (stable toolchain)
- **MSRV:** Inherited from workspace (current stable)
- **Dependencies:** No new dependencies added
- **Breaking changes:** Allowed in this version ONLY

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Users still on `ContextStrategy` | Build breaks | Migration guide with exact search-replace patterns |
| Custom `AgentStrategy` implementations accessing `runtime.context_strategy` | Build breaks | Migration guide with before/after examples |
| Missed deprecated references | Inconsistent API | CI enforces `-D warnings`, grep audit |
| Test breakage from removed types | Quality regression | Run full test suite before and after each change |

## Verification Plan

### Automated
```bash
# All 4 CI jobs must pass
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
```

### Manual Verification
```bash
# Zero deprecated items
grep -r "#\[deprecated" crates/ --include="*.rs" | wc -l  # Expected: 0
grep -r "allow(deprecated)" crates/ --include="*.rs" | wc -l  # Expected: 0

# All examples compile
for dir in examples/*/; do
  cargo check --manifest-path "${dir}Cargo.toml"
done
```

## Appendix: Deprecated Items Inventory

### Files to Delete
- `crates/traitclaw-core/src/traits/context_strategy.rs` (188 lines)
- `crates/traitclaw-core/src/traits/output_processor.rs` (159 lines)

### `#[allow(deprecated)]` Locations (35 total)
- `agent.rs` — 4 occurrences (compat imports)
- `agent_builder.rs` — 4 occurrences (compat builder methods)
- `default_strategy.rs` — 2 occurrences (compat runtime construction)
- `lib.rs` — 4 occurrences (re-exports + prelude)
- `traits/context_strategy.rs` — 3 occurrences (self-references)
- `traits/context_manager.rs` — 3 occurrences (blanket impl)
- `traits/output_processor.rs` — 6 occurrences (self-references)
- `traits/output_transformer.rs` — 3 occurrences (blanket impl)
- `traits/strategy.rs` — 3 occurrences (AgentRuntime fields)
- `test-utils/runtime.rs` — 2 occurrences (compat runtime construction)
- `traits/execution_strategy.rs` — 1 occurrence (compat)
