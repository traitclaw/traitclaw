//! Document chunking strategies for RAG pipelines.
//!
//! Chunkers split large documents into smaller segments suitable for embedding
//! and retrieval. Each chunker implements the [`Chunker`] trait.
//!
//! # Chunkers
//!
//! - [`FixedSizeChunker`] — splits by character count with configurable overlap
//! - [`SentenceChunker`] — splits on sentence boundaries (`.`, `!`, `?`)
//! - [`RecursiveChunker`] — hierarchical: paragraph → sentence → character fallback

/// Trait for all document chunking strategies.
///
/// # Example
///
/// ```rust
/// use traitclaw_rag::chunker::{Chunker, FixedSizeChunker};
///
/// let chunker = FixedSizeChunker::new(100, 20);
/// let chunks = chunker.chunk("A long piece of text...");
/// assert!(!chunks.is_empty());
/// ```
pub trait Chunker: Send + Sync + 'static {
    /// Split `text` into a list of chunks.
    ///
    /// Returns an empty `Vec` for empty input.
    fn chunk(&self, text: &str) -> Vec<String>;
}

// ─────────────────────────────────────────────────────────────────────────────
// FixedSizeChunker
// ─────────────────────────────────────────────────────────────────────────────

/// Splits text by character count with configurable overlap.
///
/// # Example
///
/// ```rust
/// use traitclaw_rag::chunker::{Chunker, FixedSizeChunker};
///
/// let chunker = FixedSizeChunker::new(200, 50);
/// let text = "a".repeat(500);
/// let chunks = chunker.chunk(&text);
/// assert!(chunks.len() >= 3);
/// ```
pub struct FixedSizeChunker {
    chunk_size: usize,
    overlap: usize,
}

impl FixedSizeChunker {
    /// Create a new `FixedSizeChunker`.
    ///
    /// # Panics
    ///
    /// Panics if `overlap >= chunk_size`.
    #[must_use]
    pub fn new(chunk_size: usize, overlap: usize) -> Self {
        assert!(
            overlap < chunk_size,
            "overlap ({overlap}) must be less than chunk_size ({chunk_size})"
        );
        Self {
            chunk_size,
            overlap,
        }
    }
}

