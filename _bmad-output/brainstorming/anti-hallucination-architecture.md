# 🛡️ TraitClaw — Anti-Hallucination Architecture

> Câu hỏi: Ở đâu trong framework có thể chống ảo giác? Dev mở rộng/tăng cường ở đâu?

---

## Hallucination xảy ra ở đâu trong Agent Loop?

```
                    ┌─────────────────────────────────┐
                    │         AGENT LOOP               │
                    │                                   │
  User Input ──────▶│ 1. CONTEXT ASSEMBLY ◄──── ⚠️ Thiếu context = hallucinate
                    │    ├─ memory recall                │
                    │    ├─ RAG retrieval                 │
                    │    └─ system prompt                 │
                    │                                   │
                    │ 2. PROMPT CONSTRUCTION ◄── ⚠️ Prompt mơ hồ = hallucinate
                    │    ├─ system + user messages       │
                    │    ├─ tool schemas                  │
                    │    └─ output format                 │
                    │                                   │
                    │ 3. LLM CALL ◄──────────── ⚠️ Model tự bịa = hallucinate
                    │    └─ provider.complete(request)    │
                    │                                   │
                    │ 4. RESPONSE PARSING ◄───── ⚠️ Parse sai = sai hành vi
                    │    ├─ text response                 │
                    │    ├─ tool calls                    │
                    │    └─ structured output             │
                    │                                   │
                    │ 5. TOOL EXECUTION ◄─────── ⚠️ Tool sai = sai kết quả
                    │    └─ observe result                │
                    │                                   │
                    │ 6. OUTPUT VALIDATION ◄──── ⚠️ Không verify = ảo giác lọt ra
                    │    └─ return to user                │
                    │                                   │
                    └─────────────────────────────────┘
```

**→ Mỗi bước đều có thể tạo ra hoặc chặn hallucination.**

---

## Framework Components chống Hallucination

### Tổng quan: 8 tầng phòng thủ

```
┌──────────────────────────────────────────────────────────────┐
│ Layer 8: EVAL & MONITORING        (phát hiện sau deploy)     │
├──────────────────────────────────────────────────────────────┤
│ Layer 7: MULTI-AGENT VALIDATION   (agent kiểm tra agent)     │
├──────────────────────────────────────────────────────────────┤
│ Layer 6: OUTPUT GUARDRAILS        (chặn trước khi trả user)  │
├──────────────────────────────────────────────────────────────┤
│ Layer 5: STRUCTURED OUTPUT        (buộc format, giảm bịa)    │
├──────────────────────────────────────────────────────────────┤
│ Layer 4: TOOL VERIFICATION        (verify kết quả tool)      │
├──────────────────────────────────────────────────────────────┤
│ Layer 3: RAG GROUNDING            (ground vào dữ liệu thật)  │
├──────────────────────────────────────────────────────────────┤
│ Layer 2: PROMPT ENGINEERING       (hướng dẫn rõ ràng)        │
├──────────────────────────────────────────────────────────────┤
│ Layer 1: CONTEXT MANAGEMENT       (đúng context = ít bịa)    │
└──────────────────────────────────────────────────────────────┘
```

---

### Layer 1: Context Management (Framework provides)

**Vấn đề:** LLM bịa khi thiếu thông tin. Cung cấp đúng context giảm hallucination đáng kể.

**Component trong TraitClaw:**

```rust
/// Memory trait — 3 tầng context
pub trait Memory: Send + Sync {
    /// Lịch sử hội thoại gần nhất
    async fn conversation(&self, session_id: &str, limit: usize) -> Result<Vec<Message>>;
    
    /// Working memory — context hiện tại của task
    async fn working(&self, session_id: &str) -> Result<WorkingContext>;
    
    /// Long-term recall — tìm kiếm semantic
    async fn recall(&self, query: &str, limit: usize) -> Result<Vec<MemoryEntry>>;
}

/// Context assembler — tổng hợp context cho mỗi LLM call
pub struct ContextAssembler {
    memory: Arc<dyn Memory>,
    retrievers: Vec<Arc<dyn Retriever>>,   // RAG sources
    max_context_tokens: usize,              // Không vượt quá context window
    relevance_threshold: f32,               // Chỉ dùng context đủ relevant
}
```

