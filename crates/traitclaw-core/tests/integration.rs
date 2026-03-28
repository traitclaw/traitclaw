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
use traitclaw_test_utils::provider::MockProvider;

// ───────────── Tests ─────────────

#[tokio::test]
async fn test_simple_text_run() {
    let agent = Agent::builder()
        .model(MockProvider::text("Hello, world!"))
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
        .model(MockProvider::text("response"))
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
        .model(MockProvider::text("ok"))
        .build()
        .unwrap();

    let s1 = agent.session_auto();
    let s2 = agent.session_auto();

    assert_ne!(s1.id(), s2.id(), "auto sessions should have unique IDs");
}

#[tokio::test]
async fn test_agent_display_format() {
    let agent = Agent::builder()
        .model(MockProvider::text("formatted"))
        .build()
        .unwrap();

    let output = agent.run("test").await.unwrap();
    assert_eq!(format!("{output}"), "formatted");
}

#[tokio::test]
async fn test_agent_with_custom_config() {
    let agent = Agent::builder()
        .model(MockProvider::text("configured"))
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

use std::sync::atomic::{AtomicUsize, Ordering};
use traitclaw_core::types::tool_call::ToolCall;

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
    async fn complete(
        &self,
        _req: CompletionRequest,
    ) -> traitclaw_core::Result<CompletionResponse> {
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

// ───────────── v0.3.0 Async Trait Integration Tests ─────────────

#[tokio::test]
async fn test_builder_with_context_manager() {
    // Verify .context_manager() builder method works end-to-end
    use traitclaw_core::traits::context_manager::ContextManager;
    use traitclaw_core::types::agent_state::AgentState;
    use traitclaw_core::types::message::Message;

    struct PassthroughManager;

    #[async_trait]
    impl ContextManager for PassthroughManager {
        async fn prepare(
            &self,
            _messages: &mut Vec<Message>,
            _context_window: usize,
            _state: &mut AgentState,
        ) {
            // No-op: pass through all messages unchanged
        }
    }

    let agent = Agent::builder()
        .model(MockProvider::text("with context manager"))
        .context_manager(PassthroughManager)
        .build()
        .unwrap();

    let output = agent.run("test").await.unwrap();
    assert_eq!(output.text(), "with context manager");
}

#[tokio::test]
async fn test_builder_with_output_transformer() {
    // Verify .output_transformer() builder method works end-to-end
    use traitclaw_core::BudgetAwareTruncator;

    let agent = Agent::builder()
        .model(MockProvider::text("with transformer"))
        .output_transformer(BudgetAwareTruncator::new(5000, 0.8))
        .build()
        .unwrap();

    let output = agent.run("test").await.unwrap();
    assert_eq!(output.text(), "with transformer");
}

#[tokio::test]
async fn test_builder_with_tool_registry() {
    // Verify .tool_registry() builder method with DynamicRegistry
    use traitclaw_core::DynamicRegistry;

    let registry = DynamicRegistry::new();
    registry.register(Arc::new(GreetTool));

    let agent = Agent::builder()
        .model(ToolCallProvider::new())
        .tool_registry(registry)
        .tool(GreetTool) // still add to legacy tools for actual execution
        .build()
        .unwrap();

    let output = agent.run("Greet Rust").await.unwrap();
    assert_eq!(output.text(), "Tool result processed!");
}

#[tokio::test]
async fn test_builder_all_v030_traits() {
    // Verify all 3 new builder methods work together
    use traitclaw_core::traits::context_manager::ContextManager;
    use traitclaw_core::types::agent_state::AgentState;
    use traitclaw_core::types::message::Message;
    use traitclaw_core::{BudgetAwareTruncator, DynamicRegistry};

    struct CustomManager;
    #[async_trait]
    impl ContextManager for CustomManager {
        async fn prepare(
            &self,
            _messages: &mut Vec<Message>,
            _context_window: usize,
            _state: &mut AgentState,
        ) {
        }
    }

    let agent = Agent::builder()
        .model(MockProvider::text("all v0.3.0"))
        .context_manager(CustomManager)
        .output_transformer(BudgetAwareTruncator::default())
        .tool_registry(DynamicRegistry::new())
        .system("Testing all v0.3.0 traits")
        .build()
        .unwrap();

    let output = agent.run("test").await.unwrap();
    assert_eq!(output.text(), "all v0.3.0");
}
