//! Mock memory backend for testing.
//!
//! [`MockMemory`] provides per-session message storage using
//! [`tokio::sync::Mutex`] for async-safe access in tests.
//!
//! Unlike the production [`InMemoryMemory`](traitclaw_core::memory::in_memory::InMemoryMemory),
//! this mock supports multi-session isolation via a `HashMap`.
//!
//! # Example
//!
//! ```rust
//! use traitclaw_test_utils::memory::MockMemory;
//!
//! let memory = MockMemory::new();
//! // Use with AgentRuntime or Agent for isolated test sessions
//! ```

use std::collections::HashMap;

use async_trait::async_trait;
use tokio::sync::Mutex;

use traitclaw_core::traits::memory::MemoryEntry;
use traitclaw_core::types::message::Message;
use traitclaw_core::{Memory, Result};

/// In-memory mock that stores messages per session.
///
/// Each session gets its own `Vec<Message>`, ensuring test
/// isolation when multiple sessions are used concurrently.
///
/// # Example
///
/// ```rust
/// use traitclaw_test_utils::memory::MockMemory;
/// use traitclaw_core::types::message::Message;
/// use traitclaw_core::Memory;
///
/// # tokio_test::block_on(async {
/// let mem = MockMemory::new();
/// mem.append("s1", Message::user("hi")).await.unwrap();
/// let msgs = mem.messages("s1").await.unwrap();
/// assert_eq!(msgs.len(), 1);
/// # });
/// ```
pub struct MockMemory {
    /// Per-session message storage.
    messages: Mutex<HashMap<String, Vec<Message>>>,
}

impl MockMemory {
    /// Create a new empty mock memory.
    pub fn new() -> Self {
        Self {
            messages: Mutex::new(HashMap::new()),
        }
    }
}

impl Default for MockMemory {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Memory for MockMemory {
    async fn messages(&self, session_id: &str) -> Result<Vec<Message>> {
        let store = self.messages.lock().await;
        Ok(store.get(session_id).cloned().unwrap_or_default())
    }

    async fn append(&self, session_id: &str, message: Message) -> Result<()> {
        let mut store = self.messages.lock().await;
        store
            .entry(session_id.to_string())
            .or_default()
            .push(message);
        Ok(())
    }

    async fn get_context(
        &self,
        _session_id: &str,
        _key: &str,
    ) -> Result<Option<serde_json::Value>> {
        Ok(None)
    }

    async fn set_context(
        &self,
        _session_id: &str,
        _key: &str,
        _value: serde_json::Value,
    ) -> Result<()> {
        Ok(())
    }

    async fn recall(&self, _query: &str, _limit: usize) -> Result<Vec<MemoryEntry>> {
        Ok(vec![])
    }

    async fn store(&self, _entry: MemoryEntry) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_new_memory_returns_empty_messages() {
        let mem = MockMemory::new();
        let msgs = mem.messages("session-1").await.unwrap();
        assert!(msgs.is_empty());
    }

    #[tokio::test]
    async fn test_append_and_retrieve() {
        let mem = MockMemory::new();
        mem.append("s1", Message::user("hello")).await.unwrap();
        mem.append("s1", Message::assistant("hi")).await.unwrap();

        let msgs = mem.messages("s1").await.unwrap();
        assert_eq!(msgs.len(), 2);
    }

    #[tokio::test]
    async fn test_sessions_are_isolated() {
        let mem = MockMemory::new();
        mem.append("s1", Message::user("one")).await.unwrap();
        mem.append("s2", Message::user("two")).await.unwrap();

        assert_eq!(mem.messages("s1").await.unwrap().len(), 1);
        assert_eq!(mem.messages("s2").await.unwrap().len(), 1);
        assert!(mem.messages("s3").await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_get_context_returns_none() {
        let mem = MockMemory::new();
        let ctx = mem.get_context("s1", "key").await.unwrap();
        assert!(ctx.is_none());
    }

    #[tokio::test]
    async fn test_recall_returns_empty() {
        let mem = MockMemory::new();
        let entries = mem.recall("query", 10).await.unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn test_mock_memory_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<MockMemory>();
    }
}
