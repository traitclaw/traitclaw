# Story 3.1: Create Migration Guide

Status: ready-for-dev

## Story

As a framework user upgrading from v0.8.0,
I want a clear migration guide documenting all breaking changes,
so that I can upgrade my project in under 5 minutes.

## Acceptance Criteria

1. `docs/migration-v0.8-to-v0.9.md` exists
2. All breaking changes listed with search-and-replace patterns
3. Before/after code examples for every breaking change
4. Method signature comparison (`ContextStrategy::prepare()` → `ContextManager::prepare()`)
5. Prelude changes documented (removed items, added items)

## Tasks / Subtasks

- [ ] Task 1: Create migration guide document (AC: #1, #2, #3, #4)
  - [ ] Create `docs/migration-v0.8-to-v0.9.md`
  - [ ] Document `ContextStrategy` → `ContextManager` rename with before/after examples
  - [ ] Document `OutputProcessor` → `OutputTransformer` rename with before/after examples
  - [ ] Document `AgentRuntime` field removals (`context_strategy`, `output_processor`)
  - [ ] Document method signature differences (sync → async, extra params)
  - [ ] Add search-and-replace patterns for each removed type
- [ ] Task 2: Document prelude changes (AC: #5)
  - [ ] List items removed from prelude
  - [ ] List items added to prelude (`CompressedMemory`, `RetryConfig`, `RetryProvider`, `DynamicRegistry`)
- [ ] Task 3: Review and polish
  - [ ] Verify all examples compile conceptually
  - [ ] Add header with version info and "5 minute migration" pitch

## Dev Notes

### Migration Guide Structure
Follow the same format as previous guides:
- `docs/migration-v0.7-to-v0.8.md` — reference for format

### Breaking Changes Summary

| Change | Search | Replace |
|--------|--------|---------|
| Trait rename | `ContextStrategy` | `ContextManager` |
| Trait rename | `OutputProcessor` | `OutputTransformer` |
| Impl rename | `NoopContextStrategy` | (use ContextManager default) |
| Impl rename | `SlidingWindowStrategy` | `RuleBasedCompressor` |
| Impl rename | `TruncateProcessor` | (use OutputTransformer default or custom) |
| Field removed | `runtime.context_strategy` | `runtime.context_manager` |
| Field removed | `runtime.output_processor` | `runtime.output_transformer` |
| Method sig | `prepare(&self, msgs, window, state)` | `async prepare(&self, msgs, window, state)` |
| Method sig | `process(&self, output) -> String` | `async transform(&self, output, tool_name, state) -> String` |

### References
- [Source: prd-v0.9.0.md#User Journeys] — < 5 minute upgrade
- [Source: epics-v0.9.0.md#Story 3.1]
- [Source: docs/migration-v0.7-to-v0.8.md] — format reference

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

### Change Log
