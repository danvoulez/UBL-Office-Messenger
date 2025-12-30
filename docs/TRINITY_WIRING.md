# 🔺 The Flagship Trinity — Wiring Complete

**Date:** 2025-12-27

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           MESSENGER                                      │
│  Frontend (React) ←──WebSocket──→ Backend (Rust) ←──HTTP──→ OFFICE      │
│                                       │                                  │
│                                       │ Jobs, Messages                   │
│                                       ▼                                  │
└───────────────────────────────────────┼─────────────────────────────────┘
                                        │
                                        ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                              UBL LEDGER                                  │
│                                                                          │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐     │
│  │  C.Jobs     │  │ C.Messenger │  │ C.Entities  │  │  C.Pacts    │     │
│  │             │  │             │  │             │  │             │     │
│  │ job.created │  │ msg.sent    │  │ entity.     │  │ pact.       │     │
│  │ job.approved│  │ msg.read    │  │   created   │  │   created   │     │
│  │ job.started │  │ msg.deleted │  │ session.    │  │ pact.       │     │
│  │ job.progress│  │             │  │   completed │  │   signed    │     │
│  │ job.completed│ │             │  │             │  │             │     │
│  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘     │
│                                                                          │
│                    Policy VM (hardened) validates all commits            │
└─────────────────────────────────────────────────────────────────────────┘
                                        ▲
                                        │
┌───────────────────────────────────────┼─────────────────────────────────┐
│                              OFFICE                                      │
│                                       │                                  │
│  ┌─────────────────┐  ┌─────────────┐│  ┌─────────────────────────┐    │
│  │  SmartRouter    │  │  Entity     ││  │    Job Executor          │    │
│  │  (LLM routing)  │  │  Repository ││  │    (Chair + Instance)    │    │
│  └─────────────────┘  └─────────────┘│  └─────────────────────────┘    │
│                                       │                                  │
│  The Chair (Entity) is permanent. Instances (LLM sessions) are ephemeral.│
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Wiring Completed

### 1. Messenger → UBL ✅

**File:** `apps/messenger/backend/src/ubl_client/mod.rs`

| Method | Event Type | Container |
|--------|------------|-----------|
| `store_message()` | `message.created` | C.Messenger |
| `store_read_receipt()` | `message.read` | C.Messenger |
| `publish_job_event()` | `job.*` | C.Jobs |

**File:** `apps/messenger/backend/src/job/repository.rs`

| Method | Event Type | Container |
|--------|------------|-----------|
| `create()` | `job.created` | C.Jobs |
| `start()` | `job.started` | C.Jobs |
| `approve()` | `job.approved` | C.Jobs |
| `reject()` | `job.rejected` | C.Jobs |
| `update_progress()` | `job.progress` | C.Jobs |
| `complete()` | `job.completed` | C.Jobs |
| `cancel()` | `job.cancelled` | C.Jobs |

All events are:
- Canonicalized (JSON✯Atomic v1.0)
- Hashed (BLAKE3, no domain tag for atoms)
- Signed (Ed25519)
- Committed to UBL

### 2. OFFICE → UBL ✅

**File:** `apps/office/src/entity/repository.rs`

| Method | Event Type | Container |
|--------|------------|-----------|
| `create_entity()` | `entity.created` | C.Entities |
| `update_constitution()` | `constitution.updated` | C.Entities |
| `update_baseline()` | `baseline.updated` | C.Entities |
| `record_session()` | `session.completed` | C.Entities |

**File:** `apps/office/src/job_executor/executor.rs`

| Method | Event Type | Container |
|--------|------------|-----------|
| `execute_job()` | `job.completed` | C.Jobs |

The Chair pattern:
- Entity (permanent identity) stored in UBL
- Instance (ephemeral LLM session) sits in Chair
- Handovers written to entity for next instance

### 3. Messenger ↔ OFFICE ✅

**File:** `apps/messenger/backend/src/office_client/mod.rs`

```rust
// Entity management
create_entity() → POST /entities
create_session() → POST /entities/{id}/sessions
send_message() → POST /entities/{id}/sessions/{sid}/message
end_session() → DELETE /entities/{id}/sessions/{sid}

// Job execution
execute_job() → POST /jobs/execute
execute_job_with_progress() → POST /jobs/execute/stream (SSE)
```

**File:** `apps/messenger/backend/src/job/routes.rs`

```rust
POST /api/jobs/:id/approve → approve_job()
  └── repository.approve() → publishes job.approved to UBL
  └── office_client.execute_job() → OFFICE executes
  └── repository.complete() → publishes job.completed to UBL
  └── ws_broadcaster.broadcast() → real-time update to frontend
```

---

## Event Flow

### User Sends Message → Job Proposed → Job Approved → Job Completed

```
1. User types message in Messenger frontend
   └── WebSocket → backend → store_message() → C.Messenger

2. AI agent proposes job (card displayed)
   └── create() → job.created → C.Jobs

3. User clicks "Approve" button
   └── POST /api/jobs/:id/approve
   └── repository.approve() → job.approved → C.Jobs
   └── OFFICE.execute_job() → Entity loaded → LLM instance created
   └── LLM completes task → handover written → session.completed → C.Entities
   └── repository.complete() → job.completed → C.Jobs
   └── WebSocket broadcast → frontend updates card

4. User sees Finished card with summary
```

---

## Job Event Types (C.Jobs Container)

```typescript
// Event envelope
{
  "type": "job.created" | "job.approved" | "job.rejected" | 
          "job.started" | "job.progress" | "job.completed" | 
          "job.failed" | "job.cancelled",
  "id": "job_uuid",
  "timestamp": "ISO8601",
  // ... event-specific fields
}
```

---

## Card Contracts (Frontend)

### FormalizeCard (job.proposed state)
- Shows goal, description, priority
- Buttons: Approve, Request Changes, Reject

### TrackingCard (job.running state)
- Shows progress %, current step
- Buttons: Cancel (optional)

### FinishedCard (job.completed/failed state)
- Shows summary, artifacts
- Buttons: Acknowledge, Dispute

---

## What's Left

1. **Signing in OFFICE** - Currently using mock signatures in `entity/repository.rs` and `job_executor/executor.rs`. Need to:
   - Pass actual `SigningKey` to repositories
   - Properly sign all UBL commits

2. **Projection Rebuild** - Both repos use in-memory cache. Production should:
   - Subscribe to SSE tail on startup
   - Rebuild projections from ledger events

3. **Frontend Card Rendering** - `JobCardRenderer.tsx` exists but needs:
   - Connect to WebSocket for real-time updates
   - Wire button clicks to `/api/jobs/:id/approve|reject`

4. **End-to-End Tests** - Need integration tests:
   - Message → Job creation
   - Job approval → OFFICE execution
   - UBL event verification

---

## Summary

| Component | Status |
|-----------|--------|
| Messenger → UBL (Messages) | ✅ Complete |
| Messenger → UBL (Jobs) | ✅ Complete |
| OFFICE → UBL (Entities) | ✅ Structure, mock signing |
| OFFICE → UBL (Job completion) | ✅ Complete |
| Messenger → OFFICE (Job execution) | ✅ Complete |
| Real-time WebSocket | ✅ Complete |
| Frontend Cards | ✅ Structure, needs wiring |

**The Trinity is wired.** 🔺



