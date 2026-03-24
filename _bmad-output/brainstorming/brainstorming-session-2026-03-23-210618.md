---
stepsCompleted: [1, 2, 3]
inputDocuments: []
session_topic: 'Rust AI Agent Framework - TraitClaw'
session_goals: 'Thiết kế kiến trúc framework/library cho dev build AI agent nhanh'
selected_approach: 'ai-recommended'
techniques_used: ['first_principles', 'scamper', 'cross_pollination', 'morphological_analysis', 'constraint_mapping', 'reverse_brainstorming']
ideas_generated: [100+]
context_file: ''
---

# 🦀 TraitClaw — Rust AI Agent Framework Brainstorming

**Date:** 2026-03-23
**Participant:** Bangvu
**Session Type:** AI-Recommended Multi-Phase Brainstorming
**Goal:** Framework/Library cho dev build AI agents nhanh bằng Rust

---

## Reference Research Summary

| Framework | Lang | Key Innovation |
|-----------|------|---------------|
| **Mastra** | TS | Model routing 40+ providers, graph workflows `.then()/.branch()/.parallel()` |
| **VoltAgent** | TS | Supervisor + sub-agents, resumable streaming, typed tools with Zod |
| **OpenClaw** | Node | 22+ channels, skills platform, multi-agent routing, live canvas |
| **GoClaw** | Go | Single binary 25MB, multi-tenant PostgreSQL, 5-layer security, agent teams |
| **ZeroClaw** | Rust | <5MB RAM, trait-based, hardware peripherals, sandboxing, SOPs |
| **Rig** | Rust | Clean LLM abstraction, agentic pipelines, RAG integration |
| **Swarms-RS** | Rust | Multi-agent orchestration, hierarchical/concurrent/graph architectures |

---

## Phase 1: First Principles Thinking 🧠

**"Nếu bắt đầu từ zero, fundamental truths của một AI agent framework là gì?"**

### Fundamental Truths

1. **Agent = LLM + Tools + Memory + Loop** — mọi agent đều là vòng lặp: nhận input → suy nghĩ (LLM) → hành động (tools) → quan sát → lặp lại
2. **LLM chỉ là text-in/text-out** — provider nào cũng chỉ nhận prompt trả text/stream
3. **Tools là side effects** — gọi API, đọc file, query DB — cần type-safe và sandboxed
4. **Memory có 3 tầng** — short-term (conversation), working (task context), long-term (knowledge)
5. **Orchestration = scheduling + routing** — ai làm gì, khi nào, theo thứ tự nào
6. **Developer Experience > Feature Count** — framework thắng khi dev viết ít code nhất để có agent hoạt động

### Những gì KHÔNG phải fundamental

- ❌ Channel integration (WhatsApp, Telegram) — đó là application layer
- ❌ Web dashboard — đó là tooling layer
- ❌ Multi-tenancy — đó là deployment layer
- ❌ Voice/TTS — đó là media adapter

### → Core Insight #1
> **TraitClaw nên tập trung vào CORE AGENT PRIMITIVES, không cố làm platform all-in-one.**
> Giống như `tokio` cho async — TraitClaw là foundation mà mọi người build trên.

---

## Phase 2: SCAMPER Method 🔧

Áp dụng 7 lenses lên concept "AI Agent Framework":

### S — Substitute (Thay thế gì?)

| # | Ý tưởng | Chi tiết |
|---|---------|----------|
| 1 | Thay JSON config bằng **Rust macros** | `#[agent]` proc macro tự generate boilerplate |
| 2 | Thay YAML workflow bằng **Rust DSL** | Type-safe workflow: `agent.then(step1).branch(cond, a, b)` |
| 3 | Thay runtime reflection bằng **compile-time dispatch** | Generic traits thay vì `dyn Any` |
| 4 | Thay HTTP polling bằng **native async streams** | `Stream<Item = AgentEvent>` |
| 5 | Thay string tool names bằng **type-safe tool registry** | `Tool<Input, Output>` trait |
| 6 | Thay prompt engineering bằng **prompt templates với typed variables** | `prompt!("Analyze {data:DataType}")` |
| 7 | Thay manual retry bằng **built-in resilience patterns** | Circuit breaker, exponential backoff trong provider layer |

### C — Combine (Kết hợp gì?)

