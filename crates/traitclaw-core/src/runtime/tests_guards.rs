//! Guard integration tests — verify guards block tool execution.

use crate::agent::Agent;
use crate::test_utils::{DenyGuard, EchoTool, SequenceProvider};
use crate::types::completion::{CompletionResponse, ResponseContent, Usage};
use crate::types::tool_call::ToolCall;

#[tokio::test]
async fn test_guard_deny_blocks_tool_ac7() {
    // AC-7: Guard deny blocks tool execution; LLM sees "blocked by guard" error
    let responses = vec![
        CompletionResponse {
            content: ResponseContent::ToolCalls(vec![ToolCall {
                id: "call_denied".into(),
                name: "echo".into(),
                arguments: serde_json::json!({"text": "blocked"}),
            }]),
            usage: Usage {
                prompt_tokens: 10,
                completion_tokens: 5,
                total_tokens: 15,
            },
        },
        CompletionResponse {
            content: ResponseContent::Text("Guard blocked me".into()),
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
        .guard(DenyGuard)
        .build()
        .unwrap();

    let output = agent.run("Try echo").await.unwrap();
    assert_eq!(output.text(), "Guard blocked me");
}
