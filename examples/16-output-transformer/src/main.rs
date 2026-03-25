//! # Output Transformer — v0.3.0 Context-Aware Output Processing
//!
//! Demonstrates using built-in `OutputTransformer` implementations
//! to process tool output with context awareness.
//!
//! Shows:
//! - `BudgetAwareTruncator` — halves limit when context is full
//! - `JsonExtractor` — extracts JSON from verbose output
//! - `TransformerChain` — piping transformers in sequence

use traitclaw::prelude::*;
use traitclaw::{BudgetAwareTruncator, JsonExtractor, TransformerChain};
use traitclaw_openai_compat::OpenAiCompatProvider;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let api_key = std::env::var("OPENAI_API_KEY").unwrap_or_else(|_| "demo-key".into());

    // Example 1: Budget-aware truncation
    println!("--- BudgetAwareTruncator ---");
    let agent = Agent::builder()
        .model(OpenAiCompatProvider::openai("gpt-4o-mini", &api_key))
        .system("You are helpful.")
        .output_transformer(BudgetAwareTruncator::new(5_000, 0.8))
        .build()?;

    let output = agent.run("Hello!").await?;
    println!("Agent: {}\n", output.text());

    // Example 2: Chained transformers (extract JSON, then truncate)
    println!("--- TransformerChain (JsonExtractor → BudgetAwareTruncator) ---");
    let chain = TransformerChain::new(vec![
        Box::new(JsonExtractor),
        Box::new(BudgetAwareTruncator::new(2_000, 0.7)),
    ]);

    let agent = Agent::builder()
        .model(OpenAiCompatProvider::openai("gpt-4o-mini", &api_key))
        .system("You are helpful.")
        .output_transformer(chain)
        .build()?;

    let output = agent.run("What is 2+2?").await?;
    println!("Agent: {}", output.text());

    Ok(())
}
