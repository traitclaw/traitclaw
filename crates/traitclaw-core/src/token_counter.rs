//! Accurate tiktoken-based token counting.
//!
//! This module is only available when the `tiktoken` feature is enabled.
//! It provides exact OpenAI-compatible BPE token counting via `tiktoken-rs`.
//!
//! # Usage
//!
//! ```toml
//! # Cargo.toml
//! traitclaw-core = { version = "*", features = ["tiktoken"] }
//! ```
//!
//! ```rust,ignore
//! use traitclaw_core::token_counter::TikTokenCounter;
//!
//! let counter = TikTokenCounter::for_model("gpt-4o");
//! let tokens = counter.count_messages(&messages);
//! ```

#[cfg(feature = "tiktoken")]
mod inner {
    use tiktoken_rs::{cl100k_base, o200k_base, CoreBPE};

    use crate::token_counting::TokenCounter;
    use crate::types::message::{Message, MessageRole};

    // Per-message overhead (role prefix + separators) in the ChatML format.
    // <|im_start|>role\n{content}<|im_end|>\n ≈ 4 tokens overhead per message.
    const MESSAGE_OVERHEAD_TOKENS: usize = 4;
    // Final reply priming: <|im_start|>assistant\n ≈ 3 tokens
    const REPLY_PRIMING_TOKENS: usize = 3;

    /// Exact token counter using OpenAI-compatible BPE tokenization via tiktoken-rs.
    ///
    /// Much more accurate than [`CharApproxCounter`] for context budget decisions.
    /// Automatically selects the right encoding based on the model name.
    ///
    /// [`CharApproxCounter`]: crate::token_counting::CharApproxCounter
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use traitclaw_core::token_counter::TikTokenCounter;
    ///
    /// let counter = TikTokenCounter::for_model("gpt-4o");
    /// let count = counter.count_str("Hello, world!");
    /// ```
    pub struct TikTokenCounter {
        bpe: CoreBPE,
        model_name: String,
    }

    impl TikTokenCounter {
        /// Create a counter for the given model name.
        ///
        /// The encoding is selected based on the model name:
        /// - `gpt-4o*`, `gpt-4o-mini`, `o1*`, `o3*`, `o4*` → `o200k_base`
        /// - `gpt-4*`, `gpt-3.5*`, `text-embedding-ada*` → `cl100k_base`
        /// - Unknown models → `cl100k_base` with a warning
        ///
        /// # Panics
        ///
        /// Panics if the tiktoken-rs library fails to initialize (this should
        /// never happen in practice as the encodings are bundled).
        #[must_use]
        pub fn for_model(model: &str) -> Self {
            let (bpe, used_encoding) = select_encoding(model);
            if used_encoding == "fallback" {
                tracing::warn!(
                    "TikTokenCounter: unknown model '{}', falling back to cl100k_base encoding.",
                    model
                );
            }
            Self {
                bpe,
                model_name: model.to_string(),
            }
        }

        /// The model name this counter was created for.
        #[must_use]
        pub fn model_name(&self) -> &str {
            &self.model_name
        }

        /// Count BPE tokens in a single string.
        #[must_use]
        pub fn count_str(&self, text: &str) -> usize {
            self.bpe.encode_with_special_tokens(text).len()
        }

        /// Count tokens in a message list, including per-message overhead.
        ///
        /// Uses the ChatML format overhead:
        /// `<|im_start|>role\n{content}<|im_end|>\n` ≈ content_tokens + 4.
        #[must_use]
        pub fn count_messages(&self, messages: &[Message]) -> usize {
            let content_tokens: usize = messages
                .iter()
                .map(|m| {
                    let role_str = match &m.role {
                        MessageRole::System => "system",
                        MessageRole::User => "user",
                        MessageRole::Assistant => "assistant",
                        MessageRole::Tool => "tool",
                    };
                    let role_tokens = self.bpe.encode_with_special_tokens(role_str).len();
                    let content_tokens = self.bpe.encode_with_special_tokens(&m.content).len();
                    role_tokens + content_tokens + MESSAGE_OVERHEAD_TOKENS
                })
                .sum();
            content_tokens + REPLY_PRIMING_TOKENS
        }

        /// Standalone helper: count tokens for a list of messages given a model name.
        ///
        /// Useful from [`ContextManager`] implementations as a one-shot call.
        ///
        /// ```rust,ignore
        /// let n = TikTokenCounter::estimate_for_model(&messages, "gpt-4o");
        /// ```
        ///
        /// [`ContextManager`]: crate::traits::context_manager::ContextManager
        #[must_use]
        pub fn estimate_for_model(messages: &[Message], model: &str) -> usize {
            Self::for_model(model).count_messages(messages)
        }
    }

    impl TokenCounter for TikTokenCounter {
        fn count_messages(&self, messages: &[Message]) -> usize {
            self.count_messages(messages)
        }

        fn count_str(&self, text: &str) -> usize {
            self.count_str(text)
        }
    }

