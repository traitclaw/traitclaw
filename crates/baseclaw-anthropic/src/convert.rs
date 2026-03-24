//! Conversion between `baseclaw-core` types and the Anthropic Messages API format.

use baseclaw_core::types::completion::{
    CompletionRequest, CompletionResponse, ResponseContent, Usage,
};
use baseclaw_core::types::message::MessageRole;
use baseclaw_core::types::tool_call::ToolCall;
use baseclaw_core::ToolSchema;

use crate::wire::{
    AnthropicContent, AnthropicMessage, AnthropicTool, ContentBlock, MessagesRequest,
    MessagesResponse, ResponseContentBlock,
};

/// Convert a `CompletionRequest` into an Anthropic `MessagesRequest`.
pub(crate) fn to_wire(req: CompletionRequest) -> MessagesRequest {
    let mut system: Option<String> = None;
    let mut messages: Vec<AnthropicMessage> = Vec::new();

    for msg in req.messages {
        match msg.role {
            MessageRole::System => {
                // Anthropic: system is a top-level field, append if multiple
                if let Some(ref mut s) = system {
                    s.push('\n');
                    s.push_str(&msg.content);
                } else {
                    system = Some(msg.content);
                }
            }
            MessageRole::User => {
                // Check if it's a tool result
                if let Some(tool_call_id) = msg.tool_call_id {
                    messages.push(AnthropicMessage {
                        role: "user".to_string(),
                        content: AnthropicContent::Blocks(vec![ContentBlock::ToolResult {
                            tool_use_id: tool_call_id,
                            content: msg.content,
                        }]),
                    });
                } else {
                    messages.push(AnthropicMessage {
                        role: "user".to_string(),
                        content: AnthropicContent::Text(msg.content),
                    });
                }
            }
            MessageRole::Assistant => {
                messages.push(AnthropicMessage {
                    role: "assistant".to_string(),
                    content: AnthropicContent::Text(msg.content),
                });
            }
            MessageRole::Tool => {
                // Represented as user message with tool_result block
                messages.push(AnthropicMessage {
                    role: "user".to_string(),
                    content: AnthropicContent::Blocks(vec![ContentBlock::ToolResult {
                        tool_use_id: msg.tool_call_id.unwrap_or_default(),
                        content: msg.content,
                    }]),
                });
            }
            // Forward-compatible: skip unknown roles (#[non_exhaustive] on MessageRole)
            _ => {}
        }
    }

    let tools: Vec<AnthropicTool> = req.tools.into_iter().map(schema_to_wire).collect();

    // Anthropic requires max_tokens — default to 4096 if not specified
    let max_tokens = req.max_tokens.unwrap_or(4096);

    MessagesRequest {
        model: req.model,
        messages,
        system,
        max_tokens,
        temperature: req.temperature,
        tools,
        stream: req.stream,
    }
}

/// Convert a `ToolSchema` to an `AnthropicTool`.
fn schema_to_wire(schema: ToolSchema) -> AnthropicTool {
    // Anthropic uses `input_schema` with a plain JSON Schema object
    // Strip the outer `$schema` and `title` that schemars adds at root level
    let input_schema = simplify_schema(schema.parameters);

    AnthropicTool {
        name: schema.name,
        description: schema.description,
        input_schema,
    }
}

/// Strip schemars metadata that Anthropic doesn't accept.
fn simplify_schema(mut schema: serde_json::Value) -> serde_json::Value {
    if let Some(obj) = schema.as_object_mut() {
        obj.remove("$schema");
        obj.remove("title");
        obj.remove("definitions");
        // Ensure type is present
        if !obj.contains_key("type") {
            obj.insert(
                "type".to_string(),
                serde_json::Value::String("object".to_string()),
            );
        }
    }
    schema
}

/// Convert an Anthropic `MessagesResponse` to a `CompletionResponse`.
pub(crate) fn from_wire(wire: MessagesResponse) -> CompletionResponse {
    let usage = Usage {
        prompt_tokens: wire.usage.input_tokens as usize,
        completion_tokens: wire.usage.output_tokens as usize,
        total_tokens: (wire.usage.input_tokens + wire.usage.output_tokens) as usize,
    };

    let mut text_parts: Vec<String> = Vec::new();
    let mut tool_calls: Vec<ToolCall> = Vec::new();

    for block in wire.content {
        match block {
            ResponseContentBlock::Text { text } => {
                text_parts.push(text);
            }
            ResponseContentBlock::ToolUse { id, name, input } => {
                tool_calls.push(ToolCall {
                    id,
                    name,
                    arguments: input,
                });
            }
        }
    }

    let content = if tool_calls.is_empty() {
        ResponseContent::Text(text_parts.join(""))
    } else {
        ResponseContent::ToolCalls(tool_calls)
    };

    CompletionResponse { content, usage }
}