impl Chunker for FixedSizeChunker {
    fn chunk(&self, text: &str) -> Vec<String> {
        if text.is_empty() {
            return Vec::new();
        }

        let chars: Vec<char> = text.chars().collect();
        let total = chars.len();

        if total <= self.chunk_size {
            return vec![text.to_string()];
        }

        let step = self.chunk_size - self.overlap;
        let mut chunks = Vec::new();
        let mut start = 0;

        while start < total {
            let end = (start + self.chunk_size).min(total);
            let chunk: String = chars[start..end].iter().collect();
            chunks.push(chunk);
            if end == total {
                break;
            }
            start += step;
        }

        chunks
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// SentenceChunker
// ─────────────────────────────────────────────────────────────────────────────

/// Splits text on sentence boundaries into fixed-size sentence groups.
///
/// Sentence delimiters: `.`, `!`, `?`. Trailing whitespace is trimmed.
///
/// # Example
///
/// ```rust
/// use traitclaw_rag::chunker::{Chunker, SentenceChunker};
///
/// let chunker = SentenceChunker::new(3);
/// let text = "Sentence one. Sentence two. Sentence three. Sentence four.";
/// let chunks = chunker.chunk(text);
/// assert_eq!(chunks.len(), 2);
/// ```
pub struct SentenceChunker {
    sentences_per_chunk: usize,
}

impl SentenceChunker {
    /// Create a new `SentenceChunker` with `sentences_per_chunk` sentences per chunk.
    ///
    /// # Panics
    ///
    /// Panics if `sentences_per_chunk == 0`.
    #[must_use]
    pub fn new(sentences_per_chunk: usize) -> Self {
        assert!(sentences_per_chunk > 0, "sentences_per_chunk must be > 0");
        Self {
            sentences_per_chunk,
        }
    }

    /// Split text into individual sentences using `.`, `!`, `?` as delimiters.
    fn split_sentences(text: &str) -> Vec<String> {
        let mut sentences = Vec::new();
        let mut current = String::new();

        for ch in text.chars() {
            current.push(ch);
            if matches!(ch, '.' | '!' | '?') {
                let s = current.trim().to_string();
                if !s.is_empty() {
                    sentences.push(s);
                }
                current.clear();
            }
        }

        // Any remainder that doesn't end with punctuation
        let remainder = current.trim().to_string();
        if !remainder.is_empty() {
            sentences.push(remainder);
        }

        sentences
    }
}

impl Chunker for SentenceChunker {
    fn chunk(&self, text: &str) -> Vec<String> {
        if text.is_empty() {
            return Vec::new();
        }

        let sentences = Self::split_sentences(text);
        if sentences.is_empty() {
            return Vec::new();
        }

        sentences
            .chunks(self.sentences_per_chunk)
            .map(|window| window.join(" "))
            .collect()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// RecursiveChunker
// ─────────────────────────────────────────────────────────────────────────────

/// Hierarchical chunker: tries paragraphs → sentences → fixed-char fallback.
///
/// - First splits by `\n\n` (paragraphs)
/// - If a paragraph is still larger than `max_chunk_size`, splits by sentences
/// - If a sentence is still too large, uses fixed-size character splitting
///
/// # Example
///
/// ```rust
/// use traitclaw_rag::chunker::{Chunker, RecursiveChunker};
///
/// let chunker = RecursiveChunker::new(200);
/// let text = "Para 1.\n\nPara 2 sentence one. Para 2 sentence two.";
/// let chunks = chunker.chunk(text);
/// assert!(!chunks.is_empty());
/// ```
pub struct RecursiveChunker {
    max_chunk_size: usize,
}

impl RecursiveChunker {
    /// Create a new `RecursiveChunker` with the given max chunk size in characters.
    ///
    /// # Panics
    ///
    /// Panics if `max_chunk_size == 0`.
    #[must_use]
    pub fn new(max_chunk_size: usize) -> Self {
        assert!(max_chunk_size > 0, "max_chunk_size must be > 0");
        Self { max_chunk_size }
    }

    fn split_by_level(text: &str, max: usize) -> Vec<String> {
        let mut result = Vec::new();

        // Level 1: split by paragraph
        for para in text.split("\n\n") {
            let para = para.trim();
            if para.is_empty() {
                continue;
            }

            if para.chars().count() <= max {
                result.push(para.to_string());
            } else {
                // Level 2: split by sentence
                let mut sentence_buf = String::new();
                for sentence in SentenceChunker::split_sentences(para) {
                    if sentence_buf.chars().count() + sentence.chars().count() + 1 <= max {
                        if !sentence_buf.is_empty() {
                            sentence_buf.push(' ');
                        }
                        sentence_buf.push_str(&sentence);
                    } else {
                        if !sentence_buf.is_empty() {
                            // Flush existing buffer
                            if sentence_buf.chars().count() <= max {
                                result.push(sentence_buf.clone());
                            } else {
                                // Level 3: fixed-char fallback
                                let fixed = FixedSizeChunker::new(max, 0);
                                result.extend(fixed.chunk(&sentence_buf));
                            }
                            sentence_buf.clear();
                        }
                        // Start new buffer with current sentence
                        sentence_buf = sentence;
                    }
                }
                if !sentence_buf.is_empty() {
                    if sentence_buf.chars().count() <= max {
                        result.push(sentence_buf);
                    } else {
                        let fixed = FixedSizeChunker::new(max, 0);
                        result.extend(fixed.chunk(&sentence_buf));
                    }
                }
            }
        }

        result
    }
}

impl Chunker for RecursiveChunker {
    fn chunk(&self, text: &str) -> Vec<String> {
        if text.is_empty() {
            return Vec::new();
        }
        Self::split_by_level(text, self.max_chunk_size)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── FixedSizeChunker ──────────────────────────────────────────────────────

    #[test]
    fn test_fixed_size_empty_input() {
        // AC #8: empty input → all chunkers return empty Vec
        let c = FixedSizeChunker::new(200, 50);
        assert!(c.chunk("").is_empty());
    }

    #[test]
    fn test_fixed_size_short_text() {
        let c = FixedSizeChunker::new(200, 50);
        let chunks = c.chunk("short");
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], "short");
    }

    #[test]
    fn test_fixed_size_produces_overlap() {
        // AC #6: 1000-char text → FixedSizeChunker(200, 50) produces ≥ 5 chunks with overlap
        let text = "a".repeat(1000);
        let c = FixedSizeChunker::new(200, 50);
        let chunks = c.chunk(&text);
        assert!(
            chunks.len() >= 5,
            "expected >= 5 chunks, got {}",
            chunks.len()
        );

        // Verify overlap: end of chunk[0] == start of chunk[1] (first 50 chars)
        let end_of_first: String = chunks[0]
            .chars()
            .rev()
            .take(50)
            .collect::<String>()
            .chars()
            .rev()
            .collect();
        let start_of_second: String = chunks[1].chars().take(50).collect();
        assert_eq!(end_of_first, start_of_second, "overlap not maintained");
    }

    #[test]
    fn test_fixed_size_each_chunk_not_exceeds_size() {
        let text = "x".repeat(500);
        let c = FixedSizeChunker::new(100, 25);
        for chunk in c.chunk(&text) {
            assert!(chunk.chars().count() <= 100);
        }
    }

    // ── SentenceChunker ───────────────────────────────────────────────────────

    #[test]
    fn test_sentence_chunker_empty() {
        // AC #8
        let c = SentenceChunker::new(3);
        assert!(c.chunk("").is_empty());
    }

    #[test]
    fn test_sentence_chunker_10_sentences_gives_4_chunks() {
        // AC #7: 10-sentence text → SentenceChunker(3) produces 4 chunks (3+3+3+1)
        let sents: Vec<String> = (1..=10).map(|i| format!("Sentence {i}.")).collect();
        let text = sents.join(" ");
        let c = SentenceChunker::new(3);
        let chunks = c.chunk(&text);
        // 10 sentences / 3 per chunk = ceil(10/3) = 4 chunks
        assert_eq!(
            chunks.len(),
            4,
            "expected 4 chunks, got {}: {:?}",
            chunks.len(),
            chunks
        );
    }

    #[test]
    fn test_sentence_chunker_single() {
        let c = SentenceChunker::new(3);
        let chunks = c.chunk("One sentence.");
        assert_eq!(chunks.len(), 1);
    }

    #[test]
    fn test_sentence_chunker_exclamation_question() {
        let c = SentenceChunker::new(2);
        let chunks = c.chunk("Hello! How are you? I'm fine.");
        assert_eq!(chunks.len(), 2);
    }

    // ── RecursiveChunker ─────────────────────────────────────────────────────

    #[test]
    fn test_recursive_chunker_empty() {
        // AC #8
        let c = RecursiveChunker::new(200);
        assert!(c.chunk("").is_empty());
    }

    #[test]
    fn test_recursive_chunker_paragraph_split() {
        let text = "Short paragraph one.\n\nShort paragraph two.";
        let c = RecursiveChunker::new(200);
        let chunks = c.chunk(text);
        assert_eq!(chunks.len(), 2);
    }

    #[test]
    fn test_recursive_chunker_long_paragraph_splits_to_sentences() {
        // A paragraph larger than max_chunk_size → falls back to sentence splitting
        let long_sentence = "word ".repeat(20); // 100 chars
        let text = format!(
            "{}. {}. {}.",
            long_sentence.trim(),
            long_sentence.trim(),
            long_sentence.trim()
        );
        let c = RecursiveChunker::new(110); // max 110 chars, sentences are ~100 chars
        let chunks = c.chunk(&text);
        assert!(
            chunks.len() >= 2,
            "expected multiple chunks for long paragraph"
        );
    }

    #[test]
    fn test_recursive_chunker_each_chunk_within_limit() {
        let long_text = format!("word. {}", "sentence text here. ".repeat(50));
        let c = RecursiveChunker::new(100);
        for chunk in c.chunk(&long_text) {
            assert!(
                chunk.chars().count() <= 100,
                "chunk exceeds max_chunk_size: {} chars",
                chunk.chars().count()
            );
        }
    }
}
