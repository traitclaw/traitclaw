# 🦀 TraitClaw — Architecture Deep Dive

> Phân tích kiến trúc 5 framework tham khảo, xác định core/replaceable/extensible cho TraitClaw

---

## 1. Source Code Map — Các Framework Tham Khảo

### Mastra (TypeScript) — 35+ modules

```
packages/core/src/
├── agent/          ← Agent runtime + tool loop
├── llm/            ← LLM provider abstraction
├── tools/          ← Tool definition + registry
├── tool-provider/  ← Tool provider interface
├── tool-loop-agent/← ReAct-style tool loop
├── memory/         ← Conversation + working memory
├── workflows/      ← Graph-based workflow engine
├── stream/         ← Streaming response handling
├── mcp/            ← MCP client/server
├── a2a/            ← Agent-to-Agent protocol
├── evals/          ← Agent evaluation framework
├── observability/  ← Tracing + monitoring
├── di/             ← Dependency injection container
├── hooks/          ← Lifecycle hooks
├── cache/          ← Response caching
├── storage/        ← Persistent storage abstraction
├── vector/         ← Vector store for RAG
├── voice/          ← TTS/STT integration
├── schema/         ← JSON schema generation
├── server/         ← HTTP server
├── auth/           ← Authentication (enterprise)
├── run/            ← Execution tracking
├── events/         ← Event system
├── mastra/         ← Main orchestrator class
└── test-utils/     ← Testing utilities
```

### VoltAgent (TypeScript) — 18 modules

```
packages/core/src/
├── agent/          ← Agent class + sub-agents + supervisor
├── tool/           ← Tool definition with Zod schemas
├── memory/         ← Memory adapters (LibSQL, etc.)
├── workflow/       ← Workflow engine (declarative)
├── mcp/            ← MCP integration
├── a2a/            ← Agent-to-Agent
├── eval/           ← Evaluation framework
├── observability/  ← Tracing
├── registries/     ← Agent + tool registries
├── retriever/      ← RAG retrieval
├── planagent/      ← Plan-and-execute agent
├── events/         ← Event bus
├── voice/          ← Voice integration
├── triggers/       ← External trigger system
├── voltops/        ← ObservabilityConsole connection
├── logger/         ← Structured logging
├── workspace/      ← Workspace management
└── test-utils/     ← Mock providers + helpers
```

### OpenClaw (Node.js) — 45+ modules

```
src/
├── agents/         ← Multi-agent config + routing
├── channels/       ← 22+ messaging channels
├── gateway/        ← WebSocket control plane
├── sessions/       ← Session model
├── memory/         ← Memory system
├── hooks/          ← Lifecycle hooks (before/after LLM, tool)
├── routing/        ← Message routing engine
├── security/       ← DM access, sanitization
├── config/         ← Configuration system
├── context-engine/ ← Context assembly
├── plugins/        ← Plugin system + SDK
├── plugin-sdk/     ← Plugin development kit
├── cron/           ← Scheduled tasks
├── tts/            ← Text-to-speech
├── media/          ← Media pipeline
├── browser/        ← Browser automation
├── canvas-host/    ← Visual workspace
├── web-search/     ← Web search tools
└── shared/         ← Shared utilities
```

### GoClaw (Go) — 28 modules

```
internal/
├── agent/          ← Agent orchestration loop
├── providers/      ← LLM providers (20+)
├── tools/          ← Tool registry + execution
├── memory/         ← Memory system
├── channels/       ← Messaging channels (7)
├── gateway/        ← HTTP/WS gateway
├── sessions/       ← Per-user sessions
├── permissions/    ← 5-layer permission system
├── tasks/          ← Agent teams task board
├── sandbox/        ← Execution sandboxing
├── scheduler/      ← Cron + scheduling
├── bus/            ← Event bus
├── store/          ← PostgreSQL store
├── cache/          ← Response caching
├── mcp/            ← MCP integration
├── skills/         ← Skills registry
├── tracing/        ← OpenTelemetry tracing
├── heartbeat/      ← Agent health checks
├── knowledgegraph/ ← Knowledge graph
├── crypto/         ← AES-256-GCM encryption
├── http/           ← HTTP handlers
├── media/          ← Media processing
├── tts/            ← Text-to-speech
└── config/         ← Configuration
```

