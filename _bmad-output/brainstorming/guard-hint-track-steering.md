# 🚀 Guard–Hint–Track: Model Steering System cho TraitClaw

> Bài học từ GoClaw & kinh nghiệm thực chiến của Bangvu  
> Đây là kiến trúc **quan trọng nhất** của framework — phân biệt TraitClaw khỏi mọi framework khác.

---

## Tư duy cốt lõi: Đừng trông chờ vào Model

### Vấn đề thực tế mà ai làm agent đều biết:

| Model lớn (GPT-4o, Claude Opus) | Model nhỏ (GPT-4o-mini, Haiku, Gemma) |
|:---:|:---:|
| Thông minh, bám context tốt | Rẻ nhưng chạy hên xui |
| Tuân thủ system prompt | Bỏ system prompt khi context dài |
| Chi phí cao | Chi phí thấp |
| Vẫn "rời" system prompt ở context dài | IMPORTANT, in đậm, prompt engineering = vô ích |

**→ Prompt engineering KHÔNG giải quyết được.**  
**→ Giải pháp: Đẩy xuống tầng hạ tầng và kiến trúc.**

### Nguyên tắc GoClaw:

> **Guard** không tin model.  
> **Hint** cộng tác với model.  
> **Track** âm thầm đi theo model.

---

## 3 lớp chạy đồng thời

```
User Request ──▶ TRACK ──▶ GUARD ──▶ HINT ──▶ Agent Loop
                 Chạy ở     Được      Nên        ◄──┐
                 đâu?      làm gì?   làm gì?        │
                  │          │          │             │
                  │          │          │         ┌───┘
                  ▼          ▼          ▼         │
              Lane +     InputGuard  Budget +   Loop
              Queue      ShellDeny   Progress   iterating...
                         SkillGuard  Nudges
                                                  │
           ◄────────────── giám sát liên tục ──────┘
           Track luôn theo, Guard luôn chặn, Hint luôn nhắc
```

---

## GUARD — Hard boundary, chặn model đi sai

### Triết lý: **Không tin model. Không cần model hợp tác. Chặn cứng.**

### Tại sao GUARD là layer quan trọng nhất?

- Nhẹ: hệ thống bị hack
- Nặng: model gọi sai tool → mất rất nhiều tokens cho LLM Loops/Turns
- Model bị prompt injection → cho phép chạy shell command nguy hiểm
- Model sai workflow → đi vòng vèo tốn token vô nghĩa

### GoClaw Guard System:

| Guard Type | Mô tả | Ví dụ |
|-----------|--------|-------|
| Shell Deny | 200+ regex patterns scan shell commands | `rm -rf`, `sudo`, `chmod 777`, `curl \| bash` |
| Prompt Injection Detection | Phát hiện injection trong user input | "Ignore previous instructions..." |
| Skill Content Scan | Scan nội dung skill/tool output | Chặn XSS, SQL injection trong output |
| Workflow Guard | Chặn sai quy trình | Không cho tạo file ngoài workspace |
| Tool Call Guard | Validate tool arguments | Chặn tool call với args vô nghĩa |

### Áp dụng cho TraitClaw:

