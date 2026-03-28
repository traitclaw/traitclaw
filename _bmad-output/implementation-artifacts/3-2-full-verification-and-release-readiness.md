# Story 3.2: Full Verification and Release Readiness

Status: ready-for-dev

## Story

As a framework maintainer,
I want to verify zero regressions before tagging v0.9.0,
so that the release is production-ready.

## Acceptance Criteria

1. `cargo fmt --all --check` passes
2. `cargo clippy --workspace --all-targets --all-features -- -D warnings` passes
3. `cargo test --workspace` passes (all remaining tests)
4. `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps` passes
5. All 26 examples compile: `for dir in examples/*/; do cargo check --manifest-path "${dir}Cargo.toml"; done`
6. `grep -rn "#[deprecated" crates/` returns 0 results
7. `grep -rn "allow(deprecated)" crates/` returns 0 results

## Tasks / Subtasks

- [ ] Task 1: Run full CI suite (AC: #1, #2, #3, #4)
  - [ ] Run `cargo fmt --all --check`
  - [ ] Run `cargo clippy --workspace --all-targets --all-features -- -D warnings`
  - [ ] Run `cargo test --workspace`
  - [ ] Run `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps`
- [ ] Task 2: Verify all examples (AC: #5)
  - [ ] Run compile check for each example in `examples/` directory
- [ ] Task 3: Verify zero deprecated items (AC: #6, #7)
  - [ ] Run grep for `#[deprecated` — expect 0
  - [ ] Run grep for `allow(deprecated)` — expect 0
- [ ] Task 4: Final audit summary
  - [ ] Record total test count (should be ~650+)
  - [ ] Record example count (should be 26)
  - [ ] Confirm v0.9.0 is release-ready

## Dev Notes

### This is a verification-only story
Do NOT write code in this story. Only run verification commands and document results.
If any check fails, create a fix immediately before marking task complete.

### Expected Results Summary
- Tests: 650+ pass (658 original minus deleted deprecated tests)
- Examples: 26 compile
- Deprecated items: 0
- CI suite: 4/4 pass

### References
- [Source: prd-v0.9.0.md#Success Criteria]
- [Source: epics-v0.9.0.md#Story 3.2]

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

### Change Log
