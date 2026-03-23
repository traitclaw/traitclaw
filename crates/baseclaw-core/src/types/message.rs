//! Message types for LLM communication.

use serde::{Deserialize, Serialize};

/// Role of a message in a conversation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    /// System instructions.
    System,
    /// User input.
    User,
    /// Assistant (LLM) response.
    Assistant,
    /// Tool result.
    Tool,
}

/// A single message in a conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// The role of the message sender.
    pub role: MessageRole,
    /// The text content of the message.
    pub content: String,
    /// Optional tool call ID (for tool result messages).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

impl Message {
    /// Create a system message.
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::System,
            content: content.into(),
            tool_call_id: None,
        }
    }

    /// Create a user message.
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::User,
            content: content.into(),
            tool_call_id: None,
        }
    }

    /// Create an assistant message.
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::Assistant,
            content: content.into(),
            tool_call_id: None,
        }
    }

    /// Create a tool result message.
    pub fn tool_result(tool_call_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::Tool,
            content: content.into(),
            tool_call_id: Some(tool_call_id.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_creation() {
        let msg = Message::user("Hello");
        assert_eq!(msg.role, MessageRole::User);
        assert_eq!(msg.content, "Hello");
        assert!(msg.tool_call_id.is_none());
    }

    #[test]
    fn test_tool_result_message() {
        let msg = Message::tool_result("call_123", "result");
        assert_eq!(msg.role, MessageRole::Tool);
        assert_eq!(msg.tool_call_id, Some("call_123".into()));
    }

    #[test]
    fn test_message_serialization() {
        let msg = Message::system("You are helpful");
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"role\":\"system\""));
        assert!(json.contains("\"content\":\"You are helpful\""));
    }
}
