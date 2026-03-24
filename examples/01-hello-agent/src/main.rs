//! # Hello Agent — The simplest TraitClaw agent
//!
//! This example demonstrates creating an AI agent in ~5 lines of code.
//! It showcases the full pipeline: Builder → Provider → LLM call → Response.

use traitclaw::prelude::*;
use traitclaw_openai_compat::OpenAiCompatProvider;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Create a provider for OpenAI
    let provider = OpenAiCompatProvider::openai(
        "gpt-4o-mini",
        std::env::var("OPENAI_API_KEY").unwrap_or_else(|_| "demo-key".into()),
    );

    // 2. Build an agent with a system prompt
    let agent = Agent::builder()
        .provider(provider)
        .system("You are a friendly assistant. Keep responses brief.")
        .build()?;

    // 3. Run a single turn and print the response
    let output = agent.run("Hello! What is TraitClaw?").await?;
    println!("{}", output.text());

    Ok(())
}
