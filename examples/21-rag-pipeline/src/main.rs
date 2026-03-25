//! # Example 21: RAG Pipeline v2
//!
//! Demonstrates the full v0.5 RAG pipeline:
//! - Chunking documents with `RecursiveChunker`
//! - Building and querying a `KeywordRetriever`
//! - Auto-grounding agent context with `RagContextManager`
//!
//! # Running
//!
//! ```sh
//! OPENAI_API_KEY=sk-... cargo run --example rag-pipeline-v2
//! ```
//! The retrieval/chunking steps work without any API key.

use traitclaw::prelude::*;
use traitclaw_openai_compat::OpenAiCompatProvider;
use traitclaw_rag::{
    chunker::{Chunker, RecursiveChunker},
    Document, GroundingStrategy, KeywordRetriever, PrependStrategy, Retriever,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🔍 RAG Pipeline v2 — chunks + hybrid retrieval\n");

    // ── 1. Chunk raw documents ───────────────────────────────
    let chunker = RecursiveChunker::new(200);
    let raw_text = "\
        Rust's ownership system ensures memory safety without garbage collection. \
        Each value has exactly one owner. When the owner goes out of scope, \
        the value is dropped automatically. This eliminates entire classes of \
        memory bugs like use-after-free and double-free.";

    let chunks = chunker.chunk(raw_text);
    println!("📦 Chunked into {} pieces:", chunks.len());
    for (i, c) in chunks.iter().enumerate() {
        println!("  Chunk {}: {} chars", i + 1, c.len());
    }

    // ── 2. Build knowledge base ──────────────────────────────
    let mut retriever = KeywordRetriever::new();

    retriever.add(Document::new(
        "rust-ownership",
        "Rust's ownership system ensures memory safety without garbage collection. \
         Each value has exactly one owner.",
    ));
    retriever.add(Document::new(
        "rust-borrowing",
        "Borrowing allows you to reference data without taking ownership. \
         You can have either one mutable reference OR any number of immutable references.",
    ));
    retriever.add(Document::new(
        "rust-lifetimes",
        "Lifetimes are Rust's way of ensuring references are valid as long as they're used.",
    ));
    retriever.add(Document::new(
        "rust-traits",
        "Traits define shared behavior and are similar to interfaces. \
         Trait bounds constrain generic code.",
    ));
    retriever.add(Document::new(
        "rust-async",
        "Async/await in Rust uses zero-cost futures. Tokio provides \
         a multi-threaded executor for efficient concurrent programming.",
    ));

    println!("\n📚 Knowledge base: 5 documents\n");

    // ── 3. Retrieve relevant documents ──────────────────────
    let query = "How does Rust handle memory and ownership?";
    println!("❓ Query: \"{query}\"\n");

    let docs = retriever.retrieve(query, 3).await?;
    println!("🔍 Top {} retrieved documents:", docs.len());
    for doc in &docs {
        let preview = if doc.content.len() > 60 {
            format!("{}…", &doc.content[..60])
        } else {
            doc.content.clone()
        };
        println!("  [{:.2}] {} — {}", doc.score, doc.id, preview);
    }

    // ── 4. Ground context ────────────────────────────────────
    let strategy = PrependStrategy;
    let context = strategy.ground(&docs);
    println!("\n🎯 Grounded system context:\n{context}\n");

    // ── 5. Run agent (requires OPENAI_API_KEY) ───────────────
    println!("🤖 Building grounded agent…\n");
    let provider = OpenAiCompatProvider::openai(
        "gpt-4o-mini",
        std::env::var("OPENAI_API_KEY").unwrap_or_else(|_| "demo-key".into()),
    );

    let agent = Agent::builder()
        .provider(provider)
        .system(format!(
            "You are a Rust expert. Answer ONLY using the provided context.\n\n{context}"
        ))
        .build()?;

    println!("📝 Running agent…");
    let response = agent.run(query).await?;
    println!("🤖 Answer: {}\n", response.text());

    println!("✅ RAG pipeline complete!");
    Ok(())
}