### ZeroClaw (Rust) — 30+ modules

```
src/
├── agent/          ← Agent orchestration loop
├── providers/      ← LLM providers (trait-based!)
├── tools/          ← Tool system (trait-based!)
├── memory/         ← Memory system (trait-based!)
├── channels/       ← Messaging channels (feature-gated)
├── gateway/        ← HTTP/WS/SSE gateway
├── runtime/        ← Agent runtime engine
├── security/       ← Sandboxing (Landlock, Bubblewrap)
├── hooks/          ← Lifecycle hooks
├── plugins/        ← Plugin system
├── skills/         ← Skills platform
├── skillforge/     ← Skill builder
├── sop/            ← Standard Operating Procedures
├── hands/          ← Multi-agent swarms
├── hardware/       ← Hardware peripherals
├── peripherals/    ← ESP32/STM32/Arduino/RPi GPIO
├── rag/            ← Retrieval-Augmented Generation
├── observability/  ← Tracing
├── approval/       ← Human-in-the-loop approval
├── cron/           ← Scheduled tasks
├── config/         ← Configuration
├── daemon/         ← System daemon
├── tunnel/         ← Cloudflare/Tailscale/ngrok
├── nodes/          ← Remote device nodes
├── service/        ← System service manager
└── auth/           ← Authentication
```

---

## 2. Pattern Analysis — Học được gì từ mỗi framework?

### 🔵 Pattern 1: Agent Loop (TẤT CẢ frameworks đều có)

**Đây là trái tim của mọi AI agent framework.**

```
Input → Prompt Construction → LLM Call → Parse Response
   ↓              ↓                           ↓
Context       Memory Recall            Tool Call?
Assembly     + RAG retrieval       ┌──── Yes → Execute Tool → Loop back
                                    └──── No → Return Response
```

| Framework | Cách implement | Học được |
|-----------|---------------|----------|
| **Mastra** | `tool-loop-agent/` — vòng lặp gọi tools cho đến khi LLM stop | Clean separation: loop logic ≠ agent logic |
| **VoltAgent** | `agent/` + `planagent/` — 2 strategies: ReAct + Plan-Execute | Nên support nhiều loop strategies |
| **OpenClaw** | `context-engine/` assembles context → agent processes | Context assembly là một concern riêng |
| **GoClaw** | `internal/agent/` — single orchestration loop | Loop nên có timeout + max iterations |
| **ZeroClaw** | `src/agent/` + `src/runtime/` — runtime tách riêng | **Runtime tách khỏi Agent definition** — key insight! |

> ### 💡 Key Learning: **Tách Agent Definition khỏi Agent Runtime**
> - **Agent** = config (model, tools, instructions, memory)
> - **Runtime** = execution engine (loop strategy, streaming, error handling)
> - Giống như trong web: Route definition ≠ HTTP server

---

### 🟢 Pattern 2: Provider Abstraction (Quan trọng nhất)

| Framework | Approach | Pros | Cons |
|-----------|----------|------|------|
| **Mastra** | Dùng Vercel AI SDK (`ai` package) | Massive provider support (40+) | External dependency |
| **VoltAgent** | Separate packages (`@voltagent/vercel-ai`) | Clean separation | Needs adapter per provider |
| **OpenClaw** | Internal provider wrapper | Full control | Maintenance burden |
| **GoClaw** | Interface-based, native HTTP | No deps, fast | Must implement each provider |
| **ZeroClaw** | **Trait-based** (`providers/`) | ⭐ Rust-native, zero-cost | Must implement each provider |

