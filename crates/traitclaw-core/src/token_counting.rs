//! Token counting utilities for context management.
//!
//! Provides both approximate (character-based) and accurate (tiktoken-based)
//! token estimation for message lists.

use crate::types::message::Message;

/// Approximate token counter using the 4-characters ≈ 1-token heuristic.
///
/// Fast and allocation-free. Suitable for most use cases where exact
/// token counts are not required.
///
/// # Example
///
/// ```rust
/// use traitclaw_core::token_counting::CharApproxCounter;
/// use traitclaw_core::types::message::{Message, MessageRole};
///
/// let counter = CharApproxCounter::new(4);
/// let messages = vec![
///     Message { role: MessageRole::User, content: "Hello world!".to_string(), tool_call_id: None },
/// ];
/// let tokens = counter.count(&messages);
/// assert_eq!(tokens, 4); // 12 chars / 4 + 1 = 4
/// ```
pub struct CharApproxCounter {
    /// Characters per token ratio.
    chars_per_token: usize,
}

impl CharApproxCounter {
    /// Create a new counter with the given characters-per-token ratio.
    ///
    /// Common values: 4 (English), 3 (CJK-heavy), 2 (code-heavy).
    #[must_use]
    pub fn new(chars_per_token: usize) -> Self {
        Self {
            chars_per_token: chars_per_token.max(1),
        }
    }

    /// Count tokens in a message list.
    #[must_use]
    pub fn count(&self, messages: &[Message]) -> usize {
        messages
            .iter()
            .map(|m| m.content.len() / self.chars_per_token + 1)
            .sum()
    }

    /// Count tokens in a single string.
    #[must_use]
    pub fn count_str(&self, text: &str) -> usize {
        text.len() / self.chars_per_token + 1
    }
}

impl Default for CharApproxCounter {
    fn default() -> Self {
        Self::new(4)
    }
}

/// Trait for pluggable token counting backends.
///
/// Implement this trait to provide accurate token counting for specific
/// models (e.g., tiktoken for OpenAI models).
pub trait TokenCounter: Send + Sync {
    /// Count tokens in a message list.
    fn count_messages(&self, messages: &[Message]) -> usize;

    /// Count tokens in a single string.
    fn count_str(&self, text: &str) -> usize;
}

impl TokenCounter for CharApproxCounter {
    fn count_messages(&self, messages: &[Message]) -> usize {
        self.count(messages)
    }

    fn count_str(&self, text: &str) -> usize {
        CharApproxCounter::count_str(self, text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::message::MessageRole;

    fn msg(content: &str) -> Message {
        Message {
            role: MessageRole::User,
            content: content.to_string(),
            tool_call_id: None,
        }
    }

    #[test]
    fn test_char_approx_default() {
        let counter = CharApproxCounter::default();
        // 12 chars / 4 + 1 = 4
        assert_eq!(counter.count_str("Hello world!"), 4);
    }

    #[test]
    fn test_char_approx_custom_ratio() {
        let counter = CharApproxCounter::new(2);
        // 12 chars / 2 + 1 = 7
        assert_eq!(counter.count_str("Hello world!"), 7);
    }

    #[test]
    fn test_char_approx_messages() {
        let counter = CharApproxCounter::default();
        let messages = vec![msg("aaaa"), msg("bbbbbbbb")]; // 4/4+1=2, 8/4+1=3 → 5
        assert_eq!(counter.count(&messages), 5);
    }

    #[test]
    fn test_char_approx_empty() {
        let counter = CharApproxCounter::default();
        assert_eq!(counter.count(&[]), 0);
        // empty string: 0/4 + 1 = 1
        assert_eq!(counter.count_str(""), 1);
    }

    #[test]
    fn test_char_approx_zero_ratio_clamped() {
        let counter = CharApproxCounter::new(0);
        // Should clamp to 1
        assert_eq!(counter.count_str("abcd"), 5); // 4/1 + 1 = 5
    }

    #[test]
    fn test_token_counter_trait() {
        let counter = CharApproxCounter::default();
        let tc: &dyn TokenCounter = &counter;
        assert_eq!(tc.count_str("abcdefgh"), 3); // 8/4 + 1 = 3
    }
}
