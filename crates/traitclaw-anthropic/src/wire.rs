//! Anthropic Messages API wire format types.
//!
//! Targets the `POST /v1/messages` endpoint.
//! Reference: <https://docs.anthropic.com/en/api/messages>

use serde::{Deserialize, Serialize};

pub(crate) const ANTHROPIC_VERSION: &str = "2023-06-01";
pub(crate) const ANTHROPIC_BASE: &str = "https://api.anthropic.com/v1";

// ─── Outgoing Request ────────────────────────────────────────────────────────

/// Top-level request body for `POST /v1/messages`.
#[derive(Debug, Serialize)]
pub(crate) struct MessagesRequest {
    pub model: String,
    pub messages: Vec<AnthropicMessage>,
    /// System prompt — top-level field, not in `messages`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    pub max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<AnthropicTool>,
    pub stream: bool,
    /// Extended thinking configuration (Claude 3.7+).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<ThinkingParam>,
}

/// Extended thinking parameter — enables Claude's chain-of-thought reasoning.
#[derive(Debug, Serialize, Clone)]
pub(crate) struct ThinkingParam {
    /// Must be `"enabled"`.
    pub r#type: String,
    /// Token budget for thinking (Anthropic recommends 5000+).
    pub budget_tokens: u32,
}

/// A message in the Anthropic wire format.
#[derive(Debug, Serialize, Clone)]
pub(crate) struct AnthropicMessage {
    pub role: String, // "user" or "assistant"
    pub content: AnthropicContent,
}

/// Content of a message — either a plain string or structured content blocks.
#[derive(Debug, Serialize, Clone)]
#[serde(untagged)]
#[allow(dead_code)]
pub(crate) enum AnthropicContent {
    /// Simple text content.
    Text(String),
    /// Structured content blocks (tool use, tool results).
    Blocks(Vec<ContentBlock>),
}

/// A content block in a structured message.
#[allow(dead_code)]
#[derive(Debug, Serialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub(crate) enum ContentBlock {
    /// Plain text block.
    Text {
        /// The text content.
        text: String,
    },
    /// Tool use request from the assistant.
    ToolUse {
        /// Unique ID for this tool call.
        id: String,
        /// Name of the tool.
        name: String,
        /// Tool input parameters.
        input: serde_json::Value,
    },
    /// Tool result from the user.
    ToolResult {
        /// The tool_use_id this result corresponds to.
        tool_use_id: String,
        /// Result content (string or structured).
        content: String,
    },
}

/// A tool definition in the Anthropic format.
#[derive(Debug, Serialize, Clone)]
pub(crate) struct AnthropicTool {
    /// Tool name.
    pub name: String,
    /// Tool description.
    pub description: String,
    /// JSON Schema for the input parameters.
    pub input_schema: serde_json::Value,
}

// ─── Incoming Response ───────────────────────────────────────────────────────

/// Top-level response from `POST /v1/messages`.
#[derive(Debug, Deserialize)]
pub(crate) struct MessagesResponse {
    pub content: Vec<ResponseContentBlock>,
    /// Why the model stopped — `"end_turn"`, `"tool_use"`, etc.
    #[allow(dead_code)]
    pub stop_reason: Option<String>,
    pub usage: AnthropicUsage,
}

/// A content block in a response.
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub(crate) enum ResponseContentBlock {
    /// Text response.
    Text {
        /// The returned text.
        text: String,
    },
    /// Tool use request.
    ToolUse {
        /// Unique ID for this tool call.
        id: String,
        /// Tool name.
        name: String,
        /// Tool input (already parsed JSON).
        input: serde_json::Value,
    },
    /// Thinking block (extended thinking). Content stored separately.
    Thinking {
        /// The thinking content (chain-of-thought reasoning).
        #[allow(dead_code)]
        thinking: String,
    },
}

/// Token usage from Anthropic (different field names from OpenAI).
#[allow(clippy::struct_field_names)]
#[derive(Debug, Deserialize)]
pub(crate) struct AnthropicUsage {
    /// Tokens used in the input (prompt).
    pub input_tokens: u32,
    /// Tokens generated in the output (completion).
    pub output_tokens: u32,
}

// ─── Streaming (SSE) ─────────────────────────────────────────────────────────

/// A typed SSE event from the Anthropic streaming API.
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub(crate) enum StreamEvent {
    /// A content block started.
    ContentBlockStart {
        /// Block index.
        index: u32,
        /// The initial content block.
        content_block: StreamContentBlock,
    },
    /// A delta update to a content block.
    ContentBlockDelta {
        /// Block index.
        index: u32,
        /// The delta.
        delta: StreamDelta,
    },
    /// A content block ended.
    ContentBlockStop {
        /// Block index.
        #[allow(dead_code)]
        index: u32,
    },
    /// The overall message has ended.
    MessageStop,
    /// Message delta (stop_reason, usage on final).
    MessageDelta {
        /// The delta.
        #[allow(dead_code)]
        delta: MessageDeltaContent,
        /// Usage for this delta.
        #[allow(dead_code)]
        usage: Option<AnthropicUsage>,
    },
    /// Ping keepalive.
    #[serde(other)]
    Other,
}

/// Initial content block in a `content_block_start` event.
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub(crate) enum StreamContentBlock {
    /// Text block starting.
    Text {
        /// Initial (usually empty) text.
        #[allow(dead_code)]
        text: String,
    },
    /// Tool use block starting.
    ToolUse {
        /// Tool call ID.
        id: String,
        /// Tool name.
        name: String,
    },
}

/// A streaming delta update.
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub(crate) enum StreamDelta {
    /// Text delta.
    TextDelta {
        /// The text fragment.
        text: String,
    },
    /// Tool input JSON delta.
    InputJsonDelta {
        /// Partial JSON string.
        partial_json: String,
    },
}

/// Content of a `message_delta` event.
#[derive(Debug, Deserialize)]
pub(crate) struct MessageDeltaContent {
    /// Reason the model stopped.
    #[allow(dead_code)]
    pub stop_reason: Option<String>,
}
