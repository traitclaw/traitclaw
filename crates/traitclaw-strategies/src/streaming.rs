//! Streaming support for reasoning strategies.
//!
//! This module provides streaming event types and adapters that extend
//! the core [`StreamEvent`](traitclaw_core::types::stream::StreamEvent) with
//! strategy-specific events like thought steps.

use serde::{Deserialize, Serialize};

use crate::common::ThoughtStep;

/// Strategy-specific streaming events.
///
/// These events wrap the core `StreamEvent` and add reasoning-specific
/// information like thought steps.
///
/// # Example
///
/// ```
/// use traitclaw_strategies::streaming::StrategyEvent;
/// use traitclaw_strategies::ThoughtStep;
///
/// let event = StrategyEvent::Thought(ThoughtStep::Think {
///     content: "Analyzing the problem...".to_string(),
/// });
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum StrategyEvent {
    /// A reasoning step was produced.
    Thought(ThoughtStep),
    /// A text delta from the LLM during reasoning.
    TextDelta(String),
    /// A tool call was initiated by the strategy.
    ToolCall {
        /// Name of the tool being called.
        tool_name: String,
        /// Arguments as JSON.
        arguments: serde_json::Value,
    },
    /// Tool result was received.
    ToolResult {
        /// Name of the tool that produced the result.
        tool_name: String,
        /// The result text.
        output: String,
    },
    /// The strategy has completed reasoning.
    Done {
        /// Total iterations used.
        iterations: usize,
        /// Total tokens consumed.
        total_tokens: usize,
    },
}

impl StrategyEvent {
    /// Returns `true` if this is a `Done` event.
    #[must_use]
    pub fn is_done(&self) -> bool {
        matches!(self, Self::Done { .. })
    }

    /// Returns `true` if this is a `Thought` event.
    #[must_use]
    pub fn is_thought(&self) -> bool {
        matches!(self, Self::Thought(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strategy_event_serialization() {
        let event = StrategyEvent::Thought(ThoughtStep::Think {
            content: "Analyzing...".into(),
        });
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("Think"));
        assert!(json.contains("Analyzing"));
    }

    #[test]
    fn test_done_event() {
        let event = StrategyEvent::Done {
            iterations: 5,
            total_tokens: 1500,
        };
        assert!(event.is_done());
        assert!(!event.is_thought());
    }

    #[test]
    fn test_tool_call_event() {
        let event = StrategyEvent::ToolCall {
            tool_name: "search".into(),
            arguments: serde_json::json!({"query": "rust"}),
        };
        assert!(!event.is_done());
    }

    #[test]
    fn test_text_delta_event() {
        let event = StrategyEvent::TextDelta("Hello".into());
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("Hello"));
    }

    #[test]
    fn test_tool_result_event() {
        let event = StrategyEvent::ToolResult {
            tool_name: "calc".into(),
            output: "42".into(),
        };
        assert!(!event.is_done());
        assert!(!event.is_thought());
    }

    // ── F4: Serde Round-Trip Tests ──────────────────────────────────────

    #[test]
    fn test_thought_event_roundtrip() {
        let event = StrategyEvent::Thought(ThoughtStep::Think {
            content: "Testing...".into(),
        });
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: StrategyEvent = serde_json::from_str(&json).unwrap();
        assert!(deserialized.is_thought());
    }

    #[test]
    fn test_done_event_roundtrip() {
        let event = StrategyEvent::Done {
            iterations: 5,
            total_tokens: 1500,
        };
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: StrategyEvent = serde_json::from_str(&json).unwrap();
        assert!(deserialized.is_done());
    }

    #[test]
    fn test_tool_call_event_roundtrip() {
        let event = StrategyEvent::ToolCall {
            tool_name: "search".into(),
            arguments: serde_json::json!({"query": "rust"}),
        };
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: StrategyEvent = serde_json::from_str(&json).unwrap();
        assert!(!deserialized.is_done());
        assert!(!deserialized.is_thought());
    }

    #[test]
    fn test_tool_result_event_roundtrip() {
        let event = StrategyEvent::ToolResult {
            tool_name: "calc".into(),
            output: "42".into(),
        };
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: StrategyEvent = serde_json::from_str(&json).unwrap();
        assert!(!deserialized.is_done());
    }

    #[test]
    fn test_text_delta_event_roundtrip() {
        let event = StrategyEvent::TextDelta("Hello world".into());
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: StrategyEvent = serde_json::from_str(&json).unwrap();
        assert!(!deserialized.is_done());
    }

    // ── F12: is_thought for Thought(Answer) ─────────────────────────────

    #[test]
    fn test_thought_answer_is_thought() {
        let event = StrategyEvent::Thought(ThoughtStep::Answer {
            content: "42".into(),
        });
        assert!(event.is_thought());
        assert!(!event.is_done());
    }
}
