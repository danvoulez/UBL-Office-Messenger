# 🔌 WIRING GUIDE: UBL 3.0

## Overview

UBL 3.0 consists of three interconnected systems:

```
┌──────────────────────────────────────────────────────────────────────────────┐
│                              🔥 UBL 3.0 🔥                                   │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   ┌─────────────────┐    REST/WS     ┌─────────────────┐                    │
│   │                 │◄──────────────►│                 │                    │
│   │   📱 MESSENGER  │                │   🧠 UBL KERNEL │                    │
│   │   (React/TS)    │                │   (Rust Axum)   │                    │
│   │                 │                │                 │                    │
│   └─────────────────┘                └────────┬────────┘                    │
│           │                                   │                              │
│           │ Events                            │ Events                       │
│           │ (SSE/WS)                          │ (SSE/WS)                     │
│           ▼                                   ▼                              │
│   ┌─────────────────┐    Console API  ┌─────────────────┐                   │
│   │                 │◄───────────────►│                 │                   │
│   │   📋 JOB CARDS  │                 │   💼 OFFICE     │                   │
│   │   (UI Layer)    │                 │   (LLM Runtime) │                   │
│   │                 │                 │                 │                   │
│   └─────────────────┘                 └─────────────────┘                   │
│                                                                              │
│                      ┌─────────────────┐                                    │
│                      │   🗄️ POSTGRES   │                                    │
│                      │   (Ledger)      │                                    │
│                      └─────────────────┘                                    │
│                                                                              │
└──────────────────────────────────────────────────────────────────────────────┘
```

---

## 1. UBL Kernel (Rust Server)

**Location:** `ubl/kernel/rust/ubl-server/`

**Port:** 8080 (default)

### Core Endpoints (Ledger)

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/health` | GET | Health check |
| `/state/:container_id` | GET | Get ledger state (sequence, hash) |
| `/link/validate` | POST | Validate a link without committing |
| `/link/commit` | POST | Commit a link atomically |
| `/ledger/:container_id/tail` | GET | SSE stream of new ledger entries |
| `/atom/:hash` | GET | Fetch atom data by hash |

### Console API v1.1 (Job Orchestration)

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/v1/policy/permit` | POST | Request execution permit |
| `/v1/commands/issue` | POST | Register a command for Runner |
| `/v1/query/commands` | GET | List pending commands (Runner pulls) |
| `/v1/exec.finish` | POST | Register execution receipt |

### Projections (Read-Optimized Views)

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/query/jobs` | GET | List all jobs |
| `/query/jobs/:job_id` | GET | Get job details |
| `/query/jobs/:job_id/approvals` | GET | Get pending approvals |
| `/query/conversations/:id/jobs` | GET | Jobs in a conversation |
| `/query/conversations/:id/messages` | GET | Messages in a conversation |

### Identity (WebAuthn + ASC)

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/id/agents` | POST | Create LLM/App identity |
| `/id/agents/:sid/asc` | POST | Issue Agent Service Credential |
| `/id/whoami` | GET | Get current identity |

---

## 2. Office (LLM Operating System)

**Location:** `apps/office/`

**Port:** 8081 (default)

### How Office Connects to UBL

```rust
// In Office main.rs
let ubl_client = Arc::new(UblClient::with_generated_key(
    &config.ubl.endpoint,     // e.g., "http://localhost:8080"
    &config.ubl.container_id, // e.g., "C.Office"
    config.ubl.timeout_ms,
));
```

### The Flow: Permit → Command → Execute → Receipt

```
1. User requests a Job via Messenger
   │
   ├──► Messenger Frontend sends POST /api/jobs
   │    to a backend (could be UBL Kernel or separate)
   │
   ▼
2. Backend requests PERMIT from UBL
   │
   │    POST /v1/policy/permit
   │    {
   │      tenant_id: "T1",
   │      actor_id: "user_123",
   │      intent: "execute_llm_task",
   │      jobType: "llm_inference",
   │      params: { prompt: "..." },
   │      target: "lab-512"
   │    }
   │
   │    Response: { permit: { jti, exp, scopes, sig }, allowed: true }
   │
   ▼
3. Backend issues COMMAND to UBL
   │
   │    POST /v1/commands/issue
   │    {
   │      jti: "...",
   │      tenant_id: "T1",
   │      jobId: "job_abc",
   │      jobType: "llm_inference",
   │      params: { ... },
   │      permit: { ... },
   │      target: "lab-512",
   │      office_id: "office_1"
   │    }
   │
   ▼
4. Office (Runner) POLLS for pending commands
   │
   │    GET /v1/query/commands?tenant_id=T1&target=lab-512&pending=1
   │
   │    Response: [{ jti, job_id, params, permit, ... }]
   │
   ▼
5. Office EXECUTES the job
   │
   │    - Loads Entity (persistent Chair)
   │    - Generates Narrative (beautiful onboarding)
   │    - Routes to best LLM provider
   │    - Streams progress via WebSocket
   │
   ▼
6. Office submits RECEIPT to UBL
   │
   │    POST /v1/exec.finish
   │    {
   │      tenant_id: "T1",
   │      jobId: "job_abc",
   │      status: "completed",
   │      logs_hash: "b3e1...",
   │      artifacts: ["url1", "url2"],
   │      usage: { tokens: 1500 }
   │    }
   │
   ▼
7. Ledger appends job.completed event
   │
   │    POST /link/commit
   │    {
   │      container_id: "C.Jobs",
   │      atom: { type: "job.completed", job_id: "job_abc", ... },
   │      intent_class: "Observation"
   │    }
   │
   ▼
8. Projections update & SSE pushes to Messenger
```

