//! Async context window management.
//!
//! [`ContextManager`] provides pluggable, async context window management.
//! It supports LLM-powered compression and accurate token counting.
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

use crate::types::agent_state::AgentState;
use crate::types::message::Message;

/// Async trait for pluggable context window management.
///
/// Called before each LLM request to ensure the message list fits within
/// the model's context window. Supports async operations such as
/// LLM-powered summarization and external token-counting APIs.
///
/// Implementations MUST NOT remove system messages.
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
    /// Override with `TikTokenCounter` for model-accurate counting.
    fn estimate_tokens(&self, messages: &[Message]) -> usize {
        messages.iter().map(|m| m.content.len() / 4 + 1).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
