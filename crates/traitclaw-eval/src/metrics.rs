//! Specialized metrics for LLM output evaluation.
//!
//! - [`LlmJudgeMetric`]: LLM-powered quality scoring
//! - [`SchemaValidationMetric`]: JSON schema schema validation
//! - [`ToolUsageMetric`]: verifies expected tool calls were made

use std::sync::Arc;

use async_trait::async_trait;

use crate::runner::AsyncMetric;

// ─────────────────────────────────────────────────────────────────────────────
// LLM Judge provider trait
// ─────────────────────────────────────────────────────────────────────────────

/// A minimal provider interface for LLM judge calls.
///
/// Implement this to connect `LlmJudgeMetric` to any LLM backend.
#[async_trait]
pub trait JudgeProvider: Send + Sync + 'static {
    /// Call the LLM with the given prompt, return the text response.
    async fn complete(&self, prompt: &str) -> traitclaw_core::Result<String>;
}

// ─────────────────────────────────────────────────────────────────────────────
// LlmJudgeMetric
// ─────────────────────────────────────────────────────────────────────────────

/// LLM-based evaluation metric with named criteria.
///
/// Calls an LLM with a custom evaluation prompt and parses a 0.0–1.0 score.
///
/// # Example
///
/// ```rust
/// use traitclaw_eval::metrics::{JudgeProvider, LlmJudgeMetric};
/// use async_trait::async_trait;
///
/// struct MockJudge;
///
/// #[async_trait]
/// impl JudgeProvider for MockJudge {
///     async fn complete(&self, _prompt: &str) -> traitclaw_core::Result<String> {
///         Ok("Score: 0.85".to_string())
///     }
/// }
///
/// let metric = LlmJudgeMetric::new(MockJudge)
///     .with_criteria("accuracy", "Is the answer factually correct?");
/// ```
pub struct LlmJudgeMetric<P: JudgeProvider> {
    provider: Arc<P>,
    criteria: Vec<(String, String)>,
}

impl<P: JudgeProvider> LlmJudgeMetric<P> {
    /// Create a new `LlmJudgeMetric` backed by the given provider.
    #[must_use]
    pub fn new(provider: P) -> Self {
        Self {
            provider: Arc::new(provider),
            criteria: Vec::new(),
        }
    }

    /// Add a named evaluation criterion.
    ///
    /// The criterion name and prompt are used to build the judge prompt.
    #[must_use]
    pub fn with_criteria(mut self, name: impl Into<String>, prompt: impl Into<String>) -> Self {
        self.criteria.push((name.into(), prompt.into()));
        self
    }
}

#[async_trait]
impl<P: JudgeProvider> AsyncMetric for LlmJudgeMetric<P> {
    fn name(&self) -> &'static str {
        "llm_judge"
    }

    async fn score(&self, input: &str, actual_output: &str, _kw: &[&str]) -> f64 {
        let criteria_text = if self.criteria.is_empty() {
            "Is this a high-quality response?".to_string()
        } else {
            self.criteria
                .iter()
                .map(|(name, prompt)| format!("- {name}: {prompt}"))
                .collect::<Vec<_>>()
                .join("\n")
        };

        let prompt = format!(
            "Evaluate the following agent response:\n\nInput: {input}\n\nResponse: {actual_output}\n\nCriteria:\n{criteria_text}\n\nProvide a score from 0.0 to 1.0. Respond with only: Score: <number>"
        );

        match self.provider.complete(&prompt).await {
            Ok(response) => parse_score(&response),
            Err(_) => 0.0,
        }
    }
}

/// Parse `Score: 0.85` or standalone `0.85` from LLM response.
pub(crate) fn parse_score(response: &str) -> f64 {
    // Try "Score: X.XX" format first
    for line in response.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("Score:") {
            if let Ok(score) = rest.trim().parse::<f64>() {
                return score.clamp(0.0, 1.0);
            }
        }
        // Fallback: try parsing the whole line as a number
        if let Ok(score) = line.parse::<f64>() {
            return score.clamp(0.0, 1.0);
        }
    }
    0.0
}