| # | Ý tưởng | Chi tiết |
|---|---------|----------|
| 8 | Tool definition + Schema generation | Từ Rust struct tự gen JSON schema cho LLM |
| 9 | Agent + Workflow = **AgentFlow** | Agent tự chạy workflow mà không cần orchestrator riêng |
| 10 | Memory + RAG = **Knowledge trait** | Unified interface cho cả recall và retrieval |
| 11 | Testing + Agent = **AgentTestKit** | Mock LLM responses cho deterministic testing |
| 12 | CLI + Framework = **`cargo traitclaw`** | Scaffold agent project, run, test, deploy từ CLI |
| 13 | Type system + Prompt = **Structured Output** | Rust types tự become output schema cho LLM |

### A — Adapt (Điều chỉnh từ đâu?)

| # | Ý tưởng | Nguồn cảm hứng |
|---|---------|----------------|
| 14 | **Actor model cho agents** | Từ Actix/Tokio actors — mỗi agent là actor với mailbox |
| 15 | **Middleware pipeline** | Từ Tower/Axum — `Layer<AgentService>` cho before/after hooks |
| 16 | **Plugin system** | Từ Bevy ECS — dynamic plugin loading cho extensibility |
| 17 | **Builder pattern** | Từ Reqwest — `Agent::builder().model("gpt-4o").tool(browser).build()` |
| 18 | **Feature flags** | Từ Cargo features — `traitclaw = { features = ["openai", "anthropic", "memory-sqlite"] }` |
| 19 | **Derive macros** | Từ Serde — `#[derive(Tool)]` auto-implement Tool trait từ struct |

### M — Modify/Magnify (Phóng đại/Sửa đổi?)

| # | Ý tưởng | Chi tiết |
|---|---------|----------|
| 20 | **Magnify type safety** | Mọi interaction đều typed — không có `serde_json::Value` ở public API |
| 21 | **Magnify zero-cost abstractions** | Provider trait compiled away, không có virtual dispatch overhead |
| 22 | **Magnify composability** | Agents composable như functions: `agent1.pipe(agent2).pipe(agent3)` |
| 23 | **Magnify error handling** | Custom error types với context chain — biết chính xác step nào fail |
| 24 | **Magnify streaming** | First-class streaming cho mọi thứ: LLM response, tool output, events |

### P — Put to Other Uses (Dùng cho gì khác?)

| # | Ý tưởng | Chi tiết |
|---|---------|----------|
| 25 | AI-powered CLI tools | `traitclaw` binary chạy agent từ terminal |
| 26 | Embedded AI agents | Chạy trên IoT/edge devices (Raspberry Pi, ESP32 via WASM) |
| 27 | Game NPC AI | Agent framework cho game development |
| 28 | Automated testing | AI agents cho integration testing |
| 29 | Data pipeline orchestration | Agent-driven ETL workflows |
| 30 | Code generation tools | IDE plugins sử dụng TraitClaw agents |

### E — Eliminate (Loại bỏ gì?)

| # | Ý tưởng | Chi tiết |
|---|---------|----------|
| 31 | **Eliminate runtime overhead** | Compile-time provider selection, zero-cost traits |
| 32 | **Eliminate boilerplate** | Proc macros cho tool definition, agent setup |
| 33 | **Eliminate config files** | Code IS config — Rust code thay cho YAML/JSON/TOML |
| 34 | **Eliminate provider lock-in** | Universal `Provider` trait, swap bằng 1 line |
| 35 | **Eliminate unsafe casts** | Full type safety từ prompt → tool → response |
| 36 | **Eliminate GC overhead** | Rust ownership model — predictable latency |

### R — Reverse (Đảo ngược?)

| # | Ý tưởng | Chi tiết |
|---|---------|----------|
| 37 | **Agent PULLS tasks** thay vì được PUSHED | Agent đăng ký interest, task queue phân phối |
| 38 | **LLM adapts to code** thay vì code adapts to LLM | Define output type, framework tự generate prompt |
| 39 | **Test-first agent dev** | Define expected behavior → framework generates agent skeleton |
| 40 | **Bottom-up composition** | Build complex agents từ simple atomic agents |

---

## Phase 3: Cross-Pollination 🌿

**"Ngành khác giải quyết vấn đề tương tự như thế nào?"**

### Từ Web Frameworks (Axum/Actix-web)

