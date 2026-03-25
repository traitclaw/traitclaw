//! Default strategy — preserves v0.1.0 agent loop behavior.
//!
//! This module provides `DefaultStrategy`, which encapsulates the original
//! agent runtime loop. When no custom strategy is configured, this strategy
//! is used automatically, ensuring backward compatibility.

use std::time::Instant;

use async_trait::async_trait;

use crate::agent::{AgentOutput, RunUsage};
use crate::traits::execution_strategy::PendingToolCall;
use crate::traits::hook::HookAction;
use crate::traits::strategy::{AgentRuntime, AgentStrategy};
use crate::types::agent_state::AgentState;
use crate::types::completion::{CompletionRequest, ResponseContent};
use crate::types::message::Message;
use crate::types::tool_call::ToolCall;
use crate::Result;

/// The default agent strategy preserving v0.1.0 behavior.
///
/// This strategy implements the standard agent loop:
/// 1. Load context (memory + system prompt + user message)
/// 2. Loop: LLM call → parse response → execute tools → repeat
/// 3. Exit when LLM returns text instead of tool calls
pub struct DefaultStrategy;

#[async_trait]
impl AgentStrategy for DefaultStrategy {
    #[tracing::instrument(skip_all, fields(session_id = session_id, model = %runtime.provider.model_info().name))]
    async fn execute(
        &self,
        runtime: &AgentRuntime,
        input: &str,
        session_id: &str,
    ) -> Result<AgentOutput> {
        let start = Instant::now();
        let model_info = runtime.provider.model_info();

        // Fire on_agent_start hooks
        for hook in &runtime.hooks {
            hook.on_agent_start(input).await;
        }

        let mut state = AgentState::new(model_info.tier, model_info.context_window);
        if let Some(budget) = runtime.config.token_budget {
            state.token_budget = budget;
        }

        let mut messages = match load_context(runtime, session_id, input).await {
            Ok(msgs) => msgs,
            Err(e) => {
                for hook in &runtime.hooks {
                    hook.on_error(&e).await;
                }
                return Err(e);
            }
        };
        let tool_schemas = runtime.tools.iter().map(|t| t.schema()).collect::<Vec<_>>();

        // === Agent Loop ===
        for _iteration in 0..runtime.config.max_iterations {
            state.iteration_count += 1;
            runtime.tracker.on_iteration(&mut state);

            inject_hints(runtime, &state, &mut messages);

            runtime
                .context_strategy
                .prepare(&mut messages, model_info.context_window, &mut state);

            let request = CompletionRequest {
                model: model_info.name.clone(),
                messages: messages.clone(),
                tools: tool_schemas.clone(),
                max_tokens: runtime.config.max_tokens,
                temperature: runtime.config.temperature,
                response_format: None,
                stream: false,
            };

            // Fire on_provider_start hooks
            for hook in &runtime.hooks {
                hook.on_provider_start(&request).await;
            }

            let provider_start = Instant::now();
            let response = match runtime.provider.complete(request).await {
                Ok(res) => res,
                Err(e) => {
                    for hook in &runtime.hooks {
                        hook.on_error(&e).await;
                    }
                    return Err(e);
                }
            };
            let provider_duration = provider_start.elapsed();

            // Fire on_provider_end hooks
            for hook in &runtime.hooks {
                hook.on_provider_end(&response, provider_duration).await;
            }

            state.token_usage += response.usage.total_tokens;
            state.total_context_tokens = response.usage.prompt_tokens;
            runtime.tracker.on_llm_response(&response, &mut state);

            match response.content {
                ResponseContent::Text(text) => {
                    let assistant_msg = Message::assistant(&text);
                    if let Err(e) = runtime.memory.append(session_id, assistant_msg).await {
                        tracing::warn!("Failed to save assistant response to memory: {e}");
                    }

                    let usage = RunUsage {
                        tokens: state.token_usage,
                        iterations: state.iteration_count,
                        duration: start.elapsed(),
                    };

                    #[allow(clippy::cast_possible_truncation)]
                    let duration_ms = usage.duration.as_millis() as u64;

                    tracing::info!(
                        iterations = usage.iterations,
                        tokens = usage.tokens,
                        duration_ms,
                        "Agent completed"
                    );

                    let output = AgentOutput::text_with_usage(text, usage);

                    // Fire on_agent_end hooks
                    for hook in &runtime.hooks {
                        hook.on_agent_end(&output, start.elapsed()).await;
                    }

                    return Ok(output);
                }
                ResponseContent::ToolCalls(tool_calls) => {
                    process_tool_calls(runtime, &tool_calls, &state, &mut messages).await;
                }
            }
        }

        let err = crate::Error::Runtime(format!(
            "Agent reached maximum iterations ({})",
            runtime.config.max_iterations
        ));

        // Fire on_error hooks
        for hook in &runtime.hooks {
            hook.on_error(&err).await;
        }

        Err(err)
    }

