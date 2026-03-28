---
stepsCompleted:
  - step-01-validate-prerequisites
  - step-02-design-epics
  - step-03-create-stories
  - step-04-final-validation
inputDocuments:
  - planning-artifacts/prd-v0.9.0.md
  - planning-artifacts/architecture-v0.9.0.md
---

# TraitClaw v0.9.0 "Hardening" - Epic Breakdown

## Overview

This document provides the complete epic and story breakdown for TraitClaw v0.9.0, decomposing the PRD and Architecture requirements into implementable stories. This is a cleanup-only release — the final version allowing breaking changes before v1.0 semver freeze.

## Requirements Inventory

### Functional Requirements

- FR1: The framework removes the `ContextStrategy` trait and all its implementations (`NoopContextStrategy`, `SlidingWindowStrategy`)
- FR2: The framework removes the `OutputProcessor` trait and all its implementations (`NoopProcessor`, `TruncateProcessor`, `ChainProcessor`)
- FR3: The framework removes the blanket impl `impl<T: ContextStrategy> ContextManager for T`
- FR4: The framework removes the blanket impl `impl<T: OutputProcessor> OutputTransformer for T`
- FR5: The framework removes `context_strategy` and `output_processor` fields from `AgentRuntime`
- FR6: The framework removes all `#[allow(deprecated)]` annotations from the codebase
- FR7: The framework deletes the source files `context_strategy.rs` and `output_processor.rs`
- FR8: The framework audits all `pub` items in `traitclaw-core` for appropriate visibility
- FR9: The framework removes deprecated types from the prelude module
- FR10: The framework adds commonly-used types to the prelude: `CompressedMemory`, `RetryConfig`, `RetryProvider`, `DynamicRegistry`
- FR11: The framework removes `pub mod context_strategy` and `pub mod output_processor` declarations from traits module
- FR12: The framework removes all deprecated re-exports from `traitclaw-core/src/lib.rs`
- FR13: `AgentBuilder::build()` produces actionable error messages identifying which required field is missing
- FR14: Error messages follow the format: "[Context]: [what happened]. [suggestion to fix]"
- FR15: All doc comments referencing `ContextStrategy` are updated to reference `ContextManager`
- FR16: All doc comments referencing `OutputProcessor` are updated to reference `OutputTransformer`
- FR17: Module-level documentation removed for deleted modules
- FR18: Migration guide `docs/migration-v0.8-to-v0.9.md` is created with breaking change documentation
- FR19: Migration guide includes search-and-replace patterns for each removed type
- FR20: Migration guide includes before/after code examples
- FR21: All 26 existing examples compile without modification
- FR22: All 658+ existing tests pass
- FR23: CI pipeline (fmt, clippy, test, docs) passes with zero warnings

### Non-Functional Requirements

- NFR1: Build time does not increase by more than 5% vs v0.8.0
- NFR2: Zero deprecated items remaining signals readiness for v1.0 freeze
- NFR3: All public items have doc comments (enforced by `#![deny(missing_docs)]`)
- NFR4: No test regressions — all 658+ tests continue to pass
- NFR5: Typical user migration from v0.8.0 should take less than 5 minutes

### Additional Requirements

- Deletion order follows reverse-dependency order per Architecture Decision 1
- `AgentRuntime` goes from 14 to 12 fields per Architecture Decision 2
- Prelude adds 4 commonly-used types per Architecture Decision 4
- Error messages follow standardized format per Architecture Decision 5

### UX Design Requirements

N/A — Library project, no UI components.

### FR Coverage Map