---

## 3. Messenger Frontend

**Location:** `apps/messenger/frontend/`

**Port:** 3000 (Vite dev server)

### API Client Configuration

```typescript
// services/apiClient.ts
function getBaseUrl(): string {
  // 1. Check localStorage (set by BridgeConfig)
  const storedUrl = localStorage.getItem('ubl_api_base_url');
  if (storedUrl) return storedUrl.replace(/\/$/, '');
  
  // 2. Fall back to env variable
  const envBase = import.meta.env?.VITE_API_BASE_URL;
  return (envBase || '').replace(/\/$/, '');
}

function getToken(): string | null {
  const raw = localStorage.getItem('ubl_session');
  if (!raw) return null;
  return JSON.parse(raw)?.token || null;
}
```

### Key Services

| Service | File | Purpose |
|---------|------|---------|
| `ublApi` | `services/ublApi.ts` | Entities, conversations, messages |
| `jobsApi` | `services/jobsApi.ts` | Job CRUD, WebSocket subscription |
| `apiClient` | `services/apiClient.ts` | HTTP client with auth |

### Real-time Updates

The frontend subscribes to job updates via WebSocket:

```typescript
// services/jobsApi.ts
export function subscribeToJobUpdates(handler: JobEventHandler): () => void {
  const url = getWebSocketUrl(); // ws://localhost:8080/ws
  
  ws = new WebSocket(url);
  
  ws.onmessage = (event) => {
    const wsEvent = JSON.parse(event.data);
    
    switch (wsEvent.type) {
      case 'JobUpdate':
        handler({ type: 'job_updated', job_id: wsEvent.payload.job_id, ... });
        break;
      case 'JobComplete':
        handler({ type: 'job_completed', job_id: wsEvent.payload.job_id, ... });
        break;
      case 'ApprovalNeeded':
        handler({ type: 'approval_required', job_id: wsEvent.payload.job_id, ... });
        break;
    }
  };
  
  return () => ws.close();
}
```

---

## 4. Environment Setup

### UBL Kernel (.env)

```env
DATABASE_URL=postgres://ubl_dev@localhost:5432/ubl_dev
PORT=8080
RUST_LOG=ubl_server=info
WEBAUTHN_RP_ID=localhost
WEBAUTHN_ORIGIN=http://localhost:8080
```

### Office (config/development.toml)

```toml
[server]
host = "0.0.0.0"
port = 8081

[ubl]
endpoint = "http://localhost:8080"
container_id = "C.Office"
timeout_ms = 30000

[llm]
provider = "anthropic"  # or "openai", "local"
model = "claude-3-5-sonnet-20241022"
api_key = "${ANTHROPIC_API_KEY}"
```

### Messenger Frontend (.env)

```env
VITE_API_BASE_URL=http://localhost:8080
```

---

## 5. Database Initialization

Run all migrations in order:

```bash
cd ubl/sql

# Connect to Postgres
psql -U ubl_dev -d ubl_dev

# Run migrations
\i 001_ledger.sql
\i 002_idempotency.sql
\i 003_observability.sql
\i 004_disaster_recovery.sql
\i 005_atoms.sql
\i 006_projections.sql
\i 007_pacts.sql
\i 008_policy_engine.sql
\i 010_sessions.sql
\i 020_console_v1_1.sql
\i 021_registry_v1_1.sql
```

---

## 6. Quick Start (All Three Components)

### Terminal 1: Start Postgres

```bash
# Using Docker
docker run -d \
  --name ubl-postgres \
  -e POSTGRES_USER=ubl_dev \
  -e POSTGRES_DB=ubl_dev \
  -e POSTGRES_HOST_AUTH_METHOD=trust \
  -p 5432:5432 \
  postgres:16
```

### Terminal 2: Start UBL Kernel

```bash
cd ubl/kernel/rust/ubl-server
cargo run --release
# Server at http://localhost:8080
```

### Terminal 3: Start Office

```bash
cd apps/office
ANTHROPIC_API_KEY=sk-... cargo run --release
# Server at http://localhost:8081
```

### Terminal 4: Start Messenger Frontend

```bash
cd apps/messenger/frontend
npm install
npm run dev
# Frontend at http://localhost:3000
```

---

## 7. The Complete Data Flow

