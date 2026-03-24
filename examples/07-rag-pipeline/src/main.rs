//! # RAG Pipeline — Retrieval-Augmented Generation with TraitClaw
//!
//! Demonstrates using `KeywordRetriever` and `GroundingStrategy` to
//! retrieve relevant documents and inject them as context for the agent.
//! No external vector database required — uses built-in BM25 scoring.

use traitclaw::prelude::*;
use traitclaw_openai_compat::OpenAiCompatProvider;
use traitclaw_rag::{Document, GroundingStrategy, KeywordRetriever, PrependStrategy, Retriever};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🔍 RAG Pipeline Demo\n");

    // ── 1. Build a knowledge base ───────────────────────────
    let mut retriever = KeywordRetriever::new();

    // Add documents (in a real app, load from files, APIs, or databases)
    retriever.add(Document::new(
        "rust-ownership",
        "Rust's ownership system ensures memory safety without garbage collection. \
         Each value has exactly one owner. When the owner goes out of scope, the value is dropped.",
    ));
    retriever.add(Document::new(
        "rust-borrowing",
        "Borrowing allows you to reference data without taking ownership. \
         You can have either one mutable reference OR any number of immutable references.",
    ));
    retriever.add(Document::new(
        "rust-lifetimes",
        "Lifetimes are Rust's way of ensuring that references are valid for as long as they're used. \
         The compiler checks lifetime annotations to prevent dangling references.",
    ));
    retriever.add(Document::new(
        "rust-traits",
        "Traits define shared behavior. They are similar to interfaces in other languages. \
         You can use trait bounds to write generic code that works with any type implementing the trait.",
    ));
    retriever.add(Document::new(
        "rust-async",
        "Async/await in Rust uses zero-cost futures. The tokio runtime provides \
         a multi-threaded executor for efficient concurrent programming.",
    ));

    println!("📚 Indexed {} documents\n", 5);

    // ── 2. Retrieve relevant documents ──────────────────────
    let query = "How does Rust handle memory safety?";
    println!("❓ Query: \"{query}\"\n");

    let docs = retriever.retrieve(query, 3).await?;
    println!("📄 Retrieved {} relevant documents:", docs.len());
    for doc in &docs {
        println!("  [{:.2}] {} — {}", doc.score, doc.id, &doc.content[..60]);
    }

    // ── 3. Ground the context ───────────────────────────────
    let strategy = PrependStrategy;
    let grounded_context = strategy.ground(&docs);
    println!("\n🎯 Grounded context:\n{grounded_context}");

    // ── 4. Use with an agent (optional — needs API key) ────
    println!("─── Agent Integration ───\n");

    let provider = OpenAiCompatProvider::openai(
        "gpt-4o-mini",
        std::env::var("OPENAI_API_KEY").unwrap_or_else(|_| "demo-key".into()),
    );

    let agent = Agent::builder()
        .provider(provider)
        .system(format!(
            "You are a Rust expert. Answer questions using ONLY the provided context.\n\n{grounded_context}"
        ))
        .build()?;

    let output = agent.run(query).await?;
    println!("🤖 Answer: {}\n", output.text());

    println!("✅ Done!");
    Ok(())
}