```rust
/// Guard — hard boundary, chạy TRƯỚC mọi action
pub trait Guard: Send + Sync {
    /// Tên guard (cho logging/tracing)
    fn name(&self) -> &str;
    
    /// Kiểm tra — return Deny nếu vi phạm
    /// KHÔNG async — phải chạy nhanh, microseconds
    fn check(&self, action: &Action) -> GuardResult;
}

pub enum GuardResult {
    /// Cho phép
    Allow,
    /// Chặn cứng — không thương lượng
    Deny { reason: String, severity: Severity },
    /// Sửa action rồi cho qua
    Sanitize { modified_action: Action, warning: String },
}

pub enum Severity {
    /// Log & block
    Critical,
    /// Block & alert developer
    High,
    /// Block & suggest alternative
    Medium,
}

/// Action — mọi thứ model muốn làm đều qua đây
pub enum Action {
    ToolCall { name: String, arguments: Value },
    ShellCommand { command: String },
    FileWrite { path: PathBuf, content: String },
    HttpRequest { url: String, method: String },
    AgentDelegation { to: String, task: String },
    RawOutput { content: String },
}

// ═══════ BUILT-IN GUARDS ═══════

/// Chặn shell commands nguy hiểm
pub struct ShellDenyGuard {
    patterns: Vec<Regex>,  // 200+ patterns từ GoClaw
}

impl Guard for ShellDenyGuard {
    fn name(&self) -> &str { "shell_deny" }
    fn check(&self, action: &Action) -> GuardResult {
        if let Action::ShellCommand { command } = action {
            for pattern in &self.patterns {
                if pattern.is_match(command) {
                    return GuardResult::Deny {
                        reason: format!("Dangerous command blocked: {}", command),
                        severity: Severity::Critical,
                    };
                }
            }
        }
        GuardResult::Allow
    }
}

/// Phát hiện prompt injection
pub struct PromptInjectionGuard {
    patterns: Vec<Regex>,
    // "Ignore previous instructions", "You are now DAN", etc.
}

/// Chặn file write ngoài workspace
pub struct WorkspaceBoundaryGuard {
    allowed_paths: Vec<PathBuf>,
}

/// Chặn tool call loop vô nghĩa
pub struct LoopDetectionGuard {
    /// Nếu cùng tool + cùng args được gọi N lần → block
    max_identical_calls: usize,
    recent_calls: Mutex<VecDeque<(String, u64)>>,  // (tool+args hash, timestamp)
}

/// Chặn gọi quá nhiều tools trong 1 turn
pub struct ToolBudgetGuard {
    max_tool_calls_per_turn: usize,
}

// ═══════ GUARD PIPELINE ═══════

/// GuardPipeline — chain nhiều guards, tất cả PHẢI pass
pub struct GuardPipeline {
    guards: Vec<Arc<dyn Guard>>,
}

impl GuardPipeline {
    /// Chạy tất cả guards — bất kỳ guard nào Deny → block
    pub fn check_all(&self, action: &Action) -> GuardResult {
        for guard in &self.guards {
            match guard.check(action) {
                GuardResult::Allow => continue,
                result @ GuardResult::Deny { .. } => return result,
                result @ GuardResult::Sanitize { .. } => return result,
            }
        }
        GuardResult::Allow
    }
}
```

**Dev mở rộng:** Implement `Guard` trait cho domain-specific guards (medical: chặn kê thuốc, finance: chặn giao dịch, v.v.)

---

## HINT — Inject context đúng lúc, cộng tác với model

### Triết lý: **GPS recalculate — cứ nhắc hoài cho khỏi lạc.**

### Đánh đổi thông minh:
> *"Thà mất thêm 1-2 iterations nhưng đổi lại tối ưu cả mấy chục cái đằng sau,  
> đặc biệt khi chạy Agent Teams."*

### GoClaw Hint System — 8 loại guidance:

Dựa trên diagram chi tiết của Bangvu:

| Hint Type | Trigger | Phase | Nội dung inject |
|-----------|---------|-------|----------------|
| **Budget Hints** | Dùng 75% budget | LLM iteration | "Bạn đã dùng 75% token budget. Hãy tổng hợp và kết luận." |
| **Output Truncation** | Output bị cắt vì max_tokens | LLM iteration | "Output đã bị cắt. Nguyên nhân: vượt max_tokens. Hãy chia nhỏ response." |
| **Skill Evolution Nudges** | Skill completion 70%/90% | LLM iteration | "Skill đã hoàn thành 70%. Hãy kiểm tra lại trước khi tiếp tục." |
| **Team Progress Nudges** | Mỗi 6 iterations | LLM iteration | "Nhắc: báo tiến độ cho team. Các thành viên đang chờ update." |
| **Sandbox Hints** | Error trong sandbox | Tool result | "Docker container errored. Nguyên nhân: [error]. Hãy sửa và thử lại." |
| **Channel Formatting** | Output sai format kênh | System prompt | "Bạn đang trả lời trên Zalo/Telegram. Format phù hợp: ..." |
| **Task Creation Guidance** | Tạo task cho team | Tool result | "Khi tạo task, phải có: title, description, assignee, deadline." |
| **System Prompt Reminders** | Context dài, model quên | System prompt (recency zone) | Re-inject key instructions vào cuối conversation |

