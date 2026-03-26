---
stepsCompleted: [1, 2, 3, 4, 5, 6]
inputDocuments: ["brainstorming/brainstorming-session-2026-03-23-210618.md", "_bmad-output/project-context.md"]
date: 2026-03-26
author: Bangvu
---

# Product Brief: TraitClaw

## Executive Summary

TraitClaw là một Rust framework giúp developers xây dựng các AI agent phức tạp một cách nhanh chóng. Điểm cốt lõi của TraitClaw là kiến trúc **layered trait system** — mọi thành phần cốt lõi của một AI agent (loop engine, cache, guard, authorization, reasoning strategy, tracing/OpenTelemetry, memory, orchestration) đều là trait có thể swap độc lập. Developer bắt đầu với defaults hợp lý và chỉ thay thế đúng layer cần thiết, không ảnh hưởng gì đến các layer còn lại.

Trong khi các framework Rust hiện tại (Rig, Swarms-RS) yêu cầu boilerplate quá nhiều hoặc lock-in vào kiến trúc cứng nhắc, TraitClaw cung cấp 5-line entry point có thể scale đến enterprise-grade multi-agent systems — với zero-cost abstractions và compile-time verified composition.

---

## Core Vision

### Problem Statement

Xây dựng production-grade AI agents bằng Rust hiện nay đòi hỏi hoặc:
1. **Boilerplate quá nhiều** — manually wire LLM providers, tools, memory, và loop logic từ scratch, hoặc
2. **Framework lock-in** — sử dụng framework hardcode agent behavior, khiến việc swap reasoning strategy, thêm guard layer, hay đổi memory backend trở nên đau đớn khi requirements thay đổi.

Kết quả: Rust developers hoặc bỏ qua AI agent development (chuyển sang Python/TypeScript), hoặc mất nhiều thời gian fighting framework hơn là building agents.

### Problem Impact

- Teams bỏ Rust cho AI workloads, mất đi performance và safety guarantees
- Production agents không thể adapt khi requirements evolution — đổi LLM provider, thêm guardrails, compliance layers
- Multi-agent systems phức tạp mất tuần thay vì ngày để build

### Why Existing Solutions Fall Short

| Solution | Gap |
|----------|-----|
| **Rig** | Clean LLM abstraction, nhưng không multi-agent, ít composability |
| **Swarms-RS** | Hỗ trợ multi-agent, nhưng API verbose, không có guard/hint system |
| **Python/TS frameworks** | DX tốt, nhưng GC overhead, không type-safe, runtime errors |
| **Roll-your-own** | Maximum control, nhưng mất hàng tuần infrastructure |

Không ai cung cấp **layered, swappable architecture** nơi loop logic, cache, guard, authz, reasoning, tracing đều là independent traits có thể override selectively.

### Proposed Solution

TraitClaw cung cấp một **progressive, layer-based agent framework**:

- **Start simple** — `Agent::with_system(provider, "Your role")` tạo fully working agent trong 1 dòng
- **Add layers selectively** — attach guard layers, cache layers, custom loop strategies, reasoning modules via builder API
- **Swap anything** — mọi layer đều là trait; thay thế default implementation mà không ảnh hưởng các layer khác
- **Scale to teams** — `AgentFactory`, `AgentPool`, và `RoundRobinGroupChat` compose agents thành production pipelines

### Key Differentiators

1. **Full layer swappability** — Loop logic, cache, guard, authz, reasoning, tracing đều là independent traits. Swap bất kỳ layer nào trong 1 dòng; phần còn lại tiếp tục hoạt động.
2. **Guard–Hint–Track steering** — Built-in composable behavior control: Guards block (hard rules), Hints guide (contextual injection), Trackers monitor (state signals).
3. **Production observability** — `Tracker` trait kết nối OpenTelemetry-compatible backends (Jaeger, Tempo, Datadog) mà không thay đổi agent logic.
4. **Swappable orchestration** — Coordination giữa agents có thể tùy chỉnh (Round-Robin, Sequential, Supervisor, custom strategy).
5. **Agent boundary enforcement** — Tool-level capability scoping ngăn privilege escalation. Guard system block hành vi ngoài scope. Typed inter-agent contracts (roadmap) validate communication tại compile time.
6. **Progressive complexity** — 5 dòng cho hello-world, cùng API cho enterprise systems. Compile-time composition verification đảm bảo misconfigured agents fail to compile, không phải fail lúc 3am.

### Agent Reliability & Trust

TraitClaw đảm bảo agent reliability qua 5 tầng:

| Tầng | Mechanisms | Status |
|------|-----------|--------|
| **Task Boundary** | Tool scoping, Guard system, ToolBudgetGuard, WorkspaceBoundaryGuard | ✅ Có |
| **Loop Safety** | LoopDetectionGuard, max iterations, AdaptiveTracker | ✅ Có |
| **Inter-Agent Trust** | Typed messages, output schema validation, agent contracts | ⚠️ Roadmap |
| **Observability** | Tracing crate, Tracker trait, OpenTelemetry support | ⚠️ Partial |
| **Failure Recovery** | Error propagation, pipeline halt on error, retry/checkpoint/fallback | ⚠️ Roadmap |

---

## Target Users

### Primary Users

#### Persona 1: "Minh" — Rust Backend Developer

