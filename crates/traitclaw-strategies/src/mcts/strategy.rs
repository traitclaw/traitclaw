//! MCTS strategy implementation with parallel branch evaluation.

use std::sync::Mutex;
use std::time::Instant;

use async_trait::async_trait;
use tokio::task::JoinSet;
use tracing;

use traitclaw_core::agent::{AgentOutput, RunUsage};
use traitclaw_core::traits::strategy::{AgentRuntime, AgentStrategy};
use traitclaw_core::types::completion::{CompletionRequest, ResponseContent};
use traitclaw_core::types::message::Message;
use traitclaw_core::{Error, Result};

use super::scoring::{default_scoring_fn, ScoringFn};
use crate::common::ThoughtStep;

/// Default number of parallel branches.
const DEFAULT_BRANCHES: usize = 5;
/// Default maximum depth.
const DEFAULT_MAX_DEPTH: usize = 3;

/// A branch evaluation result from MCTS.
#[derive(Debug, Clone)]
pub struct BranchResult {
    /// The branch index (0-based).
    pub branch_index: usize,
    /// The score assigned to this branch.
    pub score: f64,
    /// The generated answer text.
    pub answer: String,
    /// Thought steps for this branch.
    pub thought_steps: Vec<ThoughtStep>,
}

/// Monte Carlo Tree Search (MCTS) reasoning strategy.
///
/// Explores multiple reasoning paths in parallel using `tokio::task::JoinSet`
/// and selects the highest-scoring result using a configurable [`ScoringFn`].
///
/// # Example
///
/// ```no_run
/// use traitclaw_strategies::mcts::MctsStrategy;
///
/// let strategy = MctsStrategy::builder()
///     .branches(5)
///     .max_depth(3)
///     .build()
///     .unwrap();
/// ```
pub struct MctsStrategy {
    /// Number of parallel branches to explore.
    branches: usize,
    /// Maximum search depth per branch.
    max_depth: usize,
    /// Scoring function for ranking branches.
    scoring_fn: ScoringFn,
    /// Results from the last execution.
    branch_results: Mutex<Vec<BranchResult>>,
    /// Collected thought steps from the best branch.
    thought_steps: Mutex<Vec<ThoughtStep>>,
}

impl std::fmt::Debug for MctsStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MctsStrategy")
            .field("branches", &self.branches)
            .field("max_depth", &self.max_depth)
            .finish()
    }
}

impl MctsStrategy {
    /// Create a new builder for `MctsStrategy`.
    #[must_use]
    pub fn builder() -> MctsStrategyBuilder {
        MctsStrategyBuilder::default()
    }

    /// Returns the thought steps from the best branch of the last execution.
    #[must_use]
    pub fn thought_steps(&self) -> Vec<ThoughtStep> {
        self.thought_steps
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }

    /// Returns the branch results from the last execution.
    #[must_use]
    pub fn branch_results(&self) -> Vec<BranchResult> {
        self.branch_results
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }

    /// Returns the number of parallel branches.
    #[must_use]
    pub fn branches(&self) -> usize {
        self.branches
    }

    /// Returns the maximum search depth.
    #[must_use]
    pub fn max_depth(&self) -> usize {
        self.max_depth
    }
}

/// Builder for [`MctsStrategy`].
pub struct MctsStrategyBuilder {
    branches: usize,
    max_depth: usize,
    scoring_fn: Option<ScoringFn>,
}

impl std::fmt::Debug for MctsStrategyBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MctsStrategyBuilder")
            .field("branches", &self.branches)
            .field("max_depth", &self.max_depth)
            .field("scoring_fn", &self.scoring_fn.as_ref().map(|_| "<fn>"))
            .finish()
    }
}

impl Default for MctsStrategyBuilder {
    fn default() -> Self {
        Self {
            branches: DEFAULT_BRANCHES,
            max_depth: DEFAULT_MAX_DEPTH,
            scoring_fn: None,
        }
    }
}

impl MctsStrategyBuilder {
    /// Set the number of parallel branches to explore.
    ///
    /// Default: 5
    #[must_use]
    pub fn branches(mut self, n: usize) -> Self {
        self.branches = n;
        self
    }

    /// Set the maximum depth per branch.
    ///
    /// Default: 3
    #[must_use]
    pub fn max_depth(mut self, n: usize) -> Self {
        self.max_depth = n;
        self
    }

