//! LLM Provider trait.
//!
//! The [`Provider`] trait abstracts communication with any LLM service.
//! Implement this trait to add support for a new LLM provider.

use async_trait::async_trait;

use crate::types::completion::{CompletionRequest, CompletionResponse};
use crate::types::stream::CompletionStream;
use crate::Result;

// Re-export for backward compatibility — callers who used `traits::provider::ModelInfo` continue to work.
pub use crate::types::model_info::{ModelInfo, ModelTier};

/// Trait for LLM providers.
///
/// Implement this trait to add support for any LLM service.
/// The provider handles the actual HTTP communication with the LLM API.
///
/// # Example
///
/// ```rust,no_run
/// use async_trait::async_trait;
/// use baseclaw_core::prelude::*;
///
/// struct MyProvider;
///
/// #[async_trait]
/// impl Provider for MyProvider {
///     async fn complete(&self, request: CompletionRequest) -> baseclaw_core::Result<CompletionResponse> {
///         todo!("Send request to LLM API")
///     }
///
///     async fn stream(&self, request: CompletionRequest) -> baseclaw_core::Result<CompletionStream> {
///         todo!("Stream response from LLM API")
///     }
///
///     fn model_info(&self) -> &ModelInfo {
///         todo!("Return model information")
///     }
/// }
/// ```
#[async_trait]
pub trait Provider: Send + Sync + 'static {
    /// Send a completion request and get a full response.
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse>;

    /// Send a completion request and get a streaming response.
    async fn stream(&self, request: CompletionRequest) -> Result<CompletionStream>;

    /// Get information about the model.
    fn model_info(&self) -> &ModelInfo;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::completion::{ResponseContent, Usage};

    // ─── MockProvider ────────────────────────────────────────────────────────

    struct MockProvider {
        info: ModelInfo,
        response: String,
    }

    impl MockProvider {
        fn new(response: impl Into<String>) -> Self {
            Self {
                info: ModelInfo {
                    name: "mock-model".to_string(),
                    tier: ModelTier::Medium,
                    context_window: 8_192,
                    supports_tools: true,
                    supports_vision: false,
                    supports_structured: true,
                },
                response: response.into(),
            }
        }
    }

    #[async_trait]
    impl Provider for MockProvider {
        async fn complete(&self, _request: CompletionRequest) -> Result<CompletionResponse> {
            Ok(CompletionResponse {
                content: ResponseContent::Text(self.response.clone()),
                usage: Usage {
                    prompt_tokens: 10,
                    completion_tokens: 5,
                    total_tokens: 15,
                },
            })
        }

        async fn stream(&self, _request: CompletionRequest) -> Result<CompletionStream> {
            let text = self.response.clone();
            let stream = tokio_stream::once(Ok(crate::types::stream::StreamEvent::TextDelta(text)));
            Ok(Box::pin(stream))
        }

        fn model_info(&self) -> &ModelInfo {
            &self.info
        }
    }

    // ─── Tests ───────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_mock_provider_complete_ac1() {
        // AC-1: complete() accepts CompletionRequest and returns Result<CompletionResponse>
        use crate::types::message::Message;

        let provider = MockProvider::new("Hello from LLM!");
        let request = CompletionRequest {
            model: "mock-model".to_string(),
            messages: vec![Message::user("Hi")],
            tools: vec![],
            max_tokens: None,
            temperature: None,
            response_format: None,
            stream: false,
        };

        let response = provider.complete(request).await.unwrap();
        match response.content {
            ResponseContent::Text(t) => assert_eq!(t, "Hello from LLM!"),
            ResponseContent::ToolCalls(_) => panic!("expected Text"),
        }
        assert_eq!(response.usage.total_tokens, 15);
    }

    #[tokio::test]
    async fn test_mock_provider_stream_ac2() {
        // AC-2: stream() returns Result<CompletionStream>
        use crate::types::message::Message;
        use crate::types::stream::StreamEvent;
        use tokio_stream::StreamExt;

        let provider = MockProvider::new("streamed chunk");
        let request = CompletionRequest {
            model: "mock-model".to_string(),
            messages: vec![Message::user("stream test")],
            tools: vec![],
            max_tokens: None,
            temperature: None,
            response_format: None,
            stream: true,
        };

        let mut stream = provider.stream(request).await.unwrap();
        let event = stream
            .next()
            .await
            .expect("expected at least one event")
            .unwrap();
        match event {
            StreamEvent::TextDelta(t) => assert_eq!(t, "streamed chunk"),
            _ => panic!("expected TextDelta"),
        }
    }

    #[test]
    fn test_mock_provider_model_info_ac3() {
        // AC-3: model_info() returns ModelInfo with expected fields
        let provider = MockProvider::new("x");
        let info = provider.model_info();
        assert_eq!(info.name, "mock-model");
        assert_eq!(info.tier, ModelTier::Medium);
        assert_eq!(info.context_window, 8_192);
        assert!(info.supports_tools);
        assert!(!info.supports_vision);
    }

    #[test]
    fn test_provider_send_sync_static_bounds_ac5() {
        // AC-5: Provider requires Send + Sync + 'static — verified at compile time
        fn assert_bounds<T: Send + Sync + 'static>() {}
        assert_bounds::<MockProvider>();
    }
}
