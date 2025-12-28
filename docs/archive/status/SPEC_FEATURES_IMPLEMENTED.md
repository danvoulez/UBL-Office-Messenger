# 📜 Spec Features Implemented

Features from the Flagship Spec made ours.

## What We Added

### 1. Tool Audit Trail 🔧

**Location:** `office/office/src/audit/`

Every tool call is now auditable:

```rust
// Record a tool call (before execution)
let call = ToolCallBuilder::new("calendar.create_invite", "job_123")
    .purpose("Create a meeting invite")
    .inputs(sanitized_inputs)
    .build();

audit.record_call(job_id, conv_id, tenant_id, actor_id, call).await?;

// Record the result (after execution)
audit.record_result(job_id, conv_id, tenant_id, actor_id, result).await?;
```

**Events:**
- `tool.called` - Intent to execute, with sanitized inputs
- `tool.result` - What happened (success/error), with artifacts

**Error Types:**
- `PROVIDER_TIMEOUT` (retryable)
- `PROVIDER_RATE_LIMIT` (retryable)
- `PROVIDER_AUTH_REQUIRED` (not retryable)
- `INVALID_INPUT` (not retryable)
- `PROVIDER_UNAVAILABLE` (retryable)

---

### 2. PII Redaction 🔒

**Location:** `office/office/src/audit/pii.rs`

Never store raw PII in the ledger:

```rust
// Redact email: "john@example.com" → "j***@example.com"
let redacted = redact_email("john@example.com");

// Hash for correlation: "blake3:8b6c...e19a"
let hash = hash_pii("john@example.com", "tenant_123");

// Sanitize entire JSON payloads
let (sanitized, policy) = sanitize_json(&input_data, tenant_id);
```

**SanitizedAttendee:**
```rust
let attendee = SanitizedAttendee::from_email("Maria", "maria@acme.com", "tenant_123");
// attendee.email_redacted = "m***@acme.com"
// attendee.email_hash = "blake3:..."
```

---

### 3. Job FSM (Finite State Machine) ⚙️

**Location:** `office/office/src/job_executor/fsm.rs`

Strict state transitions - no job can jump states:

```
draft → proposed → approved → in_progress ↔ waiting_input → completed
                ↘ rejected        ↘ failed/cancelled
```

```rust
let mut tracker = JobStateTracker::new();

tracker.propose()?;   // draft → proposed
tracker.approve()?;   // proposed → approved
tracker.start()?;     // approved → in_progress
tracker.wait_for_input()?;  // in_progress → waiting_input
tracker.resume()?;    // waiting_input → in_progress
tracker.complete()?;  // in_progress → completed ✅

// Illegal transitions are blocked:
tracker.start()?;  // ERROR: Cannot transition from draft to in_progress
```

---

### 4. Button Provenance 🛡️

**Location:** `office/office/src/governance/provenance.rs`

No fake buttons. Button clicks are validated against real cards:

```rust
let validator = ProvenanceValidator::new();

// When Office emits a card:
validator.register_card("card_abc", "job_123", buttons).await;

// When user clicks a button:
let validated = validator.validate_action(
    "card_abc",
    "btn_approve_123",
    &claimed_action,
    user_input,
).await?;

// Validation ensures:
// 1. Card exists in ledger
// 2. Button exists in card
// 3. Action type matches button's declared action
// 4. Input provided if required
```

**Prevents:**
- Forging approvals
- Replaying buttons from different jobs
- Invented actions not offered by UI

---

### 5. Card Contracts 🎴

**Location:** `office/office/src/job_executor/cards.rs`

Full spec-compliant card types:

#### FormalizeCard (Job Proposal)
```rust
FormalizeCard {
    base: CardBase { ... },
    job: JobDefinition {
        goal: "Schedule a 30-minute call with Maria",
        inputs_needed: vec![
            InputNeeded { key: "maria_email", label: "Maria's email", status: Missing },
        ],
        expected_outputs: vec![
            ExpectedOutput { kind: Link, description: "Calendar invite" },
        ],
        constraints: vec!["Don't email until approved"],
        sla_hint: "ETA ~5 minutes",
    },
    plan_hint: vec!["Collect details", "Create invite", "Send to Maria"],
}
```

**Default Buttons:** Approve, Reject, Request changes, Ask in chat

