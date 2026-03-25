//! # Context Managers — v0.3.0 Smart Context Survival
//!
//! Demonstrates the built-in `ContextManager` implementations that
//! automatically compress conversation history when it approaches
//! the model's context window limit.
//!
//! Shows:
//! - `RuleBasedCompressor` — importance-scored message pruning
//! - `TieredCompressor`    — chained rule-based + optional LLM summarization
//!
//! Run: `cargo run -p context-managers`
//!
//! Note: This example uses `OpenAiCompatProvider`. Set `OPENAI_API_KEY`
//! or it will use a demo key (API calls will fail but structure is shown).

use traitclaw::prelude::*;
use traitclaw::{RuleBasedCompressor, TieredCompressor};
use traitclaw_openai_compat::OpenAiCompatProvider;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let api_key = std::env::var("OPENAI_API_KEY").unwrap_or_else(|_| "demo-key".into());

    // ── Example 1: RuleBasedCompressor ───────────────────────────────────
    println!("╔══════════════════════════════════════════════════╗");
    println!("║  Example 1: RuleBasedCompressor                 ║");
    println!("╚══════════════════════════════════════════════════╝\n");

    let agent = Agent::builder()
        .model(OpenAiCompatProvider::openai("gpt-4o-mini", &api_key))
        .system("You are a helpful assistant. Keep responses brief (1-2 sentences).")
        // Threshold 0.85 = compress when messages exceed 85% of context window
        // Protect the last 2 messages from removal
        .context_manager(RuleBasedCompressor::new(0.85, 2))
        .build()?;

    println!("Agent built with RuleBasedCompressor (threshold=0.85, recent=2).");
    println!("The compressor will automatically prune old messages when");
    println!("conversation history approaches 85% of the context window.\n");

    let output = agent.run("What is Rust?").await?;
    println!("Agent: {}\n", output.text());

    // ── Example 2: TieredCompressor ─────────────────────────────────────
    println!("╔══════════════════════════════════════════════════╗");
    println!("║  Example 2: TieredCompressor (rule-only)        ║");
    println!("╚══════════════════════════════════════════════════╝\n");

    let agent2 = Agent::builder()
        .model(OpenAiCompatProvider::openai("gpt-4o-mini", &api_key))
        .system("You are a coding tutor. Keep responses brief.")
        // TieredCompressor: keep last 3 messages, rule-compress the rest
        .context_manager(TieredCompressor::new(3))
        .build()?;

    println!("Agent built with TieredCompressor (keep_recent=3).");
    println!("Chains: LLM summarization (if provider given) → rule-based compression.\n");

    let session = agent2.session("tutorial");
    let out = session.say("What are Rust traits?").await?;
    println!("Agent: {}\n", out.text());

    let out = session.say("How do they compare to interfaces?").await?;
    println!("Agent: {}\n", out.text());

    // ── Summary ─────────────────────────────────────────────────────────
    println!("═══════════════════════════════════════════════════");
    println!("  Available context managers:");
    println!("    • RuleBasedCompressor — importance-scored pruning");
    println!("    • LlmCompressor       — LLM-powered summarization");
    println!("    • TieredCompressor     — chained compression tiers");
    println!();
    println!("  Usage:");
    println!("    Agent::builder()");
    println!("        .model(provider)");
    println!("        .context_manager(RuleBasedCompressor::new(0.85, 3))");
    println!("        .build()");
    println!("═══════════════════════════════════════════════════");

    Ok(())
}
