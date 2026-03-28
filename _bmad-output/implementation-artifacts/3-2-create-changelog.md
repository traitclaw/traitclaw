# Story 3.2: Create CHANGELOG.md

Status: ready-for-dev

## Story

As a **user upgrading between versions**,
I want a comprehensive changelog covering all versions from v0.1 to v1.0,
so that I can understand what changed in each release.

## Acceptance Criteria

1. `CHANGELOG.md` exists at repository root
2. Follows [Keep a Changelog](https://keepachangelog.com/) format
3. Includes entries for: v1.0.0, v0.9.0, v0.8.0, v0.7.0, v0.6.0, v0.5.0, v0.4.0, v0.3.0, v0.2.0, v0.1.0
4. Each entry uses sections: Added, Changed, Removed, Fixed (as applicable)
5. v0.9.0 entry mentions breaking changes (ContextStrategy/OutputProcessor removed)
6. v1.0.0 entry notes API freeze and ecosystem packaging

## Tasks / Subtasks

- [ ] Task 1 (AC: #1-6): Create CHANGELOG.md
  - [ ] v1.0.0 — Added: README, CHANGELOG, LICENSE, CONTRIBUTING, crates.io metadata, roadmap. Changed: API frozen under semver.
  - [ ] v0.9.0 — Removed: ContextStrategy, OutputProcessor traits. Changed: prelude enriched, builder error messages standardized.
  - [ ] v0.8.0 — Added: BudgetAwareTruncator, TransformerChain, DynamicRegistry. Changed: Quality foundation improvements.
  - [ ] v0.7.0 — Added: traitclaw-strategies crate (ReAct, CoT, MCTS). Added: ThoughtStep observability.
  - [ ] v0.6.0 — Added: AgentFactory, AgentPool, multi-agent composition. Added: traitclaw-team crate.
  - [ ] v0.5.0 — Added: traitclaw-eval crate. Added: traitclaw-rag crate.
  - [ ] v0.4.0 — Added: traitclaw-mcp crate. Added: traitclaw-steering (Guard, Hint, Tracker).
  - [ ] v0.3.0 — Added: async ContextManager and OutputTransformer traits. Added: blanket impls for backward compat.
  - [ ] v0.2.0 — Added: streaming support. Added: CompressedMemory. Changed: DefaultStrategy error handling.
  - [ ] v0.1.0 — Initial release. Core traits, Agent, AgentBuilder, Provider, Tool system.

## Dev Notes

- Reconstruct history from git log, migration guides, and PRDs
- Dates can be approximate — use git tag/commit dates if available
- Keep entries concise — 3-5 bullet points per version
- Reference migration guides where they exist: `docs/migration-v0.7-to-v0.8.md`, `docs/migration-v0.8-to-v0.9.md`
- Format header: `## [1.0.0] - 2026-03-XX`

### Git History Reference

```
55ff7ef feat(core)!: remove deprecated ContextStrategy and OutputProcessor traits
15b4972 refactor(core): remove empty runtime module
d153128 feat(core): implement v0.8.0 Quality Foundation
a7c324c feat(strategies): add traitclaw-strategies v0.7.0 reasoning crate
```

### References

- [Source: architecture-v1.0.0.md#AD6 — CHANGELOG Format]
- [Source: prd-v1.0.0.md#FR5]
- [Source: docs/migration-v0.8-to-v0.9.md — v0.9.0 breaking changes]