- **Profile:** Senior Rust dev, 3+ năm production experience. Làm fintech/infrastructure.
- **Problem:** Cần thêm AI agent vào microservice hiện có. Đã thử LangChain Python nhưng ghét runtime errors và GC latency. Rig quá basic, Swarms-RS boilerplate quá nhiều.
- **Needs:** Type-safe API, minimal boilerplate, swap provider dễ dàng, compile-time guarantees.
- **Success moment:** `cargo build` pass → agent chạy production trong 5 phút, không cần Google.

#### Persona 2: "Linh" — AI Engineer (từ Python/TypeScript)

- **Profile:** 2 năm build agents bằng LangChain/CrewAI. Chuyển sang Rust vì production performance requirements.
- **Problem:** Rust learning curve cao. Existing Rust frameworks quá "Rusty" — trait bounds, lifetimes everywhere.
- **Needs:** Progressive API (đơn giản → phức tạp), examples chạy được, migration guide rõ ràng.
- **Success moment:** Build first multi-agent pipeline trong 1 ngày thay vì 1 tuần.

#### Persona 3: "Tùng" — Platform/DevOps Engineer

- **Profile:** Quản lý AI workloads cho team. Deploy trên Kubernetes/edge environments.
- **Problem:** Python agents ăn RAM, khó monitor, deploy phức tạp (virtualenv, dependencies).
- **Needs:** Single binary deployment, OpenTelemetry integration, resource-efficient, observable agents.
- **Success moment:** Deploy AI agent service < 50MB container, full Jaeger tracing, 10x ít memory hơn Python.

### Secondary Users

- **Open-source contributors** — Muốn add provider crate mới hoặc tool crate, cần clear contribution guidelines và plugin architecture.
- **Tech leads / Architects** — Evaluate framework cho team adoption; cần benchmarks, documentation quality, community health, và enterprise readiness signals.

---

## Success Metrics

### User Success Metrics

| Metric | Target | Persona |
|--------|--------|--------|
| **Time to first agent** | ≤ 5 phút từ `cargo add` → agent chạy | Minh, Linh |
| **Time to multi-agent pipeline** | ≤ 1 ngày (vs 1 tuần với alternatives) | Linh |
| **Layer swap effort** | Thay provider/guard/memory trong ≤ 5 dòng code | Minh |
| **Container size** | < 50MB production image | Tùng |
| **Memory per agent** | < 10MB RSS (vs 100MB+ Python) | Tùng |

### Business Objectives

| Metric | Target (12 months) |
|--------|-------------------|
| **GitHub stars** | 1,000+ |
| **Monthly crate downloads** | 5,000+ |
| **Community contributors** | 10+ |
| **Production adopters** | 5+ teams publicly using |
| **Example coverage** | 25+ runnable examples |
| **Docs coverage** | 100% public API documented |

### Key Performance Indicators

- **Developer onboarding success rate:** % devs who get first agent running within 30 minutes of reading docs
- **API stability score:** Số breaking changes per minor release (target: 0)
- **Compile time overhead:** < 2% increase per new feature crate added
- **Test coverage:** ≥ 80% line coverage cho traitclaw-core

---

## MVP Scope

TraitClaw v0.5.0 đã ship với: core Agent, Provider trait, Tool system (#[derive(Tool)]), Guard-Hint-Track steering, Memory, RAG, MCP, Workflow, Eval, Server. "MVP" ở đây là **next major milestone v0.6.0 "Composition"**.

### Core Features (v0.6.0)

1. **`Agent::with_system()`** — 1-line agent creation shorthand
2. **`AgentFactory`** — shared-provider multi-agent spawning với `spawn()` và `spawn_with()`
3. **`AgentPool`** — team execution với `new()`, `from_team()`, `run_sequential()`
4. **`RoundRobinGroupChat`** — multi-turn agent collaboration với configurable termination
5. **Documentation** — migration guide, progressive example, README quickstart

### Out of Scope cho v0.6.0

| Feature | Deferred to | Lý do |
|---------|------------|-------|
| `OrchestrationStrategy` trait (swappable) | v0.7.0 | Cần design kỹ, v0.6.0 ship concrete types trước |
| Inter-agent typed contracts | v0.7.0 | Cần ổn định multi-agent API trước |
| OpenTelemetry integration | v0.7.0 | Tracker trait sẵn sàng, cần thêm OTel adapter crate |
| Task rejection protocol | v0.8.0 | Cần thêm research về UX |
| Retry/checkpoint/fallback | v0.8.0 | Complex, cần agent state persistence |
| Supervisor orchestration | v0.8.0+ | Depends on OrchestrationStrategy trait |

### MVP Success Criteria

- Tất cả 20+ examples hiện tại vẫn compile trên v0.6.0 (zero breaking changes)
- `Agent::with_system()` giảm 80% boilerplate cho single-agent use case
- Multi-agent pipeline (3 agents) hoạt động end-to-end trong ≤ 30 dòng code
- Compile time increase < 2% so với v0.5.0

### Future Vision

- **v0.7.0 "Orchestrate"** — Swappable orchestration strategies, OpenTelemetry, typed inter-agent contracts
- **v0.8.0 "Resilience"** — Retry, checkpoint/resume, fallback agents, task rejection protocol
- **v0.9.0 "Scale"** — Distributed agent execution, WASM deployment target
- **v1.0.0 "Production"** — Stable API, full benchmark suite, enterprise readiness
