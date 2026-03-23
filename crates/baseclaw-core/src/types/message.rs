//! Message types for LLM communication.

use serde::{Deserialize, Serialize};

/// Role of a message in a conversation.
///
/// This enum is marked `#[non_exhaustive]` — new roles may be added in future
/// releases (e.g., `Developer`, `Tool` variants from updated provider specs).
/// Always include a wildcard arm in exhaustive matches.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
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
    fn test_user_message_json_matches_openai_format() {
        // AC-1: exact OpenAI wire format
        let msg = Message::user("Hello");
        let json = serde_json::to_string(&msg).unwrap();
        assert_eq!(json, r#"{"role":"user","content":"Hello"}"#);
    }

    #[test]
    fn test_system_message_json_format() {
        let msg = Message::system("You are helpful");
        let json = serde_json::to_string(&msg).unwrap();
        assert_eq!(json, r#"{"role":"system","content":"You are helpful"}"#);
    }

    #[test]
    fn test_assistant_message_json_format() {
        let msg = Message::assistant("Hi there!");
        let json = serde_json::to_string(&msg).unwrap();
        assert_eq!(json, r#"{"role":"assistant","content":"Hi there!"}"#);
    }

    #[test]
    fn test_tool_result_message_json_includes_tool_call_id() {
        let msg = Message::tool_result("call_123", "42");
        let json = serde_json::to_string(&msg).unwrap();
        assert_eq!(
            json,
            r#"{"role":"tool","content":"42","tool_call_id":"call_123"}"#
        );
    }

    #[test]
    fn test_message_role_lowercase_serde() {
        // AC-2: all 4 MessageRole variants serialize to lowercase
        let cases = [
            (MessageRole::System, "\"system\""),
            (MessageRole::User, "\"user\""),
            (MessageRole::Assistant, "\"assistant\""),
            (MessageRole::Tool, "\"tool\""),
        ];
        for (role, expected) in &cases {
            assert_eq!(&serde_json::to_string(role).unwrap(), expected);
        }
    }

    #[test]
    fn test_message_round_trip_deserialize() {
        let original = Message::user("round trip");
        let json = serde_json::to_string(&original).unwrap();
        let decoded: Message = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.role, original.role);
        assert_eq!(decoded.content, original.content);
    }

    #[test]
    fn test_tool_call_id_absent_when_none() {
        let msg = Message::user("hello");
        let json = serde_json::to_string(&msg).unwrap();
        // skip_serializing_if ensures field is omitted
        assert!(!json.contains("tool_call_id"), "got: {json}");
    }

    #[test]
    fn test_message_creation() {
        let msg = Message::user("Hello");
        assert_eq!(msg.role, MessageRole::User);
        assert_eq!(msg.content, "Hello");
        assert!(msg.tool_call_id.is_none());
    }

    #[test]
    fn test_tool_result_message_fields() {
        let msg = Message::tool_result("call_123", "result");
        assert_eq!(msg.role, MessageRole::Tool);
        assert_eq!(msg.tool_call_id, Some("call_123".into()));
    }
}
