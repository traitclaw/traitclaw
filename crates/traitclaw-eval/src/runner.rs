//! Async `Metric` trait and `EvalRunner` execution engine.
//!
//! # Example
//!
//! ```rust
//! use traitclaw_eval::runner::{AsyncMetric, EvalRunner};
//! use traitclaw_eval::{EvalSuite, TestCase};
//! use async_trait::async_trait;
//!
//! struct AlwaysOne;
//!
//! #[async_trait]
//! impl AsyncMetric for AlwaysOne {
//!     fn name(&self) -> &'static str { "always_one" }
//!     async fn score(&self, _input: &str, _output: &str, _kw: &[&str]) -> f64 { 1.0 }
//! }
//!
//! # async fn example() {
//! let runner = EvalRunner::new().metric(Box::new(AlwaysOne)).threshold(0.8);
//! # }
//! ```

use std::sync::Arc;

use async_trait::async_trait;

use crate::{EvalReport, EvalSuite, TestResult};

/// Async trait for evaluation metrics.
///
/// Implement this to add custom scoring logic.
#[async_trait]
pub trait AsyncMetric: Send + Sync + 'static {
    /// Metric name — used as the key in `TestResult.scores`.
    fn name(&self) -> &'static str;

    /// Score the actual output.
    ///
    /// Returns a score from `0.0` (worst) to `1.0` (best).
    async fn score(&self, input: &str, actual_output: &str, expected_keywords: &[&str]) -> f64;
}

/// A callable async agent for use with `EvalRunner`.
///
/// Returns the agent's response for a given input string.
#[async_trait]
pub trait EvalAgent: Send + Sync {
    /// Run the agent on the given input and return a response.
    async fn respond(&self, input: &str) -> traitclaw_core::Result<String>;
}

/// Evaluation runner — executes a suite against an agent using async metrics.
pub struct EvalRunner {
    metrics: Vec<Arc<dyn AsyncMetric>>,
    threshold: f64,
}

impl EvalRunner {
    /// Create a new `EvalRunner` with no metrics and default threshold 0.7.
    #[must_use]
    pub fn new() -> Self {
        Self {
            metrics: Vec::new(),
            threshold: 0.7,
        }
    }

    /// Add a metric to score agent outputs with.
    #[must_use]
    pub fn metric(mut self, metric: Box<dyn AsyncMetric>) -> Self {
        self.metrics.push(Arc::from(metric));
        self
    }

    /// Set the minimum score threshold for a test case to pass.
    ///
    /// A case passes if **all** metric scores are ≥ threshold.
    #[must_use]
    pub fn threshold(mut self, threshold: f64) -> Self {
        self.threshold = threshold;
        self
    }

    /// Execute the evaluation suite against the agent.
    ///
    /// For each test case: calls `agent.respond(input)`, scores with all metrics,
    /// marks passed/failed, and aggregates into an `EvalReport`.
    ///
    /// # Errors
    ///
    /// Returns an error if the agent fails on any test case.
    pub async fn run(
        &self,
        agent: &dyn EvalAgent,
        suite: &EvalSuite,
    ) -> traitclaw_core::Result<EvalReport> {
        let mut results = Vec::new();
        let mut total_score = 0.0;
        let mut passed_count = 0;
        let mut score_count = 0;

        for case in suite.cases() {
            let actual_output = agent.respond(&case.input).await?;

            let keywords: Vec<&str> = case.expected_keywords.iter().map(String::as_str).collect();

            let mut scores = std::collections::HashMap::new();
            for metric in &self.metrics {
                let s = metric.score(&case.input, &actual_output, &keywords).await;
                scores.insert(metric.name().to_string(), s);
                total_score += s;
                score_count += 1;
            }

            // If no metrics configured, auto-pass based on keywords (keyword_match)
            if self.metrics.is_empty() {
                let kw_score = score_keywords(&actual_output, &keywords);
                scores.insert("keyword_match".to_string(), kw_score);
                total_score += kw_score;
                score_count += 1;
            }

            let all_pass = scores.values().all(|&s| s >= self.threshold);
            if all_pass {
                passed_count += 1;
            }

            results.push(TestResult {
                case_id: case.id.clone(),
                actual_output,
                scores,
                passed: all_pass,
            });
        }

        let average_score = if score_count > 0 {
            total_score / score_count as f64
        } else {
            0.0
        };

        Ok(EvalReport {
            suite_name: suite.name().to_string(),
            results,
            average_score,
            passed: passed_count,
            total: suite.cases().len(),
        })
    }
}

impl Default for EvalRunner {
    fn default() -> Self {
        Self::new()
    }
}

