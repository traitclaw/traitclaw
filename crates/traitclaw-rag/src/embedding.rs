//! Embedding-based vector retrieval for RAG pipelines.
//!
//! Provides the [`EmbeddingProvider`] trait and [`EmbeddingRetriever`] —
//! an in-memory cosine-similarity retriever backed by any embedding model.
//!
//! # Example
//!
//! ```rust,no_run
//! use traitclaw_rag::embedding::{EmbeddingProvider, EmbeddingRetriever};
//! use traitclaw_rag::{Document, Retriever};
//! use async_trait::async_trait;
//!
//! struct MyEmbedder;
//!
//! #[async_trait]
//! impl EmbeddingProvider for MyEmbedder {
//!     async fn embed(&self, texts: &[&str]) -> traitclaw_core::Result<Vec<Vec<f64>>> {
//!         // Return dummy vectors of dimension 3 for each text
//!         Ok(texts.iter().map(|_| vec![0.1, 0.2, 0.3]).collect())
//!     }
//! }
//!
//! # async fn example() -> traitclaw_core::Result<()> {
//! let mut retriever = EmbeddingRetriever::new(MyEmbedder);
//! retriever.add_documents(vec![
//!     Document::new("doc1", "Rust systems programming"),
//!     Document::new("doc2", "Python data science"),
//! ]).await?;
//!
//! let results = retriever.retrieve("Rust", 1).await?;
//! assert_eq!(results.len(), 1);
//! # Ok(())
//! # }
//! ```

use async_trait::async_trait;
use traitclaw_core::{Error, Result};

use crate::{Document, Retriever};

/// Async trait for computing text embeddings.
///
/// Implement this to integrate any embedding model (OpenAI, Cohere, local, etc.).
#[async_trait]
pub trait EmbeddingProvider: Send + Sync + 'static {
    /// Compute embeddings for `texts`.
    ///
    /// Returns a vector of embeddings — one per input text.
    /// All embeddings must have the same dimensionality.
    async fn embed(&self, texts: &[&str]) -> Result<Vec<Vec<f64>>>;
}

/// Stored entry: embedding vector + original document.
struct VectorEntry {
    embedding: Vec<f64>,
    document: Document,
}

/// In-memory vector retriever using cosine similarity search.
///
/// Stores document embeddings and retrieves the top-k most similar documents
/// for a query, optionally filtered by a minimum similarity threshold.
pub struct EmbeddingRetriever<P: EmbeddingProvider> {
    provider: P,
    store: Vec<VectorEntry>,
    similarity_threshold: f64,
}

impl<P: EmbeddingProvider> EmbeddingRetriever<P> {
    /// Create a new retriever backed by the given [`EmbeddingProvider`].
    #[must_use]
    pub fn new(provider: P) -> Self {
        Self {
            provider,
            store: Vec::new(),
            similarity_threshold: 0.0,
        }
    }

    /// Set the minimum cosine similarity required to include a result.
    ///
    /// Results with similarity below this threshold are excluded.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use traitclaw_rag::embedding::{EmbeddingProvider, EmbeddingRetriever};
    /// # struct Dummy;
    /// # #[async_trait::async_trait]
    /// # impl EmbeddingProvider for Dummy {
    /// #     async fn embed(&self, texts: &[&str]) -> traitclaw_core::Result<Vec<Vec<f64>>> {
    /// #         Ok(vec![vec![0.0]; texts.len()])
    /// #     }
    /// # }
    /// let retriever = EmbeddingRetriever::new(Dummy).with_similarity_threshold(0.7);
    /// ```
    #[must_use]
    pub fn with_similarity_threshold(mut self, threshold: f64) -> Self {
        self.similarity_threshold = threshold;
        self
    }

