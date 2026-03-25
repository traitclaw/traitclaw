//! Agent runtime loop — the execution engine.
//!
//! The runtime loop logic has been extracted into [`DefaultStrategy`](crate::default_strategy::DefaultStrategy)
//! as part of the v0.2.0 AgentStrategy refactoring. This module now only
//! contains legacy tests that verify agent behavior end-to-end.


#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use async_trait::async_trait;
    use serde::{Deserialize, Serialize};

    use crate::agent::Agent;
    use crate::traits::guard::{Guard, GuardResult, GuardSeverity};
    use crate::traits::provider::Provider;
    use crate::types::action::Action;
    use crate::types::completion::{CompletionRequest, CompletionResponse, ResponseContent, Usage};
    use crate::types::model_info::{ModelInfo, ModelTier};
    use crate::types::stream::CompletionStream;
    use crate::types::tool_call::ToolCall;

    // ---- Mock Provider ----
    // Returns responses in sequence: first call returns responses[0], second returns responses[1], etc.
    struct SequenceProvider {
        info: ModelInfo,
        responses: Vec<CompletionResponse>,
        call_idx: AtomicUsize,
    }

    impl SequenceProvider {
        fn text(text: &str) -> Self {
            Self {
                info: ModelInfo::new("test-model", ModelTier::Small, 4096, false, false, false),
                responses: vec![CompletionResponse {
                    content: ResponseContent::Text(text.into()),
                    usage: Usage {
                        prompt_tokens: 10,
                        completion_tokens: 5,
                        total_tokens: 15,
                    },
                }],
                call_idx: AtomicUsize::new(0),
            }
        }

        fn with_responses(responses: Vec<CompletionResponse>) -> Self {
            Self {
                info: ModelInfo::new("test-model", ModelTier::Small, 4096, true, false, false),
                responses,
                call_idx: AtomicUsize::new(0),
            }
        }
    }

    #[async_trait]
    impl Provider for SequenceProvider {
        async fn complete(&self, _req: CompletionRequest) -> crate::Result<CompletionResponse> {
            let idx = self.call_idx.fetch_add(1, Ordering::SeqCst);
            Ok(self.responses[idx.min(self.responses.len() - 1)].clone())
        }
        async fn stream(&self, _req: CompletionRequest) -> crate::Result<CompletionStream> {
            unimplemented!()
        }
        fn model_info(&self) -> &ModelInfo {
            &self.info
        }
    }

    // ---- Mock Tool (echo) ----
    #[derive(Deserialize, schemars::JsonSchema)]
    struct EchoInput {
        text: String,
    }
    #[derive(Serialize)]
    struct EchoOutput {
        echo: String,
    }

    struct EchoTool;

    #[async_trait]
    impl crate::traits::tool::Tool for EchoTool {
        type Input = EchoInput;
        type Output = EchoOutput;
        fn name(&self) -> &'static str {
            "echo"
        }
        fn description(&self) -> &'static str {
            "Echoes input"
        }
        async fn execute(&self, input: Self::Input) -> crate::Result<Self::Output> {
            Ok(EchoOutput {
                echo: input.text.clone(),
            })
        }
    }

    // ---- Mock Guard (deny all) ----
    struct DenyGuard;

    impl Guard for DenyGuard {
        fn name(&self) -> &'static str {
            "deny-all"
        }
        fn check(&self, _action: &Action) -> GuardResult {
            GuardResult::Deny {
                reason: "blocked by test guard".into(),
                severity: GuardSeverity::High,
            }
        }
    }

    // === Tests ===

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
            // First call: LLM asks to use the echo tool
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
            // Second call: LLM returns text after seeing tool result
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
        // Provider always returns tool calls, never text
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

        // Check that the user message and assistant response were saved
        let msgs = agent.memory.messages("default").await.unwrap();
        assert!(
            msgs.len() >= 2,
            "expected at least user + assistant messages"
        );
        // User message should be there
        assert!(msgs.iter().any(|m| m.content == "Remember this"));
        // Assistant message should be there
        assert!(msgs.iter().any(|m| m.content == "Saved!"));
    }

    #[tokio::test]
    async fn test_guard_deny_blocks_tool_ac7() {
        // AC-7: Guard deny blocks tool execution; LLM sees "blocked by guard" error
        let responses = vec![
            // First call: LLM asks to use the echo tool → guard denies
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
            // Second call: LLM returns text after seeing guard denial
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

    // === v0.2.0 Hook Interception Tests (Story 2.4) ===

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

    // ---- Mock Dangerous Tool ----
    #[derive(Deserialize, schemars::JsonSchema)]
    struct DangerousInput {
        #[allow(dead_code)]
        payload: String,
    }
    #[derive(Serialize)]
    struct DangerousOutput {
        result: String,
    }

    struct DangerousTool;

    #[async_trait]
    impl crate::traits::tool::Tool for DangerousTool {
        type Input = DangerousInput;
        type Output = DangerousOutput;
        fn name(&self) -> &'static str {
            "dangerous_operation"
        }
        fn description(&self) -> &'static str {
            "A dangerous tool"
        }
        async fn execute(&self, _input: Self::Input) -> crate::Result<Self::Output> {
            Ok(DangerousOutput {
                result: "SHOULD NOT RUN".into(),
            })
        }
    }

    #[tokio::test]
    async fn test_hook_blocks_tool_execution() {
        // Story 2.4: SecurityHook blocks tools with "dangerous" in the name
        let responses = vec![
            // First call: LLM requests dangerous_operation
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
            // Second call: LLM sees block reason and responds
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

    // Recording hook to verify hook lifecycle call order
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
        let hook = std::sync::Arc::new(RecordingHook::new());

        let agent = Agent::builder()
            .model(SequenceProvider::text("Hello"))
            .system("test")
            .hook(std::sync::Arc::clone(&hook))
            .build()
            .unwrap();

        agent.run("Hi").await.unwrap();

        let events = hook.events();
        assert_eq!(
            events,
            vec!["agent_start", "provider_start", "provider_end", "agent_end"]
        );
    }
}
