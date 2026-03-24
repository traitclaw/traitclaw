# 07 — RAG Pipeline

Retrieval-Augmented Generation with built-in keyword search.

## What it does

1. Indexes documents into a **BM25 keyword retriever** (no vector DB needed)
2. Retrieves relevant documents for a query
3. Grounds the context using **PrependStrategy**
4. Injects retrieved context into the agent's system prompt

## Key APIs

```rust
let mut retriever = KeywordRetriever::new();
retriever.add(Document::new("id", "content..."));

let docs = retriever.retrieve("query", 3).await?;
let context = PrependStrategy.ground(&docs);
```

## Running

```bash
export OPENAI_API_KEY="sk-..."  # optional
cargo run
```
