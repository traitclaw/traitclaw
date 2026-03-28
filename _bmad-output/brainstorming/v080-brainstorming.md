---
stepsCompleted: [1, 2, 3]
inputDocuments:
  - architecture.md
  - project-context.md
  - brainstorming/v070-reasoning-strategies.md
  - brainstorming/brainstorming-session-2026-03-23-210618.md
session_topic: 'TraitClaw v0.8.0+ Long-Term Roadmap'
session_goals: 'Align with original vision, create clear versioned roadmap to v1.0, then build real agent'
selected_approach: 'AI-Recommended + Progressive Flow'
techniques_used: ['vision-alignment-audit', 'gap-analysis', 'progressive-roadmap', 'reverse-engineering-from-goal']
ideas_generated: []
context_file: ''
---

# TraitClaw — Long-Term Roadmap: Framework → Agent

**Date:** 2026-03-28
**Participants:** Bangvu + AI Facilitator

---

## The Big Picture

```
╔══════════════════════════════════════════════════════════════════╗
║                                                                  ║
║   Phase A: LIBRARY                Phase B: APPLICATION           ║
║   ─────────────────               ─────────────────              ║
║                                                                  ║
║   TraitClaw v0.8 → v1.0          "Agent App" (working agent)    ║
║   ┌──────────────────┐           ┌───────────────────────┐      ║
║   │  ⚙️ Framework     │           │  🤖 Real AI Agent      │      ║
║   │                  │           │                       │      ║
║   │  Traits, Runtime │──build──▶│  CLI/TUI/Web agent    │      ║
║   │  Providers, Tools│  on top  │  File ops, Shell      │      ║
║   │  Memory, Strategy│           │  Code gen, Search     │      ║
║   │  Observability   │           │  Multi-agent teams    │      ║
║   └──────────────────┘           └───────────────────────┘      ║
║                                                                  ║
║   "Tokio for async"              "Axum built on Tokio"          ║
║   "TraitClaw for agents"         "Agent built on TraitClaw"     ║
║                                                                  ║
╚══════════════════════════════════════════════════════════════════╝
```

---

## 1. Vision Alignment Review

### Core Insight từ brainstorm gốc (2026-03-23)

> **"TraitClaw nên tập trung vào CORE AGENT PRIMITIVES, không cố làm platform all-in-one.
> Giống như `tokio` cho async — TraitClaw là foundation mà mọi người build trên."**

### Phần "Không phải fundamental" (đã loại từ đầu)

- ❌ Channel integration (WhatsApp, Telegram) → application layer
- ❌ Web dashboard → tooling layer
- ❌ Multi-tenancy → deployment layer
- ❌ Voice/TTS → media adapter

### Câu hỏi then chốt: Framework cần gì để build được agent thật?

Reverse-engineering từ goal "build OpenClaw-like agent" → framework cần:

| Thứ agent cần | Framework component | Status |
|----------------|---------------------|--------|
| Gọi LLM | Provider trait + impls | ✅ Có |
| Gọi tools (file, shell, search) | Tool trait + derive macro | ✅ Có |
| Nhớ conversation | Memory trait + SQLite | ✅ Có |
| Stream output realtime | Streaming + SSE | ✅ Có |
| Suy luận phức tạp | Strategy (ReAct/CoT/MCTS) | ✅ Có |
| Dùng MCP tools | MCP client | ✅ Có |
| Multi-agent routing | Team + Router | ✅ Có |
| Context window management | ContextManager + Compressor | ✅ Có |
| Phát hiện lỗi agent | Guard + Hint + Tracker | ✅ Có |
| **Debug agent behavior** | **Observability/Tracing** | ❌ Thiếu |
| **Test agent quality** | Eval framework (advanced) | ⚠️ Cơ bản |
| **API ổn định** | **Semver v1.0 guarantee** | ❌ Chưa |

**Kết luận:** Framework đã có 90% primitives! Chỉ thiếu:
1. **Observability** — debug agent behavior khi develop agent app
2. **API stability** — guarantee không breaking changes

---

## 2. Trimmed Roadmap — Đường Ngắn Nhất Đến v1.0

### Triết lý mới: YAGNI for framework, value for agent

> Chỉ thêm vào framework cái gì **thật sự cần** để build agent.
> Không thêm vì "nice to have" hoặc "competitors có".

### Bỏ gì khỏi roadmap pre-v1.0?