| # | Ý tưởng | Áp dụng |
|---|---------|---------|
| 41 | **Router pattern** | `AgentRouter` — route requests đến đúng agent dựa trên intent |
| 42 | **Middleware/Layer** | `AgentLayer` — logging, rate-limiting, guardrails, caching |
| 43 | **Extractor pattern** | `FromContext<T>` — extract data cần thiết từ agent context |
| 44 | **State management** | `AgentState<T>` — shared state giữa các tool calls |
| 45 | **Handler pattern** | Tool handlers: `async fn my_tool(input: MyInput) -> MyOutput` |

### Từ Game Engines (Bevy)

| # | Ý tưởng | Áp dụng |
|---|---------|---------|
| 46 | **Entity Component System** | Agent = Entity, capabilities = Components, loops = Systems |
| 47 | **Event system** | `EventWriter<AgentEvent>` / `EventReader<AgentEvent>` |
| 48 | **Plugin architecture** | `app.add_plugin(OpenAIPlugin)` — modular provider loading |
| 49 | **Scheduling** | Agent execution schedules — priority, frequency, conditions |
| 50 | **Resource system** | Shared resources (API keys, connections) giữa agents |

### Từ Database (SQLx/SeaORM)

| # | Ý tưởng | Áp dụng |
|---|---------|---------|
| 51 | **Compile-time query checking** | Compile-time validate prompt templates |
| 52 | **Migration system** | Agent version migrations — upgrade agent behavior safely |
| 53 | **Connection pooling** | LLM connection pooling — reuse HTTP connections |
| 54 | **Transaction model** | Agent transactions — multi-step with rollback |

### Từ Message Queues (NATS/RabbitMQ)

| # | Ý tưởng | Áp dụng |
|---|---------|---------|
| 55 | **Pub/Sub** | Agents subscribe vào topics, communicate async |
| 56 | **Dead letter queue** | Failed agent tasks được retry hoặc escalated |
| 57 | **Back pressure** | Rate limiting cho LLM calls dựa trên capacity |

### Từ Kubernetes/Cloud Native

| # | Ý tưởng | Áp dụng |
|---|---------|---------|
| 58 | **Declarative agents** | Describe desired agent state, framework reconciles |
| 59 | **Health checks** | Agent liveness/readiness probes |
| 60 | **Sidecar pattern** | Attach capabilities (memory, tools) như sidecars |
| 61 | **Operator pattern** | Custom controller cho specific agent types |

---

## Phase 4: Morphological Analysis 🔬

**Phân tích tổ hợp parameters cho core architecture:**

### Parameter Space

| Parameter | Options |
|-----------|---------|
| **Agent Model** | A1: Single-loop | A2: ReAct | A3: Plan-and-Execute | A4: Tree-of-Thought |
| **Provider Layer** | B1: Trait-based | B2: Enum-based | B3: Plugin-loaded |
| **Tool System** | C1: Derive macro | C2: Trait impl | C3: Function registration | C4: MCP native |
| **Memory** | D1: In-process | D2: External store | D3: Pluggable trait | D4: Layered (short+long) |
| **Orchestration** | E1: Sequential | E2: Graph-based | E3: Actor-based | E4: Event-driven |
| **Error Strategy** | F1: Result types | F2: Retry + fallback | F3: Circuit breaker | F4: Supervisor tree |
| **DX** | G1: Builder API | G2: Macro DSL | G3: Config-driven | G4: Derive-based |

### Top Combinations

| # | Combo | Tên concept | Mô tả |
|---|-------|-------------|--------|
| 62 | A2+B1+C1+D3+E3+F4+G1 | **"The Pragmatic"** | ReAct loop, trait providers, derive tools, pluggable memory, actor orchestration, supervisor errors, builder API |
| 63 | A3+B1+C1+D4+E2+F2+G2 | **"The Planner"** | Plan-execute, trait providers, derive tools, layered memory, graph workflow, retry/fallback, macro DSL |
| 64 | A1+B1+C2+D1+E1+F1+G1 | **"The Minimal"** | Simple loop, traits, manual tool impl, in-process memory, sequential, Result types, builder — lowest barrier to entry |
| 65 | A2+B1+C4+D4+E4+F4+G4 | **"The Ecosystem"** | ReAct, traits, MCP tools, layered memory, event-driven, supervisor tree, derive-based — maximum interop |

---

## Phase 5: Constraint Mapping & DX Deep Dive 🎯

### Developer Experience Goals

**"Dev cần bao nhiêu code để có working agent?"**

