//! Mock LLM provider for deterministic testing.
//!
//! [`MockProvider`] returns pre-defined [`CompletionResponse`] values in
//! sequence, enabling fully deterministic tests without real API calls.
//!
//! When the sequence is exhausted, subsequent calls return the **last**
//! response (clamp-to-end behavior).
//!
//! # Quick Start
//!
//! ```rust
//! use traitclaw_test_utils::provider::MockProvider;
//!
//! // Single text response — used for simple agent tests
//! let p = MockProvider::text("Hello!");
//!
//! // Tool call → text — used for ReAct loop tests
//! # use traitclaw_core::types::tool_call::ToolCall;
//! let p = MockProvider::tool_then_text(
//!     vec![ToolCall {
//!         id: "1".into(),
//!         name: "echo".into(),
//!         arguments: r#"{"text":"hi"}"#.into(),
//!     }],
//!     "Done.",
//! );
//! ```

use std::sync::atomic::{AtomicUsize, Ordering};

use async_trait::async_trait;

use traitclaw_core::traits::provider::Provider;
use traitclaw_core::types::completion::{
    CompletionRequest, CompletionResponse, ResponseContent, Usage,
};
use traitclaw_core::types::model_info::{ModelInfo, ModelTier};
use traitclaw_core::types::stream::CompletionStream;
use traitclaw_core::types::tool_call::ToolCall;
use traitclaw_core::{Error, Result};

/// Default usage stats for mock responses.
fn default_usage() -> Usage {
    Usage {
        prompt_tokens: 10,
        completion_tokens: 5,
        total_tokens: 15,
    }
}

/// Deterministic mock provider that returns responses in sequence.
///
/// Each call to [`complete()`](Provider::complete) returns the next
/// response from the internal sequence. When all responses have been
/// returned, subsequent calls return the **last** response (clamp
/// behavior, not wrap-around).
///
/// # Thread Safety
///
/// `MockProvider` is [`Send`] + [`Sync`] by design — it uses
/// [`AtomicUsize`] for lock-free call indexing.
///
/// # Example
///
/// ```rust
/// use traitclaw_test_utils::provider::MockProvider;
///
/// let p = MockProvider::text("hello");
/// // p.complete(req).await returns "hello" every time
/// ```
pub struct MockProvider {
    /// Model information returned by [`Provider::model_info`].
    pub info: ModelInfo,
    /// Ordered list of responses to return.
    pub responses: Vec<CompletionResponse>,
    /// Tracks the current position in the response sequence.
    call_idx: AtomicUsize,
    /// Optional: return an error instead of a response.
    error_message: Option<String>,
}

impl MockProvider {
    /// Create a provider that always returns a single text response.
    ///
    /// # Example
    ///
    /// ```rust
    /// use traitclaw_test_utils::provider::MockProvider;
    ///
    /// let p = MockProvider::text("I am a mock LLM");
    /// ```
    pub fn text(text: &str) -> Self {
        Self {
            info: ModelInfo::new("mock-model", ModelTier::Small, 4096, false, false, false),
            responses: vec![CompletionResponse {
                content: ResponseContent::Text(text.into()),
                usage: default_usage(),
            }],
            call_idx: AtomicUsize::new(0),
            error_message: None,
        }
    }

    /// Create a provider with an explicit sequence of responses.
    ///
    /// Responses are returned in order. Once exhausted, the last
    /// response is repeated.
    ///
    /// # Panics
    ///
    /// Panics if `responses` is empty.
    ///
    /// # Example
    ///
    /// ```rust
    /// use traitclaw_test_utils::provider::MockProvider;
    /// use traitclaw_core::types::completion::{CompletionResponse, ResponseContent, Usage};
    ///
    /// let p = MockProvider::sequence(vec![
    ///     CompletionResponse {
    ///         content: ResponseContent::Text("first".into()),
    ///         usage: Usage { prompt_tokens: 10, completion_tokens: 5, total_tokens: 15 },
    ///     },
    ///     CompletionResponse {
    ///         content: ResponseContent::Text("second".into()),
    ///         usage: Usage { prompt_tokens: 10, completion_tokens: 5, total_tokens: 15 },
    ///     },
    /// ]);
    /// ```
    pub fn sequence(responses: Vec<CompletionResponse>) -> Self {
        assert!(
            !responses.is_empty(),
            "MockProvider::sequence requires at least one response"
        );
        Self {
            info: ModelInfo::new("mock-model", ModelTier::Small, 4096, true, false, false),
            responses,
            call_idx: AtomicUsize::new(0),
            error_message: None,
        }
    }

