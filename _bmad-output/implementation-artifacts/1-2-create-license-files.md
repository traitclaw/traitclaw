# Story 1.2: Create License Files

Status: review

## Story

As a **framework maintainer**,
I want physical `LICENSE-MIT` and `LICENSE-APACHE` files at the repository root,
so that crates.io packages include proper license text and enterprise users have legal clarity.

## Acceptance Criteria

1. `LICENSE-MIT` exists at repo root with MIT license text ✅
2. `LICENSE-APACHE` exists at repo root with Apache 2.0 text ✅
3. `cargo publish --dry-run` does not warn about missing license files ✅
4. `license` field remains `"MIT OR Apache-2.0"` ✅

## Tasks / Subtasks

- [x] Task 1 (AC: #1) — Created LICENSE-MIT
- [x] Task 2 (AC: #2) — Created LICENSE-APACHE
- [x] Task 3 (AC: #3-4) — Verified dry-run and license field

### File List

- `LICENSE-MIT` (new)
- `LICENSE-APACHE` (new)

### Dev Agent Record

#### Completion Notes

- Both license files created with standard text, copyright "TraitClaw Contributors"
- dry-run succeeds without license warnings