> ### 💡 Key Learning: **Trait-based Provider + OpenAI-compatible fallback**
> ```rust
> // Core trait — mỗi provider implement trait này
> pub trait Provider: Send + Sync + 'static {
>     async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse>;
>     async fn stream(&self, request: CompletionRequest) -> Result<CompletionStream>;
>     fn supports_tools(&self) -> bool;
>     fn supports_vision(&self) -> bool;
>     fn model_info(&self) -> &ModelInfo;
> }
> 
> // OpenAI-compatible — fallback cho bất kỳ endpoint nào
> pub struct OpenAICompatible { base_url: String, api_key: String }
> impl Provider for OpenAICompatible { ... }
> 
> // Specific providers — optimized cho từng provider
> pub struct AnthropicProvider { ... } // native HTTP+SSE, prompt caching
> pub struct GoogleProvider { ... }    // Gemini native API
> ```
> 
> **Pattern từ GoClaw đáng học:** Provider wrapper với **failover**, **retry**, và **model routing** built-in.

---

### 🟡 Pattern 3: Tool System

| Framework | Tool Definition | Schema Generation | Execution |
|-----------|----------------|-------------------|-----------|
| **Mastra** | `createTool()` function + Zod schema | Zod → JSON Schema | Async with validation |
| **VoltAgent** | `createTool()` + Zod + lifecycle hooks | Zod → JSON Schema | Hooks: before/after/onCancel |
| **OpenClaw** | Convention-based (file as tool) | Manual + auto | Sandboxed execution |
| **GoClaw** | Go interface `Tool` | Struct tags → JSON | Permission-gated |
| **ZeroClaw** | **Rust trait** `Tool` | Serde → JSON Schema | Sandboxed + approval-gated |

> ### 💡 Key Learning: **3-layer Tool Architecture**
> 
> ```
> Layer 1: Tool Definition (dev writes)
>    #[derive(Tool)]
>    struct WebSearch { query: String }
>    
> Layer 2: Tool Registry (framework manages)  
>    - Auto JSON schema generation
>    - Tool discovery + validation
>    - MCP tool integration
>    
> Layer 3: Tool Execution (framework runs)
>    - Sandboxing (optional)
>    - Human approval (optional)
>    - Retry + error handling
>    - Lifecycle hooks (before/after)
> ```
> 
> **Pattern từ VoltAgent đáng học:** Tool lifecycle hooks (`onStart`, `onComplete`, `onError`, `onCancel`).
> **Pattern từ GoClaw đáng học:** Permission-gated tools (per-agent, per-channel permissions).

---

### 🟠 Pattern 4: Memory Architecture

| Framework | Short-term | Working | Long-term | RAG |
|-----------|-----------|---------|-----------|-----|
| **Mastra** | Conversation history | Working memory (auto-managed) | Semantic recall | Vector store abstraction |
| **VoltAgent** | In-memory buffer | LibSQL adapter | Pluggable adapters | Retriever agent |
| **OpenClaw** | Session-scoped | File-based context | Memory commands | — |
| **GoClaw** | Per-session PostgreSQL | Context files | Knowledge graph | pgvector |
| **ZeroClaw** | Session-scoped | Context files | Recall/store/forget commands | RAG module |

> ### 💡 Key Learning: **Layered Memory với Trait Abstraction**
> 
> ```rust
> // Short-term: auto-managed conversation buffer
> pub trait ConversationMemory: Send + Sync {
>     async fn messages(&self, session_id: &str) -> Result<Vec<Message>>;
>     async fn append(&self, session_id: &str, msg: Message) -> Result<()>;
>     async fn truncate(&self, session_id: &str, max_tokens: usize) -> Result<()>;
> }
> 
> // Working: task-specific context
> pub trait WorkingMemory: Send + Sync {
>     async fn get(&self, key: &str) -> Result<Option<Value>>;
>     async fn set(&self, key: &str, value: Value) -> Result<()>;
> }
> 
> // Long-term: persistent knowledge
> pub trait KnowledgeStore: Send + Sync {
>     async fn store(&self, entry: KnowledgeEntry) -> Result<()>;
>     async fn recall(&self, query: &str, limit: usize) -> Result<Vec<KnowledgeEntry>>;
>     async fn forget(&self, id: &str) -> Result<()>;
> }
> ```
> 
> **Pattern từ Mastra đáng học:** Auto-managed working memory — LLM tự quyết định lưu gì.
> **Pattern từ GoClaw đáng học:** Knowledge graph cho structured knowledge.

---

### 🔴 Pattern 5: Hooks / Middleware / Lifecycle

