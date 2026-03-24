---
stepsCompleted: [1, 2, 3, 4, 5, 6]
inputDocuments:
  - brainstorming-session-2026-03-23-210618.md
  - architecture-deep-dive.md
  - anti-hallucination-architecture.md
  - guard-hint-track-steering.md
workflowType: 'architecture'
project_name: 'TraitClaw'
user_name: 'Bangvu'
date: '2026-03-23'
---

# TraitClaw — Architecture Document

> Rust AI Agent Framework — Simple by default, powerful when needed.

---

## 1. Vision & Positioning

**TraitClaw** is a Rust library for building AI agents. Not a platform, not an application — a **library** that other developers use to build their own AI agents quickly.

| Attribute | Decision |
|-----------|---------|
| **Type** | Library/Framework (like Axum for web, TraitClaw for AI agents) |
| **Language** | Rust |
| **Target Users** | Developers building AI agent applications |
| **Core Value** | Simple 5-line start, progressively powerful, model-agnostic steering |
| **Differentiator** | Guard–Hint–Track model steering system + Rust type safety |

### Positioning Map

```
                    Framework (library)
                         │
                    ┌────┤
                    │ TraitClaw ◄── HERE
                    │    │
         Rig ──────┘    │
                        │
    ────────────────────┼──────────────────
    Simple              │              Full Platform
    (just LLM calls)    │              (everything built-in)
                        │
                   ┌────┘
                   │
         ZeroClaw ─┤── OpenClaw ── GoClaw
                   │
              Application
```

---

## 2. Architecture Overview

### Single Dependency Design

```toml
# Dev only needs this:
[dependencies]
traitclaw = { version = "0.1", features = ["openai"] }

# Progressive enhancement:
traitclaw = { version = "0.1", features = [
    "openai",      # LLM provider
    "sqlite",      # persistent memory
    "steering",    # Guard-Hint-Track
    "rag",         # RAG pipeline
    "mcp",         # MCP protocol
] }
```

### Internal Crate Architecture

```
traitclaw (meta-crate — re-exports everything)
│
├── traitclaw-core          CORE — traits, types, runtime
├── traitclaw-macros         CORE — #[derive(Tool)], #[tool], prompt!()
│
├── traitclaw-openai         feature = "openai"
├── traitclaw-anthropic      feature = "anthropic"
├── traitclaw-google         feature = "google"
├── traitclaw-ollama         feature = "ollama"
├── traitclaw-openai-compat  feature = "openai-compat"  (any compatible endpoint)
│
├── traitclaw-memory-sqlite  feature = "sqlite"
├── traitclaw-memory-pg      feature = "postgres"
├── traitclaw-memory-redis   feature = "redis"
│
├── traitclaw-steering       feature = "steering"  (Guard-Hint-Track)
├── traitclaw-rag            feature = "rag"
├── traitclaw-mcp            feature = "mcp"
├── traitclaw-team           feature = "team"       (multi-agent)
├── traitclaw-workflow       feature = "workflow"
├── traitclaw-eval           feature = "eval"
└── traitclaw-server         feature = "server"     (HTTP/WS server)
```

**Default features:** `["openai-compat", "macros"]` — enough for a working agent with zero config.

---

## 3. Core Traits (`traitclaw-core`)

### 3.1 Provider — LLM Abstraction

```rust
pub trait Provider: Send + Sync + 'static {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse>;
    async fn stream(&self, request: CompletionRequest) -> Result<CompletionStream>;
    fn model_info(&self) -> &ModelInfo;
}

pub struct ModelInfo {
    pub name: String,
    pub tier: ModelTier,          // Small / Medium / Large
    pub context_window: usize,
    pub supports_tools: bool,
    pub supports_vision: bool,
    pub supports_structured: bool,
}
```

**Rationale:** Trait-based (like ZeroClaw) for zero-cost abstraction. `ModelInfo` includes `tier` for Guard-Hint-Track auto-configuration.

