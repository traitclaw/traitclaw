# Migration Guide: TraitClaw v0.4 → v0.5

This guide covers breaking changes, new features, and migration steps for upgrading from TraitClaw v0.4.0 to v0.5.0 "Ecosystem".

---

## Breaking Changes

### 1. `Metric` trait is now async (`traitclaw-eval`)

The `Metric` trait has been replaced by `AsyncMetric`. All custom metric implementations must be updated.

**Before (v0.4):**

```rust
pub trait Metric: Send + Sync {
    fn name(&self) -> &str;
    fn evaluate(&self, case: &TestCase) -> f64;
}
```

**After (v0.5):**

```rust
#[async_trait]
pub trait AsyncMetric: Send + Sync {
    fn name(&self) -> &str;
    async fn evaluate(&self, case: &TestCase) -> f64;
}
```

**Migration:** If your metric is synchronous, wrap it with `SyncMetricAdapter`:

```rust
use traitclaw_eval::{AsyncMetric, SyncMetricAdapter};

// Your old synchronous metric:
struct MyMetric;
impl Metric for MyMetric {
    fn name(&self) -> &str { "my_metric" }
    fn evaluate(&self, case: &TestCase) -> f64 { 1.0 }
}

// Wrap it — no logic changes required:
let metric = SyncMetricAdapter(MyMetric);
```

---

## New Features

### RAG Pipeline (`traitclaw-rag`)

```rust
use traitclaw_rag::{
    Document, KeywordRetriever, HybridRetriever, RagContextManager,
    chunker::{RecursiveChunker, Chunker},
};

// Chunk documents for better retrieval
let chunker = RecursiveChunker::new(512, 64);
let chunks = chunker.chunk("Long document text...");

// Build hybrid retriever (keyword + embedding)
let mut retriever = KeywordRetriever::new();
retriever.add(Document::new("doc-1", "Rust ownership rules..."));

let docs = retriever.retrieve("ownership", 3).await?;

// Auto-ground agent context with RagContextManager
let rag_manager = RagContextManager::new(retriever, 3, PrependStrategy);
// Plug into AgentBuilder: .context_manager(rag_manager)
```

### Multi-Agent Teams (`traitclaw-team`)

```rust
use traitclaw_team::{TeamRunner, TeamContext, run_verification_chain};

// Sequential pipeline
let team = TeamRunner::builder()
    .agent("researcher", researcher_agent)
    .agent("writer", writer_agent)
    .build();

let result = team.run("Research and write about async Rust").await?;

// Verification loop (generate → verify → retry)
let result = run_verification_chain(
    &generator,
    &verifier,
    "Draft a summary",
    3, // max retries
).await?;
```

### RetryPolicy (`traitclaw-openai-compat`)

```rust
use traitclaw_openai_compat::RetryPolicy;
use std::time::Duration;

let policy = RetryPolicy::exponential(3, Duration::from_millis(500))
    .with_jitter()
    .with_max_delay(Duration::from_secs(30));

// Retryable status codes: 429, 500, 502, 503, 504
assert!(RetryPolicy::is_retryable(429));
```

### New Provider Constructors (`traitclaw-openai`)

```rust
use traitclaw_openai::{deepseek, xai};

// DeepSeek — reads DEEPSEEK_API_KEY
let provider = deepseek("deepseek-chat");

// xAI Grok — reads XAI_API_KEY
let provider = xai("grok-2");
```

### Anthropic Extended Thinking (`traitclaw-anthropic`)

```rust
use traitclaw_anthropic::AnthropicProvider;

let provider = AnthropicProvider::new("claude-3-7-sonnet-20250219", api_key)
    .with_extended_thinking(true);       // default 10,000 token budget
    // or:
    // .with_thinking_budget(20_000);   // custom budget
```

### Steering Guards (`traitclaw-steering`)

```rust
use traitclaw_steering::guards::{RateLimitGuard, ContentFilterGuard};

// Limit to 60 calls/minute (rolling window)
let rate = RateLimitGuard::new(60);

// Block PII (email, phone, SSN) + custom patterns
let filter = ContentFilterGuard::new()
    .with_custom_patterns(vec!["(?i)secret_key"]);
```

---

## Dependency Changes

Add to your `Cargo.toml` for new features:

```toml
[dependencies]
traitclaw-rag  = { version = "0.5" }    # RAG pipeline
traitclaw-team = { version = "0.5" }    # Multi-agent teams
traitclaw-eval = { version = "0.5" }    # Async eval runner
```

---

## Summary Table

| Feature | v0.4 | v0.5 |
|---|---|---|
| Eval `Metric` trait | sync | **async** |
| RAG chunking | ❌ | ✅ `FixedSize`, `Sentence`, `Recursive` |
| RAG hybrid retrieval | keyword only | ✅ keyword + embedding |
| Multi-agent teams | basic | ✅ `TeamRunner`, `VerificationChain` |
| Retry backoff | ❌ | ✅ `RetryPolicy` |
| DeepSeek / xAI | ❌ | ✅ `deepseek()`, `xai()` |
| Extended thinking | ❌ | ✅ `with_extended_thinking()` |
| Rate limit guard | ❌ | ✅ `RateLimitGuard` |
| Content filter guard | ❌ | ✅ `ContentFilterGuard` |
