# Story 3.3: Create CONTRIBUTING.md

Status: done

## Story

As an **open-source contributor**,
I want clear contribution guidelines,
so that I know how to submit PRs, what code style to follow, and how to test.

## Acceptance Criteria

1. `CONTRIBUTING.md` exists at repository root
2. Includes: how to file issues, PR process, code style guide, testing requirements, crate structure guide, commit convention
3. Document is concise (< 200 lines)

## Tasks / Subtasks

- [ ] Task 1 (AC: #1-3): Create CONTRIBUTING.md
  - [ ] Getting Started section (fork, clone, build)
  - [ ] Filing Issues (bug template, feature request)
  - [ ] PR Process (branch naming, CI checks, review)
  - [ ] Code Style (`cargo fmt`, `cargo clippy -- -D warnings`)
  - [ ] Testing (`cargo test --workspace`, `cargo doc`)
  - [ ] Crate Structure Guide (which crate for what)
  - [ ] Commit Convention (conventional commits: `feat`, `fix`, `docs`, `refactor`)

## Dev Notes

### Crate Structure Guide Content

```
crates/traitclaw-core/     — Core traits? Add it here
crates/traitclaw-macros/    — Proc macros? Here
crates/traitclaw-openai/    — OpenAI provider changes
crates/traitclaw-anthropic/ — Anthropic provider changes
crates/traitclaw-*/         — Feature-specific crates
examples/                   — New example? Add numbered directory
```

### CI Checks to Document

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
```

### References

- [Source: prd-v1.0.0.md#FR7]
- [Source: epics-v1.0.0.md#Story 3.3]
