---
stepsCompleted: ["step-01-validate-prerequisites", "step-02-design-epics", "step-03-create-stories", "step-04-final-validation"]
inputDocuments:
  - planning-artifacts/prd-v0.4.0.md
  - planning-artifacts/prd-v0.3.0.md
  - project-context.md
---

# TraitClaw v0.4.0 — Epic Breakdown

## Overview

This document provides the complete epic and story breakdown for TraitClaw v0.4.0 "Power Tools", decomposing the requirements from the PRD into implementable stories. All features implement existing traits from v0.3.0 — no new traits are introduced.

## Requirements Inventory

### Functional Requirements

FR1: Implement `GroupedRegistry` in `traitclaw-core` with named tool groups, group-level activation/deactivation, multi-group support, and `RwLock`-based interior mutability.

FR2: Implement `AdaptiveRegistry` in `traitclaw-core` that auto-limits active tools based on `ModelTier` (Small=5, Medium=15, Large=unlimited) with configurable limits and priority-based tool selection.

FR3: Implement `ProgressiveTransformer` in `traitclaw-core` with two-phase output processing: LLM-generated summary first, full output available via virtual tool on demand. Accepts any `Provider` for summarization.

FR4: Implement `TikTokenCounter` in `traitclaw-core` behind `"tiktoken"` feature flag using `tiktoken-rs`. Maps model names to encodings with `cl100k_base` fallback.

FR5: Implement `McpToolRegistry` in `traitclaw-mcp` that discovers and registers tools from MCP servers, maps MCP schemas to TraitClaw `ToolSchema`, routes execution through MCP protocol, and supports auto-reconnection.

FR6: Create examples (`19-grouped-registry`, `20-mcp-tool-registry`) and migration guide (`docs/migration-v0.3-to-v0.4.md`).

### Non-Functional Requirements

NFR1: Zero overhead on default path — `SimpleRegistry` remains the default. New registries only used when explicitly configured.

NFR2: Backward compatibility — All v0.3.0 code must compile and run without modification on v0.4.0. No breaking changes.

NFR3: Feature-gated dependencies — `tiktoken-rs` behind `"tiktoken"` flag. MCP deps already in `traitclaw-mcp`.

NFR4: `RwLock` overhead in `GroupedRegistry` — read lock < 1ns. Write lock only during activate/deactivate.

NFR5: MSRV Rust 1.75+.

### FR Coverage Map

| FR | Epic | Description |
|----|------|-------------|
| FR1 | Epic 13: Advanced Tool Registries | GroupedRegistry |
| FR2 | Epic 13: Advanced Tool Registries | AdaptiveRegistry |
| FR3 | Epic 14: Progressive Output Processing | ProgressiveTransformer |
| FR4 | Epic 14: Progressive Output Processing | TikTokenCounter |
| FR5 | Epic 15: MCP Tool Discovery | McpToolRegistry |
| FR6 | Epic 16: Documentation & Examples | Migration guide + examples |

## Epic List

- **Epic 13: Advanced Tool Registries** — GroupedRegistry and AdaptiveRegistry implementations (Phase 1)
- **Epic 14: Progressive Output Processing** — ProgressiveTransformer and TikTokenCounter (Phase 2)
- **Epic 15: MCP Tool Discovery** — McpToolRegistry with MCP protocol integration (Phase 3)
- **Epic 16: Documentation & Examples** — Migration guide, examples, polish (Phase 4)

> **Note:** Epic numbering continues from v0.3.0 (Epics 8-12) for sprint tracking consistency.

---

## Epic 13: Advanced Tool Registries

**Goal:** Developers can organize tools into named groups and auto-limit tool counts by model tier, reducing schema overhead for agents with many tools.

**FRs:** FR1, FR2 | **NFRs:** NFR1, NFR2, NFR4

### Story 13.1: GroupedRegistry Implementation

As a developer with 30+ tools,
I want to organize them into named groups and activate/deactivate groups at runtime,
So that only relevant tool schemas are sent to the LLM, saving tokens.

**Acceptance Criteria:**