    /// Set a custom scoring function for ranking branches.
    ///
    /// If not set, a default length/structure heuristic is used.
    #[must_use]
    pub fn scoring(mut self, f: ScoringFn) -> Self {
        self.scoring_fn = Some(f);
        self
    }

    /// Build the `MctsStrategy`.
    ///
    /// # Errors
    ///
    /// Returns `Error::Config` if `branches` or `max_depth` is 0.
    pub fn build(self) -> Result<MctsStrategy> {
        if self.branches == 0 {
            return Err(Error::Config(
                "MctsStrategy: branches must be greater than 0".into(),
            ));
        }
        if self.max_depth == 0 {
            return Err(Error::Config(
                "MctsStrategy: max_depth must be greater than 0".into(),
            ));
        }

        Ok(MctsStrategy {
            branches: self.branches,
            max_depth: self.max_depth,
            scoring_fn: self.scoring_fn.unwrap_or_else(default_scoring_fn),
            branch_results: Mutex::new(Vec::new()),
            thought_steps: Mutex::new(Vec::new()),
        })
    }
}

#[async_trait]
impl AgentStrategy for MctsStrategy {
    async fn execute(
        &self,
        runtime: &AgentRuntime,
        input: &str,
        session_id: &str,
    ) -> Result<AgentOutput> {
        let start = Instant::now();

        // Clear previous results
        self.branch_results
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clear();
        self.thought_steps
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clear();

        // Save user message to memory
        if let Err(e) = runtime
            .memory
            .append(session_id, Message::user(input))
            .await
        {
            tracing::warn!("Failed to save user message to memory: {e}");
        }

        let model_info = runtime.provider.model_info();
        let mut total_tokens: usize = 0;

        // Spawn parallel branches
        let mut join_set: JoinSet<Result<(usize, String, Vec<ThoughtStep>, usize)>> =
            JoinSet::new();

        for branch_idx in 0..self.branches {
            let provider = runtime.provider.clone();
            let model_name = model_info.name.clone();
            let input_owned = input.to_string();
            let max_tokens = runtime.config.max_tokens;
            let base_temp = runtime.config.temperature.unwrap_or(0.7);
            let depth = self.max_depth;

            join_set.spawn(async move {
                // Vary temperature per branch for diversity, clamped to [0.0, 2.0]
                let temp = (base_temp + (branch_idx as f32 * 0.05)).min(2.0);
                let mut branch_steps = Vec::new();

                // System prompt for this branch
                let system = format!(
                    "You are reasoning branch {}. Think carefully about the problem \
                     and provide your best answer. Be thorough in your analysis \
                     (up to {} reasoning depths).",
                    branch_idx + 1,
                    depth,
                );

                let messages = vec![Message::system(&system), Message::user(&input_owned)];

                branch_steps.push(ThoughtStep::Think {
                    content: format!("Branch {} exploring solution...", branch_idx + 1),
                });

                let request = CompletionRequest {
                    model: model_name,
                    messages,
                    tools: vec![],
                    max_tokens,
                    temperature: Some(temp),
                    response_format: None,
                    stream: false,
                };

                let response = provider.complete(request).await?;
                let tokens = response.usage.total_tokens;

                match response.content {
                    ResponseContent::Text(text) => {
                        branch_steps.push(ThoughtStep::Answer {
                            content: text.clone(),
                        });
                        Ok((branch_idx, text, branch_steps, tokens))
                    }
                    ResponseContent::ToolCalls(_) => Err(Error::Runtime(
                        "MctsStrategy: unexpected tool calls in branch evaluation".into(),
                    )),
                }
            });
        }

        // Collect results
        let mut results = Vec::new();
        while let Some(join_result) = join_set.join_next().await {
            match join_result {
                Ok(Ok((branch_idx, answer, steps, tokens))) => {
                    let score = (self.scoring_fn)(&answer);
                    if score.is_nan() {
                        tracing::warn!(branch_idx, "ScoringFn returned NaN, treating as 0.0");
                    }
                    let safe_score = if score.is_nan() { 0.0 } else { score };
                    total_tokens += tokens;
                    results.push(BranchResult {
                        branch_index: branch_idx,
                        score: safe_score,
                        answer,
                        thought_steps: steps,
                    });
                }
                Ok(Err(e)) => {
                    tracing::warn!("MCTS branch failed: {e}");
                }
                Err(e) => {
                    tracing::warn!("MCTS branch panicked: {e}");
                }
            }
        }

        if results.is_empty() {
            return Err(Error::Runtime("MctsStrategy: all branches failed".into()));
        }

        // Select best branch
        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let best = &results[0];

        // Store results
        let best_steps = best.thought_steps.clone();
        let best_answer = best.answer.clone();
        let best_branch = best.branch_index;
        let best_score = best.score;

        *self
            .branch_results
            .lock()
            .unwrap_or_else(|e| e.into_inner()) = results;
        *self.thought_steps.lock().unwrap_or_else(|e| e.into_inner()) = best_steps;

        // Save best answer to memory
        if let Err(e) = runtime
            .memory
            .append(session_id, Message::assistant(&best_answer))
            .await
        {
            tracing::warn!("Failed to save assistant response: {e}");
        }

        let usage = RunUsage {
            tokens: total_tokens,
            iterations: self.branches,
            duration: start.elapsed(),
        };

        tracing::info!(best_branch, best_score, "MCTS selected best branch");

        Ok(AgentOutput::text_with_usage(best_answer, usage))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_builder_defaults() {
        let strategy = MctsStrategy::builder().build().unwrap();
        assert_eq!(strategy.branches(), DEFAULT_BRANCHES);
        assert_eq!(strategy.max_depth(), DEFAULT_MAX_DEPTH);
        assert!(strategy.thought_steps().is_empty());
        assert!(strategy.branch_results().is_empty());
    }

    #[test]
    fn test_builder_custom() {
        let strategy = MctsStrategy::builder()
            .branches(3)
            .max_depth(2)
            .build()
            .unwrap();
        assert_eq!(strategy.branches(), 3);
        assert_eq!(strategy.max_depth(), 2);
    }

    #[test]
    fn test_builder_custom_scoring() {
        let scorer: ScoringFn = Arc::new(|_| 0.5);
        let strategy = MctsStrategy::builder().scoring(scorer).build().unwrap();
        assert_eq!(strategy.branches(), DEFAULT_BRANCHES);
    }

    #[test]
    fn test_builder_rejects_zero_branches() {
        let result = MctsStrategy::builder().branches(0).build();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("branches"));
    }