**Dev mở rộng:** Implement `Memory` trait với backend khác, thêm `Retriever` sources.

---

### Layer 2: Prompt Engineering (Framework provides templates)

**Vấn đề:** Prompt mơ hồ → LLM đoán → hallucinate.

**Component trong TraitClaw:**

```rust
/// Built-in anti-hallucination prompt patterns
pub struct PromptBuilder {
    system_instructions: String,
    anti_hallucination_rules: Vec<String>,  // Framework adds these automatically
    output_schema: Option<JsonSchema>,
    citations_required: bool,
}

impl PromptBuilder {
    /// Framework tự thêm anti-hallucination rules
    pub fn with_grounding(mut self) -> Self {
        self.anti_hallucination_rules.extend([
            "Only answer based on the provided context.".into(),
            "If the context doesn't contain enough information, say 'I don't have enough information'.".into(),
            "Never fabricate facts, URLs, dates, or statistics.".into(),
            "Cite the source for every factual claim.".into(),
        ]);
        self
    }
    
    /// Buộc các model trả lời có citations
    pub fn require_citations(mut self) -> Self {
        self.citations_required = true;
        self
    }
}
```

**Dev mở rộng:** Custom prompt rules, domain-specific instructions.

---

### Layer 3: RAG Grounding (Extensible crate)

**Vấn đề:** LLM chỉ biết training data → cần external knowledge để ground.

**Component: `traitclaw-rag`**

```rust
/// Retriever trait — source bất kỳ
pub trait Retriever: Send + Sync {
    /// Tìm documents liên quan đến query
    async fn retrieve(&self, query: &str, opts: RetrieveOptions) -> Result<Vec<Document>>;
}

/// Grounding strategy — cách dùng retrieved docs
pub enum GroundingStrategy {
    /// Chỉ trả lời nếu tìm thấy evidence
    StrictEvidence { min_confidence: f32 },
    /// Dùng docs làm context nhưng cho phép suy luận  
    ContextualReasoning,
    /// No evidence → tự động abstain
    AbstainIfNoEvidence,
}

/// Built-in retrievers
pub struct VectorRetriever { /* vector DB */ }
pub struct KeywordRetriever { /* BM25 search */ }
pub struct HybridRetriever { /* vector + keyword + rerank */ }
pub struct GraphRetriever { /* knowledge graph */ }
```

**Dev mở rộng:** Custom `Retriever` (database, API, file system), custom `GroundingStrategy`.

---

### Layer 4: Tool Verification (Framework provides hooks)

**Vấn đề:** LLM gọi tool với arguments sai, hoặc interpret kết quả sai.

**Component trong TraitClaw:**

```rust
/// Tool trait với built-in verification
pub trait Tool: Send + Sync {
    type Input: DeserializeOwned + JsonSchema + Validate; // ← auto-validate input
    type Output: Serialize;
    
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    
    /// Framework validates Input trước khi execute
    async fn execute(&self, input: Self::Input) -> Result<Self::Output>;
    
    /// Optional: verify output sau execute
    fn verify_output(&self, output: &Self::Output) -> Result<VerificationResult> {
        Ok(VerificationResult::Accepted)
    }
}

/// Input validation — Rust type system chống hallucinate arguments
/// LLM bịa URL? → Url type validate format
/// LLM bịa số? → Range<i32> validate range
/// LLM bịa enum? → Rust enum chỉ accept known values
#[derive(Tool, Validate)]
#[tool(description = "Search the web")]
struct WebSearch {
    #[validate(length(min = 1, max = 500))]
    query: String,
    
    #[validate(range(min = 1, max = 50))]
    max_results: u32,
}
```

**Dev mở rộng:** Custom `Validate` rules, custom `verify_output()`.

