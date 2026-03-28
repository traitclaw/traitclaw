//! SQLite memory backend for the `TraitClaw` AI agent framework.
//!
//! Provides persistent conversation history, working memory, and FTS5-powered
//! long-term recall — all backed by a single `SQLite` database file.
//!
//! # Quick Start
//!
//! ```rust,no_run
//! use traitclaw_memory_sqlite::SqliteMemory;
//!
//! let memory = SqliteMemory::new("./agent.db").expect("Failed to open database");
//! ```

#![deny(missing_docs)]
#![allow(clippy::redundant_closure)]

use std::sync::Mutex;

use async_trait::async_trait;
use rusqlite::Connection;
use serde_json::Value;
use traitclaw_core::traits::memory::{Memory, MemoryEntry};
use traitclaw_core::types::message::{Message, MessageRole};
use traitclaw_core::Result;

/// SQLite-backed memory backend.
///
/// Uses a single `SQLite` database file to persist:
/// - **Conversation memory** — `sessions` + `messages` tables
/// - **Working memory** — `working_memory` (key/value per session)
/// - **Long-term memory** — `long_term_memory` + FTS5 virtual table
pub struct SqliteMemory {
    conn: Mutex<Connection>,
}

impl SqliteMemory {
    /// Open (or create) a SQLite database at the given path.
    ///
    /// The schema is auto-created/migrated on first access.
    ///
    /// # Errors
    ///
    /// Returns an error if the database cannot be opened or the schema
    /// cannot be created.
    pub fn new(path: &str) -> Result<Self> {
        let conn = Connection::open(path)
            .map_err(|e| traitclaw_core::Error::Runtime(format!("SQLite open error: {e}")))?;

        init_schema(&conn)?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Create an in-memory SQLite database (useful for testing).
    ///
    /// # Errors
    ///
    /// Returns an error if the schema cannot be created.
    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()
            .map_err(|e| traitclaw_core::Error::Runtime(format!("SQLite open error: {e}")))?;

        init_schema(&conn)?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }
}

fn init_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS sessions (
            id         TEXT PRIMARY KEY,
            created_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))
        );

        CREATE TABLE IF NOT EXISTS messages (
            id         INTEGER PRIMARY KEY AUTOINCREMENT,
            session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
            role       TEXT NOT NULL,
            content    TEXT NOT NULL DEFAULT '',
            tool_call_id TEXT,
            created_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))
        );
        CREATE INDEX IF NOT EXISTS idx_messages_session ON messages(session_id);

        CREATE TABLE IF NOT EXISTS working_memory (
            session_id TEXT NOT NULL,
            key        TEXT NOT NULL,
            value      TEXT NOT NULL,
            PRIMARY KEY (session_id, key)
        );

        CREATE TABLE IF NOT EXISTS long_term_memory (
            id         TEXT PRIMARY KEY,
            content    TEXT NOT NULL,
            metadata   TEXT,
            created_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))
        );

        CREATE VIRTUAL TABLE IF NOT EXISTS long_term_fts
            USING fts5(content, content_rowid='rowid');
        ",
    )
    .map_err(|e| traitclaw_core::Error::Runtime(format!("Schema init error: {e}")))?;

    Ok(())
}

fn role_to_str(role: &MessageRole) -> &'static str {
    match role {
        MessageRole::System => "system",
        MessageRole::User => "user",
        MessageRole::Assistant => "assistant",
        MessageRole::Tool => "tool",
        _ => "unknown",
    }
}

fn str_to_role(s: &str) -> MessageRole {
    match s {
        "system" => MessageRole::System,
        "assistant" => MessageRole::Assistant,
        "tool" => MessageRole::Tool,
        // "user" and anything unknown default to User
        _ => MessageRole::User,
    }
}

