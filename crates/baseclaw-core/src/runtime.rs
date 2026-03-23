//! Agent runtime loop — the execution engine.
//!
//! Orchestrates the LLM call → tool execution → steering cycle.

use std::sync::Arc;

use crate::agent::{Agent, AgentOutput};
use crate::traits::guard::GuardResult;
use crate::traits::tool::ErasedTool;
use crate::types::action::Action;
use crate::types::agent_state::AgentState;
use crate::types::completion::{CompletionRequest, ResponseContent};
use crate::types::message::Message;
use crate::types::tool_call::ToolCall;
use crate::Result;

/// Run the full agent loop.
pub(crate) async fn run_agent(agent: &Agent, input: &str) -> Result<AgentOutput> {
    let model_info = agent.provider.model_info();
    let session_id = "default"; // TODO: session management

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
                process_tool_calls(agent, &tool_calls, &mut state, &mut messages).await;
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
        if messages.is_empty()
            || messages[0].role != crate::types::message::MessageRole::System
        {
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

/// Process tool calls: guard check, execute, collect results.
async fn process_tool_calls(
    agent: &Agent,
    tool_calls: &[ToolCall],
    state: &mut AgentState,
    messages: &mut Vec<Message>,
) {
    // Add assistant message summarizing tool calls
    let summary: Vec<String> = tool_calls
        .iter()
        .map(|tc| format!("{}({})", tc.name, tc.arguments))
        .collect();
    messages.push(Message::assistant(format!(
        "[Tool calls: {}]",
        summary.join(", ")
    )));

    for tc in tool_calls {
        let result = execute_guarded_tool(agent, tc, state).await;
        messages.push(Message::tool_result(&tc.id, &result));
        tracing::debug!(tool = tc.name.as_str(), "Tool call processed");
    }
}

/// Run guard checks and execute a single tool call.
async fn execute_guarded_tool(agent: &Agent, tc: &ToolCall, state: &mut AgentState) -> String {
    let action = Action::ToolCall {
        name: tc.name.clone(),
        arguments: tc.arguments.clone(),
    };

    // Check guards
    for guard in &agent.guards {
        match guard.check(&action) {
            GuardResult::Allow => {}
            GuardResult::Deny { reason, severity } => {
                tracing::warn!(
                    guard = guard.name(),
                    tool = tc.name.as_str(),
                    ?severity,
                    "Guard denied action: {reason}"
                );
                return format!("Error: Action blocked by guard: {reason}");
            }
            GuardResult::Sanitize { warning, .. } => {
                tracing::info!(guard = guard.name(), "Guard sanitized: {warning}");
            }
        }
    }

    // Execute the tool
    execute_tool(&agent.tools, &tc.name, &tc.arguments, state).await
}

/// Find and execute a tool by name.
async fn execute_tool(
    tools: &[Arc<dyn ErasedTool>],
    name: &str,
    arguments: &serde_json::Value,
    _state: &mut AgentState,
) -> String {
    if let Some(tool) = tools.iter().find(|t| t.name() == name) {
        match tool.execute_json(arguments.clone()).await {
            Ok(output) => serde_json::to_string(&output)
                .unwrap_or_else(|e| format!("Error serializing output: {e}")),
            Err(e) => format!("Error executing tool: {e}"),
        }
    } else {
        let available: Vec<_> = tools.iter().map(|t| t.name().to_string()).collect();
        format!(
            "Error: Tool '{name}' not found. Available: {}",
            available.join(", ")
        )
    }
}