---

### Layer 5: Structured Output (Framework provides)

**Vấn đề:** Free-form text → LLM bịa thoải mái. Structured output → giới hạn creativity.

**Component trong TraitClaw:**

```rust
/// Structured output — Rust struct IS the output schema
/// LLM PHẢI trả về đúng format → giảm bịa
#[derive(Deserialize, JsonSchema)]
pub struct FactCheckResult {
    pub claim: String,
    pub is_supported: bool,       // phải trả true/false, không bịa được
    pub evidence: Vec<Citation>,  // phải có citation
    pub confidence: f32,          // phải tự đánh giá confidence
}

#[derive(Deserialize, JsonSchema)]
pub struct Citation {
    pub source: String,
    pub quote: String,  // trích dẫn chính xác từ source
    pub relevance: f32,
}

/// Agent với structured output
let agent = Agent::builder()
    .model(openai("gpt-4o"))
    .output_schema::<FactCheckResult>()  // ← buộc format
    .build();

// LLM trả về FactCheckResult thay vì free text
let result: FactCheckResult = agent.run_structured("Is Rust memory-safe?").await?;
```

**Dev mở rộng:** Custom output types, domain-specific schemas.

---

### Layer 6: Output Guardrails (Extensible middleware)

**Vấn đề:** Chặn hallucination trước khi trả cho user. Tuyến phòng thủ cuối cùng.

**Component trong TraitClaw:**

```rust
/// Guardrail trait — chặn/sửa output trước khi trả user
pub trait Guardrail: Send + Sync {
    async fn check(&self, output: &AgentOutput, context: &Context) -> Result<GuardrailDecision>;
}

pub enum GuardrailDecision {
    /// Output OK, cho qua
    Pass,
    /// Output có vấn đề, sửa lại
    Modify(String),
    /// Output nguy hiểm, block hoàn toàn
    Block(String),
    /// Cần human review
    RequireReview(String),
    /// Gửi lại cho LLM với feedback
    Retry { feedback: String, max_retries: u32 },
}

// ═══════ BUILT-IN GUARDRAILS ═══════

/// Chặn response không có evidence
pub struct EvidenceGuardrail {
    pub min_citations: usize,
    pub require_source_urls: bool,
}

/// Phát hiện confident-nhưng-sai pattern
pub struct ConfidenceCalibrationGuardrail {
    /// Nếu agent quá tự tin nhưng query mơ hồ → flag
    pub flag_overconfident: bool,
}

/// Chặn fabricated URLs, emails, số liệu
pub struct FabricationDetector {
    pub check_urls: bool,      // verify URL format + domain exists
    pub check_emails: bool,    // verify email format
    pub check_statistics: bool, // flag unverified numbers
}

/// Consistency check — so sánh với context
pub struct ConsistencyGuardrail {
    /// So sánh response vs provided context
    pub check_context_alignment: bool,
    /// Flag thông tin không có trong context
    pub flag_unsupported_claims: bool,
}

// ═══════ USAGE ═══════

let agent = Agent::builder()
    .model(openai("gpt-4o"))
    .guardrail(EvidenceGuardrail { min_citations: 1, require_source_urls: true })
    .guardrail(FabricationDetector::default())
    .guardrail(ConsistencyGuardrail::default())
    .build();
```

**Dev mở rộng:** Custom `Guardrail` cho domain-specific rules (medical, legal, financial).

---

### Layer 7: Multi-Agent Verification (Extensible pattern)

**Vấn đề:** 1 agent tự kiểm tra mình = biased. Agent khác kiểm tra = objective hơn.

**Component trong TraitClaw:**

