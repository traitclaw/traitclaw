# Story 3.1: ToolRegistry Trait Definition

Status: in-progress

## Story

As a framework developer,
I want the `ToolRegistry` trait defined in `traitclaw-core`,
so that tools can be dynamically activated/deactivated at runtime.

## Acceptance Criteria

1. `traitclaw-core/src/traits/tool_registry.rs` exists with `ToolRegistry` trait
2. Read methods: `get_tools()`, `find_tool(name: &str)`
3. Write methods: `register()`, `unregister()`, `set_enabled()`
4. `SimpleRegistry` wraps existing `Vec<Arc<dyn ErasedTool>>` with no behavior change
5. Trait requires `Send + Sync`, uses `&self` with interior mutability
6. Unit test confirms trait is object-safe
7. Rustdoc with examples

## Tasks / Subtasks

- [ ] Task 1: Create `tool_registry.rs` with trait definition
- [ ] Task 2: Implement `SimpleRegistry` (immutable, wraps Vec)
- [ ] Task 3: Register module and add re-exports
- [ ] Task 4: Add unit tests
- [ ] Task 5: Run full test suite

## Dev Agent Record

### Agent Model Used
Gemini 2.5 Pro
### Completion Notes List
### File List
### Change Log
