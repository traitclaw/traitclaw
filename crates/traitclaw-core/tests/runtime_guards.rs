//! Guard integration tests — verify guards block tool execution.
//!
//! Migrated from `src/runtime/tests_guards.rs` to use shared test utilities.

use traitclaw_core::prelude::*;
use traitclaw_core::types::completion::{CompletionResponse, ResponseContent, Usage};
use traitclaw_core::types::tool_call::ToolCall;
use traitclaw_test_utils::provider::MockProvider;
use traitclaw_test_utils::tools::{DenyGuard, EchoTool};

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
        .model(MockProvider::sequence(responses))
        .system("You are a test bot")
        .tool(EchoTool)
        .guard(DenyGuard)
        .build()
        .unwrap();

    let output = agent.run("Try echo").await.unwrap();
    assert_eq!(output.text(), "Guard blocked me");
}