| Feature | Verdict | Lý do |
|---------|---------|-------|
| `traitclaw-workflow` (DAG engine) | 🔴 **Defer post-v1.0** | Agent app có thể dùng ReAct + Team, chưa cần DAG |
| `traitclaw-server` (HTTP/WS) | 🔴 **Defer post-v1.0** | Agent app ban đầu là CLI, server là deployment concern |
| `traitclaw-google` (Gemini provider) | 🔴 **Defer** | `openai-compat` đã cover Gemini |
| `traitclaw-memory-pg` (PostgreSQL) | 🔴 **Defer** | SQLite đủ cho agent app |
| `traitclaw-memory-redis` | 🔴 **Defer** | SQLite đủ cho agent app |
| A2A protocol | 🔴 **Defer** | Protocol còn quá mới |
| Config-driven agents (YAML) | 🔴 **Defer** | Code-first tốt hơn cho Rust devs |

### Giữ gì cho pre-v1.0?

| Feature | Verdict | Lý do |
|---------|---------|-------|
| Testing infrastructure | 🟢 **v0.8.0** | Shared test crate, CI pipeline, coverage |
| Observability (OTEL tracing) | 🟢 **v0.8.0** | Cần để debug agent behavior khi dev |
| API audit + cleanup | 🟢 **v0.9.0** | Prerequisite cho v1.0 |
| Comprehensive testing | 🟢 **v0.9.0** | Fill coverage gaps, property-based tests |
| v1.0.0 stable release | 🟢 **v1.0** | Gate để bắt đầu agent app |

---

## 3. Final Roadmap: 3 Versions Để v1.0

```
 NOW          v0.8.0              v0.9.0            v1.0.0
  │        Quality Foundation   Hardening           Stable         THEN
  │            🧪🔭               🔒                🏆             │
  ▼            │                   │                 │              ▼
──────────────┼───────────────────┼─────────────────┼──────────────────
Framework     │  Test infra       │  API audit      │  Freeze       Agent
v0.7.0        │  CI pipeline      │  Deprecations   │  Semver       App
(current)     │  Coverage         │  Coverage gaps  │  Docs         v0.1
              │  Tracing/Events   │  Error messages │  Benchmarks
              │  Cost tracking    │  Migration guide│
              │  ~6-8 stories     │  ~4-5 stories   │  ~3-4 stories
              │  ~1-2 sprints     │  ~1 sprint      │  ~1 sprint
```

---

### v0.8.0 — "Quality Foundation" 🧪🔭

**Goal:** Nâng cấp test infrastructure + thêm observability. Nền tảng chất lượng cho đường đến v1.0.

#### Part A: Testing Infrastructure 🧪

| Feature | Location | Mô tả |
|---------|----------|--------|
| `traitclaw-test-utils` | NEW crate | Shared MockProvider, MockMemory, MockTools, make_runtime() |
| Deduplicate mocks | core + strategies | Xóa duplicate test_utils, dùng shared crate |
| CI pipeline | `.github/workflows/ci.yml` | `cargo fmt`, `clippy`, `test --workspace`, `doc` |
| Coverage setup | CI + local | `cargo-llvm-cov` — baseline coverage report |
| Coverage targets | All crates | Đo baseline, set targets: core 80%, optional 70% |

#### Part B: Observability 🔭

| Feature | Location | Mô tả |
|---------|----------|--------|
| Tracing spans | `traitclaw-core` | `tracing::instrument` cho mọi LLM call, tool call, guard check |
| `AgentEvent` enum | `traitclaw-core` | Structured events: `LlmStart`, `LlmEnd`, `ToolCall`, `GuardBlock`, `HintTriggered` |
| Event callback | `traitclaw-core` | `AgentBuilder::on_event(callback)` — observe mọi hoạt động |
| Cost estimation | `traitclaw-core` | `RunUsage` mở rộng: `estimated_cost_usd` dựa vào model pricing |
| Example | `examples/26-*` | Observability demo với `tracing-subscriber` |

**Tại sao Testing trước?**
- Mock duplicate giữa crates → technical debt ngay bây giờ
- CI pipeline → catch regressions sớm trước khi thêm observability code
- Coverage baseline → biết cần test gì thêm cho v0.9.0 Hardening

**Tại sao Observability cùng version?**
- Cả hai đều phục vụ "quality" — test quality + runtime quality
- Observability tests cần shared test utils → dependency tự nhiên

**Kích thước:** ~6-8 stories, ~1-2 sprints
**Breaking changes:** Zero — purely additive

---

### v0.9.0 — "Hardening" 🔒

**Goal:** Chuẩn bị API cho freeze. Sau version này, public API phải "done".

#### Part A: API Hardening 🔒

