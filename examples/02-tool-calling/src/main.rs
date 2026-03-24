//! # Tool Calling — Defining and using tools with BaseClaw
//!
//! Demonstrates how to define custom tools using the `Tool` trait and
//! register them with an agent for function-calling workflows.

use async_trait::async_trait;
use baseclaw::prelude::*;
use baseclaw_openai_compat::OpenAiCompatProvider;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// ── Tool 1: Calculator ─────────────────────────────────────

#[derive(Deserialize, JsonSchema)]
struct CalculatorInput {
    /// Mathematical expression to evaluate (e.g., "2 + 3")
    expression: String,
}

#[derive(Serialize)]
struct CalculatorOutput {
    result: String,
}

struct Calculator;

#[async_trait]
impl Tool for Calculator {
    type Input = CalculatorInput;
    type Output = CalculatorOutput;

    fn name(&self) -> &str {
        "calculator"
    }

    fn description(&self) -> &str {
        "Evaluate a mathematical expression"
    }

    async fn execute(&self, input: Self::Input) -> baseclaw::Result<Self::Output> {
        let result = format!("Result of '{}' = (computed)", input.expression);
        Ok(CalculatorOutput { result })
    }
}

// ── Tool 2: Weather Lookup ──────────────────────────────────

#[derive(Deserialize, JsonSchema)]
struct WeatherInput {
    /// City name to look up weather for
    city: String,
}

#[derive(Serialize)]
struct WeatherOutput {
    temperature: String,
    condition: String,
}

struct WeatherLookup;

#[async_trait]
impl Tool for WeatherLookup {
    type Input = WeatherInput;
    type Output = WeatherOutput;

    fn name(&self) -> &str {
        "weather_lookup"
    }

    fn description(&self) -> &str {
        "Get current weather for a city"
    }

    async fn execute(&self, input: Self::Input) -> baseclaw::Result<Self::Output> {
        Ok(WeatherOutput {
            temperature: "22°C".into(),
            condition: format!("Sunny in {}", input.city),
        })
    }
}

// ── Main ────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let provider = OpenAiCompatProvider::openai(
        "gpt-4o-mini",
        std::env::var("OPENAI_API_KEY").unwrap_or_else(|_| "demo-key".into()),
    );

    // Register tools with the agent
    let agent = Agent::builder()
        .provider(provider)
        .system("You are a helpful assistant with access to tools.")
        .tool(Calculator)
        .tool(WeatherLookup)
        .build()?;

    // The agent will decide when to call tools based on the prompt
    let output = agent
        .run("What is 42 * 17? Also, what's the weather in Tokyo?")
        .await?;

    println!("Response: {}", output.text());
    Ok(())
}
