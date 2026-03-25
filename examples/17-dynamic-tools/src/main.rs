//! # Dynamic Tools — v0.3.0 Runtime Tool Management
//!
//! Demonstrates using `DynamicRegistry` to add, remove, and toggle
//! tools at runtime without rebuilding the agent.

use std::sync::Arc;

use async_trait::async_trait;
use traitclaw::prelude::*;
use traitclaw::DynamicRegistry;
use traitclaw_openai_compat::OpenAiCompatProvider;

// ── Example tools ──

struct SearchTool;

#[async_trait]
impl ErasedTool for SearchTool {
    fn name(&self) -> &str {
        "search"
    }
    fn description(&self) -> &str {
        "Search the web"
    }
    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: "search".into(),
            description: "Search the web for information".into(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": { "query": { "type": "string" } }
            }),
        }
    }
    async fn execute_json(&self, input: serde_json::Value) -> traitclaw::Result<serde_json::Value> {
        let query = input["query"].as_str().unwrap_or("?");
        Ok(serde_json::json!(format!("Search results for: {query}")))
    }
}

struct CalcTool;

#[async_trait]
impl ErasedTool for CalcTool {
    fn name(&self) -> &str {
        "calc"
    }
    fn description(&self) -> &str {
        "Simple calculator"
    }
    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: "calc".into(),
            description: "Evaluate math expressions".into(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": { "expression": { "type": "string" } }
            }),
        }
    }
    async fn execute_json(&self, input: serde_json::Value) -> traitclaw::Result<serde_json::Value> {
        let expr = input["expression"].as_str().unwrap_or("0");
        Ok(serde_json::json!(format!("Result of {expr} = 42")))
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let provider = OpenAiCompatProvider::openai(
        "gpt-4o-mini",
        std::env::var("OPENAI_API_KEY").unwrap_or_else(|_| "demo-key".into()),
    );

    // Create a dynamic registry wrapped in Arc for shared ownership
    let registry = Arc::new(DynamicRegistry::new());
    registry.register(Arc::new(SearchTool));
    registry.register(Arc::new(CalcTool));

    println!("--- Initial tools: {} ---", registry.len());
    for tool in registry.get_tools() {
        println!("  ✓ {} — {}", tool.name(), tool.description());
    }

    // Build agent with dynamic registry
    // Note: we pass a SimpleRegistry here since ToolRegistry requires ownership.
    // The DynamicRegistry is used for runtime management separately.
    let agent = Agent::builder()
        .model(provider)
        .system("You are a helpful assistant with tools.")
        .tool(SearchTool)
        .tool(CalcTool)
        .build()?;

    let output = agent.run("Hello!").await?;
    println!("\nAgent: {}\n", output.text());

    // Disable search tool at runtime
    registry.set_enabled("search", false);
    println!("--- After disabling search: {} tools ---", registry.len());
    for tool in registry.get_tools() {
        println!("  ✓ {} — {}", tool.name(), tool.description());
    }

    // Re-enable and unregister
    registry.set_enabled("search", true);
    registry.unregister("calc");
    println!(
        "\n--- After re-enable search + remove calc: {} tools ---",
        registry.len()
    );
    for tool in registry.get_tools() {
        println!("  ✓ {} — {}", tool.name(), tool.description());
    }

    Ok(())
}