    fn stream(
        &self,
        runtime: &AgentRuntime,
        input: &str,
        session_id: &str,
    ) -> std::pin::Pin<
        Box<dyn tokio_stream::Stream<Item = Result<crate::types::stream::StreamEvent>> + Send>,
    > {
        // Fire synchronous starting hooks if any (currently all hooks in v0.2.0 are async so we emit them inside the spawned stream task)
        crate::streaming::stream_runtime(runtime.clone(), input.to_string(), session_id.to_string())
    }
}

/// Load conversation context: history + system prompt + user message.
async fn load_context(
    runtime: &AgentRuntime,
    session_id: &str,
    input: &str,
) -> Result<Vec<Message>> {
    let mut messages = runtime
        .memory
        .messages(session_id)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!("Failed to load memory (continuing fresh): {e}");
            Vec::new()
        });

    if let Some(ref system_prompt) = runtime.config.system_prompt {
        if messages.is_empty() || messages[0].role != crate::types::message::MessageRole::System {
            messages.insert(0, Message::system(system_prompt));
        }
    }

    let user_msg = Message::user(input);
    messages.push(user_msg.clone());

    if let Err(e) = runtime.memory.append(session_id, user_msg).await {
        tracing::warn!("Failed to save user message to memory: {e}");
    }

    Ok(messages)
}

/// Check hints and inject guidance messages.
fn inject_hints(runtime: &AgentRuntime, state: &AgentState, messages: &mut Vec<Message>) {
    for hint in &runtime.hints {
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

/// Process tool calls with hook interception support.
async fn process_tool_calls(
    runtime: &AgentRuntime,
    tool_calls: &[ToolCall],
    state: &AgentState,
    messages: &mut Vec<Message>,
) {
    if tool_calls.is_empty() {
        tracing::debug!("process_tool_calls: empty tool-call slice, skipping");
        return;
    }

    let summary: Vec<String> = tool_calls
        .iter()
        .map(|tc| format!("{}({})", tc.name, tc.arguments))
        .collect();
    messages.push(Message::assistant(format!(
        "[Tool calls: {}]",
        summary.join(", ")
    )));

    // Check hooks for interception before executing
    for tc in tool_calls {
        let mut blocked = false;

        for hook in &runtime.hooks {
            if let HookAction::Block(reason) =
                hook.before_tool_execute(&tc.name, &tc.arguments).await
            {
                messages.push(Message::tool_result(&tc.id, &reason));
                tracing::debug!(
                    tool = tc.name.as_str(),
                    reason = reason.as_str(),
                    "Tool blocked by hook"
                );
                blocked = true;
                break;
            }
        }

        if blocked {
            continue;
        }

        let tool_start = Instant::now();

        // Execute single tool via execution strategy
        let pending = vec![PendingToolCall::from(tc)];
        let results = runtime
            .execution_strategy
            .execute_batch(pending, &runtime.tools, &runtime.guards, state)
            .await;

        for result in results {
            let processed = runtime.output_processor.process(result.output);

            // Fire after_tool_execute hooks
            for hook in &runtime.hooks {
                hook.after_tool_execute(&tc.name, &processed, tool_start.elapsed())
                    .await;
            }

            messages.push(Message::tool_result(&result.id, &processed));
            tracing::debug!(tool_call_id = result.id.as_str(), "Tool call processed");
        }
    }
}
