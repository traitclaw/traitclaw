---
stepsCompleted:
  - step-01-validate-prerequisites
  - step-02-design-epics
  - step-03-create-stories
  - step-04-final-validation
inputDocuments:
  - prd-v1.0.0.md
  - architecture-v1.0.0.md
---

# TraitClaw v1.0.0 — Epic Breakdown

## Overview

This document provides the complete epic and story breakdown for TraitClaw v1.0.0 "Production Ready", decomposing the PRD and Architecture requirements into implementable stories for the stabilization and packaging release.

## Requirements Inventory

### Functional Requirements

- FR1: Workspace version bumped from `0.6.0` to `1.0.0`
- FR2: All 14 crate versions synchronized via `version.workspace = true`
- FR3: No public API changes — all signatures remain identical to v0.9.0
- FR4: README.md overhauled with badges, quickstart, feature matrix, architecture, "Why TraitClaw?"
- FR5: CHANGELOG.md created covering v0.1.0–v1.0.0 (Keep a Changelog format)
- FR6: LICENSE-MIT and LICENSE-APACHE files created at repository root
- FR7: CONTRIBUTING.md created with PR process, code style, testing requirements
- FR8: All 14 `Cargo.toml` files include description, keywords, categories, repository, readme, documentation
- FR9: Each crate includes `[package.metadata.docs.rs]` section with `all-features = true`
- FR10: Module-level doc comments provide meaningful overview for each crate
- FR11: Crate-level rustdoc includes link to README and repository
- FR12: `cargo publish --dry-run` succeeds for all 14 crates
- FR13: All 14 crates published in correct dependency order (4 levels)
- FR14: Published crate README renders correctly on crates.io
- FR15: Semver policy documented: API frozen until v2.0
- FR16: MSRV policy documented: Rust 1.75, bumps only on minor versions
- FR17: Deprecation policy documented: 2 minor versions notice before removal
- FR18: README includes roadmap section covering v1.1–v1.3
- FR19: Each future version has clear scope and codename

### Non-Functional Requirements

- NFR1: Build time unchanged vs v0.9.0 (no new code)
- NFR2: All public items have doc comments (`#![deny(missing_docs)]`)
- NFR3: No unnecessary files in published crate (< 1MB per crate)
- NFR4: Dual license MIT/Apache-2.0 (Rust ecosystem standard)
- NFR5: All existing CI checks continue to pass

### Additional Requirements (Architecture)

- AD1: Workspace inheritance pattern for all shared Cargo.toml fields
- AD2: Publish in strict topological dependency order (4 levels)
- AD3: Physical LICENSE files at repository root
- AD4: Sub-crate README template pointing to main project
- AD5: Standardized Cargo.toml metadata template across all crates
- AD6: Keep a Changelog format for CHANGELOG.md
- AD7: Semver policy including deprecation rules

### UX Design Requirements

N/A — this is a library/framework crate with no UI.

### FR Coverage Map

| FR | Epic | Story |
|----|------|-------|
| FR1, FR2, FR3 | Epic 1 | 1.1 |
| FR6 | Epic 1 | 1.2 |
| FR8, FR9, FR10, FR11 | Epic 2 | 2.1 |
| (sub-crate READMEs) | Epic 2 | 2.2 |
| FR4, FR18, FR19 | Epic 3 | 3.1 |
| FR5 | Epic 3 | 3.2 |
| FR7 | Epic 3 | 3.3 |
| FR15, FR16, FR17 | Epic 3 | 3.4 |
| FR12 | Epic 4 | 4.1 |
| FR13, FR14 | Epic 4 | 4.2 |
| NFR1-5 | Epic 4 | 4.1, 4.2 |

## Epic List

1. **Epic 1: Version Bump & License Setup** — Foundation changes required before all other work
2. **Epic 2: Cargo.toml Metadata & docs.rs Config** — Make all 14 crates publish-ready
3. **Epic 3: Documentation & Ecosystem Files** — README, CHANGELOG, CONTRIBUTING, policies
4. **Epic 4: Publish Verification & Release** — Dry-run, publish, and post-publish verification

---

## Epic 1: Version Bump & License Setup

**Goal:** Establish the v1.0.0 version across the workspace and create physical license files required for crates.io publication.

**Dependencies:** None — this is the foundation epic.

### Story 1.1: Bump Workspace Version to 1.0.0

As a **framework maintainer**,
I want the workspace version bumped from `0.6.0` to `1.0.0`,
So that all 14 crates are synchronized at the stable release version.

**Acceptance Criteria:**

**Given** the root `Cargo.toml` has `version = "0.6.0"` under `[workspace.package]`
**When** I change it to `version = "1.0.0"`
**Then** `cargo metadata` reports all 14 workspace crates at version `1.0.0`
**And** `cargo build --workspace` compiles successfully
**And** `cargo test --workspace` passes all 649+ tests
**And** no other code changes are introduced (FR3)

### Story 1.2: Create License Files

As a **framework maintainer**,
I want physical `LICENSE-MIT` and `LICENSE-APACHE` files at the repository root,
So that crates.io packages include proper license text and enterprise users have legal clarity.