    /// Create a provider that returns tool calls first, then a final text response.
    ///
    /// This is the standard pattern for testing ReAct-style agent loops.
    ///
    /// # Example
    ///
    /// ```rust
    /// use traitclaw_test_utils::provider::MockProvider;
    /// use traitclaw_core::types::tool_call::ToolCall;
    ///
    /// let p = MockProvider::tool_then_text(
    ///     vec![ToolCall {
    ///         id: "call_1".into(),
    ///         name: "search".into(),
    ///         arguments: r#"{"query":"rust"}"#.into(),
    ///     }],
    ///     "Here are the results.",
    /// );
    /// ```
    pub fn tool_then_text(tool_calls: Vec<ToolCall>, final_text: &str) -> Self {
        Self::sequence(vec![
            CompletionResponse {
                content: ResponseContent::ToolCalls(tool_calls),
                usage: default_usage(),
            },
            CompletionResponse {
                content: ResponseContent::Text(final_text.into()),
                usage: default_usage(),
            },
        ])
    }

    /// Create a provider that always returns tool calls (never text).
    ///
    /// Useful for testing tool-budget guards and loop detection.
    pub fn always_tool_calls(tool_calls: Vec<ToolCall>) -> Self {
        Self {
            info: ModelInfo::new("mock-model", ModelTier::Small, 4096, true, false, false),
            responses: vec![CompletionResponse {
                content: ResponseContent::ToolCalls(tool_calls),
                usage: default_usage(),
            }],
            call_idx: AtomicUsize::new(0),
            error_message: None,
        }
    }

    /// Create a provider that always returns an error.
    ///
    /// Useful for testing error handling paths in strategies and agents.
    ///
    /// # Example
    ///
    /// ```rust
    /// use traitclaw_test_utils::provider::MockProvider;
    ///
    /// let p = MockProvider::error("API rate limit exceeded");
    /// // p.complete(req).await will return Err(Error::Runtime(...))
    /// ```
    pub fn error(msg: &str) -> Self {
        Self {
            info: ModelInfo::new("mock-model", ModelTier::Small, 4096, false, false, false),
            responses: vec![],
            call_idx: AtomicUsize::new(0),
            error_message: Some(msg.to_string()),
        }
    }

    /// Returns how many times `complete()` has been called.
    pub fn call_count(&self) -> usize {
        self.call_idx.load(Ordering::SeqCst)
    }
}

#[async_trait]
impl Provider for MockProvider {
    async fn complete(&self, _req: CompletionRequest) -> Result<CompletionResponse> {
        // Error path — always returns error if configured
        if let Some(msg) = &self.error_message {
            return Err(Error::Runtime(msg.clone()));
        }

        let idx = self.call_idx.fetch_add(1, Ordering::SeqCst);
        Ok(self.responses[idx.min(self.responses.len() - 1)].clone())
    }

    async fn stream(&self, _req: CompletionRequest) -> Result<CompletionStream> {
        unimplemented!("MockProvider does not support streaming")
    }