// ─────────────────────────────────────────────────────────────────────────────
// SchemaValidationMetric
// ─────────────────────────────────────────────────────────────────────────────

/// Validates that the agent output is valid JSON matching an expected schema shape.
///
/// The "schema" here is a `serde_json::Value` used as a **template** — all keys
/// present in the schema must be present in the output. A full JSON schema validator
/// would require an external crate; this is a lightweight key-presence check.
///
/// # Example
///
/// ```rust
/// use traitclaw_eval::metrics::SchemaValidationMetric;
/// use traitclaw_eval::runner::AsyncMetric;
///
/// let metric = SchemaValidationMetric::new(serde_json::json!({
///     "name": "string",
///     "score": "number"
/// }));
/// ```
pub struct SchemaValidationMetric {
    schema: serde_json::Value,
}

impl SchemaValidationMetric {
    /// Create a new `SchemaValidationMetric` with the given expected schema.
    #[must_use]
    pub fn new(schema: serde_json::Value) -> Self {
        Self { schema }
    }
}

#[async_trait]
impl AsyncMetric for SchemaValidationMetric {
    fn name(&self) -> &'static str {
        "schema_validation"
    }

    async fn score(&self, _input: &str, actual_output: &str, _kw: &[&str]) -> f64 {
        // Parse output as JSON
        let Ok(output_val) = serde_json::from_str::<serde_json::Value>(actual_output) else {
            return 0.0; // not valid JSON
        };

        // Check that all schema keys exist in output
        let schema_obj = match &self.schema {
            serde_json::Value::Object(m) => m,
            _ => return if output_val == self.schema { 1.0 } else { 0.0 },
        };

        let output_obj = match &output_val {
            serde_json::Value::Object(m) => m,
            _ => return 0.0,
        };

        if schema_obj.is_empty() {
            return 1.0;
        }

        let present = schema_obj
            .keys()
            .filter(|k| output_obj.contains_key(*k))
            .count();
        present as f64 / schema_obj.len() as f64
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ToolUsageMetric
// ─────────────────────────────────────────────────────────────────────────────

/// Checks whether the agent output mentions expected tool names.
///
/// A simple heuristic: score = fraction of expected tool names mentioned
/// in the output. Works well when agents include tool call summaries in responses.
///
/// # Example
///
/// ```rust
/// use traitclaw_eval::metrics::ToolUsageMetric;
/// use traitclaw_eval::runner::AsyncMetric;
///
/// let metric = ToolUsageMetric::new(vec!["search", "calculator"]);
/// ```
pub struct ToolUsageMetric {
    expected_tools: Vec<String>,
}

impl ToolUsageMetric {
    /// Create a new `ToolUsageMetric` expecting the given tool names.
    #[must_use]
    pub fn new(expected_tools: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            expected_tools: expected_tools.into_iter().map(Into::into).collect(),
        }
    }
}

