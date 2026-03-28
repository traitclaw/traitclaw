# Story 2.1: Add Cargo.toml Metadata to All Crates

Status: done

## Story

As a **framework maintainer**,
I want all 14 `Cargo.toml` files to include complete publication metadata,
so that each crate displays correct information on crates.io and docs.rs.

## Acceptance Criteria

1. Every crate has `keywords` (up to 5 terms including `"ai"`, `"agent"`, `"llm"`)
2. Every crate has `categories` from official crates.io taxonomy
3. Every crate has `readme = "README.md"`
4. Every crate has `documentation` pointing to docs.rs URL
5. Every crate has `repository.workspace = true`
6. Every crate has `[package.metadata.docs.rs]` with `all-features = true` and `rustdoc-args = ["--cfg", "docsrs"]`
7. `cargo publish --dry-run -p <crate>` succeeds for all 14 crates
8. Every crate's `lib.rs` has a `//!` module-level doc comment describing purpose (FR10)
9. Every crate's `lib.rs` includes `#![deny(missing_docs)]` (NFR2)
10. `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps` passes with zero warnings (FR11)

## Tasks / Subtasks

- [ ] Task 1 (AC: #5): Add `repository.workspace = true` to workspace Cargo.toml if not present
- [ ] Task 2 (AC: #1-6): Update each crate's Cargo.toml with metadata
  - [ ] `traitclaw-core` — keywords: `["ai", "agent", "llm", "framework", "traits"]`, categories: `["api-bindings", "asynchronous"]`
  - [ ] `traitclaw-macros` — keywords: `["ai", "agent", "llm", "derive", "macro"]`, categories: `["development-tools::procedural-macro-helpers"]`
  - [ ] `traitclaw-test-utils` — keywords: `["ai", "agent", "testing", "mock", "test-utils"]`, categories: `["development-tools::testing"]`
  - [ ] `traitclaw-openai` — keywords: `["ai", "agent", "openai", "gpt", "llm"]`, categories: `["api-bindings", "asynchronous"]`
  - [ ] `traitclaw-anthropic` — keywords: `["ai", "agent", "anthropic", "claude", "llm"]`, categories: `["api-bindings", "asynchronous"]`
  - [ ] `traitclaw-openai-compat` — keywords: `["ai", "agent", "openai", "ollama", "llm"]`, categories: `["api-bindings", "asynchronous"]`
  - [ ] `traitclaw-steering` — keywords: `["ai", "agent", "guardrails", "steering", "safety"]`, categories: `["api-bindings"]`
  - [ ] `traitclaw-memory-sqlite` — keywords: `["ai", "agent", "memory", "sqlite", "persistence"]`, categories: `["database"]`
  - [ ] `traitclaw-mcp` — keywords: `["ai", "agent", "mcp", "tools", "protocol"]`, categories: `["api-bindings"]`
  - [ ] `traitclaw-rag` — keywords: `["ai", "agent", "rag", "retrieval", "embedding"]`, categories: `["api-bindings"]`
  - [ ] `traitclaw-team` — keywords: `["ai", "agent", "multi-agent", "team", "orchestration"]`, categories: `["api-bindings", "asynchronous"]`
  - [ ] `traitclaw-eval` — keywords: `["ai", "agent", "evaluation", "benchmark", "testing"]`, categories: `["development-tools::testing"]`
  - [ ] `traitclaw-strategies` — keywords: `["ai", "agent", "react", "mcts", "reasoning"]`, categories: `["api-bindings", "asynchronous"]`
  - [ ] `traitclaw` — keywords: `["ai", "agent", "llm", "framework", "rust"]`, categories: `["development-tools"]`
- [ ] Task 3 (AC: #8-9): Verify module-level docs
  - [ ] Verify each crate's `lib.rs` has `//!` doc comments
  - [ ] Verify each crate's `lib.rs` has `#![deny(missing_docs)]`
- [ ] Task 4 (AC: #7, #10): Verification
  - [ ] Run `cargo publish --dry-run -p <crate>` for all 14 crates
  - [ ] Run `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps`

## Dev Notes

- `repository` field should use workspace inheritance — add to `[workspace.package]` if not already there (currently at line 49)
- `documentation` format: `https://docs.rs/traitclaw-{name}`
- Categories must be from https://crates.io/category_slugs — check validity
- `all-features = true` in docs.rs config ensures feature-gated items appear in docs
- Most crates already have `#![deny(missing_docs)]` — verify, don't assume

### Cargo.toml Template Per Crate

```toml
# Add after existing [package] fields:
readme = "README.md"
documentation = "https://docs.rs/traitclaw-{name}"
keywords = [...]
categories = [...]

# Add new section:
[package.metadata.docs.rs]
all-features = true
rustdoc-args = ["--cfg", "docsrs"]
```

### Project Structure Notes

All crate Cargo.toml files are at: `crates/{crate-name}/Cargo.toml`

### References

- [Source: architecture-v1.0.0.md#AD5 — Cargo.toml Metadata Standard]
- [Source: architecture-v1.0.0.md#P1 — Workspace Inheritance Pattern]
- [Source: prd-v1.0.0.md#FR8-FR11 — Metadata requirements]
