//! Chain-of-Thought strategy implementation.

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

/// Default maximum reasoning steps.
const DEFAULT_MAX_STEPS: usize = 5;

/// Chain-of-Thought (CoT) reasoning strategy.
///
/// Instructs the LLM to reason step-by-step before producing a final answer.
/// Each reasoning step is captured as a [`ThoughtStep::Think`] event, making
/// the reasoning process transparent and auditable.
///
/// # Example
///
/// ```no_run
/// use traitclaw_strategies::cot::ChainOfThoughtStrategy;
///
/// let strategy = ChainOfThoughtStrategy::builder()
///     .max_steps(5)
///     .build()
///     .unwrap();
/// ```
pub struct ChainOfThoughtStrategy {
    /// Maximum number of reasoning steps.
    max_steps: usize,
    /// Collected thought steps from the last execution.
    thought_steps: Mutex<Vec<ThoughtStep>>,
}

impl std::fmt::Debug for ChainOfThoughtStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChainOfThoughtStrategy")
            .field("max_steps", &self.max_steps)
            .finish()
    }
}

impl ChainOfThoughtStrategy {
    /// Create a new builder for `ChainOfThoughtStrategy`.
    #[must_use]
    pub fn builder() -> ChainOfThoughtStrategyBuilder {
        ChainOfThoughtStrategyBuilder::default()
    }

    /// Returns the thought steps collected during the last execution.
    #[must_use]
    pub fn thought_steps(&self) -> Vec<ThoughtStep> {
        self.thought_steps
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }

    /// Returns the maximum number of steps.
    #[must_use]
    pub fn max_steps(&self) -> usize {
        self.max_steps
    }
}

/// Builder for [`ChainOfThoughtStrategy`].
#[derive(Debug)]
pub struct ChainOfThoughtStrategyBuilder {
    max_steps: usize,
}

impl Default for ChainOfThoughtStrategyBuilder {
    fn default() -> Self {
        Self {
            max_steps: DEFAULT_MAX_STEPS,
        }
    }
}

impl ChainOfThoughtStrategyBuilder {
    /// Set the maximum number of reasoning steps.
    ///
    /// Default: 5
    #[must_use]
    pub fn max_steps(mut self, n: usize) -> Self {
        self.max_steps = n;
        self
    }

    /// Build the `ChainOfThoughtStrategy`.
    ///
    /// # Errors
    ///
    /// Returns `Error::Config` if `max_steps` is 0.
    pub fn build(self) -> Result<ChainOfThoughtStrategy> {
        if self.max_steps == 0 {
            return Err(Error::Config(
                "ChainOfThoughtStrategy: max_steps must be greater than 0".into(),
            ));
        }

        Ok(ChainOfThoughtStrategy {
            max_steps: self.max_steps,
            thought_steps: Mutex::new(Vec::new()),
        })
    }
}

#[async_trait]
impl AgentStrategy for ChainOfThoughtStrategy {
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

        // Build CoT system prompt
        let cot_prompt = format!(
            "You are a reasoning assistant. Think step by step to solve the problem.\n\
             Break your reasoning into at most {} numbered steps.\n\
             Format each step as:\n\
             Step N: [your reasoning]\n\
             \n\
             After your reasoning steps, provide your final answer on a new line starting with:\n\
             Final Answer: [your answer]",
            self.max_steps
        );

        let messages = vec![Message::system(&cot_prompt), Message::user(input)];

        // Save user message to memory
        if let Err(e) = runtime
            .memory
            .append(session_id, Message::user(input))
            .await
        {
            tracing::warn!("Failed to save user message to memory: {e}");
        }

        let model_info = runtime.provider.model_info();

        let request = CompletionRequest {
            model: model_info.name.clone(),
            messages,
            tools: vec![], // CoT doesn't use tools
            max_tokens: runtime.config.max_tokens,
            temperature: runtime.config.temperature,
            response_format: None,
            stream: false,
        };

        let response = runtime.provider.complete(request).await?;

        match response.content {
            ResponseContent::Text(text) => {
                // Parse reasoning steps from the response
                self.parse_thought_steps(&text);

                // Save to memory
                if let Err(e) = runtime
                    .memory
                    .append(session_id, Message::assistant(&text))
                    .await
                {
                    tracing::warn!("Failed to save assistant response: {e}");
                }

                let usage = RunUsage {
                    tokens: response.usage.total_tokens,
                    iterations: 1,
                    duration: start.elapsed(),
                };

                Ok(AgentOutput::text_with_usage(text, usage))
            }
            ResponseContent::ToolCalls(_) => {
                // CoT should not produce tool calls
                Err(Error::Runtime(
                    "ChainOfThoughtStrategy: unexpected tool calls in CoT response".into(),
                ))
            }
        }
    }
}