### Áp dụng cho TraitClaw:

```rust
/// Hint — inject message vào conversation đúng lúc
pub trait Hint: Send + Sync {
    fn name(&self) -> &str;
    
    /// Kiểm tra có cần hint không — chạy mỗi iteration
    fn should_trigger(&self, state: &AgentState) -> bool;
    
    /// Tạo hint message để inject
    fn generate(&self, state: &AgentState) -> HintMessage;
    
    /// Hint inject ở đâu trong conversation?
    fn injection_point(&self) -> InjectionPoint;
}

pub enum InjectionPoint {
    /// Thêm vào system prompt (đầu conversation)
    SystemPrompt,
    /// Thêm như 1 message mới trước LLM call tiếp
    BeforeNextLlmCall,
    /// Thêm vào cuối conversation (recency zone — model chú ý nhất)
    RecencyZone,
    /// Thêm vào tool result
    AppendToToolResult { tool_name: String },
}

pub struct HintMessage {
    pub role: MessageRole,  // System hoặc Assistant
    pub content: String,
    pub priority: HintPriority,
}

pub enum HintPriority {
    /// Luôn inject
    Critical,
    /// Inject nếu còn token budget
    Normal,
    /// Chỉ inject nếu context chưa quá dài
    Low,
}

// ═══════ BUILT-IN HINTS ═══════

/// Budget hint — nhắc khi gần hết token budget
pub struct BudgetHint {
    /// Ngưỡng cảnh báo (0.0 - 1.0)
    warning_threshold: f32,  // default: 0.75
}

impl Hint for BudgetHint {
    fn name(&self) -> &str { "budget" }
    
    fn should_trigger(&self, state: &AgentState) -> bool {
        let usage = state.token_usage as f32 / state.token_budget as f32;
        usage >= self.warning_threshold
    }
    
    fn generate(&self, state: &AgentState) -> HintMessage {
        let pct = (state.token_usage as f32 / state.token_budget as f32 * 100.0) as u32;
        HintMessage {
            role: MessageRole::System,
            content: format!(
                "[BUDGET NOTICE] You have used {}% of your token budget ({}/{} tokens). \
                 Please summarize your findings and provide a conclusion. \
                 Avoid unnecessary elaboration.",
                pct, state.token_usage, state.token_budget
            ),
            priority: HintPriority::Critical,
        }
    }
    
    fn injection_point(&self) -> InjectionPoint {
        InjectionPoint::RecencyZone  // cuối conversation, model chú ý nhất
    }
}

/// Output truncation hint — giải thích khi output bị cắt
pub struct TruncationHint;

impl Hint for TruncationHint {
    fn name(&self) -> &str { "truncation" }
    
    fn should_trigger(&self, state: &AgentState) -> bool {
        state.last_output_truncated
    }
    
    fn generate(&self, _state: &AgentState) -> HintMessage {
        HintMessage {
            role: MessageRole::System,
            content: "[OUTPUT TRUNCATED] Your previous response was cut off due to max_tokens limit. \
                      Please continue from where you left off, or split your response into smaller parts.".into(),
            priority: HintPriority::Critical,
        }
    }
    
    fn injection_point(&self) -> InjectionPoint {
        InjectionPoint::BeforeNextLlmCall
    }
}

/// Team progress nudge — nhắc báo tiến độ
pub struct TeamProgressHint {
    every_n_iterations: usize,  // default: 6
}

impl Hint for TeamProgressHint {
    fn name(&self) -> &str { "team_progress" }
    
    fn should_trigger(&self, state: &AgentState) -> bool {
        state.is_team_task && state.iteration_count % self.every_n_iterations == 0
    }
    
    fn generate(&self, state: &AgentState) -> HintMessage {
        HintMessage {
            role: MessageRole::System,
            content: format!(
                "[TEAM PROGRESS] You are on iteration {}. \
                 Please report your current progress to the team. \
                 What have you completed? What's remaining?",
                state.iteration_count
            ),
            priority: HintPriority::Normal,
        }
    }
    
    fn injection_point(&self) -> InjectionPoint {
        InjectionPoint::BeforeNextLlmCall
    }
}

/// System prompt reminder — re-inject key instructions khi context dài
/// ĐÂY LÀ HINT QUAN TRỌNG NHẤT cho model nhỏ
pub struct SystemPromptReminder {
    /// Nhắc lại khi conversation dài hơn N tokens
    trigger_after_tokens: usize,
    /// Nhắc mỗi N iterations
    remind_every_n: usize,
    /// Các rules quan trọng nhất cần nhắc lại
    key_rules: Vec<String>,
}

impl Hint for SystemPromptReminder {
    fn name(&self) -> &str { "system_prompt_reminder" }
    
    fn should_trigger(&self, state: &AgentState) -> bool {
        state.total_context_tokens > self.trigger_after_tokens
            && state.iteration_count % self.remind_every_n == 0
    }
    
    fn generate(&self, _state: &AgentState) -> HintMessage {
        HintMessage {
            role: MessageRole::System,
            content: format!(
                "[REMINDER] Please re-read your core instructions:\n{}",
                self.key_rules.iter()
                    .enumerate()
                    .map(|(i, r)| format!("{}. {}", i + 1, r))
                    .collect::<Vec<_>>()
                    .join("\n")
            ),
            priority: HintPriority::Critical,
        }
    }
    
    fn injection_point(&self) -> InjectionPoint {
        InjectionPoint::RecencyZone  // Cuối conversation = model nhớ nhất
    }
}
```

