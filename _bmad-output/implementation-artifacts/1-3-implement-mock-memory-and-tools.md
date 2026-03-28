# Story 1.3: Implement `MockMemory` and `MockTools`

Status: review

## Story

As a framework contributor,
I want shared `MockMemory` and mock tool types (EchoTool, FailTool),
so that I can test memory and tool-calling scenarios without boilerplate.

## Acceptance Criteria

1. ✅ `MockMemory::new()` creates a memory that stores/retrieves messages per session
2. ✅ `MockMemory` implements all `Memory` trait methods
3. ✅ `EchoTool` implements `Tool` — accepts `{ text: String }`, returns `{ echo: text }`
4. ✅ `FailTool` implements `Tool` — always returns `Err(Error::Runtime("tool failure"))`
5. ✅ All types implement `Send + Sync` and are `pub`
6. ✅ All types have doc comments with `/// # Example` blocks
7. ✅ `cargo test -p traitclaw-test-utils` passes with unit tests for each type

## Dev Agent Record

### Agent Model Used
Antigravity (Google DeepMind)

### Completion Notes List
- MockMemory: per-session HashMap storage, Default impl, 6 unit tests + 2 doc-tests
- EchoTool: typed Input/Output, 3 unit tests + 1 doc-test
- FailTool: always-error pattern, 2 unit tests + 1 doc-test
- Send + Sync static assertions for all types
- Added tokio-test dev-dependency for doc-test `block_on`

### File List
- MODIFIED: `crates/traitclaw-test-utils/src/memory.rs`
- MODIFIED: `crates/traitclaw-test-utils/src/tools.rs`
- MODIFIED: `crates/traitclaw-test-utils/Cargo.toml`