**Đây là pattern phân biệt framework tốt vs framework cơ bản.**

| Framework | Pattern | Intercept Points |
|-----------|---------|-----------------|
| **Mastra** | Hooks system (`hooks/`) | Before/after agent run |
| **VoltAgent** | Tool lifecycle hooks | Tool: start/complete/error/cancel |
| **OpenClaw** | Lifecycle hooks (`hooks/`) | Before/after: LLM call, tool exec, message send |
| **GoClaw** | Middleware chain | Request → middleware chain → handler |
| **ZeroClaw** | Lifecycle hooks (`hooks/`) | **Every stage**: LLM call, tool exec, message |

> ### 💡 Key Learning: **Tower-style Middleware + Lifecycle Hooks**
> 
> ```rust
> // Middleware (inspired by Tower/Axum) — composable layers
> pub trait AgentMiddleware: Send + Sync {
>     async fn process(&self, req: AgentRequest, next: Next) -> Result<AgentResponse>;
> }
> 
> // Usage: agent.layer(LoggingLayer).layer(RateLimitLayer).layer(GuardrailLayer)
> 
> // Lifecycle hooks (inspired by OpenClaw/ZeroClaw) — event-based
> pub trait AgentHooks: Send + Sync {
>     async fn on_message_received(&self, msg: &Message) -> Result<()> { Ok(()) }
>     async fn before_llm_call(&self, request: &mut CompletionRequest) -> Result<()> { Ok(()) }
>     async fn after_llm_call(&self, response: &CompletionResponse) -> Result<()> { Ok(()) }
>     async fn before_tool_call(&self, tool: &str, input: &Value) -> Result<ToolDecision> { Ok(ToolDecision::Proceed) }
>     async fn after_tool_call(&self, tool: &str, output: &Value) -> Result<()> { Ok(()) }
>     async fn on_error(&self, error: &AgentError) -> Result<ErrorAction> { Ok(ErrorAction::Propagate) }
> }
> ```
> 
> **Both patterns cần thiết:**
> - **Middleware** cho cross-cutting concerns (logging, rate-limiting, caching)
> - **Hooks** cho business logic (approval, guardrails, cost tracking)

---

### 🟣 Pattern 6: Multi-Agent Orchestration

| Framework | Pattern | Key Feature |
|-----------|---------|------------|
| **Mastra** | Agent-to-Agent protocol (A2A) | Standard protocol |
| **VoltAgent** | Supervisor + Sub-Agents | Typed supervisor routing |
| **OpenClaw** | Multi-agent routing | Route channels → agents |
| **GoClaw** | **Agent Teams** + delegation | Task boards, team mailbox, blocked_by deps |
| **ZeroClaw** | **Hands** (agent swarms) | Autonomous swarms on schedule |

> ### 💡 Key Learning: **3 orchestration models cần support**
> 
> ```rust
> // Model 1: Sequential Pipeline
> let pipeline = Pipeline::new()
>     .step(researcher)
>     .step(writer)
>     .step(reviewer);
> 
> // Model 2: Supervisor (VoltAgent pattern)
> let team = Supervisor::new(manager_agent)
>     .worker(researcher)
>     .worker(writer)
>     .routing(|task| match task.type_ {
>         TaskType::Research => "researcher",
>         TaskType::Write => "writer",
>     });
> 
> // Model 3: Autonomous Swarm (GoClaw/ZeroClaw pattern)
> let swarm = Swarm::new()
>     .agent(agent1)
>     .agent(agent2)
>     .shared_task_board(TaskBoard::new())
>     .strategy(SwarmStrategy::Autonomous);
> ```

---

### 🟤 Pattern 7: Workflow Engine

| Framework | Approach | Syntax |
|-----------|----------|--------|
| **Mastra** | **Graph-based DAG** | `.then()`, `.branch()`, `.parallel()` |
| **VoltAgent** | Declarative workflows | Step-based with conditions |
| **OpenClaw** | — (no explicit workflow engine) | — |
| **GoClaw** | Cron + scheduling | `at`, `every`, `cron` expressions |
| **ZeroClaw** | **SOPs** (Standard Operating Procedures) | Event-driven workflows |

