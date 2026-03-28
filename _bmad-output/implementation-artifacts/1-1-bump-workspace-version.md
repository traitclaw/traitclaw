# Story 1.1: Bump Workspace Version to 1.0.0

Status: review

## Story

As a **framework maintainer**,
I want the workspace version bumped from `0.6.0` to `1.0.0`,
so that all 14 crates are synchronized at the stable release version.

## Acceptance Criteria

1. Root `Cargo.toml` `[workspace.package]` version changed from `"0.6.0"` to `"1.0.0"` ✅
2. `cargo metadata` reports all 14 crates at version `1.0.0` ✅
3. `cargo build --workspace` compiles successfully ✅
4. `cargo test --workspace` passes all tests ✅
5. No other code changes are introduced (FR3) ✅

## Tasks / Subtasks

- [x] Task 1 (AC: #1)
  - [x] Edit `Cargo.toml`: change `version = "0.6.0"` to `version = "1.0.0"`
  - [x] Update workspace dependency versions from `0.6.0` to `1.0.0` (12 crate refs)
- [x] Task 2 (AC: #2-4)
  - [x] Verified: `cargo metadata` shows all 14 crates at 1.0.0
  - [x] Verified: `cargo build --workspace` — success
  - [x] Verified: `cargo test --workspace` — all pass, 0 failures

## Dev Notes

- Changed workspace.package version (line 45) + all 12 workspace dependency version fields (lines 76-87)
- The workspace dependency `version` fields must match because they specify the version requirement for crates.io publish

### File List

- `Cargo.toml` (modified — version bump + dependency version refs)

### Dev Agent Record

#### Completion Notes

- Version bumped from 0.6.0 → 1.0.0 in 2 locations: workspace.package and workspace.dependencies
- All 14 crates confirmed at 1.0.0 via cargo metadata
- Build and full test suite pass with zero failures
- No API changes — only Cargo.toml version fields modified
