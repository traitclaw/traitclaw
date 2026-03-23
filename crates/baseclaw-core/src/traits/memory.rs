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
///
/// This struct is `#[non_exhaustive]` — new fields (e.g., `embedding`, `score`)
/// may be added in future releases without breaking changes.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct MemoryEntry {
    /// Unique identifier for this entry.
    pub id: String,
    /// The content of the memory.
    pub content: String,
    /// Optional metadata associated with this entry.
    pub metadata: Option<Value>,
    /// Unix timestamp (seconds) when this entry was created.
    pub created_at: u64,
}

impl MemoryEntry {
    /// Create a new `MemoryEntry` with `created_at` set to the current system time.
    ///
    /// Falls back to `0` if the system clock is not available (e.g., in WASM).
    #[must_use]
    pub fn now(id: impl Into<String>, content: impl Into<String>) -> Self {
        let created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        Self {
            id: id.into(),
            content: content.into(),
            metadata: None,
            created_at,
        }
    }
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

    // === Session Lifecycle (default impls — override for persistent backends) ===

    /// Create a new session and return its ID.
    ///
    /// Default impl generates a random UUID v4. Override to use custom ID schemes.
    async fn create_session(&self) -> Result<String> {
        Ok(uuid::Uuid::new_v4().to_string())
    }

    /// List all known session IDs.
    ///
    /// Default impl always returns an empty vec. Override for persistent backends.
    async fn list_sessions(&self) -> Result<Vec<String>> {
        Ok(vec![])
    }

    /// Delete a session and all associated conversation history and working memory.
    ///
    /// Default impl is a no-op. Override for persistent backends.
    ///
    /// # Note on long-term memory
    ///
    /// Long-term memory (`store`/`recall`) is **global** across all sessions and
    /// is intentionally NOT cleared by this method. Use a separate cleanup
    /// strategy if per-session long-term memory isolation is required.
    async fn delete_session(&self, _session_id: &str) -> Result<()> {
        Ok(())
    }
}