**Given** `GroupedRegistry` is implemented in `traitclaw-core`
**When** I create a grouped registry with `.group("name", tools)`
**Then** it implements the `ToolRegistry` trait from v0.3.0
**And** `.activate("name")` activates a group, `.deactivate("name")` deactivates it
**And** multiple groups can be active simultaneously
**And** `active_schemas()` returns only schemas from active groups
**And** `active_tools()` returns only tools from active groups
**And** `find(name)` searches across all groups (active or not)
**And** interior mutability via `RwLock` enables runtime group switching
**And** unit test: activate group A → deactivate group A → activate group B verifies correct schemas
**And** unit test: concurrent read access during group switch is safe
**And** re-exported via `traitclaw::prelude::*`

### Story 13.2: AdaptiveRegistry Implementation

As a developer deploying across different model tiers,
I want tools automatically limited based on model capabilities,
So that small models aren't overwhelmed by too many tool schemas.

**Acceptance Criteria:**

**Given** `AdaptiveRegistry` is implemented in `traitclaw-core`
**When** I create with `AdaptiveRegistry::new(tools)`
**Then** it implements the `ToolRegistry` trait from v0.3.0
**And** default limits: Small=5, Medium=15, Large=unlimited
**And** `.with_limits(small, medium, large)` allows custom limits
**And** `active_schemas()` returns at most `limit` schemas based on current `ModelTier`
**And** tools registered first have higher priority (first registered = first selected)
**And** `ModelTier` is read from `ModelInfo` via the configured `Provider`
**And** unit test: verify Small tier returns exactly 5 tools from a 30-tool set
**And** unit test: verify Large tier returns all tools
**And** re-exported via `traitclaw::prelude::*`

---

## Epic 14: Progressive Output Processing

**Goal:** Developers can reduce context waste from large tool outputs through progressive disclosure and accurate token counting.

**FRs:** FR3, FR4 | **NFRs:** NFR1, NFR2, NFR3

### Story 14.1: ProgressiveTransformer Implementation

As a developer whose tools return large outputs,
I want outputs summarized first with full content available on demand,
So that most interactions save significant tokens.

**Acceptance Criteria:**

**Given** `ProgressiveTransformer` is implemented in `traitclaw-core`
**When** a tool produces output exceeding `max_summary_length`
**Then** it implements the `OutputTransformer` trait from v0.3.0
**And** Phase 1: LLM-generated summary (configurable max length) is returned to the agent
**And** Phase 2: full output is cached and available via virtual `__get_full_output` tool
**And** `ProgressiveTransformer::new(provider, max_length)` accepts any `Provider`
**And** `.with_summary_prompt("...")` allows custom summarization prompt
**And** if LLM summarization fails, falls back to character truncation
**And** if output is shorter than `max_summary_length`, output is passed through unchanged
**And** unit test: large output → summary returned, `__get_full_output` returns original
**And** unit test: short output → passed through without LLM call
**And** unit test: LLM failure → graceful fallback to truncation
**And** re-exported via `traitclaw::prelude::*`

### Story 14.2: TikTokenCounter Implementation

As a developer who needs accurate token counting,
I want exact OpenAI-compatible tokenization,
So that context budget decisions are precise.

**Acceptance Criteria:**

**Given** `TikTokenCounter` is implemented in `traitclaw-core` behind `"tiktoken"` feature flag
**When** I enable `traitclaw = { features = ["tiktoken"] }`
**Then** `TikTokenCounter::for_model("gpt-4o")` creates a counter with the correct encoding
**And** `count_tokens(&[Message])` returns exact token count
**And** unknown models fall back to `cl100k_base` encoding
**And** can be used as the `estimate_tokens()` override in `ContextManager` implementations
**And** `tiktoken-rs` is an optional dependency (not pulled without feature flag)
**And** without feature flag, compilation succeeds and `TikTokenCounter` type is not available
**And** accuracy test: CharApprox vs TikToken on 100 sample messages shows < 2% error for English
**And** re-exported via `traitclaw::prelude::*` when feature is enabled

---

## Epic 15: MCP Tool Discovery

