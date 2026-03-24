# Story 2.1: Tool Trait & ErasedTool

Status: review

## Story

As a developer,
I want a `Tool` trait with associated Input/Output types,
So that tools are type-safe and have auto-generated schemas.

## Acceptance Criteria

1. **Given** the Tool trait is defined **When** I implement it with typed Input/Output **Then** `schema()` returns a `ToolSchema` with JSON Schema from the Input type
2. **And** `execute()` takes validated Input and returns typed Output
3. **And** `ErasedTool` wrapper enables `Vec<Arc<dyn ErasedTool>>` storage in Agent
4. **And** ErasedTool handles JSON → Input deserialization → execute → Output → JSON

## Tasks / Subtasks

- [x] Task 1: Define `Tool` trait (AC: 1, 2)
  - [x] Associated types: `type Input: DeserializeOwned + JsonSchema;` and `type Output: Serialize;`
  - [x] Methods: `fn name(&self) -> &str`, `fn description(&self) -> &str`
  - [x] `fn schema(&self) -> ToolSchema` — generates JSON Schema from Input type
  - [x] `async fn execute(&self, input: Self::Input) -> Result<Self::Output>`
  - [x] `Send + Sync + 'static` bounds
- [x] Task 2: Define `ErasedTool` trait (AC: 3, 4)
  - [x] `fn name(&self) -> &str`
  - [x] `fn description(&self) -> &str`
  - [x] `fn schema(&self) -> ToolSchema`
  - [x] `async fn call(&self, args: Value) -> Result<Value>` — handles JSON serde
  - [x] Blanket impl: `impl<T: Tool> ErasedTool for T`
- [x] Task 3: Define `ToolSchema` type (AC: 1)
  - [x] Wrapper around schemars-generated JSON Schema
  - [x] Serializes to OpenAI tool format
- [x] Task 4: Write tests (AC: all)
  - [x] Test manual Tool implementation
  - [x] Test ErasedTool wrapping and JSON round-trip
  - [x] Test schema generation from schemars

## Dev Notes

### Architecture Requirements
- Tool trait uses Rust type system as schema truth (AD-4)
- ErasedTool is the type-erased version for heterogeneous storage
- `schemars` crate generates JSON Schema from Rust types
- ToolSchema must serialize to match OpenAI `tools` parameter format

### Critical Patterns
```rust
// Tool trait with associated types
pub trait Tool: Send + Sync + 'static {
    type Input: DeserializeOwned + JsonSchema;
    type Output: Serialize;
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn schema(&self) -> ToolSchema;
    async fn execute(&self, input: Self::Input) -> Result<Self::Output>;
}
```

### References
- [Source: _bmad-output/architecture.md#3.2 Tool]
- [Source: _bmad-output/project-context.md#Technology Stack - schemars]

## Dev Agent Record

### Agent Model Used
gemini-2.5-pro (antigravity)

### Debug Log References
- `cargo test -p traitclaw-core -- traits::tool` → 8 passed + 1 doc-test
- `cargo clippy --all-targets` → clean

### Completion Notes List
- Tool trait + ErasedTool + ToolSchema were already implemented from earlier scaffolding.
- Added 8 comprehensive tests covering all 4 ACs.
- Tests verify: typed I/O, schema generation via schemars, Arc<dyn ErasedTool> storage, JSON round-trip, invalid input error handling, schema equality between Tool and ErasedTool.

### File List
- `crates/traitclaw-core/src/traits/tool.rs` (tests added)

### Change Log
- 2026-03-24: Task 4 tests added. All ACs verified.
