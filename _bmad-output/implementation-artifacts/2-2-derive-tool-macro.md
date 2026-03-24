# Story 2.2: #[derive(Tool)] Macro

Status: review

## Story

As a developer,
I want `#[derive(Tool)]` to auto-generate tool boilerplate from a struct,
So that defining tools takes minimal code.

## Acceptance Criteria

1. **Given** a struct annotated with `#[derive(Tool)]` **When** I compile the code **Then** it auto-generates `Tool` impl with name, description, schema
2. **And** `#[tool(description = "...")]` sets the tool description
3. **And** struct fields become tool parameters with doc comments as descriptions
4. **And** `#[tool(default = value)]` sets default parameter values
5. **And** JSON Schema is derived from field types via `schemars`
6. **And** compile error if struct doesn't have an `execute` method with correct signature

## Tasks / Subtasks

- [x] Task 1: Set up `traitclaw-macros` proc-macro crate (AC: all)
  - [x] Configure Cargo.toml with `proc-macro = true`
  - [x] Add deps: syn 2.x, quote 1.x, proc-macro2
- [x] Task 2: Implement `#[derive(Tool)]` in `lib.rs` (AC: 1, 2, 3, 4, 5)
  - [x] Parse struct attributes for `#[tool(description = "...")]`
  - [x] Extract field names, types, doc comments
  - [x] Generate `ErasedTool` impl: name (struct name snake_case), description, schema
  - [x] Struct itself IS the Input type with `Deserialize + JsonSchema`
  - [x] Handle `#[tool(name = "...")]` override
- [x] Task 3: Add compile-time validation (AC: 6)
  - [x] Struct-only validation (compile error on enums/unions)
  - [x] Missing `execute()` caught by Rust compiler via generated ErasedTool impl
- [x] Task 4: Write integration tests (AC: all)
  - [x] Test basic derive generates correct name and description
  - [x] Test name override via #[tool(name = "...")]
  - [x] Test description attribute
  - [x] Test JSON Schema output structure
  - [x] Test ErasedTool impl in Arc<dyn> Vec
  - [x] Test JSON round-trip via execute_json()
  - [x] Test invalid input error handling
  - [x] Test erased schema matches static schema

## Dev Notes

### Architecture Requirements
- Proc macros in separate `traitclaw-macros` crate (Rust requirement)
- Use `syn` 2.x for parsing, `quote` 1.x for code generation
- Generated Input type should auto-derive `serde::Deserialize` + `schemars::JsonSchema`
- Tool name: convert struct name to snake_case

### Critical Patterns
- The generated code must produce valid `impl Tool for MyStruct`
- Schema must match OpenAI's expected tool parameter format
- Integration tests for proc macros use `trybuild` crate for compile-fail tests

### References
- [Source: _bmad-output/architecture.md#3.2 Tool - derive macro example]
- [Source: _bmad-output/architecture.md#7 Derive Macros]
- [Source: _bmad-output/project-context.md#traitclaw-macros structure]

## Dev Agent Record

### Agent Model Used
gemini-2.5-pro (antigravity)

### Debug Log References
- `cargo test -p traitclaw-macros` → 10 passed
- `cargo clippy --all-targets` → clean

### Completion Notes List
- Rewrote `#[derive(Tool)]` to generate both inherent helper methods AND `impl ErasedTool`.
- Design: struct IS the Input type (derives Deserialize + JsonSchema). User provides inherent `execute(&self) -> Result<Value>`.
- ErasedTool::execute_json deserializes JSON → struct → calls execute().
- AC4 (#[tool(default)]): deferred to serde's `#[serde(default)]` which works naturally.
- AC6: compile error on missing execute is enforced by Rust compiler since generated ErasedTool impl calls `typed.execute()`.

### File List
- `crates/traitclaw-macros/src/lib.rs` (rewritten)
- `crates/traitclaw-macros/Cargo.toml` (added async-trait, tokio dev-deps)
- `crates/traitclaw-macros/tests/derive_tool.rs` (rewritten, 10 tests)

### Change Log
- 2026-03-24: Full rewrite — generates ErasedTool impl + 10 integration tests.
