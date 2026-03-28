//! Transformer tests — BudgetAwareTruncator, JsonExtractor, TransformerChain, ProgressiveTransformer.
//!
//! Migrated from `src/transformers.rs` inline tests to use shared test utilities.

use std::sync::Arc;

use traitclaw_core::traits::output_transformer::OutputTransformer;
use traitclaw_core::transformers::{
    BudgetAwareTruncator, JsonExtractor, ProgressiveTransformer, TransformerChain,
};
use traitclaw_core::types::agent_state::AgentState;
use traitclaw_core::types::model_info::ModelTier;
use traitclaw_test_utils::provider::MockProvider;

fn state_with_utilization(util: f64) -> AgentState {
    let window = 1000;
    let mut state = AgentState::new(ModelTier::Medium, window);
    state.total_context_tokens = (util * window as f64) as usize;
    state
}

// ── BudgetAwareTruncator ─────────────────────────────────────────────

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

// ── JsonExtractor ────────────────────────────────────────────────────

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

// ── TransformerChain ─────────────────────────────────────────────────

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

// ── ProgressiveTransformer ───────────────────────────────────────────

#[tokio::test]
async fn test_progressive_short_output_passthrough() {
    // AC #9: short output → passed through without LLM call
    let provider = Arc::new(MockProvider::error("mock failure")); // would fail if called
    let transformer = ProgressiveTransformer::new(provider, 500);
    let state = state_with_utilization(0.0);

    let short = "short output".to_string();
    let result = transformer
        .transform(short.clone(), "my_tool", &state)
        .await;
    assert_eq!(result, short); // unchanged
}

#[tokio::test]
async fn test_progressive_large_output_summarized() {
    // AC #8: large output → summary returned + cache populated
    let provider = Arc::new(MockProvider::text("This is the summary."));
    let transformer = ProgressiveTransformer::new(provider, 50);
    let state = state_with_utilization(0.0);

    let large_output = "x".repeat(500);
    let result = transformer
        .transform(large_output.clone(), "search_tool", &state)
        .await;

    assert!(result.contains("This is the summary."));
    assert!(result.contains("__get_full_output"));
    assert!(result.contains("search_tool"));

    // Cache should contain full output
    let retriever = transformer.retriever_tool();
    assert!(retriever.has_cached("search_tool"));
    assert_eq!(retriever.retrieve("search_tool"), large_output);
}

#[tokio::test]
async fn test_progressive_llm_failure_fallback() {
    // AC #10: LLM failure → graceful truncation fallback
    let provider = Arc::new(MockProvider::error("mock failure"));
    let transformer = ProgressiveTransformer::new(provider, 20);
    let state = state_with_utilization(0.0);

    let large_output = "a".repeat(200);
    let result = transformer.transform(large_output, "tool_x", &state).await;

    // Starts with first 20 chars
    assert!(result.starts_with("aaaaaaaaaaaaaaaaaaaa"));
    assert!(result.contains("LLM summarization failed"));
}

#[tokio::test]
async fn test_full_output_retriever_not_found() {
    // AC #8: retriever returns error message when cache is empty
    let transformer = ProgressiveTransformer::new(Arc::new(MockProvider::text("x")), 50);
    let retriever = transformer.retriever_tool();
    let result = retriever.retrieve("nonexistent_tool");
    assert!(result.contains("No cached output found"));
}

#[tokio::test]
async fn test_progressive_custom_prompt() {
    let provider = Arc::new(MockProvider::text("custom summary"));
    let transformer =
        ProgressiveTransformer::new(provider, 10).with_summary_prompt("Brief: {output}");
    let state = state_with_utilization(0.0);

    let result = transformer.transform("x".repeat(100), "t", &state).await;
    assert!(result.contains("custom summary"));
}
