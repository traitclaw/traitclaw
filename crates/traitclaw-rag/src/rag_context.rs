//! `RagContextManager` — automatic document retrieval and injection.
//!
//! Implements [`ContextManager`] from `traitclaw-core` so that a RAG pipeline
//! integrates transparently with the `Agent` builder.
//!
//! On every `prepare()` call, the manager:
//! 1. Extracts the last user message as the retrieval query
//! 2. Retrieves relevant documents via the configured [`Retriever`]
//! 3. Formats them with the configured [`GroundingStrategy`]
//! 4. Prepends the grounded context as a **system message**
//!
//! # Example
//!
//! ```rust
//! use traitclaw_rag::{Document, KeywordRetriever};
//! use traitclaw_rag::rag_context::RagContextManager;
//!
//! # async fn example() -> traitclaw_core::Result<()> {
//! let mut retriever = KeywordRetriever::new();
//! retriever.add(Document::new("doc1", "Rust is a systems language."));
//!
//! let manager = RagContextManager::new(retriever);
//! # Ok(())
//! # }
//! ```

use async_trait::async_trait;
use traitclaw_core::{
    traits::context_manager::ContextManager,
    types::{
        agent_state::AgentState,
        message::{Message, MessageRole},
    },
};

use crate::{GroundingStrategy, PrependStrategy, Retriever};

/// A [`ContextManager`] that retrieves documents and injects them as grounded context.
pub struct RagContextManager<R: Retriever, G: GroundingStrategy = PrependStrategy> {
    retriever: R,
    grounding: G,
    max_docs: usize,
}

impl<R: Retriever> RagContextManager<R, PrependStrategy> {
    /// Create a new `RagContextManager` with `PrependStrategy` as the default grounding.
    #[must_use]
    pub fn new(retriever: R) -> Self {
        Self {
            retriever,
            grounding: PrependStrategy,
            max_docs: 5,
        }
    }
}

impl<R: Retriever, G: GroundingStrategy> RagContextManager<R, G> {
    /// Set the grounding strategy used to format retrieved documents.
    #[must_use]
    pub fn with_grounding<G2: GroundingStrategy>(self, grounding: G2) -> RagContextManager<R, G2> {
        RagContextManager {
            retriever: self.retriever,
            grounding,
            max_docs: self.max_docs,
        }
    }

    /// Maximum number of retrieved documents to inject (default: 5).
    #[must_use]
    pub fn with_max_docs(mut self, max_docs: usize) -> Self {
        self.max_docs = max_docs;
        self
    }
}

