//! Context window management strategies.
//!
//! These strategies control how the message context is pruned to stay within
//! the model's context window limits.

use crate::types::agent_state::AgentState;
use crate::types::message::{Message, MessageRole};

/// Trait for pluggable context window management.
///
/// Called before each LLM request to ensure the message list fits within
/// the model's context window.
///
/// Implementations MUST NOT remove system messages.
pub trait ContextStrategy: Send + Sync {
    /// Prepare the message list by pruning if necessary.
    ///
    /// `context_window` is the model's maximum token capacity.
    fn prepare(&self, messages: &mut Vec<Message>, context_window: usize, state: &mut AgentState);
}

/// Does nothing — no automatic context management.
///
/// Use this when you want full manual control over context length.
pub struct NoopContextStrategy;

impl ContextStrategy for NoopContextStrategy {
    fn prepare(
        &self,
        _messages: &mut Vec<Message>,
        _context_window: usize,
        _state: &mut AgentState,
    ) {
    }
}

/// Sliding window strategy: removes oldest non-system messages when the
/// estimated token count exceeds a threshold fraction of the context window.
///
/// Token estimation uses the approximation: **4 characters ≈ 1 token**.
pub struct SlidingWindowStrategy {
    /// Fraction of `context_window` at which pruning begins (default: 0.85).
    threshold: f64,
}

impl Default for SlidingWindowStrategy {
    fn default() -> Self {
        Self { threshold: 0.85 }
    }
}

impl SlidingWindowStrategy {
    /// Create a strategy with a custom threshold (0.0–1.0).
    #[must_use]
    pub fn new(threshold: f64) -> Self {
        Self {
            threshold: threshold.clamp(0.0, 1.0),
        }
    }

    /// Estimate token count for a message list (4 chars ≈ 1 token).
    fn estimate_tokens(messages: &[Message]) -> usize {
        messages.iter().map(|m| m.content.len() / 4 + 1).sum()
    }
}

impl ContextStrategy for SlidingWindowStrategy {
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss
    )]
    fn prepare(&self, messages: &mut Vec<Message>, context_window: usize, state: &mut AgentState) {
        let max_tokens = (context_window as f64 * self.threshold) as usize;
        let mut removed = false;

        while Self::estimate_tokens(messages) > max_tokens && messages.len() > 1 {
            // Find the first non-system message to remove
            if let Some(idx) = messages.iter().position(|m| m.role != MessageRole::System) {
                messages.remove(idx);
                removed = true;
            } else {
                break; // Only system messages remain
            }
        }

        if removed {
            state.last_output_truncated = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::types::model_info::ModelTier;

    fn make_msg(role: MessageRole, content: &str) -> Message {
        Message {
            role,
            content: content.to_string(),
            tool_call_id: None,
        }
    }

    #[test]
    fn test_noop_does_nothing() {
        let strategy = NoopContextStrategy;
        let mut messages = vec![
            make_msg(MessageRole::System, "sys"),
            make_msg(MessageRole::User, &"x".repeat(10000)),
        ];
        let original_len = messages.len();
        let mut state = AgentState::new(ModelTier::Small, 4096);

        strategy.prepare(&mut messages, 100, &mut state);

        assert_eq!(messages.len(), original_len);
        assert!(!state.last_output_truncated);
    }

    #[test]
    fn test_sliding_window_removes_old_messages() {
        let strategy = SlidingWindowStrategy::default();
        let mut messages = vec![
            make_msg(MessageRole::System, "system prompt"),
            make_msg(MessageRole::User, &"a".repeat(4000)), // ~1000 tokens
            make_msg(MessageRole::Assistant, &"b".repeat(4000)), // ~1000 tokens
            make_msg(MessageRole::User, &"c".repeat(4000)), // ~1000 tokens
        ];
        let mut state = AgentState::new(ModelTier::Small, 4096);

        // context_window=2000, threshold=0.85 → max ~1700 tokens
        // Total ≈ 3000+ tokens → should remove oldest non-system messages
        strategy.prepare(&mut messages, 2000, &mut state);

        assert!(messages.len() < 4, "should have removed some messages");
        // System message must be preserved
        assert_eq!(messages[0].role, MessageRole::System);
        assert!(state.last_output_truncated);
    }

    #[test]
    fn test_sliding_window_preserves_system_messages() {
        let strategy = SlidingWindowStrategy::default();
        let mut messages = vec![
            make_msg(MessageRole::System, &"s".repeat(400)),
            make_msg(MessageRole::User, "hi"),
        ];
        let mut state = AgentState::new(ModelTier::Small, 4096);

        // Under threshold, nothing removed
        strategy.prepare(&mut messages, 10000, &mut state);

        assert_eq!(messages.len(), 2);
        assert!(!state.last_output_truncated);
    }

    #[test]
    fn test_agent_state_flag_set_on_truncation() {
        let strategy = SlidingWindowStrategy::new(0.5);
        let mut messages = vec![
            make_msg(MessageRole::System, "sys"),
            make_msg(MessageRole::User, &"x".repeat(8000)),
            make_msg(MessageRole::Assistant, &"y".repeat(8000)),
        ];
        let mut state = AgentState::new(ModelTier::Small, 4096);

        strategy.prepare(&mut messages, 1000, &mut state);

        assert!(state.last_output_truncated);
    }
}