    #[test]
    fn test_builder_rejects_zero_depth() {
        let result = MctsStrategy::builder().max_depth(0).build();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("max_depth"));
    }

    #[test]
    fn test_object_safety() {
        let strategy = MctsStrategy::builder().build().unwrap();
        let _boxed: Box<dyn AgentStrategy> = Box::new(strategy);
    }

    // ── F1: Async Execution Tests ───────────────────────────────────────

    #[tokio::test]
    async fn test_execute_selects_best_branch() {
        use traitclaw_test_utils::provider::MockProvider;
        use traitclaw_test_utils::runtime::make_runtime;

        // All branches get same text, scoring differentiates by length
        let provider = MockProvider::text("Branch answer with enough content to score well.");
        let runtime = make_runtime(provider, vec![]);

        let strategy = MctsStrategy::builder()
            .branches(3)
            .max_depth(1)
            .build()
            .unwrap();

        let result = strategy.execute(&runtime, "test question", "s1").await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.text().contains("Branch answer"));

        // Should have branch results
        let results = strategy.branch_results();
        assert_eq!(results.len(), 3);

        // All branches should have scores
        for br in &results {
            assert!(br.score >= 0.0);
            assert!(br.score <= 1.0);
        }
    }

    // ── F2: Error Path — all branches fail ──────────────────────────────

    #[tokio::test]
    async fn test_execute_all_branches_fail() {
        use traitclaw_test_utils::provider::MockProvider;
        use traitclaw_test_utils::runtime::make_runtime;

        let provider = MockProvider::error("provider failed");
        let runtime = make_runtime(provider, vec![]);

        let strategy = MctsStrategy::builder().branches(2).build().unwrap();

        let result = strategy.execute(&runtime, "question", "s1").await;
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("all branches failed"),
            "Expected 'all branches failed' error: {err}"
        );
    }

    // ── F2: Error Path — unexpected tool calls ──────────────────────────

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

        let strategy = MctsStrategy::builder().branches(1).build().unwrap();

        let result = strategy.execute(&runtime, "question", "s1").await;
        // All branches fail because they get unexpected tool calls
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("all branches failed"),
            "Unexpected tool calls should cause branch failure: {err}"
        );
    }
}