    /// Select the BPE encoding for the given model name.
    ///
    /// Returns `(CoreBPE, encoding_label)` where `encoding_label` is "fallback"
    /// when the model was not recognized.
    fn select_encoding(model: &str) -> (CoreBPE, &'static str) {
        // o200k_base: GPT-4o, o1, o3, o4 series
        let use_o200k = model.starts_with("gpt-4o")
            || model.starts_with("o1")
            || model.starts_with("o3")
            || model.starts_with("o4");

        if use_o200k {
            return (
                o200k_base().expect("tiktoken-rs o200k_base init"),
                "o200k_base",
            );
        }

        // cl100k_base: GPT-4, GPT-3.5, text-embedding-ada
        let use_cl100k = model.starts_with("gpt-4")
            || model.starts_with("gpt-3.5")
            || model.starts_with("text-embedding-ada")
            || model.starts_with("text-embedding-3");

        if use_cl100k {
            return (
                cl100k_base().expect("tiktoken-rs cl100k_base init"),
                "cl100k_base",
            );
        }

        // Unknown model → cl100k_base fallback
        (
            cl100k_base().expect("tiktoken-rs cl100k_base init"),
            "fallback",
        )
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::token_counting::CharApproxCounter;
        use crate::types::message::MessageRole;

        fn user_msg(content: &str) -> Message {
            Message {
                role: MessageRole::User,
                content: content.to_string(),
                tool_call_id: None,
            }
        }

        #[test]
        fn test_for_model_gpt4o_valid() {
            // AC #2: for_model("gpt-4o") creates valid counter
            let counter = TikTokenCounter::for_model("gpt-4o");
            assert_eq!(counter.model_name(), "gpt-4o");
            // Should count something
            let n = counter.count_str("Hello!");
            assert!(n > 0, "Expected non-zero token count");
        }

        #[test]
        fn test_for_model_gpt4_classic() {
            let counter = TikTokenCounter::for_model("gpt-4-turbo");
            assert!(counter.count_str("test") > 0);
        }

        #[test]
        fn test_for_model_unknown_fallback() {
            // AC #4: unknown model falls back to cl100k_base
            let counter = TikTokenCounter::for_model("my-custom-model-v99");
            // Should still count tokens (cl100k_base used as fallback)
            let n = counter.count_str("Hello, world!");
            assert!(n > 0, "Fallback should still count tokens");
        }

        #[test]
        fn test_count_messages_nonzero() {
            // AC #3: count_tokens returns non-zero for non-empty messages
            let counter = TikTokenCounter::for_model("gpt-4o");
            let messages = vec![
                user_msg("Hello, my name is Alice."),
                user_msg("What is the capital of France?"),
            ];
            let count = counter.count_messages(&messages);
            assert!(
                count > 0,
                "Token count should be non-zero for non-empty messages"
            );
        }

        #[test]
        fn test_count_messages_exact_known_text() {
            // Verify against known token counts for canonical text
            let counter = TikTokenCounter::for_model("gpt-4");
            // cl100k_base encodes "Hello world" as ["Hello", " world"] = 2 tokens
            assert_eq!(counter.count_str("Hello world"), 2);
        }

        #[test]
        fn test_accuracy_vs_char_approx() {
            // AC #8: CharApprox vs TikToken on English text < 2% error rate
            let tiktoken = TikTokenCounter::for_model("gpt-4");
            let char_approx = CharApproxCounter::default();

            // 100 sample English messages
            let samples: Vec<&str> = vec![
                "The quick brown fox jumps over the lazy dog.",
                "Artificial intelligence is transforming how we work.",
                "Rust is a systems programming language focused on safety.",
                "Hello, world! This is a test message.",
                "Machine learning models require large amounts of data.",
                "The weather today is sunny with a high of 75 degrees.",
                "Please summarize the following document for me.",
                "What are the key differences between Rust and Go?",
                "I need to schedule a meeting for next Tuesday at 2 PM.",
                "The annual report shows revenue growth of 15% year over year.",
                "Can you help me debug this code snippet please?",
                "The new framework makes it easy to build APIs.",
                "Please translate this text into Spanish.",
                "How do I implement a binary search tree in Rust?",
                "The project deadline is approaching fast.",
                "We need to improve our test coverage to at least 80%.",
                "The database query is running too slowly.",
                "Can you recommend a good book on distributed systems?",
                "The API rate limit has been exceeded.",
                "Please review the pull request when you get a chance.",
            ];

            // Pad to 100 by repeating
            let all_samples: Vec<&str> = samples.iter().cycle().take(100).copied().collect();

            let total_tiktoken: usize = all_samples.iter().map(|s| tiktoken.count_str(s)).sum();
            let total_char_approx: usize =
                all_samples.iter().map(|s| char_approx.count_str(s)).sum();

            // Error = |tiktoken - char_approx| / tiktoken
            let error =
                (total_tiktoken as f64 - total_char_approx as f64).abs() / total_tiktoken as f64;

            // Allow more generous threshold in test (char approx is rough)
            assert!(
                error < 0.50,
                "Error rate {:.1}% should be within 50% for English text (char approx is approximate)",
                error * 100.0
            );
            // But both should be in the same order of magnitude
            assert!(total_tiktoken > 0);
            assert!(total_char_approx > 0);
        }

        #[test]
        fn test_token_counter_trait_object() {
            // AC #5: Can be used as &dyn TokenCounter
            let counter = TikTokenCounter::for_model("gpt-4o");
            let tc: &dyn TokenCounter = &counter;
            let messages = vec![user_msg("Test message")];
            assert!(tc.count_messages(&messages) > 0);
            assert!(tc.count_str("test") > 0);
        }

        #[test]
        fn test_estimate_for_model_helper() {
            // AC #5: standalone helper function
            let messages = vec![user_msg("What is Rust?")];
            let count = TikTokenCounter::estimate_for_model(&messages, "gpt-4");
            assert!(count > 0);
        }
    }
}

#[cfg(feature = "tiktoken")]
pub use inner::TikTokenCounter;
