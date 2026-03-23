//! OpenAI-compatible wire format types for `POST /v1/chat/completions`.
//!
//! Compatible with: OpenAI, Ollama, Groq, Mistral, Together AI, vLLM, Azure OpenAI.

use serde::{Deserialize, Serialize};

// ─── Outgoing Request ────────────────────────────────────────────────────────

/// Top-level request body for `/v1/chat/completions`.
#[derive(Debug, Serialize)]
pub(crate) struct ChatRequest {
    pub model: String,
    pub messages: Vec<WireMessage>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<WireTool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    pub stream: bool,
}

/// A message in the wire format.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct WireMessage {
    pub role: String,
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<WireToolCall>>,
    /// Present when `role = "tool"` — identifies which `tool_call_id` this responds to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// A tool definition in the request.
#[derive(Debug, Serialize, Clone)]
pub(crate) struct WireTool {
    #[serde(rename = "type")]
    pub kind: String, // always "function"
    pub function: WireFunctionDef,
}

/// Function definition inside a tool.
#[derive(Debug, Serialize, Clone)]
pub(crate) struct WireFunctionDef {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

// ─── Incoming Response ───────────────────────────────────────────────────────

/// Top-level response from `/v1/chat/completions`.
#[derive(Debug, Deserialize)]
pub(crate) struct ChatResponse {
    pub choices: Vec<Choice>,
    pub usage: Option<TokenCounts>,
}

/// One completion choice.
#[derive(Debug, Deserialize)]
pub(crate) struct Choice {
    pub message: WireAssistantMessage,
    /// Why the model stopped — `"stop"`, `"tool_calls"`, `"length"`, etc.
    #[allow(dead_code)]
    pub finish_reason: Option<String>,
}

/// Assistant message in a response.
#[derive(Debug, Deserialize)]
pub(crate) struct WireAssistantMessage {
    pub content: Option<String>,
    pub tool_calls: Option<Vec<WireToolCall>>,
}

/// Tool call returned by the model.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct WireToolCall {
    pub id: String,
    #[serde(rename = "type")]
    #[allow(dead_code)]
    pub kind: String, // "function"
    pub function: WireFunctionCall,
}

/// Function call detail inside a tool call.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct WireFunctionCall {
    pub name: String,
    pub arguments: String, // JSON string from the model
}

/// Token usage counts from the response.
///
/// Field names mirror the OpenAI wire format exactly.
#[allow(clippy::struct_field_names)]
#[derive(Debug, Deserialize)]
pub(crate) struct TokenCounts {
    /// Tokens used in the prompt.
    pub prompt_tokens: u32,
    /// Tokens generated in the completion.
    pub completion_tokens: u32,
    /// Total tokens used.
    pub total_tokens: u32,
}

// ─── Streaming (SSE) ─────────────────────────────────────────────────────────

/// One SSE chunk from a streaming response.
#[derive(Debug, Deserialize)]
pub(crate) struct StreamChunk {
    pub choices: Vec<StreamChoice>,
    /// Usage data (some providers send it on the final chunk).
    #[allow(dead_code)]
    pub usage: Option<TokenCounts>,
}

/// One choice inside a streaming chunk.
#[derive(Debug, Deserialize)]
pub(crate) struct StreamChoice {
    pub delta: StreamDelta,
    pub finish_reason: Option<String>,
}

/// Delta content in a streaming chunk.
#[derive(Debug, Deserialize)]
pub(crate) struct StreamDelta {
    pub content: Option<String>,
    pub tool_calls: Option<Vec<StreamToolCallDelta>>,
}

/// Partial tool call in a delta.
#[derive(Debug, Deserialize)]
pub(crate) struct StreamToolCallDelta {
    pub index: u32,
    pub id: Option<String>,
    #[serde(rename = "type")]
    #[allow(dead_code)]
    pub kind: Option<String>,
    pub function: Option<StreamFunctionDelta>,
}

/// Partial function call inside a streaming delta.
#[derive(Debug, Deserialize)]
pub(crate) struct StreamFunctionDelta {
    pub name: Option<String>,
    pub arguments: Option<String>,
}
