//! Hook integration tests — v0.2.0 lifecycle hooks and tool interception.

use std::sync::Arc;

use async_trait::async_trait;

use crate::agent::Agent;
use crate::test_utils::{DangerousTool, EchoTool, SequenceProvider};
use crate::types::completion::{CompletionRequest, CompletionResponse, ResponseContent, Usage};
use crate::types::tool_call::ToolCall;

// === Security Hook ===

/// Security hook that blocks tools with "dangerous" in the name.
struct SecurityHook;

#[async_trait]
impl crate::traits::hook::AgentHook for SecurityHook {
    async fn before_tool_execute(
        &self,
        name: &str,
        _args: &serde_json::Value,
    ) -> crate::traits::hook::HookAction {
        if name.contains("dangerous") {
            crate::traits::hook::HookAction::Block(
                "Blocked by security policy: tool name contains 'dangerous'".into(),
            )
        } else {
            crate::traits::hook::HookAction::Continue
        }
    }
}

// === Tests ===

#[tokio::test]
async fn test_hook_blocks_tool_execution() {
    // Story 2.4: SecurityHook blocks tools with "dangerous" in the name
    let responses = vec![
        CompletionResponse {
            content: ResponseContent::ToolCalls(vec![ToolCall {
                id: "call_danger".into(),
                name: "dangerous_operation".into(),
                arguments: serde_json::json!({"payload": "evil"}),
            }]),
            usage: Usage {
                prompt_tokens: 10,
                completion_tokens: 5,
                total_tokens: 15,
            },
        },
        CompletionResponse {
            content: ResponseContent::Text(
                "I understand, the dangerous operation was blocked.".into(),
            ),
            usage: Usage {
                prompt_tokens: 30,
                completion_tokens: 10,
                total_tokens: 40,
            },
        },
    ];

    let agent = Agent::builder()
        .model(SequenceProvider::with_responses(responses))
        .system("You are a test bot")
        .tool(DangerousTool)
        .hook(SecurityHook)
        .build()
        .unwrap();

    let output = agent.run("Run the dangerous operation").await.unwrap();
    assert_eq!(
        output.text(),
        "I understand, the dangerous operation was blocked."
    );
}

#[tokio::test]
async fn test_hook_allows_safe_tool_execution() {
    // SecurityHook allows tools without "dangerous" in the name
    let responses = vec![
        CompletionResponse {
            content: ResponseContent::ToolCalls(vec![ToolCall {
                id: "call_safe".into(),
                name: "echo".into(),
                arguments: serde_json::json!({"text": "safe"}),
            }]),
            usage: Usage {
                prompt_tokens: 10,
                completion_tokens: 5,
                total_tokens: 15,
            },
        },
        CompletionResponse {
            content: ResponseContent::Text("Echo worked fine".into()),
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
        .hook(SecurityHook)
        .build()
        .unwrap();

    let output = agent.run("Use echo").await.unwrap();
    assert_eq!(output.text(), "Echo worked fine");
}

// === Recording Hook for Lifecycle Verification ===

struct RecordingHook {
    events: std::sync::Mutex<Vec<String>>,
}

impl RecordingHook {
    fn new() -> Self {
        Self {
            events: std::sync::Mutex::new(Vec::new()),
        }
    }

    fn events(&self) -> Vec<String> {
        self.events.lock().unwrap().clone()
    }
}

#[async_trait]
impl crate::traits::hook::AgentHook for RecordingHook {
    async fn on_agent_start(&self, _input: &str) {
        self.events.lock().unwrap().push("agent_start".into());
    }

    async fn on_agent_end(
        &self,
        _output: &crate::agent::AgentOutput,
        _duration: std::time::Duration,
    ) {
        self.events.lock().unwrap().push("agent_end".into());
    }

    async fn on_provider_start(&self, _request: &CompletionRequest) {
        self.events.lock().unwrap().push("provider_start".into());
    }

    async fn on_provider_end(
        &self,
        _response: &CompletionResponse,
        _duration: std::time::Duration,
    ) {
        self.events.lock().unwrap().push("provider_end".into());
    }
}

#[tokio::test]
async fn test_hook_lifecycle_order() {
    // Verify hooks fire in correct order: agent_start → provider_start → provider_end → agent_end
    let hook = Arc::new(RecordingHook::new());

    let agent = Agent::builder()
        .model(SequenceProvider::text("Hello"))
        .system("test")
        .hook(Arc::clone(&hook))
        .build()
        .unwrap();

    agent.run("Hi").await.unwrap();

    let events = hook.events();
    assert_eq!(
        events,
        vec!["agent_start", "provider_start", "provider_end", "agent_end"]
    );
}
