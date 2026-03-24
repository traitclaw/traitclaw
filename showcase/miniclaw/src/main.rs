//! # Miniclaw — A mini AI assistant built with TraitClaw
//!
//! Demonstrates progressive enhancement: start with a basic agent,
//! then add memory, tools, and steering — each in just a few lines.
//!
//! Total: ~150 lines (well under the 1000-line budget)

use std::io::Write;

use async_trait::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use traitclaw::prelude::*;
use traitclaw_memory_sqlite::SqliteMemory;
use traitclaw_openai_compat::OpenAiCompatProvider;
use traitclaw_steering::Steering;

// ═══════════════════════════════════════════════════════════
// Step 3: Tools (+30 lines)
// ═══════════════════════════════════════════════════════════

#[derive(Deserialize, JsonSchema)]
struct ReadFileInput {
    /// Path to the file to read
    path: String,
}

#[derive(Serialize)]
struct ReadFileOutput {
    content: String,
}

struct ReadFileTool;

#[async_trait]
impl Tool for ReadFileTool {
    type Input = ReadFileInput;
    type Output = ReadFileOutput;

    fn name(&self) -> &str {
        "read_file"
    }

    fn description(&self) -> &str {
        "Read the contents of a file from disk"
    }

    async fn execute(&self, input: Self::Input) -> traitclaw::Result<Self::Output> {
        let content = std::fs::read_to_string(&input.path)
            .unwrap_or_else(|e| format!("Error reading {}: {e}", input.path));
        Ok(ReadFileOutput { content })
    }
}

#[derive(Deserialize, JsonSchema)]
struct SearchInput {
    /// Search query
    query: String,
}

#[derive(Serialize)]
struct SearchOutput {
    results: Vec<String>,
}

struct WebSearchTool;

#[async_trait]
impl Tool for WebSearchTool {
    type Input = SearchInput;
    type Output = SearchOutput;

    fn name(&self) -> &str {
        "web_search"
    }

    fn description(&self) -> &str {
        "Search the web for information"
    }

    async fn execute(&self, input: Self::Input) -> traitclaw::Result<Self::Output> {
        Ok(SearchOutput {
            results: vec![
                format!("Result 1 for: {}", input.query),
                format!("Result 2 for: {}", input.query),
            ],
        })
    }
}

// ═══════════════════════════════════════════════════════════
// Main — Progressive Enhancement
// ═══════════════════════════════════════════════════════════

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🤖 Miniclaw — Mini AI Assistant\n");

    // ── Step 1: Basic Agent (~10 lines) ──────────────────
    let provider = OpenAiCompatProvider::openai(
        "gpt-4o-mini",
        std::env::var("OPENAI_API_KEY").unwrap_or_else(|_| "demo-key".into()),
    );

    // ── Step 2: + Memory (+5 lines) ──────────────────────
    let memory = SqliteMemory::new("miniclaw.db")?;
    let session_id = memory.create_session().await?;
    println!("📝 Session: {session_id}");

    // ── Step 4: + Steering (+3 lines) ────────────────────
    let _steering = Steering::auto();

    // ── Build the agent ──────────────────────────────────
    let agent = Agent::builder()
        .provider(provider)
        .system(
            "You are Miniclaw, a helpful AI assistant built with TraitClaw. \
             You have access to tools for reading files and searching the web. \
             Be concise and helpful.",
        )
        .tool(ReadFileTool)
        .tool(WebSearchTool)
        .build()?;

    // ── Interactive REPL ─────────────────────────────────
    println!("💬 Type your message (or 'quit' to exit):\n");

    loop {
        print!("you> ");
        std::io::stdout().flush()?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() {
            continue;
        }
        if input == "quit" || input == "exit" {
            break;
        }

        // Save user message to memory
        memory.append(&session_id, Message::user(input)).await?;

        // Run the agent
        let output = agent.run(input).await?;
        let text = output.text().to_string();

        // Save assistant response to memory
        memory
            .append(&session_id, Message::assistant(&text))
            .await?;

        println!("🤖 {text}\n");
    }

    // Show conversation history
    let history = memory.messages(&session_id).await?;
    println!("\n📜 Conversation history ({} messages):", history.len());
    for msg in &history {
        let role = match msg.role {
            MessageRole::User => "you",
            MessageRole::Assistant => "bot",
            _ => "sys",
        };
        println!(
            "  [{role}] {}",
            msg.content.chars().take(80).collect::<String>()
        );
    }

    Ok(())
}
