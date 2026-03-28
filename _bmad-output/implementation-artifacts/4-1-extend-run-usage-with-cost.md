# Story 4.1: Extend `RunUsage` with Cost Estimation

Status: ready-for-dev

## Story

As an agent developer,
I want `RunUsage` to include `estimated_cost_usd` after an agent run,
so that I know exactly how much each agent invocation costs.

## Acceptance Criteria

1. `RunUsage` has new field: `estimated_cost_usd: f64` (defaults to `0.0`)
2. `ModelPricing` struct with `prompt_per_1m_tokens: f64` and `completion_per_1m_tokens: f64`
3. `AgentBuilder::with_pricing(table: HashMap<String, ModelPricing>)` sets pricing table
4. `DefaultStrategy` accumulates token usage per LLM call and computes cost at run end
5. If model name not found in pricing table, cost is `0.0` with `tracing::warn!`
6. Existing code using `RunUsage` continues to work (field defaults to `0.0`)
7. `ModelPricing` derives `Debug`, `Clone` and is `Send + Sync`
8. `default_pricing()` provides a built-in table for popular models
9. All existing tests pass

## Tasks / Subtasks

- [ ] Task 1: Create pricing types (AC: #2, #7)
  - [ ] Create `crates/traitclaw-core/src/types/pricing.rs`
  - [ ] Define `ModelPricing { pub prompt_per_1m_tokens: f64, pub completion_per_1m_tokens: f64 }`
  - [ ] `#[derive(Debug, Clone)]`
  - [ ] Doc comments with example pricing values
- [ ] Task 2: Extend `RunUsage` (AC: #1, #6)
  - [ ] Add `pub estimated_cost_usd: f64` to `RunUsage` struct (already `#[derive(Default)]` so `f64` defaults to `0.0`)
  - [ ] Existing `RunUsage::default()` still works — no breaking change
- [ ] Task 3: Add pricing to builder (AC: #3)
  - [ ] Add `pricing_table: Option<HashMap<String, ModelPricing>>` to `AgentBuilder`
  - [ ] Add `with_pricing()` method
  - [ ] Thread pricing through to `Agent` and `AgentRuntime`
- [ ] Task 4: Add pricing to `AgentRuntime` (AC: #4)
  - [ ] Add `pricing_table: Option<HashMap<String, ModelPricing>>` to `AgentRuntime`
  - [ ] Update `Agent::to_runtime()` and `stream_with_session()`
- [ ] Task 5: Implement cost calculation in `DefaultStrategy` (AC: #4, #5)
  - [ ] After each `provider.complete()`, accumulate `prompt_tokens` and `completion_tokens`
  - [ ] At run end, compute cost from pricing table
  - [ ] If model not found: `tracing::warn!(model = %name, "No pricing for model, cost will be 0.0")`
  - [ ] Set `usage.estimated_cost_usd = calculated_cost`
- [ ] Task 6: Default pricing table (AC: #8)
  - [ ] `pub fn default_pricing() -> HashMap<String, ModelPricing>`
  - [ ] Include: `gpt-4o`, `gpt-4o-mini`, `gpt-3.5-turbo`, `claude-3-5-sonnet`, `claude-3-5-haiku`
  - [ ] Prices based on current API pricing (as of March 2026)
- [ ] Task 7: Module registration and re-exports
  - [ ] Add `pub mod pricing;` to `types.rs`
  - [ ] Re-export `ModelPricing` and `default_pricing` from `lib.rs` and `prelude`
- [ ] Task 8: Tests (AC: #9)
  - [ ] Existing tests pass: `cargo test --workspace`
  - [ ] Test: `RunUsage::default().estimated_cost_usd == 0.0`
  - [ ] Test: cost calculation with known pricing → expected value
  - [ ] Test: unknown model → cost = 0.0

## Dev Notes

### RunUsage Modification

```rust
// BEFORE (agent.rs):
#[derive(Debug, Clone, Default)]
pub struct RunUsage {
    pub tokens: usize,
    pub iterations: usize,
    pub duration: std::time::Duration,
}

// AFTER:
#[derive(Debug, Clone, Default)]
pub struct RunUsage {
    pub tokens: usize,
    pub iterations: usize,
    pub duration: std::time::Duration,
    /// Estimated cost in USD based on model pricing. `0.0` if no pricing configured.
    pub estimated_cost_usd: f64,
}
```

This is backward compatible because:
- `RunUsage` is already `#[non_exhaustive]` (via `AgentOutput`)
- `Default` for `f64` is `0.0`
- Existing construction via struct literal adds `..Default::default()` or now needs the field

### Cost Calculation

```rust
fn calculate_cost(
    model: &str,
    prompt_tokens: u32,
    completion_tokens: u32,
    pricing: &Option<HashMap<String, ModelPricing>>,
) -> f64 {
    let Some(table) = pricing else { return 0.0 };
    let Some(price) = table.get(model) else {
        tracing::warn!(model = %model, "No pricing for model, cost will be 0.0");
        return 0.0;
    };
    (prompt_tokens as f64 / 1_000_000.0) * price.prompt_per_1m_tokens
        + (completion_tokens as f64 / 1_000_000.0) * price.completion_per_1m_tokens
}
```

### Default Pricing Example

```rust
pub fn default_pricing() -> HashMap<String, ModelPricing> {
    let mut m = HashMap::new();
    m.insert("gpt-4o".into(), ModelPricing { prompt_per_1m_tokens: 2.50, completion_per_1m_tokens: 10.00 });
    m.insert("gpt-4o-mini".into(), ModelPricing { prompt_per_1m_tokens: 0.15, completion_per_1m_tokens: 0.60 });
    m.insert("claude-3-5-sonnet-20241022".into(), ModelPricing { prompt_per_1m_tokens: 3.00, completion_per_1m_tokens: 15.00 });
    m
}
```

### References

- [crates/traitclaw-core/src/agent.rs](file:///Users/admin/Desktop/Projects/traitclaw/crates/traitclaw-core/src/agent.rs) — RunUsage struct (line 28)
- [crates/traitclaw-core/src/default_strategy.rs](file:///Users/admin/Desktop/Projects/traitclaw/crates/traitclaw-core/src/default_strategy.rs) — usage construction (line 118)
- [_bmad-output/planning-artifacts/architecture-v0.8.0.md](file:///Users/admin/Desktop/Projects/traitclaw/_bmad-output/planning-artifacts/architecture-v0.8.0.md) — Decision 5: RunUsage Cost Estimation
- FR20-FR22 in PRD v0.8.0

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