### 3.2 Tool — Tool Definition

```rust
pub trait Tool: Send + Sync + 'static {
    type Input: DeserializeOwned + JsonSchema;
    type Output: Serialize;

    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn schema(&self) -> ToolSchema;  // auto-generated from Input type
    async fn execute(&self, input: Self::Input) -> Result<Self::Output>;
}

// Derive macro eliminates boilerplate:
#[derive(Tool)]
#[tool(description = "Search the web for information")]
struct WebSearch {
    /// The search query
    query: String,
    /// Maximum results to return
    #[tool(default = 10)]
    max_results: u32,
}

impl WebSearch {
    async fn execute(&self, input: WebSearchInput) -> Result<Vec<SearchResult>> {
        // implementation
    }
}
```

**Rationale:** Rust type system as schema truth. `#[derive(Tool)]` auto-generates JSON Schema from struct fields. No Zod, no runtime validation — compile-time guarantee.

### 3.3 Memory — 3-Layer Memory

```rust
pub trait Memory: Send + Sync + 'static {
    // Short-term: conversation history
    async fn messages(&self, session: &str) -> Result<Vec<Message>>;
    async fn append(&self, session: &str, msg: Message) -> Result<()>;

    // Working: task-specific context
    async fn get_context(&self, session: &str, key: &str) -> Result<Option<Value>>;
    async fn set_context(&self, session: &str, key: &str, val: Value) -> Result<()>;

    // Long-term: semantic recall
    async fn recall(&self, query: &str, limit: usize) -> Result<Vec<MemoryEntry>>;
    async fn store(&self, entry: MemoryEntry) -> Result<()>;
}
```

**Default:** In-memory implementation (included in core, zero deps).

### 3.4 Guard / Hint / Tracker — Steering Traits

```rust
/// Guard — hard boundary, blocks dangerous actions
pub trait Guard: Send + Sync {
    fn name(&self) -> &str;
    fn check(&self, action: &Action) -> GuardResult;  // NOT async — must be fast
}

/// Hint — inject guidance messages at the right time
pub trait Hint: Send + Sync {
    fn name(&self) -> &str;
    fn should_trigger(&self, state: &AgentState) -> bool;
    fn generate(&self, state: &AgentState) -> HintMessage;
    fn injection_point(&self) -> InjectionPoint;
}

/// Tracker — monitor runtime state, feed signals to Guard/Hint
pub trait Tracker: Send + Sync {
    fn on_iteration(&self, state: &mut AgentState);
    fn on_tool_call(&self, name: &str, args: &Value, state: &mut AgentState);
    fn on_llm_response(&self, response: &CompletionResponse, state: &mut AgentState);
    fn recommended_concurrency(&self, state: &AgentState) -> usize;
}
```

**Key design decision:** Traits are in Core (zero-cost when unused via NoopGuard/NoopHint/NoopTracker). Implementations are in `traitclaw-steering` optional crate.

### 3.5 Agent — The Main API

```rust
pub struct Agent {
    provider: Arc<dyn Provider>,
    tools: Vec<Arc<dyn ErasedTool>>,
    memory: Arc<dyn Memory>,
    guards: Vec<Arc<dyn Guard>>,
    hints: Vec<Arc<dyn Hint>>,
    tracker: Arc<dyn Tracker>,
    config: AgentConfig,
}

impl Agent {
    pub fn builder() -> AgentBuilder { ... }
    pub async fn run(&self, input: &str) -> Result<AgentOutput> { ... }
    pub async fn stream(&self, input: &str) -> Result<AgentStream> { ... }
    pub async fn run_structured<T: DeserializeOwned + JsonSchema>(
        &self, input: &str
    ) -> Result<T> { ... }
}
```

---

## 4. Agent Runtime — The Loop

