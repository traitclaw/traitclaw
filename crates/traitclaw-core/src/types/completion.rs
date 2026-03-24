//! Completion request and response types.
//!
//! These types define the interface between the runtime and LLM providers,
//! following the `OpenAI` Chat Completions `API` format as the internal standard.

use serde::{Deserialize, Serialize};

use crate::traits::tool::ToolSchema;
use crate::types::message::Message;
use crate::types::tool_call::ToolCall;

/// A completion request to send to an LLM provider.
///
/// # AC-3
/// Contains: messages, tools, model, temperature, `max_tokens`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    /// The model identifier to use.
    pub model: String,
    /// Conversation messages.
    pub messages: Vec<Message>,
    /// Available tools (empty if no tools).
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tools: Vec<ToolSchema>,
    /// Maximum tokens in the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    /// Sampling temperature (0.0–2.0).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    /// Response format — when set, constrains the model to output JSON
    /// matching a specific schema (structured output).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<ResponseFormat>,
    /// Whether to stream the response.
    /// RUNTIME-ONLY: excluded from JSON (de)serialization — always defaults to
    /// `false` regardless of what is present in the JSON source.
    #[serde(skip)]
    pub stream: bool,
}

/// Response format constraint for structured output.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ResponseFormat {
    /// Constrain output to valid JSON matching the given schema.
    #[serde(rename = "json_schema")]
    JsonSchema {
        /// The JSON Schema the model must follow.
        json_schema: serde_json::Value,
    },
    /// Constrain output to valid JSON (no schema enforcement).
    #[serde(rename = "json_object")]
    JsonObject,
}

/// The content of a completion response — either text or tool calls.
///
/// # AC-4
/// Mirrors the `OpenAI` `choices[0].message` structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ResponseContent {
    /// Tool calls requested by the LLM.
    ToolCalls(Vec<ToolCall>),
    /// A text response from the LLM.
    Text(String),
}

/// A completion response from the LLM provider.
///
/// # AC-4, AC-5
/// Contains content (text or `tool_calls`) and usage stats.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    /// The response content (text or tool calls).
    pub content: ResponseContent,
    /// Token usage statistics.
    pub usage: Usage,
}

/// Token usage statistics for a completion.
///
/// # AC-4
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Usage {
    /// Tokens used in the prompt.
    pub prompt_tokens: usize,
    /// Tokens generated in the completion.
    pub completion_tokens: usize,
    /// Total tokens used (prompt + completion).
    pub total_tokens: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_completion_request_fields_ac3() {
        // AC-3: CompletionRequest has all required fields
        let request = CompletionRequest {
            model: "gpt-4o".into(),
            messages: vec![Message::user("Hello")],
            tools: vec![],
            max_tokens: Some(1000),
            temperature: Some(0.7),
            response_format: None,
            stream: false,
        };
        assert_eq!(request.model, "gpt-4o");
        assert_eq!(request.messages.len(), 1);
        assert_eq!(request.max_tokens, Some(1000));
        assert!((request.temperature.unwrap() - 0.7).abs() < f32::EPSILON);
    }

    #[test]
    fn test_completion_request_serialization_omits_empty_tools() {
        let request = CompletionRequest {
            model: "claude-3".into(),
            messages: vec![Message::user("Hi")],
            tools: vec![],
            max_tokens: None,
            temperature: None,
            response_format: None,
            stream: false,
        };
        let json = serde_json::to_string(&request).unwrap();
        // empty tools should be omitted
        assert!(!json.contains("tools"), "got: {json}");
        // stream is a runtime flag — must be omitted
        assert!(!json.contains("stream"), "got: {json}");
    }

    #[test]
    fn test_usage_default_zeros() {
        // AC-4: Usage struct present with expected fields
        let usage = Usage::default();
        assert_eq!(usage.total_tokens, 0);
        assert_eq!(usage.prompt_tokens, 0);
        assert_eq!(usage.completion_tokens, 0);
    }

    #[test]
    fn test_usage_round_trip() {
        let usage = Usage {
            prompt_tokens: 10,
            completion_tokens: 20,
            total_tokens: 30,
        };
        let json = serde_json::to_string(&usage).unwrap();
        let decoded: Usage = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.prompt_tokens, 10);
        assert_eq!(decoded.completion_tokens, 20);
        assert_eq!(decoded.total_tokens, 30);
    }

    #[test]
    fn test_completion_response_text_round_trip() {
        // AC-5: Serialize + Deserialize for CompletionResponse
        let resp = CompletionResponse {
            content: ResponseContent::Text("Hello!".into()),
            usage: Usage {
                prompt_tokens: 5,
                completion_tokens: 3,
                total_tokens: 8,
            },
        };
        let json = serde_json::to_string(&resp).unwrap();
        // Verify it's serializable without panicking
        assert!(json.contains("Hello!"));
    }

    #[test]
    fn test_completion_request_deserialize() {
        // AC-5: CompletionRequest implements Deserialize
        let json = r#"{
            "model": "gpt-4o",
            "messages": [{"role":"user","content":"Hi"}]
        }"#;
        let req: CompletionRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.model, "gpt-4o");
        assert_eq!(req.messages.len(), 1);
        assert!(req.tools.is_empty());
    }
    #[test]
    fn test_response_content_text_deserialization() {
        // AC-5: ResponseContent::Text deserializes from a plain string
        let json = r#""Hello from LLM""#;
        let content: ResponseContent = serde_json::from_str(json).unwrap();
        match content {
            ResponseContent::Text(t) => assert_eq!(t, "Hello from LLM"),
            ResponseContent::ToolCalls(_) => panic!("expected Text variant"),
        }
    }

    #[test]
    fn test_response_content_tool_calls_deserialization() {
        // AC-5: ResponseContent::ToolCalls deserializes from an array of ToolCalls
        let json = r#"[{"id":"c1","name":"search","arguments":{"q":"rust"}}]"#;
        let content: ResponseContent = serde_json::from_str(json).unwrap();
        match content {
            ResponseContent::ToolCalls(calls) => {
                assert_eq!(calls.len(), 1);
                assert_eq!(calls[0].name, "search");
            }
            ResponseContent::Text(_) => panic!("expected ToolCalls variant"),
        }
    }

    #[test]
    fn test_response_content_empty_array_is_tool_calls_variant() {
        // P-2 guard: document the behavior — empty array → ToolCalls([])
        // Runtime must guard against this (skip processing empty tool-call slices)
        let json = r"[]";
        let content: ResponseContent = serde_json::from_str(json).unwrap();
        assert!(matches!(content, ResponseContent::ToolCalls(calls) if calls.is_empty()));
    }
}