#### TrackingCard (In Progress)
```rust
TrackingCard {
    base: CardBase { ... },
    progress: ProgressInfo {
        percent: Some(50),
        status_line: "Creating calendar invite...",
        steps: vec![
            Step { key: "collect", label: "Collect details", state: Done },
            Step { key: "create", label: "Create invite", state: Doing },
            Step { key: "send", label: "Send to Maria", state: Todo },
        ],
        waiting_on: vec![WaitingOn { entity_id: "dan", display_name: "Dan" }],
    },
}
```

**Default Buttons:** Got it, Provide info, Dispute, Cancel, Ask in chat

#### FinishedCard (Completed)
```rust
FinishedCard {
    base: CardBase { ... },
    outcome: Outcome {
        result: Completed,
        summary: "Created invite and sent to maria@acme.com",
    },
    artifacts: vec![
        ArtifactRef {
            kind: Link,
            title: "Calendar invite (Google Meet)",
            url: Some("https://calendar.example/invite/abc123"),
        },
    ],
    next_actions: vec![NextAction { label: "Follow-up", ... }],
}
```

**Default Buttons:** Accept, Dispute, Follow-up, Ask in chat

---

## Architecture Summary

```
┌─────────────────────────────────────────────────────────────┐
│                     NEW COMPONENTS                          │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  audit/                                                     │
│  ├── tool_audit.rs   → tool.called/tool.result events       │
│  ├── pii.rs          → redact_email, hash_pii, sanitize     │
│  └── events.rs       → AuditEvent types                     │
│                                                             │
│  job_executor/                                              │
│  ├── fsm.rs          → JobState, JobFsm, transitions        │
│  └── cards.rs        → Formalize/Tracking/Finished cards    │
│                                                             │
│  governance/                                                │
│  └── provenance.rs   → Button provenance validation         │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│                     FLOW                                    │
│                                                             │
│  1. User requests work                                      │
│  2. Office proposes job (FSM: draft → proposed)             │
│  3. FormalizeCard sent with buttons                         │
│  4. User clicks Approve                                     │
│  5. Provenance validates button click                       │
│  6. FSM: proposed → approved → in_progress                  │
│  7. Tool calls recorded (tool.called)                       │
│  8. PII sanitized before storage                            │
│  9. Tool results recorded (tool.result)                     │
│  10. TrackingCard updates sent                              │
│  11. FSM: in_progress → completed                           │
│  12. FinishedCard with artifacts                            │
│  13. Everything reconstructible from ledger                 │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Key Principles Applied

1. **Auditability**: Every tool call recorded with sanitized inputs/outputs
2. **No Raw PII**: Emails/phones redacted, hashed for correlation
3. **Strict FSM**: No illegal job state transitions
4. **Button Provenance**: No forged approvals
5. **Spec-Compliant Cards**: Exact structure from the flagship spec

---

## Files Added

### Backend (Office)
| File | Purpose |
|------|---------|
| `office/src/audit/mod.rs` | Audit module exports |
| `office/src/audit/tool_audit.rs` | Tool call/result recording |
| `office/src/audit/pii.rs` | PII redaction utilities |
| `office/src/audit/events.rs` | Audit event types |
| `office/src/job_executor/fsm.rs` | Job state machine |
| `office/src/job_executor/cards.rs` | Card contracts |
| `office/src/governance/provenance.rs` | Button validation |

### Frontend (Messenger)
| File | Purpose |
|------|---------|
| `frontend/types/cards.ts` | TypeScript card types (matches Rust) |
| `frontend/components/cards/JobCardRenderer.tsx` | Beautiful card rendering |
| `frontend/styles/job-cards.css` | Card styling with animations |

## Files Modified

| File | Change |
|------|--------|
| `office/src/lib.rs` | Added `audit` module, new error types |
| `office/src/job_executor/mod.rs` | Export FSM and Cards |
| `office/src/governance/mod.rs` | Export Provenance |
| `office/src/ubl_client/mod.rs` | Added `commit_atom` method |
| `office/Cargo.toml` | Added `regex` dependency |
| `frontend/styles/design-tokens.css` | Extended accent colors, surface aliases |
| `frontend/styles/index.css` | Import job-cards.css |

---

## What's Next?

With these foundations in place, the system now has:

1. **Full Auditability** - Every tool call is recorded
2. **PII Safety** - No raw emails in the ledger
3. **Strict State Machine** - Jobs can't skip steps
4. **Button Security** - No forged approvals
5. **Beautiful Cards** - Spec-compliant, animated UI

The next logical steps would be:
- Wire cards to WebSocket for real-time updates
- Integrate audit trail with LLM execution
- Add Dreaming Cycle integration
- Build the "Schedule a meeting" golden demo

---

*"If it's not in the ledger, it didn't happen."* 📜