- FR1: Epic 1 - Remove ContextStrategy trait + implementations
- FR2: Epic 1 - Remove OutputProcessor trait + implementations
- FR3: Epic 1 - Remove ContextManager blanket impl
- FR4: Epic 1 - Remove OutputTransformer blanket impl
- FR5: Epic 1 - Remove AgentRuntime deprecated fields
- FR6: Epic 1 - Remove all allow(deprecated) annotations
- FR7: Epic 1 - Delete source files
- FR8: Epic 2 - Audit pub items
- FR9: Epic 1 (Story 1.3) - Clean prelude of deprecated items
- FR10: Epic 2 - Add commonly-used items to prelude
- FR11: Epic 1 - Remove module declarations
- FR12: Epic 1 - Remove deprecated re-exports
- FR13: Epic 2 - Actionable builder error messages
- FR14: Epic 2 - Standardized error format
- FR15: Epic 1 - Update doc comments (ContextStrategy)
- FR16: Epic 1 - Update doc comments (OutputProcessor)
- FR17: Epic 1 - Remove module-level docs for deleted modules
- FR18: Epic 3 - Create migration guide
- FR19: Epic 3 - Include search-and-replace patterns
- FR20: Epic 3 - Include before/after code examples
- FR21: Epic 3 - Verify all examples compile
- FR22: Epic 3 - Verify all tests pass
- FR23: Epic 3 - Verify CI pipeline

## Epic List

### Epic 1: Deprecated Type Removal
Remove all deprecated traits (`ContextStrategy`, `OutputProcessor`), their implementations, blanket impls, and all associated `#[allow(deprecated)]` annotations. After this epic, zero deprecated items remain in the codebase.
**FRs covered:** FR1, FR2, FR3, FR4, FR5, FR6, FR7, FR9, FR11, FR12, FR15, FR16, FR17

### Epic 2: API Audit & Polish
Audit the public API surface, clean up the prelude, and improve error messages for better developer experience.
**FRs covered:** FR8, FR10, FR13, FR14

### Epic 3: Migration Guide & Verification
Create comprehensive migration documentation and verify zero regressions across all examples and tests.
**FRs covered:** FR18, FR19, FR20, FR21, FR22, FR23

---

## Epic 1: Deprecated Type Removal

### Story 1.1: Remove Blanket Implementations

As a framework maintainer,
I want to remove the bridge blanket impls that convert deprecated traits into new traits,
So that the codebase has no hidden compatibility layers.

**Acceptance Criteria:**

**Given** the file `crates/traitclaw-core/src/traits/context_manager.rs` contains a blanket impl `impl<T: ContextStrategy> ContextManager for T`
**When** I remove the blanket impl block (lines ~82-95) and associated `#[allow(deprecated)]` import
**Then** the blanket impl no longer exists
**And** the test `test_blanket_impl_delegates_to_context_strategy` is also removed
**And** the `#[allow(deprecated)] use crate::traits::context_strategy::ContextStrategy;` import is removed
**And** the module-level doc referencing `ContextStrategy` is updated

**Given** the file `crates/traitclaw-core/src/traits/output_transformer.rs` contains a blanket impl `impl<T: OutputProcessor> OutputTransformer for T`
**When** I remove the blanket impl block (lines ~82-89) and associated `#[allow(deprecated)]` import
**Then** the blanket impl no longer exists
**And** the test `test_blanket_impl_delegates_to_output_processor` is also removed
**And** the `#[allow(deprecated)] use crate::traits::output_processor::OutputProcessor;` import is removed
**And** the module-level doc referencing `OutputProcessor` is updated

**Given** all blanket impls are removed
**When** I run `cargo check --workspace`
**Then** the workspace compiles (because nothing depends on the blanket impls externally)

---

### Story 1.2: Remove AgentRuntime Deprecated Fields

As a framework maintainer,
I want to remove the deprecated `context_strategy` and `output_processor` fields from `AgentRuntime`,
So that custom strategy authors see only current, relevant fields.

**Acceptance Criteria:**

**Given** the file `crates/traitclaw-core/src/traits/strategy.rs` defines `AgentRuntime` with 14 pub fields
**When** I remove the `context_strategy: Arc<dyn ContextStrategy>` field
**And** I remove the `output_processor: Arc<dyn OutputProcessor>` field
**And** I remove the `#[allow(deprecated)]` imports for `ContextStrategy` and `OutputProcessor`
**And** I remove the `#[allow(deprecated)]` on the struct definition
**Then** `AgentRuntime` has 12 pub fields
**And** no deprecated imports remain in `strategy.rs`

