//! Example: Lifecycle Hooks
//!
//! Demonstrates AgentHook for observability and tool interception.
//! Shows three hook patterns:
//! 1. LoggingHook — built-in tracing integration
//! 2. MetricsHook — custom metrics collector
//! 3. SecurityHook — tool execution policy enforcement

use async_trait::async_trait;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use traitclaw_core::agent::AgentOutput;
use traitclaw_core::traits::hook::{AgentHook, HookAction};
use traitclaw_core::types::completion::CompletionResponse;

/// Collects metrics: total LLM calls, total tokens, average latency.
struct MetricsHook {
    llm_calls: AtomicUsize,
    total_tokens: AtomicUsize,
    total_latency_ms: AtomicUsize,
    tool_calls: AtomicUsize,
    blocked_calls: AtomicUsize,
}

impl MetricsHook {
    fn new() -> Self {
        Self {
            llm_calls: AtomicUsize::new(0),
            total_tokens: AtomicUsize::new(0),
            total_latency_ms: AtomicUsize::new(0),
            tool_calls: AtomicUsize::new(0),
            blocked_calls: AtomicUsize::new(0),
        }
    }

    fn report(&self) {
        let calls = self.llm_calls.load(Ordering::Relaxed);
        let tokens = self.total_tokens.load(Ordering::Relaxed);
        let latency = self.total_latency_ms.load(Ordering::Relaxed);
        let tools = self.tool_calls.load(Ordering::Relaxed);
        let blocked = self.blocked_calls.load(Ordering::Relaxed);
        let avg_latency = if calls > 0 { latency / calls } else { 0 };

        println!("\n📊 Agent Metrics Report:");
        println!("  LLM calls:     {calls}");
        println!("  Total tokens:  {tokens}");
        println!("  Avg latency:   {avg_latency}ms");
        println!("  Tool calls:    {tools}");
        println!("  Blocked calls: {blocked}");
    }
}

#[async_trait]
impl AgentHook for MetricsHook {
    async fn on_agent_start(&self, input: &str) {
        println!("🚀 Agent started (input: {} chars)", input.len());
    }

    async fn on_agent_end(&self, _output: &AgentOutput, duration: Duration) {
        println!("✅ Agent completed in {duration:?}");
    }

    async fn on_provider_end(&self, response: &CompletionResponse, duration: Duration) {
        self.llm_calls.fetch_add(1, Ordering::Relaxed);
        self.total_tokens
            .fetch_add(response.usage.total_tokens, Ordering::Relaxed);
        #[allow(clippy::cast_possible_truncation)]
        let ms = duration.as_millis() as usize;
        self.total_latency_ms.fetch_add(ms, Ordering::Relaxed);
    }

    async fn after_tool_execute(&self, name: &str, _result: &str, duration: Duration) {
        self.tool_calls.fetch_add(1, Ordering::Relaxed);
        println!("🔧 Tool '{name}' completed in {duration:?}");
    }
}

/// Blocks dangerous tools based on a deny-list.
struct SecurityHook {
    denied_tools: Vec<String>,
}

impl SecurityHook {
    fn new(denied_tools: Vec<String>) -> Self {
        Self { denied_tools }
    }
}

#[async_trait]
impl AgentHook for SecurityHook {
    async fn before_tool_execute(&self, name: &str, _args: &serde_json::Value) -> HookAction {
        if self.denied_tools.iter().any(|d| name.contains(d.as_str())) {
            println!("🚫 BLOCKED: tool '{name}' matches security deny-list");
            HookAction::Block(format!("Tool '{name}' is blocked by security policy"))
        } else {
            HookAction::Continue
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== Lifecycle Hooks Example ===");
    println!();
    println!("This example demonstrates three hook patterns:");
    println!("  1. MetricsHook — collects LLM call stats");
    println!("  2. SecurityHook — blocks denied tools");
    println!("  3. LoggingHook — built-in tracing (traitclaw_core::traits::hook::LoggingHook)");
    println!();

    // Demonstrate creating hooks
    let metrics = Arc::new(MetricsHook::new());
    let security = SecurityHook::new(vec!["delete".into(), "drop".into(), "rm".into()]);

    println!("Hooks created:");
    println!("  ✅ MetricsHook (shared via Arc for post-run reporting)");
    println!("  ✅ SecurityHook (denies: delete, drop, rm)");

    // Show how to wire them:
    println!();
    println!("Usage with Agent::builder():");
    println!("  Agent::builder()");
    println!("      .model(my_provider)");
    println!("      .hook(Arc::clone(&metrics))");
    println!("      .hook(security)");
    println!("      .hook(LoggingHook::new(tracing::Level::INFO))");
    println!("      .build()?;");

    // Show metrics report
    metrics.report();

    // Demonstrate security hook evaluation
    println!();
    let action = security
        .before_tool_execute("delete_file", &serde_json::json!({"path": "/etc"}))
        .await;
    match action {
        HookAction::Block(reason) => println!("→ Block result: {reason}"),
        HookAction::Continue => println!("→ Allowed"),
    }

    let action = security
        .before_tool_execute("read_file", &serde_json::json!({"path": "/tmp"}))
        .await;
    match action {
        HookAction::Block(reason) => println!("→ Block result: {reason}"),
        HookAction::Continue => println!("→ Allowed: read_file passes security"),
    }

    Ok(())
}
