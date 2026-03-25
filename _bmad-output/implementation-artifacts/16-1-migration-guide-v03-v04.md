# Story 16.1: Migration Guide v0.3 → v0.4

Status: done

## Story

As a developer upgrading from v0.3.0,
I want a clear migration guide,
so that I understand what's new and how to adopt features.

## Acceptance Criteria

1. `docs/migration-v0.3-to-v0.4.md` is created
2. Confirms "No breaking changes. v0.3.0 code compiles unchanged."
3. Lists all new implementations with code snippets
4. Shows `GroupedRegistry` usage with before/after comparison
5. Shows `TikTokenCounter` feature flag enablement
6. Shows `McpToolRegistry` connection setup
7. Shows `ProgressiveTransformer` usage

## Tasks / Subtasks

- [x] Task 1: Created `docs/migration-v0.3-to-v0.4.md` with "No breaking changes" opening (AC: #1, #2)
- [x] Task 2: Feature adoption guides for all 6 new features (AC: #3-#7)
  - [x] GroupedRegistry before/after comparison
  - [x] AdaptiveRegistry usage
  - [x] ProgressiveTransformer usage
  - [x] TikTokenCounter: Cargo.toml feature flag + API
  - [x] McpToolRegistry: single-server connection
  - [x] MultiServerMcpRegistry: multi-server with prefixing

## Dev Notes

- Follow pattern of `docs/migration-v0.2-to-v0.3.md`
- Keep code snippets runnable and self-contained
- Emphasize that all v0.3.0 code works unchanged

### References

- [Source: docs/migration-v0.2-to-v0.3.md] — format reference

## Dev Agent Record

### Agent Model Used
Gemini 2.5 Flash

### Completion Notes List
- All 6 v0.4.0 features documented with code snippets
- Follows v0.2→v0.3 migration doc format exactly
- "No breaking changes" confirmed at top

### File List
- `docs/migration-v0.3-to-v0.4.md` — new document

### Change Log
- 2026-03-26: Initial creation
