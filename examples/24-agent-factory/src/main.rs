//! # Example 24: Agent Factory — v0.6.0 Composition APIs
//!
//! Demonstrates all four v0.6.0 composition APIs progressively:
//!
//! 1. **`Agent::with_system()`** — single-line agent creation
//! 2. **`AgentFactory::spawn()`** — multi-agent from shared provider
//! 3. **`AgentPool::run_sequential()`** — pipeline execution
//! 4. **`RoundRobinGroupChat::run()`** — multi-turn collaboration
//!
//! This example runs fully offline using mock closures — no API key needed.
//!
//! # Running
//!
//! ```sh
//! cargo run -p agent-factory
//! ```

use traitclaw::prelude::*;
use traitclaw_core::{AgentFactory, AgentPool};
use traitclaw_team::{group_chat::RoundRobinGroupChat, pool_from_team, AgentRole, Team};

// ─── Mock Provider (no real API key needed) ───────────────────────────────────

use async_trait::async_trait;
use std::sync::Arc;
use traitclaw_core::{
    traits::provider::Provider,
    types::{
        completion::{CompletionRequest, CompletionResponse, ResponseContent, Usage},
        model_info::{ModelInfo, ModelTier},
        stream::CompletionStream,
    },
};

/// A mock provider that returns the agent's system prompt with the user input echoed back.
struct MockProvider {
    info: ModelInfo,
    role_label: String,
}

impl MockProvider {
    fn new(role_label: &str) -> Self {
        Self {
            info: ModelInfo::new("mock-gpt", ModelTier::Medium, 128_000, true, false, false),
            role_label: role_label.to_string(),
        }
    }
}

#[async_trait]
impl Provider for MockProvider {
    async fn complete(&self, req: CompletionRequest) -> traitclaw_core::Result<CompletionResponse> {
        let last_user = req
            .messages
            .iter()
            .rev()
            .find(|m| m.role == traitclaw_core::types::message::MessageRole::User)
            .map(|m| m.content.clone())
            .unwrap_or_else(|| "(no input)".to_string());

        // Trim to first 120 chars for readable demo output
        let trimmed: String = last_user.chars().take(120).collect();
        let ellipsis = if last_user.len() > 120 { "…" } else { "" };

        Ok(CompletionResponse {
            content: ResponseContent::Text(format!(
                "[{}] → {}{}",
                self.role_label, trimmed, ellipsis
            )),
            usage: Usage {
                prompt_tokens: 10,
                completion_tokens: 20,
                total_tokens: 30,
            },
        })
    }

    async fn stream(&self, _req: CompletionRequest) -> traitclaw_core::Result<CompletionStream> {
        unimplemented!("streaming not used in this demo")
    }

    fn model_info(&self) -> &ModelInfo {
        &self.info
    }
}