    /// Embed and store documents in the in-memory vector store.
    ///
    /// Calls `embed()` exactly once with all document texts.
    ///
    /// # Errors
    ///
    /// Returns an error if the embedding provider fails or returns the wrong
    /// number of embeddings.
    pub async fn add_documents(&mut self, documents: Vec<Document>) -> Result<()> {
        if documents.is_empty() {
            return Ok(());
        }

        let texts: Vec<&str> = documents.iter().map(|d| d.content.as_str()).collect();
        let embeddings = self.provider.embed(&texts).await?;

        if embeddings.len() != documents.len() {
            return Err(Error::Runtime(format!(
                "EmbeddingProvider returned {} embeddings for {} documents",
                embeddings.len(),
                documents.len()
            )));
        }

        for (doc, emb) in documents.into_iter().zip(embeddings) {
            self.store.push(VectorEntry {
                embedding: emb,
                document: doc,
            });
        }

        Ok(())
    }

    /// Number of stored documents.
    #[must_use]
    pub fn len(&self) -> usize {
        self.store.len()
    }

    /// Whether the vector store is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.store.is_empty()
    }
}

#[async_trait]
impl<P: EmbeddingProvider> Retriever for EmbeddingRetriever<P> {
    /// Embed `query`, compute cosine similarity with all stored docs, return top-k.
    async fn retrieve(&self, query: &str, limit: usize) -> Result<Vec<Document>> {
        if self.store.is_empty() {
            return Ok(Vec::new());
        }

        let query_embs = self.provider.embed(&[query]).await?;
        let query_emb = query_embs
            .into_iter()
            .next()
            .ok_or_else(|| Error::Runtime("EmbeddingProvider returned empty for query".into()))?;

        let mut scored: Vec<(f64, &Document)> = self
            .store
            .iter()
            .map(|entry| {
                let sim = cosine_similarity(&query_emb, &entry.embedding);
                (sim, &entry.document)
            })
            .filter(|(sim, _)| *sim >= self.similarity_threshold)
            .collect();

        // Sort by similarity descending
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(limit);

        let results = scored
            .into_iter()
            .map(|(sim, doc)| {
                let mut d = doc.clone();
                d.score = sim;
                d
            })
            .collect();

        Ok(results)
    }
}

/// Compute cosine similarity between two vectors.
///
/// Returns 0.0 if either vector has zero magnitude.
#[allow(clippy::cast_precision_loss)]
fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let mag_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
    let mag_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();

    if mag_a == 0.0 || mag_b == 0.0 {
        return 0.0;
    }

    dot / (mag_a * mag_b)
}

// ─────────────────────────────────────────────────────────────────────────────
// Test helper: Counting embedder
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
pub(crate) mod test_helpers {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    use super::*;

    /// Tracking embedder: counts embed() calls and returns deterministic vectors.
    pub struct CountingEmbedder {
        pub call_count: Arc<AtomicUsize>,
        #[allow(dead_code)]
        pub dim: usize,
    }

    impl CountingEmbedder {
        pub fn new(dim: usize) -> Self {
            Self {
                call_count: Arc::new(AtomicUsize::new(0)),
                dim,
            }
        }
    }

    #[async_trait]
    impl EmbeddingProvider for CountingEmbedder {
        async fn embed(&self, texts: &[&str]) -> Result<Vec<Vec<f64>>> {
            self.call_count.fetch_add(1, Ordering::Relaxed);
            // Generate slightly different vectors per text (based on char count)
            Ok(texts
                .iter()
                .map(|t| {
                    let base = (t.len() % 10) as f64 / 10.0;
                    vec![base, 1.0 - base, 0.5]
                })
                .collect())
        }
    }

    /// Simple embedder that uses specific vectors for known texts.
    pub struct FixedEmbedder(pub Vec<Vec<f64>>);