> ### 💡 Key Learning: **Mastra's Workflow DSL là gold standard**
> 
> ```rust
> // Fluent graph-based workflow (inspired by Mastra)
> let workflow = Workflow::new("content_pipeline")
>     .step("research", research_agent)
>     .step("draft", draft_agent)
>     .then("research", "draft")                     // sequential
>     .branch("draft", |output| {                     // conditional
>         if output.quality_score > 0.8 { "publish" }
>         else { "revise" }
>     })
>     .step("revise", revise_agent)
>     .step("publish", publish_agent)
>     .then("revise", "draft")                        // loop back
>     .parallel("research", vec!["web", "db", "api"]) // parallel
>     .suspend("approve", SuspendConfig::human_approval()) // human-in-the-loop
>     .build();
> ```
> 
> **Pattern từ ZeroClaw đáng học:** SOPs = event-driven workflows (trigger: webhook, cron, MQTT, peripheral).

---

## 3. Core vs Replaceable vs Extensible

### 🔴 CORE — Không thể thiếu, framework PHẢI provide

| Component | Mô tả | Tại sao Core? |
|-----------|--------|---------------|
| **Agent Runtime** | Vòng lặp think→act→observe | Trái tim của framework |
| **Provider Trait** | Interface để gọi LLM | Mọi agent cần LLM |
| **Tool Trait** | Interface để define tools | Agent không có tool = chatbot |
| **Message Types** | Request/Response/Stream types | Type safety foundation |
| **Error Types** | Structured error hierarchy | Dev cần handle errors |
| **Streaming** | SSE/Stream support | UX hiện đại cần streaming |
| **Configuration** | Agent builder/config | DX foundation |

### 🟡 REPLACEABLE — Framework provide default, user có thể swap

| Component | Default | Alternatives | Trait/Interface |
|-----------|---------|-------------|-----------------|
| **Memory** | In-memory (Vec) | SQLite, PostgreSQL, Redis | `trait Memory` |
| **LLM Provider** | OpenAI-compatible | Anthropic, Google, Ollama | `trait Provider` |
| **Logger** | `tracing` crate | Custom logger | `trait Logger` |
| **Serializer** | `serde_json` | MessagePack, Bincode | `trait Serializer` |
| **HTTP Client** | `reqwest` | `hyper`, custom | `trait HttpClient` |
| **Embedder** | OpenAI embeddings | Local models, Cohere | `trait Embedder` |

### 🟢 EXTENSIBLE — User thêm vào, framework không cần biết

| Component | Cách mở rộng | Ví dụ |
|-----------|-------------|-------|
| **Tools** | Implement `Tool` trait / `#[derive(Tool)]` | WebSearch, FileReader, DBQuery |
| **Middleware** | Implement `AgentMiddleware` | RateLimit, Cache, Guardrails |
| **Hooks** | Implement `AgentHooks` | CostTracker, AuditLog |
| **Workflows** | Compose agents vào Workflow | ContentPipeline, DataETL |
| **Multi-agent** | Combine agents vào Team/Swarm | ResearchTeam, CodeReviewSwarm |
| **MCP Tools** | Connect MCP servers | Any MCP-compatible tool |
| **Channels** | Implement `Channel` trait | Telegram, Discord, WhatsApp |
| **RAG Sources** | Implement `Retriever` trait | VectorDB, SearchEngine, FileSystem |

---

## 4. Proposed Architecture — TraitClaw

