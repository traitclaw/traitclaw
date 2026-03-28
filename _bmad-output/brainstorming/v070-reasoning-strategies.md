---
stepsCompleted: [1]
session_topic: 'TraitClaw v0.7.0 Scope and Direction'
session_goals: 'Define v0.7.0 theme, features, and architecture decisions'
---

# TraitClaw v0.7.0 "Reasoning" — Brainstorming Session

**Date:** 2026-03-28
**Participants:** Bangvu + AI Facilitator

## Session Overview

**Topic:** Defining the scope and direction for TraitClaw v0.7.0
**Goals:** Determine what features to include, crate structure, and feature flag strategy

## Key Decisions

### Theme: "Reasoning" — Built-in Strategies

v0.7.0 follows the natural progression of TraitClaw:
- v0.5.0 made crates *runnable*
- v0.6.0 made composition *effortless*
- v0.7.0 makes agents *smarter* with built-in reasoning strategies

### Scope — IN v0.7.0

| Feature | Description | Crate |
|---------|-------------|-------|
| `ReActStrategy` | Think → Act → Observe reasoning loop with tool calling | `traitclaw-strategies` |
| `MctsStrategy` | Monte Carlo Tree Search for complex reasoning | `traitclaw-strategies` |
| `ChainOfThoughtStrategy` | Step-by-step reasoning injection | `traitclaw-strategies` |
| `StreamingOutputTransformer` | Streaming output transformation trait | `traitclaw-core` |
| Example upgrades | Upgrade `11-custom-strategy` + new per-strategy examples | examples/ |
| Migration guide | `docs/migration-v0.6-to-v0.7.md` | docs/ |

### Scope — DEFERRED to v0.8.0

| Feature | Reason |
|---------|--------|
| DAG execution engine | Large scope, independent theme |
| Config-driven agent spawn (YAML/TOML) | Better fits a "DX" themed release |

### Architecture Decisions

1. **New crate:** `traitclaw-strategies` — keeps `traitclaw-core` minimal
2. **Feature flags:** Default all-on (batteries-included), each strategy can be opted-out
3. **Philosophy:** Built-in everything, user chooses what to use

### Analysis: User Implementability

| Strategy | Difficulty for User | Built-in Value |
|----------|-------------------|----------------|
| ChainOfThought | 🟢 Trivial (prompt/hint would suffice) | Convenience + standardization |
| ReAct | 🟡 Medium (~80 LOC custom strategy) | Saves effort + battle-tested impl |
| MCTS | 🔴 Hard (~400 LOC, parallelism) | High value — genuinely hard to DIY |

**Decision:** Build all three regardless of difficulty. Philosophy is batteries-included — whether user uses them is their choice.

### Feature Flag Design

```toml
# Default: batteries-included (all strategies)
traitclaw-strategies = "0.7.0"

# Power user opt-out:
traitclaw-strategies = { version = "0.7.0", default-features = false, features = ["react"] }
```

## Next Steps

- [ ] Create PRD v0.7.0 (bmad-create-prd workflow)
- [ ] Create Epics and Stories
- [ ] Implementation