```
┌────────────────────────────────────────────────────────┐
│                     AGENT RUNTIME                       │
│                                                         │
│  Input ──▶ Context Assembly ──▶ Build Request           │
│                 │                      │                │
│            Memory.messages()    System Prompt            │
│            Memory.recall()     + Tool Schemas            │
│                                + Hint Messages  ◄─ HINT │
│                                                         │
│            ┌─── LLM Call ◄──── Request ─────┘           │
│            │                                            │
│            ▼                                            │
│      Parse Response                                     │
│            │                                            │
│      ┌─────┴─────┐                                      │
│      │           │                                      │
│    Text      ToolCall                                   │
│      │           │                                      │
│      │     Guard.check() ◄──────────────────────  GUARD │
│      │           │                                      │
│      │     ┌─────┴─────┐                                │
│      │   Allow       Deny                               │
│      │     │           │                                 │
│      │   Execute    Inject                               │
│      │   Tool       Error                                │
│      │     │         Msg                                 │
│      │     ▼           │                                 │
│      │   Result ───────┘                                 │
│      │     │                                            │
│      │   Tracker.on_tool_call() ◄───────────────  TRACK │
│      │     │                                            │
│      │     └──▶ Loop back to LLM Call                   │
│      │                                                  │
│      ▼                                                  │
│   Output ──▶ Memory.append() ──▶ Return                 │
│                                                         │
└────────────────────────────────────────────────────────┘
```

---

## 5. Guard–Hint–Track Steering System

### Philosophy (from GoClaw battle-tested experience)

> **Don't rely on the model. Don't rely on prompts. Build infrastructure that steers.**

| Layer | Principle | When | Cost |
|-------|-----------|------|------|
| **Guard** | Don't trust model | Before every action | 0 tokens (regex/rules) |
| **Hint** | Collaborate with model | Mid-loop, inject messages | 1-2 iterations, saves 20+ |
| **Track** | Silently follow model | Continuous monitoring | 0 tokens (state tracking) |

### Model Tier Auto-Adaptation

| Config | Small (Haiku, Phi) | Medium (Sonnet, 4o-mini) | Large (Opus, GPT-4o) |
|--------|:---:|:---:|:---:|
| Hint frequency | Every 3 iter | Every 6 iter | Every 12 iter |
| Concurrency | 1 (serial) | 3 | Full |
| Context throttle | 50% | 60% | 80% |
| System prompt remind | Every 4 iter | Every 8 iter | Every 15 iter |

### Built-in Guards (in `traitclaw-steering`)

- `ShellDenyGuard` — 200+ regex patterns
- `PromptInjectionGuard` — injection detection
- `LoopDetectionGuard` — detect repetitive tool calls
- `ToolBudgetGuard` — max tool calls per turn
- `WorkspaceBoundaryGuard` — restrict file operations

### Built-in Hints (in `traitclaw-steering`)

- `BudgetHint` — warn at 75% token budget
- `TruncationHint` — explain output truncation
- `SystemPromptReminder` — re-inject key instructions at recency zone
- `TeamProgressHint` — remind progress reporting every N iterations

---

## 6. Core vs Optional

### 🔴 Core (`traitclaw-core` — always included)

| Component | Why Core |
|-----------|---------|
| `Agent` struct + builder | Heart of the framework |
| `Provider` trait | Every agent needs LLM |
| `Tool` trait | Agent without tools = chatbot |
| `Memory` trait + InMemory impl | Default memory, zero deps |
| Guard/Hint/Tracker **traits** | Hook points in runtime (Noop by default) |
| Message/Stream/Error types | Foundation types |
| Runtime loop engine | Execution engine |

### 🟡 Optional (feature flags)

| Feature | Crate | What it adds |
|---------|-------|-------------|
| `"openai"` | traitclaw-openai | OpenAI provider |
| `"anthropic"` | traitclaw-anthropic | Anthropic provider (prompt caching) |
| `"ollama"` | traitclaw-ollama | Local model support |
| `"sqlite"` | traitclaw-memory-sqlite | SQLite memory backend |
| `"steering"` | traitclaw-steering | Guard-Hint-Track implementations |
| `"rag"` | traitclaw-rag | Retriever trait + pipeline |
| `"mcp"` | traitclaw-mcp | MCP client/server |
| `"team"` | traitclaw-team | Multi-agent orchestration |
| `"workflow"` | traitclaw-workflow | Graph-based workflows |
| `"eval"` | traitclaw-eval | Agent evaluation metrics |
| `"server"` | traitclaw-server | HTTP/WS server for agents |