#[async_trait]
impl Memory for SqliteMemory {
    async fn messages(&self, session_id: &str) -> Result<Vec<Message>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| traitclaw_core::Error::Runtime(format!("Lock error: {e}")))?;
        let mut stmt = conn
            .prepare("SELECT role, content, tool_call_id FROM messages WHERE session_id = ?1 ORDER BY id")
            .map_err(|e| traitclaw_core::Error::Runtime(format!("Query error: {e}")))?;

        let rows = stmt
            .query_map([session_id], |row| {
                let role: String = row.get(0)?;
                let content: String = row.get(1)?;
                let tool_call_id: Option<String> = row.get(2)?;
                Ok(Message {
                    role: str_to_role(&role),
                    content,
                    tool_call_id,
                })
            })
            .map_err(|e| traitclaw_core::Error::Runtime(format!("Query error: {e}")))?;

        let mut messages = Vec::new();
        for row in rows {
            messages
                .push(row.map_err(|e| traitclaw_core::Error::Runtime(format!("Row error: {e}")))?);
        }
        Ok(messages)
    }

    async fn append(&self, session_id: &str, message: Message) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| traitclaw_core::Error::Runtime(format!("Lock error: {e}")))?;
        conn.execute(
            "INSERT INTO messages (session_id, role, content, tool_call_id) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![
                session_id,
                role_to_str(&message.role),
                message.content,
                message.tool_call_id,
            ],
        )
        .map_err(|e| traitclaw_core::Error::Runtime(format!("Insert error: {e}")))?;
        Ok(())
    }

    async fn get_context(&self, session_id: &str, key: &str) -> Result<Option<Value>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| traitclaw_core::Error::Runtime(format!("Lock error: {e}")))?;
        let result: rusqlite::Result<String> = conn.query_row(
            "SELECT value FROM working_memory WHERE session_id = ?1 AND key = ?2",
            rusqlite::params![session_id, key],
            |row| row.get(0),
        );
        match result {
            Ok(json_str) => {
                let val: Value = serde_json::from_str(&json_str).unwrap_or(Value::String(json_str));
                Ok(Some(val))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(traitclaw_core::Error::Runtime(format!("Query error: {e}"))),
        }
    }

    async fn set_context(&self, session_id: &str, key: &str, value: Value) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| traitclaw_core::Error::Runtime(format!("Lock error: {e}")))?;
        let json_str = serde_json::to_string(&value)
            .map_err(|e| traitclaw_core::Error::Runtime(format!("JSON error: {e}")))?;
        conn.execute(
            "INSERT OR REPLACE INTO working_memory (session_id, key, value) VALUES (?1, ?2, ?3)",
            rusqlite::params![session_id, key, json_str],
        )
        .map_err(|e| traitclaw_core::Error::Runtime(format!("Insert error: {e}")))?;
        Ok(())
    }

    async fn recall(&self, query: &str, limit: usize) -> Result<Vec<MemoryEntry>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| traitclaw_core::Error::Runtime(format!("Lock error: {e}")))?;
        let mut stmt = conn
            .prepare(
                "SELECT m.id, m.content, m.metadata, m.created_at
                 FROM long_term_fts f
                 JOIN long_term_memory m ON m.rowid = f.rowid
                 WHERE long_term_fts MATCH ?1
                 ORDER BY rank
                 LIMIT ?2",
            )
            .map_err(|e| traitclaw_core::Error::Runtime(format!("Query error: {e}")))?;

        let rows = stmt
            .query_map(rusqlite::params![query, limit], |row| {
                let id: String = row.get(0)?;
                let content: String = row.get(1)?;
                let metadata_str: Option<String> = row.get(2)?;
                let created_at: u64 = row.get(3)?;
                let metadata = metadata_str.and_then(|s| serde_json::from_str::<Value>(&s).ok());
                let mut entry = MemoryEntry::now(id, content);
                entry.metadata = metadata;
                entry.created_at = created_at;
                Ok(entry)
            })
            .map_err(|e| traitclaw_core::Error::Runtime(format!("Query error: {e}")))?;

        let mut entries = Vec::new();
        for row in rows {
            entries
                .push(row.map_err(|e| traitclaw_core::Error::Runtime(format!("Row error: {e}")))?);
        }
        Ok(entries)
    }

    async fn store(&self, entry: MemoryEntry) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| traitclaw_core::Error::Runtime(format!("Lock error: {e}")))?;
        let metadata_str = entry
            .metadata
            .as_ref()
            .map(|v| serde_json::to_string(v).unwrap_or_default());

        conn.execute(
            "INSERT OR REPLACE INTO long_term_memory (id, content, metadata, created_at) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![entry.id, entry.content, metadata_str, entry.created_at],
        )
        .map_err(|e| traitclaw_core::Error::Runtime(format!("Insert error: {e}")))?;

        // Insert into FTS index
        let rowid: i64 = conn
            .query_row(
                "SELECT rowid FROM long_term_memory WHERE id = ?1",
                rusqlite::params![entry.id],
                |row| row.get(0),
            )
            .map_err(|e| traitclaw_core::Error::Runtime(format!("Rowid error: {e}")))?;

        conn.execute(
            "INSERT OR REPLACE INTO long_term_fts (rowid, content) VALUES (?1, ?2)",
            rusqlite::params![rowid, entry.content],
        )
        .map_err(|e| traitclaw_core::Error::Runtime(format!("FTS insert error: {e}")))?;

        Ok(())
    }

    async fn create_session(&self) -> Result<String> {
        let id = uuid::Uuid::new_v4().to_string();
        let conn = self
            .conn
            .lock()
            .map_err(|e| traitclaw_core::Error::Runtime(format!("Lock error: {e}")))?;
        conn.execute("INSERT INTO sessions (id) VALUES (?1)", [&id])
            .map_err(|e| traitclaw_core::Error::Runtime(format!("Insert error: {e}")))?;
        Ok(id)
    }

    async fn list_sessions(&self) -> Result<Vec<String>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| traitclaw_core::Error::Runtime(format!("Lock error: {e}")))?;
        let mut stmt = conn
            .prepare("SELECT id FROM sessions ORDER BY created_at")
            .map_err(|e| traitclaw_core::Error::Runtime(format!("Query error: {e}")))?;
        let rows = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|e| traitclaw_core::Error::Runtime(format!("Query error: {e}")))?;

        let mut ids = Vec::new();
        for row in rows {
            ids.push(row.map_err(|e| traitclaw_core::Error::Runtime(format!("Row error: {e}")))?);
        }
        Ok(ids)
    }

    async fn delete_session(&self, session_id: &str) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| traitclaw_core::Error::Runtime(format!("Lock error: {e}")))?;
        // CASCADE will delete messages; manually delete working_memory
        conn.execute(
            "DELETE FROM working_memory WHERE session_id = ?1",
            [session_id],
        )
        .map_err(|e| traitclaw_core::Error::Runtime(format!("Delete error: {e}")))?;
        conn.execute("DELETE FROM messages WHERE session_id = ?1", [session_id])
            .map_err(|e| traitclaw_core::Error::Runtime(format!("Delete error: {e}")))?;
        conn.execute("DELETE FROM sessions WHERE id = ?1", [session_id])
            .map_err(|e| traitclaw_core::Error::Runtime(format!("Delete error: {e}")))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_session_lifecycle() {
        let mem = SqliteMemory::in_memory().unwrap();
        let sid = mem.create_session().await.unwrap();

        let sessions = mem.list_sessions().await.unwrap();
        assert!(sessions.contains(&sid));

        mem.delete_session(&sid).await.unwrap();
        let sessions = mem.list_sessions().await.unwrap();
        assert!(!sessions.contains(&sid));
    }

    #[tokio::test]
    async fn test_conversation_persistence() {
        let mem = SqliteMemory::in_memory().unwrap();
        let sid = mem.create_session().await.unwrap();

        mem.append(&sid, Message::user("Hello")).await.unwrap();
        mem.append(&sid, Message::assistant("Hi there!"))
            .await
            .unwrap();

        let msgs = mem.messages(&sid).await.unwrap();
        assert_eq!(msgs.len(), 2);
        assert!(matches!(msgs[0].role, MessageRole::User));
        assert_eq!(msgs[0].content, "Hello");
        assert!(matches!(msgs[1].role, MessageRole::Assistant));
        assert_eq!(msgs[1].content, "Hi there!");
    }

    #[tokio::test]
    async fn test_working_memory() {
        let mem = SqliteMemory::in_memory().unwrap();
        let sid = mem.create_session().await.unwrap();

        // Initially empty
        assert!(mem.get_context(&sid, "task").await.unwrap().is_none());

        // Set and get
        mem.set_context(&sid, "task", serde_json::json!("coding"))
            .await
            .unwrap();
        let val = mem.get_context(&sid, "task").await.unwrap().unwrap();
        assert_eq!(val, serde_json::json!("coding"));

        // Overwrite
        mem.set_context(&sid, "task", serde_json::json!("testing"))
            .await
            .unwrap();
        let val = mem.get_context(&sid, "task").await.unwrap().unwrap();
        assert_eq!(val, serde_json::json!("testing"));
    }

    #[tokio::test]
    async fn test_long_term_store_and_recall() {
        let mem = SqliteMemory::in_memory().unwrap();

        mem.store(MemoryEntry::now(
            "1",
            "Rust is a systems programming language",
        ))
        .await
        .unwrap();
        mem.store(MemoryEntry::now("2", "Python is great for data science"))
            .await
            .unwrap();
        mem.store(MemoryEntry::now("3", "Rust has zero-cost abstractions"))
            .await
            .unwrap();

        let results = mem.recall("Rust programming", 10).await.unwrap();
        assert!(!results.is_empty());
        // FTS5 should rank Rust-related entries higher
        assert!(results.iter().any(|r| r.content.contains("Rust")));
    }

    #[tokio::test]
    async fn test_recall_empty() {
        let mem = SqliteMemory::in_memory().unwrap();
        let results = mem.recall("anything", 10).await.unwrap();
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn test_delete_session_clears_messages_and_context() {
        let mem = SqliteMemory::in_memory().unwrap();
        let sid = mem.create_session().await.unwrap();

        mem.append(&sid, Message::user("test")).await.unwrap();
        mem.set_context(&sid, "key", serde_json::json!("val"))
            .await
            .unwrap();

        mem.delete_session(&sid).await.unwrap();

        let msgs = mem.messages(&sid).await.unwrap();
        assert!(msgs.is_empty());
        assert!(mem.get_context(&sid, "key").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_tool_message_with_call_id() {
        let mem = SqliteMemory::in_memory().unwrap();
        let sid = mem.create_session().await.unwrap();

        mem.append(
            &sid,
            Message {
                role: MessageRole::Tool,
                content: "result".into(),
                tool_call_id: Some("call_1".into()),
            },
        )
        .await
        .unwrap();

        let msgs = mem.messages(&sid).await.unwrap();
        assert_eq!(msgs.len(), 1);
        assert!(matches!(msgs[0].role, MessageRole::Tool));
        assert_eq!(msgs[0].tool_call_id, Some("call_1".into()));
    }
}
