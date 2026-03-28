# Story 4.1: Dry-Run Verification

Status: done

## Story

As a **framework maintainer**,
I want to verify all 14 crates pass `cargo publish --dry-run`,
so that I can catch any issues before the real publish.

## Acceptance Criteria

1. `cargo publish --dry-run -p <crate>` succeeds (exit code 0) for all 14 crates in dependency order
2. `cargo package --list -p <crate>` shows no leaked test/dev files
3. Each package is < 1MB in size (NFR3)
4. All CI checks pass: fmt, clippy, test, docs

## Tasks / Subtasks

- [ ] Task 1 (AC: #4): Run CI checks
  - [ ] `cargo fmt --all --check`
  - [ ] `cargo clippy --workspace --all-targets -- -D warnings`
  - [ ] `cargo test --workspace`
  - [ ] `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps`
- [ ] Task 2 (AC: #1): Dry-run all crates in order
  - [ ] Level 0: `cargo publish --dry-run -p traitclaw-core`
  - [ ] Level 1: `cargo publish --dry-run -p traitclaw-test-utils`
  - [ ] Level 1: `cargo publish --dry-run -p traitclaw-macros`
  - [ ] Level 1: `cargo publish --dry-run -p traitclaw-openai-compat`
  - [ ] Level 1: `cargo publish --dry-run -p traitclaw-anthropic`
  - [ ] Level 1: `cargo publish --dry-run -p traitclaw-eval`
  - [ ] Level 1: `cargo publish --dry-run -p traitclaw-mcp`
  - [ ] Level 1: `cargo publish --dry-run -p traitclaw-memory-sqlite`
  - [ ] Level 1: `cargo publish --dry-run -p traitclaw-rag`
  - [ ] Level 1: `cargo publish --dry-run -p traitclaw-steering`
  - [ ] Level 1: `cargo publish --dry-run -p traitclaw-team`
  - [ ] Level 2: `cargo publish --dry-run -p traitclaw-openai`
  - [ ] Level 2: `cargo publish --dry-run -p traitclaw-strategies`
  - [ ] Level 3: `cargo publish --dry-run -p traitclaw`
- [ ] Task 3 (AC: #2-3): Check package contents and size
  - [ ] Run `cargo package --list -p <crate>` for each — verify no tests/ or benches/ leaked
  - [ ] If leaks found, add `exclude` to Cargo.toml
  - [ ] Check package sizes

## Dev Notes

- Dry-run does NOT require crates.io auth
- If dry-run fails with "aborting upload due to registry...", that's expected for `--dry-run` — check the preceding output for actual errors
- Common failure: missing `description`, `license`, or `readme` fields → should be fixed by Story 2.1
- If file leaks found: add to Cargo.toml `[package]` section:
  ```toml
  exclude = ["tests/", "benches/"]
  ```

### References

- [Source: architecture-v1.0.0.md#AD2 — Publish Strategy]
- [Source: prd-v1.0.0.md#FR12]
