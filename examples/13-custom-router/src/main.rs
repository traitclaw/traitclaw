//! Example: Custom Router
//!
//! Demonstrates the Router trait for multi-agent orchestration.
//! Shows SequentialRouter, LeaderRouter, and a custom PriorityRouter.

use traitclaw_team::router::*;
use traitclaw_team::{AgentRole, Team};

/// A custom router that routes based on keyword matching.
///
/// Messages containing "code" go to "coder", "write" to "writer",
/// otherwise to "leader".
struct KeywordRouter;

impl Router for KeywordRouter {
    fn route(&self, message: &TeamMessage, state: &TeamState) -> RoutingDecision {
        // After all agents have responded, complete
        if state.iteration > state.agents.len() {
            let output = state
                .message_history
                .last()
                .map_or_else(String::new, |m| m.content.clone());
            return RoutingDecision::Complete(output);
        }

        let content = message.content.to_lowercase();
        if content.contains("code") || content.contains("implement") {
            RoutingDecision::SendTo("coder".into())
        } else if content.contains("write") || content.contains("document") {
            RoutingDecision::SendTo("writer".into())
        } else {
            RoutingDecision::SendTo("leader".into())
        }
    }
}

fn main() {
    println!("=== Custom Router Example ===\n");

    // --- 1. Sequential Router ---
    println!("1️⃣  Sequential Router");
    let team = Team::new("pipeline_team")
        .add_role(AgentRole::new("researcher", "Research topics"))
        .add_role(AgentRole::new("writer", "Write content"))
        .add_role(AgentRole::new("editor", "Edit and polish"));

    let mut state = TeamState::new(vec![
        "researcher".into(),
        "writer".into(),
        "editor".into(),
    ]);

    let msg = TeamMessage::new("user", "Write an article about Rust");
    for i in 0..4 {
        let decision = team.router().route(&msg, &state);
        println!("  Step {i}: {decision:?}");
        state.current_index = i + 1;
        if matches!(decision, RoutingDecision::Complete(_)) {
            break;
        }
    }

    // --- 2. Leader Router ---
    println!("\n2️⃣  Leader Router");
    let team = Team::new("led_team")
        .add_role(AgentRole::new("leader", "Orchestrate the team"))
        .add_role(AgentRole::new("coder", "Write code"))
        .add_role(AgentRole::new("tester", "Write tests"))
        .with_router(LeaderRouter::new("leader"));

    let state = TeamState::new(vec!["leader".into(), "coder".into(), "tester".into()]);

    // User → leader
    let msg = TeamMessage::new("user", "Build a calculator");
    println!("  User → {:?}", team.router().route(&msg, &state));

    // Leader delegates to coder
    let msg = TeamMessage::new("leader", "@coder: Implement add/subtract functions");
    println!("  Leader → {:?}", team.router().route(&msg, &state));

    // Leader finalizes
    let msg = TeamMessage::new("leader", "Calculator is complete with all features.");
    println!("  Leader → {:?}", team.router().route(&msg, &state));

    // --- 3. Custom Keyword Router ---
    println!("\n3️⃣  Keyword Router (custom)");
    let team = Team::new("keyword_team")
        .add_role(AgentRole::new("leader", "General coordination"))
        .add_role(AgentRole::new("coder", "Implementation"))
        .add_role(AgentRole::new("writer", "Documentation"))
        .with_router(KeywordRouter);

    let state = TeamState::new(vec!["leader".into(), "coder".into(), "writer".into()]);

    let cases = [
        ("user", "Please implement the login feature"),
        ("user", "Write the API documentation"),
        ("user", "What should we do next?"),
    ];

    for (sender, content) in &cases {
        let msg = TeamMessage::new(*sender, *content);
        let decision = team.router().route(&msg, &state);
        println!("  \"{content}\" → {decision:?}");
    }
}
