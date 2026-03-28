//! Scoring function types for MCTS branch evaluation.

use std::sync::Arc;

/// A scoring function that evaluates a candidate answer.
///
/// Returns a `f64` score where higher values indicate better quality.
/// The function receives the candidate answer text and should return
/// a score in the range `[0.0, 1.0]`.
///
/// # Example
///
/// ```
/// use std::sync::Arc;
/// use traitclaw_strategies::mcts::ScoringFn;
///
/// let length_scorer: ScoringFn = Arc::new(|answer: &str| {
///     // Simple scoring: longer answers score higher (up to 1.0)
///     (answer.len() as f64 / 500.0).min(1.0)
/// });
///
/// assert!(length_scorer("short") < length_scorer("a longer answer"));
/// ```
pub type ScoringFn = Arc<dyn Fn(&str) -> f64 + Send + Sync>;

/// Default scoring function that scores based on response length and structure.
///
/// This is a simple heuristic; production use should provide a custom
/// `ScoringFn` (e.g., using LLM self-evaluation).
pub(crate) fn default_scoring_fn() -> ScoringFn {
    Arc::new(|answer: &str| {
        let len_score = (answer.len() as f64 / 200.0).min(1.0);
        let has_structure = if answer.contains('\n') { 0.1 } else { 0.0 };
        (len_score + has_structure).min(1.0)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_scoring_fn_returns_bounded_values() {
        let scorer = default_scoring_fn();
        assert!(scorer("") >= 0.0);
        assert!(scorer("") <= 1.0);
        assert!(scorer("a".repeat(500).as_str()) <= 1.0);
    }

    #[test]
    fn test_custom_scoring_fn() {
        let scorer: ScoringFn = Arc::new(
            |answer: &str| {
                if answer.contains("correct") {
                    1.0
                } else {
                    0.0
                }
            },
        );
        assert_eq!(scorer("correct answer"), 1.0);
        assert_eq!(scorer("wrong answer"), 0.0);
    }
}