```rust
/// Chain-of-Verification pattern
pub struct VerificationChain {
    generator: Arc<dyn AgentBehavior>,   // Agent tạo response
    verifier: Arc<dyn AgentBehavior>,    // Agent kiểm tra response
    strategy: VerificationStrategy,
}

pub enum VerificationStrategy {
    /// Generate → Verify → Accept/Reject
    SimpleVerify,
    
    /// Generate → Verify → Nếu fail → Generate lại với feedback
    VerifyAndRetry { max_retries: u32 },
    
    /// Generate → tạo câu hỏi kiểm tra → trả lời → so sánh
    ChainOfVerification,
    
    /// Generate → N agents vote → majority wins
    ConsensusVoting { num_voters: usize, threshold: f32 },
    
    /// Generate → check từng claim vs knowledge base
    ClaimByClaimVerification,
}

// ═══════ USAGE ═══════

let researcher = Agent::builder()
    .name("researcher")
    .model(anthropic("claude-sonnet"))
    .tool(WebSearch)
    .build();

let fact_checker = Agent::builder()
    .name("fact_checker")  
    .model(openai("gpt-4o"))
    .instructions("Verify every claim. Flag anything without evidence.")
    .build();

let verified_agent = VerificationChain::new(researcher, fact_checker)
    .strategy(VerificationStrategy::VerifyAndRetry { max_retries: 2 });

// Response sẽ được fact-check trước khi trả user
let result = verified_agent.run("What is Rust's market share?").await?;
```

**Dev mở rộng:** Custom verification strategies, domain-specific verifiers.

---

### Layer 8: Eval & Monitoring (Extensible crate)

**Vấn đề:** Phát hiện hallucination trong production. Continuous improvement.

**Component: `traitclaw-eval`**

```rust
/// Eval metric trait — đo lường chất lượng agent
pub trait EvalMetric: Send + Sync {
    fn name(&self) -> &str;
    async fn evaluate(&self, input: &str, output: &str, context: &EvalContext) -> Result<EvalScore>;
}

/// Built-in anti-hallucination eval metrics
pub struct HallucinationRate;     // % responses chứa fabricated info
pub struct Faithfulness;          // output có faithful vs context không?
pub struct AnswerRelevancy;       // output có trả lời đúng câu hỏi không?
pub struct ContextPrecision;      // context retrieved có relevant không?
pub struct ContextRecall;         // đã retrieve đủ context chưa?
pub struct CitationAccuracy;      // citations có chính xác không?

/// Eval runner
let eval = EvalSuite::new()
    .metric(Faithfulness::new())
    .metric(HallucinationRate::new())
    .metric(CitationAccuracy::new())
    .dataset(test_cases);

let report = eval.run(&agent).await?;
// Faithfulness: 0.94, HallucinationRate: 0.03, CitationAccuracy: 0.91
```

**Dev mở rộng:** Custom eval metrics, custom datasets, CI integration.

---

## Tổng hợp: Mapping Framework Components → Anti-Hallucination

| Layer | Component | Thuộc về | Dev mở rộng? |
|-------|-----------|---------|-------------|
| 1. Context | `Memory` trait, `ContextAssembler` | **Core** | ✅ Custom memory backends, retriever sources |
| 2. Prompt | `PromptBuilder`, anti-hallucination rules | **Core** | ✅ Custom rules, domain instructions |
| 3. RAG | `Retriever` trait, `GroundingStrategy` | **Extension** (`traitclaw-rag`) | ✅ Custom retrievers, grounding strategies |
| 4. Tool Verify | `Tool::verify_output()`, input validation | **Core** | ✅ Custom validation rules |
| 5. Structured | Output schema via Rust types + JsonSchema | **Core** | ✅ Custom output types |
| 6. Guardrails | `Guardrail` trait, built-in detectors | **Core** (trait) + **Extension** (implementations) | ✅ Custom guardrails |
| 7. Multi-Agent | `VerificationChain`, voting, CoVe | **Extension** (`traitclaw-team`) | ✅ Custom strategies |
| 8. Eval | `EvalMetric` trait, built-in metrics | **Extension** (`traitclaw-eval`) | ✅ Custom metrics, datasets |

---

## Lifecycle Hooks — Dev can intercept ANYWHERE

