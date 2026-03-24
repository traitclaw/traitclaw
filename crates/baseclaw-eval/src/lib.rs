//! Evaluation framework for the `BaseClaw` AI agent framework.
//!
//! Provides `EvalSuite`, `TestCase`, and `Metric` abstractions for
//! measuring agent quality. Includes built-in metrics for relevancy
//! and keyword matching.
//!
//! # Quick Start
//!
//! ```rust
//! use baseclaw_eval::{EvalSuite, TestCase, KeywordMetric, Metric};
//!
//! let suite = EvalSuite::new("quality_tests")
//!     .add_case(TestCase::new("greeting", "Say hello")
//!         .expect_contains("hello"));
//!
//! assert_eq!(suite.name(), "quality_tests");
//! assert_eq!(suite.cases().len(), 1);
//!
//! let metric = KeywordMetric;
//! let score = metric.score("Say hello", "Hello! How can I help?", &["hello"]);
//! assert!(score > 0.0);
//! ```

#![deny(warnings)]
#![deny(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::cast_precision_loss)] // usize→f64 for scoring is acceptable
#![allow(clippy::doc_markdown)]

use serde::{Deserialize, Serialize};

/// A suite of evaluation test cases.
pub struct EvalSuite {
    name: String,
    cases: Vec<TestCase>,
}

impl EvalSuite {
    /// Create a new evaluation suite.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            cases: Vec::new(),
        }
    }

    /// Add a test case to the suite.
    #[must_use]
    pub fn add_case(mut self, case: TestCase) -> Self {
        self.cases.push(case);
        self
    }

    /// Get the suite name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get all test cases.
    #[must_use]
    pub fn cases(&self) -> &[TestCase] {
        &self.cases
    }
}

/// A single evaluation test case.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    /// Test case identifier.
    pub id: String,
    /// Input prompt for the agent.
    pub input: String,
    /// Expected output keywords (for keyword matching).
    pub expected_keywords: Vec<String>,
    /// Optional expected exact output.
    pub expected_output: Option<String>,
}

impl TestCase {
    /// Create a new test case.
    #[must_use]
    pub fn new(id: impl Into<String>, input: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            input: input.into(),
            expected_keywords: Vec::new(),
            expected_output: None,
        }
    }

    /// Add an expected keyword to match in the output.
    #[must_use]
    pub fn expect_contains(mut self, keyword: impl Into<String>) -> Self {
        self.expected_keywords.push(keyword.into());
        self
    }

    /// Set the expected exact output.
    #[must_use]
    pub fn expect_output(mut self, output: impl Into<String>) -> Self {
        self.expected_output = Some(output.into());
        self
    }
}

/// A single test case result.
#[derive(Debug, Clone, Serialize)]
pub struct TestResult {
    /// Test case ID.
    pub case_id: String,
    /// The actual output from the agent.
    pub actual_output: String,
    /// Metric scores (metric_name to score 0.0-1.0).
    pub scores: std::collections::HashMap<String, f64>,
    /// Whether the test passed (all scores above threshold).
    pub passed: bool,
}

/// An evaluation report.
#[derive(Debug, Clone, Serialize)]
pub struct EvalReport {
    /// Suite name.
    pub suite_name: String,
    /// Individual test results.
    pub results: Vec<TestResult>,
    /// Average score across all tests and metrics.
    pub average_score: f64,
    /// Number of tests that passed.
    pub passed: usize,
    /// Total number of tests.
    pub total: usize,
}

impl EvalReport {
    /// Generate a human-readable summary.
    #[must_use]
    pub fn summary(&self) -> String {
        format!(
            "Eval Report: {}\n  Passed: {}/{} ({:.1}%)\n  Average Score: {:.2}",
            self.suite_name,
            self.passed,
            self.total,
            if self.total > 0 {
                self.passed as f64 / self.total as f64 * 100.0
            } else {
                0.0
            },
            self.average_score,
        )
    }
}