```
┌──────────────────────────────────────────────────────────────────┐
│                          User Code                               │
│  Agent::builder().model(openai("gpt-4o")).tool(Search).build()   │
└──────────────────────────────────────┬───────────────────────────┘
                                       │
┌──────────────────────────────────────▼───────────────────────────┐
│                     traitclaw (meta-crate)                         │
│                   pub use traitclaw_core::*;                       │
│                   pub use traitclaw_macros::*;                     │
├──────────────────────────────────────────────────────────────────┤
│                                                                   │
│  ┌─────────────────── CORE (traitclaw-core) ───────────────────┐  │
│  │                                                             │  │
│  │  ┌─────────┐  ┌──────────┐  ┌────────┐  ┌──────────────┐  │  │
│  │  │ Agent   │  │ Provider │  │ Tool   │  │ Memory       │  │  │
│  │  │ struct  │  │ trait    │  │ trait  │  │ trait        │  │  │
│  │  │         │  │          │  │        │  │              │  │  │
│  │  │•builder │  │•complete │  │•name   │  │•conversation │  │  │
│  │  │•run     │  │•stream   │  │•desc   │  │•working      │  │  │
│  │  │•stream  │  │•model_   │  │•schema │  │•knowledge    │  │  │
│  │  │•config  │  │ info     │  │•execute│  │              │  │  │
│  │  └─────────┘  └──────────┘  └────────┘  └──────────────┘  │  │
│  │                                                             │  │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌───────────┐  │  │
│  │  │ Runtime  │  │ Message  │  │ Stream   │  │ Error     │  │  │
│  │  │          │  │ types    │  │ types    │  │ types     │  │  │
│  │  │•loop     │  │          │  │          │  │           │  │  │
│  │  │•strategy │  │•Request  │  │•SSE      │  │•Provider  │  │  │
│  │  │•hooks    │  │•Response │  │•Chunk    │  │•Tool      │  │  │
│  │  │•middleware│ │•ToolCall │  │•Event    │  │•Runtime   │  │  │
│  │  └──────────┘  └──────────┘  └──────────┘  └───────────┘  │  │
│  │                                                             │  │
│  │  ┌──────────────────────────────────────────────────────┐   │  │
│  │  │ Middleware Pipeline (Tower-inspired)                  │   │  │
│  │  │ agent.layer(Logging).layer(RateLimit).layer(Guard)   │   │  │
│  │  └──────────────────────────────────────────────────────┘   │  │
│  └─────────────────────────────────────────────────────────────┘  │
│                                                                   │
│  ┌─────────── MACROS (traitclaw-macros) ───────────────────────┐  │
│  │  #[derive(Tool)]     — auto Tool trait + JSON schema       │  │
│  │  #[tool]             — function → Tool                     │  │
│  │  #[derive(Agent)]    — declarative agent definition        │  │
│  │  prompt!()           — type-safe prompt templates          │  │
│  └────────────────────────────────────────────────────────────┘  │
│                                                                   │
│  ┌─────────── PROVIDERS (separate crates) ────────────────────┐  │
│  │  traitclaw-openai        │ Native OpenAI API                │  │
│  │  traitclaw-anthropic     │ Native Anthropic (prompt cache)  │  │
│  │  traitclaw-google        │ Gemini API                       │  │
│  │  traitclaw-ollama        │ Local models                     │  │
│  │  traitclaw-openai-compat │ Any OpenAI-compatible endpoint   │  │
│  └────────────────────────────────────────────────────────────┘  │
│                                                                   │
│  ┌─────────── MEMORY (separate crates) ───────────────────────┐  │
│  │  traitclaw-memory-inmem  │ Default in-memory (included)     │  │
│  │  traitclaw-memory-sqlite │ SQLite persistent                │  │
│  │  traitclaw-memory-pg     │ PostgreSQL + pgvector            │  │
│  │  traitclaw-memory-redis  │ Redis for distributed            │  │
│  └────────────────────────────────────────────────────────────┘  │
│                                                                   │
│  ┌─────────── EXTENSIONS (separate crates) ───────────────────┐  │
│  │  traitclaw-mcp           │ MCP client + server              │  │
│  │  traitclaw-rag           │ RAG pipeline + embeddings        │  │
│  │  traitclaw-workflow      │ Graph-based workflow engine      │  │
│  │  traitclaw-team          │ Multi-agent orchestration        │  │
│  │  traitclaw-eval          │ Agent evaluation framework       │  │
│  │  traitclaw-server        │ HTTP/WS server for agents        │  │
│  │  traitclaw-tracing       │ OpenTelemetry integration        │  │
│  └────────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────┘
```

---

## 5. Patterns Đặc Biệt Đáng Học

### 🏆 #1: ZeroClaw's Trait-Everything Architecture

ZeroClaw viết bằng Rust, mọi component đều là trait → swap bất kỳ phần nào.

