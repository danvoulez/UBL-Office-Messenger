# UBL Compliance Analysis: Prompt 3

## Executive Summary

The prompt is **mostly aligned** with UBL principles but **lacks critical implementation details** about the UBL event commitment flow. It correctly identifies containers, events, and integration patterns, but doesn't specify the **boundary → inbox → projections** architecture or the **TDLN → ubl-link → commit** flow.

**Score: 7/10** ✅ Good vision, needs UBL-specific implementation details

---

## ✅ What's Good

### 1. Container Architecture
- ✅ Correctly identifies C.Jobs, C.Messenger, C.Office containers
- ✅ Mentions container-specific events
- ✅ Understands container isolation

### 2. Event-Driven Design
- ✅ Events are the primary communication mechanism
- ✅ Event types are well-defined (`job.created`, `job.started`, etc.)
- ✅ Events flow through the ledger

### 3. Integration Patterns
- ✅ HTTP/WebSocket/SSE for inter-system communication
- ✅ No direct container imports (correct!)
- ✅ Systems communicate via APIs

### 4. Trust Architecture
- ✅ Mentions L0-L5 trust levels
- ✅ Approval workflows
- ✅ Pacts for authorization

---

## ❌ What's Missing (Critical)

### 1. Container Structure Not Specified

**Missing:**
- No mention of `boundary/`, `inbox/`, `local/`, `outbox/`, `projections/` directories
- No explanation of the data flow: `local → boundary → ubl-link → ledger → inbox → projections`

**Should be:**
```rust
C.Jobs/
├── boundary/     # TDLN: draft → ubl-link → commit
├── inbox/        # SSE tail → process events
├── local/        # HTTP handlers, validation
├── outbox/       # Draft creation (ephemeral)
├── projections/  # Derive state from ledger
├── pacts/        # Approval pacts
└── policy/       # Container policy
```

### 2. Event Commitment Flow Not Detailed

**Current (in prompt):**
```rust
// Just shows event types, not HOW they're committed
Event {
    type: "job.created",
    payload: { ... }
}
```

**Should be:**
```rust
// 1. Create draft (local/outbox)
let draft = JobCreatedDraft {
    job_id: "job_123",
    title: "Create report",
    // ... semantic data
};

// 2. TDLN: Convert to ubl-atom (boundary)
let atom = canonicalize(&draft)?;
let atom_hash = hash_atom(&atom)?;

// 3. Get current state
let state = ubl_client.get_state("C.Jobs").await?;

// 4. Build ubl-link (boundary)
let link = LinkCommit {
    version: 1,
    container_id: "C.Jobs",
    expected_sequence: state.sequence + 1,
    previous_hash: state.last_hash,
    atom_hash,
    intent_class: IntentClass::Observation,
    physics_delta: 0,
    author_pubkey: author_pubkey,
    signature: sign(&link_signing_bytes, &signing_key)?,
};

// 5. Commit to ledger (boundary → kernel)
let receipt = ubl_client.commit(&link).await?;

// 6. Ledger emits SSE event → inbox processes → projections update
```

### 3. State Derivation Not Mentioned

**Missing:**
- No mention that state MUST be derived from ledger projections
- No explanation of how queries work (via projections, not direct DB)

**Should be:**
```rust
// ❌ WRONG (direct DB access)
let jobs = db.query("SELECT * FROM jobs WHERE status = 'active'").await?;

// ✅ CORRECT (projections from ledger)
let events = ledger.tail("C.Jobs").await?;
let jobs = projections::derive_jobs_from_events(events).await?;
```

### 4. Intent Classes Not Specified

**Missing:**
- Events don't specify `intent_class` (Observation, Conservation, Entropy, Evolution)
- No mention of `physics_delta` requirements

**Should be:**
```rust
// Job lifecycle events are mostly Observation (Δ = 0)
JobCreated { ... } → IntentClass::Observation, physics_delta: 0
JobStarted { ... } → IntentClass::Observation, physics_delta: 0
JobProgress { ... } → IntentClass::Observation, physics_delta: 0

// Job completion might be Entropy if value is created
JobCompleted { value_created: 1000 } → IntentClass::Entropy, physics_delta: 1000
```

### 5. SSE Tail Pattern Not Explained

**Missing:**
- No mention of `/ledger/:container_id/tail` SSE endpoint
- No explanation of how containers receive real-time updates

**Should be:**
```rust
// Container inbox subscribes to ledger tail
let mut stream = ubl_client.tail("C.Jobs").await?;
while let Some(event) = stream.next().await {
    inbox::process_event(event).await?;
    projections::update_state(event).await?;
}
```

### 6. No Direct DB Access Rule Not Enforced

**Missing:**
- Prompt shows HTTP endpoints that might imply direct DB access
- No explicit rule: "Containers MUST NOT access database directly"

