//! Shared thought step types for strategy observability.

use serde::{Deserialize, Serialize};

/// Represents a single step in a strategy's reasoning process.
///
/// Each variant corresponds to a distinct phase of reasoning.
/// Strategies emit `ThoughtStep` events during execution, enabling
/// developers to observe, log, and debug the reasoning process.
///
/// # Examples
///
/// ```
/// use traitclaw_strategies::ThoughtStep;
///
/// let step = ThoughtStep::Think {
///     content: "I need to search for the answer.".to_string(),
/// };
/// println!("{:?}", step);
///
/// // Serialize to JSON
/// let json = serde_json::to_string(&step).unwrap();
/// assert!(json.contains("Think"));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ThoughtStep {
    /// The strategy is reasoning about the problem.
    Think {
        /// The reasoning content produced by the LLM.
        content: String,
    },
    /// The strategy is invoking a tool.
    Act {
        /// Name of the tool being called.
        tool_name: String,
        /// Input arguments for the tool call.
        tool_input: serde_json::Value,
    },
    /// The strategy received output from a tool invocation.
    Observe {
        /// The output returned by the tool.
        tool_output: String,
    },
    /// The strategy has produced a final answer.
    Answer {
        /// The final answer content.
        content: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_think_serialization() {
        let step = ThoughtStep::Think {
            content: "Let me analyze this problem.".to_string(),
        };
        let json = serde_json::to_string(&step).unwrap();
        assert!(json.contains("\"type\":\"Think\""));
        assert!(json.contains("Let me analyze this problem."));
    }

    #[test]
    fn test_act_serialization() {
        let step = ThoughtStep::Act {
            tool_name: "search".to_string(),
            tool_input: serde_json::json!({"query": "rust async"}),
        };
        let json = serde_json::to_string(&step).unwrap();
        assert!(json.contains("\"type\":\"Act\""));
        assert!(json.contains("\"tool_name\":\"search\""));
        assert!(json.contains("rust async"));
    }

    #[test]
    fn test_observe_serialization() {
        let step = ThoughtStep::Observe {
            tool_output: "Found 42 results.".to_string(),
        };
        let json = serde_json::to_string(&step).unwrap();
        assert!(json.contains("\"type\":\"Observe\""));
        assert!(json.contains("Found 42 results."));
    }

    #[test]
    fn test_answer_serialization() {
        let step = ThoughtStep::Answer {
            content: "The answer is 42.".to_string(),
        };
        let json = serde_json::to_string(&step).unwrap();
        assert!(json.contains("\"type\":\"Answer\""));
        assert!(json.contains("The answer is 42."));
    }

    #[test]
    fn test_clone() {
        let step = ThoughtStep::Think {
            content: "Cloneable step.".to_string(),
        };
        let cloned = step.clone();
        let original_json = serde_json::to_string(&step).unwrap();
        let cloned_json = serde_json::to_string(&cloned).unwrap();
        assert_eq!(original_json, cloned_json);
    }

    #[test]
    fn test_debug() {
        let step = ThoughtStep::Think {
            content: "Debug output.".to_string(),
        };
        let debug = format!("{:?}", step);
        assert!(debug.contains("Think"));
        assert!(debug.contains("Debug output."));
    }

    // ── F3: Serde Round-Trip Tests ──────────────────────────────────────

    #[test]
    fn test_think_roundtrip() {
        let step = ThoughtStep::Think {
            content: "Round-trip test.".to_string(),
        };
        let json = serde_json::to_string(&step).unwrap();
        let deserialized: ThoughtStep = serde_json::from_str(&json).unwrap();
        assert!(
            matches!(deserialized, ThoughtStep::Think { content } if content == "Round-trip test.")
        );
    }

    #[test]
    fn test_act_roundtrip() {
        let step = ThoughtStep::Act {
            tool_name: "search".to_string(),
            tool_input: serde_json::json!({"query": "rust"}),
        };
        let json = serde_json::to_string(&step).unwrap();
        let deserialized: ThoughtStep = serde_json::from_str(&json).unwrap();
        assert!(
            matches!(deserialized, ThoughtStep::Act { tool_name, .. } if tool_name == "search")
        );
    }

    #[test]
    fn test_observe_roundtrip() {
        let step = ThoughtStep::Observe {
            tool_output: "42 results".to_string(),
        };
        let json = serde_json::to_string(&step).unwrap();
        let deserialized: ThoughtStep = serde_json::from_str(&json).unwrap();
        assert!(
            matches!(deserialized, ThoughtStep::Observe { tool_output } if tool_output == "42 results")
        );
    }

    #[test]
    fn test_answer_roundtrip() {
        let step = ThoughtStep::Answer {
            content: "The answer.".to_string(),
        };
        let json = serde_json::to_string(&step).unwrap();
        let deserialized: ThoughtStep = serde_json::from_str(&json).unwrap();
        assert!(
            matches!(deserialized, ThoughtStep::Answer { content } if content == "The answer.")
        );
    }
}
