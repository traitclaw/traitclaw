//! In-memory implementation of the [`Memory`] trait.
//!
//! This is the default memory backend, included in `traitclaw-core` with zero
//! external dependencies. Suitable for prototyping and testing.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;
use tokio::sync::RwLock;

use crate::traits::memory::{Memory, MemoryEntry};
use crate::types::message::Message;
use crate::Result;

/// In-memory implementation of the [`Memory`] trait.
///
/// All data is stored in memory and lost when the process exits.
/// This is the default memory backend when no other is configured.
///
/// # Example
///
/// ```rust
/// use traitclaw_core::memory::in_memory::InMemoryMemory;
///
/// let memory = InMemoryMemory::new();
/// ```
#[derive(Debug, Default)]
pub struct InMemoryMemory {
    /// Conversation messages keyed by session ID.
    messages: Arc<RwLock<HashMap<String, Vec<Message>>>>,
    /// Working memory: `session_id` -> (key -> value).
    context: Arc<RwLock<HashMap<String, HashMap<String, Value>>>>,
    /// Long-term memory entries.
    long_term: Arc<RwLock<Vec<MemoryEntry>>>,
}

impl InMemoryMemory {
    /// Create a new empty in-memory store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl Memory for InMemoryMemory {
    async fn messages(&self, session_id: &str) -> Result<Vec<Message>> {
        let store = self.messages.read().await;
        Ok(store.get(session_id).cloned().unwrap_or_default())
    }

    async fn append(&self, session_id: &str, message: Message) -> Result<()> {
        let mut store = self.messages.write().await;
        store
            .entry(session_id.to_string())
            .or_default()
            .push(message);
        Ok(())
    }

    async fn get_context(&self, session_id: &str, key: &str) -> Result<Option<Value>> {
        let store = self.context.read().await;
        Ok(store.get(session_id).and_then(|ctx| ctx.get(key)).cloned())
    }

    async fn set_context(&self, session_id: &str, key: &str, value: Value) -> Result<()> {
        let mut store = self.context.write().await;
        store
            .entry(session_id.to_string())
            .or_default()
            .insert(key.to_string(), value);
        Ok(())
    }

    async fn recall(&self, query: &str, limit: usize) -> Result<Vec<MemoryEntry>> {
        // NOTE: uses simple substring matching. An empty `query` matches ALL entries and
        // results are silently truncated to `limit`. Callers should treat a full result
        // set as potentially truncated rather than exhaustive.
        let store = self.long_term.read().await;
        let results = store
            .iter()
            .filter(|entry| entry.content.contains(query))
            .take(limit)
            .cloned()
            .collect();
        Ok(results)
    }

    async fn store(&self, entry: MemoryEntry) -> Result<()> {
        let mut store = self.long_term.write().await;
        store.push(entry);
        Ok(())
    }

    // === Session Lifecycle (overridden — full impl) ===

    async fn create_session(&self) -> Result<String> {
        let id = uuid::Uuid::new_v4().to_string();
        // Pre-create the session bucket so it appears in list_sessions immediately.
        let mut store = self.messages.write().await;
        store.entry(id.clone()).or_default();
        Ok(id)
    }

    async fn list_sessions(&self) -> Result<Vec<String>> {
        let store = self.messages.read().await;
        Ok(store.keys().cloned().collect())
    }

    /// # Note on long-term memory
    ///
    /// Long-term memory is **global** and intentionally NOT cleared.
    /// See [`Memory::delete_session`] for details.
    async fn delete_session(&self, session_id: &str) -> Result<()> {
        self.messages.write().await.remove(session_id);
        self.context.write().await.remove(session_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::memory::Memory;

    #[tokio::test]
    async fn test_append_and_get_messages() {
        let memory = InMemoryMemory::new();
        memory
            .append("session1", Message::user("Hello"))
            .await
            .unwrap();
        memory
            .append("session1", Message::assistant("Hi!"))
            .await
            .unwrap();

        let messages = memory.messages("session1").await.unwrap();
        assert_eq!(messages.len(), 2);
    }

    #[tokio::test]
    async fn test_empty_session_returns_empty() {
        let memory = InMemoryMemory::new();
        let messages = memory.messages("nonexistent").await.unwrap();
        assert!(messages.is_empty());
    }

    #[tokio::test]
    async fn test_working_memory() {
        let memory = InMemoryMemory::new();
        memory
            .set_context("s1", "key1", serde_json::json!("value1"))
            .await
            .unwrap();

        let val = memory.get_context("s1", "key1").await.unwrap();
        assert_eq!(val, Some(serde_json::json!("value1")));

        let none = memory.get_context("s1", "missing").await.unwrap();
        assert!(none.is_none());
    }

    #[tokio::test]
    async fn test_long_term_memory() {
        let memory = InMemoryMemory::new();
        memory
            .store(MemoryEntry::now("1", "Rust is great"))
            .await
            .unwrap();

        let results = memory.recall("Rust", 10).await.unwrap();
        assert_eq!(results.len(), 1);

        let empty = memory.recall("Python", 10).await.unwrap();
        assert!(empty.is_empty());
    }

    #[tokio::test]
    async fn test_session_isolation_ac1() {
        // AC-1: sessions are independent — messages from session A don't appear in B
        let memory = InMemoryMemory::new();
        memory
            .append("session-a", Message::user("Hello A"))
            .await
            .unwrap();
        memory
            .append("session-b", Message::user("Hello B"))
            .await
            .unwrap();

        let msgs_a = memory.messages("session-a").await.unwrap();
        let msgs_b = memory.messages("session-b").await.unwrap();
        assert_eq!(msgs_a.len(), 1);
        assert_eq!(msgs_b.len(), 1);
        assert_eq!(msgs_a[0].content, "Hello A");
        assert_eq!(msgs_b[0].content, "Hello B");
    }

    #[tokio::test]
    async fn test_session_lifecycle() {
        // AC-1 / Task 3: create_session, list_sessions, delete_session
        let memory = InMemoryMemory::new();
        let id = memory.create_session().await.unwrap();
        assert!(!id.is_empty());

        memory.append(&id, Message::user("hi")).await.unwrap();

        let sessions = memory.list_sessions().await.unwrap();
        assert!(sessions.contains(&id));

        memory.delete_session(&id).await.unwrap();
        let after = memory.messages(&id).await.unwrap();
        assert!(after.is_empty());
    }

    #[test]
    fn test_in_memory_memory_default_ac5() {
        // AC-5: InMemoryMemory implements Default
        let _mem: InMemoryMemory = InMemoryMemory::default();
    }
}
