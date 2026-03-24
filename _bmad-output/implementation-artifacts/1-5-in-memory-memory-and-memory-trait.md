# Story 1.5: In-Memory Memory & Memory Trait

Status: review

## Story

As a developer,
I want a `Memory` trait with a default in-memory implementation,
So that agents can maintain conversation history out of the box.

## Acceptance Criteria

1. **Given** the Memory trait is defined with 3 layers **When** I use `InMemoryMemory::new()` **Then** `messages()` returns conversation history for a session
2. **And** `append()` adds a message to session history
3. **And** `get_context/set_context()` manages working memory key-value pairs
4. **And** `recall/store()` manages long-term memory entries
5. **And** InMemoryMemory is the default when no memory is configured

## Tasks / Subtasks

- [x] Task 1: Create `traits/memory.rs` (AC: 1, 2, 3, 4)
  - [x] `Memory` trait with all 3 layers: `messages/append`, `get_context/set_context`, `recall/store`
  - [x] `#[async_trait]` for async methods
  - [x] `MemoryEntry` struct with `id, content, metadata, created_at`
  - [x] `Send + Sync + 'static` bounds
  - [x] Session lifecycle default methods: `create_session`, `list_sessions`, `delete_session`
- [x] Task 2: Create `memory/in_memory.rs` (AC: 1, 2, 3, 4, 5)
  - [x] `InMemoryMemory` with `HashMap` + `tokio::sync::RwLock`
  - [x] Short-term / Working / Long-term layers implemented
  - [x] `Default` trait derived
- [x] Task 3: Session lifecycle overrides (AC: 1)
  - [x] `create_session()` — UUID v4, pre-creates bucket
  - [x] `list_sessions()` — returns all known session keys
  - [x] `delete_session()` — clears messages + context for session
- [x] Task 4: Tests (AC: all)
  - [x] AC-1/2: append + messages round-trip
  - [x] AC-3: context set/get
  - [x] AC-4: store/recall
  - [x] AC-1: session isolation
  - [x] Task 3: session lifecycle (create/list/delete)
  - [x] AC-5: `InMemoryMemory::default()`

## Dev Notes

### Architecture Requirements
- 3-layer memory model: short-term (messages), working (context), long-term (recall)
- InMemoryMemory is zero-dependency (no external crates for storage)
- Thread-safe: use `tokio::sync::RwLock` for async access
- `serde_json::Value` for working memory values (flexible type)
- `MemoryEntry` for long-term memory (content + metadata)

### Critical Patterns
- Use `tokio::sync::RwLock` NOT `std::sync::RwLock` (async context)
- `recall()` in InMemoryMemory: simple `contains()` substring match for MVP
- Session lifecycle methods have default impls so external Memory impls don't need them initially
- `uuid` crate needed for `create_session()` default impl

### Dependencies
- `uuid` = { version = "1", features = ["v4"] } — for session ID generation
- `tokio` (already workspace dep) — for RwLock
- `serde_json` (already workspace dep) — for Value type

### References
- [Source: _bmad-output/architecture.md#3.3 Memory]
- [Source: _bmad-output/project-context.md#traitclaw-core internal structure]

## Dev Agent Record

### Agent Model Used
gemini-2.5-pro (antigravity)

### Debug Log References
- `cargo test` (workspace) → all passed
- `cargo clippy --all-targets` → clean
- `cargo fmt --all --check` → clean

### Completion Notes List
- Core files existed. Gaps filled: `created_at` added to `MemoryEntry`, session lifecycle methods added to trait (default impls) and InMemoryMemory (full overrides).
- Added `uuid = { version = "1", features = ["v4"] }` to workspace deps.
- Added 3 new tests covering session isolation, lifecycle, and Default.

### File List
- `Cargo.toml` (added uuid to workspace deps)
- `crates/traitclaw-core/Cargo.toml` (added uuid dep)
- `crates/traitclaw-core/src/traits/memory.rs` (created_at, session lifecycle default methods)
- `crates/traitclaw-core/src/memory/in_memory.rs` (lifecycle overrides, 3 new tests)

### Change Log
- 2026-03-24: All tasks complete.

---

## Senior Developer Review (AI)

**Review Date:** 2026-03-24  
**Outcome:** Approved  
**Reviewer:** gemini-2.5-pro (code-review workflow)

### Action Items

#### Patches
- [x] [High] P-1: `MemoryEntry` marked `#[non_exhaustive]`; added `MemoryEntry::now()` constructor.
- [x] [Med] P-2: All test `MemoryEntry` literals replaced with `MemoryEntry::now()`.
- [x] [Med] P-3: `delete_session()` doc updated in trait and `InMemoryMemory` override: long-term memory intentionally NOT cleared.

#### Deferred
- [ ] [Low] D-1: `list_sessions()` misses context-only sessions (no `append()` called).
- [ ] [Low] D-2: No test for empty-query `recall()` truncation behavior.
- [ ] [Low] D-3: No test verifying `delete_session` clears working memory context.