---

## 7. Developer Experience

### Progressive Complexity

```rust
// ═══ Level 1: Hello Agent (5 lines) ═══
use traitclaw::prelude::*;

let agent = Agent::builder()
    .model(openai("gpt-4o-mini"))
    .system("You are a helpful assistant")
    .build();

let response = agent.run("Hello!").await?;

// ═══ Level 2: + Tools (10 lines) ═══
#[derive(Tool)]
#[tool(description = "Search the web")]
struct WebSearch { query: String }
// impl execute...

let agent = Agent::builder()
    .model(openai("gpt-4o-mini"))
    .tool(WebSearch)
    .build();

// ═══ Level 3: + Memory (1 extra line) ═══
let agent = Agent::builder()
    .model(openai("gpt-4o-mini"))
    .tool(WebSearch)
    .memory(SqliteMemory::new("./agent.db"))  // +1 line
    .build();

// ═══ Level 4: + Steering (1 extra line) ═══
let agent = Agent::builder()
    .model(openai("gpt-4o-mini"))
    .tool(WebSearch)
    .memory(SqliteMemory::new("./agent.db"))
    .steering(Steering::auto())               // +1 line, auto-config by model tier
    .build();

// ═══ Level 5: Fine-grained control ═══
let agent = Agent::builder()
    .model(openai("gpt-4o-mini"))
    .tool(WebSearch)
    .memory(SqliteMemory::new("./agent.db"))
    .guard(ShellDenyGuard::default())
    .guard(LoopDetectionGuard::new(3))
    .hint(BudgetHint::at(0.75))
    .hint(SystemPromptReminder::every(4).rules(["Always cite sources"]))
    .tracker(AdaptiveTracker::for_model(ModelTier::Small))
    .build();
```

### Derive Macros

| Macro | Purpose |
|-------|---------|
| `#[derive(Tool)]` | Struct → Tool with auto JSON Schema |
| `#[tool]` | Function → Tool |

---

## 8. Examples & Showcase

### Progressive Examples

```
examples/
├── 01-hello-agent/          5 lines, basic chat
├── 02-tool-calling/         Agent + custom tools
├── 03-memory/               Persistent conversation
├── 04-streaming/            SSE streaming response
├── 05-structured-output/    Rust types as output schema
├── 06-steering/             Guard-Hint-Track demo
├── 07-multi-agent/          Agent team collaboration
├── 08-rag/                  RAG pipeline
├── 09-mcp-server/           Agent as MCP server
```

### Showcase: `miniclaw`

A mini OpenClaw/ZeroClaw built with TraitClaw (~500-1000 lines) proving the framework can build real agent applications.

**Progressive enhancement demo** — each step adds 10-30 lines:

| Step | Adds | Lines Added |
|------|------|:-----------:|
| 1. Basic | 1 agent, CLI channel | ~50 |
| 2. + Memory | SQLite persistence | +20 |
| 3. + Tools | Web search, file tools | +30 |
| 4. + Steering | Guard-Hint-Track | +10 |
| 5. + Multi-agent | Agent routing | +40 |
| 6. + Telegram | Channel trait impl | +50 |
| 7. + Custom Guard | Domain-specific guard | +20 |

**If adding each option only costs 10-30 lines → DX is proven good.**  
**If it requires refactoring → architecture is wrong.**

---

## 9. Technology Choices

