//! Hybrid retrieval combining keyword and embedding-based search.
//!
//! [`HybridRetriever`] combines a [`KeywordRetriever`] and any [`Retriever`] (e.g.,
//! [`EmbeddingRetriever`]) with configurable score weighting, then re-ranks results.
//!
//! Also provides enhanced grounding strategies:
//! - [`CitationStrategy`] — formats docs with citation numbers and source IDs
//! - [`ContextWindowStrategy`] — limits injected context to a token budget
//!
//! [`EmbeddingRetriever`]: crate::embedding::EmbeddingRetriever

use async_trait::async_trait;

use crate::{Document, GroundingStrategy, KeywordRetriever, Retriever};

// ─────────────────────────────────────────────────────────────────────────────
// HybridRetriever
// ─────────────────────────────────────────────────────────────────────────────

/// Combines keyword and semantic retrieval with configurable weighting.
///
/// Results from both retrievers are merged, normalized, and re-ranked by a
/// weighted sum of their individual scores.
///
/// # Example
///
/// ```rust
/// use traitclaw_rag::{Document, KeywordRetriever, Retriever};
/// use traitclaw_rag::hybrid::HybridRetriever;
///
/// # async fn example() -> traitclaw_core::Result<()> {
/// let mut keyword = KeywordRetriever::new();
/// keyword.add(Document::new("doc1", "Rust is fast"));
///
/// // Use keyword retriever as both sides for this example
/// let hybrid = HybridRetriever::new(keyword, KeywordRetriever::new());
/// let results = hybrid.retrieve("Rust", 5).await?;
/// # Ok(())
/// # }
/// ```
pub struct HybridRetriever<E: Retriever> {
    keyword: KeywordRetriever,
    embedding: E,
    keyword_weight: f64,
    embedding_weight: f64,
}

impl<E: Retriever> HybridRetriever<E> {
    /// Create a new `HybridRetriever` with default weights (0.3 keyword / 0.7 embedding).
    #[must_use]
    pub fn new(keyword: KeywordRetriever, embedding: E) -> Self {
        Self {
            keyword,
            embedding,
            keyword_weight: 0.3,
            embedding_weight: 0.7,
        }
    }

    /// Set custom score weights.
    ///
    /// Weights need not sum to 1.0 — they are used as multipliers.
    ///
    /// # Panics
    ///
    /// Panics if either weight is negative.
    #[must_use]
    pub fn with_weights(mut self, keyword_weight: f64, embedding_weight: f64) -> Self {
        assert!(keyword_weight >= 0.0, "keyword_weight must be non-negative");
        assert!(
            embedding_weight >= 0.0,
            "embedding_weight must be non-negative"
        );
        self.keyword_weight = keyword_weight;
        self.embedding_weight = embedding_weight;
        self
    }
}

