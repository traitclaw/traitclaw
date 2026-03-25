//! # Example 23: Eval Runner
//!
//! Demonstrates v0.5 async evaluation:
//! - Defining test suites with `EvalSuite` and `TestCase`
//! - Running evaluations with `EvalRunner` + built-in `KeywordMetric`
//! - Custom async metric via `AsyncMetric` trait
//! - Reading `EvalReport` summary
//!
//! # Running
//!
//! This example is fully offline — uses a mock agent.
//!
//! ```sh
//! cargo run -p eval-runner
//! ```

use async_trait::async_trait;
use traitclaw_core::Result as CoreResult;
use traitclaw_eval::{
    runner::{AsyncMetric, EvalAgent, EvalRunner},
    EvalSuite, KeywordMetric, SyncMetricAdapter, TestCase,
};

// ── Mock Agent ────────────────────────────────────────────────────────────────

/// A mock agent that returns a canned response for demo purposes.
struct MockAgent;

#[async_trait]
impl EvalAgent for MockAgent {
    async fn respond(&self, input: &str) -> CoreResult<String> {
        // Simulate a helpful response
        let response = match input {
            q if q.contains("Rust") && q.contains("memory") => {
                "Rust ensures memory safety through ownership. There is no garbage collector."
            }
            q if q.contains("async") => {
                "Async Rust uses futures and the tokio runtime for concurrent programming."
            }
            q if q.contains("trait") => {
                "Traits in Rust define shared behavior, similar to interfaces."
            }
            _ => "I'm not sure about that. Please ask about Rust.",
        };
        Ok(response.to_string())
    }
}

// ── Custom Async Metric ───────────────────────────────────────────────────────

/// A simple length metric: penalizes very short responses (< 30 chars = 0.5).
struct LengthMetric;

#[async_trait]
impl AsyncMetric for LengthMetric {
    fn name(&self) -> &'static str {
        "length"
    }

    async fn score(&self, _input: &str, actual_output: &str, _kw: &[&str]) -> f64 {
        if actual_output.len() >= 60 {
            1.0
        } else if actual_output.len() >= 30 {
            0.75
        } else {
            0.5
        }
    }
}

// ── Main ──────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("📊 Eval Runner Demo\n");

    // ── 1. Build evaluation suite ────────────────────────────
    let suite = EvalSuite::new("rust_qa_v1")
        .add_case(
            TestCase::new("memory-safety", "How does Rust handle memory safety?")
                .expect_contains("ownership")
                .expect_contains("memory"),
        )
        .add_case(
            TestCase::new("async-runtime", "Explain async programming in Rust.")
                .expect_contains("async")
                .expect_contains("tokio"),
        )
        .add_case(
            TestCase::new("traits", "What are Rust traits?")
                .expect_contains("trait")
                .expect_contains("interface"),
        )
        .add_case(
            TestCase::new("off-topic", "What is the capital of France?").expect_contains("Paris"), // Agent won't say Paris → should fail
        );

    println!(
        "📋 Suite: '{}' — {} test cases\n",
        suite.name(),
        suite.cases().len()
    );

    // ── 2. Run with keyword metric + custom length metric ────
    let runner = EvalRunner::new()
        .metric(Box::new(SyncMetricAdapter(KeywordMetric)))
        .metric(Box::new(LengthMetric))
        .threshold(0.7);

    let agent = MockAgent;
    let report = runner
        .run(&agent, &suite)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    // ── 3. Print results ─────────────────────────────────────
    println!("── Results ──────────────────────────────────────────────\n");
    for result in &report.results {
        let status = if result.passed {
            "✅ PASS"
        } else {
            "❌ FAIL"
        };
        println!("{status} [{}]", result.case_id);
        println!(
            "  Output: {}",
            &result.actual_output[..result.actual_output.len().min(80)]
        );
        for (metric, score) in &result.scores {
            println!("  {}: {:.2}", metric, score);
        }
        println!();
    }

    // ── 4. Print summary ─────────────────────────────────────
    println!("── Summary ──────────────────────────────────────────────\n");
    println!("  Suite:         {}", report.suite_name);
    println!("  Total cases:   {}", report.total);
    println!("  Passed:        {}/{}", report.passed, report.total);
    println!("  Average score: {:.2}", report.average_score);
    println!(
        "  Result:        {}",
        if report.passed == report.total {
            "🎉 All passed!"
        } else {
            "⚠️  Some cases failed."
        }
    );

    println!("\n✅ Eval runner demo complete!");
    Ok(())
}
