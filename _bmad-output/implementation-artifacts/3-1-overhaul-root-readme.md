# Story 3.1: Overhaul Root README

Status: done

## Story

As a **developer evaluating TraitClaw for the first time**,
I want a professional README with badges, quickstart, and feature overview,
so that I can quickly understand what TraitClaw offers and how to get started.

## Acceptance Criteria

1. README contains badges: crates.io version, docs.rs, CI status, license, MSRV
2. One-liner: "A Rust AI Agent Framework — Simple by default, powerful when needed"
3. "Why TraitClaw?" section with key differentiators
4. Quickstart code block (5-line agent creation)
5. Feature Matrix table with ✅/🔜 status
6. Architecture Overview showing layered trait system
7. Examples section listing all 23+ examples with descriptions
8. Roadmap section with v1.1–v1.3 plans and codenames (FR18, FR19)
9. License section stating dual MIT/Apache-2.0
10. README renders correctly on GitHub
11. README passes markdown lint validation (no broken links, no syntax issues)
12. README renders correctly as crates.io README (no broken relative links)

## Tasks / Subtasks

- [ ] Task 1 (AC: #1-2): Header and badges
  - [ ] Add badge row: crates.io, docs.rs, CI, license, MSRV
  - [ ] Add one-liner description
- [ ] Task 2 (AC: #3): Why TraitClaw? section
  - [ ] Composable trait architecture
  - [ ] Type-safe tool calling via `#[derive(Tool)]`
  - [ ] Multi-strategy reasoning (ReAct, CoT, MCTS)
  - [ ] Production-grade observability via tracing
- [ ] Task 3 (AC: #4): Quickstart code block
  - [ ] Simple 5-line agent with provider + tool + run
- [ ] Task 4 (AC: #5): Feature Matrix
  - [ ] Core Agent & Builder ✅
  - [ ] 8 Core Traits ✅
  - [ ] Tool System + `#[derive(Tool)]` ✅
  - [ ] Reasoning Strategies ✅
  - [ ] Multi-Agent Teams ✅
  - [ ] 3 Providers ✅
  - [ ] Memory (InMemory, SQLite, Compressed) ✅
  - [ ] Observability ✅
  - [ ] Benchmarks 🔜 v1.1
  - [ ] Orchestration Strategy 🔜 v1.1
  - [ ] Inter-agent Contracts 🔜 v1.2
  - [ ] Retry/Checkpoint 🔜 v1.3
- [ ] Task 5 (AC: #6): Architecture Overview
  - [ ] Layered diagram: Core Traits → Implementations → Meta-Crate
- [ ] Task 6 (AC: #7): Examples section
  - [ ] List all 23 examples with 1-line descriptions
- [ ] Task 7 (AC: #8): Roadmap section
  - [ ] v1.1.0 "Benchmark & Orchestrate"
  - [ ] v1.2.0 "Contracts"
  - [ ] v1.3.0 "Resilience"
- [ ] Task 8 (AC: #9): License section
- [ ] Task 9 (AC: #10-12): Verification
  - [ ] Preview on GitHub
  - [ ] Check for broken relative links (all links must be absolute for crates.io)

## Dev Notes

- Current README exists at `/Users/admin/Desktop/Projects/traitclaw/README.md` (~9.8KB)
- Badge format examples:
  ```markdown
  [![Crates.io](https://img.shields.io/crates/v/traitclaw.svg)](https://crates.io/crates/traitclaw)
  [![docs.rs](https://docs.rs/traitclaw/badge.svg)](https://docs.rs/traitclaw)
  [![License](https://img.shields.io/crates/l/traitclaw.svg)](LICENSE-MIT)
  [![MSRV](https://img.shields.io/badge/MSRV-1.75-blue.svg)]()
  ```
- **CRITICAL**: crates.io does NOT resolve relative links. All links to repo files must be absolute: `https://github.com/traitclaw/traitclaw/tree/main/examples/...`
- After creating root README, copy to `crates/traitclaw/README.md` (Story 2.2 dependency)

### References

- [Source: prd-v1.0.0.md#FR4, FR18, FR19]
- [Source: architecture-v1.0.0.md#AD4 — README Strategy]
- [Source: epics-v1.0.0.md#Story 3.1]