**Goal:** Developers can discover and use tools from MCP-compatible servers with zero manual tool definition, enabling seamless integration with the growing MCP ecosystem.

**FRs:** FR5 | **NFRs:** NFR2, NFR3

### Story 15.1: McpToolRegistry Core Implementation

As a developer using MCP-compatible tool servers,
I want to auto-discover and register tools from MCP servers,
So that I can use any MCP tool without manual schema definition.

**Acceptance Criteria:**

**Given** `McpToolRegistry` is implemented in `traitclaw-mcp`
**When** I call `McpToolRegistry::connect("http://localhost:3000").await`
**Then** it implements the `ToolRegistry` trait from v0.3.0
**And** tools are discovered from the MCP server via `tools/list` method
**And** MCP tool schemas are mapped to TraitClaw `ToolSchema` format
**And** `find(name)` returns an MCP-backed tool that routes execution through MCP protocol
**And** `active_schemas()` returns all discovered tool schemas
**And** integration test with mock MCP server: discover 5 tools, execute one, verify result

### Story 15.2: McpToolRegistry Resilience & Multi-Server

As a developer using multiple MCP servers,
I want resilient connections and multi-server support,
So that my agent can access tools from multiple MCP sources reliably.

**Acceptance Criteria:**

**Given** `McpToolRegistry` supports multiple servers
**When** I connect to multiple servers via `.add_server("http://...")`
**Then** tools from all servers are aggregated in a single registry
**And** tool names are prefixed with server name to avoid collisions if configured
**And** auto-reconnection on connection loss with configurable retry policy
**And** if a server is unreachable, other servers' tools remain available
**And** `.refresh()` re-discovers tools from all connected servers
**And** unit test: server disconnect → reconnect → tools re-available
**And** unit test: 2 servers with overlapping tool names → no collision

---

## Epic 16: Documentation & Examples

**Goal:** Developers can upgrade from v0.3.0 with zero code changes and learn new features through guided examples.

**FRs:** FR6 | **NFRs:** NFR2

### Story 16.1: Migration Guide v0.3 → v0.4

As a developer upgrading from v0.3.0,
I want a clear migration guide,
So that I understand what's new and how to adopt features.

**Acceptance Criteria:**

**Given** `docs/migration-v0.3-to-v0.4.md` is created
**When** I read the guide
**Then** it confirms "No breaking changes. v0.3.0 code compiles unchanged."
**And** it lists all new implementations with code snippets
**And** it shows `GroupedRegistry` usage with before/after comparison
**And** it shows `TikTokenCounter` feature flag enablement

### Story 16.2: Grouped Registry Example

As a developer,
I want `examples/19-grouped-registry/` demonstrating tool group management,
So that I can learn how to organize and switch tool groups.

**Acceptance Criteria:**

**Given** `examples/19-grouped-registry/` is created
**When** I run the example
**Then** it demonstrates `GroupedRegistry` with 3 groups, activating/switching between them
**And** console output shows which tool schemas are active per group
**And** example compiles and runs successfully

### Story 16.3: MCP Tool Registry Example

As a developer,
I want `examples/20-mcp-tool-registry/` demonstrating MCP tool discovery,
So that I can learn how to connect to MCP servers.

**Acceptance Criteria:**

**Given** `examples/20-mcp-tool-registry/` is created
**When** I run the example
**Then** it demonstrates `McpToolRegistry` connecting to a local MCP server
**And** it discovers tools and shows their schemas
**And** example compiles with `cargo build` (runtime requires MCP server)

---

## Implementation Order (Dependency Graph)

```
Phase 1 (Week 1-2):
  Epic 13: Story 13.1 → 13.2

Phase 2 (Week 3-4):
  Epic 14: Story 14.1 → 14.2

Phase 3 (Week 5-6):
  Epic 15: Story 15.1 → 15.2

Phase 4 (Week 7-8):
  Epic 16: Stories 16.1, 16.2, 16.3 (parallel)
```

**Critical Path:** Epics 13 and 14 are independent and could run in parallel. Epic 15 depends on existing `traitclaw-mcp` patterns. Epic 16 depends on all prior epics.

**Total:** 4 epics, 9 stories
