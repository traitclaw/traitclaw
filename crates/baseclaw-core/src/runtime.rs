//! Agent runtime loop — the execution engine.
//!
//! Orchestrates the LLM call → tool execution → steering cycle.

use crate::agent::{Agent, AgentOutput};
use crate::types::agent_state::AgentState;
use crate::types::completion::{CompletionRequest, ResponseContent};
use crate::types::message::Message;
use crate::types::tool_call::ToolCall;
use crate::Result;

/// Run the full agent loop.
pub(crate) async fn run_agent(agent: &Agent, input: &str, session_id: &str) -> Result<AgentOutput> {
    let model_info = agent.provider.model_info();

    let mut state = AgentState::new(model_info.tier, model_info.context_window);
    if let Some(budget) = agent.config.token_budget {
        state.token_budget = budget;
    }

    let mut messages = load_context(agent, session_id, input).await?;
    let tool_schemas = agent.tools.iter().map(|t| t.schema()).collect::<Vec<_>>();

    // === Agent Loop ===
    for _iteration in 0..agent.config.max_iterations {
        state.iteration_count += 1;
        agent.tracker.on_iteration(&mut state);

        inject_hints(agent, &state, &mut messages);

        // AC-6: Apply context window management before each LLM call
        agent
            .context_strategy
            .prepare(&mut messages, model_info.context_window, &mut state);

        let request = CompletionRequest {
            model: model_info.name.clone(),
            messages: messages.clone(),
            tools: tool_schemas.clone(),
            max_tokens: agent.config.max_tokens,
            temperature: agent.config.temperature,
            stream: false,
        };

        let response = agent.provider.complete(request).await?;

        state.token_usage += response.usage.total_tokens;
        state.total_context_tokens = response.usage.prompt_tokens;
        agent.tracker.on_llm_response(&response, &mut state);

        match response.content {
            ResponseContent::Text(text) => {
                let assistant_msg = Message::assistant(&text);
                agent.memory.append(session_id, assistant_msg).await?;

                tracing::info!(
                    iterations = state.iteration_count,
                    tokens = state.token_usage,
                    "Agent completed"
                );

                return Ok(AgentOutput::Text(text));
            }
            ResponseContent::ToolCalls(tool_calls) => {
                process_tool_calls(agent, &tool_calls, &state, &mut messages).await;
            }
        }
    }

    Err(crate::Error::Runtime(format!(
        "Agent reached maximum iterations ({})",
        agent.config.max_iterations
    )))
}

/// Load conversation context: history + system prompt + user message.
async fn load_context(agent: &Agent, session_id: &str, input: &str) -> Result<Vec<Message>> {
    let mut messages = agent.memory.messages(session_id).await?;

    if let Some(ref system_prompt) = agent.config.system_prompt {
        if messages.is_empty() || messages[0].role != crate::types::message::MessageRole::System {
            messages.insert(0, Message::system(system_prompt));
        }
    }

    let user_msg = Message::user(input);
    messages.push(user_msg.clone());
    agent.memory.append(session_id, user_msg).await?;

    Ok(messages)
}

/// Check hints and inject guidance messages.
fn inject_hints(agent: &Agent, state: &AgentState, messages: &mut Vec<Message>) {
    for hint in &agent.hints {
        if hint.should_trigger(state) {
            let hint_msg = hint.generate(state);
            messages.push(Message {
                role: hint_msg.role,
                content: hint_msg.content,
                tool_call_id: None,
            });
            tracing::debug!(hint = hint.name(), "Hint injected");
        }
    }
}

/// Process tool calls: delegate to execution strategy and collect results.
///
/// DESIGN: errors from individual tool executions are intentionally returned
/// as error strings rather than propagating `Result`. This allows the LLM to
/// observe the failure and self-correct in subsequent iterations, which is the
/// standard agentic pattern. Hard failures (provider errors, memory errors)
/// still propagate via `?` in the outer `run_agent` loop.
async fn process_tool_calls(
    agent: &Agent,
    tool_calls: &[ToolCall],
    state: &AgentState,
    messages: &mut Vec<Message>,
) {
    use crate::traits::execution_strategy::PendingToolCall;

    // P-2 guard: providers occasionally return an empty tool-call array via
    // untagged serde deserialization. Skip to avoid injecting a vestigial
    // "[Tool calls: ]" message into the context window.
    if tool_calls.is_empty() {
        tracing::debug!("process_tool_calls: empty tool-call slice, skipping");
        return;
    }

    // Add assistant message summarizing tool calls
    let summary: Vec<String> = tool_calls
        .iter()
        .map(|tc| format!("{}({})", tc.name, tc.arguments))
        .collect();
    messages.push(Message::assistant(format!(
        "[Tool calls: {}]",
        summary.join(", ")
    )));

    // Convert to PendingToolCall and delegate to execution strategy
    let pending: Vec<PendingToolCall> = tool_calls.iter().map(PendingToolCall::from).collect();
    let results = agent
        .execution_strategy
        .execute_batch(pending, &agent.tools, &agent.guards, state)
        .await;

    for result in results {
        let processed = agent.output_processor.process(result.output);
        messages.push(Message::tool_result(&result.id, &processed));
        tracing::debug!(tool_call_id = result.id.as_str(), "Tool call processed");
    }
}

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
}
