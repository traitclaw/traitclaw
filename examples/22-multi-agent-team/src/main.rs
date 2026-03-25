//! # Example 22: Multi-Agent Team
//!
//! Demonstrates v0.5 multi-agent orchestration:
//! - `TeamRunner` for sequential pipelines
//! - `run_verification_chain` for generate-verify-retry loops
//! - `TeamContext` for sharing state between agents
//!
//! # Running
//!
//! This example works fully offline — all agents are mock closures.
//!
//! ```sh
//! cargo run -p multi-agent-team
//! ```

use traitclaw_team::{run_verification_chain, TeamContext, TeamRunner};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🤝 Multi-Agent Team Demo\n");

    // ── 1. TeamRunner: sequential pipeline ───────────────────
    println!("── Step 1: Sequential Pipeline (researcher → summarizer → reviewer) ──\n");

    let mut runner = TeamRunner::new(10);

    runner.bind("researcher", |input: String| async move {
        Ok(format!(
            "[Research] Found 3 key facts about: {input}\n\
             Fact 1: Rust ensures memory safety via ownership.\n\
             Fact 2: No garbage collector overhead.\n\
             Fact 3: Zero-cost abstractions."
        ))
    });

    runner.bind("summarizer", |input: String| async move {
        let lines: Vec<&str> = input.lines().collect();
        Ok(format!(
            "[Summary] {} — condensed to {} lines.",
            lines[0],
            lines.len()
        ))
    });

    runner.bind("reviewer", |input: String| async move {
        Ok(format!("[Approved ✓] {input}"))
    });

    runner.set_sequence(&["researcher", "summarizer", "reviewer"]);

    let result = runner.run("Rust programming language").await?;
    println!("📝 Final output:\n{result}\n");

    // ── 2. VerificationChain: generate-verify-retry ───────────
    println!("── Step 2: Verification Chain (generate → verify → retry if needed) ──\n");

    let attempt = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let attempt_clone = attempt.clone();

    let result = run_verification_chain(
        "Write a haiku about Rust",
        3,
        move |prompt: String| {
            let n = attempt_clone.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            async move {
                println!("  🖊  Generator attempt #{n}: processing…");
                if n == 0 {
                    // First draft — missing syllable structure
                    Ok(format!("Draft {n}: {prompt} — just a rough draft."))
                } else {
                    // Improved draft on retry
                    Ok(
                        "Borrow without fear,\nOwnership keeps memory clean,\nRust never forgets."
                            .to_string(),
                    )
                }
            }
        },
        |output: String| async move {
            println!("  🔍 Verifier checking output…");
            if output.contains("Draft") {
                Err(format!(
                    "Not a proper haiku yet: \"{output}\". Please use 5-7-5 syllable structure."
                ))
            } else {
                println!("  ✅ Haiku accepted!");
                Ok(output)
            }
        },
    )
    .await?;

    println!("\n📜 Final haiku:\n{result}\n");

    // ── 3. TeamContext: shared state ──────────────────────────
    println!("── Step 3: Shared TeamContext ──\n");

    let ctx = TeamContext::new();

    ctx.set("topic", serde_json::json!("async Rust"));
    ctx.set("quality_gate", serde_json::json!("passed"));
    ctx.set("iteration", serde_json::json!(2));

    println!("  topic       = {:?}", ctx.get("topic"));
    println!("  quality_gate = {:?}", ctx.get("quality_gate"));
    println!("  iteration   = {:?}", ctx.get("iteration"));
    println!("  missing_key = {:?}", ctx.get("missing_key"));

    println!("\n✅ Multi-agent team demo complete!");
    Ok(())
}
