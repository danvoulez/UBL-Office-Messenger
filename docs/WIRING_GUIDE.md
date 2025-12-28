# 🔌 UBL Wiring Guide

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                                        USER ZONE                                                    │
│                                                                                                     │
│   ┌─────────────────────────┐                                                                      │
│   │     MESSENGER           │                                                                      │
│   │   (React Frontend)      │                                                                      │
│   │   apps/messenger/       │                                                                      │
│   └───────────┬─────────────┘                                                                      │
│               │                                                                                     │
│               │ HTTP/SSE                                                                            │
│               ▼                                                                                     │
├─────────────────────────────────────────────────────────────────────────────────────────────────────┤
│                                        LAB 256 (API ZONE)                                           │
│                                                                                                     │
│   ┌─────────────────────────────────────────────────────────────────────────────────────────────┐  │
│   │                                   UBL KERNEL                                                 │  │
│   │                              ubl/kernel/rust/ubl-server/                                     │  │
│   │                                                                                              │  │
│   │   ┌───────────────────┐   ┌───────────────────┐   ┌───────────────────┐                     │  │
│   │   │ Core Routes       │   │ Console v1.1      │   │ Projections       │                     │  │
│   │   │ /link/commit      │   │ /v1/policy/permit │   │ /query/jobs       │                     │  │
│   │   │ /ledger/:id/tail  │   │ /v1/commands/issue│   │ /query/office/*   │                     │  │
│   │   │ /state/:id        │   │ /v1/exec.finish   │   │ /query/messages   │                     │  │
│   │   └───────────────────┘   └───────────────────┘   └───────────────────┘                     │  │
│   │           │                         │                       │                                │  │
│   │           └─────────────────────────┼───────────────────────┘                                │  │
│   │                                     │                                                        │  │
│   │                              ┌──────┴──────┐                                                 │  │
│   │                              │ PostgreSQL  │                                                 │  │
│   │                              │ (ledger_*)  │                                                 │  │
│   │                              └─────────────┘                                                 │  │
│   └─────────────────────────────────────────────────────────────────────────────────────────────┘  │
│                                         │                                                           │
│   ┌─────────────────────────┐           │           ┌─────────────────────────┐                    │
│   │       OFFICE            │◄──────────┴───────────│      RUNNER             │                    │
│   │   (LLM Runtime)         │                       │   (Sandbox Executor)    │                    │
│   │   apps/office/          │                       │   ubl/runner/           │                    │
│   └─────────────────────────┘                       └─────────────────────────┘                    │
│                                                                 │                                   │
└─────────────────────────────────────────────────────────────────┼───────────────────────────────────┘
                                                                  │
                                                                  ▼
┌─────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                                        LAB 512 (SANDBOX ZONE)                                       │
│                                                                                                     │
│   ┌─────────────────────────────────────────────────────────────────────────────────────────────┐  │
│   │                               ISOLATED EXECUTION                                             │  │
│   │                               (nsjail/sandbox-exec)                                          │  │
│   │                                                                                              │  │
│   │   - S3/Artifacts storage                                                                     │  │
│   │   - Git repositories                                                                         │  │
│   │   - File system access                                                                       │  │
│   │   - Network (whitelisted egress)                                                             │  │
│   └─────────────────────────────────────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────────────────────────────────┘
```

## Containers

| Container | Color | Function | Zone |
|-----------|-------|----------|------|
| **C.Messenger** | 🟢 Verde | Chat, messages, conversations | LAB 256 |
| **C.Jobs** | 🔵 Azul | Work tracking, approvals | LAB 256 |
| **C.Office** | ⬛ Preto | LLM entities, sessions, audit | LAB 256 |
| **C.Runner** | 🟡 Amarelo | Sandbox execution, artifacts | LAB 512 |
| **C.Pacts** | 🔴 Vermelho | Collective authority, consensus | LAB 256 |
| **C.Policy** | ⚪ Branco | Rules, risk levels | LAB 256 |

## Flow: Message → Job → Execution

```
1. USER types message in Messenger
   │
   ▼
2. Messenger calls POST /messenger/messages
   │
   ▼
3. UBL Kernel:
   - Commits to C.Messenger ledger
   - Updates projection_messages
   - Sends SSE to subscribers
   │
   ▼
4. OFFICE (subscribed to C.Messenger SSE):
   - Receives message event
   - Builds ContextFrame
   - Calls LLM
   - LLM decides: propose job
   │
   ▼
5. OFFICE calls POST /v1/policy/permit (jobType: "file_organize")
   │
   ▼
6. UBL Kernel evaluates policy:
   - Risk level L2 → Grant permit
   │
   ▼
7. OFFICE calls POST /v1/commands/issue
   │
   ▼
8. UBL Kernel:
   - Commits command to C.Jobs ledger
   - Updates projection_jobs
   │
   ▼
9. RUNNER polls GET /v1/query/commands?pending=1
   - Receives command
   - Pulls artifacts
   │
   ▼
10. RUNNER executes in LAB 512 sandbox
    - File operations
    - Git commands
    │
    ▼
11. RUNNER calls POST /v1/exec.finish with receipt
    - Signed with runner key
    │
    ▼
12. UBL Kernel:
    - Verifies runner signature
    - Commits receipt to C.Jobs ledger
    - Updates projection_jobs
    │
    ▼
13. Messenger receives SSE update
    - Shows job completion to user
```

## Endpoint Reference

### UBL Kernel (port 8080)

#### Core Ledger
```
GET  /health                          → Health check
GET  /state/:container_id             → Container state (sequence, hash)
POST /link/validate                   → Validate link draft
POST /link/commit                     → Atomic append to ledger
GET  /ledger/:container_id/tail       → SSE stream of events
GET  /atom/:hash                      → Fetch atom by hash
```

#### Console v1.1 (Governance)
```
POST /v1/policy/permit                → Request permit for action
POST /v1/commands/issue               → Queue command for Runner
GET  /v1/query/commands               → List pending commands
POST /v1/exec.finish                  → Submit execution receipt
```

#### Identity (WebAuthn)
```
POST /id/register/begin               → Start passkey registration
POST /id/register/finish              → Complete registration
POST /id/login/begin                  → Start passkey login
POST /id/login/finish                 → Complete login
POST /id/stepup/begin                 → Start step-up auth (L4/L5)
POST /id/stepup/finish                → Complete step-up
POST /id/agents                       → Create LLM agent
POST /id/agents/:sid/asc              → Issue Agent Service Credential
```

#### Projections (Query)
```
GET  /query/jobs                      → List all jobs
GET  /query/jobs/:job_id              → Get job details
GET  /query/jobs/:job_id/approvals    → Get pending approvals
GET  /query/conversations/:id/jobs    → Jobs in conversation
GET  /query/conversations/:id/messages → Messages in conversation
GET  /query/office/entities           → List LLM entities
GET  /query/office/entities/:id       → Get entity details
GET  /query/office/entities/:id/sessions → Session history
GET  /query/office/entities/:id/handovers → Handover history
GET  /query/office/entities/:id/handovers/latest → Latest handover
GET  /query/office/audit              → Audit trail
```

#### Messenger Boundary
```
GET  /messenger/bootstrap             → Initial load (conversations, entities)
POST /messenger/messages              → Send message
GET  /messenger/conversations         → List conversations
POST /messenger/conversations         → Create conversation
POST /messenger/jobs/:id/approve      → Approve job
POST /messenger/jobs/:id/reject       → Reject job
GET  /messenger/entities              → List entities
```

### Office Runtime (apps/office/)

Office is a **client** of UBL, not a server. It:
1. Subscribes to UBL SSE for C.Messenger/C.Jobs events
2. Calls `/v1/policy/permit` before any mutation
3. Calls `/link/commit` to record events to C.Office
4. Calls `/v1/commands/issue` to queue work for Runner

#### Internal Modules
- `context/` - ContextFrame, Builder, Narrator, Memory
- `governance/` - Sanity, Constitution, Dreaming, Simulation
- `entity/` - Entity (Chair), Instance, Guardian
- `session/` - Session, Handover, Modes, TokenBudget
- `audit/` - Events, ToolAudit
- `job_executor/` - JobExecutor, ConversationContext
- `ubl_client/` - HTTP client for UBL Gateway

### Runner (ubl/runner/)

Runner is a **pull-only** worker that:
1. Polls `/v1/query/commands?pending=1`
2. Executes in isolated sandbox (LAB 512)
3. Signs receipts with Ed25519
4. Submits via `/v1/exec.finish`

## Authentication Flow

```
┌─────────────────────────────────────────────────────────────────────────┐
│                          IDENTITY HIERARCHY                              │
│                                                                          │
│   ┌─────────────────────────────────────────────────────────────────┐   │
│   │                         HUMAN (Passkey)                          │   │
│   │                                                                  │   │
│   │   - Full identity in UBL                                         │   │
│   │   - Can do L0-L5 operations                                      │   │
│   │   - Step-up required for L4/L5                                   │   │
│   │   - Owns entities, approves jobs                                 │   │
│   │                                                                  │   │
│   │              │                                                   │   │
│   │              │ Issues ASC (Agent Service Credential)             │   │
│   │              ▼                                                   │   │
│   │                                                                  │   │
│   │   ┌─────────────────────────────────────────────────────────┐   │   │
│   │   │                    LLM AGENT (ASC)                       │   │   │
│   │   │                                                          │   │   │
│   │   │   - Limited scopes (containers, intents, max_delta)      │   │   │
│   │   │   - Can only do L0-L2 operations                         │   │   │
│   │   │   - NO Entropy, NO Evolution                             │   │   │
│   │   │   - Worker, not owner                                    │   │   │
│   │   │                                                          │   │   │
│   │   │              │                                           │   │   │
│   │   │              │ Proposes jobs                             │   │   │
│   │   │              ▼                                           │   │   │
│   │   │                                                          │   │   │
│   │   │   ┌─────────────────────────────────────────────────┐   │   │   │
│   │   │   │                   RUNNER                         │   │   │   │
│   │   │   │                                                  │   │   │   │
│   │   │   │   - Ed25519 keypair (persistent)                 │   │   │   │
│   │   │   │   - Executes approved commands only              │   │   │   │
│   │   │   │   - Signs receipts for accountability            │   │   │   │
│   │   │   │   - Isolated in LAB 512                          │   │   │   │
│   │   │   │                                                  │   │   │   │
│   │   │   └─────────────────────────────────────────────────┘   │   │   │
│   │   └─────────────────────────────────────────────────────────┘   │   │
│   └─────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────┘
```

## Database Schema (Projections)

```sql
-- C.Messenger projections
projection_messages (message_id, conversation_id, sender_id, content, ...)
projection_conversations (conversation_id, title, participants, ...)

-- C.Jobs projections
projection_jobs (job_id, conversation_id, title, status, assigned_to, ...)
projection_approvals (approval_id, job_id, approver_id, decision, ...)

-- C.Office projections
office_entities (entity_id, name, entity_type, constitution, ...)
office_sessions (session_id, entity_id, session_type, tokens_used, ...)
office_handovers (handover_id, entity_id, session_id, content, ...)
office_audit_log (audit_id, entity_id, event_type, event_data, ...)

-- Core ledger (source of truth)
ledger_entry (id, container_id, sequence, entry_hash, entry_json, ...)
```

## Quick Start

```bash
# 1. Start PostgreSQL
createdb ubl_dev

# 2. Run migrations
cd ubl/kernel/rust/ubl-server
psql ubl_dev < ../../sql/*.sql

# 3. Start UBL Kernel
cargo run

# 4. Start Messenger Frontend
cd apps/messenger/frontend
npm install && npm run dev

# 5. (Optional) Start Office
cd apps/office
cargo run

# 6. (Optional) Start Runner
cd ubl/runner
npx tsx pull_only.ts
```

## Environment Variables

```bash
# UBL Kernel
DATABASE_URL=postgres://user@localhost:5432/ubl_dev
WEBAUTHN_RP_ID=localhost
WEBAUTHN_ORIGIN=http://localhost:8080

# Office
UBL_ENDPOINT=http://localhost:8080
LLM_PROVIDER=anthropic
ANTHROPIC_API_KEY=sk-...

# Runner
UBL_ENDPOINT=http://localhost:8080
UBL_KEYS_DIR=~/.ubl/keys
```

## Done Checklist

- [x] UBL Kernel with PostgreSQL ledger
- [x] Console v1.1 (permits, commands, receipts)
- [x] Identity (WebAuthn/Passkey)
- [x] Projections (jobs, messages, office)
- [x] SSE with Last-Event-ID support
- [x] Messenger Frontend (React)
- [x] Office (LLM Runtime with full spec)
- [x] Runner (pull-only, signed receipts)
- [x] Container structure (C.Messenger, C.Jobs, C.Office, C.Runner)
- [ ] Production PostgreSQL (Unix socket, not TCP)
- [ ] CI/CD with passkey-signed permits
- [ ] End-to-end tests
