//! Integration tests for the full agent lifecycle.
//!
//! Uses a mock provider to test agent runs end-to-end without
//! hitting a real LLM API.

use std::sync::Arc;

use async_trait::async_trait;
use traitclaw_core::prelude::*;
use traitclaw_core::types::completion::{
    CompletionRequest, CompletionResponse, ResponseContent, Usage,
};
use traitclaw_core::types::model_info::{ModelInfo, ModelTier};
use traitclaw_core::types::stream::{CompletionStream, StreamEvent};

// ───────────── Mock Provider ─────────────

struct MockProvider {
    model_info: ModelInfo,
    response: String,
}

impl MockProvider {
    fn new(response: impl Into<String>) -> Self {
        Self {
            model_info: ModelInfo::new("mock", ModelTier::Small, 4_096, false, false, false),
            response: response.into(),
        }
    }
}

#[async_trait]
impl Provider for MockProvider {
    async fn complete(&self, _req: CompletionRequest) -> traitclaw_core::Result<CompletionResponse> {
        Ok(CompletionResponse {
            content: ResponseContent::Text(self.response.clone()),
            usage: Usage {
                prompt_tokens: 10,
                completion_tokens: 5,
                total_tokens: 15,
            },
        })
    }

    async fn stream(&self, _req: CompletionRequest) -> traitclaw_core::Result<CompletionStream> {
        let (tx, rx) = tokio::sync::mpsc::channel(4);
        let resp = self.response.clone();
        tokio::spawn(async move {
            let _ = tx.send(Ok(StreamEvent::TextDelta(resp))).await;
            let _ = tx.send(Ok(StreamEvent::Done)).await;
        });
        Ok(Box::pin(tokio_stream::wrappers::ReceiverStream::new(rx)))
    }

    fn model_info(&self) -> &ModelInfo {
        &self.model_info
    }
}

// ───────────── Tests ─────────────

#[tokio::test]
async fn test_simple_text_run() {
    let agent = Agent::builder()
        .model(MockProvider::new("Hello, world!"))
        .system("You are helpful")
        .build()
        .unwrap();

    let output = agent.run("Hi").await.unwrap();

    assert_eq!(output.text(), "Hello, world!");
    assert!(!output.is_error());
    assert!(output.usage.tokens > 0, "should track token usage");
    assert_eq!(output.usage.iterations, 1, "should complete in 1 iteration");
    assert!(
        output.usage.duration.as_nanos() > 0,
        "should track duration"
    );
}

#[tokio::test]
async fn test_session_isolation() {
    let agent = Agent::builder()
        .model(MockProvider::new("response"))
        .system("You are helpful")
        .build()
        .unwrap();

    let session_a = agent.session("user-A");
    let session_b = agent.session("user-B");

    let out_a = session_a.say("Hello from A").await.unwrap();
    let out_b = session_b.say("Hello from B").await.unwrap();

    // Both should succeed independently
    assert_eq!(out_a.text(), "response");
    assert_eq!(out_b.text(), "response");

    // Sessions have different IDs
    assert_ne!(session_a.id(), session_b.id());
}

#[tokio::test]
async fn test_session_auto_generates_unique_ids() {
    let agent = Agent::builder()
        .model(MockProvider::new("ok"))
        .build()
        .unwrap();

    let s1 = agent.session_auto();
    let s2 = agent.session_auto();

    assert_ne!(s1.id(), s2.id(), "auto sessions should have unique IDs");
}

#[tokio::test]
async fn test_agent_display_format() {
    let agent = Agent::builder()
        .model(MockProvider::new("formatted"))
        .build()
        .unwrap();

    let output = agent.run("test").await.unwrap();
    assert_eq!(format!("{output}"), "formatted");
}

#[tokio::test]
async fn test_agent_with_custom_config() {
    let agent = Agent::builder()
        .model(MockProvider::new("configured"))
        .system("Custom system prompt")
        .max_iterations(5)
        .max_tokens(1024)
        .temperature(0.3)
        .token_budget(10_000)
        .build()
        .unwrap();

    let output = agent.run("test").await.unwrap();
    assert_eq!(output.text(), "configured");
}

#[tokio::test]
async fn test_builder_without_provider_fails() {
    let result = Agent::builder().system("test").build();
    assert!(result.is_err());
    let err_msg = match result {
        Err(e) => e.to_string(),
        Ok(_) => panic!("Expected error"),
    };
    assert!(err_msg.contains("No provider"));
}

// ───────────── Tool Execution Tests ─────────────

use traitclaw_core::types::tool_call::ToolCall;
use std::sync::atomic::{AtomicUsize, Ordering};

struct ToolCallProvider {
    model_info: ModelInfo,
    call_count: Arc<AtomicUsize>,
}

impl ToolCallProvider {
    fn new() -> Self {
        Self {
            model_info: ModelInfo::new("mock-tc", ModelTier::Small, 4_096, false, false, false),
            call_count: Arc::new(AtomicUsize::new(0)),
        }
    }
}

#[async_trait]
impl Provider for ToolCallProvider {
    async fn complete(&self, _req: CompletionRequest) -> traitclaw_core::Result<CompletionResponse> {
        let n = self.call_count.fetch_add(1, Ordering::SeqCst);
        let content = if n == 0 {
            // First call: request a tool call
            ResponseContent::ToolCalls(vec![ToolCall {
                id: "call_1".into(),
                name: "greet".into(),
                arguments: r#"{"name":"Rust"}"#.into(),
            }])
        } else {
            // Second call: return text after tool result
            ResponseContent::Text("Tool result processed!".into())
        };

        Ok(CompletionResponse {
            content,
            usage: Usage {
                prompt_tokens: 10,
                completion_tokens: 5,
                total_tokens: 15,
            },
        })
    }

    async fn stream(&self, _req: CompletionRequest) -> traitclaw_core::Result<CompletionStream> {
        unimplemented!()
    }

    fn model_info(&self) -> &ModelInfo {
        &self.model_info
    }
}

// A simple tool for testing
struct GreetTool;

#[async_trait]
impl ErasedTool for GreetTool {
    fn name(&self) -> &'static str {
        "greet"
    }
    fn description(&self) -> &'static str {
        "Greet someone"
    }
    fn schema(&self) -> traitclaw_core::traits::tool::ToolSchema {
        traitclaw_core::traits::tool::ToolSchema {
            name: "greet".into(),
            description: "Greet someone by name".into(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "name": {"type": "string"}
                }
            }),
        }
    }
    async fn execute_json(
        &self,
        args: serde_json::Value,
    ) -> std::result::Result<serde_json::Value, traitclaw_core::Error> {
        let name = args["name"].as_str().unwrap_or("world");
        Ok(serde_json::json!(format!("Hello, {name}!")))
    }
}

#[tokio::test]
async fn test_tool_call_lifecycle() {
    let agent = Agent::builder()
        .model(ToolCallProvider::new())
        .system("You are helpful")
        .tool(GreetTool)
        .build()
        .unwrap();

    let output = agent.run("Greet Rust").await.unwrap();

    assert_eq!(output.text(), "Tool result processed!");
    assert_eq!(
        output.usage.iterations, 2,
        "should take 2 iterations (tool call + response)"
    );
}
