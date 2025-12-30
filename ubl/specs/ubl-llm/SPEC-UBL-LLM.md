# SPEC-UBL-LLM v1.0

## UBL Large Language Model Gateway Specification

**Status:** Draft – Ready for Implementation  
**Governed by:** SPEC-UBL-CORE v1.0  
**Consumed by:** OFFICE, Messenger, any UBL-native application  
**Independent of:** specific LLM providers, model versions

---

## 1. Definition

The `ubl-llm` gateway is the **unified LLM access layer** for all UBL-native applications.

It provides:
- **Single API** for all LLM interactions
- **Smart routing** to appropriate providers based on task
- **Auditable logging** of all LLM calls to the ledger
- **Provider abstraction** with automatic fallback
- **Task-tailored prompts** and response formatting

Formally:
```
LlmGateway : (Request, TaskType, Context) → (Response, Receipt)
```

The gateway **does not interpret** the semantic content of prompts. It routes, executes, and records.

---

## 2. Principle: LLM Calls as Ledger Events

Every LLM interaction becomes a **verifiable fact** in the UBL ledger:

```
Request → LlmLink → Membrane → Ledger → Execution → Receipt → Ledger
```

This ensures:
- Complete audit trail of all AI interactions
- Reproducibility (same request → same hash)
- Cost tracking and optimization
- Governance and policy enforcement

---

## 3. Container: C.LLM

The LLM gateway operates as container `C.LLM`:

```
C.LLM/
├── boundary/      # Commit LLM requests/responses to ledger
├── inbox/         # Receive events (usage tracking, rate limits)
├── local/         # Request validation, caching
├── outbox/        # Pending requests
├── projections/   # Usage stats, cost tracking
├── pacts/         # Provider authorization
└── policy/        # Rate limits, model access rules
```

---

## 4. Request Structure

### 4.1 LlmRequest

```rust
LlmRequest := ⟨
  request_id,       // Unique identifier
  task_type,        // TaskType enum
  messages,         // Vec<Message>
  preferences,      // RoutingPreferences
  context,          // RequestContext
⟩
```

### 4.2 Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `request_id` | Hash₃₂ | yes | Unique request identity |
| `task_type` | TaskType | yes | Semantic task classification |
| `messages` | Vec<Message> | yes | Conversation messages |
| `preferences` | RoutingPreferences | no | Routing hints |
| `context` | RequestContext | yes | Origin context |

---

## 5. TaskType — Semantic Classification

The gateway uses task types to inform routing, **not** to interpret meaning:

```rust
pub enum TaskType {
    // Code-focused tasks → Claude preferred
    CodeReview,
    CodeGeneration,
    CodeDebug,
    CodeExplain,
    
    // Analysis tasks → Claude preferred (long context)
    DocumentAnalysis,
    DataExtraction,
    Summarization,
    
    // Creative tasks → GPT-4 preferred
    CreativeWriting,
    Brainstorming,
    ContentGeneration,
    
    // Fast/simple tasks → Flash models
    QuickAnswer,
    Classification,
    Translation,
    
    // Conversation → Balanced
    Conversation,
    
    // Custom (with provider hint)
    Custom(String),
}
```

---

## 6. Smart Router

### 6.1 Routing Matrix (Default)

| TaskType | Primary | Secondary | Tertiary |
|----------|---------|-----------|----------|
| CodeReview | Claude Sonnet | GPT-4o | Gemini Pro |
| CodeGeneration | Claude Sonnet | GPT-4o | — |
| DocumentAnalysis | Claude Sonnet | Gemini Pro | GPT-4o |
| CreativeWriting | GPT-4o | Claude Sonnet | — |
| QuickAnswer | Gemini Flash | GPT-4o-mini | Claude Haiku |
| Conversation | Claude Sonnet | GPT-4o | Gemini Pro |

### 6.2 Routing Preferences

```rust
pub struct RoutingPreferences {
    pub speed: SpeedPreference,      // Fast | Normal | Thorough
    pub cost: CostPreference,        // Optimize | Balanced | Ignore
    pub provider_hint: Option<Provider>,  // Force specific provider
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
}
```

### 6.3 Routing Algorithm

```rust
fn route(request: &LlmRequest) -> Provider {
    // 1. Check provider hint (explicit override)
    if let Some(provider) = request.preferences.provider_hint {
        return provider;
    }
    
    // 2. Get routing matrix for task type
    let matrix = get_routing_matrix(&request.task_type);
    
    // 3. Filter by availability
    let available = matrix.filter(|p| p.is_available());
    
    // 4. Apply preferences
    let scored = available.map(|p| score(p, &request.preferences));
    
    // 5. Return highest scoring provider
    scored.max_by_key(|s| s.score).provider
}
```

---

## 7. Provider Abstraction

### 7.1 Provider Trait