**Given** `AgentRuntime` fields are removed
**When** I update all code that constructs `AgentRuntime` in these files:
  - `crates/traitclaw-core/src/default_strategy.rs`
  - `crates/traitclaw-core/src/agent.rs`
  - `crates/traitclaw-test-utils/src/runtime.rs`
  - `crates/traitclaw-strategies/src/**` (ReAct, CoT, MCTS strategies that construct `AgentRuntime`)
**Then** the construction sites no longer populate `context_strategy` or `output_processor`
**And** the `#[allow(deprecated)]` annotations at those sites are removed
**And** `cargo check --workspace` compiles

---

### Story 1.3: Remove Deprecated Re-exports and Module Declarations

As a framework maintainer,
I want to remove all deprecated type re-exports from `lib.rs` and the prelude,
So that users importing from `traitclaw_core` see only current types.

**Acceptance Criteria:**

**Given** `crates/traitclaw-core/src/lib.rs` re-exports `ContextStrategy`, `NoopContextStrategy`, `SlidingWindowStrategy`, `ChainProcessor`, `NoopProcessor`, `OutputProcessor`, `TruncateProcessor` with `#[allow(deprecated)]`
**When** I remove all these re-exports (lines ~51-63)
**And** I remove the `pub mod context_strategy` and `pub mod output_processor` from the `traits` module declaration in `lib.rs`
**Then** the deprecated types are no longer accessible via `traitclaw_core::{ContextStrategy, OutputProcessor, ...}`

**Given** the prelude module contains deprecated re-exports
**When** I remove `ContextStrategy`, `NoopContextStrategy`, `SlidingWindowStrategy`, `OutputProcessor`, `TruncateProcessor` from the prelude
**Then** `traitclaw_core::prelude::*` no longer includes deprecated types

**Given** all re-exports and module declarations are removed
**When** I run `cargo check --workspace`
**Then** the workspace compiles (no examples use deprecated types directly)

---

### Story 1.4: Delete Deprecated Source Files and Final Cleanup

As a framework maintainer,
I want to delete the deprecated trait source files and remove all remaining `#[allow(deprecated)]` annotations,
So that zero deprecated code remains in the codebase.

**Acceptance Criteria:**

**Given** all references to `ContextStrategy` and `OutputProcessor` have been removed in Stories 1.1-1.3
**When** I delete `crates/traitclaw-core/src/traits/context_strategy.rs` (188 lines)
**And** I delete `crates/traitclaw-core/src/traits/output_processor.rs` (159 lines)
**Then** the files no longer exist on disk

**Given** the source files are deleted
**When** I run `grep -rn "allow(deprecated)" crates/ --include="*.rs"`
**Then** zero results are returned

**Given** all deprecated code is removed
**When** I run `grep -rn "#\[deprecated" crates/ --include="*.rs"`
**Then** zero results are returned

**Given** all cleanup is complete
**When** I run `cargo test --workspace`
**Then** all remaining tests pass (658+ minus the deleted blanket impl tests)
**And** `cargo clippy --workspace --all-targets --all-features -- -D warnings` passes with zero warnings

---

## Epic 2: API Audit & Polish

### Story 2.1: Prelude Enrichment

As an agent developer,
I want commonly-used types available in the prelude,
So that `use traitclaw::prelude::*` gives me everything I need for typical usage.

**Acceptance Criteria:**

**Given** the prelude in `crates/traitclaw-core/src/lib.rs` is missing common types
**When** I add `CompressedMemory` to the prelude
**And** I add `RetryConfig` and `RetryProvider` to the prelude
**And** I add `DynamicRegistry` to the prelude
**Then** these types are accessible via `traitclaw_core::prelude::*`

**Given** the prelude is updated
**When** I review the complete prelude contents
**Then** every item in the prelude is non-deprecated and commonly used
**And** `cargo doc --workspace --no-deps` shows correct prelude documentation