fn score_keywords(output: &str, keywords: &[&str]) -> f64 {
    if keywords.is_empty() {
        return 1.0;
    }
    let lower = output.to_lowercase();
    let matched = keywords.iter().filter(|&&kw| lower.contains(kw)).count();
    matched as f64 / keywords.len() as f64
}

// ── Adapters for sync Metric ─────────────────────────────────────────────────

/// Wraps a sync `Metric` impl as an `AsyncMetric`.
pub struct SyncMetricAdapter<M: crate::Metric>(pub M);

#[async_trait]
impl<M: crate::Metric> AsyncMetric for SyncMetricAdapter<M> {
    fn name(&self) -> &'static str {
        self.0.name()
    }

    async fn score(&self, input: &str, actual_output: &str, expected_keywords: &[&str]) -> f64 {
        self.0.score(input, actual_output, expected_keywords)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{EvalSuite, TestCase};

    struct EchoAgent;

    #[async_trait]
    impl EvalAgent for EchoAgent {
        async fn respond(&self, input: &str) -> traitclaw_core::Result<String> {
            Ok(format!("echo: {input}"))
        }
    }

    struct FixedMetric(f64, &'static str);

    #[async_trait]
    impl AsyncMetric for FixedMetric {
        fn name(&self) -> &'static str {
            self.1
        }
        async fn score(&self, _: &str, _: &str, _: &[&str]) -> f64 {
            self.0
        }
    }

    struct KeywordAsyncMetric;

    #[async_trait]
    impl AsyncMetric for KeywordAsyncMetric {
        fn name(&self) -> &'static str {
            "keyword"
        }
        async fn score(&self, _: &str, output: &str, kw: &[&str]) -> f64 {
            if kw.is_empty() {
                return 1.0;
            }
            let low = output.to_lowercase();
            let m = kw.iter().filter(|&&k| low.contains(k)).count();
            m as f64 / kw.len() as f64
        }
    }

    #[tokio::test]
    async fn test_eval_runner_three_cases() {
        // AC #7: 3 test cases with KeywordMetric → report with 3 results
        let suite = EvalSuite::new("suite")
            .add_case(TestCase::new("c1", "hello").expect_contains("echo"))
            .add_case(TestCase::new("c2", "world").expect_contains("echo"))
            .add_case(TestCase::new("c3", "foo").expect_contains("echo"));

        let runner = EvalRunner::new()
            .metric(Box::new(KeywordAsyncMetric))
            .threshold(0.8);

        let report = runner.run(&EchoAgent, &suite).await.unwrap();

        assert_eq!(report.results.len(), 3);
        assert_eq!(report.total, 3);
        // EchoAgent always includes "echo" → all should pass
        assert_eq!(report.passed, 3);
    }

    #[tokio::test]
    async fn test_eval_runner_threshold_fail() {
        // AC #8: threshold 0.8 → case scoring 0.0 marked as failed
        let suite =
            EvalSuite::new("s").add_case(TestCase::new("c1", "hello").expect_contains("xyzabc")); // won't match

        let runner = EvalRunner::new()
            .metric(Box::new(KeywordAsyncMetric))
            .threshold(0.8);

        let report = runner.run(&EchoAgent, &suite).await.unwrap();
        assert_eq!(report.passed, 0, "case with 0.0 keyword score should fail");
    }

    #[tokio::test]
    async fn test_eval_runner_average_score() {
        // Average across metrics and cases
        let suite = EvalSuite::new("s")
            .add_case(TestCase::new("c1", "hello"))
            .add_case(TestCase::new("c2", "world"));

        let runner = EvalRunner::new()
            .metric(Box::new(FixedMetric(0.8, "m")))
            .threshold(0.7);

        let report = runner.run(&EchoAgent, &suite).await.unwrap();
        assert!((report.average_score - 0.8).abs() < 1e-6);
        assert_eq!(report.passed, 2);
    }

    #[tokio::test]
    async fn test_sync_metric_adapter() {
        let adapter = SyncMetricAdapter(crate::KeywordMetric);
        let score = adapter.score("in", "hello world", &["hello"]).await;
        assert!((score - 1.0).abs() < 1e-6);
    }

    #[tokio::test]
    async fn test_empty_suite_gives_zero_results() {
        let suite = EvalSuite::new("empty");
        let runner = EvalRunner::new().metric(Box::new(KeywordAsyncMetric));
        let report = runner.run(&EchoAgent, &suite).await.unwrap();
        assert_eq!(report.results.len(), 0);
        assert_eq!(report.total, 0);
    }
}