#[cfg(test)]
mod tests {
    use super::*;
    use baseclaw_core::types::message::Message;

    fn make_request() -> CompletionRequest {
        baseclaw_core::types::completion::CompletionRequest {
            model: "claude-3-5-sonnet-20241022".into(),
            messages: vec![Message::system("You are helpful"), Message::user("Hello")],
            tools: vec![],
            max_tokens: Some(2000),
            temperature: Some(0.5),
            response_format: None,
            stream: false,
        }
    }

    #[test]
    fn test_system_extracted_to_top_level() {
        let wire = to_wire(make_request());
        assert_eq!(wire.system, Some("You are helpful".into()));
        // System message should NOT appear in messages array
        assert_eq!(wire.messages.len(), 1);
        assert_eq!(wire.messages[0].role, "user");
    }

    #[test]
    fn test_multiple_system_messages_concatenated() {
        let mut req = make_request();
        req.messages.insert(1, Message::system("Be concise"));
        let wire = to_wire(req);
        assert_eq!(wire.system, Some("You are helpful\nBe concise".into()));
    }

    #[test]
    fn test_max_tokens_defaults_to_4096() {
        let mut req = make_request();
        req.max_tokens = None;
        let wire = to_wire(req);
        assert_eq!(wire.max_tokens, 4096);
    }

    #[test]
    fn test_tool_role_becomes_user_tool_result() {
        let mut req = make_request();
        req.messages.push(Message {
            role: MessageRole::Tool,
            content: "result data".into(),
            tool_call_id: Some("call_1".into()),
        });
        let wire = to_wire(req);
        // Tool message becomes user with tool_result block
        let last = wire.messages.last().unwrap();
        assert_eq!(last.role, "user");
    }

    #[test]
    fn test_tool_schemas_mapped() {
        let mut req = make_request();
        req.tools.push(ToolSchema {
            name: "search".into(),
            description: "Search the web".into(),
            parameters: serde_json::json!({ "type": "object", "$schema": "http://...", "title": "Search" }),
        });
        let wire = to_wire(req);
        assert_eq!(wire.tools.len(), 1);
        assert_eq!(wire.tools[0].name, "search");
        // $schema and title should be stripped
        assert!(!wire.tools[0]
            .input_schema
            .as_object()
            .unwrap()
            .contains_key("$schema"));
        assert!(!wire.tools[0]
            .input_schema
            .as_object()
            .unwrap()
            .contains_key("title"));
    }

    #[test]
    fn test_from_wire_text_response() {
        let wire = MessagesResponse {
            content: vec![ResponseContentBlock::Text {
                text: "Hello!".into(),
            }],
            stop_reason: Some("end_turn".into()),
            usage: crate::wire::AnthropicUsage {
                input_tokens: 10,
                output_tokens: 5,
            },
        };
        let resp = from_wire(wire);
        match resp.content {
            ResponseContent::Text(t) => assert_eq!(t, "Hello!"),
            ResponseContent::ToolCalls(_) => panic!("expected text"),
        }
        assert_eq!(resp.usage.prompt_tokens, 10);
        assert_eq!(resp.usage.completion_tokens, 5);
        assert_eq!(resp.usage.total_tokens, 15);
    }

    #[test]
    fn test_from_wire_tool_use() {
        let wire = MessagesResponse {
            content: vec![ResponseContentBlock::ToolUse {
                id: "toolu_1".into(),
                name: "search".into(),
                input: serde_json::json!({"q": "rust"}),
            }],
            stop_reason: Some("tool_use".into()),
            usage: crate::wire::AnthropicUsage {
                input_tokens: 20,
                output_tokens: 10,
            },
        };
        let resp = from_wire(wire);
        match resp.content {
            ResponseContent::ToolCalls(calls) => {
                assert_eq!(calls.len(), 1);
                assert_eq!(calls[0].name, "search");
                assert_eq!(calls[0].id, "toolu_1");
            }
            ResponseContent::Text(_) => panic!("expected tool calls"),
        }
    }
}
