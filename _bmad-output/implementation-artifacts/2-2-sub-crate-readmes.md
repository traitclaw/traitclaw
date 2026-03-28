# Story 2.2: Create Sub-Crate README Files

Status: done

## Story

As a **new user discovering a sub-crate on crates.io**,
I want each sub-crate to have a README explaining what it is and linking to the main project,
so that I can find the full documentation and examples.

## Acceptance Criteria

1. All 14 crates have a `README.md` in their directory
2. Each sub-crate README contains: crate name, description matching Cargo.toml, link to main crates.io page, link to docs.rs, link to examples, license section
3. Meta-crate `traitclaw` README is a copy of the root README
4. `cargo package --list -p <crate>` includes `README.md` for all crates

## Tasks / Subtasks

- [ ] Task 1 (AC: #1-2): Create sub-crate READMEs (13 crates)
  - [ ] `traitclaw-core/README.md`
  - [ ] `traitclaw-macros/README.md`
  - [ ] `traitclaw-test-utils/README.md`
  - [ ] `traitclaw-openai/README.md`
  - [ ] `traitclaw-anthropic/README.md`
  - [ ] `traitclaw-openai-compat/README.md`
  - [ ] `traitclaw-steering/README.md`
  - [ ] `traitclaw-memory-sqlite/README.md`
  - [ ] `traitclaw-mcp/README.md`
  - [ ] `traitclaw-rag/README.md`
  - [ ] `traitclaw-team/README.md`
  - [ ] `traitclaw-eval/README.md`
  - [ ] `traitclaw-strategies/README.md`
- [ ] Task 2 (AC: #3): Copy root README to meta-crate
  - [ ] Copy root `README.md` to `crates/traitclaw/README.md`
- [ ] Task 3 (AC: #4): Verify packaging
  - [ ] Run `cargo package --list -p <crate>` for all 14

## Dev Notes

### Sub-Crate README Template

```markdown
# traitclaw-{name}

{description from Cargo.toml}

This crate is part of the [TraitClaw](https://crates.io/crates/traitclaw) AI Agent Framework.

## Documentation

- [API Reference (docs.rs)](https://docs.rs/traitclaw-{name})
- [TraitClaw Guide](https://github.com/traitclaw/traitclaw)
- [Examples](https://github.com/traitclaw/traitclaw/tree/main/examples)

## License

Licensed under either of [Apache License, Version 2.0](../../LICENSE-APACHE) or [MIT license](../../LICENSE-MIT) at your option.
```

### Crate Descriptions (from existing Cargo.toml)

| Crate | Description |
|-------|-------------|
| traitclaw-core | Core traits, types, and runtime for the TraitClaw AI Agent Framework |
| traitclaw-macros | Procedural macros for the TraitClaw AI Agent Framework |
| traitclaw-openai | OpenAI provider for the TraitClaw AI Agent Framework |
| traitclaw-anthropic | Anthropic provider for the TraitClaw AI Agent Framework |
| traitclaw-openai-compat | OpenAI-compatible provider for the TraitClaw AI Agent Framework |
| traitclaw-steering | Guard, Hint, and Tracker implementations for TraitClaw |
| traitclaw-memory-sqlite | SQLite-backed persistent memory for TraitClaw |
| traitclaw-mcp | Model Context Protocol client for TraitClaw |
| traitclaw-rag | RAG pipeline traits and implementations for TraitClaw |
| traitclaw-team | Multi-agent team coordination for TraitClaw |
| traitclaw-eval | Agent evaluation framework for TraitClaw |
| traitclaw-strategies | Reasoning strategies (ReAct, CoT, MCTS) for TraitClaw |
| traitclaw-test-utils | Shared test mocks and helpers for TraitClaw |

**Note:** Descriptions may need verification — run `grep description crates/*/Cargo.toml` to get current values.

### References

- [Source: architecture-v1.0.0.md#P2 — Sub-Crate README Pattern]
- [Source: architecture-v1.0.0.md#AD4 — README Strategy]
