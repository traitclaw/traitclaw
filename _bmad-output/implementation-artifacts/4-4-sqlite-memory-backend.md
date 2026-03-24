# Story 4.4: SQLite Memory Backend

Status: ready-for-dev

## Story

As a developer,
I want persistent memory using SQLite,
So that agent conversations survive restarts.

## Acceptance Criteria

1. **Given** `traitclaw-memory-sqlite` crate with feature `"sqlite"` **When** I use `.memory(SqliteMemory::new("./agent.db"))` **Then** messages are persisted to SQLite database
2. **And** schema: `sessions(id, created_at)`, `messages(session_id, role, content, tool_call_id, created_at)`
3. **And** schema: `working_memory(session_id, key, value)`, `long_term_memory(id, content, metadata)`
4. **And** long-term recall uses FTS5 virtual table for text search
5. **And** implements full `Memory` trait including session lifecycle
6. **And** database schema is auto-created/migrated on `new()`
7. **And** concurrent access is handled safely
8. **And** uses `rusqlite` with `bundled` feature

## Tasks / Subtasks

- [ ] Task 1: Create `traitclaw-memory-sqlite` crate (AC: all)
  - [ ] Cargo.toml with rusqlite (bundled feature) and traitclaw-core
- [ ] Task 2: Define schema and migrations (AC: 2, 3, 4)
  - [ ] sessions table
  - [ ] messages table
  - [ ] working_memory table
  - [ ] long_term_memory table with FTS5
  - [ ] Auto-create on init
- [ ] Task 3: Implement `SqliteMemory` (AC: 1, 5, 6, 7, 8)
  - [ ] Implement full Memory trait
  - [ ] Thread-safe with connection pooling or Mutex
  - [ ] Auto-migrate schema on `new()`
- [ ] Task 4: Write tests (AC: all)
  - [ ] Test persistence across instances
  - [ ] Test FTS5 recall
  - [ ] Test session lifecycle
  - [ ] Test concurrent access

## Dev Notes

### Architecture Requirements
- `rusqlite` with `bundled` feature (bundles SQLite, no system dep)
- FTS5 for full-text search in long-term memory
- Connection: use Mutex<Connection> for simplicity, or r2d2 pool
- Schema auto-created on first use — no migration tool needed for MVP

### References
- [Source: _bmad-output/architecture.md#6 Optional - sqlite]
- [Source: _bmad-output/epics.md#Story 4.4]

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

### Change Log
