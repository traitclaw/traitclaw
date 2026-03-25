//! Built-in [`OutputTransformer`] implementations for common use cases.
//!
//! These transformers can be used directly or composed for more complex processing.

use async_trait::async_trait;

use crate::traits::output_transformer::OutputTransformer;
use crate::types::agent_state::AgentState;

/// Truncates output to a maximum character count, respecting context utilization.
///
/// When context utilization exceeds the `aggressive_threshold`, the limit is
/// halved to preserve context budget.
///
/// # Example
///
/// ```rust
/// use traitclaw_core::transformers::BudgetAwareTruncator;
///
/// let t = BudgetAwareTruncator::new(1000, 0.8);
/// ```
pub struct BudgetAwareTruncator {
    max_chars: usize,
    aggressive_threshold: f32,
}

impl BudgetAwareTruncator {
    /// Create a new truncator.
    ///
    /// - `max_chars`: Maximum output length in characters.
    /// - `aggressive_threshold`: Context utilization (0.0–1.0) above which
    ///   truncation becomes more aggressive (halved limit).
    #[must_use]
    pub fn new(max_chars: usize, aggressive_threshold: f32) -> Self {
        Self {
            max_chars,
            aggressive_threshold: aggressive_threshold.clamp(0.0, 1.0),
        }
    }
}

impl Default for BudgetAwareTruncator {
    fn default() -> Self {
        Self::new(10_000, 0.8)
    }
}

#[async_trait]
impl OutputTransformer for BudgetAwareTruncator {
    async fn transform(&self, output: String, _tool_name: &str, state: &AgentState) -> String {
        let limit = if state.context_utilization() > self.aggressive_threshold {
            self.max_chars / 2
        } else {
            self.max_chars
        };

        if output.len() <= limit {
            return output;
        }

        // Truncate at char boundary
        let truncated: String = output.chars().take(limit).collect();
        format!(
            "{truncated}\n\n[output truncated from {} to {limit} chars]",
            output.len()
        )
    }
}

/// Extracts JSON from tool output, discarding surrounding text.
///
/// Useful for tools that embed JSON in verbose output.
pub struct JsonExtractor;

#[async_trait]
impl OutputTransformer for JsonExtractor {
    async fn transform(&self, output: String, _tool_name: &str, _state: &AgentState) -> String {
        // Try to find JSON object or array in the output
        if let Some(start) = output.find('{') {
            if let Some(end) = output.rfind('}') {
                if end >= start {
                    return output[start..=end].to_string();
                }
            }
        }
        if let Some(start) = output.find('[') {
            if let Some(end) = output.rfind(']') {
                if end >= start {
                    return output[start..=end].to_string();
                }
            }
        }
        // No JSON found, return as-is
        output
    }
}

/// Pipes output through multiple transformers in order.
pub struct TransformerChain {
    transformers: Vec<Box<dyn OutputTransformer>>,
}

impl TransformerChain {
    /// Create a chain from a list of transformers.
    #[must_use]
    pub fn new(transformers: Vec<Box<dyn OutputTransformer>>) -> Self {
        Self { transformers }
    }
}

#[async_trait]
impl OutputTransformer for TransformerChain {
    async fn transform(&self, mut output: String, tool_name: &str, state: &AgentState) -> String {
        for t in &self.transformers {
            output = t.transform(output, tool_name, state).await;
        }
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::model_info::ModelTier;

    fn state_with_utilization(util: f64) -> AgentState {
        let window = 1000;
        let mut state = AgentState::new(ModelTier::Medium, window);
        state.total_context_tokens = (util * window as f64) as usize;
        state
    }

    #[tokio::test]
    async fn test_budget_truncator_under_limit() {
        let t = BudgetAwareTruncator::new(100, 0.8);
        let state = state_with_utilization(0.5);
        let result = t.transform("short".to_string(), "test", &state).await;
        assert_eq!(result, "short");
    }

    #[tokio::test]
    async fn test_budget_truncator_over_limit() {
        let t = BudgetAwareTruncator::new(10, 0.8);
        let state = state_with_utilization(0.5);
        let result = t.transform("a".repeat(100), "test", &state).await;
        assert!(result.contains("[output truncated"));
        assert!(result.starts_with("aaaaaaaaaa"));
    }

    #[tokio::test]
    async fn test_budget_truncator_aggressive() {
        let t = BudgetAwareTruncator::new(20, 0.8);
        let state = state_with_utilization(0.9); // above threshold
                                                 // Limit becomes 20/2 = 10
        let result = t.transform("a".repeat(50), "test", &state).await;
        assert!(result.contains("[output truncated"));
        // Should truncate to 10 chars
        let first_line: &str = result.lines().next().unwrap();
        assert_eq!(first_line.len(), 10);
    }

    #[tokio::test]
    async fn test_json_extractor_object() {
        let t = JsonExtractor;
        let state = state_with_utilization(0.0);
        let result = t
            .transform(
                "Here is the result: {\"key\": \"value\"} done.".to_string(),
                "test",
                &state,
            )
            .await;
        assert_eq!(result, "{\"key\": \"value\"}");
    }

    #[tokio::test]
    async fn test_json_extractor_array() {
        let t = JsonExtractor;
        let state = state_with_utilization(0.0);
        let result = t
            .transform("Output: [1, 2, 3] end".to_string(), "test", &state)
            .await;
        assert_eq!(result, "[1, 2, 3]");
    }

    #[tokio::test]
    async fn test_json_extractor_no_json() {
        let t = JsonExtractor;
        let state = state_with_utilization(0.0);
        let result = t.transform("plain text".to_string(), "test", &state).await;
        assert_eq!(result, "plain text");
    }

    #[tokio::test]
    async fn test_transformer_chain() {
        let chain = TransformerChain::new(vec![
            Box::new(JsonExtractor),
            Box::new(BudgetAwareTruncator::new(5, 0.8)),
        ]);
        let state = state_with_utilization(0.5);
        let result = chain
            .transform(
                "Result: {\"key\": \"long_value_here\"}".to_string(),
                "test",
                &state,
            )
            .await;
        // First extracts JSON, then truncates to 5 chars
        assert!(result.contains("[output truncated"));
    }
}
