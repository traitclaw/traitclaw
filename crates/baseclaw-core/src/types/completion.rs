//! Completion request and response types.

use serde::{Deserialize, Serialize};

use crate::types::message::Message;
use crate::types::tool_call::ToolCall;
use crate::traits::tool::ToolSchema;

/// A completion request to send to an LLM provider.
#[derive(Debug, Clone, Serialize)]
pub struct CompletionRequest {
    /// The model to use.
    pub model: String,
    /// Conversation messages.
    pub messages: Vec<Message>,
    /// Available tools (empty if no tools).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<ToolSchema>,
    /// Maximum tokens in the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    /// Sampling temperature.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    /// Whether to stream the response.
    #[serde(skip)]
    pub stream: bool,
}

/// The content of a completion response.
#[derive(Debug, Clone, Deserialize)]
pub enum ResponseContent {
    /// A text response from the LLM.
    Text(String),
    /// Tool calls requested by the LLM.
    ToolCalls(Vec<ToolCall>),
}

/// A completion response from the LLM.
#[derive(Debug, Clone)]
pub struct CompletionResponse {
    /// The response content (text or tool calls).
    pub content: ResponseContent,
    /// Token usage statistics.
    pub usage: Usage,
}

/// Token usage statistics for a completion.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Usage {
    /// Tokens used in the prompt.
    pub prompt_tokens: usize,
    /// Tokens generated in the completion.
    pub completion_tokens: usize,
    /// Total tokens used.
    pub total_tokens: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_completion_request_creation() {
        let request = CompletionRequest {
            model: "gpt-4o".into(),
            messages: vec![Message::user("Hello")],
            tools: vec![],
            max_tokens: Some(1000),
            temperature: Some(0.7),
            stream: false,
        };
        assert_eq!(request.model, "gpt-4o");
        assert_eq!(request.messages.len(), 1);
    }

    #[test]
    fn test_usage_default() {
        let usage = Usage::default();
        assert_eq!(usage.total_tokens, 0);
    }
}