**Should be:**
```rust
// ❌ WRONG: Direct DB access
impl JobRepository {
    async fn create_job(&self, job: Job) -> Result<()> {
        sqlx::query("INSERT INTO jobs ...").execute(&self.db).await?;
    }
}

// ✅ CORRECT: Via UBL ledger
impl JobRepository {
    async fn create_job(&self, job: Job) -> Result<Receipt> {
        let draft = JobCreatedDraft::from(job);
        boundary::commit_job_created(draft).await
    }
}
```

---

## 🔧 Required Fixes

### Fix 1: Add Container Structure Section

Add to prompt:
```markdown
### C.Jobs Container Structure

```
C.Jobs/
├── boundary/
│   └── job_boundary.rs      # TDLN: draft → ubl-link → commit
├── inbox/
│   └── job_inbox.rs         # SSE tail → process events → update projections
├── local/
│   └── job_local.rs         # HTTP handlers, validation (no DB access)
├── outbox/
│   └── job_outbox.rs        # Draft creation (ephemeral)
├── projections/
│   └── job_projections.rs   # Derive job state from ledger events
├── pacts/
│   └── ref.json             # Approval pacts
└── policy/
    └── ref.json             # Container policy
```
```

### Fix 2: Add Event Commitment Flow

Add to prompt:
```markdown
### Event Commitment Flow

1. **Draft Creation** (outbox/local)
   - User creates job → draft created
   - Draft is ephemeral, not committed yet

2. **TDLN Translation** (boundary)
   - Draft → canonicalize → ubl-atom
   - Hash atom → atom_hash

3. **Build ubl-link** (boundary)
   - Get current state from ledger
   - Build LinkCommit with sequence, previous_hash
   - Sign with author key

4. **Commit** (boundary → kernel)
   - POST /link/commit
   - Membrane validates
   - Ledger appends atomically

5. **Projection Update** (inbox → projections)
   - SSE event from ledger tail
   - Inbox processes event
   - Projections derive new state
```

### Fix 3: Add State Derivation Rules

Add to prompt:
```markdown
### State Derivation Rules

**CRITICAL:** All state MUST be derived from ledger projections.

- ❌ NO direct database queries
- ✅ Query via projections (derived from ledger events)
- ✅ Real-time updates via SSE tail
- ✅ State is always reconstructible from ledger
```

### Fix 4: Specify Intent Classes

Add to prompt:
```markdown
### Job Event Intent Classes

| Event | Intent Class | Physics Delta | Reason |
|-------|-------------|---------------|--------|
| `job.created` | Observation | 0 | Record fact |
| `job.started` | Observation | 0 | Record fact |
| `job.progress` | Observation | 0 | Record fact |
| `job.completed` | Observation or Entropy | 0 or +value | Depends on value creation |
| `approval.requested` | Observation | 0 | Record fact |
| `approval.decided` | Observation | 0 | Record fact |
```

### Fix 5: Add SSE Tail Pattern

Add to prompt:
```markdown
### Real-Time Updates

Containers receive updates via SSE tail:

```rust
// Subscribe to ledger tail
let mut stream = ubl_client.tail("C.Jobs").await?;

while let Some(entry) = stream.next().await {
    // Process event
    inbox::process_entry(entry).await?;
    
    // Update projections
    projections::update(entry).await?;
    
    // Broadcast to WebSocket clients
    websocket::broadcast(entry).await?;
}
```
```

---

## 📊 Compliance Checklist

| Requirement | Status | Notes |
|-------------|--------|-------|
| Container structure (boundary/inbox/etc) | ❌ | Not specified |
| TDLN flow documented | ❌ | Missing |
| Event commitment flow | ❌ | Shows events, not flow |
| State derivation from projections | ❌ | Not mentioned |
| Intent classes specified | ❌ | Missing |
| Physics delta rules | ❌ | Missing |
| SSE tail pattern | ❌ | Not explained |
| No direct DB access | ⚠️ | Implied but not enforced |
| Container isolation | ✅ | Correct |
| Event-driven design | ✅ | Correct |
| Trust architecture | ✅ | Correct |
| Integration patterns | ✅ | Correct |

---

## 🎯 Recommendations

### High Priority
1. **Add container structure section** with boundary/inbox/projections
2. **Document TDLN flow** from draft to ledger commit
3. **Specify state derivation** rules (projections only)
4. **Add intent classes** for each event type

### Medium Priority
5. **Document SSE tail pattern** for real-time updates
6. **Add physics delta** requirements
7. **Clarify no DB access** rule explicitly

### Low Priority
8. Add examples of projection queries
9. Add error handling for commitment failures
10. Add retry logic for failed commits

---

## ✅ Conclusion

The prompt has **excellent vision** and **correct high-level architecture**, but needs **UBL-specific implementation details** to be fully compliant. The missing pieces are:

1. Container internal structure (boundary/inbox/projections)
2. TDLN → ubl-link → commit flow
3. State derivation from projections
4. Intent classes and physics rules

**Recommendation:** Add a new section "UBL Implementation Details" covering these points before implementation begins.

