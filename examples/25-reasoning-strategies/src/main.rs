//! # Reasoning Strategies Example
//!
//! Demonstrates the three built-in reasoning strategies available in
//! `traitclaw-strategies`:
//!
//! 1. **ReAct** — Think→Act→Observe loops with tool use
//! 2. **Chain-of-Thought** — Structured step-by-step reasoning
//! 3. **MCTS** — Monte Carlo Tree Search with parallel branches
//!
//! Each strategy implements `AgentStrategy`, so they are interchangeable
//! with `DefaultStrategy` or any custom strategy.
//!
//! ## Usage
//!
//! ```bash
//! OPENAI_API_KEY=sk-... cargo run -p reasoning-strategies
//! ```

use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use traitclaw::prelude::*;
use traitclaw_strategies::{
    ChainOfThoughtStrategy, MctsStrategy, ReActStrategy, StrategyEvent, ThoughtStep,
};

// ── Example Tool ─────────────────────────────────────────────────────────

#[derive(serde::Deserialize, schemars::JsonSchema)]
struct CalculateInput {
    expression: String,
}

#[derive(serde::Serialize)]
struct CalculateOutput {
    result: String,
}

struct CalculateTool;

#[async_trait]
impl Tool for CalculateTool {
    type Input = CalculateInput;
    type Output = CalculateOutput;

    fn name(&self) -> &str {
        "calculate"
    }
    fn description(&self) -> &str {
        "Evaluate a math expression"
    }

    async fn execute(&self, input: Self::Input) -> traitclaw::Result<Self::Output> {
        // Simple eval for demo
        Ok(CalculateOutput {
            result: format!("Result of '{}' = (computed)", input.expression),
        })
    }
}

// ── Demo Functions ───────────────────────────────────────────────────────

fn demo_strategy_events() {
    println!("=== Strategy Event Types ===\n");

    let events = vec![
        StrategyEvent::Thought(ThoughtStep::Think {
            content: "Let me analyze the problem...".into(),
        }),
        StrategyEvent::ToolCall {
            tool_name: "calculate".into(),
            arguments: serde_json::json!({"expression": "2 + 2"}),
        },
        StrategyEvent::ToolResult {
            tool_name: "calculate".into(),
            output: "4".into(),
        },
        StrategyEvent::Thought(ThoughtStep::Answer {
            content: "The answer is 4.".into(),
        }),
        StrategyEvent::Done {
            iterations: 2,
            total_tokens: 150,
        },
    ];

    for event in &events {
        println!("  {:?}", event);
    }

    // Serialize to JSON
    println!("\n  JSON serialization:");
    for event in &events {
        let json = serde_json::to_string(event).unwrap();
        println!("    {json}");
    }
    println!();
}

fn demo_react_builder() {
    println!("=== ReAct Strategy Builder ===\n");

    let strategy = ReActStrategy::builder()
        .max_iterations(10)
        .system_prompt("You are a helpful assistant that thinks step by step.")
        .build()
        .unwrap();

    println!("  max_iterations: {}", strategy.max_iterations());
    println!("  thought_steps:  {:?}", strategy.thought_steps());
    println!("  Debug:          {:?}", strategy);
    println!();
}

fn demo_cot_builder() {
    println!("=== Chain-of-Thought Strategy Builder ===\n");

    let strategy = ChainOfThoughtStrategy::builder()
        .max_steps(5)
        .build()
        .unwrap();

    println!("  max_steps:     {}", strategy.max_steps());
    println!("  thought_steps: {:?}", strategy.thought_steps());
    println!("  Debug:         {:?}", strategy);
    println!();
}

fn demo_mcts_builder() {
    println!("=== MCTS Strategy Builder ===\n");

    let strategy = MctsStrategy::builder()
        .branches(5)
        .max_depth(3)
        .scoring(Arc::new(|answer: &str| {
            // Custom scoring: prefer longer, structured answers
            let len_score = (answer.len() as f64 / 200.0).min(1.0);
            let structure_bonus = if answer.contains('\n') { 0.1 } else { 0.0 };
            (len_score + structure_bonus).min(1.0)
        }))
        .build()
        .unwrap();

    println!("  branches:       {}", strategy.branches());
    println!("  max_depth:      {}", strategy.max_depth());
    println!("  branch_results: {:?}", strategy.branch_results());
    println!("  Debug:          {:?}", strategy);
    println!();
}

fn demo_interchangeability() {
    println!("=== Strategy Interchangeability ===\n");

    let strategies: Vec<Box<dyn AgentStrategy>> = vec![
        Box::new(DefaultStrategy),
        Box::new(ReActStrategy::builder().build().unwrap()),
        Box::new(ChainOfThoughtStrategy::builder().build().unwrap()),
        Box::new(MctsStrategy::builder().build().unwrap()),
    ];

    let names = [
        "DefaultStrategy",
        "ReActStrategy",
        "ChainOfThoughtStrategy",
        "MctsStrategy",
    ];

    for (strategy, name) in strategies.iter().zip(names.iter()) {
        // All strategies are Box<dyn AgentStrategy> — fully interchangeable
        let _ = strategy; // would call strategy.execute(runtime, input, session_id)
        println!("  ✅ {name} → Box<dyn AgentStrategy>");
    }
    println!();
}

fn demo_thought_step_types() {
    println!("=== ThoughtStep Types ===\n");

    let steps = vec![
        ThoughtStep::Think {
            content: "I need to find the square root of 144.".into(),
        },
        ThoughtStep::Act {
            tool_name: "calculate".into(),
            tool_input: serde_json::json!({"expression": "sqrt(144)"}),
        },
        ThoughtStep::Observe {
            tool_output: "12".into(),
        },
        ThoughtStep::Answer {
            content: "The square root of 144 is 12.".into(),
        },
    ];

    for step in &steps {
        let json = serde_json::to_string(step).unwrap();
        println!("  {json}");
    }
    println!();
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("╔══════════════════════════════════════════╗");
    println!("║   TraitClaw Reasoning Strategies v0.7.0  ║");
    println!("╚══════════════════════════════════════════╝\n");

    demo_thought_step_types();
    demo_strategy_events();
    demo_react_builder();
    demo_cot_builder();
    demo_mcts_builder();
    demo_interchangeability();

    println!("─── All strategy builders and types verified ───\n");
    println!("To run strategies against an LLM, set OPENAI_API_KEY and");
    println!("configure an Agent with .strategy(react_strategy) etc.\n");

    Ok(())
}
