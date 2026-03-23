//! LLM Provider trait.
//!
//! The [`Provider`] trait abstracts communication with any LLM service.
//! Implement this trait to add support for a new LLM provider.

use async_trait::async_trait;

use crate::types::completion::{CompletionRequest, CompletionResponse};
use crate::types::stream::CompletionStream;
use crate::Result;

/// Information about the model being used.
#[derive(Debug, Clone)]
pub struct ModelInfo {
    /// Model name (e.g., "gpt-4o", "claude-sonnet").
    pub name: String,
    /// Model capability tier for steering auto-configuration.
    pub tier: ModelTier,
    /// Maximum context window size in tokens.
    pub context_window: usize,
    /// Whether the model supports tool/function calling.
    pub supports_tools: bool,
    /// Whether the model supports image/vision input.
    pub supports_vision: bool,
    /// Whether the model supports structured JSON output.
    pub supports_structured: bool,
}

/// Model capability tier used by the steering system to auto-configure
/// Guard, Hint, and Tracker behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelTier {
    /// Small models (e.g., Haiku, Phi, Gemma).
    /// Need aggressive steering: hints every 3 iterations, serial concurrency.
    Small,
    /// Medium models (e.g., Sonnet, GPT-4o-mini).
    /// Moderate steering: hints every 6 iterations, limited concurrency.
    Medium,
    /// Large models (e.g., Opus, GPT-4o, Gemini Ultra).
    /// Light steering: hints every 12 iterations, full concurrency.
    Large,
}

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
