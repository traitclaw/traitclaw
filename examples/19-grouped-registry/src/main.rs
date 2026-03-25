//! Example 19: GroupedRegistry — Organized, switchable tool groups.
//!
//! Demonstrates how to organize tools into named groups and switch
//! between them at runtime. No real API calls needed.
//!
//! Run with: `cargo run -p grouped-registry-example`

use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;
use traitclaw::prelude::*;

// ──────────────────────────────────────────────────────────────────────────────
// Dummy tools — implements ErasedTool directly, no real API calls
// ──────────────────────────────────────────────────────────────────────────────

struct DummyTool {
    name: &'static str,
    description: &'static str,
}

#[async_trait]
impl ErasedTool for DummyTool {
    fn name(&self) -> &str {
        self.name
    }

    fn description(&self) -> &str {
        self.description
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: self.name.to_string(),
            description: self.description.to_string(),
            parameters: serde_json::json!({ "type": "object", "properties": {} }),
        }
    }

    async fn execute_json(&self, _input: Value) -> traitclaw_core::Result<Value> {
        Ok(Value::String(format!("[{}] executed", self.name)))
    }
}

fn tool(name: &'static str, description: &'static str) -> Arc<dyn ErasedTool> {
    Arc::new(DummyTool { name, description })
}

// ──────────────────────────────────────────────────────────────────────────────
// Main
// ──────────────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    println!("═══════════════════════════════════════════════════════════");
    println!("  Example 19: GroupedRegistry — Switchable Tool Groups");
    println!("═══════════════════════════════════════════════════════════\n");

    // ── Build the registry with 3 named groups (builder API) ─────────────────
    let registry = GroupedRegistry::new()
        .group(
            "search",
            vec![
                tool("web_search", "Search the web for information"),
                tool("wiki_lookup", "Look up a topic on Wikipedia"),
            ],
        )
        .group(
            "code",
            vec![
                tool("run_tests", "Run the test suite"),
                tool("lint_code", "Lint the source code"),
                tool("format_code", "Auto-format source code"),
            ],
        )
        .group(
            "data",
            vec![
                tool("read_csv", "Read a CSV file"),
                tool("write_json", "Write data as JSON"),
                tool("sql_query", "Execute a SQL query"),
            ],
        );

    println!("Registered 3 groups:");
    println!("  • search  (2 tools): web_search, wiki_lookup");
    println!("  • code    (3 tools): run_tests, lint_code, format_code");
    println!("  • data    (3 tools): read_csv, write_json, sql_query");

    // ── Step 1: no groups active yet ──────────────────────────────────────────
    println!("\n[Step 1] No groups active");
    print_active_tools(&registry);

    // ── Step 2: activate 'search' at runtime ─────────────────────────────────
    println!("\n[Step 2] Activate 'search' group");
    registry.activate_group("search");
    print_active_tools(&registry);

    // ── Step 3: switch to 'code' ──────────────────────────────────────────────
    println!("\n[Step 3] Deactivate 'search', activate 'code'");
    registry.deactivate_group("search");
    registry.activate_group("code");
    print_active_tools(&registry);

    // ── Step 4: activate 'code' + 'data' together ────────────────────────────
    println!("\n[Step 4] Also activate 'data' — now code + data are active");
    registry.activate_group("data");
    print_active_tools(&registry);

    // ── Step 5: find_tool (searches all groups, active or not) ───────────────
    println!("\n[Step 5] find_tool(\"web_search\") — 'search' group is deactivated");
    match registry.find_tool("web_search") {
        Some(t) => println!(
            "  → Found: {} (note: find_tool searches all groups)",
            t.name()
        ),
        None => println!("  → Not found"),
    }

    println!("\n[Step 6] find_tool(\"run_tests\") — 'code' group is active");
    match registry.find_tool("run_tests") {
        Some(t) => println!("  → Found: {}", t.name()),
        None => println!("  → Not found"),
    }

    // ── Step 6: group status checks ──────────────────────────────────────────
    println!("\n[Step 7] Group status:");
    println!(
        "  is_group_active(\"search\"): {}",
        registry.is_group_active("search")
    );
    println!(
        "  is_group_active(\"code\"):   {}",
        registry.is_group_active("code")
    );
    println!(
        "  is_group_active(\"data\"):   {}",
        registry.is_group_active("data")
    );

    println!("\n✅ Example complete!");
}

fn print_active_tools(registry: &GroupedRegistry) {
    let tools = registry.get_tools();
    if tools.is_empty() {
        println!("  Active tools: (none)");
    } else {
        println!("  Active tools ({}):", tools.len());
        for t in &tools {
            println!("    • {} — {}", t.name(), t.description());
        }
    }
}