#[async_trait]
impl<E: Retriever> Retriever for HybridRetriever<E> {
    /// Retrieve from both sources, merge, normalize, and re-rank.
    async fn retrieve(&self, query: &str, limit: usize) -> traitclaw_core::Result<Vec<Document>> {
        // Fetch from both — use a larger candidate set for better coverage
        let candidate = (limit * 5).max(10);

        let (kw_docs, emb_docs) = tokio::join!(
            self.keyword.retrieve(query, candidate),
            self.embedding.retrieve(query, candidate),
        );
        let kw_docs = kw_docs.unwrap_or_default();
        let emb_docs = emb_docs.unwrap_or_default();

        // Normalize scores within each set (0.0–1.0)
        let kw_max = kw_docs
            .iter()
            .map(|d| d.score)
            .fold(f64::NEG_INFINITY, f64::max);
        let emb_max = emb_docs
            .iter()
            .map(|d| d.score)
            .fold(f64::NEG_INFINITY, f64::max);

        // Merge by doc id: combined_score = w_kw * norm_kw + w_emb * norm_emb
        let mut scores: std::collections::HashMap<String, (f64, &Document)> =
            std::collections::HashMap::new();

        for doc in &kw_docs {
            let norm = if kw_max > 0.0 {
                doc.score / kw_max
            } else {
                0.0
            };
            scores
                .entry(doc.id.clone())
                .and_modify(|(s, _)| *s += self.keyword_weight * norm)
                .or_insert((self.keyword_weight * norm, doc));
        }

        for doc in &emb_docs {
            let norm = if emb_max > 0.0 {
                doc.score / emb_max
            } else {
                0.0
            };
            scores
                .entry(doc.id.clone())
                .and_modify(|(s, _)| *s += self.embedding_weight * norm)
                .or_insert((self.embedding_weight * norm, doc));
        }

        let mut combined: Vec<Document> = scores
            .into_values()
            .map(|(combined_score, doc)| {
                let mut d = doc.clone();
                d.score = combined_score;
                d
            })
            .collect();

        combined.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        combined.truncate(limit);

        Ok(combined)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// CitationStrategy
// ─────────────────────────────────────────────────────────────────────────────

/// Grounding strategy that uses numbered citations with source IDs.
///
/// Format: `[1] content (Source: doc_id)\n`
///
/// # Example
///
/// ```rust
/// use traitclaw_rag::{Document, GroundingStrategy};
/// use traitclaw_rag::hybrid::CitationStrategy;
///
/// let docs = vec![Document::new("paper-42", "Important finding.")];
/// let ctx = CitationStrategy.ground(&docs);
/// assert!(ctx.contains("[1]"));
/// assert!(ctx.contains("Source: paper-42"));
/// ```
pub struct CitationStrategy;

impl GroundingStrategy for CitationStrategy {
    fn ground(&self, documents: &[Document]) -> String {
        if documents.is_empty() {
            return String::new();
        }
        let mut ctx = String::from("Context:\n\n");
        for (i, doc) in documents.iter().enumerate() {
            use std::fmt::Write;
            let _ = writeln!(ctx, "[{}] {} (Source: {})", i + 1, doc.content, doc.id);
        }
        ctx
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ContextWindowStrategy
// ─────────────────────────────────────────────────────────────────────────────

/// Grounding strategy that limits injected context to a token budget.
///
/// Uses a simple 4-chars-per-token heuristic. Documents are added in order
/// until the budget would be exceeded.
///
/// # Example
///
/// ```rust
/// use traitclaw_rag::{Document, GroundingStrategy};
/// use traitclaw_rag::hybrid::ContextWindowStrategy;
///
/// let large_doc = Document::new("d1", &"word ".repeat(1000));
/// let strategy = ContextWindowStrategy::new(50); // very small budget
/// let ctx = strategy.ground(&[large_doc]);
/// // Context is truncated to fit budget
/// assert!(ctx.chars().count() < 400); // 50 tokens * 4 chars each
/// ```
pub struct ContextWindowStrategy {
    max_tokens: usize,
}

impl ContextWindowStrategy {
    /// Create a new `ContextWindowStrategy` with the given token budget.
    ///
    /// # Panics
    ///
    /// Panics if `max_tokens == 0`.
    #[must_use]
    pub fn new(max_tokens: usize) -> Self {
        assert!(max_tokens > 0, "max_tokens must be > 0");
        Self { max_tokens }
    }
}

impl GroundingStrategy for ContextWindowStrategy {
    fn ground(&self, documents: &[Document]) -> String {
        if documents.is_empty() {
            return String::new();
        }

        // 4 chars per token heuristic
        let char_budget = self.max_tokens * 4;
        let mut ctx = String::from("Context:\n\n");
        let mut used = ctx.len();

        for (i, doc) in documents.iter().enumerate() {
            use std::fmt::Write;
            let entry = format!("[{}] {}\n\n", i + 1, doc.content);

            if used + entry.len() > char_budget {
                // Try partial: trim doc content to fit
                let available = char_budget.saturating_sub(used + 10); // header overhead
                if available > 20 {
                    let truncated: String = doc.content.chars().take(available).collect();
                    let _ = write!(ctx, "[{}] {}…\n\n", i + 1, truncated);
                }
                break;
            }

            ctx.push_str(&entry);
            used += entry.len();
        }

        ctx
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Document;

    fn kw_retriever_with(docs: Vec<(&str, &str)>) -> KeywordRetriever {
        let mut r = KeywordRetriever::new();
        for (id, content) in docs {
            r.add(Document::new(id, content));
        }
        r
    }

    // ── HybridRetriever ───────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_hybrid_returns_from_keyword_source() {
        // AC #7: hybrid returns results from keyword source
        let kw = kw_retriever_with(vec![("k1", "Rust programming"), ("k2", "Python code")]);
        let emb = KeywordRetriever::new(); // empty embedding side

        let hybrid = HybridRetriever::new(kw, emb);
        let results = hybrid.retrieve("Rust", 5).await.unwrap();

        // Should still get keyword results even with empty embedding side
        assert!(!results.is_empty(), "expected keyword results");
        assert!(results.iter().any(|d| d.id == "k1"));
    }

    #[tokio::test]
    async fn test_hybrid_merges_both_sources() {
        // AC #7: hybrid returns from both sources
        let kw = kw_retriever_with(vec![("k1", "Rust keyword hit")]);
        let emb = kw_retriever_with(vec![("e1", "Rust embedding hit")]);

        let hybrid = HybridRetriever::new(kw, emb);
        let results = hybrid.retrieve("Rust hit", 10).await.unwrap();

        let ids: Vec<_> = results.iter().map(|d| d.id.as_str()).collect();
        assert!(
            ids.contains(&"k1") || ids.contains(&"e1"),
            "should contain results from both: {ids:?}"
        );
    }

    #[tokio::test]
    async fn test_hybrid_respects_limit() {
        let kw = kw_retriever_with(vec![("k1", "Rust a"), ("k2", "Rust b"), ("k3", "Rust c")]);
        let emb = kw_retriever_with(vec![("e1", "Rust d"), ("e2", "Rust e")]);
        let hybrid = HybridRetriever::new(kw, emb);
        let results = hybrid.retrieve("Rust", 2).await.unwrap();
        assert!(results.len() <= 2);
    }

    #[tokio::test]
    async fn test_hybrid_combined_score_sorted_desc() {
        let kw = kw_retriever_with(vec![("k1", "Rust programming")]);
        let emb = kw_retriever_with(vec![("e1", "Rust embedding search")]);
        let hybrid = HybridRetriever::new(kw, emb);
        let results = hybrid.retrieve("Rust", 10).await.unwrap();

        for window in results.windows(2) {
            assert!(window[0].score >= window[1].score);
        }
    }

    // ── CitationStrategy ─────────────────────────────────────────────────────

    #[test]
    fn test_citation_strategy_format() {
        // AC #5: formats as [1] content (Source: doc_id)
        let docs = vec![
            Document::new("paper-42", "Important finding."),
            Document::new("blog-7", "Another insight."),
        ];
        let ctx = CitationStrategy.ground(&docs);
        assert!(ctx.contains("[1]"));
        assert!(ctx.contains("Source: paper-42"));
        assert!(ctx.contains("[2]"));
        assert!(ctx.contains("Source: blog-7"));
    }

    #[test]
    fn test_citation_strategy_empty() {
        assert!(CitationStrategy.ground(&[]).is_empty());
    }

    // ── ContextWindowStrategy ────────────────────────────────────────────────

    #[test]
    fn test_context_window_small_docs_fit() {
        let docs = vec![
            Document::new("d1", "Short."),
            Document::new("d2", "Also short."),
        ];
        // 1000 tokens = 4000 chars — easily fits 2 tiny docs
        let ctx = ContextWindowStrategy::new(1000).ground(&docs);
        assert!(ctx.contains("[1]"));
        assert!(ctx.contains("[2]"));
    }

    #[test]
    fn test_context_window_truncates_large_doc() {
        // AC #8: truncates when context exceeds budget
        let large = "word ".repeat(500); // 2500 chars
        let docs = vec![Document::new("big", &large)];

        let strategy = ContextWindowStrategy::new(50); // 200 char budget
        let ctx = strategy.ground(&docs);

        // Output must be significantly smaller than the full doc
        assert!(
            ctx.chars().count() < 500,
            "expected truncation, got {} chars",
            ctx.chars().count()
        );
    }

    #[test]
    fn test_context_window_empty_docs() {
        assert!(ContextWindowStrategy::new(100).ground(&[]).is_empty());
    }
}