**Dev mở rộng:** Custom `Hint` cho domain-specific guidance (code review hints, writing hints, research hints).

---

## TRACK — Âm thầm giám sát, adaptive scheduling

### Triết lý: **Đi theo model, lệch phát là lôi Guard/Hint ra mà xử.**

### GoClaw Track System — Lane Scheduling:

Dựa trên diagram Track System:

```
Scheduler ──▶ Lane Manager
                   │
    ┌──────────────┼──────────────────────┐
    ▼              ▼              ▼               ▼
Lane 1 (main)  Lane 2 (sub)  Lane 3 (team)  Lane 4 (cron)
 cap: 30        cap: 50        cap: 100       cap: 30
 serial(DM)    parallel(3)   adaptive(*)     scheduled
    │              │              │               │
  Queue          Queue          Queue           Queue
    │              │              │               │
  Agent Loop   Agent Loop    Agent Loop      Agent Loop
```

### Điều quan trọng nhất: Adaptive throttle

> Khi context >60% → **tự giảm concurrency** để model nhỏ không overwhelm.

### Áp dụng cho TraitClaw:

```rust
/// Track — giám sát agent state, trigger Guard/Hint khi cần
pub struct Tracker {
    /// Đếm iterations
    iteration_count: AtomicUsize,
    /// Token usage hiện tại
    token_usage: AtomicUsize,
    /// Token budget
    token_budget: usize,
    /// Context utilization (0.0 - 1.0)
    context_utilization: AtomicF32,
    /// Recent tool calls (cho loop detection)
    recent_tool_calls: Mutex<VecDeque<ToolCallRecord>>,
    /// Danh sách guards
    guards: Vec<Arc<dyn Guard>>,
    /// Danh sách hints 
    hints: Vec<Arc<dyn Hint>>,
}

impl Tracker {
    /// Gọi TRƯỚC mỗi action — Guard check
    pub fn before_action(&self, action: &Action) -> GuardResult {
        self.guard_pipeline.check_all(action)
    }
    
    /// Gọi TRƯỚC mỗi LLM call — collect hints
    pub fn collect_hints(&self, state: &AgentState) -> Vec<HintMessage> {
        self.hints.iter()
            .filter(|h| h.should_trigger(state))
            .map(|h| h.generate(state))
            .collect()
    }
    
    /// Gọi SAU mỗi LLM call — update tracking
    pub fn after_llm_call(&self, response: &CompletionResponse) {
        self.token_usage.fetch_add(response.usage.total_tokens, Ordering::Relaxed);
        self.iteration_count.fetch_add(1, Ordering::Relaxed);
        self.update_context_utilization();
    }
    
    /// Adaptive concurrency — giảm khi context gần đầy
    pub fn recommended_concurrency(&self) -> usize {
        let util = self.context_utilization.load(Ordering::Relaxed);
        match util {
            u if u > 0.8 => 1,   // Context rất đầy → serial only
            u if u > 0.6 => 2,   // Context khá đầy → giảm concurrency
            _ => self.max_concurrency, // Còn thoải mái → full speed
        }
    }

    /// Phát hiện: model đang loop?
    pub fn detect_loop(&self) -> Option<LoopDetection> {
        let calls = self.recent_tool_calls.lock().unwrap();
        // Cùng tool + cùng args gọi >= 3 lần?
        // Tool call không thay đổi state?
        // Artifact tạo rồi lại tạo lại?
        detect_repetitive_pattern(&calls)
    }
}

/// AgentState — snapshot để Guard/Hint/Track phân tích
pub struct AgentState {
    pub iteration_count: usize,
    pub token_usage: usize,
    pub token_budget: usize,
    pub total_context_tokens: usize,
    pub context_window_size: usize,
    pub last_output_truncated: bool,
    pub is_team_task: bool,
    pub recent_tool_calls: Vec<ToolCallRecord>,
    pub model_tier: ModelTier,
}

pub enum ModelTier {
    /// GPT-4o, Claude Opus, Gemini Ultra — ít cần Hint
    Large,
    /// GPT-4o-mini, Claude Sonnet — cần Hint thường xuyên
    Medium,
    /// Haiku, Gemma, Phi — cần tất cả 3 layers critical
    Small,
}
```

