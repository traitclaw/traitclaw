//! # Context Manager — v0.3.0 Async Context Management
//!
//! Demonstrates implementing a custom `ContextManager` that logs
//! context utilization before each LLM call and conditionally
//! trims old messages when approaching the window limit.

use async_trait::async_trait;
use traitclaw::prelude::*;
use traitclaw::types::agent_state::AgentState;
use traitclaw::types::message::Message;
use traitclaw_openai_compat::OpenAiCompatProvider;

/// A context manager that logs utilization and aggressively trims
/// when context usage exceeds 80%.
struct LoggingContextManager;

#[async_trait]
impl ContextManager for LoggingContextManager {
    async fn prepare(
        &self,
        messages: &mut Vec<Message>,
        context_window: usize,
        state: &mut AgentState,
    ) {
        let utilization = state.context_utilization();
        println!(
            "  [ContextManager] utilization: {:.0}% ({}/{} tokens, {} messages)",
            utilization * 100.0,
            state.total_context_tokens,
            context_window,
            messages.len(),
        );

        // When approaching the limit, keep only system + last 4 messages
        if utilization > 0.8 && messages.len() > 5 {
            println!("  [ContextManager] ⚠ Trimming context (>{:.0}% used)", 80.0);
            let system = messages.first().cloned();
            let tail: Vec<_> = messages.iter().rev().take(4).cloned().collect();
            messages.clear();
            if let Some(sys) = system {
                messages.push(sys);
            }
            messages.extend(tail.into_iter().rev());
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let provider = OpenAiCompatProvider::openai(
        "gpt-4o-mini",
        std::env::var("OPENAI_API_KEY").unwrap_or_else(|_| "demo-key".into()),
    );

    let agent = Agent::builder()
        .model(provider)
        .system("You are a friendly assistant. Keep responses brief.")
        .context_manager(LoggingContextManager)
        .build()?;

    println!("--- Using custom ContextManager ---\n");

    let output = agent.run("What is Rust?").await?;
    println!("Agent: {}\n", output.text());

    let session = agent.session("demo");
    let out = session.say("Tell me about ownership.").await?;
    println!("Agent: {}\n", out.text());

    let out = session.say("And borrowing?").await?;
    println!("Agent: {}", out.text());

    Ok(())
}
