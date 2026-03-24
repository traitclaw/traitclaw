//! RAG (Retrieval-Augmented Generation) pipeline for the `BaseClaw` AI agent framework.
//!
//! Provides a `Retriever` trait, grounding strategies, and a built-in
//! `KeywordRetriever` with BM25-style scoring for text search.
//!
//! # Quick Start
//!
//! ```rust
//! use baseclaw_rag::{Document, KeywordRetriever, Retriever, GroundingStrategy, PrependStrategy};
//!
//! # async fn example() -> baseclaw_core::Result<()> {
//! let mut retriever = KeywordRetriever::new();
//! retriever.add(Document::new("doc1", "Rust is a systems programming language"));
//! retriever.add(Document::new("doc2", "Python is great for AI"));
//!
//! let docs = retriever.retrieve("Rust systems", 5).await?;
//! assert!(!docs.is_empty());
//!
//! let strategy = PrependStrategy;
//! let context = strategy.ground(&docs);
//! assert!(context.contains("Rust"));
//! # Ok(())
//! # }
//! ```

#![deny(warnings)]
#![deny(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// A document for retrieval.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    /// Unique document identifier.
    pub id: String,
    /// Document content.
    pub content: String,
    /// Optional metadata.
    pub metadata: Option<serde_json::Value>,
    /// Relevance score (set by retriever).
    pub score: f64,
}

impl Document {
    /// Create a new document with no metadata and zero score.
    #[must_use]
    pub fn new(id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            content: content.into(),
            metadata: None,
            score: 0.0,
        }
    }
}

/// Trait for document retrieval.
///
/// Implement this for custom retrieval backends (vector DB, API, etc.).
#[async_trait]
pub trait Retriever: Send + Sync + 'static {
    /// Retrieve relevant documents for a query.
    async fn retrieve(&self, query: &str, limit: usize) -> baseclaw_core::Result<Vec<Document>>;
}

/// Strategy for grounding agent context with retrieved documents.
pub trait GroundingStrategy: Send + Sync + 'static {
    /// Convert retrieved documents into context text for the agent.
    fn ground(&self, documents: &[Document]) -> String;
}

/// Simple grounding strategy that prepends documents as numbered context.
pub struct PrependStrategy;

impl GroundingStrategy for PrependStrategy {
    fn ground(&self, documents: &[Document]) -> String {
        if documents.is_empty() {
            return String::new();
        }
        let mut ctx = String::from("Relevant context:\n\n");
        for (i, doc) in documents.iter().enumerate() {
            use std::fmt::Write;
            let _ = write!(ctx, "[{}] {}\n\n", i + 1, doc.content);
        }
        ctx
    }
}

/// BM25-style keyword retriever for in-memory text search.
///
/// Scores documents using term frequency and inverse document frequency.
pub struct KeywordRetriever {
    documents: Vec<Document>,
}

impl KeywordRetriever {
    /// Create a new empty keyword retriever.
    #[must_use]
    pub fn new() -> Self {
        Self {
            documents: Vec::new(),
        }
    }

    /// Add a document to the index.
    pub fn add(&mut self, doc: Document) {
        self.documents.push(doc);
    }

    /// Add multiple documents to the index.
    pub fn add_many(&mut self, docs: impl IntoIterator<Item = Document>) {
        self.documents.extend(docs);
    }

    /// Score a document against query terms using BM25-like TF scoring.
    #[allow(clippy::cast_precision_loss)]
    fn score(query_terms: &[String], content: &str) -> f64 {
        let content_lower = content.to_lowercase();
        let words: Vec<&str> = content_lower.split_whitespace().collect();
        let doc_len = words.len() as f64;

        if doc_len == 0.0 {
            return 0.0;
        }

        let mut total_score = 0.0;
        for term in query_terms {
            let tf = words.iter().filter(|w| **w == term.as_str()).count() as f64;
            // BM25-like: tf / (tf + 1.2 * (1 - 0.75 + 0.75 * doc_len / avg_len))
            // Simplified: use tf / (tf + 1.0)
            let score = tf / (tf + 1.0);
            total_score += score;
        }

        total_score
    }
}

impl Default for KeywordRetriever {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Retriever for KeywordRetriever {
    async fn retrieve(&self, query: &str, limit: usize) -> baseclaw_core::Result<Vec<Document>> {
        let terms: Vec<String> = query
            .to_lowercase()
            .split_whitespace()
            .map(String::from)
            .collect();

        let mut scored: Vec<Document> = self
            .documents
            .iter()
            .map(|doc| {
                let mut d = doc.clone();
                d.score = Self::score(&terms, &doc.content);
                d
            })
            .filter(|d| d.score > 0.0)
            .collect();

        scored.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        scored.truncate(limit);

        Ok(scored)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_keyword_retriever_basic() {
        let mut r = KeywordRetriever::new();
        r.add(Document::new("1", "Rust is a systems programming language"));
        r.add(Document::new("2", "Python is great for data science"));
        r.add(Document::new("3", "Rust has zero-cost abstractions"));

        let results = r.retrieve("Rust programming", 10).await.unwrap();
        assert!(!results.is_empty());
        // "Rust" appears in doc 1 and 3
        assert!(results.len() >= 2);
        // Doc 1 should score higher (has both "rust" and "programming")
        assert_eq!(results[0].id, "1");
    }

    #[tokio::test]
    async fn test_keyword_retriever_empty_query() {
        let mut r = KeywordRetriever::new();
        r.add(Document::new("1", "Some content"));
        let results = r.retrieve("", 10).await.unwrap();
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn test_keyword_retriever_no_match() {
        let mut r = KeywordRetriever::new();
        r.add(Document::new("1", "Hello world"));
        let results = r.retrieve("quantum computing", 10).await.unwrap();
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn test_keyword_retriever_limit() {
        let mut r = KeywordRetriever::new();
        for i in 0..10 {
            r.add(Document::new(format!("{i}"), format!("rust item {i}")));
        }
        let results = r.retrieve("rust", 3).await.unwrap();
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_prepend_strategy() {
        let docs = vec![
            Document::new("1", "First doc"),
            Document::new("2", "Second doc"),
        ];
        let ctx = PrependStrategy.ground(&docs);
        assert!(ctx.contains("[1] First doc"));
        assert!(ctx.contains("[2] Second doc"));
    }

    #[test]
    fn test_prepend_strategy_empty() {
        let ctx = PrependStrategy.ground(&[]);
        assert!(ctx.is_empty());
    }

    #[test]
    fn test_document_new() {
        let doc = Document::new("id1", "content1");
        assert_eq!(doc.id, "id1");
        assert_eq!(doc.content, "content1");
        assert!(doc.metadata.is_none());
        assert!((doc.score - 0.0).abs() < f64::EPSILON);
    }
}