```rust
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Provider identifier
    fn id(&self) -> ProviderId;
    
    /// Check availability
    async fn is_available(&self) -> bool;
    
    /// Execute completion request
    async fn complete(&self, request: &ProviderRequest) -> Result<ProviderResponse>;
    
    /// Stream completion (optional)
    async fn stream(&self, request: &ProviderRequest) -> Result<impl Stream<Item = StreamChunk>>;
    
    /// Estimate cost for request
    fn estimate_cost(&self, request: &ProviderRequest) -> Cost;
    
    /// Models supported
    fn models(&self) -> Vec<ModelInfo>;
}
```

### 7.2 Built-in Providers

```rust
pub enum ProviderId {
    Anthropic,      // Claude models
    OpenAI,         // GPT models
    Google,         // Gemini models
    Local,          // Ollama/local models
}
```

---

## 8. Ledger Integration

### 8.1 LLM Request Event

When a request is made:

```rust
Event::LlmRequestReceived {
    request_id: Hash32,
    task_type: TaskType,
    message_hash: Hash32,  // Hash of messages (not content!)
    container_id: String,  // Origin container
    entity_id: String,     // Requesting entity
    timestamp: u128,
}
```

### 8.2 LLM Response Event

When response is received:

```rust
Event::LlmResponseCompleted {
    request_id: Hash32,
    provider: ProviderId,
    model: String,
    response_hash: Hash32,  // Hash of response (not content!)
    usage: TokenUsage,
    latency_ms: u64,
    cost: Cost,
    timestamp: u128,
}
```

### 8.3 Privacy-Preserving Audit

The ledger stores **hashes** of content, not content itself.
This enables:
- Proof that a specific request was made
- Verification of response authenticity
- Cost tracking
- No sensitive data in ledger

---

## 9. API Endpoints

### 9.1 Core Endpoints

```
POST /llm/complete              # Synchronous completion
POST /llm/stream                # Streaming completion
GET  /llm/providers             # List available providers
GET  /llm/usage                 # Usage statistics
GET  /llm/usage/:entity_id      # Usage by entity
```

### 9.2 Request Format

```json
{
  "task_type": "code_review",
  "messages": [
    {"role": "system", "content": "You are a code reviewer..."},
    {"role": "user", "content": "Review this code: ..."}
  ],
  "preferences": {
    "speed": "normal",
    "cost": "optimize"
  },
  "context": {
    "container_id": "C.Messenger",
    "entity_id": "agent_robofab",
    "conversation_id": "conv_123"
  }
}
```

### 9.3 Response Format

```json
{
  "content": "...",
  "provider": "anthropic",
  "model": "claude-3-5-sonnet-20241022",
  "usage": {
    "input_tokens": 150,
    "output_tokens": 500,
    "total_tokens": 650
  },
  "latency_ms": 1250,
  "cost": {
    "amount": 0.0045,
    "currency": "USD"
  },
  "receipt": {
    "entry_hash": "abc123...",
    "sequence": 42
  }
}
```

---

## 10. Caching Strategy

### 10.1 Request Deduplication

```rust
cache_key := BLAKE3(task_type || messages_hash || model)
```

If identical request within TTL, return cached response.

### 10.2 Semantic Caching (Optional)

For deterministic tasks (classification, extraction), cache based on input hash.

---

## 11. Error Handling

### 11.1 Error Types

```rust
pub enum LlmError {
    ProviderUnavailable(ProviderId),
    RateLimited { retry_after: Duration },
    ContextTooLong { max: u32, actual: u32 },
    InvalidRequest(String),
    AuthenticationFailed(ProviderId),
    CostLimitExceeded { limit: Cost, required: Cost },
}
```

### 11.2 Fallback Strategy

```
Primary fails → Try secondary
Secondary fails → Try tertiary
All fail → Return error with details
```

---

## 12. Governance Integration

### 12.1 Policy Enforcement

Before execution, check:
- Entity has LLM access
- Container is authorized for task type
- Rate limits not exceeded
- Cost budget available

### 12.2 Pact Requirements

High-cost operations may require pact approval:

```rust
if estimated_cost > threshold {
    require_pact!(L3, "cost-approval");
}
```

---

## 13. Invariants

1. **All LLM calls are ledger events** — No exceptions
2. **Content never stored in ledger** — Only hashes
3. **Router is deterministic** — Same inputs → same routing
4. **Fallback is automatic** — Apps don't handle provider failures
5. **Costs are tracked** — Every token is accounted

---

## 14. Definition

`ubl-llm` is the bridge between UBL-native applications and the world of large language models.

It ensures that **every AI interaction is auditable, governed, and optimized** without applications needing to understand providers.

---

## 15. Implementation Order

1. `ubl-llm` crate with provider trait
2. Claude provider implementation
3. Router with basic task→provider mapping
4. HTTP endpoints in ubl-server
5. Ledger integration (events)
6. OpenAI + Gemini providers
7. Caching layer
8. Governance integration