**Acceptance Criteria:**

**Given** the repository root has no LICENSE files
**When** I create `LICENSE-MIT` with the MIT license text (copyright: TraitClaw Contributors)
**And** I create `LICENSE-APACHE` with the Apache License 2.0 text
**Then** both files exist at the repo root
**And** `cargo publish --dry-run -p traitclaw-core` does not warn about missing license files
**And** the `license` field in workspace Cargo.toml remains `"MIT OR Apache-2.0"`

---

## Epic 2: Cargo.toml Metadata & docs.rs Config

**Goal:** Ensure all 14 crates have complete Cargo.toml metadata fields required for crates.io publication and docs.rs rendering.

**Dependencies:** Epic 1 (version must be 1.0.0 first).

### Story 2.1: Add Cargo.toml Metadata to All Crates

As a **framework maintainer**,
I want all 14 `Cargo.toml` files to include complete publication metadata,
So that each crate displays correct information on crates.io and docs.rs.

**Acceptance Criteria:**

**Given** each crate's `Cargo.toml` currently has `name`, `version.workspace`, `edition.workspace`, `rust-version.workspace`, `license.workspace`, and `description`
**When** I add `keywords`, `categories`, `readme`, `documentation`, and `[package.metadata.docs.rs]` to each crate
**Then** every crate has:
- `keywords` — up to 5 relevant terms including `"ai"`, `"agent"`, `"llm"`
- `categories` — from official crates.io taxonomy (e.g., `"api-bindings"`, `"asynchronous"`)
- `readme = "README.md"`
- `documentation` — points to docs.rs URL
- `repository.workspace = true`
- `[package.metadata.docs.rs]` with `all-features = true` and `rustdoc-args = ["--cfg", "docsrs"]`
**And** `cargo publish --dry-run -p <crate>` succeeds for all 14 crates
**And** every crate's `lib.rs` has a `//!` module-level doc comment describing the crate's purpose (FR10)
**And** every crate's `lib.rs` includes `#![deny(missing_docs)]` (NFR2)
**And** `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps` passes with zero warnings (FR11)

**Technical Notes:**
- Use workspace inheritance for `repository`
- Meta-crate `traitclaw` should use categories `["development-tools"]`
- Provider crates should include keyword `"openai"`, `"anthropic"` as relevant

### Story 2.2: Create Sub-Crate README Files

As a **new user discovering a sub-crate on crates.io**,
I want each sub-crate to have a README explaining what it is and linking to the main project,
So that I can find the full documentation and examples.

**Acceptance Criteria:**

**Given** most sub-crates have no `README.md` in their directory
**When** I create a `README.md` for each of the 14 crates
**Then** each README contains:
- Crate name and one-line description matching `Cargo.toml` description
- Link to main TraitClaw crates.io page
- Link to docs.rs for this specific crate
- Link to examples in the GitHub repository
- License section referencing root LICENSE files
**And** the meta-crate `traitclaw` README is a copy of the root README
**And** `cargo package --list -p <crate>` includes `README.md` for all crates

---

## Epic 3: Documentation & Ecosystem Files

**Goal:** Create and overhaul all ecosystem documentation files: README, CHANGELOG, CONTRIBUTING, and policy docs.

**Dependencies:** Epic 1, Epic 2 (metadata needed for README badges).

### Story 3.1: Overhaul Root README

As a **developer evaluating TraitClaw for the first time**,
I want a professional README with badges, quickstart, and feature overview,
So that I can quickly understand what TraitClaw offers and how to get started.

**Acceptance Criteria:**

**Given** the current README exists but lacks crates.io badges and comprehensive feature matrix
**When** I overhaul the README
**Then** it contains:
- **Badges:** crates.io version, docs.rs, CI status, license, MSRV
- **One-liner:** "A Rust AI Agent Framework — Simple by default, powerful when needed"
- **Why TraitClaw?** section with key differentiators
- **Quickstart** code block (5-line agent creation)
- **Feature Matrix** table covering all capabilities with ✅/🔜 status
- **Architecture Overview** showing layered trait system
- **Examples** section listing all 23+ examples with descriptions
- **Roadmap** section with v1.1–v1.3 plans and codenames (FR18, FR19)
- **License** section stating dual MIT/Apache-2.0
**And** the README renders correctly on GitHub
**And** the README passes markdown lint validation (no broken links, no syntax issues)
**And** the README renders correctly when used as crates.io README (no broken relative links)

### Story 3.2: Create CHANGELOG.md

As a **user upgrading between versions**,
I want a comprehensive changelog covering all versions from v0.1 to v1.0,
So that I can understand what changed in each release.

**Acceptance Criteria:**

