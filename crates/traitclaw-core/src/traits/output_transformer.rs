//! Async tool output transformation.
//!
//! [`OutputTransformer`] provides context-aware, async tool output processing.
//! It adds context-awareness (tool name and agent state) and supports async operations
//! such as LLM-powered summarization of tool output.
//!
//! # Example
//!
//! ```rust
//! use traitclaw_core::traits::output_transformer::OutputTransformer;
//! use traitclaw_core::types::agent_state::AgentState;
//! use async_trait::async_trait;
//!
//! struct BudgetAwareTransformer {
//!     max_chars: usize,
//! }
//!
//! #[async_trait]
//! impl OutputTransformer for BudgetAwareTransformer {
//!     async fn transform(
//!         &self,
//!         output: String,
//!         tool_name: &str,
//!         state: &AgentState,
//!     ) -> String {
//!         // Truncate more aggressively when context is nearly full
//!         let budget = if state.context_utilization() > 0.8 {
//!             self.max_chars / 2
//!         } else {
//!             self.max_chars
//!         };
//!         if output.len() > budget {
//!             format!("{}...\n[truncated]", &output[..budget])
//!         } else {
//!             output
//!         }
//!     }
//! }
//! ```

use async_trait::async_trait;

use crate::types::agent_state::AgentState;

/// Async trait for context-aware tool output transformation.
///
/// Called after each tool execution to process the output before adding it
/// to the message context. Supports async operations such as LLM-powered
/// summarization.
#[async_trait]
pub trait OutputTransformer: Send + Sync {
    /// Transform tool output, optionally using context about which tool
    /// produced it and the current agent state.
    ///
    /// `tool_name` identifies the tool that produced the output.
    /// `state` provides runtime context (token usage, iteration count, etc.).
    async fn transform(&self, output: String, tool_name: &str, state: &AgentState) -> String;

    /// Estimate token count for a given output string.
    ///
    /// Default: 4-characters ≈ 1-token approximation.
    fn estimate_output_tokens(&self, output: &str) -> usize {
        output.len() / 4 + 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::model_info::ModelTier;
    use std::sync::Arc;

    // ── Object safety: confirm Arc<dyn OutputTransformer> compiles ───────
    #[test]
    fn test_output_transformer_is_object_safe() {
        struct Dummy;

        #[async_trait]
        impl OutputTransformer for Dummy {
            async fn transform(
                &self,
                output: String,
                _tool_name: &str,
                _state: &AgentState,
            ) -> String {
                output
            }
        }

        let _: Arc<dyn OutputTransformer> = Arc::new(Dummy);
    }

    // ── Default estimate_output_tokens() ────────────────────────────────
    #[test]
    fn test_default_estimate_output_tokens() {
        struct Dummy;

        #[async_trait]
        impl OutputTransformer for Dummy {
            async fn transform(
                &self,
                output: String,
                _tool_name: &str,
                _state: &AgentState,
            ) -> String {
                output
            }
        }

        let t = Dummy;
        // 400 chars → 400/4 + 1 = 101 tokens
        assert_eq!(t.estimate_output_tokens(&"a".repeat(400)), 101);
        // empty → 0/4 + 1 = 1 token
        assert_eq!(t.estimate_output_tokens(""), 1);
    }

    // ── Context-aware transformer test ──────────────────────────────────
    #[tokio::test]
    async fn test_context_aware_transformer() {
        struct ToolAwareTransformer;

        #[async_trait]
        impl OutputTransformer for ToolAwareTransformer {
            async fn transform(
                &self,
                output: String,
                tool_name: &str,
                state: &AgentState,
            ) -> String {
                format!(
                    "[tool={}, util={:.0}%] {}",
                    tool_name,
                    state.context_utilization() * 100.0,
                    output
                )
            }
        }

        let t = ToolAwareTransformer;
        let mut state = AgentState::new(ModelTier::Medium, 1000);
        state.total_context_tokens = 750;

        let result = t.transform("data".to_string(), "search", &state).await;
        assert!(result.contains("tool=search"), "should include tool name");
        assert!(result.contains("75%"), "should include utilization");
        assert!(result.contains("data"), "should include original output");
    }
}
