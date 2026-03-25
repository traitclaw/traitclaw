//! Async context window management — the v0.3.0 evolution of [`ContextStrategy`].
//!
//! [`ContextManager`] is the async replacement for the sync [`ContextStrategy`] trait.
//! It supports LLM-powered compression and accurate token counting.
//!
//! A blanket implementation is provided so that any existing [`ContextStrategy`]
//! implementation automatically works as a [`ContextManager`] with zero code changes.
//!
//! # Example
//!
//! ```rust
//! use traitclaw_core::traits::context_manager::ContextManager;
//! use traitclaw_core::types::message::Message;
//! use traitclaw_core::types::agent_state::AgentState;
//! use async_trait::async_trait;
//!
//! struct MyCompressor;
//!
//! #[async_trait]
//! impl ContextManager for MyCompressor {
//!     async fn prepare(
//!         &self,
//!         messages: &mut Vec<Message>,
//!         context_window: usize,
//!         state: &mut AgentState,
//!     ) {
//!         // Custom async compression logic
//!         let tokens = self.estimate_tokens(messages);
//!         if tokens > context_window {
//!             // Compress...
//!         }
//!     }
//! }
//! ```

use async_trait::async_trait;

#[allow(deprecated)]
use crate::traits::context_strategy::ContextStrategy;
use crate::types::agent_state::AgentState;
use crate::types::message::Message;

/// Async trait for pluggable context window management.
///
/// Called before each LLM request to ensure the message list fits within
/// the model's context window. Supports async operations such as
/// LLM-powered summarization and external token-counting APIs.
///
/// Implementations MUST NOT remove system messages.
///
/// # Migration from `ContextStrategy`
///
/// `ContextManager` replaces the sync [`ContextStrategy`] trait.
/// Existing `ContextStrategy` implementations work automatically via a blanket impl.
/// See the [migration guide](https://github.com/traitclaw/traitclaw/docs/migration-v0.2-to-v0.3.md)
/// for details.
#[async_trait]
pub trait ContextManager: Send + Sync {
    /// Prepare the message list by pruning or compressing if necessary.
    ///
    /// `context_window` is the model's maximum token capacity.
    /// This method is async to support LLM-powered compression strategies.
    async fn prepare(
        &self,
        messages: &mut Vec<Message>,
        context_window: usize,
        state: &mut AgentState,
    );

    /// Estimate the total token count for a message list.
    ///
    /// Default implementation uses the 4-characters ≈ 1-token approximation.
    /// Override with [`TikTokenCounter`] for model-accurate counting.
    fn estimate_tokens(&self, messages: &[Message]) -> usize {
        messages.iter().map(|m| m.content.len() / 4 + 1).sum()
    }
}

// ---------------------------------------------------------------------------
// Blanket impl: any ContextStrategy automatically becomes a ContextManager
// ---------------------------------------------------------------------------

#[allow(deprecated)]
#[async_trait]
impl<T: ContextStrategy + 'static> ContextManager for T {
    async fn prepare(
        &self,
        messages: &mut Vec<Message>,
        context_window: usize,
        state: &mut AgentState,
    ) {
        // Delegate to the sync ContextStrategy::prepare
        ContextStrategy::prepare(self, messages, context_window, state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::model_info::ModelTier;
    use std::sync::Arc;

    // ── Object safety: confirm Arc<dyn ContextManager> compiles ──────────
    #[test]
    fn test_context_manager_is_object_safe() {
        struct Dummy;

        #[async_trait]
        impl ContextManager for Dummy {
            async fn prepare(
                &self,
                _messages: &mut Vec<Message>,
                _context_window: usize,
                _state: &mut AgentState,
            ) {
            }
        }

        let _: Arc<dyn ContextManager> = Arc::new(Dummy);
    }

    // ── Blanket impl: ContextStrategy → ContextManager ──────────────────
    #[tokio::test]
    async fn test_blanket_impl_delegates_to_context_strategy() {
        #[allow(deprecated)]
        use crate::traits::context_strategy::SlidingWindowStrategy;
        use crate::types::message::MessageRole;

        let strategy = SlidingWindowStrategy::default();
        let mut messages = vec![
            Message {
                role: MessageRole::System,
                content: "system".to_string(),
                tool_call_id: None,
            },
            Message {
                role: MessageRole::User,
                content: "x".repeat(8000),
                tool_call_id: None,
            },
            Message {
                role: MessageRole::Assistant,
                content: "y".repeat(8000),
                tool_call_id: None,
            },
        ];
        let mut state = AgentState::new(ModelTier::Small, 4096);

        // Call through the ContextManager trait (blanket impl)
        ContextManager::prepare(&strategy, &mut messages, 2000, &mut state).await;

        // SlidingWindowStrategy should have removed some messages
        assert!(
            messages.len() < 3,
            "blanket impl should delegate to sync prepare"
        );
        assert_eq!(
            messages[0].role,
            MessageRole::System,
            "system message preserved"
        );
    }

    // ── Default estimate_tokens() ───────────────────────────────────────
    #[test]
    fn test_default_estimate_tokens() {
        struct Dummy;

        #[async_trait]
        impl ContextManager for Dummy {
            async fn prepare(
                &self,
                _messages: &mut Vec<Message>,
                _context_window: usize,
                _state: &mut AgentState,
            ) {
            }
        }

        let cm = Dummy;
        let messages = vec![
            Message {
                role: crate::types::message::MessageRole::User,
                content: "a".repeat(400), // 400 chars → 400/4 + 1 = 101 tokens
                tool_call_id: None,
            },
            Message {
                role: crate::types::message::MessageRole::Assistant,
                content: "b".repeat(800), // 800 chars → 800/4 + 1 = 201 tokens
                tool_call_id: None,
            },
        ];

        let tokens = cm.estimate_tokens(&messages);
        assert_eq!(
            tokens, 302,
            "4-chars ≈ 1-token: (400/4+1) + (800/4+1) = 302"
        );
    }
}