| # | Ý tưởng | Code ước lượng |
|---|---------|---------------|
| 66 | **5-line agent** | `Agent::builder().model(gpt4o).tool(search).build().run("query")` |
| 67 | **Derive-based tools** | `#[derive(Tool)] struct Search { query: String }` → 3 lines |
| 68 | **Zero-config defaults** | Default provider (env var), default memory (in-process) |
| 69 | **Scaffold CLI** | `cargo traitclaw new my-agent` → working project |
| 70 | **Hot reload dev** | `cargo traitclaw dev` — watch files, reload agents |

### API Design Ideas

| # | Ý tưởng | Code example |
|---|---------|-------------|
| 71 | **Fluent builder** | `Agent::new("assistant").instructions("...").model(openai("gpt-4o")).tool(web_search).memory(sqlite("./mem.db"))` |
| 72 | **Derive agent** | `#[derive(Agent)] #[agent(model = "gpt-4o")] struct MyAgent { #[tool] search: WebSearch }` |
| 73 | **Function-as-agent** | `#[agent] async fn summarize(input: &str) -> Summary { ... }` |
| 74 | **Pipeline composition** | `let pipeline = classify.then(route).branch(|c| match c { ... })` |
| 75 | **Multi-agent** | `Supervisor::new(vec![researcher, writer, reviewer]).strategy(RoundRobin)` |
| 76 | **Workflow DSL** | `workflow! { fetch -> analyze -> if good { publish } else { revise -> analyze } }` |
| 77 | **Streaming** | `agent.stream("query").for_each(|chunk| println!("{chunk}")).await` |
| 78 | **Testing** | `agent.with_mock(responses![("hello", "world")]).run("hello").assert_eq("world")` |

### Provider Abstraction Ideas

| # | Ý tưởng | Chi tiết |
|---|---------|----------|
| 79 | **Universal Provider trait** | `trait Provider: Send + Sync { async fn complete(&self, req: Request) -> Stream<Response>; }` |
| 80 | **Feature-gated providers** | `traitclaw-openai`, `traitclaw-anthropic`, `traitclaw-google` as separate crates |
| 81 | **OpenAI-compatible fallback** | Any OpenAI-compatible endpoint works out-of-box |
| 82 | **Model routing** | `Router::new().route("fast/*", groq).route("smart/*", claude).route("cheap/*", ollama)` |
| 83 | **Provider middleware** | `provider.with(cache).with(rate_limit).with(logging)` |
| 84 | **Cost tracking** | Built-in token counting + cost estimation per provider |

### Memory Architecture Ideas

| # | Ý tưởng | Chi tiết |
|---|---------|----------|
| 85 | **3-layer memory** | Conversation (short) → Working (session) → Knowledge (persistent) |
| 86 | **Memory trait** | `trait Memory { async fn recall(&self, query: &str) -> Vec<MemoryEntry>; }` |
| 87 | **Semantic recall** | Embedding-based retrieval built into memory layer |
| 88 | **Memory adapters** | SQLite, PostgreSQL, Redis, in-memory — pluggable |
| 89 | **Auto-summarization** | Long conversations auto-compressed to working memory |
| 90 | **Shared memory** | Multiple agents share knowledge base |

### Tool System Ideas

| # | Ý tưởng | Chi tiết |
|---|---------|----------|
| 91 | **Derive macro** | `#[derive(Tool)] #[tool(desc = "Search")] struct WebSearch { query: String }` |
| 92 | **Function tools** | `#[tool] async fn search(query: &str) -> Vec<Result> { ... }` |
| 93 | **MCP integration** | Native MCP client/server — use any MCP tool |
| 94 | **Tool composition** | `search.then(summarize).with_retry(3)` |
| 95 | **Sandboxed execution** | Tools run in WASM sandbox optionally |
| 96 | **Tool approval** | Human-in-the-loop approval for dangerous tools |
| 97 | **Auto JSON schema** | Rust struct → JSON schema for LLM tool calling |

---

## Phase 6: Reverse Brainstorming ⚠️

**"Làm sao để TraitClaw THẤT BẠI?"** → đảo ngược = cách thành công

