//! # Example 21: RAG Pipeline v2 — Full v0.5.0 Feature Showcase
//!
//! Demonstrates the complete v0.5.0 RAG pipeline, covering all new types:
//!
//! - **`RecursiveChunker`** — split raw text into overlapping chunks
//! - **`HybridRetriever`** — combine keyword + semantic search with score weighting
//! - **`CitationStrategy`** — format retrieved docs with numbered citations
//! - **`ContextWindowStrategy`** — cap injected context to a token budget
//! - **`RagContextManager`** — wire a retriever + grounding into the Agent builder
//!
//! The retrieval/chunking steps run without any API key.
//! The final agent step requires `OPENAI_API_KEY`.
//!
//! # Running
//!
//! ```sh
//! OPENAI_API_KEY=sk-... cargo run --example rag-pipeline-v2
//! ```

use traitclaw::prelude::*;
use traitclaw_openai_compat::OpenAiCompatProvider;
use traitclaw_rag::{
    chunker::{Chunker, RecursiveChunker},
    hybrid::{CitationStrategy, ContextWindowStrategy, HybridRetriever},
    rag_context::RagContextManager,
    Document, GroundingStrategy, KeywordRetriever, Retriever,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🔍 RAG Pipeline v2 — HybridRetriever + RagContextManager\n");

    // ── Step 1: Chunk raw documents ──────────────────────────────────────────
    println!("=== Step 1: RecursiveChunker ===\n");

    let chunker = RecursiveChunker::new(150); // 150-char target chunk size
    let raw_corpus = [
        "Rust's ownership system ensures memory safety without garbage collection. \
         Each value has exactly one owner. When the owner goes out of scope, \
         the value is dropped automatically, eliminating use-after-free and double-free bugs.",
        "Borrowing lets you reference data without taking ownership. \
         You can have either one mutable reference OR any number of immutable references \
         at the same time — the borrow checker enforces this at compile time.",
        "Async/await in Rust uses zero-cost futures. The tokio runtime provides \
         a multi-threaded executor. Futures are lazy: they do nothing until polled \
         by an executor, keeping resource usage minimal.",
    ];

    let mut retriever_kw = KeywordRetriever::new();
    let mut retriever_sem = KeywordRetriever::new(); // simulates a semantic retriever

    for (i, text) in raw_corpus.iter().enumerate() {
        let chunks = chunker.chunk(text);
        println!("  Doc {} → {} chunk(s):", i + 1, chunks.len());
        for (j, chunk) in chunks.iter().enumerate() {
            let id = format!("doc{}-chunk{}", i + 1, j + 1);
            println!(
                "    [{}] {} chars: \"{}…\"",
                id,
                chunk.len(),
                &chunk[..chunk.len().min(40)]
            );
            retriever_kw.add(Document::new(&id, chunk));
            // Simulate 'semantic' side with slightly different perspective docs
            retriever_sem.add(Document::new(
                format!("sem-{id}").as_str(),
                &format!("[semantic] {chunk}"),
            ));
        }
    }

    // ── Step 2: HybridRetriever — merge keyword + semantic ───────────────────
    println!("\n=== Step 2: HybridRetriever (0.4 keyword / 0.6 semantic) ===\n");

    let hybrid = HybridRetriever::new(retriever_kw, retriever_sem).with_weights(0.4, 0.6);

    let query = "How does Rust handle memory safety?";
    println!("  Query: \"{query}\"\n");

    let results = hybrid.retrieve(query, 3).await?;
    println!("  Top {} hybrid results:", results.len());
    for doc in &results {
        println!(
            "    [{:.3}] {} — \"{}…\"",
            doc.score,
            doc.id,
            &doc.content[..doc.content.len().min(50)]
        );
    }

    // ── Step 3: CitationStrategy — numbered citations with source IDs ─────────
    println!("\n=== Step 3: CitationStrategy ===\n");

    let citation_ctx = CitationStrategy.ground(&results);
    println!("{citation_ctx}");

    // ── Step 4: ContextWindowStrategy — token-budget grounding ───────────────
    println!("=== Step 4: ContextWindowStrategy (200 token budget) ===\n");

    let windowed_ctx = ContextWindowStrategy::new(200).ground(&results);
    println!(
        "  Context length: {} chars (budget: ~800 chars)\n",
        windowed_ctx.len()
    );

    // ── Step 5: RagContextManager — wire into Agent builder ──────────────────
    println!("=== Step 5: RagContextManager + Agent ===\n");

    // Build a fresh retriever for the manager
    let mut mgr_retriever = KeywordRetriever::new();
    for (i, text) in raw_corpus.iter().enumerate() {
        for (j, chunk) in chunker.chunk(text).iter().enumerate() {
            mgr_retriever.add(Document::new(
                format!("mgr-doc{}-chunk{}", i + 1, j + 1).as_str(),
                chunk,
            ));
        }
    }

    // RagContextManager auto-retrieves + injects context on every agent.run() call
    let rag_manager = RagContextManager::new(mgr_retriever)
        .with_grounding(CitationStrategy)
        .with_max_docs(3);

    let provider = OpenAiCompatProvider::openai(
        "gpt-4o-mini",
        std::env::var("OPENAI_API_KEY").unwrap_or_else(|_| "demo-key".into()),
    );

    let agent = Agent::builder()
        .provider(provider)
        .system("You are a Rust expert. Answer ONLY using the provided context.")
        .context_manager(rag_manager)
        .build()?;

    println!("  Running grounded agent with query: \"{query}\"\n");
    let response = agent.run(query).await?;
    println!("  🤖 Answer: {}\n", response.text());

    println!("✅ RAG pipeline v2 complete!");
    Ok(())
}
