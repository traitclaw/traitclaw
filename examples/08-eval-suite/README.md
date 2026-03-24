# 08 — Eval Suite

Measure agent quality with built-in evaluation metrics.

## What it does

1. Defines test cases with expected keywords
2. Scores agent responses using **KeywordMetric** and **LengthRelevancyMetric**
3. Generates an evaluation report with pass/fail and average scores

## Key APIs

```rust
let suite = EvalSuite::new("quality")
    .add_case(TestCase::new("greeting", "Say hello")
        .expect_contains("hello"));

let score = KeywordMetric.score("input", "output", &["hello"]);
```

## Running

```bash
cargo run  # no API key needed — uses simulated responses
```