---

### Story 2.2: Builder Error Message Improvement

As an agent developer,
I want clear, actionable error messages when I misconfigure the AgentBuilder,
So that I can quickly identify and fix configuration issues.

**Acceptance Criteria:**

**Given** `AgentBuilder::build()` is called without setting a provider
**When** the build validation runs
**Then** the error message is `"AgentBuilder: no provider configured. Use .provider(my_provider) before .build()"`

**Given** `AgentBuilder::build()` encounters any validation error
**When** the error is returned
**Then** the message follows the format: `"[Component]: [what happened]. Use .[method]() to fix."`

**Given** error messages are updated
**When** I run `cargo test --workspace`
**Then** any tests asserting on error message content are updated to match the new format
**And** all tests pass

---

### Story 2.3: Public API Visibility Audit

As a framework maintainer,
I want every `pub` item reviewed for appropriate visibility,
So that the API surface is clean and intentional before v1.0 freeze.

**Acceptance Criteria:**

**Given** `traitclaw-core` has `pub mod` declarations for all modules
**When** I audit each module
**Then** no module exposes internal implementation details unnecessarily
**And** `pub(crate) mod streaming` remains correctly scoped

**Given** all `pub` items are audited
**When** I run `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps`
**Then** documentation builds with zero warnings
**And** no broken doc links exist

---

## Epic 3: Migration Guide & Verification

### Story 3.1: Create Migration Guide

As a framework user upgrading from v0.8.0,
I want a clear migration guide documenting all breaking changes,
So that I can upgrade my project in under 5 minutes.

**Acceptance Criteria:**

**Given** I create `docs/migration-v0.8-to-v0.9.md`
**When** a user reads the guide
**Then** it lists all breaking changes:
  - `ContextStrategy` → `ContextManager` (trait rename)
  - `OutputProcessor` → `OutputTransformer` (trait rename)
  - `AgentRuntime.context_strategy` field removed
  - `AgentRuntime.output_processor` field removed
  - Prelude changes (removed deprecated, added new)

**And** each breaking change includes:
  - Search pattern (what to find in their code)
  - Replace pattern (what to replace with)
  - Before/after code example

**And** the guide confirms method signatures are nearly identical:
  - `ContextStrategy::prepare()` (sync) → `ContextManager::prepare()` (async)
  - `OutputProcessor::process(output)` → `OutputTransformer::transform(output, tool_name, state)` (async, extra params)

---

### Story 3.2: Full Verification & Release Readiness

As a framework maintainer,
I want to verify zero regressions before tagging v0.9.0,
So that the release is production-ready.

**Acceptance Criteria:**

**Given** all changes from Epics 1-2 are complete
**When** I run the full CI suite:
```
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
```
**Then** all 4 checks pass with zero errors and zero warnings

**Given** the CI suite passes
**When** I check all 26 examples:
```
for dir in examples/*/; do cargo check --manifest-path "${dir}Cargo.toml"; done
```
**Then** all 26 examples compile

**Given** all verification passes
**When** I run the deprecated item audit:
```
grep -rn "#\[deprecated" crates/ --include="*.rs" | wc -l
grep -rn "allow(deprecated)" crates/ --include="*.rs" | wc -l
```
**Then** both counts are 0

---

## Summary

| Epic | Stories | FRs Covered | Breaking Changes |
|------|---------|-------------|------------------|
| Epic 1: Deprecated Type Removal | 4 | FR1-7, FR9, FR11-12, FR15-17 | ⚠️ Yes |
| Epic 2: API Audit & Polish | 3 | FR8, FR10, FR13-14 | No |
| Epic 3: Migration Guide & Verification | 2 | FR18-23 | No |
| **Total** | **9** | **23 FRs** | |

**Estimated sprint size:** ~1 sprint (9 stories, mostly mechanical removal work)
**Dependency order:** Epic 1 → Epic 2 → Epic 3 (strictly sequential)