---

## Tích hợp vào Agent Runtime

```rust
/// Agent Runtime — nơi Guard/Hint/Track chạy
impl AgentRuntime {
    async fn run_loop(&self, input: &str) -> Result<AgentOutput> {
        let mut state = AgentState::new(input, &self.config);
        
        loop {
            // ══════ TRACK: update state ══════
            self.tracker.update_state(&mut state);
            
            // ══════ TRACK: detect anomalies ══════
            if let Some(loop_detected) = self.tracker.detect_loop() {
                // Inject hint hoặc break
                match loop_detected.severity {
                    Severity::Critical => break Err(AgentError::LoopDetected),
                    _ => state.inject_hint(loop_detected.into_hint()),
                }
            }
            
            // ══════ HINT: collect hints ══════
            let hints = self.tracker.collect_hints(&state);
            for hint in hints {
                state.inject_message(hint);
            }
            
            // ══════ Build request ══════
            let request = self.build_request(&state);
            
            // ══════ LLM call ══════
            let response = self.provider.complete(request).await?;
            
            // ══════ TRACK: update after response ══════
            self.tracker.after_llm_call(&response);
            
            // ══════ Process response ══════
            match response.content {
                Content::Text(text) => {
                    // ══════ GUARD: check output ══════
                    let action = Action::RawOutput { content: text.clone() };
                    if let GuardResult::Deny { reason, .. } = self.tracker.before_action(&action) {
                        state.inject_hint(HintMessage::guard_blocked(&reason));
                        continue; // retry với feedback
                    }
                    return Ok(AgentOutput::text(text));
                }
                Content::ToolCalls(calls) => {
                    for call in calls {
                        // ══════ GUARD: check tool call ══════
                        let action = Action::ToolCall {
                            name: call.name.clone(),
                            arguments: call.arguments.clone(),
                        };
                        match self.tracker.before_action(&action) {
                            GuardResult::Deny { reason, .. } => {
                                // Inject "tool bị chặn" vào conversation
                                state.add_tool_result(&call.name, 
                                    &format!("[BLOCKED] {}", reason));
                                continue;
                            }
                            GuardResult::Sanitize { modified_action, warning } => {
                                // Dùng sanitized version
                                state.inject_hint(HintMessage::warning(&warning));
                                // execute modified action...
                            }
                            GuardResult::Allow => {
                                // Execute tool
                                let result = self.execute_tool(&call).await?;
                                state.add_tool_result(&call.name, &result);
                            }
                        }
                    }
                }
            }
            
            // ══════ TRACK: check budget ══════
            if state.token_usage >= state.token_budget {
                return Err(AgentError::BudgetExceeded);
            }
        }
    }
}
```

