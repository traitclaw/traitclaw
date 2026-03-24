//! Conversion helpers between `baseclaw-core` types and OpenAI wire format.

use baseclaw_core::types::completion::{CompletionResponse, ResponseContent, Usage};
use baseclaw_core::types::message::{Message, MessageRole};
use baseclaw_core::types::tool_call::ToolCall;
use baseclaw_core::{Error, Result, ToolSchema};

use crate::wire::{ChatRequest, ChatResponse, WireFunctionDef, WireMessage, WireTool};
use baseclaw_core::types::completion::CompletionRequest;

/// Convert a `CompletionRequest` into an OpenAI-compatible `ChatRequest`.
pub(crate) fn to_wire(req: CompletionRequest) -> ChatRequest {
    let messages = req.messages.into_iter().map(msg_to_wire).collect();
    let tools = req.tools.into_iter().map(schema_to_wire).collect();

    ChatRequest {
        model: req.model,
        messages,
        tools,
        max_tokens: req.max_tokens,
        temperature: req.temperature,
        response_format: req
            .response_format
            .map(|rf| serde_json::to_value(rf).unwrap_or_default()),
        stream: req.stream,
    }
}

/// Convert a core `Message` to a wire `WireMessage`.
fn msg_to_wire(msg: Message) -> WireMessage {
    let role = match msg.role {
        MessageRole::System => "system",
        MessageRole::User => "user",
        MessageRole::Assistant => "assistant",
        MessageRole::Tool => "tool",
        // Forward-compatible: unknown roles pass through as-is via serde string.
        // This arm satisfies #[non_exhaustive] on MessageRole.
        _ => "unknown",
    }
    .to_string();

    WireMessage {
        role,
        content: Some(msg.content),
        tool_calls: None,
        tool_call_id: msg.tool_call_id,
        name: None,
    }
}

/// Convert a `ToolSchema` to a wire `WireTool`.
fn schema_to_wire(schema: ToolSchema) -> WireTool {
    WireTool {
        kind: "function".to_string(),
        function: WireFunctionDef {
            name: schema.name,
            description: schema.description,
            parameters: schema.parameters,
        },
    }
}

/// Convert an OpenAI wire `ChatResponse` to a `CompletionResponse`.
pub(crate) fn from_wire(wire: ChatResponse) -> Result<CompletionResponse> {
    let usage = wire.usage.map_or_else(
        || Usage {
            prompt_tokens: 0,
            completion_tokens: 0,
            total_tokens: 0,
        },
        |tc| Usage {
            prompt_tokens: tc.prompt_tokens as usize,
            completion_tokens: tc.completion_tokens as usize,
            total_tokens: tc.total_tokens as usize,
        },
    );

    let choice = wire
        .choices
        .into_iter()
        .next()
        .ok_or_else(|| Error::provider("Empty choices in response"))?;

    let content = if let Some(tool_calls) = choice.message.tool_calls {
        let calls = tool_calls.into_iter().map(wire_tool_call_to_core).collect();
        ResponseContent::ToolCalls(calls)
    } else {
        let text = choice.message.content.unwrap_or_default();
        ResponseContent::Text(text)
    };

    Ok(CompletionResponse { content, usage })
}

