//! # Steering — Guards, Hints, and auto-configuration
//!
//! Demonstrates using TraitClaw's steering system to protect and guide agents
//! with Guards (block harmful content), Hints (inject context), and
//! the `Steering::auto()` one-liner for automatic configuration.

use traitclaw::prelude::*;
use traitclaw_openai_compat::OpenAiCompatProvider;
use traitclaw_steering::guards::ShellDenyGuard;
use traitclaw_steering::Steering;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let provider = OpenAiCompatProvider::openai(
        "gpt-4o-mini",
        std::env::var("OPENAI_API_KEY").unwrap_or_else(|_| "demo-key".into()),
    );

    // ── Method 1: Steering::auto() — one-liner auto-configuration ────
    println!("=== Steering::auto() ===\n");

    let steering = Steering::auto();
    println!("Auto-configured steering");

    // Use the steering components (consuming accessors)
    let guards = steering.guards();
    println!("  Guards: {}\n", guards.len());

    // ── Method 2: Manual Guard demo ──────────────────────────────────
    println!("=== Manual Guard Demo ===\n");

    let guard = ShellDenyGuard::new();

    // Test safe action — a simple shell command
    let safe_action = Action::ShellCommand {
        command: "echo hello".into(),
    };
    let safe_result = guard.check(&safe_action);
    println!("Safe action (echo hello): {safe_result:?}");

    // Test dangerous action — destructive shell command
    let dangerous_action = Action::ShellCommand {
        command: "rm -rf /".into(),
    };
    let dangerous_result = guard.check(&dangerous_action);
    println!("Dangerous action (rm -rf /): {dangerous_result:?}");

    // ── Method 3: Full agent with steering ───────────────────────────
    println!("\n=== Agent with Steering ===\n");

    let agent = Agent::builder()
        .provider(provider)
        .system("You are a helpful coding assistant.")
        .build()?;

    let output = agent.run("What is a good Rust HTTP library?").await?;
    println!("Response: {}", output.text());

    Ok(())
}