impl ChainOfThoughtStrategy {
    /// Parse the LLM output into ThoughtStep events.
    fn parse_thought_steps(&self, text: &str) {
        let mut steps = self.thought_steps.lock().unwrap_or_else(|e| e.into_inner());

        for line in text.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("Step ") && trimmed.contains(':') {
                // Extract content after "Step N:"
                if let Some((_, content)) = trimmed.split_once(':') {
                    steps.push(ThoughtStep::Think {
                        content: content.trim().to_string(),
                    });
                }
            } else if let Some(answer) = trimmed.strip_prefix("Final Answer:") {
                steps.push(ThoughtStep::Answer {
                    content: answer.trim().to_string(),
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_defaults() {
        let strategy = ChainOfThoughtStrategy::builder().build().unwrap();
        assert_eq!(strategy.max_steps(), DEFAULT_MAX_STEPS);
        assert!(strategy.thought_steps().is_empty());
    }

    #[test]
    fn test_builder_custom_max_steps() {
        let strategy = ChainOfThoughtStrategy::builder()
            .max_steps(3)
            .build()
            .unwrap();
        assert_eq!(strategy.max_steps(), 3);
    }

    #[test]
    fn test_builder_rejects_zero_steps() {
        let result = ChainOfThoughtStrategy::builder().max_steps(0).build();
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("max_steps"));
    }

    #[test]
    fn test_object_safety() {
        let strategy = ChainOfThoughtStrategy::builder().build().unwrap();
        let _boxed: Box<dyn AgentStrategy> = Box::new(strategy);
    }

    #[test]
    fn test_parse_thought_steps() {
        let strategy = ChainOfThoughtStrategy::builder().build().unwrap();
        let text = "Step 1: Analyze the input.\nStep 2: Consider the options.\nFinal Answer: 42";
        strategy.parse_thought_steps(text);
        let steps = strategy.thought_steps();
        assert_eq!(steps.len(), 3);
        assert!(
            matches!(&steps[0], ThoughtStep::Think { content } if content == "Analyze the input.")
        );
        assert!(
            matches!(&steps[1], ThoughtStep::Think { content } if content == "Consider the options.")
        );
        assert!(matches!(&steps[2], ThoughtStep::Answer { content } if content == "42"));
    }

    // ── F1: Async Execution Tests ───────────────────────────────────────

    #[tokio::test]
    async fn test_execute_text_response() {
        use traitclaw_test_utils::provider::MockProvider;
        use traitclaw_test_utils::runtime::make_runtime;

        let provider =
            MockProvider::text("Step 1: Think about it.\nStep 2: Evaluate.\nFinal Answer: 42");
        let runtime = make_runtime(provider, vec![]);

        let strategy = ChainOfThoughtStrategy::builder().build().unwrap();
        let result = strategy.execute(&runtime, "question", "s1").await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.text().contains("42"));

        // Should have parsed Think + Think + Answer steps
        let steps = strategy.thought_steps();
        assert_eq!(steps.len(), 3);
    }

    // ── F2: Error Path Tests ────────────────────────────────────────────

    #[tokio::test]
    async fn test_execute_unexpected_tool_calls() {
        use traitclaw_core::types::tool_call::ToolCall;
        use traitclaw_test_utils::provider::MockProvider;
        use traitclaw_test_utils::runtime::make_runtime;

        let tc = ToolCall {
            id: "call_1".to_string(),
            name: "search".to_string(),
            arguments: serde_json::json!({}),
        };
        let provider = MockProvider::always_tool_calls(vec![tc]);
        let runtime = make_runtime(provider, vec![]);

        let strategy = ChainOfThoughtStrategy::builder().build().unwrap();
        let result = strategy.execute(&runtime, "question", "s1").await;

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("unexpected tool calls"),
            "Expected 'unexpected tool calls' error: {err}"
        );
    }

    // ── F5: parse_thought_steps Edge Cases ──────────────────────────────

    #[test]
    fn test_parse_empty_input() {
        let strategy = ChainOfThoughtStrategy::builder().build().unwrap();
        strategy.parse_thought_steps("");
        assert!(strategy.thought_steps().is_empty());
    }

    #[test]
    fn test_parse_no_steps() {
        let strategy = ChainOfThoughtStrategy::builder().build().unwrap();
        strategy.parse_thought_steps("Just a plain text response with no structured steps.");
        assert!(strategy.thought_steps().is_empty());
    }

    #[test]
    fn test_parse_only_final_answer() {
        let strategy = ChainOfThoughtStrategy::builder().build().unwrap();
        strategy.parse_thought_steps("Final Answer: The result is 7.");
        let steps = strategy.thought_steps();
        assert_eq!(steps.len(), 1);
        assert!(
            matches!(&steps[0], ThoughtStep::Answer { content } if content == "The result is 7.")
        );
    }

    #[test]
    fn test_parse_malformed_step() {
        let strategy = ChainOfThoughtStrategy::builder().build().unwrap();
        // "Step " without a number shouldn't match because there's no ":"
        strategy.parse_thought_steps("Step without colon\nStep 1: Valid step");
        let steps = strategy.thought_steps();
        assert_eq!(steps.len(), 1);
        assert!(matches!(&steps[0], ThoughtStep::Think { content } if content == "Valid step"));
    }

    #[test]
    fn test_parse_multiline_mixed() {
        let strategy = ChainOfThoughtStrategy::builder().build().unwrap();
        let text =
            "Some preamble\nStep 1: First\nIrrelevant line\nStep 2: Second\nFinal Answer: Done";
        strategy.parse_thought_steps(text);
        let steps = strategy.thought_steps();
        assert_eq!(steps.len(), 3);
    }
}