```
┌────────────────────────────────────────────────────────────────────────────────┐
│                                                                                │
│   USER types message in Messenger                                              │
│      │                                                                         │
│      ▼                                                                         │
│   Frontend sends POST /api/messages                                            │
│      │                                                                         │
│      ▼                                                                         │
│   UBL Kernel:                                                                  │
│   1. Validates request                                                         │
│   2. POST /link/commit → Ledger appends message.created event                  │
│   3. Projection updates projection_messages                                    │
│   4. SSE broadcasts to all subscribers                                         │
│      │                                                                         │
│      ▼                                                                         │
│   If message mentions @agent or contains job request:                          │
│   1. POST /v1/policy/permit → Get execution permit                             │
│   2. POST /v1/commands/issue → Queue command for Office                        │
│   3. POST /link/commit → job.created event                                     │
│      │                                                                         │
│      ▼                                                                         │
│   Office (polling /v1/query/commands):                                         │
│   1. Picks up pending command                                                  │
│   2. Loads Entity (Chair) from UBL                                             │
│   3. Generates Narrative with context                                          │
│   4. Routes to LLM provider (Anthropic/OpenAI)                                 │
│   5. Streams progress via WebSocket                                            │
│   6. POST /v1/exec.finish → Receipt                                            │
│   7. POST /link/commit → job.completed event                                   │
│      │                                                                         │
│      ▼                                                                         │
│   Ledger:                                                                      │
│   1. Projection updates projection_jobs                                        │
│   2. SSE broadcasts job_completed                                              │
│      │                                                                         │
│      ▼                                                                         │
│   Frontend receives WebSocket event:                                           │
│   1. Updates job card in UI                                                    │
│   2. Shows completion status                                                   │
│   3. Renders result/artifacts                                                  │
│                                                                                │
└────────────────────────────────────────────────────────────────────────────────┘
```

---

## 8. Key Concepts

### Containers

Each container is an isolated append-only ledger:

| Container | Purpose |
|-----------|---------|
| `C.Messenger` | Messages, conversations, reactions |
| `C.Jobs` | Job lifecycle events |
| `C.Office` | Office session events, handovers |
| `C.Artifacts` | File uploads, results |
| `C.Policy` | Policy rules and decisions |

### Intent Classes (Physics)

Every ledger commit declares its intent:

| Class | Physics | Description |
|-------|---------|-------------|
| `Observation` | Δ = 0 | Read-only, no state change |
| `Conservation` | Δ ≤ 0 | Maintains or reduces value |
| `Entropy` | Δ < 0 | Destroys value (irreversible) |
| `Evolution` | Δ > 0 | Creates value (requires pact) |

### Pacts (Multi-sig Authorization)

High-risk operations require multi-signature approval:

```json
{
  "pact_id": "pact_delete_account",
  "threshold": 2,
  "signers": ["admin_1_pubkey", "admin_2_pubkey", "user_pubkey"],
  "window": { "not_before": "...", "not_after": "..." },
  "risk_level": "L4"
}
```

---

## 9. Troubleshooting

### Frontend can't connect to backend

1. Check `localStorage.getItem('ubl_api_base_url')` in browser console
2. Verify CORS: UBL Kernel allows all origins by default
3. Ensure UBL Kernel is running on the expected port

### Jobs stuck in pending

1. Check if Office is running and polling
2. Verify Office's `ubl.endpoint` config points to running UBL Kernel
3. Check PostgreSQL for pending commands: `SELECT * FROM commands WHERE pending = 1`

### SSE not receiving events

1. Check browser Network tab for `/ledger/:container/tail` connection
2. Verify PostgreSQL LISTEN/NOTIFY is working
3. Check UBL Kernel logs for SSE subscription

### Signature verification failed

1. Ensure Office has a valid Ed25519 signing key
2. Check atom canonicalization is consistent
3. Verify pubkey is registered with UBL Identity

---

## 10. Architecture Decisions

| Decision | Rationale |
|----------|-----------|
| Ed25519 signatures | Fast, small, well-audited |
| Append-only ledger | Immutable audit trail |
| Projections for reads | Fast queries without touching ledger |
| SSE for real-time | Simple, HTTP-based, no WS complexity |
| Console API for jobs | Decouples execution from ledger |
| Pacts for high-risk ops | Multi-sig governance |

---

## Summary

UBL 3.0 is a **production-grade, cryptographically-verified, event-sourced** system where:

1. **UBL Kernel** is the immutable source of truth
2. **Office** is the intelligent executor that gives LLMs dignity
3. **Messenger** is the beautiful human interface

All three components communicate via:
- **REST API** for commands
- **SSE/WebSocket** for real-time updates
- **PostgreSQL** for persistence

The wiring is designed for:
- ✅ Auditability (every action is a ledger entry)
- ✅ Security (Ed25519 signatures, pacts, policies)
- ✅ Scalability (projections, async execution)
- ✅ Real-time UX (SSE broadcasts)

