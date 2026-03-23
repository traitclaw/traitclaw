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