// ─────────────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🏭  TraitClaw v0.6.0 – Composition API Demo\n");
    println!("{}\n", "═".repeat(60));

    // ─── TIER 1: Agent::with_system() ────────────────────────────────────────
    println!("📌 TIER 1: Agent::with_system() — single-line agent creation\n");

    let analyst = Agent::with_system(
        MockProvider::new("Analyst"),
        "You are a data analyst. Answer concisely.",
    );

    let output = analyst
        .run("What are the key metrics for AI agent reliability?")
        .await?;
    println!("  Agent output: {}\n", output.text());
    println!("{}\n", "─".repeat(60));

    // ─── TIER 2: AgentFactory::spawn() ───────────────────────────────────────
    println!("🏭 TIER 2: AgentFactory — spawn multiple agents from shared provider\n");

    // In production you'd use Arc<dyn Provider> (e.g., OpenAI) here.
    // For demo: each clone is a fresh MockProvider with same label.
    let factory = AgentFactory::from_arc(Arc::new(MockProvider::new("Factory")));

    let researcher = factory.spawn("You are a researcher. Find key facts.");
    let writer = factory.spawn("You are a writer. Summarize findings clearly.");
    let reviewer = factory.spawn("You are a reviewer. Validate the summary.");

    println!("  Spawned {} agents from factory", 3);
    let r_out = researcher.run("Investigate async Rust patterns").await?;
    println!("  Researcher: {}", r_out.text());
    let w_out = writer.run("Produce a clear summary of key facts").await?;
    println!("  Writer:     {}", w_out.text());
    let rev_out = reviewer.run("Validate: is the summary complete?").await?;
    println!("  Reviewer:   {}", rev_out.text());
    println!("{}\n", "─".repeat(60));

    // ─── TIER 3: AgentPool::run_sequential() ─────────────────────────────────
    println!("🔗 TIER 3: AgentPool — sequential pipeline (output chaining)\n");

    let pool = AgentPool::new(vec![
        Agent::with_system(
            MockProvider::new("Researcher"),
            "You research topics in depth.",
        ),
        Agent::with_system(
            MockProvider::new("Writer"),
            "You write clear, concise summaries.",
        ),
        Agent::with_system(
            MockProvider::new("Editor"),
            "You polish and finalize content.",
        ),
    ]);

    let pipeline_output = pool
        .run_sequential("Rust's ownership model and why it prevents data races")
        .await?;
    println!(
        "  Pipeline result ({} agents):\n  {}\n",
        pool.len(),
        pipeline_output.text()
    );

    // ─── TIER 3b: pool_from_team() ────────────────────────────────────────────
    println!("🔗 TIER 3b: pool_from_team() — Team → AgentPool bridge\n");

    let team = Team::new("content_team")
        .add_role(
            AgentRole::new("researcher", "Research topics")
                .with_system_prompt("You research topics thoroughly."),
        )
        .add_role(
            AgentRole::new("writer", "Write summaries")
                .with_system_prompt("You write clear technical summaries."),
        );

    let team_pool = pool_from_team(&team, MockProvider::new("TeamAgent"))?;
    println!(
        "  Team '{}' → pool with {} agents",
        team.name(),
        team_pool.len()
    );
    let team_output = team_pool
        .run_sequential("Explain zero-cost abstractions")
        .await?;
    println!("  Output: {}\n", team_output.text());
    println!("{}\n", "─".repeat(60));

    // ─── TIER 4: RoundRobinGroupChat ─────────────────────────────────────────
    println!("💬 TIER 4: RoundRobinGroupChat — multi-turn collaborative discussion\n");

    let chat = RoundRobinGroupChat::new(vec![
        Agent::with_system(
            MockProvider::new("Alice"),
            "You are Alice, a systems programmer.",
        ),
        Agent::with_system(
            MockProvider::new("Bob"),
            "You are Bob, a distributed systems expert.",
        ),
    ])
    .with_max_rounds(4);

    let chat_result = chat
        .run("Discuss: is async Rust ready for production?")
        .await?;

    println!(
        "  Chat ran for {} rounds ({} messages in transcript):",
        chat_result.transcript.len() - 1,
        chat_result.transcript.len()
    );
    for (i, msg) in chat_result.transcript.iter().enumerate() {
        let role = format!("{:?}", msg.role);
        let preview: String = msg.content.chars().take(80).collect();
        println!("    [{}] {}: {}…", i, role, preview);
    }
    println!("\n  Final message: {}", chat_result.final_message);

    println!("\n{}", "═".repeat(60));
    println!("✅  Composition API demo complete!\n");
    println!("  📚 All v0.6.0 APIs demonstrated:");
    println!("     • Agent::with_system()     → single-line creation");
    println!("     • AgentFactory::spawn()    → shared-provider factory");
    println!("     • AgentPool::run_sequential() → output chaining");
    println!("     • pool_from_team()         → Team → AgentPool bridge");
    println!("     • RoundRobinGroupChat::run() → multi-turn chat");

    Ok(())
}