/// Convert a wire `WireToolCall` to a core `ToolCall`.
pub(crate) fn wire_tool_call_to_core(tc: crate::wire::WireToolCall) -> ToolCall {
    // Model returns arguments as a JSON string — parse into Value
    let arguments = serde_json::from_str(&tc.function.arguments)
        .unwrap_or_else(|_| serde_json::Value::String(tc.function.arguments.clone()));

    ToolCall {
        id: tc.id,
        name: tc.function.name,
        arguments,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use baseclaw_core::types::completion::ResponseFormat;

    fn make_request() -> CompletionRequest {
        CompletionRequest {
            model: "gpt-4o".into(),
            messages: vec![Message::system("You are helpful"), Message::user("Hello")],
            tools: vec![],
            max_tokens: Some(1000),
            temperature: Some(0.7),
            response_format: None,
            stream: false,
        }
    }

    #[test]
    fn test_to_wire_basic_fields() {
        let wire = to_wire(make_request());
        assert_eq!(wire.model, "gpt-4o");
        assert_eq!(wire.messages.len(), 2);
        assert_eq!(wire.messages[0].role, "system");
        assert_eq!(wire.messages[1].role, "user");
        assert_eq!(wire.max_tokens, Some(1000));
        assert!(!wire.stream);
    }

    #[test]
    fn test_to_wire_tool_schemas() {
        let mut req = make_request();
        req.tools.push(ToolSchema {
            name: "search".into(),
            description: "Search the web".into(),
            parameters: serde_json::json!({ "type": "object" }),
        });
        let wire = to_wire(req);
        assert_eq!(wire.tools.len(), 1);
        assert_eq!(wire.tools[0].kind, "function");
        assert_eq!(wire.tools[0].function.name, "search");
    }

    #[test]
    fn test_to_wire_response_format_forwarded() {
        let mut req = make_request();
        req.response_format = Some(ResponseFormat::JsonObject);
        let wire = to_wire(req);
        assert!(wire.response_format.is_some());
    }

    #[test]
    fn test_to_wire_no_response_format() {
        let wire = to_wire(make_request());
        assert!(wire.response_format.is_none());
    }

    #[test]
    fn test_from_wire_text_response() {
        use crate::wire::{ChatResponse, Choice, TokenCounts, WireAssistantMessage};
        let wire = ChatResponse {
            choices: vec![Choice {
                message: WireAssistantMessage {
                    content: Some("Hello world".into()),
                    tool_calls: None,
                },
                finish_reason: Some("stop".into()),
            }],
            usage: Some(TokenCounts {
                prompt_tokens: 10,
                completion_tokens: 5,
                total_tokens: 15,
            }),
        };
        let resp = from_wire(wire).unwrap();
        match resp.content {
            ResponseContent::Text(t) => assert_eq!(t, "Hello world"),
            ResponseContent::ToolCalls(_) => panic!("expected text"),
        }
        assert_eq!(resp.usage.total_tokens, 15);
    }

    #[test]
    fn test_from_wire_tool_calls() {
        use crate::wire::{
            ChatResponse, Choice, WireAssistantMessage, WireFunctionCall, WireToolCall,
        };
        let wire = ChatResponse {
            choices: vec![Choice {
                message: WireAssistantMessage {
                    content: None,
                    tool_calls: Some(vec![WireToolCall {
                        id: "call_1".into(),
                        kind: "function".into(),
                        function: WireFunctionCall {
                            name: "search".into(),
                            arguments: r#"{"q":"rust"}"#.into(),
                        },
                    }]),
                },
                finish_reason: Some("tool_calls".into()),
            }],
            usage: None,
        };
        let resp = from_wire(wire).unwrap();
        match resp.content {
            ResponseContent::ToolCalls(calls) => {
                assert_eq!(calls.len(), 1);
                assert_eq!(calls[0].name, "search");
                assert_eq!(calls[0].arguments["q"], "rust");
            }
            ResponseContent::Text(_) => panic!("expected tool calls"),
        }
    }

    #[test]
    fn test_from_wire_empty_choices_errors() {
        use crate::wire::ChatResponse;
        let wire = ChatResponse {
            choices: vec![],
            usage: None,
        };
        assert!(from_wire(wire).is_err());
    }

    #[test]
    fn test_msg_to_wire_tool_role() {
        let msg = Message {
            role: MessageRole::Tool,
            content: "result".into(),
            tool_call_id: Some("call_1".into()),
        };
        let wire = msg_to_wire(msg);
        assert_eq!(wire.role, "tool");
        assert_eq!(wire.tool_call_id, Some("call_1".into()));
    }
}