| # | Cách thất bại | → Cách thành công |
|---|---------------|-------------------|
| 98 | Quá nhiều abstraction, dev không hiểu | → **Layered API**: simple default, power underneath |
| 99 | Compile time quá lâu do macros | → **Optional macros**, manual impl luôn available |
| 100 | Lock-in vào 1 provider | → **Trait-based providers**, swap 1 dòng |
| 101 | Documentation tệ | → **Executable examples**, mỗi feature có runnable example |
| 102 | Breaking changes liên tục | → **Semver strict**, deprecation warnings trước |
| 103 | Không test được | → **MockProvider**, deterministic testing built-in |
| 104 | Chậm hơn TypeScript alternatives | → **Benchmark suite**, CI benchmark trên mỗi PR |
| 105 | Community không contribute được | → **Plugin system**, contribute không cần PR vào core |

---

## 🏗️ Architecture Vision: Emergent Design

Từ 105+ ý tưởng trên, kiến trúc tự nhiên emerge:

```
┌─────────────────────────────────────────────────┐
│                  traitclaw (meta-crate)           │
│  Re-exports everything, feature-gated           │
├─────────────────────────────────────────────────┤
│                                                  │
│  ┌──────────┐  ┌──────────┐  ┌──────────────┐  │
│  │traitclaw- │  │traitclaw- │  │ traitclaw-    │  │
│  │  core    │  │  macros  │  │   cli        │  │
│  │          │  │          │  │              │  │
│  │• Agent   │  │• #[tool] │  │• new project │  │
│  │• Tool    │  │• #[agent]│  │• dev server  │  │
│  │• Memory  │  │• derive  │  │• test runner │  │
│  │• Provider│  │  Tool    │  │• benchmark   │  │
│  │• Workflow│  │          │  │              │  │
│  │• Stream  │  │          │  │              │  │
│  └──────────┘  └──────────┘  └──────────────┘  │
│                                                  │
│  ┌─────────────────────────────────────────────┐ │
│  │            Provider Crates                   │ │
│  │  traitclaw-openai    traitclaw-anthropic       │ │
│  │  traitclaw-google    traitclaw-ollama          │ │
│  │  traitclaw-openai-compat (any endpoint)       │ │
│  └─────────────────────────────────────────────┘ │
│                                                  │
│  ┌─────────────────────────────────────────────┐ │
│  │            Memory Crates                     │ │
│  │  traitclaw-memory-sqlite                      │ │
│  │  traitclaw-memory-postgres                    │ │
│  │  traitclaw-memory-redis                       │ │
│  └─────────────────────────────────────────────┘ │
│                                                  │
│  ┌─────────────────────────────────────────────┐ │
│  │            Extension Crates                  │ │
│  │  traitclaw-mcp      traitclaw-rag             │ │
│  │  traitclaw-eval     traitclaw-server           │ │
│  └─────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────┘
```

### Core Traits (traitclaw-core)

```rust
// Provider — kết nối LLM
trait Provider: Send + Sync + 'static {
    async fn complete(&self, req: CompletionRequest) -> Result<CompletionStream>;
    fn model_info(&self) -> ModelInfo;
}

// Tool — agent gọi functions
trait Tool: Send + Sync + 'static {
    type Input: DeserializeOwned + JsonSchema;
    type Output: Serialize;
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    async fn execute(&self, input: Self::Input) -> Result<Self::Output>;
}

// Memory — agent nhớ context
trait Memory: Send + Sync + 'static {
    async fn store(&self, entry: MemoryEntry) -> Result<()>;
    async fn recall(&self, query: &str, limit: usize) -> Result<Vec<MemoryEntry>>;
    async fn conversation(&self, id: &str) -> Result<Vec<Message>>;
}

// Agent — core abstraction
trait AgentBehavior: Send + Sync + 'static {
    async fn run(&self, input: &str) -> Result<AgentOutput>;
    async fn stream(&self, input: &str) -> Result<AgentStream>;
}
```

### Developer Experience: "Hello World" → Advanced

