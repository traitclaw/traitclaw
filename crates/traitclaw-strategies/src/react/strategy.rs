//! ReAct strategy implementation.

use std::sync::Mutex;
use std::time::Instant;

use async_trait::async_trait;
use tracing;

use traitclaw_core::agent::{AgentOutput, RunUsage};
use traitclaw_core::traits::strategy::{AgentRuntime, AgentStrategy};
use traitclaw_core::types::completion::{CompletionRequest, ResponseContent};
use traitclaw_core::types::message::Message;
use traitclaw_core::{Error, Result};

use crate::common::ThoughtStep;

/// Default maximum ReAct iterations.
const DEFAULT_MAX_ITERATIONS: usize = 10;

/// ReAct (Reasoning + Acting) strategy.
///
/// Implements an autonomous Think→Act→Observe loop where the agent
/// reasons about a problem, calls tools, observes results, and repeats
/// until it produces a final answer or hits `max_iterations`.
///
/// # Example
///
/// ```no_run
/// use traitclaw_strategies::react::ReActStrategy;
///
/// let strategy = ReActStrategy::builder()
///     .max_iterations(10)
///     .build()
///     .unwrap();
/// ```
pub struct ReActStrategy {
    /// Maximum number of Think→Act→Observe cycles.
    max_iterations: usize,
    /// Optional custom system prompt for ReAct formatting.
    system_prompt: Option<String>,
    /// Collected thought steps from the last execution.
    thought_steps: Mutex<Vec<ThoughtStep>>,
}

impl std::fmt::Debug for ReActStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReActStrategy")
            .field("max_iterations", &self.max_iterations)
            .field("system_prompt", &self.system_prompt)
            .finish()
    }
}

impl ReActStrategy {
    /// Create a new builder for `ReActStrategy`.
    #[must_use]
    pub fn builder() -> ReActStrategyBuilder {
        ReActStrategyBuilder::default()
    }

    /// Returns the thought steps collected during the last execution.
    #[must_use]
    pub fn thought_steps(&self) -> Vec<ThoughtStep> {
        self.thought_steps
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }

    /// Push a thought step, recovering gracefully from mutex poisoning.
    fn push_step(&self, step: ThoughtStep) {
        self.thought_steps
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push(step);
    }

    /// Returns the maximum number of iterations.
    #[must_use]
    pub fn max_iterations(&self) -> usize {
        self.max_iterations
    }

    /// Build the ReAct system prompt that instructs the LLM to follow
    /// the Think/Act/Observe format.
    fn react_system_prompt(&self) -> String {
        if let Some(ref custom) = self.system_prompt {
            return custom.clone();
        }

        "You are a reasoning agent that follows the ReAct pattern. For each step:\n\
         \n\
         1. Think: Analyze the problem and decide what to do next.\n\
         2. Act: If you need information, call an available tool.\n\
         3. Observe: Review the tool's output.\n\
         4. Repeat until you have enough information.\n\
         5. When ready, provide your final answer.\n\
         \n\
         Use the available tools when needed. Think step by step."
            .to_string()
    }
}

/// Builder for [`ReActStrategy`].
#[derive(Debug)]
pub struct ReActStrategyBuilder {
    max_iterations: usize,
    system_prompt: Option<String>,
}

impl Default for ReActStrategyBuilder {
    fn default() -> Self {
        Self {
            max_iterations: DEFAULT_MAX_ITERATIONS,
            system_prompt: None,
        }
    }
}

impl ReActStrategyBuilder {
    /// Set the maximum number of Think→Act→Observe cycles.
    ///
    /// Default: 10
    #[must_use]
    pub fn max_iterations(mut self, n: usize) -> Self {
        self.max_iterations = n;
        self
    }

    /// Set a custom system prompt for ReAct formatting.
    ///
    /// If not set, a default ReAct instruction prompt is used.
    #[must_use]
    pub fn system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(prompt.into());
        self
    }

    /// Build the `ReActStrategy`.
    ///
    /// # Errors
    ///
    /// Returns `Error::Config` if `max_iterations` is 0.
    pub fn build(self) -> Result<ReActStrategy> {
        if self.max_iterations == 0 {
            return Err(Error::Config(
                "ReActStrategy: max_iterations must be greater than 0".into(),
            ));
        }

        Ok(ReActStrategy {
            max_iterations: self.max_iterations,
            system_prompt: self.system_prompt,
            thought_steps: Mutex::new(Vec::new()),
        })
    }
}

