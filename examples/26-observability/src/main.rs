//! # Observability Example — Structured Tracing for AI Agents
//!
//! Demonstrates TraitClaw v0.8.0 observability features:
//! - **Structured tracing** with the `tracing` crate
//! - **GenAI semantic conventions** (`gen_ai.*` attributes)
//! - **Component-level filtering** via `RUST_LOG` targets
//! - **OpenTelemetry compatibility** via `tracing-opentelemetry` bridge
//!
//! ## How It Works
//!
//! TraitClaw instruments all core runtime operations with `tracing` spans:
//! - `gen_ai.chat` — LLM provider calls (model, input/output tokens)
//! - `tool.call` — Tool executions (tool name, success/failure)
//! - `guard.check` — Guard evaluations (guard name, allow/deny/sanitize)
//!
//! You choose the backend by configuring a `tracing::Subscriber`:
//! - **Console** (`tracing-subscriber::fmt`) — for local development
//! - **Jaeger/Tempo** (`tracing-opentelemetry` + OTLP exporter) — for production
//! - **Langfuse** (OTEL-native endpoint) — for LLM-specific observability
//!
//! ## Running
//!
//! ```bash
//! # Show all TraitClaw spans (LLM, tool, guard, hint):
//! RUST_LOG=traitclaw=info cargo run -p observability-example
//!
//! # Show only LLM calls:
//! RUST_LOG=traitclaw::llm=info cargo run -p observability-example
//!
//! # Show everything including hint injections:
//! RUST_LOG=traitclaw=debug cargo run -p observability-example
//! ```

use tracing_subscriber::EnvFilter;

fn main() {
    // ─────────────────────────────────────────────────────────────────
    // Step 1: Configure tracing subscriber
    // ─────────────────────────────────────────────────────────────────
    //
    // This is the ONLY setup needed. TraitClaw emits spans automatically.
    // You can swap this for any tracing-compatible backend:
    //
    //   - tracing-opentelemetry → Jaeger, Grafana Tempo, Datadog
    //   - Langfuse OTLP endpoint → LLM-specific observability
    //   - Custom tracing::Layer → build your own event handler
    //
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("traitclaw=info")),
        )
        .with_target(true) // Show span targets like "traitclaw::llm"
        .init();

    println!("=== TraitClaw Observability Example ===");
    println!();

    // ─────────────────────────────────────────────────────────────────
    // Step 2: Demonstrate tracing span structure
    // ─────────────────────────────────────────────────────────────────

    println!("TraitClaw automatically instruments these operations:");
    println!();
    println!("  🔭 LLM Calls → span: gen_ai.chat");
    println!("     Attributes: gen_ai.system, gen_ai.request.model,");
    println!("                 gen_ai.usage.input_tokens, gen_ai.usage.output_tokens");
    println!();
    println!("  🔧 Tool Calls → span: tool.call");
    println!("     Attributes: tool.name, tool.success");
    println!();
    println!("  🛡️  Guard Checks → span: guard.check");
    println!("     Attributes: guard.name, guard.result (allow/deny/sanitize/panic)");
    println!();
    println!("  💡 Hint Injection → event: debug level");
    println!("     Attributes: hint_name");
    println!();

    // ─────────────────────────────────────────────────────────────────
    // Step 3: Show what trace output looks like
    // ─────────────────────────────────────────────────────────────────

    // Simulate the spans that TraitClaw would emit during agent.run()
    let llm_span = tracing::info_span!(
        target: "traitclaw::llm",
        "gen_ai.chat",
        gen_ai.system = "traitclaw",
        gen_ai.request.model = "gpt-4o-mini",
        gen_ai.usage.input_tokens = tracing::field::Empty,
        gen_ai.usage.output_tokens = tracing::field::Empty,
    );
    {
        let _guard = llm_span.enter();
        tracing::info!(target: "traitclaw::llm", "Sending completion request");
        // Simulate response
        llm_span.record("gen_ai.usage.input_tokens", 42_u32);
        llm_span.record("gen_ai.usage.output_tokens", 18_u32);
        tracing::info!(target: "traitclaw::llm", "LLM response received");
    }

    let tool_span = tracing::info_span!(
        target: "traitclaw::tool",
        "tool.call",
        tool.name = "calculator",
        tool.success = tracing::field::Empty,
    );
    {
        let _guard = tool_span.enter();
        tracing::info!(target: "traitclaw::tool", "Executing tool");
        tool_span.record("tool.success", true);
    }

    let guard_span = tracing::info_span!(
        target: "traitclaw::guard",
        "guard.check",
        guard.name = "content_policy",
        guard.result = tracing::field::Empty,
    );
    {
        let _guard = guard_span.enter();
        guard_span.record("guard.result", "allow");
    }

    tracing::debug!(
        target: "traitclaw::hint",
        hint_name = "token_budget_warning",
        "Hint injected"
    );

    println!();
    println!("─── Filter Examples ───");
    println!();
    println!("  RUST_LOG=traitclaw=info          → All LLM + tool + guard spans");
    println!("  RUST_LOG=traitclaw::llm=debug     → Only LLM calls (detailed)");
    println!("  RUST_LOG=traitclaw::tool=info      → Only tool executions");
    println!("  RUST_LOG=traitclaw::guard=info     → Only guard checks");
    println!("  RUST_LOG=traitclaw=debug           → Everything including hints");
    println!();

    // ─────────────────────────────────────────────────────────────────
    // Step 4: OpenTelemetry / Langfuse integration guide
    // ─────────────────────────────────────────────────────────────────

    println!("─── OTEL / Langfuse Integration ───");
    println!();
    println!("  To export spans to Jaeger, Langfuse, or any OTEL backend:");
    println!();
    println!("  1. Add dependencies:");
    println!("     tracing-opentelemetry = \"0.26\"");
    println!("     opentelemetry = \"0.25\"");
    println!("     opentelemetry-otlp = {{ version = \"0.25\", features = [\"grpc-tonic\"] }}");
    println!();
    println!("  2. Configure the OTEL layer:");
    println!("     let tracer = opentelemetry_otlp::new_pipeline()");
    println!("         .with_endpoint(\"http://localhost:4317\")  // or Langfuse OTLP URL");
    println!("         .install_batch()?;");
    println!("     let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);");
    println!("     tracing_subscriber::registry().with(otel_layer).init();");
    println!();
    println!("  That's it! All TraitClaw spans flow automatically to your backend.");
    println!("  Langfuse parses gen_ai.* attributes into its LLM dashboard natively.");
}