**Given** no CHANGELOG.md exists
**When** I create `CHANGELOG.md` at the repository root
**Then** it follows [Keep a Changelog](https://keepachangelog.com/) format
**And** it includes entries for: v1.0.0, v0.9.0, v0.8.0, v0.7.0, v0.6.0, v0.5.0, v0.4.0, v0.3.0, v0.2.0, v0.1.0
**And** each entry uses sections: Added, Changed, Removed, Fixed (as applicable)
**And** v0.9.0 entry mentions breaking changes (ContextStrategy/OutputProcessor removed)
**And** v1.0.0 entry notes API freeze and ecosystem packaging

### Story 3.3: Create CONTRIBUTING.md

As an **open-source contributor**,
I want clear contribution guidelines,
So that I know how to submit PRs, what code style to follow, and how to test.

**Acceptance Criteria:**

**Given** no CONTRIBUTING.md exists
**When** I create `CONTRIBUTING.md` at the repository root
**Then** it includes:
- How to file issues (bug reports, feature requests)
- PR process (fork, branch, PR, CI checks)
- Code style guide (cargo fmt, cargo clippy rules)
- Testing requirements (run `cargo test --workspace`)
- Crate structure guide (which crate to modify for what)
- Commit message convention (conventional commits)
**And** the document is concise (< 200 lines)

### Story 3.4: Document Semver, MSRV, and Deprecation Policies

As a **user depending on TraitClaw in production**,
I want clear stability policies,
So that I know when breaking changes can happen and how long deprecated APIs will be supported.

**Acceptance Criteria:**

**Given** no formal policy documentation exists
**When** I add policy sections to the README (or a separate `docs/policies.md`)
**Then** it documents:
- **Semver:** API frozen at 1.0, additions in minor, breaks only in major
- **MSRV:** 1.75, bumped only in minor releases, documented in CHANGELOG
- **Deprecation:** 2 minor versions notice; deprecated in v1.x removes earliest in v2.0
**And** the policies are consistent with Architecture Decision AD7

---

## Epic 4: Publish Verification & Release

**Goal:** Verify all crates are publish-ready, publish to crates.io, and perform post-publish verification.

**Dependencies:** Epic 1, Epic 2, Epic 3 (all preparation complete).

### Story 4.1: Dry-Run Verification

As a **framework maintainer**,
I want to verify all 14 crates pass `cargo publish --dry-run`,
So that I can catch any issues before the real publish.

**Acceptance Criteria:**

**Given** all metadata, READMEs, and licenses are in place
**When** I run `cargo publish --dry-run -p <crate>` for all 14 crates in dependency order
**Then** all 14 dry-runs succeed with exit code 0
**And** `cargo package --list -p <crate>` shows no leaked test/dev files
**And** each package is < 1MB in size (NFR3)
**And** all CI checks pass: fmt, clippy, test, docs

**Technical Notes:**
- Run in topological order: core → test-utils → macros → providers → features → meta
- Add `exclude = ["tests/", "benches/"]` to Cargo.toml if dev files are leaked

### Story 4.2: Publish to crates.io and Post-Publish Verification

As a **framework maintainer**,
I want all 14 crates published to crates.io in correct dependency order,
So that users can install TraitClaw via `cargo add traitclaw`.

**Acceptance Criteria:**

**Given** all dry-runs pass and CI is green
**When** I publish all 14 crates in dependency order with 30-second delays between publishes
**Then** all 14 crates are live on crates.io
**And** `cargo add traitclaw` works from a fresh project
**And** `cargo doc` renders correctly after adding the dependency
**And** README renders correctly on each crate's crates.io page
**And** docs.rs builds succeed for all crates

**Technical Notes:**
- Publish order (from Architecture AD2):
  1. `traitclaw-core`
  2. `traitclaw-test-utils`, `traitclaw-macros`, `traitclaw-openai-compat`, `traitclaw-anthropic`, `traitclaw-eval`, `traitclaw-mcp`, `traitclaw-memory-sqlite`, `traitclaw-rag`, `traitclaw-steering`, `traitclaw-team`
  3. `traitclaw-openai`, `traitclaw-strategies`
  4. `traitclaw`
- Sleep 30s between each publish for crates.io index propagation
- Create publish script at `scripts/publish.sh` for repeatability

### Post-Publish Checklist

After all crates are published and verified:

- [ ] Create git tag: `git tag -a v1.0.0 -m "TraitClaw v1.0.0 — Production Ready"`
- [ ] Push tag: `git push origin v1.0.0`
- [ ] Create GitHub Release from tag with CHANGELOG entry as body
- [ ] Verify docs.rs builds complete for all 14 crates
- [ ] Announce release (if applicable)

---

## Summary

| Epic | Stories | Est. Effort |
|------|---------|-------------|
| **1. Version Bump & License** | 2 stories | 15 min |
| **2. Cargo.toml Metadata & READMEs** | 2 stories | 1 hour |
| **3. Documentation & Ecosystem** | 4 stories | 2.5 hours |
| **4. Publish Verification & Release** | 2 stories | 45 min |
| **Total** | **10 stories** | **~4.5 hours** |

### Implementation Order

```
Epic 1 (1.1 → 1.2)
    ↓
Epic 2 (2.1 → 2.2)
    ↓
Epic 3 (3.1 → 3.2 → 3.3 → 3.4) [3.2–3.4 can be parallel]
    ↓
Epic 4 (4.1 → 4.2)
```
