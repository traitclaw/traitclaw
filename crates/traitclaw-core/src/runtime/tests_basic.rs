//! Basic agent runtime tests — text responses, tool calls, memory, error handling.
//!
//! These tests verify core agent behaviors (AC-3 through AC-6).

use crate::agent::Agent;
use crate::test_utils::{EchoTool, SequenceProvider};
use crate::types::completion::{CompletionResponse, ResponseContent, Usage};
use crate::types::tool_call::ToolCall;

#[tokio::test]
async fn test_simple_text_response_ac3() {
    // AC-3: text response → AgentOutput::Text
    let agent = Agent::builder()
        .model(SequenceProvider::text("Hello, world!"))
        .system("You are a test bot")
        .build()
        .unwrap();

    let output = agent.run("Hi").await.unwrap();
    assert_eq!(output.text(), "Hello, world!");
}

#[tokio::test]
async fn test_tool_call_then_text_ac4_ac5() {
    // AC-4: tool_calls → execute → feed results → loop
    // AC-5: loop terminates when LLM returns text
    let responses = vec![
        CompletionResponse {
            content: ResponseContent::ToolCalls(vec![ToolCall {
                id: "call_1".into(),
                name: "echo".into(),
                arguments: serde_json::json!({"text": "ping"}),
            }]),
            usage: Usage {
                prompt_tokens: 10,
                completion_tokens: 5,
                total_tokens: 15,
            },
        },
        CompletionResponse {
            content: ResponseContent::Text("Tool said: ping".into()),
            usage: Usage {
                prompt_tokens: 20,
                completion_tokens: 5,
                total_tokens: 25,
            },
        },
    ];

    let agent = Agent::builder()
        .model(SequenceProvider::with_responses(responses))
        .system("You are a test bot")
        .tool(EchoTool)
        .build()
        .unwrap();

    let output = agent.run("Use echo").await.unwrap();
    assert_eq!(output.text(), "Tool said: ping");
}

#[tokio::test]
async fn test_max_iterations_error_ac5() {
    // AC-5: loop terminates on max iterations → error
    let tool_response = CompletionResponse {
        content: ResponseContent::ToolCalls(vec![ToolCall {
            id: "call_loop".into(),
            name: "echo".into(),
            arguments: serde_json::json!({"text": "loop"}),
        }]),
        usage: Usage {
            prompt_tokens: 10,
            completion_tokens: 5,
            total_tokens: 15,
        },
    };

    let agent = Agent::builder()
        .model(SequenceProvider::with_responses(vec![tool_response]))
        .system("You loop forever")
        .tool(EchoTool)
        .max_iterations(3)
        .build()
        .unwrap();

    let result = agent.run("Loop").await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("maximum iterations"), "got: {err}");
}

#[tokio::test]
async fn test_memory_saved_ac6() {
    // AC-6: conversation is saved to memory after completion
    let agent = Agent::builder()
        .model(SequenceProvider::text("Saved!"))
        .system("You are a test bot")
        .build()
        .unwrap();

    agent.run("Remember this").await.unwrap();

    let msgs = agent.memory.messages("default").await.unwrap();
    assert!(
        msgs.len() >= 2,
        "expected at least user + assistant messages"
    );
    assert!(msgs.iter().any(|m| m.content == "Remember this"));
    assert!(msgs.iter().any(|m| m.content == "Saved!"));
}

#[tokio::test]
async fn test_unknown_tool_returns_error_message() {
    // AC-5: If tool name not found → returns error message to LLM (no crash)
    let responses = vec![
        CompletionResponse {
            content: ResponseContent::ToolCalls(vec![ToolCall {
                id: "call_bad".into(),
                name: "nonexistent_tool".into(),
                arguments: serde_json::json!({}),
            }]),
            usage: Usage {
                prompt_tokens: 10,
                completion_tokens: 5,
                total_tokens: 15,
            },
        },
        CompletionResponse {
            content: ResponseContent::Text("I see the error".into()),
            usage: Usage {
                prompt_tokens: 20,
                completion_tokens: 5,
                total_tokens: 25,
            },
        },
    ];

    let agent = Agent::builder()
        .model(SequenceProvider::with_responses(responses))
        .system("You are a test bot")
        .tool(EchoTool)
        .build()
        .unwrap();

    let output = agent.run("Use nonexistent").await.unwrap();
    assert_eq!(output.text(), "I see the error");
}

#[tokio::test]
async fn test_bad_tool_args_returns_error_message() {
    // AC-6: If deserialization fails → returns descriptive error to LLM (no crash)
    let responses = vec![
        CompletionResponse {
            content: ResponseContent::ToolCalls(vec![ToolCall {
                id: "call_bad_args".into(),
                name: "echo".into(),
                arguments: serde_json::json!({"wrong_field": 123}),
            }]),
            usage: Usage {
                prompt_tokens: 10,
                completion_tokens: 5,
                total_tokens: 15,
            },
        },
        CompletionResponse {
            content: ResponseContent::Text("Bad args handled".into()),
            usage: Usage {
                prompt_tokens: 20,
                completion_tokens: 5,
                total_tokens: 25,
            },
        },
    ];

    let agent = Agent::builder()
        .model(SequenceProvider::with_responses(responses))
        .system("You are a test bot")
        .tool(EchoTool)
        .build()
        .unwrap();

    let output = agent.run("Bad args").await.unwrap();
    assert_eq!(output.text(), "Bad args handled");
}
