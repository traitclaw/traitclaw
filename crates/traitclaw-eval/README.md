# traitclaw-eval

[![crates.io](https://img.shields.io/crates/v/traitclaw-eval.svg)](https://crates.io/crates/traitclaw-eval)
[![docs.rs](https://docs.rs/traitclaw-eval/badge.svg)](https://docs.rs/traitclaw-eval)

**Evaluation framework for TraitClaw — test suites, metrics, and quality reports for AI agents.**

Measure agent quality with structured test cases and pluggable metrics. Includes built-in keyword matching and length-relevancy scoring. Run evaluations deterministically without hitting LLM APIs.

## Usage

```rust
use traitclaw_eval::{EvalSuite, TestCase, KeywordMetric, LengthRelevancyMetric, Metric};

// Define test cases
let suite = EvalSuite::new("quality_tests")
    .add_case(
        TestCase::new("greeting", "Say hello")
            .expect_contains("hello")
            .expect_contains("help"),
    )
    .add_case(
        TestCase::new("math", "What is 2+2?")
            .expect_contains("4"),
    );

// Score with built-in metrics
let keyword_score = KeywordMetric.score("input", "Hello! How can I help?", &["hello", "help"]);
// → 1.0 (both keywords found)

let length_score = LengthRelevancyMetric.score("input", "response text", &[]);
// → 0.0..1.0 (penalizes too-short or too-long responses)
```

## Components

| Component | Purpose |
|-----------|---------|
| `EvalSuite` | Container for test cases |
| `TestCase` | Input prompt + expected keywords/output |
| `Metric` (trait) | Pluggable scoring function (0.0 → 1.0) |
| `KeywordMetric` | Fraction of expected keywords found in output |
| `LengthRelevancyMetric` | Penalizes responses outside 2-10x input length |
| `EvalReport` | Summary with pass/fail counts and average scores |
| `TestResult` | Per-test scores and pass/fail status |

## Custom Metrics

```rust
impl Metric for MyMetric {
    fn name(&self) -> &'static str { "my_metric" }
    fn score(&self, input: &str, output: &str, keywords: &[&str]) -> f64 {
        // Return 0.0 (worst) to 1.0 (best)
    }
}
```

## License

Licensed under either of [Apache License, Version 2.0](../../LICENSE-APACHE) or [MIT License](../../LICENSE-MIT) at your option.
