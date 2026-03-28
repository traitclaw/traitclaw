# Story 2.1: Create GitHub Actions CI Workflow

Status: ready-for-dev

## Story

As a framework contributor,
I want a GitHub Actions CI workflow that runs fmt, clippy, test, and doc checks on every push and PR,
so that code quality is verified automatically before merge.

## Acceptance Criteria

1. **Given** a push or PR to any branch, **When** GitHub Actions triggers `.github/workflows/ci.yml`, **Then** `cargo fmt --all --check` runs and fails on formatting violations
2. **And** `cargo clippy --workspace --all-targets -- -D warnings` runs and fails on lint warnings
3. **And** `cargo test --workspace` runs the full test suite
4. **And** `cargo doc --workspace --no-deps` verifies documentation builds with `-D warnings`
5. **And** the pipeline uses stable Rust toolchain only (no nightly)
6. **And** total pipeline completes in ≤ 10 minutes on GitHub Actions standard runner

## Tasks / Subtasks

- [ ] Task 1: Update existing `.github/workflows/ci.yml` (AC: #1-6)
  - [ ] Update `clippy` job: add `--workspace` flag and `--all-features` to match architecture spec
  - [ ] Add `docs` job: `cargo doc --workspace --no-deps` with `RUSTDOCFLAGS: "-D warnings"` env
  - [ ] Migrate all cache actions from `actions/cache@v4` to `Swatinem/rust-cache@v2` (more efficient, auto-keyed)
  - [ ] Add `CARGO_TERM_COLOR: always` env at workflow level (already present)
  - [ ] Ensure all 4 jobs run in parallel (independent `runs-on` blocks, no `needs` dependencies)
- [ ] Task 2: Verify pipeline correctness locally (AC: #1-4)
  - [ ] Run `cargo fmt --all --check` locally — must pass
  - [ ] Run `cargo clippy --workspace --all-targets --all-features -- -D warnings` locally — must pass
  - [ ] Run `cargo test --workspace` locally — all 600+ tests pass
  - [ ] Run `cargo doc --workspace --no-deps` with `RUSTDOCFLAGS="-D warnings"` — must pass
- [ ] Task 3: Validate action versions and Rust toolchain (AC: #5)
  - [ ] Confirm `dtolnay/rust-toolchain@stable` — no nightly
  - [ ] Confirm `actions/checkout@v4` pinned
  - [ ] Confirm `Swatinem/rust-cache@v2` pinned

## Dev Notes

### ⚠️ CRITICAL: Existing CI File

A CI file **already exists** at `.github/workflows/ci.yml` with 3 jobs:
- `test` — runs `cargo test --workspace`
- `clippy` — runs `cargo clippy --all-targets -- -D warnings` (⚠️ missing `--workspace` and `--all-features`)
- `fmt` — runs `cargo fmt --all -- --check`

**What's missing vs. architecture spec (Decision 6):**
1. **`docs` job** — needs `cargo doc --workspace --no-deps` with `RUSTDOCFLAGS: "-D warnings"`
2. **`clippy` flags** — needs `--workspace` and `--all-features` added
3. **Cache strategy** — currently uses `actions/cache@v4` with manual paths; architecture spec prefers `Swatinem/rust-cache@v2` (auto-keyed, more efficient)

### Architecture Reference

From [architecture-v0.8.0.md — Decision 6]:

```yaml
# Target structure — 4 parallel jobs
jobs:
  fmt:        # cargo fmt --all --check
  clippy:     # cargo clippy --all-targets --all-features -- -D warnings
  test:       # cargo test --workspace --all-features
  docs:       # cargo doc --workspace --no-deps (RUSTDOCFLAGS: "-D warnings")
```

Key rules:
- `Swatinem/rust-cache@v2` for all jobs except `fmt` (formatting is fast)
- `dtolnay/rust-toolchain@stable` — stable only, never nightly (NFR9)
- `CARGO_TERM_COLOR: always` at workflow level
- All 4 jobs run in parallel — no `needs` dependencies

### Anti-Patterns to Avoid

| ❌ Don't | ✅ Do |
|----------|------|
| Use `actions/cache@v4` with manual paths | Use `Swatinem/rust-cache@v2` (auto-keyed) |
| Use `@nightly` toolchain | Use `@stable` only |
| Add `needs:` between jobs | Keep all 4 jobs parallel |
| Use `--all-features` on fmt | Only add `--all-features` to clippy and test |
| Add coverage generation to this story | Coverage is Story 2.2 |

### Project Structure Notes

- File: `.github/workflows/ci.yml` (MODIFY existing, do not create new)
- No other files need modification for this story
- No workspace `Cargo.toml` changes needed

### Previous Story Intelligence

Epic 1 (Shared Test Infrastructure) is complete. All 5 stories done:
- `traitclaw-test-utils` crate created with MockProvider, MockMemory, EchoTool, FailTool, DenyGuard, DangerousTool
- All workspace mocks consolidated — zero duplicates
- All tests migrated to integration tests under `tests/`
- Workspace stable: 600+ tests passing

### References

- [Source: _bmad-output/planning-artifacts/architecture-v0.8.0.md#Decision 6: CI Pipeline Structure]
- [Source: _bmad-output/planning-artifacts/epics-v0.8.0.md#Story 2.1]
- [Source: .github/workflows/ci.yml — existing file]

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
