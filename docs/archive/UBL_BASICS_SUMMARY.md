# UBL Basics Summary

## Core Concepts

### 1. Container Universal Structure

Each container follows this structure:
```
C.Jobs/
├── boundary/     # Bridge container ↔ kernel (TDLN → LINK → MEMBRANE)
├── inbox/        # Events already in ledger (from SSE tail)
├── local/        # User/UX & light validations
├── outbox/       # Local intents (pre-TDLN), ephemeral
├── projections/  # Derived read-only states
├── pacts/        # Pact definitions (ref.json)
├── policy/       # Policy definitions (ref.json)
└── README.md     # Container documentation
```

### 2. Data Flow Pattern

```
[local] --draft--> [boundary] --signing_bytes--> [ubl-link] --(signature)--> [membrane]
                                                            \--Accept--> [ledger] --tail--> [projections]
```

**Key Points:**
- **Local**: User input, ephemeral drafts (no DB access)
- **Boundary**: Converts drafts to ubl-link, signs, validates, commits
- **Inbox**: Receives SSE events from ledger tail
- **Projections**: Derives read-only state from ledger events

### 3. Event Structure

Events are committed as **ubl-atoms** (canonical JSON) wrapped in **ubl-links**:

```typescript
// Local intent (semantic, local language)
const draft = {
  type: "job.created",
  job_id: "job_123",
  title: "Create report",
  // ... semantic data
};

// TDLN transforms to ubl-atom (canonical JSON)
const atom = canonicalize(draft); // {"job_id":"job_123","title":"Create report","type":"job.created"}

// Hash the atom
const atom_hash = BLAKE3("ubl:atom\n" + atom);

// Build ubl-link (envelope)
const link = {
  version: 1,
  container_id: "C.Jobs",
  expected_sequence: 42,
  previous_hash: "...",
  atom_hash: atom_hash,
  intent_class: "Observation", // or Conservation, Entropy, Evolution
  physics_delta: 0, // for Observation
  author_pubkey: "...",
  signature: "..."
};

// Commit to ledger
POST /link/commit → { status: "ACCEPTED", receipt: {...} }
```

### 4. Intent Classes (Physics)

| Class | Physics Delta | Use Case |
|-------|---------------|----------|
| `Observation` | `Δ = 0` | Record facts, no value change |
| `Conservation` | `ΣΔ = 0` | Transfer value (paired commits) |
| `Entropy` | `Δ ≠ 0` | Create/destroy value (authorized) |
| `Evolution` | N/A | Change rules (authorized) |

**For C.Jobs:**
- Most events are `Observation` (Δ = 0) - recording job lifecycle
- Job creation/completion might be `Entropy` if it involves value creation

### 5. Container Rules

**CRITICAL RULES:**
1. **No direct DB access** - Only via kernel API (`/link/commit`, `/state`, `/ledger/:id/tail`)
2. **No container imports** - Containers communicate only via ubl-links
3. **State is derived** - All state comes from ledger projections
4. **Semantic isolation** - Each container has its own language (L)
5. **Cryptographic proof** - All effects must be provable via ledger

### 6. C.Jobs Container Requirements

Based on the prompt, C.Jobs needs:

**Events:**
- `job.created` - New job created
- `job.started` - Job execution started
- `job.progress` - Job progress update
- `job.completed` - Job finished successfully
- `job.cancelled` - Job cancelled
- `approval.requested` - Approval needed
- `approval.decided` - Approval decision made

**Structure:**
```
C.Jobs/
├── boundary/
│   └── job_boundary.rs      # Convert drafts → ubl-link → commit
├── inbox/
│   └── job_inbox.rs         # Process SSE events from ledger
├── local/
│   └── job_local.rs         # HTTP handlers, validation
├── outbox/
│   └── job_outbox.rs        # Draft creation (ephemeral)
├── projections/
│   └── job_projections.rs   # Derive job state from events
├── pacts/
│   └── ref.json             # Approval pacts
├── policy/
│   └── ref.json             # Container policy
└── README.md
```

**Queries Needed:**
- List jobs (filter by status, assignee, etc.)
- Get job details
- List pending approvals
- Get job events/receipt

### 7. Integration Points

**C.Jobs ↔ Office:**
- Office calls `POST /jobs/execute` (via HTTP)
- Office publishes approval requests
- Office receives approval decisions

**C.Jobs ↔ UBL Messenger:**
- Messenger creates jobs via `POST /api/jobs`
- Messenger receives job cards via WebSocket
- Messenger sends approval decisions

**C.Jobs ↔ UBL Ledger:**
- All events committed via `/link/commit`
- State queries via `/state/:container_id`
- Real-time updates via `/ledger/:container_id/tail` (SSE)

### 8. Implementation Checklist

- [ ] Create C.Jobs container structure
- [ ] Define job event types (ubl-atoms)
- [ ] Implement boundary layer (draft → link → commit)
- [ ] Implement inbox layer (SSE → projections)
- [ ] Implement local layer (HTTP handlers)
- [ ] Implement projections (state derivation)
- [ ] Define pacts for approvals
- [ ] Add queries for jobs/approvals
- [ ] Integration tests with Office
- [ ] Integration tests with Messenger

---

**Key Insight:** UBL separates **meaning** (local semantics) from **proof** (cryptographic verification). The container's local language can be anything, but all effects must be provable via the ledger.