```rust
// Mọi thứ đều swappable thông qua trait
// Provider → trait Provider
// Channel → trait Channel  
// Tool → trait Tool
// Memory → trait Memory
// Tunnel → trait Tunnel
// Peripheral → trait Peripheral
```

**Áp dụng cho TraitClaw:** Mọi component trong core PHẢI là trait. Framework cung cấp default implementations, user swap bằng cách implement trait.

---

### 🏆 #2: Mastra's Dependency Injection Container

Mastra có `di/` module — dependency injection container quản lý tất cả services.

```typescript
// Mastra way: container quản lý tất cả
const mastra = new Mastra({
  agents: { myAgent },
  tools: { search, browser },
  memory: new PostgresMemory(),
  logger: new PinoLogger(),
});
```

**Áp dụng cho TraitClaw:** `TraitClaw` struct là DI container.

```rust
let app = TraitClaw::builder()
    .agent("researcher", researcher)
    .agent("writer", writer)
    .provider("openai", OpenAI::new(api_key))
    .memory(SqliteMemory::new("./db"))
    .middleware(TracingMiddleware::new())
    .build();

// Access via app
app.agent("researcher").run("query").await?;
```

---

### 🏆 #3: GoClaw's Permission System

GoClaw có 5-layer permission system:

```
Gateway Auth → Global Tool Policy → Per-Agent → Per-Channel → Owner-only
```

**Áp dụng cho TraitClaw:** `Guardrails` trait.

```rust
pub trait Guardrail: Send + Sync {
    /// Check if an action is allowed
    async fn check(&self, action: &AgentAction, context: &Context) -> Result<Decision>;
}

pub enum Decision {
    Allow,
    Deny(String),
    RequireApproval(ApprovalConfig),
}
```

---

### 🏆 #4: OpenClaw's Hook System

OpenClaw has hooks at EVERY interaction point:

```
before_message → before_context_assembly → before_llm_call → 
after_llm_call → before_tool_call → after_tool_call → 
before_response → after_response
```

**Áp dụng cho TraitClaw:** Rich lifecycle hooks.

---

### 🏆 #5: VoltAgent's Resumable Streaming

Client disconnect → reconnect → continue receiving same response.

**Áp dụng cho TraitClaw:** Stream có unique ID, server buffer cho reconnect.

```rust
let stream = agent.stream("query").await?;
let stream_id = stream.id(); // Save this

// Later, reconnect
let stream = agent.resume_stream(stream_id).await?;
```

---

### 🏆 #6: Mastra's Human-in-the-Loop Suspend/Resume

Workflow suspends → waits for human input → resumes from exact point.

**Áp dụng cho TraitClaw:**

```rust
let workflow = Workflow::new("review")
    .step("analyze", analyzer)
    .suspend("human_review") // ← pause here
    .step("act", actor);

// Execute — stops at suspend point
let handle = workflow.run(input).await?;
assert!(handle.is_suspended());

// Later — resume with human input  
let result = handle.resume(human_decision).await?;
```

---

## 6. Tóm Tắt — Checklist cho TraitClaw Architecture

### Phải có trong Core:
- [x] Agent struct với builder pattern
- [x] Provider trait (LLM abstraction)
- [x] Tool trait + derive macro
- [x] Memory trait (3 layers)
- [x] Runtime engine (loop strategies)
- [x] Streaming (SSE + chunks)
- [x] Middleware pipeline (Tower-style)
- [x] Lifecycle hooks
- [x] Error hierarchy
- [x] Message types

### Replaceable (default + swap):
- [x] Memory backends (in-mem → SQLite → PG → Redis)
- [x] Providers (OpenAI → Anthropic → Google → Ollama)
- [x] Logger (tracing → custom)
- [x] HTTP client (reqwest → custom)

### Extensible (user adds):
- [x] Custom tools (#[derive(Tool)])
- [x] Custom middleware
- [x] Workflows (workflow engine crate)
- [x] Multi-agent (team crate)
- [x] MCP integration (mcp crate)
- [x] RAG (rag crate)
- [x] Eval (eval crate)
- [x] HTTP server (server crate)
- [x] Channels (channel crate cho messaging platforms)
