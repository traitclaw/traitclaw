//! # Multi-Agent — Team composition and verification chains
//!
//! Demonstrates using `Team`, `AgentRole`, and `VerificationChain`
//! to compose multiple agents with specialized roles, and set up
//! generate-then-verify workflows.

use baseclaw_team::{AgentRole, Team, VerificationChain, VerifyResult};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("👥 Multi-Agent Team Demo\n");

    // ── 1. Define a content pipeline team ───────────────────
    let team = Team::new("content_pipeline")
        .add_role(
            AgentRole::new("researcher", "Research topics in depth using web search")
                .with_system_prompt("You are a thorough researcher. Find facts, cite sources."),
        )
        .add_role(
            AgentRole::new("writer", "Write clear, engaging summaries from research")
                .with_system_prompt("You are a skilled writer. Create concise, readable content."),
        )
        .add_role(
            AgentRole::new("reviewer", "Review content for accuracy and quality")
                .with_system_prompt("You are a strict editor. Check facts and improve clarity."),
        );

    println!("📋 Team: {}", team.name());
    println!("   Roles: {}\n", team.roles().len());

    for role in team.roles() {
        println!("  🎭 {} — {}", role.name, role.description);
        if let Some(prompt) = &role.system_prompt {
            println!("     System: {}", &prompt[..prompt.len().min(60)]);
        }
    }

    // ── 2. Role routing ─────────────────────────────────────
    println!("\n🔍 Role Lookup:");
    if let Some(researcher) = team.find_role("researcher") {
        println!("  Found: {} — {}", researcher.name, researcher.description);
    }
    if team.find_role("designer").is_none() {
        println!("  Not found: designer (not part of this team)");
    }

    // ── 3. Verification chain ───────────────────────────────
    println!("\n🔁 Verification Chain:");
    let chain = VerificationChain::new().with_max_retries(3);
    println!("  Max retries: {}", chain.max_retries);

    // Simulate a verification workflow
    let results = vec![
        VerifyResult::Rejected("Grammar issues found in paragraph 2".into()),
        VerifyResult::Rejected("Missing source citation for claim about performance".into()),
        VerifyResult::Accepted("Content meets all quality standards".into()),
    ];

    for (attempt, result) in results.iter().enumerate() {
        let icon = if result.is_accepted() { "✅" } else { "🔄" };
        match result {
            VerifyResult::Accepted(msg) => println!("  {icon} Attempt {}: PASS — {msg}", attempt + 1),
            VerifyResult::Rejected(msg) => println!("  {icon} Attempt {}: RETRY — {msg}", attempt + 1),
        }
    }

    // ── 4. Example: build agents per role ───────────────────
    println!("\n🏗️ Building agents per role:");
    println!("  (In a real app, each role maps to an Agent with its own provider)\n");

    for role in team.roles() {
        println!("  let {}_agent = Agent::builder()", role.name);
        println!("      .provider(openai(\"gpt-4o-mini\"))");
        if let Some(prompt) = &role.system_prompt {
            println!("      .system(\"{}\")", &prompt[..prompt.len().min(40)]);
        }
        println!("      .build()?;\n");
    }

    println!("✅ Done!");
    Ok(())
}
