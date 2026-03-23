//! Tool output processing — pluggable post-processing of tool results.
//!
//! By default, [`TruncateProcessor`] limits tool output to 10,000 characters.

/// Trait for processing tool output before adding it to the message context.
///
/// Implementations MUST be sync (no I/O) and fast.
pub trait OutputProcessor: Send + Sync {
    /// Process a tool output string and return the (possibly modified) result.
    fn process(&self, output: String) -> String;
}

/// Returns output unchanged.
pub struct NoopProcessor;

impl OutputProcessor for NoopProcessor {
    fn process(&self, output: String) -> String {
        output
    }
}

/// Truncates output exceeding `max_chars` with a `"...\n[output truncated]"` suffix.
pub struct TruncateProcessor {
    /// Maximum number of characters before truncation.
    max_chars: usize,
}

impl Default for TruncateProcessor {
    fn default() -> Self {
        Self { max_chars: 10_000 }
    }
}

impl TruncateProcessor {
    /// Create a processor with a custom character limit.
    #[must_use]
    pub fn new(max_chars: usize) -> Self {
        Self { max_chars }
    }
}

impl OutputProcessor for TruncateProcessor {
    fn process(&self, output: String) -> String {
        // Count characters, not bytes, to avoid panicking on multi-byte UTF-8.
        let char_count = output.chars().count();
        if char_count <= self.max_chars {
            output
        } else {
            // Find the byte offset of the Nth character boundary.
            let byte_offset = output
                .char_indices()
                .nth(self.max_chars)
                .map_or(output.len(), |(idx, _)| idx);
            let mut truncated = output[..byte_offset].to_string();
            truncated.push_str("...\n[output truncated]");
            truncated
        }
    }
}

/// Composes multiple processors in order — output flows through each stage.
pub struct ChainProcessor {
    processors: Vec<Box<dyn OutputProcessor>>,
}

impl ChainProcessor {
    /// Create a chain from a list of processors.
    #[must_use]
    pub fn new(processors: Vec<Box<dyn OutputProcessor>>) -> Self {
        Self { processors }
    }
}

impl OutputProcessor for ChainProcessor {
    fn process(&self, mut output: String) -> String {
        for p in &self.processors {
            output = p.process(output);
        }
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_noop_returns_unchanged() {
        let p = NoopProcessor;
        let input = "hello world".to_string();
        assert_eq!(p.process(input.clone()), input);
    }

    #[test]
    fn test_truncate_at_boundary() {
        let p = TruncateProcessor::new(10);
        let short = "12345".to_string();
        assert_eq!(
            p.process(short.clone()),
            short,
            "should not truncate short input"
        );

        let exact = "1234567890".to_string();
        assert_eq!(
            p.process(exact.clone()),
            exact,
            "should not truncate exact-length input"
        );

        let long = "12345678901".to_string();
        let result = p.process(long);
        assert!(
            result.ends_with("[output truncated]"),
            "should truncate: {result}"
        );
        assert!(
            result.starts_with("1234567890"),
            "should keep first 10 chars"
        );
    }

    #[test]
    fn test_chain_applies_in_order() {
        struct UpperCase;
        impl OutputProcessor for UpperCase {
            fn process(&self, output: String) -> String {
                output.to_uppercase()
            }
        }

        let chain = ChainProcessor::new(vec![
            Box::new(UpperCase),
            Box::new(TruncateProcessor::new(5)),
        ]);

        let result = chain.process("hello world".to_string());
        // First: "HELLO WORLD", then truncate to 5
        assert!(result.starts_with("HELLO"), "got: {result}");
        assert!(result.contains("[output truncated]"));
    }
}