#[async_trait]
impl AsyncMetric for ToolUsageMetric {
    fn name(&self) -> &'static str {
        "tool_usage"
    }

    async fn score(&self, _input: &str, actual_output: &str, _kw: &[&str]) -> f64 {
        if self.expected_tools.is_empty() {
            return 1.0;
        }

        let output_lower = actual_output.to_lowercase();
        let found = self
            .expected_tools
            .iter()
            .filter(|tool| output_lower.contains(tool.to_lowercase().as_str()))
            .count();

        found as f64 / self.expected_tools.len() as f64
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── LlmJudgeMetric ───────────────────────────────────────────────────────

    struct MockJudge(String);

    #[async_trait]
    impl JudgeProvider for MockJudge {
        async fn complete(&self, _prompt: &str) -> traitclaw_core::Result<String> {
            Ok(self.0.clone())
        }
    }

    #[tokio::test]
    async fn test_llm_judge_parses_score() {
        // AC #6: mock returns 0.85 → metric score = 0.85
        let metric = LlmJudgeMetric::new(MockJudge("Score: 0.85".to_string()))
            .with_criteria("accuracy", "Is it accurate?");

        let score = metric.score("input", "output", &[]).await;
        assert!((score - 0.85).abs() < 1e-6, "expected 0.85, got {score}");
    }

    #[tokio::test]
    async fn test_llm_judge_clamps_above_one() {
        let metric = LlmJudgeMetric::new(MockJudge("Score: 1.5".to_string()));
        let score = metric.score("in", "out", &[]).await;
        assert!((score - 1.0).abs() < 1e-6);
    }

    #[tokio::test]
    async fn test_llm_judge_invalid_response_returns_zero() {
        let metric = LlmJudgeMetric::new(MockJudge("I cannot provide a score.".to_string()));
        let score = metric.score("in", "out", &[]).await;
        assert!((score - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_parse_score_variants() {
        assert!((parse_score("Score: 0.75") - 0.75).abs() < 1e-6);
        assert!((parse_score("0.90") - 0.90).abs() < 1e-6);
        assert!((parse_score("no score here") - 0.0).abs() < 1e-6);
        assert!((parse_score("Score: 1.5") - 1.0).abs() < 1e-6); // clamped
    }

    // ── SchemaValidationMetric ───────────────────────────────────────────────

    #[tokio::test]
    async fn test_schema_validation_valid_json() {
        // AC #7: valid JSON with all required keys → score = 1.0
        let metric = SchemaValidationMetric::new(serde_json::json!({
            "name": "string",
            "score": "number"
        }));
        let output = r#"{"name": "test", "score": 42}"#;
        let score = metric.score("in", output, &[]).await;
        assert!((score - 1.0).abs() < 1e-6, "expected 1.0, got {score}");
    }

    #[tokio::test]
    async fn test_schema_validation_partial_keys() {
        let metric = SchemaValidationMetric::new(serde_json::json!({
            "name": "string",
            "score": "number",
            "extra": "string"
        }));
        let output = r#"{"name": "test"}"#; // only 1/3 keys
        let score = metric.score("in", output, &[]).await;
        // 1/3 ≈ 0.333
        assert!(score < 0.5, "expected < 0.5, got {score}");
    }

    #[tokio::test]
    async fn test_schema_validation_invalid_json() {
        // AC #7: invalid JSON → score = 0.0
        let metric = SchemaValidationMetric::new(serde_json::json!({"name": "string"}));
        let score = metric.score("in", "not json at all", &[]).await;
        assert!((score - 0.0).abs() < 1e-6);
    }

    // ── ToolUsageMetric ──────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_tool_usage_all_found() {
        let metric = ToolUsageMetric::new(vec!["search", "calculator"]);
        let score = metric
            .score("in", "I used search and calculator tools", &[])
            .await;
        assert!((score - 1.0).abs() < 1e-6);
    }

    #[tokio::test]
    async fn test_tool_usage_partial() {
        let metric = ToolUsageMetric::new(vec!["search", "calculator"]);
        let score = metric.score("in", "I only used search", &[]).await;
        assert!((score - 0.5).abs() < 1e-6);
    }

    #[tokio::test]
    async fn test_tool_usage_none_found() {
        let metric = ToolUsageMetric::new(vec!["search"]);
        let score = metric.score("in", "I didn't call any tools", &[]).await;
        assert!((score - 0.0).abs() < 1e-6);
    }

    #[tokio::test]
    async fn test_tool_usage_empty_expected() {
        let metric = ToolUsageMetric::new(Vec::<String>::new());
        let score = metric.score("in", "anything", &[]).await;
        assert!((score - 1.0).abs() < 1e-6);
    }
}