| Area | Choice | Rationale |
|------|--------|-----------|
| Async runtime | `tokio` | Industry standard, best ecosystem |
| HTTP client | `reqwest` | Most popular, built on hyper |
| Serialization | `serde` + `serde_json` | Standard Rust |
| JSON Schema | `schemars` | Auto-generate from Rust types |
| CLI | `clap` | Standard Rust CLI framework |
| Logging | `tracing` | Structured, async-aware |
| Error handling | `thiserror` | Clean derive for error types |
| Proc macros | `syn` + `quote` | Standard for derive macros |
| Streaming | `tokio-stream` + SSE | Async streams |

### Provider Protocol Standard (AD-9)

All providers implement `CompletionRequest` / `CompletionResponse` in **OpenAI Chat Completions format** ([spec](https://platform.openai.com/docs/api-reference/chat)).

```
POST /v1/chat/completions
{ "model": "...", "messages": [...], "tools": [...], "stream": false }
```

| Provider | Implementation | Notes |
|----------|---------------|-------|
| OpenAI, Groq, Mistral, Ollama, vLLM, Azure | `traitclaw-openai-compat` | Zero transform — wire-compatible |
| OpenAI (native features) | `traitclaw-openai` | Extends compat with logprobs, etc. |
| Anthropic | `traitclaw-anthropic` | ~20 lines transform: `system` field, `input_tokens` → `prompt_tokens` |
| Google Gemini | `traitclaw-google` | Use Gemini's OpenAI-compat endpoint by default |

**No third-party LLM SDK is used.** Provider crates call the HTTP API directly via `reqwest`, keeping the dependency tree minimal.

---

## 10. MVP Roadmap

### Phase 1: Core (Week 1-2)

- [ ] `traitclaw-core`: Agent, Provider, Tool, Memory traits
- [ ] `traitclaw-macros`: `#[derive(Tool)]`
- [ ] `traitclaw-openai-compat`: OpenAI-compatible provider
- [ ] In-memory default memory
- [ ] Runtime loop (with Guard/Hint/Tracker hook points)
- [ ] Streaming support
- [ ] Examples 01-04

### Phase 2: Steering + Ecosystem (Week 3-4)

- [ ] `traitclaw-steering`: Guard-Hint-Track implementations
- [ ] `traitclaw-openai`: Native OpenAI provider
- [ ] `traitclaw-memory-sqlite`: SQLite memory
- [ ] Structured output support
- [ ] Examples 05-06

### Phase 3: Advanced (Week 5-8)

- [ ] `traitclaw-anthropic`: Anthropic provider
- [ ] `traitclaw-team`: Multi-agent orchestration
- [ ] `traitclaw-rag`: RAG pipeline
- [ ] `traitclaw-mcp`: MCP protocol
- [ ] `traitclaw-eval`: Evaluation framework
- [ ] `traitclaw-server`: HTTP/WS server
- [ ] Showcase: `miniclaw`
- [ ] Examples 07-09

---

## 11. Key Architectural Decisions

| # | Decision | Rationale |
|---|----------|-----------|
| AD-1 | Single `traitclaw` dependency with feature flags | Simple DX, dev doesn't choose between crates |
| AD-2 | Guard/Hint/Track traits in core, impls in optional crate | Zero-cost when unused, available when needed |
| AD-3 | Trait-based extensibility for all components | Rust zero-cost abstraction, dev swaps any part |
| AD-4 | `#[derive(Tool)]` for tool definition | Eliminate boilerplate, compile-time schema |
| AD-5 | In-memory default, everything else feature-gated | Zero deps for basic agents |
| AD-6 | ModelTier in ModelInfo | Auto-configure steering per model capability |
| AD-7 | Builder pattern for Agent | Progressive complexity, discoverable API |
| AD-8 | Showcase examples prove architecture | If miniclaw needs hacks → design is wrong |
| AD-9 | OpenAI Chat Completions wire format as internal standard | De facto industry standard; Ollama/Groq/vLLM/Azure are all compatible out-of-box. Anthropic needs a thin transform layer (~20 lines). Avoids SDK lock-in while staying standards-compliant. |