```rust
pub trait AgentHooks: Send + Sync {
    // ═══ LAYER 1: Context ═══
    /// Modify context before LLM sees it — thêm grounding rules
    async fn on_context_assembled(&self, ctx: &mut ContextBundle) -> Result<()> { Ok(()) }
    
    // ═══ LAYER 2: Prompt ═══  
    /// Modify prompt before sending — inject guardrail instructions
    async fn before_llm_call(&self, req: &mut CompletionRequest) -> Result<()> { Ok(()) }
    
    // ═══ LAYER 3-4: Response ═══
    /// Check response right after LLM returns — first chance to catch hallucination
    async fn after_llm_call(&self, resp: &CompletionResponse) -> Result<HookDecision> {
        Ok(HookDecision::Continue)
    }
    
    // ═══ LAYER 4: Tool ═══
    /// Validate tool call before execution — prevent bad tool args
    async fn before_tool_call(&self, name: &str, input: &Value) -> Result<HookDecision> {
        Ok(HookDecision::Continue)
    }
    
    /// Verify tool result — catch tool errors before LLM sees them
    async fn after_tool_call(&self, name: &str, output: &Value) -> Result<HookDecision> {
        Ok(HookDecision::Continue)
    }
    
    // ═══ LAYER 6: Final output ═══
    /// Last chance — guardrail check before returning to user
    async fn before_response(&self, output: &mut AgentOutput) -> Result<HookDecision> {
        Ok(HookDecision::Continue)
    }
}

pub enum HookDecision {
    Continue,                          // OK, proceed
    Modify(Value),                     // Replace with modified version
    Retry { feedback: String },        // Send back to LLM with feedback
    Block { reason: String },          // Stop entirely
    Escalate { to: String },           // Send to another agent/human
}
```

---

## So sánh: Các framework xử lý hallucination thế nào?

| Feature | Mastra | VoltAgent | OpenClaw | GoClaw | ZeroClaw | **TraitClaw** |
|---------|--------|-----------|----------|--------|----------|-------------|
| RAG/Grounding | ✅ Vector store | ✅ Retriever agent | ❌ | ✅ pgvector | ✅ RAG module | ✅ `Retriever` trait |
| Guardrails | ❌ | ✅ Runtime guardrails | ❌ | ❌ | ❌ | ✅ `Guardrail` trait |
| Structured Output | ✅ Zod schemas | ✅ Zod schemas | ❌ | ❌ | ❌ | ✅ Rust types + JsonSchema |
| Evals | ✅ Built-in evals | ✅ Eval framework | ❌ | ❌ | ❌ | ✅ `EvalMetric` trait |
| Lifecycle Hooks | ✅ Hooks | ✅ Tool hooks | ✅ Rich hooks | ✅ Middleware | ✅ Rich hooks | ✅ 6-point hooks |
| Multi-Agent Verify | ❌ | ✅ Supervisor | ❌ | ✅ Teams | ✅ Hands | ✅ `VerificationChain` |
| Tool Validation | ✅ Zod | ✅ Zod | ❌ | ✅ Go types | ✅ Rust types | ✅ Rust types + Validate |
| Citation Required | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ `require_citations()` |
| Confidence Score | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ Structured output field |

### 💡 TraitClaw Advantage: **Rust Type System = Built-in Hallucination Prevention**

TypeScript frameworks dùng Zod (runtime validation). TraitClaw dùng **Rust type system** = **compile-time guarantee**:

```rust
// Rust KHÔNG CHO PHÉP LLM trả về kiểu sai
// Nếu schema nói "is_verified: bool" → chỉ có thể là true/false
// Nếu schema nói "category: Category" → chỉ có thể là enum values
// Nếu schema nói "confidence: f32" → phải là số thực

// TypeScript (runtime error — phát hiện lúc chạy):
// const result = await agent.run(query); // có thể crash runtime

// Rust (compile-time + runtime validation — phát hiện sớm hơn):
// let result: FactCheck = agent.run_structured(query).await?; // type-safe
```
