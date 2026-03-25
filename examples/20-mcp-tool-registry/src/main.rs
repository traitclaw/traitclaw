//! Example 20: McpToolRegistry — Auto-discover tools from MCP servers.
//!
//! This example shows how to connect to MCP servers, discover their tools,
//! and use them with an Agent. Set `MCP_SERVER_CMD` env var to point to a
//! real MCP server, or the example will demonstrate graceful error handling.
//!
//! Run with: `cargo run -p mcp-tool-registry-example`
//! With a real server: `MCP_SERVER_CMD="npx @modelcontextprotocol/server-filesystem" cargo run -p mcp-tool-registry-example`

use traitclaw::prelude::*;
use traitclaw_mcp::{McpToolRegistry, MultiServerMcpRegistry};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("═══════════════════════════════════════════════════════════");
    println!("  Example 20: McpToolRegistry — MCP Tool Discovery");
    println!("═══════════════════════════════════════════════════════════\n");

    // ── Part 1: Single-server McpToolRegistry ────────────────────────────────
    single_server_demo().await;

    // ── Part 2: Multi-server with prefixing ─────────────────────────────────
    multi_server_demo().await;

    println!("\n✅ Example complete!");
    Ok(())
}

async fn single_server_demo() {
    println!("─── Part 1: Single-Server McpToolRegistry ───\n");

    // Read server command from environment, or use a demo default
    let server_cmd = std::env::var("MCP_SERVER_CMD").unwrap_or_else(|_| "echo".to_string()); // 'echo' will fail gracefully
    let args: Vec<String> = server_cmd.split_whitespace().map(String::from).collect();
    let program = &args[0];
    let arg_refs: Vec<&str> = args[1..].iter().map(String::as_str).collect();

    println!("Connecting to MCP server: {server_cmd}");
    println!("(Set MCP_SERVER_CMD env var to point to a real MCP server)\n");

    match McpToolRegistry::stdio(program, &arg_refs).await {
        Ok(registry) => {
            println!("✓ Connected! Discovered {} tool(s):", registry.len());
            for tool in registry.get_tools() {
                println!("  • {}", tool.name());
                println!("    {}", tool.description());
                let schema = tool.schema();
                if !schema.parameters.is_null() {
                    println!("    Schema: {}", schema.parameters);
                }
                println!();
            }

            // Try executing the first tool (if any)
            if let Some(first) = registry.get_tools().into_iter().next() {
                println!("Executing tool '{}'...", first.name());
                match first.execute_json(serde_json::json!({})).await {
                    Ok(result) => println!("Result: {result}"),
                    Err(e) => println!("Error (expected without correct args): {e}"),
                }
            }
        }
        Err(e) => {
            println!("⚠ Could not connect to MCP server: {e}");
            println!("  This is expected when no real MCP server is configured.");
            println!("  The registry handles errors gracefully — your Agent continues working");
            println!("  with other tools that are available.");

            // Show what the API looks like even when server is unavailable
            demonstrate_api_shape();
        }
    }
}

async fn multi_server_demo() {
    println!("\n─── Part 2: MultiServerMcpRegistry (Multi-Server) ───\n");

    println!("Building multi-server registry with 2 servers...");
    println!("(Both will gracefully fail without real MCP servers)\n");

    let result = MultiServerMcpRegistry::builder()
        .with_prefix(true) // tools named "server::tool_name"
        .add_stdio("fs", "echo", &["mcp-fs-stub"]) // stub — will fail gracefully
        .add_stdio("git", "echo", &["mcp-git-stub"]) // stub — will fail gracefully
        .build()
        .await;

    match result {
        Ok(registry) => {
            println!("Healthy servers: {}", registry.healthy_server_count());
            println!("Unhealthy servers: {}", registry.unhealthy_server_count());
            println!("Total tools available: {}", registry.len());

            if registry.len() > 0 {
                println!("\nDiscovered tools (prefixed names):");
                for tool in registry.get_tools() {
                    println!("  • {}", tool.name());

                    // Show how to resolve the prefix
                    if let Some((server, bare_name)) =
                        MultiServerMcpRegistry::resolve_prefix(tool.name())
                    {
                        println!("    Server: {server}, Tool: {bare_name}");
                    }
                }
            } else {
                println!("\nNo tools available (servers unavailable — expected in demo).");
                println!("\nWith real MCP servers, the registry would expose tools like:");
                println!("  • fs::read_file");
                println!("  • fs::write_file");
                println!("  • git::create_commit");
                println!("  • git::diff");
                println!("\nPrefix format prevents name collisions when multiple servers");
                println!("expose a tool with the same base name (e.g., 'search').");
            }
        }
        Err(e) => {
            println!("Registry build error: {e}");
        }
    }
}

fn demonstrate_api_shape() {
    println!("\n─── API Reference (for when MCP server is configured) ───\n");
    println!(
        r#"
// Single server:
let registry = McpToolRegistry::stdio("npx", &["@modelcontextprotocol/server-filesystem"]).await?;
println!("Tools: {{}}", registry.len());

// Use with Agent:
let agent = Agent::builder()
    .model(provider)
    .tool_registry(Arc::new(registry))
    .build()?;

// Multi-server with name prefixing:
let registry = MultiServerMcpRegistry::builder()
    .with_prefix(true)
    .add_stdio("fs",  "npx", &["@modelcontextprotocol/server-filesystem"])
    .add_stdio("git", "uvx", &["mcp-server-git"])
    .build()
    .await?;

// Tools appear as "fs::read_file", "git::create_commit", etc.
"#
    );
}
