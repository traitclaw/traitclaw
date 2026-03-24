//! # MCP Client — Connect to MCP tool servers
//!
//! Demonstrates connecting to Model Context Protocol (MCP) servers
//! over stdio transport, discovering available tools, and using them
//! with a BaseClaw agent. MCP tools implement `ErasedTool` so they
//! work seamlessly alongside native tools.
//!
//! ## Prerequisites
//!
//! Install an MCP server to connect to, e.g.:
//! ```bash
//! npm install -g @modelcontextprotocol/server-filesystem
//! ```

use baseclaw::prelude::*;
use baseclaw_mcp::McpServer;
use baseclaw_openai_compat::OpenAiCompatProvider;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🔌 MCP Client Demo\n");

    // ── 1. Connect to an MCP server ─────────────────────────
    // This connects to a filesystem MCP server via stdio transport.
    // The server provides tools like read_file, write_file, list_directory, etc.
    println!("📡 Connecting to MCP filesystem server...\n");

    let server = match McpServer::stdio(
        "npx",
        &["-y", "@modelcontextprotocol/server-filesystem", "/tmp"],
    )
    .await
    {
        Ok(server) => {
            println!("✅ Connected!\n");
            server
        }
        Err(e) => {
            println!("⚠️  Could not connect to MCP server: {e}");
            println!("    Make sure npx is installed (comes with Node.js)");
            println!("    The server will be auto-installed on first run.\n");
            println!("──── Showing API usage without a live server ────\n");

            // Show what the code looks like even without a server
            println!("    let server = McpServer::stdio(\"npx\", &[args]).await?;");
            println!("    let tools = server.tools();  // Vec<Arc<dyn ErasedTool>>");
            println!("    let agent = Agent::builder()");
            println!("        .provider(provider)");
            println!("        .tools(tools)  // MCP tools work like native tools!");
            println!("        .build()?;\n");
            return Ok(());
        }
    };

    // ── 2. Discover available tools ─────────────────────────
    let tools = server.tools();
    println!("🔧 Discovered {} MCP tools:", tools.len());
    for tool in tools {
        println!("  • {} — {}", tool.name(), tool.description());
    }

    // ── 3. Use MCP tools with an agent ──────────────────────
    println!("\n🤖 Creating agent with MCP tools...\n");

    let provider = OpenAiCompatProvider::openai(
        "gpt-4o-mini",
        std::env::var("OPENAI_API_KEY").unwrap_or_else(|_| "demo-key".into()),
    );

    let mut builder = Agent::builder()
        .provider(provider)
        .system("You are a helpful assistant with filesystem access via MCP tools.");

    // Register all discovered MCP tools
    for tool in tools {
        builder = builder.tool_arc(tool.clone());
    }

    let agent = builder.build()?;

    let output = agent.run("List the files in /tmp").await?;
    println!("Response: {}", output.text());

    println!("\n✅ Done!");
    Ok(())
}
