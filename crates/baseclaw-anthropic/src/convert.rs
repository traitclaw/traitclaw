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
