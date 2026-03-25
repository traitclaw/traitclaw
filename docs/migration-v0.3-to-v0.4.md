# Migration Guide: v0.3.0 → v0.4.0 "Power Tools"

## TL;DR

**No breaking changes.** All v0.3.0 code compiles and runs unchanged on v0.4.0.

v0.4.0 adds advanced tool registries, exact token counting, LLM-based output
summarization, and MCP tool discovery. All are opt-in additions.

## What's New

| Feature | Type | Where |
|---------|------|--------|
| `GroupedRegistry` | Tool registry | `traitclaw-core` |
| `AdaptiveRegistry` | Tool registry | `traitclaw-core` |
| `ProgressiveTransformer` | Output transformer | `traitclaw-core` |
| `TikTokenCounter` | Token counter | `traitclaw-core` (feature-gated) |
| `McpToolRegistry` | MCP integration | `traitclaw-mcp` |
| `MultiServerMcpRegistry` | Multi-MCP integration | `traitclaw-mcp` |

---

## 1. GroupedRegistry — Named Tool Groups

Organize tools into named groups and activate/deactivate them at runtime.

**Before (v0.3.0):**

```rust
use traitclaw::DynamicRegistry;

let registry = DynamicRegistry::new();
registry.register(Arc::new(SearchTool));
registry.register(Arc::new(CodeGenTool));
// Can only enable/disable individual tools
```

**After (v0.4.0):**

```rust
use traitclaw::GroupedRegistry;

let mut registry = GroupedRegistry::new();
registry.add_group("search", vec![Arc::new(SearchTool), Arc::new(WebFetchTool)]);
registry.add_group("code", vec![Arc::new(CodeGenTool), Arc::new(LintTool)]);

// Activate one group at a time
registry.activate("search");
println!("Active tools: {}", registry.get_tools().len()); // 2

registry.activate("code");
println!("Active tools: {}", registry.get_tools().len()); // 2 (switched)

// Or activate multiple groups
registry.activate_all(&["search", "code"]);
```

---

## 2. AdaptiveRegistry — Tier-Based Tool Limiting

Automatically limit the number of tools exposed based on model tier.

```rust
use traitclaw::{AdaptiveRegistry, TierLimits};
use traitclaw_core::types::model_info::ModelTier;

let registry = AdaptiveRegistry::new(ModelTier::Small)
    .with_limits(TierLimits {
        small: 5,
        medium: 15,
        large: usize::MAX,
    });

// With 20 registered tools and ModelTier::Small, only 5 will be returned
// by get_tools() — the highest-priority ones first
```

---

## 3. ProgressiveTransformer — LLM-Powered Output Summarization

Summarize large tool outputs instead of truncating them.

```rust
use traitclaw::{ProgressiveTransformer, BudgetAwareTruncator};
use std::sync::Arc;

// Summarize outputs > 2000 chars via LLM
let transformer = ProgressiveTransformer::new(Arc::clone(&provider), 2_000);

let agent = Agent::builder()
    .model(provider)
    .output_transformer(transformer.clone())
    // Register the retriever so the agent can fetch the full output
    .tool(transformer.retriever_tool())
    .build()?;

// Short outputs pass through unchanged.
// Large outputs are summarized by the LLM, with a note:
//   "[Full output (8423 chars) cached. Call __get_full_output with
//    {"tool_name": "search"} to retrieve it.]"
```

---

## 4. TikTokenCounter — Exact OpenAI Token Counting

Replace the 4-chars-per-token heuristic with exact BPE counting.

**Cargo.toml:**

```toml
[dependencies]
traitclaw-core = { version = "0.4", features = ["tiktoken"] }
```

**Usage:**

```rust
use traitclaw_core::token_counter::TikTokenCounter;

let counter = TikTokenCounter::for_model("gpt-4o");

// Count tokens in a string
let n = counter.count_str("Hello, world!");

// Count tokens in a message list (includes per-message overhead)
let n = counter.count_messages(&messages);

// One-shot estimate for any model
let n = TikTokenCounter::estimate_for_model(&messages, "gpt-4");
```

Supported encodings:
- `gpt-4o`, `o1*`, `o3*`, `o4*` → `o200k_base`
- `gpt-4*`, `gpt-3.5*`, `text-embedding-*` → `cl100k_base`
- Unknown models → `cl100k_base` with a warning

---

## 5. McpToolRegistry — MCP Tool Discovery

Connect to any MCP server and use its tools without manual schema definition.

```rust
use traitclaw_mcp::McpToolRegistry;

// Connect to a local MCP server (stdio child process)
let registry = McpToolRegistry::stdio("npx", &["@modelcontextprotocol/server-filesystem"]).await?;

println!("Discovered {} tools", registry.len());

// Use as a ToolRegistry in AgentBuilder
let agent = Agent::builder()
    .model(provider)
    .tool_registry(registry)
    .build()?;
```

---

## 6. MultiServerMcpRegistry — Multiple MCP Servers

Aggregate tools from multiple MCP servers with collision-safe prefixing.

```rust
use traitclaw_mcp::MultiServerMcpRegistry;

let registry = MultiServerMcpRegistry::builder()
    .with_prefix(true)                   // enables "server::tool" naming
    .add_stdio("fs", "npx", &["@modelcontextprotocol/server-filesystem"])
    .add_stdio("git", "uvx", &["mcp-server-git"])
    .build()
    .await?;

// Tools are named "fs::read_file", "git::create_commit", etc.
println!("Healthy servers: {}", registry.healthy_server_count());
println!("Total tools: {}", registry.len());
```

Servers that fail to connect are marked unhealthy. Other servers' tools
remain accessible without interruption.

---

## Deprecation Timeline

No deprecations in v0.4.0. All v0.3.0 APIs remain supported.

| Status | Planned removal |
|--------|----------------|
| `ContextStrategy` (from v0.2.0) | v1.0.0 |
| `OutputProcessor` (from v0.2.0) | v1.0.0 |