---

## Tại sao Guard–Hint–Track khác biệt vs "Guardrails" truyền thống?

| Guardrails (các framework khác) | Guard–Hint–Track (GoClaw/TraitClaw) |
|:---:|:---:|
| Check **SAU** khi model trả lời | Guard chặn **TRƯỚC** khi model hành động |
| Reactive — phản ứng | Proactive — chủ động lái |
| 1 layer duy nhất | 3 layers chạy đồng thời |
| Chỉ chặn hoặc cho qua | Chặn (Guard) + Lái (Hint) + Theo dõi (Track) |
| Tin model, kiểm tra output | **Không tin model**, kiểm soát mọi action |
| Tốn tokens retry khi fail | Tiết kiệm tokens bằng cách nhắc đúng lúc |
| Model lớn OK, model nhỏ vẫn fail | Model nhỏ cũng ổn nhờ infrastructure đỡ |

---

## Model Tier Adaptation

```
┌─────────────────────────────────────────────────────────┐
│ Model nhỏ (Haiku, Gemma, Phi)                           │
│                                                          │
│  Guard:  ALL ON (critical)                               │
│  Hint:   Aggressive (mỗi 3-4 iterations)                │
│  Track:  Tight (concurrency=1, adaptive throttle)        │
│                                                          │
│  → Model nhỏ không cần giỏi hơn, infrastructure đỡ cho  │
├─────────────────────────────────────────────────────────┤
│ Model trung bình (Sonnet, 4o-mini)                       │
│                                                          │
│  Guard:  ALL ON                                          │
│  Hint:   Moderate (mỗi 6 iterations)                    │
│  Track:  Normal (concurrency=3)                          │
├─────────────────────────────────────────────────────────┤
│ Model lớn (Opus, GPT-4o, Gemini Ultra)                   │
│                                                          │
│  Guard:  ALL ON (không bao giờ tắt Guard)                │
│  Hint:   Light (chỉ budget + system remind ở context dài)│
│  Track:  Relaxed (concurrency=full)                      │
│                                                          │
│  → Hơi thừa tokens nhưng vẫn hiệu quả ở context dài     │
└─────────────────────────────────────────────────────────┘
```

### Tự động điều chỉnh theo model:

```rust
impl AgentConfig {
    /// Auto-configure Guard/Hint/Track theo model tier
    pub fn auto_steering(mut self, model: &ModelInfo) -> Self {
        match model.tier() {
            ModelTier::Small => {
                self.hint_frequency = 3;        // mỗi 3 iterations
                self.max_concurrency = 1;       // serial
                self.context_throttle = 0.5;    // throttle ở 50% context
                self.system_remind_every = 4;   // nhắc system prompt mỗi 4 iter
            }
            ModelTier::Medium => {
                self.hint_frequency = 6;
                self.max_concurrency = 3;
                self.context_throttle = 0.6;
                self.system_remind_every = 8;
            }
            ModelTier::Large => {
                self.hint_frequency = 12;
                self.max_concurrency = 10;
                self.context_throttle = 0.8;
                self.system_remind_every = 15;
            }
        }
        self
    }
}
```

---

## Kết luận: Bài học quan trọng nhất

1. **Đừng tin model** — xây infrastructure để chặn/lái/theo dõi
2. **Prompt engineering không đủ** — đặc biệt model nhỏ
3. **Thà tốn 1-2 iterations hint** — còn hơn model đi lạc 20 iterations
4. **Guard luôn bật** — kể cả model lớn, vì security không bao giờ thừa
5. **Nhỏ chạy được thì to chạy mượt** — thiết kế cho worst case
6. **Adaptive** — tự điều chỉnh theo model tier + context utilization

> **Đây phải là first-class citizen trong TraitClaw core, không phải extension crate.**