/// Trait for evaluation metrics.
pub trait Metric: Send + Sync + 'static {
    /// Metric name.
    fn name(&self) -> &'static str;

    /// Score the actual output against the expected criteria.
    ///
    /// Returns a score from 0.0 (worst) to 1.0 (best).
    fn score(&self, input: &str, actual_output: &str, expected_keywords: &[&str]) -> f64;
}

/// Built-in keyword matching metric.
///
/// Scores based on the fraction of expected keywords found in the output.
pub struct KeywordMetric;

impl Metric for KeywordMetric {
    fn name(&self) -> &'static str {
        "keyword_match"
    }

    fn score(&self, _input: &str, actual_output: &str, expected_keywords: &[&str]) -> f64 {
        if expected_keywords.is_empty() {
            return 1.0;
        }
        let output_lower = actual_output.to_lowercase();
        let matched = expected_keywords
            .iter()
            .filter(|kw| output_lower.contains(&kw.to_lowercase()))
            .count();
        matched as f64 / expected_keywords.len() as f64
    }
}

/// Built-in length-based relevancy metric.
///
/// Penalizes very short or very long responses relative to input length.
pub struct LengthRelevancyMetric;

impl Metric for LengthRelevancyMetric {
    fn name(&self) -> &'static str {
        "length_relevancy"
    }

    fn score(&self, input: &str, actual_output: &str, _expected_keywords: &[&str]) -> f64 {
        let input_len = input.len() as f64;
        let output_len = actual_output.len() as f64;

        if output_len == 0.0 {
            return 0.0;
        }

        // Ideal ratio: output is 2-10x the input length
        let ratio = output_len / input_len.max(1.0);
        if (2.0..=10.0).contains(&ratio) {
            1.0
        } else if ratio < 2.0 {
            ratio / 2.0
        } else {
            (10.0 / ratio).min(1.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eval_suite_builder() {
        let suite = EvalSuite::new("test_suite")
            .add_case(TestCase::new("t1", "Hello"))
            .add_case(TestCase::new("t2", "World"));
        assert_eq!(suite.name(), "test_suite");
        assert_eq!(suite.cases().len(), 2);
    }

    #[test]
    fn test_test_case_builder() {
        let tc = TestCase::new("t1", "prompt")
            .expect_contains("keyword1")
            .expect_contains("keyword2")
            .expect_output("exact output");
        assert_eq!(tc.expected_keywords.len(), 2);
        assert_eq!(tc.expected_output, Some("exact output".into()));
    }

    #[test]
    fn test_keyword_metric_all_match() {
        let m = KeywordMetric;
        let score = m.score("input", "hello world foo", &["hello", "world"]);
        assert!((score - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_keyword_metric_partial_match() {
        let m = KeywordMetric;
        let score = m.score("input", "hello there", &["hello", "world"]);
        assert!((score - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_keyword_metric_no_match() {
        let m = KeywordMetric;
        let score = m.score("input", "nothing here", &["hello", "world"]);
        assert!((score - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_keyword_metric_empty_keywords() {
        let m = KeywordMetric;
        let score = m.score("input", "anything", &[]);
        assert!((score - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_length_relevancy_ideal() {
        let m = LengthRelevancyMetric;
        // Output 5x input = ideal range
        let score = m.score("hello", "hello world this is a response text here!", &[]);
        assert!(score > 0.5);
    }

    #[test]
    fn test_length_relevancy_empty_output() {
        let m = LengthRelevancyMetric;
        let score = m.score("hello", "", &[]);
        assert!((score - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_eval_report_summary() {
        let report = EvalReport {
            suite_name: "test".into(),
            results: vec![],
            average_score: 0.85,
            passed: 8,
            total: 10,
        };
        let s = report.summary();
        assert!(s.contains("8/10"));
        assert!(s.contains("80.0%"));
    }
}
