# traitclaw-rag

[![crates.io](https://img.shields.io/crates/v/traitclaw-rag.svg)](https://crates.io/crates/traitclaw-rag)
[![docs.rs](https://docs.rs/traitclaw-rag/badge.svg)](https://docs.rs/traitclaw-rag)

**RAG pipeline for TraitClaw — Retriever trait, grounding strategies, and BM25 keyword search.**

Build Retrieval-Augmented Generation pipelines in Rust with pluggable retrieval backends. Includes a built-in `KeywordRetriever` with BM25-style scoring so you can get started without any external vector database.

## Usage

```rust
use traitclaw_rag::{Document, KeywordRetriever, Retriever, GroundingStrategy, PrependStrategy};

// 1. Index documents
let mut retriever = KeywordRetriever::new();
retriever.add(Document::new("doc1", "Rust is a systems programming language"));
retriever.add(Document::new("doc2", "Python is great for data science"));

// 2. Retrieve relevant documents
let docs = retriever.retrieve("Rust systems", 3).await?;

// 3. Ground context for the agent
let context = PrependStrategy.ground(&docs);
// → "Relevant context:\n\n[1] Rust is a systems..."
```

## Architecture

```
Query → Retriever → Documents → GroundingStrategy → Context → Agent
```

| Component | Description |
|-----------|-------------|
| `Document` | Content with ID, metadata, and relevance score |
| `Retriever` (trait) | Pluggable retrieval backend — implement for vector DBs, APIs, etc. |
| `KeywordRetriever` | Built-in BM25-style keyword matching (no external deps) |
| `GroundingStrategy` (trait) | Converts retrieved documents into agent context |
| `PrependStrategy` | Numbers and prepends documents as context |

## Custom Retrievers

Implement the `Retriever` trait for your backend:

```rust
#[async_trait]
impl Retriever for MyVectorDb {
    async fn retrieve(&self, query: &str, limit: usize) -> Result<Vec<Document>> {
        // Query your vector database (Qdrant, Pinecone, pgvector, etc.)
    }
}
```

## License

Licensed under either of [Apache License, Version 2.0](../../LICENSE-APACHE) or [MIT License](../../LICENSE-MIT) at your option.
