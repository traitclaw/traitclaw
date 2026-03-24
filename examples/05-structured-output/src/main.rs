//! # Structured Output — Type-safe JSON responses with TraitClaw
//!
//! Demonstrates using `agent.run_structured::<T>()` to get responses
//! that are automatically parsed into Rust types. The framework injects
//! the JSON schema into the prompt (or uses native structured output
//! when the model supports it).

use schemars::JsonSchema;
use serde::Deserialize;
use traitclaw::prelude::*;
use traitclaw_openai_compat::OpenAiCompatProvider;

// ── Define your output types ────────────────────────────────

/// A movie review with structured fields.
#[derive(Debug, Deserialize, JsonSchema)]
struct MovieReview {
    /// Title of the movie
    title: String,
    /// Rating from 1-10
    rating: u8,
    /// One-sentence summary
    summary: String,
    /// List of pros
    pros: Vec<String>,
    /// List of cons
    cons: Vec<String>,
}

/// A simple sentiment analysis result.
#[derive(Debug, Deserialize, JsonSchema)]
struct SentimentResult {
    /// The detected sentiment
    sentiment: String,
    /// Confidence score from 0.0 to 1.0
    confidence: f64,
    /// Key phrases that influenced the analysis
    key_phrases: Vec<String>,
}

// ── Main ────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let provider = OpenAiCompatProvider::openai(
        "gpt-4o-mini",
        std::env::var("OPENAI_API_KEY").unwrap_or_else(|_| "demo-key".into()),
    );

    let agent = Agent::builder()
        .provider(provider)
        .system("You are a helpful analyst. Always respond with accurate, structured data.")
        .build()?;

    // Example 1: Movie Review
    println!("🎬 Structured Movie Review\n");
    let review: MovieReview = agent
        .run_structured("Review the movie 'Inception' by Christopher Nolan")
        .await?;

    println!("Title: {}", review.title);
    println!("Rating: {}/10", review.rating);
    println!("Summary: {}", review.summary);
    println!("Pros: {:?}", review.pros);
    println!("Cons: {:?}", review.cons);

    // Example 2: Sentiment Analysis
    println!("\n📊 Structured Sentiment Analysis\n");
    let sentiment: SentimentResult = agent
        .run_structured("Analyze the sentiment: 'Rust is incredibly fast and safe, but the learning curve is steep'")
        .await?;

    println!("Sentiment: {}", sentiment.sentiment);
    println!("Confidence: {:.0}%", sentiment.confidence * 100.0);
    println!("Key phrases: {:?}", sentiment.key_phrases);

    Ok(())
}