#[async_trait]
impl AgentStrategy for ReActStrategy {
    async fn execute(
        &self,
        runtime: &AgentRuntime,
        input: &str,
        session_id: &str,
    ) -> Result<AgentOutput> {
        let start = Instant::now();

        // Clear thought steps from any previous execution
        self.thought_steps
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clear();

        // Build messages with ReAct system prompt
        let react_prompt = self.react_system_prompt();
        let mut messages = vec![Message::system(&react_prompt), Message::user(input)];

        // Load memory if available
        let history = runtime
            .memory
            .messages(session_id)
            .await
            .unwrap_or_default();
        if !history.is_empty() {
            // Insert history after system prompt, before user message
            let user_msg = messages.pop().unwrap();
            messages.extend(history);
            messages.push(user_msg);
        }

        // Save user message to memory
        if let Err(e) = runtime
            .memory
            .append(session_id, Message::user(input))
            .await
        {
            tracing::warn!("Failed to save user message to memory: {e}");
        }

        let tool_schemas = runtime.tools.iter().map(|t| t.schema()).collect::<Vec<_>>();
        let model_info = runtime.provider.model_info();
        let mut total_tokens: usize = 0;

        // === ReAct Loop ===
        for iteration in 0..self.max_iterations {
            tracing::debug!(iteration, "ReAct iteration");

            let request = CompletionRequest {
                model: model_info.name.clone(),
                messages: messages.clone(),
                tools: tool_schemas.clone(),
                max_tokens: runtime.config.max_tokens,
                temperature: runtime.config.temperature,
                response_format: None,
                stream: false,
            };

            let response = runtime.provider.complete(request).await?;
            total_tokens += response.usage.total_tokens;

            match response.content {
                ResponseContent::Text(text) => {
                    // LLM produced text — this is a Think or Answer step
                    self.push_step(ThoughtStep::Answer {
                        content: text.clone(),
                    });

                    // Save to memory
                    if let Err(e) = runtime
                        .memory
                        .append(session_id, Message::assistant(&text))
                        .await
                    {
                        tracing::warn!("Failed to save assistant response: {e}");
                    }

                    let usage = RunUsage {
                        tokens: total_tokens,
                        iterations: iteration + 1,
                        duration: start.elapsed(),
                    };

                    return Ok(AgentOutput::text_with_usage(text, usage));
                }
                ResponseContent::ToolCalls(tool_calls) => {
                    // LLM requested tool calls — Act step
                    for tc in &tool_calls {
                        self.push_step(ThoughtStep::Act {
                            tool_name: tc.name.clone(),
                            tool_input: tc.arguments.clone(),
                        });

                        // Add assistant message with tool call reference
                        messages.push(Message::assistant(format!(
                            "[Tool call: {}({})]",
                            tc.name, tc.arguments
                        )));

                        // Execute tool via ErasedTool::execute_json
                        let tool_result = if let Some(tool) =
                            runtime.tools.iter().find(|t| t.name() == tc.name)
                        {
                            match tool.execute_json(tc.arguments.clone()).await {
                                Ok(output) => output.to_string(),
                                Err(e) => format!("Tool error: {e}"),
                            }
                        } else {
                            format!("Tool '{}' not found", tc.name)
                        };

                        // Observe step
                        self.push_step(ThoughtStep::Observe {
                            tool_output: tool_result.clone(),
                        });

                        messages.push(Message::tool_result(&tc.id, &tool_result));
                    }
                }
            }
        }

        // Hit max iterations without producing answer
        let err = Error::Runtime(format!(
            "ReActStrategy: reached maximum iterations ({}) without producing an answer",
            self.max_iterations
        ));
        Err(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_defaults() {
        let strategy = ReActStrategy::builder().build().unwrap();
        assert_eq!(strategy.max_iterations(), DEFAULT_MAX_ITERATIONS);
        assert!(strategy.thought_steps().is_empty());
    }

    #[test]
    fn test_builder_custom_max_iterations() {
        let strategy = ReActStrategy::builder().max_iterations(5).build().unwrap();
        assert_eq!(strategy.max_iterations(), 5);
    }

    #[test]
    fn test_builder_custom_system_prompt() {
        let strategy = ReActStrategy::builder()
            .system_prompt("Custom prompt")
            .build()
            .unwrap();
        assert_eq!(strategy.react_system_prompt(), "Custom prompt");
    }

    #[test]
    fn test_builder_rejects_zero_iterations() {
        let result = ReActStrategy::builder().max_iterations(0).build();
        assert!(result.is_err());
        let err_str = result.unwrap_err().to_string();
        assert!(err_str.contains("max_iterations"));
    }

    #[test]
    fn test_object_safety() {
        let strategy = ReActStrategy::builder().build().unwrap();
        let _boxed: Box<dyn AgentStrategy> = Box::new(strategy);
    }

    #[test]
    fn test_default_system_prompt() {
        let strategy = ReActStrategy::builder().build().unwrap();
        let prompt = strategy.react_system_prompt();
        assert!(prompt.contains("ReAct"));
        assert!(prompt.contains("Think"));
        assert!(prompt.contains("Act"));
        assert!(prompt.contains("Observe"));
    }

    // ── F1: Async Execution Tests ───────────────────────────────────────

    #[tokio::test]
    async fn test_execute_text_response() {
        use traitclaw_test_utils::provider::MockProvider;
        use traitclaw_test_utils::runtime::make_runtime;

        let provider = MockProvider::text("The answer is 42.");
        let runtime = make_runtime(provider, vec![]);

        let strategy = ReActStrategy::builder().build().unwrap();
        let result = strategy
            .execute(&runtime, "What is the answer?", "s1")
            .await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.text(), "The answer is 42.");

        // Should have one Answer thought step
        let steps = strategy.thought_steps();
        assert_eq!(steps.len(), 1);
        assert!(
            matches!(&steps[0], ThoughtStep::Answer { content } if content == "The answer is 42.")
        );
    }

    #[tokio::test]
    async fn test_execute_tool_call_then_answer() {
        use traitclaw_core::types::tool_call::ToolCall;
        use traitclaw_test_utils::provider::MockProvider;
        use traitclaw_test_utils::runtime::make_runtime;

        let tool_call = ToolCall {
            id: "call_1".to_string(),
            name: "nonexistent_tool".to_string(),
            arguments: serde_json::json!({}),
        };
        let provider = MockProvider::tool_then_text(vec![tool_call], "Final answer.");
        let runtime = make_runtime(provider, vec![]);

        let strategy = ReActStrategy::builder().build().unwrap();
        let result = strategy.execute(&runtime, "use a tool", "s1").await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.text(), "Final answer.");

        // Should have Act + Observe + Answer steps
        let steps = strategy.thought_steps();
        assert!(steps.len() >= 3);
        assert!(
            matches!(&steps[0], ThoughtStep::Act { tool_name, .. } if tool_name == "nonexistent_tool")
        );
        assert!(
            matches!(&steps[1], ThoughtStep::Observe { tool_output } if tool_output.contains("not found"))
        );
    }

    // ── F2: Error Path Tests ────────────────────────────────────────────

    #[tokio::test]
    async fn test_execute_max_iterations_exhausted() {
        use traitclaw_core::types::tool_call::ToolCall;
        use traitclaw_test_utils::provider::MockProvider;
        use traitclaw_test_utils::runtime::make_runtime;

        // Provider always returns tool calls, never text
        let tc = ToolCall {
            id: "call_loop".to_string(),
            name: "loop_tool".to_string(),
            arguments: serde_json::json!({}),
        };
        let provider = MockProvider::always_tool_calls(vec![tc]);
        let runtime = make_runtime(provider, vec![]);

        let strategy = ReActStrategy::builder().max_iterations(2).build().unwrap();

        let result = strategy.execute(&runtime, "infinite loop", "s1").await;
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("maximum iterations"),
            "Expected max iterations error: {err}"
        );
    }
}