    #[async_trait]
    impl EmbeddingProvider for FixedEmbedder {
        async fn embed(&self, texts: &[&str]) -> Result<Vec<Vec<f64>>> {
            // Returns embeddings cycling through the provided list
            Ok(texts
                .iter()
                .enumerate()
                .map(|(i, _)| self.0[i % self.0.len()].clone())
                .collect())
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use std::sync::atomic::Ordering;
    use std::sync::Arc;

    use super::test_helpers::*;
    use super::*;
    use crate::Document;

    fn make_docs(n: usize) -> Vec<Document> {
        (0..n)
            .map(|i| Document::new(format!("doc{i}"), format!("document content {i}")))
            .collect()
    }

    #[tokio::test]
    async fn test_add_documents_calls_embed_once() {
        // AC #9: add_documents calls embed() exactly once with all texts
        let embedder = CountingEmbedder::new(3);
        let count = embedder.call_count.clone();
        let mut retriever = EmbeddingRetriever::new(embedder);
        retriever.add_documents(make_docs(10)).await.unwrap();

        assert_eq!(
            count.load(Ordering::Relaxed),
            1,
            "embed should be called exactly once"
        );
        assert_eq!(retriever.len(), 10);
    }

    #[tokio::test]
    async fn test_retrieve_returns_at_most_limit() {
        // AC #7: 10 docs → query returns ≤ limit results
        let mut retriever = EmbeddingRetriever::new(CountingEmbedder::new(3));
        retriever.add_documents(make_docs(10)).await.unwrap();

        let results = retriever.retrieve("content", 3).await.unwrap();
        assert!(
            results.len() <= 3,
            "expected ≤3 results, got {}",
            results.len()
        );
    }

    #[tokio::test]
    async fn test_retrieve_sorted_by_similarity_desc() {
        // AC #7: results sorted by similarity descending
        let mut retriever = EmbeddingRetriever::new(CountingEmbedder::new(3));
        retriever.add_documents(make_docs(5)).await.unwrap();

        let results = retriever.retrieve("query", 5).await.unwrap();
        for window in results.windows(2) {
            assert!(
                window[0].score >= window[1].score,
                "results not sorted: {} < {}",
                window[0].score,
                window[1].score
            );
        }
    }

    #[tokio::test]
    async fn test_similarity_threshold_filters_results() {
        // AC #8: threshold 0.9 → fewer results than threshold 0.5
        let vecs = vec![
            vec![1.0, 0.0, 0.0], // identical to query → sim = 1.0
            vec![0.0, 1.0, 0.0], // orthogonal → sim = 0.0
            vec![0.7, 0.7, 0.0], // partial match → sim ≈ 0.49
        ];

        let mut retriever_low =
            EmbeddingRetriever::new(FixedEmbedder(vecs.clone())).with_similarity_threshold(0.0);
        retriever_low.add_documents(make_docs(3)).await.unwrap();
        let results_low = retriever_low.retrieve("doc", 10).await.unwrap();

        let mut retriever_high =
            EmbeddingRetriever::new(FixedEmbedder(vecs.clone())).with_similarity_threshold(0.95);
        retriever_high.add_documents(make_docs(3)).await.unwrap();
        let results_high = retriever_high.retrieve("doc", 10).await.unwrap();

        // High threshold → fewer results
        assert!(
            results_high.len() < results_low.len() || results_high.len() <= 1,
            "high threshold should filter more: low={}, high={}",
            results_low.len(),
            results_high.len()
        );
    }

    #[tokio::test]
    async fn test_empty_store_returns_empty() {
        let retriever = EmbeddingRetriever::new(CountingEmbedder::new(3));
        let results = retriever.retrieve("any query", 10).await.unwrap();
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn test_add_empty_documents() {
        let mut retriever = EmbeddingRetriever::new(CountingEmbedder::new(3));
        retriever.add_documents(vec![]).await.unwrap();
        assert!(retriever.is_empty());
    }

    #[test]
    fn test_cosine_similarity_identical() {
        let v = vec![1.0, 2.0, 3.0];
        let sim = cosine_similarity(&v, &v);
        assert!((sim - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        let sim = cosine_similarity(&a, &b);
        assert!(sim.abs() < 1e-9);
    }

    #[test]
    fn test_cosine_similarity_zero_vector() {
        let a = vec![0.0, 0.0];
        let b = vec![1.0, 0.0];
        assert!(cosine_similarity(&a, &b).abs() < f64::EPSILON);
    }

    #[test]
    fn test_embedding_retriever_is_retriever_trait_object() {
        // Can be used as Arc<dyn Retriever>
        let r = EmbeddingRetriever::new(CountingEmbedder::new(3));
        let _: Arc<dyn Retriever> = Arc::new(r);
    }
}
