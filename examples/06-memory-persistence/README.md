# 06 — Memory Persistence

Persist conversations and knowledge with SQLite.

## What it does

1. Creates a **SQLite memory backend** (file or in-memory)
2. Stores multi-turn conversations across sessions
3. Demonstrates **working memory** (key-value per session)
4. Shows **long-term recall** with FTS5 full-text search

## Key APIs

```rust
let memory = SqliteMemory::new("./agent.db")?;        // persistent
let memory = SqliteMemory::in_memory()?;               // for testing

let session = memory.create_session().await?;           // new session
memory.append(&session, Message::user("Hi")).await?;    // store message
let history = memory.messages(&session).await?;         // retrieve

memory.set_context(&session, "key", json!("val")).await?; // working memory
memory.store(MemoryEntry::now("id", "content")).await?;    // long-term
let results = memory.recall("search query", 5).await?;    // FTS5 recall
```

## Running

```bash
cargo run
```