```rust
// === Level 1: 5 dòng — Simple Agent ===
use traitclaw::prelude::*;

#[tokio::main]
async fn main() {
    let agent = Agent::quick("gpt-4o", "You are helpful");
    println!("{}", agent.run("Hello!").await.unwrap());
}

// === Level 2: Tools ===
#[derive(Tool)]
#[tool(description = "Search the web")]
struct WebSearch {
    query: String,
}

impl WebSearch {
    async fn execute(&self) -> Vec<SearchResult> { /* ... */ }
}

let agent = Agent::builder()
    .model(openai("gpt-4o"))
    .instructions("Research assistant")
    .tool(WebSearch)
    .build();

// === Level 3: Multi-Agent ===
let researcher = Agent::builder()
    .name("researcher")
    .model(anthropic("claude-sonnet"))
    .tool(WebSearch)
    .build();

let writer = Agent::builder()
    .name("writer")
    .model(openai("gpt-4o"))
    .build();

let team = Team::new(vec![researcher, writer])
    .supervisor(openai("gpt-4o-mini"))
    .strategy(Sequential);

// === Level 4: Workflows ===
let pipeline = Workflow::new()
    .step("classify", classify_agent)
    .step("process", process_agent)
    .branch("route", |result| match result.category {
        "urgent" => "escalate",
        _ => "respond",
    })
    .step("escalate", escalate_agent)
    .step("respond", respond_agent);

// === Level 5: Streaming + Memory ===
let agent = Agent::builder()
    .model(openai("gpt-4o"))
    .memory(SqliteMemory::new("./agent.db"))
    .middleware(|req, next| {
        println!("→ {}", req.input);
        let res = next.run(req).await;
        println!("← {:?}", res);
        res
    })
    .build();

let mut stream = agent.stream("Tell me about Rust").await?;
while let Some(chunk) = stream.next().await {
    print!("{}", chunk.text);
}
```

---

## 🎯 Key Differentiators vs Competition

| Aspect | TS Frameworks (Mastra/VoltAgent) | Existing Rust (Rig/Swarms) | **TraitClaw** |
|--------|----------------------------------|----------------------------|-------------|
| **Performance** | GC pauses, high memory | Good but basic API | ⚡ Zero-cost abstractions, predictable latency |
| **Type Safety** | TypeScript (structural) | Rust (but verbose) | 🦀 Derive macros = type-safe + ergonomic |
| **DX** | Excellent (JS ecosystem) | Verbose, steep learning | 🎯 Progressive: 5-line → enterprise |
| **Composition** | Good via JS | Limited | 🔗 Pipe, branch, parallel — fluent API |
| **Testing** | Jest mocks | Manual | 🧪 Built-in MockProvider + TestKit |
| **Deployment** | Node runtime needed | Binary | 📦 Static binary, WASM, embedded |
| **MCP** | Full support | Partial | 🔌 Native MCP client + server |

---

## 📋 MVP Scope Proposal

### Phase 1: Core (v0.1.0)
- [ ] `traitclaw-core`: Agent, Tool, Provider, Memory traits
- [ ] `traitclaw-macros`: `#[derive(Tool)]`, `#[tool]` function macro
- [ ] `traitclaw-openai`: OpenAI provider (+ any compatible endpoint)
- [ ] `traitclaw`: meta-crate with prelude
- [ ] 5 working examples
- [ ] README + docs.rs documentation

### Phase 2: Ecosystem (v0.2.0)
- [ ] `traitclaw-anthropic`: Anthropic provider
- [ ] `traitclaw-memory-sqlite`: SQLite memory adapter
- [ ] `traitclaw-mcp`: MCP client integration
- [ ] Workflow engine (sequential + branch)
- [ ] `cargo traitclaw new` CLI scaffolding

### Phase 3: Advanced (v0.3.0)
- [ ] Multi-agent (Team, Supervisor)
- [ ] `traitclaw-rag`: RAG integration
- [ ] `traitclaw-eval`: Agent evaluation framework
- [ ] `traitclaw-server`: HTTP server for agents
- [ ] Streaming + middleware pipeline

---

## 💡 Naming Alternatives Brainstormed

| # | Tên | Ý nghĩa |
|---|-----|---------|
| 1 | **TraitClaw** | Base (foundation) + Claw (Rust crab) — nền tảng cho AI agents |
| 2 | **Ferrite** | Sắt (Fe) — mạnh mẽ, bền vững, Rust-inspired |
| 3 | **Forge** | Lò rèn — nơi tạo ra agents |
| 4 | **Anvil** | Đe — công cụ cơ bản nhất cho thợ rèn |
| 5 | **Crabby** | Playful crab reference |
| 6 | **IronAgent** | Sắt + Agent — mạnh mẽ và rõ ràng |
| 7 | **RustAgent** | Trực tiếp, dễ search |
| 8 | **Oxid** | Oxide — Rust (gỉ sét) = oxide hóa |

---

*Session generated 105+ ideas across 6 brainstorming techniques.*
*Architecture vision emerged from convergent ideation phase.*
