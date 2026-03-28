# Story 2.2: Add Code Coverage with `cargo-llvm-cov`

Status: ready-for-dev

## Story

As a framework contributor,
I want code coverage reports generated locally and in CI,
so that I can identify untested code and track coverage improvements.

## Acceptance Criteria

1. **Given** `cargo-llvm-cov` is installed locally, **When** I run `cargo llvm-cov --workspace --html`, **Then** an HTML coverage report is generated in `target/llvm-cov/html/`
2. **And** CI workflow includes a `coverage` job that generates and archives the coverage report
3. **And** the coverage job uses `cargo-llvm-cov` with `--workspace` flag
4. **And** the coverage job archives the HTML report as a GitHub Actions artifact
5. **And** the coverage job is **optional** — does not block PR merges (uses `continue-on-error: true` or separate non-required status)
6. **And** coverage baseline is measured and documented in a comment in the CI file

## Tasks / Subtasks

- [ ] Task 1: Add coverage job to `.github/workflows/ci.yml` (AC: #2-5)
  - [ ] Add `coverage` job with `runs-on: ubuntu-latest`
  - [ ] Install `cargo-llvm-cov` via `cargo install cargo-llvm-cov` or use `taiki-e/install-action@cargo-llvm-cov`
  - [ ] Run `cargo llvm-cov --workspace --all-features --html`
  - [ ] Upload `target/llvm-cov/html/` as GitHub Actions artifact using `actions/upload-artifact@v4`
  - [ ] Mark job as optional: `continue-on-error: true`
  - [ ] Use `Swatinem/rust-cache@v2` for build cache
- [ ] Task 2: Verify local execution (AC: #1)
  - [ ] Install `cargo-llvm-cov` locally: `cargo install cargo-llvm-cov`
  - [ ] Run `cargo llvm-cov --workspace --html`
  - [ ] Verify report generated at `target/llvm-cov/html/index.html`
  - [ ] Document current coverage baseline percentage
- [ ] Task 3: Add .gitignore entry (if needed)
  - [ ] Ensure `target/` is already in `.gitignore` (coverage artifacts live there)

## Dev Notes

### ⚠️ CRITICAL: This builds on Story 2.1

Story 2.1 updates the existing CI file to have 4 parallel jobs (fmt, clippy, test, docs). This story adds a 5th **optional** job: `coverage`.

### Architecture Reference

From [architecture-v0.8.0.md — Decision 6]:
> Coverage (`cargo-llvm-cov`) added as **optional 5th job** — non-blocking

### Coverage Job Template

```yaml
coverage:
  name: Coverage
  runs-on: ubuntu-latest
  continue-on-error: true  # Non-blocking — does not fail PRs
  steps:
    - uses: actions/checkout@v4

    - name: Install Rust toolchain
      uses: dtolnay/rust-toolchain@stable

    - name: Install cargo-llvm-cov
      uses: taiki-e/install-action@cargo-llvm-cov

    - uses: Swatinem/rust-cache@v2

    - name: Generate coverage report
      run: cargo llvm-cov --workspace --all-features --html

    - name: Upload coverage report
      uses: actions/upload-artifact@v4
      with:
        name: coverage-report
        path: target/llvm-cov/html/
        retention-days: 14
```

### Key Decisions

| Decision | Rationale |
|----------|-----------|
| `continue-on-error: true` | Coverage is informational, never blocks PRs |
| `taiki-e/install-action` | Faster than `cargo install` (pre-built binaries) |
| `--workspace --all-features` | Measures coverage across all crates including feature-gated code |
| `retention-days: 14` | Keep reports for 2 weeks to track trends |
| `--html` format | Human-readable; lcov can be added later for codecov.io integration |

### Anti-Patterns to Avoid

| ❌ Don't | ✅ Do |
|----------|------|
| Make coverage a required check | Keep `continue-on-error: true` |
| Use `cargo install cargo-llvm-cov` in CI | Use `taiki-e/install-action` (10x faster) |
| Set coverage thresholds that block merges | Document baseline only |
| Use `@nightly` for coverage | `cargo-llvm-cov` works with stable since Rust 1.60+ |
| Add codecov.io / coveralls integration | Deferred — local HTML reports are sufficient for v0.8.0 |

### Previous Story Intelligence

Story 2.1 establishes the CI pipeline pattern with:
- `Swatinem/rust-cache@v2` for caching
- `dtolnay/rust-toolchain@stable` for toolchain
- `CARGO_TERM_COLOR: always` env
- 4 parallel jobs — this adds a 5th

### Project Structure Notes

- File: `.github/workflows/ci.yml` (MODIFY — append coverage job after Story 2.1's 4 jobs)
- No other files need modification
- `target/llvm-cov/` is already covered by `target/` gitignore

### References

- [Source: _bmad-output/planning-artifacts/architecture-v0.8.0.md#Decision 6: CI Pipeline Structure]
- [Source: _bmad-output/planning-artifacts/epics-v0.8.0.md#Story 2.2]
- [Source: https://github.com/taiki-e/cargo-llvm-cov — cargo-llvm-cov docs]

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