#[async_trait]
impl<R: Retriever, G: GroundingStrategy> ContextManager for RagContextManager<R, G> {
    /// Prepare messages: retrieve docs for last user query, prepend as system message.
    async fn prepare(
        &self,
        messages: &mut Vec<Message>,
        _context_window: usize,
        _state: &mut AgentState,
    ) {
        // Extract the last user message as the query
        let query = messages
            .iter()
            .rev()
            .find(|m| m.role == MessageRole::User)
            .map(|m| m.content.clone())
            .unwrap_or_default();

        if query.is_empty() {
            return;
        }

        // Retrieve relevant documents
        let docs = match self.retriever.retrieve(&query, self.max_docs).await {
            Ok(d) => d,
            Err(_) => return, // fail silently — don't break the agent
        };

        if docs.is_empty() {
            return;
        }

        // Format with grounding strategy
        let grounded_context = self.grounding.ground(&docs);

        if grounded_context.is_empty() {
            return;
        }

        // Prepend as system message
        messages.insert(
            0,
            Message {
                role: MessageRole::System,
                content: grounded_context,
                tool_call_id: None,
            },
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use traitclaw_core::types::model_info::ModelTier;

    use super::*;
    use crate::{Document, KeywordRetriever};

    fn user_msg(content: &str) -> Message {
        Message {
            role: MessageRole::User,
            content: content.to_string(),
            tool_call_id: None,
        }
    }

    fn system_msg(content: &str) -> Message {
        Message {
            role: MessageRole::System,
            content: content.to_string(),
            tool_call_id: None,
        }
    }

    fn make_retriever(docs: Vec<(&str, &str)>) -> KeywordRetriever {
        let mut r = KeywordRetriever::new();
        for (id, content) in docs {
            r.add(Document::new(id, content));
        }
        r
    }

    fn state() -> AgentState {
        AgentState::new(ModelTier::Large, 128_000)
    }

    #[tokio::test]
    async fn test_rag_context_manager_prepends_grounding() {
        // AC #5, #6: retriever returns docs → context prepended as system message
        let retriever = make_retriever(vec![
            ("d1", "Rust is a systems language"),
            ("d2", "Rust has zero-cost abstractions"),
            ("d3", "Rust ownership model"),
        ]);

        let manager = RagContextManager::new(retriever);
        let mut messages = vec![user_msg("Tell me about Rust")];
        let mut st = state();
        manager.prepare(&mut messages, 128_000, &mut st).await;

        // First message should be a system message with context
        assert_eq!(messages[0].role, MessageRole::System);
        assert!(
            messages[0].content.contains("Rust"),
            "context should mention Rust"
        );
        // Original user message preserved at index 1
        assert_eq!(messages[1].role, MessageRole::User);
    }

    #[tokio::test]
    async fn test_rag_no_relevant_docs_unchanged() {
        // AC #7: no relevant docs → context unchanged
        let retriever = make_retriever(vec![("d1", "Python is great for data science")]);

        let manager = RagContextManager::new(retriever);
        let mut messages = vec![user_msg("Tell me about quantum computing")];
        let mut st = state();
        manager.prepare(&mut messages, 128_000, &mut st).await;

        // No system message prepended
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].role, MessageRole::User);
    }

    #[tokio::test]
    async fn test_rag_empty_messages_unchanged() {
        let retriever = make_retriever(vec![("d1", "some content")]);
        let manager = RagContextManager::new(retriever);
        let mut messages: Vec<Message> = vec![];
        let mut st = state();
        manager.prepare(&mut messages, 128_000, &mut st).await;
        assert!(messages.is_empty());
    }

    #[tokio::test]
    async fn test_rag_max_docs_limits_injection() {
        // AC #3: with_max_docs(1) → only 1 doc injected
        let retriever = make_retriever(vec![
            ("d1", "Rust systems programming"),
            ("d2", "Rust async programming"),
            ("d3", "Rust embedded programming"),
        ]);

        let manager = RagContextManager::new(retriever).with_max_docs(1);
        let mut messages = vec![user_msg("Rust programming")];
        let mut st = state();
        manager.prepare(&mut messages, 128_000, &mut st).await;

        assert_eq!(messages[0].role, MessageRole::System);
        // PrependStrategy format: "[1] content\n\n[2] content..."
        // With max_docs=1 there should only be [1] and no [2]
        assert!(
            messages[0].content.contains("[1]"),
            "should have first citation"
        );
        assert!(
            !messages[0].content.contains("[2]"),
            "should not have second citation with max_docs=1"
        );
    }

    #[tokio::test]
    async fn test_rag_user_message_found_among_others() {
        // query is extracted from last user message even when system messages present
        let retriever = make_retriever(vec![("d1", "Rust systems programming")]);
        let manager = RagContextManager::new(retriever);

        let mut messages = vec![
            system_msg("You are a helpful assistant"),
            user_msg("Tell me about Rust"),
        ];
        let mut st = state();
        manager.prepare(&mut messages, 128_000, &mut st).await;

        // System context prepended (now at index 0, original system at 1)
        assert!(messages.len() >= 2);
        assert_eq!(messages[0].role, MessageRole::System);
    }

    #[tokio::test]
    async fn test_rag_implements_context_manager_trait() {
        // Can be used as Arc<dyn ContextManager>
        let retriever = KeywordRetriever::new();
        let manager = RagContextManager::new(retriever);
        let _: Arc<dyn ContextManager> = Arc::new(manager);
    }
}
