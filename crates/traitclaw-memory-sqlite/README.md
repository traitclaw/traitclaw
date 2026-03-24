# traitclaw-memory-sqlite

[![crates.io](https://img.shields.io/crates/v/traitclaw-memory-sqlite.svg)](https://crates.io/crates/traitclaw-memory-sqlite)
[![docs.rs](https://docs.rs/traitclaw-memory-sqlite/badge.svg)](https://docs.rs/traitclaw-memory-sqlite)

**SQLite memory backend for TraitClaw — persistent conversations, working memory, and FTS5-powered long-term recall.**

Store conversation history and knowledge in a single SQLite database file. No external services, no network calls — just one `.db` file that persists across agent restarts.

## Features

| Feature | Description |
|---------|-------------|
| **Conversation memory** | Store and retrieve per-session message history |
| **Working memory** | Key-value store per session (e.g., user preferences, task state) |
| **Long-term memory** | FTS5 full-text search over stored knowledge |
| **Session management** | Create, list, and delete conversation sessions |

## Usage

```rust
use traitclaw::prelude::*;
use traitclaw_memory_sqlite::SqliteMemory;

// Persistent storage
let memory = SqliteMemory::new("./agent.db")?;

// Or in-memory (for tests)
let memory = SqliteMemory::in_memory()?;

// Session lifecycle
let session = memory.create_session().await?;
memory.append(&session, Message::user("Hello")).await?;
let history = memory.messages(&session).await?;

// Working memory (key-value)
memory.set_context(&session, "user_name", json!("Alex")).await?;
let name = memory.get_context(&session, "user_name").await?;

// Long-term recall (FTS5 search)
memory.store(MemoryEntry::now("id", "Rust has zero-cost abstractions")).await?;
let results = memory.recall("Rust abstractions", 5).await?;
```

## Schema

All data lives in a single SQLite file with these tables:

| Table | Purpose |
|-------|---------|
| `sessions` | Session metadata |
| `messages` | Conversation history (per session) |
| `working_memory` | Key-value facts (per session) |
| `long_term_memory` | Knowledge entries for recall |
| `long_term_fts` | FTS5 virtual table for full-text search |

## License

Licensed under either of [Apache License, Version 2.0](../../LICENSE-APACHE) or [MIT License](../../LICENSE-MIT) at your option.