| Feature | Location | Mô tả |
|---------|----------|--------|
| API audit | All crates | Review mọi `pub` item — có cần expose không? |
| Deprecation cleanup | `traitclaw-core` | Remove `ContextStrategy`, `OutputProcessor` (deprecated since v0.3.0) |
| Error message improvement | `traitclaw-core` | Actionable error messages: "did you forget to...?" |
| Migration guide | `docs/` | `migration-v0.8-to-v0.9.md` — last breaking changes |
| Clippy + doc audit | All crates | `#![deny(missing_docs)]` enforced everywhere |

#### Part B: Advanced Testing 🧪

| Feature | Tool | Mô tả |
|---------|------|--------|
| Property-based testing | `proptest` | Fuzz guards, strategies, tool schemas — tìm edge cases |
| Snapshot testing | `insta` | Lock output formats, detect unintended changes |
| Benchmark suite | `criterion` | Performance baselines cho core runtime, provider calls |
| Coverage gap filling | `cargo-llvm-cov` | Đạt targets: core 80%, optional 70% |

**Tại sao?**
- v1.0 = semver promise = "không breaking changes cho đến v2.0"
- Cần 1 version "cleanup" trước khi freeze
- Đây là version DUY NHẤT được phép breaking changes

**Kích thước:** ~4-5 stories, ~1 sprint
**Breaking changes:** ⚠️ Yes — last chance

---

### v1.0.0 — "Stable" 🏆

**Goal:** API freeze. Semver guarantee. Sẵn sàng build agent app.

| Feature | Location | Mô tả |
|---------|----------|--------|
| Version bump | All Cargo.toml | `version = "1.0.0"` |
| API freeze | All crates | No more public API changes |
| Comprehensive docs | `docs/` | Full guide: getting started → advanced → architecture |
| Benchmark suite | `benchmarks/` | Latency, throughput, memory vs competition |
| CHANGELOG | Root | Complete changelog from v0.1.0 → v1.0.0 |
| crates.io publish | N/A | Publish all 13+ crates |

**Kích thước:** ~3-4 stories, ~1 sprint
**Breaking changes:** Zero — this IS the stable

---

## 4. Post-v1.0: Agent App + Framework Extensions

Sau v1.0, 2 track song song:

```
v1.0.0 ──────┬────────────────────────────────────────▶
             │
             ├─ Track A: Agent App (NEW PROJECT)
             │  ├─ v0.1: CLI agent, basic tools (file, shell)
             │  ├─ v0.2: MCP integration, multi-agent
             │  ├─ v0.3: Advanced reasoning (ReAct/CoT)
             │  └─ ...
             │
             └─ Track B: Framework Extensions (v1.x semver-safe)
                ├─ v1.1: traitclaw-server (HTTP/WS)
                ├─ v1.2: traitclaw-workflow (DAG engine)
                ├─ v1.3: More providers (Gemini native, etc.)
                └─ v1.4: More memory backends (PG, Redis)
```

### Agent App — Separate Project ✅

**Quyết định:** Agent app sẽ là **project/repo riêng**, không nằm trong TraitClaw workspace.

- TraitClaw = pure library (giống Tokio, Axum)
- Agent app = application built ON TraitClaw (giống web app built on Axum)
- `showcase/miniclaw` vẫn giữ lại trong TraitClaw repo như demo/showcase

**Tên, repo, architecture của agent app — quyết định sau v1.0.**

---

## 5. Summary: Con Đường Rõ Ràng

| Milestone | Nội dung | Ước tính | Output |
|-----------|----------|----------|--------|
| **v0.8.0** | Quality Foundation (Testing + Observability) | ~1-2 sprints | Shared test crate, CI, coverage, tracing, events |
| **v0.9.0** | Hardening | ~1 sprint | Clean API, last breaking changes |
| **v1.0.0** | Stable | ~1 sprint | Semver guarantee, crates.io |
| **Agent v0.1** | Real agent app (separate project) | ~2 sprints | Working CLI AI agent |

**Tổng: ~4 sprints (v0.8 → v1.0) + 2 sprints (Agent v0.1) = ~6 sprints đến agent hoạt động thật.**

---

## 6. Alignment Check

| Principle | Đảm bảo? |
|-----------|----------|
| "Library, not platform" | ✅ Framework stays library, agent is separate project |
| "CORE AGENT PRIMITIVES only" | ✅ Không thêm workflow/server trước v1.0 — chỉ obs + cleanup |
| "Giống tokio cho async" | ✅ Tokio stable → build Axum. TraitClaw stable → build Agent. |
| Progressive complexity | ✅ Agent app proves DX works in practice |
| AD-8: Showcase proves design | ✅ Agent app = ultimate showcase |