    fn model_info(&self) -> &ModelInfo {
        &self.info
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use traitclaw_core::types::message::Message;

    fn dummy_request() -> CompletionRequest {
        CompletionRequest {
            model: "mock-model".to_string(),
            messages: vec![Message::user("test")],
            tools: vec![],
            max_tokens: None,
            temperature: None,
            response_format: None,
            stream: false,
        }
    }

    #[tokio::test]
    async fn test_text_returns_correct_response() {
        let p = MockProvider::text("hello");
        let resp = p.complete(dummy_request()).await.unwrap();
        match resp.content {
            ResponseContent::Text(t) => assert_eq!(t, "hello"),
            ResponseContent::ToolCalls(_) => panic!("expected Text"),
        }
    }

    #[tokio::test]
    async fn test_text_returns_same_response_on_multiple_calls() {
        let p = MockProvider::text("constant");
        for _ in 0..5 {
            let resp = p.complete(dummy_request()).await.unwrap();
            match &resp.content {
                ResponseContent::Text(t) => assert_eq!(t, "constant"),
                ResponseContent::ToolCalls(_) => panic!("expected Text"),
            }
        }
        assert_eq!(p.call_count(), 5);
    }

    #[tokio::test]
    async fn test_sequence_returns_in_order() {
        let p = MockProvider::sequence(vec![
            CompletionResponse {
                content: ResponseContent::Text("first".into()),
                usage: default_usage(),
            },
            CompletionResponse {
                content: ResponseContent::Text("second".into()),
                usage: default_usage(),
            },
        ]);

        let r1 = p.complete(dummy_request()).await.unwrap();
        let r2 = p.complete(dummy_request()).await.unwrap();

        match r1.content {
            ResponseContent::Text(t) => assert_eq!(t, "first"),
            _ => panic!("expected first"),
        }
        match r2.content {
            ResponseContent::Text(t) => assert_eq!(t, "second"),
            _ => panic!("expected second"),
        }
    }

    #[tokio::test]
    async fn test_sequence_clamps_to_last_response() {
        let p = MockProvider::sequence(vec![
            CompletionResponse {
                content: ResponseContent::Text("only".into()),
                usage: default_usage(),
            },
            CompletionResponse {
                content: ResponseContent::Text("last".into()),
                usage: default_usage(),
            },
        ]);

        // Exhaust sequence
        let _ = p.complete(dummy_request()).await.unwrap(); // "only"
        let _ = p.complete(dummy_request()).await.unwrap(); // "last"

        // Beyond sequence — should clamp to "last"
        let r3 = p.complete(dummy_request()).await.unwrap();
        let r4 = p.complete(dummy_request()).await.unwrap();

        match r3.content {
            ResponseContent::Text(t) => assert_eq!(t, "last"),
            _ => panic!("expected last"),
        }
        match r4.content {
            ResponseContent::Text(t) => assert_eq!(t, "last"),
            _ => panic!("expected last"),
        }
    }

    #[tokio::test]
    async fn test_tool_then_text_returns_tool_calls_then_text() {
        let tool_call = ToolCall {
            id: "call_1".into(),
            name: "echo".into(),
            arguments: r#"{"text":"hi"}"#.into(),
        };
        let p = MockProvider::tool_then_text(vec![tool_call.clone()], "done");

        let r1 = p.complete(dummy_request()).await.unwrap();
        match &r1.content {
            ResponseContent::ToolCalls(calls) => {
                assert_eq!(calls.len(), 1);
                assert_eq!(calls[0].name, "echo");
            }
            ResponseContent::Text(_) => panic!("expected ToolCalls on first call"),
        }

        let r2 = p.complete(dummy_request()).await.unwrap();
        match r2.content {
            ResponseContent::Text(t) => assert_eq!(t, "done"),
            ResponseContent::ToolCalls(_) => panic!("expected Text on second call"),
        }
    }

    #[tokio::test]
    async fn test_error_returns_error() {
        let p = MockProvider::error("rate limited");
        let result = p.complete(dummy_request()).await;
        assert!(result.is_err());
        let err_str = result.unwrap_err().to_string();
        assert!(err_str.contains("rate limited"), "got: {err_str}");
    }

    #[tokio::test]
    async fn test_always_tool_calls_never_returns_text() {
        let tool_call = ToolCall {
            id: "1".into(),
            name: "search".into(),
            arguments: "{}".into(),
        };
        let p = MockProvider::always_tool_calls(vec![tool_call]);

        for _ in 0..3 {
            let resp = p.complete(dummy_request()).await.unwrap();
            assert!(
                matches!(resp.content, ResponseContent::ToolCalls(_)),
                "expected ToolCalls"
            );
        }
    }

    #[test]
    fn test_mock_provider_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<MockProvider>();
    }

    #[test]
    fn test_call_count_tracks_invocations() {
        let p = MockProvider::text("x");
        assert_eq!(p.call_count(), 0);
    }

    #[test]
    fn test_model_info_returns_expected_defaults() {
        let p = MockProvider::text("x");
        let info = p.model_info();
        assert_eq!(info.name, "mock-model");
        assert_eq!(info.context_window, 4096);
    }
}
