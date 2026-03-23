//! Memory trait for agent state persistence.
//!
//! The [`Memory`] trait provides a 3-layer memory system:
//! - **Conversation**: Short-term message history
//! - **Working**: Task-specific key-value context
//! - **Long-term**: Semantic recall across sessions

use async_trait::async_trait;
use serde_json::Value;

use crate::types::message::Message;
use crate::Result;

/// A stored memory entry for long-term recall.
#[derive(Debug, Clone)]
pub struct MemoryEntry {
    /// Unique identifier for this entry.
    pub id: String,
    /// The content of the memory.
    pub content: String,
    /// Optional metadata associated with this entry.
    pub metadata: Option<Value>,
}

/// Trait for the 3-layer memory system.
///
/// Provides conversation history, working memory, and long-term recall.
/// Implement this trait for custom storage backends (`SQLite`, `PostgreSQL`, Redis, etc.).
#[async_trait]
pub trait Memory: Send + Sync + 'static {
    // === Conversation Memory (short-term) ===

    /// Get conversation messages for a session.
    async fn messages(&self, session_id: &str) -> Result<Vec<Message>>;

    /// Append a message to the conversation history.
    async fn append(&self, session_id: &str, message: Message) -> Result<()>;

    // === Working Memory (task-specific) ===

    /// Get a value from working memory.
    async fn get_context(&self, session_id: &str, key: &str) -> Result<Option<Value>>;

    /// Set a value in working memory.
    async fn set_context(&self, session_id: &str, key: &str, value: Value) -> Result<()>;

    // === Long-term Memory (semantic recall) ===

    /// Search for relevant memories.
    async fn recall(&self, query: &str, limit: usize) -> Result<Vec<MemoryEntry>>;

    /// Store a new memory entry.
    async fn store(&self, entry: MemoryEntry) -> Result<()>;
}
