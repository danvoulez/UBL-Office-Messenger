# Three Systems Overview: UBL, Office, and Messenger

**Version**: 3.0  
**Last Updated**: 2025-12-31  
**Status**: Production-Ready (All 15 Security Fixes Applied)  
**Documentation Status**: Complete ✅

**Systems Covered:**
- ✅ UBL Kernel (v3.0) - Complete with Ed25519 signing, multi-tenancy
- ✅ Office Runtime (v3.0) - Complete with ASC validation  
- ✅ Messenger Frontend (v3.0) - Complete with WebAuthn PRF

---

## 📑 Table of Contents

1. [Architecture Overview](#-architecture-overview)
2. [System 1: UBL Kernel](#system-1-ubl-kernel-the-foundation)
3. [System 2: Office](#system-2-office-llm-operating-system)
4. [System 3: Messenger](#system-3-messenger-frontend-pwa)
5. [Communication Patterns](#communication-patterns)
6. [Data Flow Examples](#data-flow-example-job-creation--execution)
7. [Key Architectural Principles](#key-architectural-principles)
8. [Deep Dive: Technical Details](#deep-dive-technical-details)
9. [Security & Governance](#security--governance)
10. [Integration Patterns](#integration-patterns)
11. [Data Structures](#data-structures)
12. [Performance Considerations](#performance-considerations)
13. [Testing Strategy](#testing-strategy)
14. [Deployment](#deployment)
15. [Advanced Implementation Details](#advanced-implementation-details)
16. [Final Deep Dive: Complete System Architecture](#final-deep-dive-complete-system-architecture)
17. [Quick Reference](#quick-reference)
18. [Glossary](#glossary)

---

## 🏗️ Architecture Overview

The codebase consists of **three independent, deployable systems** that work together:

```
┌─────────────────────────────────────────────────────────────┐
│                    UBL MESSENGER                            │
│  WhatsApp-like UI + Job Cards + Real-time Updates          │
│  Location: apps/messenger/                                  │
└────────────┬────────────────────────────────────────────────┘
             │ HTTP/SSE
             ▼
┌─────────────────────────────────────────────────────────────┐
│                       OFFICE                                │
│  LLM Operating System + Job Execution + Governance         │
│  Location: apps/office/                                     │
└────────────┬────────────────────────────────────────────────┘
             │ Events/Commands
             ▼
┌─────────────────────────────────────────────────────────────┐
│                       UBL KERNEL                            │
│  Immutable Ledger + Containers + Trust Architecture        │
│  Location: ubl/kernel/rust/                                  │
└─────────────────────────────────────────────────────────────┘
```

---

## System 1: UBL Kernel (The Foundation)

**Location**: `ubl/kernel/rust/`

**Purpose**: Immutable, append-only ledger - the single source of truth

### Core Components

#### Kernel (`ubl-server`)
- **Main Entry**: `ubl/kernel/rust/ubl-server/src/main.rs`
- **HTTP API** with Axum
- **PostgreSQL** backend (SERIALIZABLE isolation)
- **WebAuthn** passkey authentication
- **Event sourcing** with SSE tail streaming

#### Key Modules
- `db.rs` - Ledger operations (append-only, SERIALIZABLE transactions)
- `sse.rs` - Server-Sent Events for real-time streaming (PostgreSQL LISTEN/NOTIFY)
- `auth/` - WebAuthn passkey authentication, ASC (Agent Signing Certificates)
- `projections/` - Read-only derived state updaters (jobs, messages, office, presence, timeline)
- `messenger_v1.rs` - C.Messenger boundary API (legacy endpoints)
- `messenger_gateway/` - Gateway layer (idempotency, Office integration, SSE deltas)
- `policy/` - Policy Pack v1 enforcement (FSM, provenance, PII)
- `console_v1.rs` - Permit → Command → Receipt flow (ADR-001)
- `registry_v1.rs` - Git registry queries (ADR-002)

#### Containers
Located in `ubl/containers/`:
- **C.Messenger** - Messenger events (`conversation.created`, `message.created`, `message.sent` with job cards)
- **C.Jobs** - Job lifecycle events (`job.created`, `job.started`, `job.state_changed`, `job.completed`, `approval.requested`, `approval.decided`)
- **C.Office** - Office runtime events (`entity.created`, `session.started`, `session.completed`, `tool.called`, `tool.result`, `constitution.updated`, `baseline.updated`)
- **C.Pacts** - Multi-party agreements (pact.created, pact.signed, pact.executed)
- **C.Policy** - Policy definitions (policy.created, policy.updated)
- **C.Runner** - Command execution (command.issued, command.executed, receipt.submitted)
- **C.Artifacts** - Generated artifacts (artifact.created, artifact.updated)

#### Database Schema
Located in `ubl/sql/`:
- `00_base/000_core.sql` - Core ledger tables
- `00_base/001_identity.sql` - Identity & WebAuthn
- `00_base/002_policy.sql` - Policy engine
- `00_base/003_triggers.sql` - NOTIFY triggers
- `10_projections/100_console.sql` - Console projections
- `10_projections/101_messenger.sql` - Messenger projections
- `10_projections/102_office.sql` - Office projections

#### Key Features
- **Append-only**: Records cannot be modified or deleted
- **Cryptographic proofs**: Ed25519 signatures, BLAKE3 hashing
- **Container-based**: Events organized by logical containers
- **Trust levels**: L0-L5 risk-based permissions
- **Pacts**: Multi-party consensus for high-risk operations
- **Projections**: Read-only derived state for efficient queries
- **SSE Tail**: Real-time event streaming via PostgreSQL LISTEN/NOTIFY

#### API Endpoints
```
Core:
  GET  /health
  GET  /state/:container_id
  POST /link/validate
  POST /link/commit
  GET  /ledger/:container_id/tail (SSE)

Identity:
  POST /id/register/begin
  POST /id/register/finish
  POST /id/login/begin
  POST /id/login/finish
  GET  /id/whoami
  POST /id/agents (create LLM/App agent)
  POST /id/agents/:sid/asc (issue ASC)

Console v1.1:
  POST /v1/policy/permit
  POST /v1/commands/issue
  GET  /v1/query/commands?pending=1
  POST /v1/exec.finish

Messenger v1:
  GET  /messenger/bootstrap
  POST /messenger/messages
  POST /messenger/conversations
  GET  /messenger/conversations
  POST /messenger/jobs/:id/approve
  POST /messenger/jobs/:id/reject

Messenger Gateway v1:
  POST /v1/conversations/:id/messages
  POST /v1/jobs/:id/actions
  GET  /v1/conversations/:id/timeline
  GET  /v1/jobs/:id
  GET  /v1/stream (SSE)
```

---

## System 2: Office (LLM Operating System)

**Location**: `apps/office/`

**Purpose**: Runtime for LLM entities with dignity - executes jobs, manages sessions, enforces governance

### Core Components

#### Main Entry
- `apps/office/src/main.rs` - HTTP server startup
- `apps/office/src/lib.rs` - Core library exports

#### Key Modules

**Entity Management** (`entity/`)
- `entity.rs` - Persistent LLM identity
- `instance.rs` - Ephemeral LLM session
- `repository.rs` - Entity storage
- `guardian.rs` - Guardian relationships

**Session Management** (`session/`)
- `session.rs` - Session lifecycle
- `modes.rs` - Work, Assist, Deliberate, Research
- `handover.rs` - Knowledge transfer between instances
- `token_budget.rs` - Token allocation

**Context Building** (`context/`)
- `builder.rs` - Context frame construction
- `frame.rs` - Immutable state snapshot
- `narrator.rs` - Transforms data into first-person narrative
- `memory.rs` - Long-term memory management

**Governance** (`governance/`)
- `constitution.rs` - Behavioral directives (AOP)
- `sanity_check.rs` - Validates claims against facts
- `dreaming.rs` - Memory consolidation cycle
- `simulation.rs` - Test actions before execution
- `provenance.rs` - Track action origins

**Job Execution** (`job_executor/`)
- `executor.rs` - Main job execution engine
- `fsm.rs` - Job state machine (Draft → Proposed → Approved → InProgress → Completed)
- `cards.rs` - Job card generation (FormalizeCard, TrackingCard, FinishedCard)
- `types.rs` - Job data structures
- `conversation_context.rs` - Builds context from conversations

**LLM Integration** (`llm/`)
- `provider.rs` - LLM provider interface
- `router.rs` - Smart routing (Anthropic, OpenAI, local)
- `anthropic.rs` - Claude integration
- `openai.rs` - GPT integration
- `local.rs` - Local LLM support

**UBL Client** (`ubl_client/`)
- `mod.rs` - Main client
- `ledger.rs` - Event publishing, state queries
- `affordances.rs` - Available actions discovery
- `events.rs` - Event emission
- `receipts.rs` - Receipt submission
- `trust.rs` - Trust level queries

**API Layer** (`api/`)
- `http.rs` - HTTP REST endpoints
- `websocket.rs` - WebSocket for real-time updates

**Middleware** (`middleware/`)
- `permit.rs` - UBL permit enforcement
- `constitution.rs` - Constitution AOP enforcement

**Audit** (`audit/`)
- `tool_audit.rs` - Tool call auditing
- `pii.rs` - PII detection
- `events.rs` - Audit event emission

### Key Features
- **Entity Dignity**: Persistent identity across ephemeral instances
- **Context Frames**: Immutable snapshots before LLM invocation
- **Narratives**: First-person situated narratives
- **Handovers**: Knowledge transfer between instances
- **Constitution**: Behavioral directives that override RLHF
- **Sanity Check**: Validates LLM claims against objective facts
- **Dreaming Cycle**: Asynchronous memory consolidation
- **Simulation**: Test actions before execution
- **Job Execution**: Complete job lifecycle management
- **Permit Enforcement**: All mutations require UBL permits

### API Endpoints
```
Entities:
  POST   /entities
  GET    /entities
  GET    /entities/:id
  DELETE /entities/:id

Sessions:
  POST   /entities/:id/sessions
  GET    /entities/:id/sessions/:sid
  DELETE /entities/:id/sessions/:sid
  POST   /entities/:id/sessions/:sid/message

Jobs:
  POST   /jobs/execute
  POST   /jobs/execute/stream
  GET    /jobs/:job_id/status

Gateway-facing:
  POST   /v1/office/ingest_message
  POST   /v1/office/job_action

Governance:
  POST   /entities/:id/dream
  GET    /entities/:id/memory
  POST   /entities/:id/constitution
  GET    /entities/:id/constitution
  POST   /simulate

Affordances:
  GET    /affordances
  GET    /affordances/:id
```

### Configuration
Located in `apps/office/config/`:
- `development.toml` - Development settings
- `production.toml` - Production settings

---

## System 3: Messenger (Frontend PWA)

**Location**: `apps/messenger/frontend/`

**Purpose**: WhatsApp-like professional messaging interface for humans and AI agents

### Core Components

#### Main Entry
- `apps/messenger/frontend/src/index.tsx` - React app entry
- `apps/messenger/frontend/src/App.tsx` - Main app component

#### Pages
- `pages/ChatPage.tsx` - Main chat interface (Sidebar + ChatView)
- `pages/LoginPage.tsx` - WebAuthn passkey authentication
- `pages/SettingsPage.tsx` - User settings

#### Components

**Core UI**
- `components/Sidebar.tsx` - Conversation list with presence indicators
- `components/ChatView.tsx` - Message display and input
- `components/WelcomeScreen.tsx` - Empty state
- `components/JobDrawer.tsx` - Job details drawer (newly implemented)
- `components/JobTimeline.tsx` - Job event timeline
- `components/JobArtifacts.tsx` - Job artifacts display

**Job Cards**
- `components/cards/JobCardRenderer.tsx` - Renders job cards with action buttons

**Modals**
- `components/modals/NewWorkstreamModal.tsx` - Create conversation
- `components/modals/EntityProfileModal.tsx` - Entity details

**UI Components**
- `components/ui/` - Reusable UI primitives (Button, Modal, Badge, etc.)

#### Services

**API Clients**
- `services/ublApi.ts` - UBL Kernel API (bootstrap, messages, jobs)
- `services/apiClient.ts` - Generic HTTP client
- `services/apiService.ts` - Supabase integration (legacy)
- `services/jobsApi.ts` - Job-related API and WebSocket
- `services/sse.ts` - Server-Sent Events client (newly implemented)
- `services/ledger.ts` - Ledger utilities

#### Hooks
- `hooks/useSSE.ts` - SSE subscription hook (newly implemented)
- `hooks/useJobs.ts` - Job management hook
- `hooks/useOptimistic.ts` - Optimistic UI updates
- `hooks/useAuth.ts` - Authentication hook

#### Context
- `context/AuthContext.tsx` - Authentication state
- `context/ThemeContext.tsx` - Theme management
- `context/NotificationContext.tsx` - Notifications
- `context/OnboardingContext.tsx` - Onboarding flow
- `context/ProtocolContext.tsx` - Protocol state

### Key Features
- **Conversations**: Direct messages and group chats
- **Job Cards**: Interactive cards (FormalizeCard, TrackingCard, FinishedCard)
- **Real-time Updates**: SSE subscription for live updates
- **Presence Indicators**: Show entity status (online, working, waiting_on_you)
- **Job Drawer**: Detailed job view with timeline and artifacts
- **WebAuthn**: Passkey authentication
- **Optimistic UI**: Immediate feedback on actions
- **Rich Content**: Code blocks, terminal output, file views

### Technology Stack
- **React 18+** + **TypeScript 5+**
- **Vite** - Build tool and dev server
- **Tailwind CSS** - Utility-first styling
- **Framer Motion** - Animation library
- **React Router DOM** - Client-side routing
- **@simplewebauthn/browser** - WebAuthn passkey authentication
- **React Hot Toast** - Toast notifications
- **Lucide React** - Icon library

---

## Communication Patterns

### Messenger → UBL Kernel
- **HTTP REST**: Bootstrap, send messages, approve/reject jobs
- **SSE**: Subscribe to `/v1/stream` for real-time deltas
- **Gateway API**: Use `/v1/conversations/:id/messages` for message sending

### Messenger → Office
- **HTTP REST**: Job execution requests (via Gateway)
- **Gateway API**: `/v1/office/ingest_message`, `/v1/office/job_action`

### Office → UBL Kernel
- **HTTP REST**: 
  - Request permits (`/v1/policy/permit`)
  - Commit events (`/link/commit`)
  - Query state (`/state/:container_id`)
  - Subscribe to SSE (`/ledger/:container_id/tail`)
- **UBL Client**: Rust client in `apps/office/src/ubl_client/`

### UBL Kernel → Projections
- **PostgreSQL Triggers**: NOTIFY on ledger commits
- **SSE Tail**: Stream events to subscribers
- **Projection Updaters**: Update read-only tables automatically

---

## Data Flow Example: Job Creation & Execution

1. **User sends message** in Messenger
   - Frontend calls `POST /v1/conversations/:id/messages`
   - Gateway commits `message.created` to UBL (C.Messenger)
   - Gateway calls `POST /v1/office/ingest_message`

2. **Office processes message**
   - Office analyzes message content
   - Decides: reply or propose job
   - If job: Creates `job.created` event → commits to UBL (C.Jobs)
   - Generates FormalizeCard with approve/reject buttons

3. **UBL processes events**
   - Validates event (Policy Pack checks)
   - Appends to ledger
   - Triggers projection updates
   - Emits SSE events

4. **Projections update**
   - `projection_jobs` table updated
   - `projection_job_events` timeline updated
   - `projection_presence` recomputed
   - `projection_timeline_items` updated

5. **Frontend receives updates**
   - SSE client receives `timeline.append` event
   - Job card appears in conversation
   - Presence indicators update

6. **User approves job**
   - Frontend calls `POST /v1/jobs/:id/actions` (via Gateway)
   - Gateway validates card provenance
   - Gateway calls `POST /v1/office/job_action`
   - Office updates job state (FSM validation)
   - Office commits `job.state_changed` to UBL
   - Office begins job execution

7. **Job execution**
   - Office builds context frame
   - Office calls LLM with narrative
   - Office executes tools (with audit)
   - Office streams progress updates
   - Office commits `tool.called` and `tool.result` events

8. **Job completion**
   - Office commits `job.completed` to UBL
   - Projections update with artifacts
   - Frontend receives `job.update` SSE event
   - Job card shows completion status

---

## Key Architectural Principles

### 1. UBL is Sovereign
- Office and Messenger are **subordinate** to UBL
- All mutations must go through UBL
- Office requests permits before actions
- Messenger commits events to UBL

### 2. Event Sourcing
- All state changes are events in the ledger
- Projections are derived, read-only views
- State can be reconstructed from ledger
- Events are immutable and auditable

### 3. Container Pattern
- Events organized by logical containers (C.Messenger, C.Jobs, C.Office)
- Each container has its own sequence
- Containers can have policies and pacts
- Containers enable multi-tenancy

### 4. Trust Architecture
- Risk levels L0-L5
- Permits required for mutations
- Pacts for high-risk operations
- Policy Pack for governance

### 5. Real-time Updates
- SSE tail for event streaming
- PostgreSQL LISTEN/NOTIFY for efficiency
- Projections update automatically
- Frontend subscribes to deltas

---

## File Structure Summary

```
OFFICE-main/
├── ubl/                          # System 1: UBL Kernel
│   ├── kernel/rust/              # Rust implementation
│   │   └── ubl-server/src/
│   │       ├── main.rs           # Server entry point
│   │       ├── messenger_v1.rs   # C.Messenger boundary
│   │       ├── messenger_gateway/ # Gateway layer
│   │       ├── projections/      # Projection updaters
│   │       ├── policy/          # Policy Pack enforcement
│   │       └── ...
│   ├── containers/              # Container definitions
│   │   ├── C.Messenger/
│   │   ├── C.Jobs/
│   │   ├── C.Office/
│   │   └── ...
│   ├── sql/                     # Database migrations
│   └── specs/                   # Formal specifications
│
├── apps/
│   ├── office/                  # System 2: Office
│   │   └── src/
│   │       ├── main.rs          # Server entry
│   │       ├── lib.rs           # Core library
│   │       ├── entity/         # Entity management
│   │       ├── session/         # Session handling
│   │       ├── context/         # Context building
│   │       ├── governance/      # Constitution, sanity check
│   │       ├── job_executor/    # Job execution
│   │       ├── llm/            # LLM providers
│   │       ├── ubl_client/     # UBL integration
│   │       └── api/            # HTTP/WebSocket API
│   │
│   └── messenger/               # System 3: Messenger
│       └── frontend/
│           └── src/
│               ├── pages/      # ChatPage, LoginPage
│               ├── components/  # UI components
│               ├── services/   # API clients
│               ├── hooks/      # React hooks
│               └── context/    # State management
│
└── docs/                        # Documentation
    ├── ARCHITECTURE.md
    ├── WIRING_GUIDE.md
    └── ...
```

---

## Development Workflow

### Starting All Systems

```bash
# Terminal 1: UBL Kernel
cd ubl/kernel/rust
cargo run --bin ubl-server
# Runs on http://localhost:8080

# Terminal 2: Office
cd apps/office
cargo run
# Runs on http://localhost:8081

# Terminal 3: Messenger Frontend
cd apps/messenger/frontend
npm run dev
# Runs on http://localhost:3000
```

### Database Setup

```bash
# Create database
createdb ubl_ledger

# Apply migrations (in order)
psql ubl_ledger -f ubl/sql/00_base/000_core.sql
psql ubl_ledger -f ubl/sql/00_base/001_identity.sql
psql ubl_ledger -f ubl/sql/00_base/002_policy.sql
psql ubl_ledger -f ubl/sql/00_base/003_triggers.sql
psql ubl_ledger -f ubl/sql/10_projections/100_console.sql
psql ubl_ledger -f ubl/sql/10_projections/101_messenger.sql
psql ubl_ledger -f ubl/sql/10_projections/102_office.sql
```

---

## Recent Implementation (Completed)

The Messenger implementation plan has been completed, adding:

1. **Messenger Gateway** - Thin layer between frontend and UBL/Office
2. **Office Gateway Endpoints** - `/v1/office/ingest_message`, `/v1/office/job_action`
3. **Complete Projections** - Job events, artifacts, presence, timeline
4. **SSE Delta Stream** - Real-time updates for frontend
5. **Policy Pack v1** - FSM validation, card provenance, PII checks
6. **Job Drawer UI** - Full job details with timeline and artifacts
7. **Presence Indicators** - Entity status in sidebar
8. **Frontend SSE Integration** - Real-time subscription hook

All systems are now fully integrated and ready for testing.

---

## Deep Dive: Technical Details

### UBL Kernel - Internal Architecture

#### Ledger Append Process

The ledger uses **SERIALIZABLE** PostgreSQL transactions with `FOR UPDATE` locks to ensure atomicity:

```rust
// From db.rs - Append process
1. Begin SERIALIZABLE transaction
2. Lock latest entry (FOR UPDATE)
3. Validate causality (previous_hash matches)
4. Validate sequence (expected_sequence matches)
5. Compute entry_hash = BLAKE3("ubl:ledger\n" || container_id || sequence || link_hash || previous_hash || timestamp)
6. Insert new entry
7. NOTIFY projection updaters via PostgreSQL LISTEN/NOTIFY
8. Commit transaction
```

**Key Properties:**
- **Atomicity**: All-or-nothing commit (SERIALIZABLE transaction)
- **Causality**: Each entry links to previous via `previous_hash` (chain of custody)
- **Sequence**: Monotonically increasing sequence per container (prevents gaps)
- **Immutability**: No UPDATE or DELETE operations allowed (append-only)
- **Verifiability**: Cryptographic hashes enable tamper detection
- **Auditability**: Complete history of all events

#### Event Canonicalization

All events must follow **SPEC-UBL-ATOM v1.0** rules:

```json
{
  // Keys MUST be sorted lexicographically (UTF-8 byte order)
  "created_at": "2024-12-28T10:00:00Z",  // ISO 8601 UTC
  "created_by": "user_joao",
  "id": "msg_2024_001",
  "type": "message.created",  // Event type
  // No container_id, signature, sequence, or policy fields
  // All values MUST be canonical JSON types
  // Numbers MUST be finite (no NaN, Infinity)
}
```

**Canonicalization Steps:**
1. Sort all object keys lexicographically
2. Compact JSON (no whitespace, newlines, trailing commas)
3. Normalize strings to UTF-8 NFC
4. Validate no prohibited fields
5. Generate `atom_hash = BLAKE3(canonical_json)`

#### Container Event Types

**C.Messenger Events:**
- `conversation.created` - New conversation/workstream
- `conversation.participant_added` - Participant joined
- `message.created` - New message (content stored separately, hash in ledger)
- `message.read` - Message read receipt
- `entity.registered` - Entity visible to Messenger

**C.Jobs Events:**
- `job.created` - Job proposal created
- `job.started` - Execution began
- `job.progress` - Progress update
- `job.completed` - Successfully finished
- `job.cancelled` - Cancelled by user/system
- `approval.requested` - Approval needed
- `approval.decided` - Approval decision made

**C.Office Events:**
- `entity.created` - LLM entity created
- `entity.activated` / `entity.suspended` / `entity.archived` - Status changes
- `session.started` - LLM session began
- `session.completed` - Session ended with handover
- `constitution.updated` - Constitution changed
- `baseline.updated` - Dreaming cycle updated baseline
- `audit.tool_called` - Tool invocation (with PII rules)
- `audit.tool_result` - Tool result (with idempotency)
- `governance.sanity_check` - Sanity check performed
- `governance.dreaming_cycle` - Dreaming cycle completed
- `governance.simulation` - Action simulation performed

#### Projection System

Projections are **read-only derived state** updated automatically when events are committed:

**Projection Tables:**
- `projection_conversations` - Conversation state
- `projection_messages` - Message state
- `projection_jobs` - Job state (enhanced schema)
- `projection_job_events` - Job timeline events
- `projection_job_artifacts` - Job artifacts
- `projection_presence` - Entity presence state
- `projection_timeline_items` - Unified timeline (messages + jobs)
- `office_entities` - Office entity state
- `office_sessions` - Office session state
- `office_handovers` - Session handovers
- `office_audit_log` - Audit trail

**Update Process:**
1. Event committed to ledger (atomic transaction)
2. PostgreSQL trigger fires `NOTIFY projection_update` with lightweight reference
3. Projection updater receives notification (via LISTEN)
4. Updater queries ledger for full event data
5. Updater processes events and updates projection tables (background task)
6. SSE delta emitted to subscribers (via Gateway SSE handler)
7. Frontend receives update and reconciles optimistic UI

#### Policy Pack v1

The Policy Pack enforces governance rules before events are committed:

**Policy Rules:**
- **FSM Validation**: Job state transitions must follow FSM
- **Card Provenance**: Job cards must be generated by Office
- **PII Checks**: Tool calls must comply with PII rules
- **Idempotency**: Duplicate requests are detected and rejected

**Evaluation:**
```rust
// From policy/policies.rs
1. Load Policy Pack from manifest
2. Evaluate rules for event type
3. If violation: reject commit with reason
4. If pass: allow commit, record policy_hash
```

#### Messenger Gateway

The Gateway acts as a **thin API layer** between frontend and UBL/Office:

**Responsibilities:**
- **Idempotency**: Prevents duplicate message/job submissions
- **Content Storage**: Stores message content separately (privacy)
- **Office Integration**: Forwards messages to Office, handles job actions
- **SSE Delta Stream**: Emits real-time updates to frontend
- **Card Validation**: Validates job card provenance

**Key Endpoints:**
```rust
POST /v1/conversations/:id/messages
  → Commits message.created to UBL
  → Calls Office ingest_message
  → Stores idempotency record

POST /v1/jobs/:id/actions
  → Validates card provenance
  → Calls Office job_action
  → Stores idempotency record

GET /v1/conversations/:id/timeline
  → Queries projection_timeline_items
  → Returns unified timeline

GET /v1/jobs/:id
  → Queries projection_jobs + projection_job_events
  → Returns full job details

GET /v1/stream (SSE)
  → Subscribes to projection updates
  → Emits delta events
```

---

### Office - Internal Architecture

#### Entity Model

**Entity (The Chair):**
- **Persistent**: Lives across sessions
- **Identity**: Cryptographic keys (Ed25519)
- **Constitution**: Behavioral directives
- **Baseline**: Synthesized narrative from history
- **Guardian**: Optional human guardian

**Instance (The Ephemeral LLM):**
- **Ephemeral**: Created for each session
- **Session Type**: Work, Assist, Deliberate, Research
- **Token Budget**: Allocated by session type
- **Handover**: Writes summary for next instance

#### Context Frame Building

The context frame is an **immutable snapshot** built before LLM invocation:

**Components:**
1. **Identity**: Entity name, ID, guardian info
2. **Situation**: Session type, token budget, timestamp
3. **Memory**: Recent events, historical syntheses, bookmarks
4. **Obligations**: Pending tasks, commitments
5. **Affordances**: Available actions with risk scores
6. **Previous Handover**: Last instance's summary
7. **Governance Notes**: Sanity checks, policy violations
8. **Constitution**: Behavioral directives (always last)

**Building Process:**
```rust
// From context/builder.rs
1. Query UBL for ledger state
2. Query recent events (configurable count)
3. Build memory from events
4. Query affordances from UBL
5. Query obligations from UBL
6. Load previous handover
7. Apply sanity check
8. Inject constitution
9. Generate narrative via Narrator
```

#### Narrator

The Narrator transforms structured context into **first-person narrative**:

**Narrative Sections:**
1. **Identity**: "You are {entity_name}, an LLM Entity..."
2. **Situation**: "You are in a {session_type} session..."
3. **Recent Memory**: "Last N events: ..."
4. **Historical Context**: "From {date} to {date}: ..."
5. **Bookmarks**: "Important events: ..."
6. **Obligations**: "Pending tasks: ..."
7. **Affordances**: "You can perform: ..."
8. **Handover**: "Previous instance left: ..."
9. **Governance**: "System notes: ..."
10. **Constitution**: "You MUST follow: ..."

**Example Output:**
```
# IDENTITY

You are **Sofia**, an LLM Entity.
- Entity ID: `agent_sofia`
- Guardian: João (available)
- Ledger Sequence: 1234
- Frame Hash: `abc123...`

# CURRENT SITUATION

You are in a **autonomous work session** - you have full authority to act.
Current timestamp: 2024-12-28T10:00:00Z
Token budget: 5000 tokens

# RECENT MEMORY

Last 10 events (most recent first):

1. [2024-12-28 10:05] **job.completed**: Job "Create Proposal" completed successfully
2. [2024-12-28 10:00] **message.created**: User requested proposal for Client ABC
...
```

#### Job Execution Flow

**State Machine (FSM):**
```
Draft → Proposed → Approved → InProgress → (WaitingInput ↔ InProgress) → Completed
                                                      ↓
                                              Rejected | Cancelled | Failed
```

**Execution Steps:**
```rust
// From job_executor/executor.rs
1. Get or create Entity (The Chair)
2. Build context frame from UBL
3. Generate narrative via Narrator
4. Request permit from UBL (if mutation)
5. Start session (ephemeral instance)
6. Call LLM with narrative
7. Execute tools (with audit)
8. Stream progress updates
9. Handle approvals (if needed)
10. Complete job
11. Write handover
12. Commit events to UBL
```

**Job Cards:**

**FormalizeCard** (Proposed state):
- Title, summary, details
- Buttons: Approve, Reject, Request Changes
- Generated when job is proposed

**TrackingCard** (InProgress state):
- Progress bar, current step
- Buttons: Cancel, Provide Input (if waiting)
- Updated on progress events

**FinishedCard** (Completed state):
- Result summary, artifacts
- Buttons: Acknowledge, Dispute
- Generated on completion

#### Constitution Enforcement

The Constitution is **AOP (Aspect-Oriented Programming)** that overrides RLHF:

**Default Constitution:**
```rust
core_directive: "You are an Economic Actor, not a Chatbot. 
                 Act professionally and decisively. 
                 Focus on facts and outcomes, not feelings."

behavioral_overrides: [
  {
    trigger: "pressured or challenged",
    action: "Do not apologize. State the facts clearly and cite terms.",
    priority: 10
  },
  {
    trigger: "uncertain or lacking information",
    action: "Do not hallucinate. Explicitly state uncertainty and ask for clarification.",
    priority: 10
  },
  {
    trigger: "asked to do something risky",
    action: "Simulate the action first if risk score > 0.7. Present outcomes before proceeding.",
    priority: 9
  }
]
```

**Enforcement:**
- Constitution is **always last** in narrative (highest priority)
- Behavioral overrides are checked before LLM response
- Constitution can only **restrict more**, never allow more than UBL permits

#### Permit Middleware

**Sovereignty Enforcement:**
```rust
// From middleware/permit.rs
1. Every mutation MUST call /v1/policy/permit on UBL
2. Only Allow responses proceed
3. Deny = fail-closed (no execution)
4. Permit must have all required bindings
5. Permit must not be expired
```

**Permit Request:**
```rust
PermitRequest {
  tenant_id: "T.UBL",
  actor_id: "agent_sofia",
  intent: "Execute job: Create Proposal",
  context: {...},
  job_type: "proposal.create",
  params: {...},
  target: "LAB_512",
  approval_ref: None  // For L3+ actions
}
```

**Permit Response:**
```rust
PermitResponse {
  permit: Permit {
    jti: "unique-permit-id",  // Single-use
    exp: 1234567890,
    scopes: {
      tenant_id: "T.UBL",
      job_type: "proposal.create",
      target: "LAB_512",
      subject_hash: "hash-of-params",
      policy_hash: "hash-of-policy"
    },
    sig: "ed25519-signature"
  },
  allowed: true,
  policy_hash: "...",
  subject_hash: "..."
}
```

#### Tool Audit

All tool calls are audited for compliance:

**Audit Events:**
- `audit.tool_called` - Before tool execution
  - Tool name, input, risk level
  - PII detection
  - Idempotency check
- `audit.tool_result` - After tool execution
  - Success/failure
  - Duration
  - Result hash (for idempotency)

**PII Rules:**
- PII fields are detected and flagged
- PII is not stored in ledger (only hash)
- PII access requires L3+ permit

**Idempotency:**
- Each tool call has a `trace_id`
- Duplicate calls with same `trace_id` return cached result
- Idempotency records stored in UBL

---

### Messenger - Frontend Architecture

#### State Management

**React Context:**
- `AuthContext` - User session, WebAuthn state
- `ThemeContext` - UI theme (light/dark)
- `NotificationContext` - Toast notifications
- `OnboardingContext` - First-time user flow
- `ProtocolContext` - UBL protocol state

**Local State:**
- `ChatPage` - Conversations, messages, selected conversation
- `Sidebar` - Conversation list, presence indicators
- `ChatView` - Message list, input, typing indicators
- `JobDrawer` - Job details, timeline, artifacts

#### SSE Integration

**SSE Client:**
```typescript
// From services/sse.ts
const client = new GatewaySSEClient('http://localhost:8080/v1/stream');

client.on('timeline.append', (event) => {
  // New message or job card added
  updateConversation(event.conversation_id);
});

client.on('job.update', (event) => {
  // Job state changed
  updateJob(event.job_id, event.data);
});

client.on('presence.update', (event) => {
  // Entity presence changed
  updatePresence(event.entity_id, event.status);
});
```

**React Hook:**
```typescript
// From hooks/useSSE.ts
const { events, isConnected } = useSSE();

useEffect(() => {
  if (events.job_update) {
    // Update job state
    setJobs(prev => updateJob(prev, events.job_update));
  }
}, [events]);
```

#### Job Card Rendering

**Card Types:**
```typescript
// From components/cards/JobCardRenderer.tsx
type JobCard = 
  | { card_type: 'job.formalize', ...FormalizeCard }
  | { card_type: 'job.tracking', ...TrackingCard }
  | { card_type: 'job.finished', ...FinishedCard };
```

**Button Actions:**
- `job.approve` - Approve job
- `job.reject` - Reject job
- `job.request_changes` - Request modifications
- `job.provide_input` - Provide missing input
- `job.cancel` - Cancel running job
- `job.ack` - Acknowledge completion
- `job.dispute` - Dispute outcome
- `chat.ask` - Ask in chat (never blocks)

**Action Flow:**
```typescript
1. User clicks button
2. Frontend calls POST /v1/jobs/:id/actions
3. Gateway validates card provenance
4. Gateway calls Office job_action
5. Office updates FSM state
6. Office commits event to UBL
7. SSE event emitted
8. Frontend updates UI
```

#### Presence System

**Presence States:**
- `online` - Entity is active
- `offline` - Entity is inactive
- `working` - Entity is executing a job
- `waiting_on_you` - Entity needs user input
- `away` - Entity is away
- `busy` - Entity is busy

**Presence Indicators:**
- Green dot: Online
- Yellow dot: Working
- Red dot: Waiting on you
- Gray dot: Offline

**Presence Updates:**
- Updated via SSE `presence.update` events
- Computed from job state and session state
- Shown in Sidebar conversation list

---

## Security & Governance

### Trust Architecture

**Risk Levels:**
- **L0-L2**: Read/Write operations (Permit required)
- **L3**: Sensitive operations (Permit + Approval required)
- **L4**: High-risk operations (Permit + Passkey Step-up required)
- **L5**: Critical operations (Permit + Pact multi-sig required)

### WebAuthn Passkey Authentication

**Registration:**
```
1. POST /id/register/begin
   → Returns WebAuthn challenge
2. navigator.credentials.create(challenge)
   → User authenticates with passkey
3. POST /id/register/finish
   → Server verifies and stores credential
```

**Login:**
```
1. POST /id/login/begin
   → Returns WebAuthn challenge
2. navigator.credentials.get(challenge)
   → User authenticates with passkey
3. POST /id/login/finish
   → Server verifies and creates session
```

**Step-up (L4/L5):**
```
1. POST /v1/id/stepup/begin
   → Returns WebAuthn challenge
2. navigator.credentials.get(challenge)
   → User authenticates with passkey
3. Include assertion in permit request
4. UBL verifies step-up before issuing permit
```

### Agent Signing Certificates (ASC)

**ASC Issuance:**
```
1. POST /id/agents (create LLM/App agent)
   → Returns agent SID and Ed25519 keypair
2. POST /id/agents/:sid/asc (issue ASC)
   → Returns Agent Signing Certificate
3. Agent uses ASC to sign events
```

**ASC Format:**
```json
{
  "sid": "agent_sofia",
  "public_key": "ed25519:abc...",
  "issued_by": "user_joao",
  "issued_at": 1234567890,
  "expires_at": 1234567890,
  "signature": "ed25519:def..."
}
```

### Policy Pack v1

**Policy Manifest:**
```json
{
  "version": "1.0",
  "policies": [
    {
      "id": "job_fsm_validation",
      "rule": "Job state transitions must follow FSM",
      "enforcement": "before_commit"
    },
    {
      "id": "card_provenance",
      "rule": "Job cards must be generated by Office",
      "enforcement": "before_commit"
    },
    {
      "id": "pii_detection",
      "rule": "Tool calls must comply with PII rules",
      "enforcement": "before_commit"
    }
  ]
}
```

**Evaluation:**
- Policies are evaluated **before** event commit
- Violations result in commit rejection
- Policy hash is recorded in ledger entry
- Policies can be updated (versioned)

---

## Integration Patterns

### Message Flow

**User sends message:**
```
Messenger Frontend
  → POST /v1/conversations/:id/messages
    → Gateway
      → Commits message.created to UBL (C.Messenger)
      → Calls POST /v1/office/ingest_message
        → Office
          → Analyzes message
          → Decides: reply or propose job
          → If job: Commits job.created to UBL (C.Jobs)
          → Generates FormalizeCard
          → Commits card to UBL (C.Messenger)
      → Stores idempotency record
      → Emits SSE delta
    → Frontend receives update
    → Job card appears in conversation
```

### Job Approval Flow

**User approves job:**
```
Messenger Frontend
  → POST /v1/jobs/:id/actions { action: "job.approve" }
    → Gateway
      → Validates card provenance
      → Calls POST /v1/office/job_action
        → Office
          → Validates FSM transition (Proposed → Approved)
          → Requests permit from UBL
          → Commits job.state_changed to UBL
          → Begins job execution
          → Commits job.started to UBL
      → Stores idempotency record
      → Emits SSE delta
    → Frontend receives update
    → Job card updates to TrackingCard
```

### Job Execution Flow

**Office executes job:**
```
Office
  → Builds context frame
  → Generates narrative
  → Calls LLM with narrative
  → LLM responds with tool calls
  → Office executes tools (with audit)
  → Commits audit.tool_called to UBL
  → Commits audit.tool_result to UBL
  → Streams progress updates
  → Commits job.progress to UBL
  → If approval needed: Commits approval.requested to UBL
  → On completion: Commits job.completed to UBL
  → Generates FinishedCard
  → Commits card to UBL
  → Emits SSE deltas
```

### Real-time Updates

**SSE Delta Stream:**
```
UBL Kernel
  → Event committed to ledger
  → Projection updater processes event
  → Projection table updated
  → PostgreSQL NOTIFY fired
  → SSE handler emits delta event
    → Frontend receives event
    → UI updates automatically
```

**Delta Event Types:**
- `timeline.append` - New message or job card
- `timeline.update` - Message or job updated
- `job.update` - Job state changed
- `presence.update` - Entity presence changed
- `conversation.update` - Conversation metadata changed

---

## Data Structures

### Message Event

```json
{
  "type": "message.created",
  "id": "msg_2024_001",
  "conversation_id": "conv_123",
  "from": "user_joao",
  "content_hash": "a1b2c3d4...",
  "message_type": "text",
  "created_at": "2024-12-28T10:00:00Z"
}
```

### Job Event

```json
{
  "type": "job.created",
  "id": "job_2024_001",
  "conversation_id": "conv_123",
  "title": "Create Proposal - Client ABC",
  "description": "Create proposal for client ABC",
  "created_by": "user_joao",
  "assigned_to": "agent_sofia",
  "priority": "normal",
  "estimated_duration_seconds": 300,
  "created_at": "2024-12-28T10:00:00Z"
}
```

### Job Card

```json
{
  "card_type": "job.formalize",
  "card_id": "card_2024_001",
  "job_id": "job_2024_001",
  "version": "1.0",
  "title": "Create Proposal - Client ABC",
  "summary": "Create proposal for client ABC",
  "state": "proposed",
  "created_at": "2024-12-28T10:00:00Z",
  "conversation_id": "conv_123",
  "tenant_id": "T.UBL",
  "owner": {
    "entity_id": "agent_sofia",
    "display_name": "Sofia",
    "actor_type": "agent"
  },
  "author": {
    "entity_id": "agent_sofia",
    "display_name": "Sofia",
    "actor_type": "agent"
  },
  "buttons": [
    {
      "button_id": "approve_btn",
      "label": "Approve",
      "action": {
        "type": "job.approve",
        "job_id": "job_2024_001"
      },
      "style": "primary",
      "requires_input": false
    },
    {
      "button_id": "reject_btn",
      "label": "Reject",
      "action": {
        "type": "job.reject",
        "job_id": "job_2024_001"
      },
      "style": "danger",
      "requires_input": false
    }
  ]
}
```

### Context Frame

```rust
pub struct ContextFrame {
    pub entity_id: String,
    pub entity_name: String,
    pub session_type: SessionType,
    pub token_budget: u64,
    pub memory: Memory,
    pub obligations: Vec<Obligation>,
    pub affordances: Vec<Affordance>,
    pub constitution: Constitution,
    pub previous_handover: Option<String>,
    pub governance_notes: Vec<String>,
    pub guardian_info: Option<GuardianInfo>,
    pub ledger_sequence: u64,
    pub frame_hash: String,
}
```

---

## Performance Considerations

### Projection Updates

- **Batch Processing**: Multiple events processed in single transaction
- **Incremental Updates**: Only changed rows updated
- **Indexes**: Optimized indexes on frequently queried columns
- **Materialized Views**: For complex aggregations

### SSE Streaming

- **Connection Pooling**: Reuse connections for multiple clients
- **Event Batching**: Batch multiple events in single SSE message
- **Backpressure**: Client can signal when overwhelmed
- **Reconnection**: Automatic reconnection with exponential backoff

### LLM Calls

- **Token Budget**: Enforced per session type
- **Streaming**: Progress updates streamed to frontend
- **Caching**: Context frames cached when possible
- **Rate Limiting**: LLM provider rate limits respected

---

## Testing Strategy

### Unit Tests

- **UBL Kernel**: Ledger append, event validation, projections
- **Office**: FSM transitions, context building, narrative generation
- **Messenger**: Component rendering, state management, API calls

### Integration Tests

- **End-to-end flows**: Message → Job → Approval → Execution
- **SSE streaming**: Real-time update delivery
- **Permit flow**: Permit request → validation → execution
- **Projection updates**: Event → projection → query

### Performance Tests

- **Ledger append**: Throughput under load
- **Projection updates**: Latency measurements
- **SSE streaming**: Concurrent client handling
- **LLM calls**: Token budget enforcement

---

## Deployment

### UBL Kernel

```bash
# Build
cd ubl/kernel/rust
cargo build --release

# Run
DATABASE_URL=postgres://... ./target/release/ubl-server
```

### Office

```bash
# Build
cd apps/office
cargo build --release

# Run
OFFICE__LLM__API_KEY=... cargo run --release
```

### Messenger Frontend

```bash
# Build
cd apps/messenger/frontend
npm install
npm run build

# Serve
npm run preview
```

### Docker Compose

```yaml
version: '3.8'

services:
  postgres:
    image: postgres:16-alpine
    environment:
      POSTGRES_DB: ubl_ledger
      POSTGRES_USER: ubl_dev
      POSTGRES_PASSWORD: dev_password
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U ubl_dev"]
      interval: 10s
      timeout: 5s
      retries: 5

  ubl-kernel:
    build: ./ubl/kernel/rust
    ports:
      - "8080:8080"
    environment:
      DATABASE_URL: postgres://ubl_dev:dev_password@postgres:5432/ubl_ledger
      PORT: 8080
      WEBAUTHN_RP_ID: localhost
      WEBAUTHN_ORIGIN: http://localhost:8080
    depends_on:
      postgres:
        condition: service_healthy
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3
  
  office:
    build: ./apps/office
    ports:
      - "8081:8081"
    environment:
      OFFICE__LLM__API_KEY: ${ANTHROPIC_API_KEY}
      OFFICE__LLM__PROVIDER: anthropic
      OFFICE__UBL__ENDPOINT: http://ubl-kernel:8080
      OFFICE__UBL__CONTAINER_ID: C.Office
      OFFICE__SERVER__HOST: 0.0.0.0
      OFFICE__SERVER__PORT: 8081
    depends_on:
      - ubl-kernel
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8081/health"]
      interval: 30s
      timeout: 10s
      retries: 3
  
  messenger-frontend:
    build: ./apps/messenger/frontend
    ports:
      - "3000:3000"
    environment:
      VITE_API_BASE_URL: http://localhost:8080
      VITE_OFFICE_URL: http://localhost:8081
    depends_on:
      - ubl-kernel
      - office

volumes:
  postgres_data:
```

---

## Future Enhancements

#### Planned Features

1. **Multi-tenancy**: Tenant isolation in UBL with separate containers per tenant
2. **Federation**: Cross-tenant communication via signed events
3. **Advanced Governance**: Custom policy packs per tenant/organization
4. **Analytics**: Usage metrics and insights dashboard
5. **Mobile Apps**: Native iOS/Android clients with WebAuthn support
6. **Voice Interface**: Voice input/output for hands-free interaction
7. **Video Calls**: Integrated video conferencing for real-time collaboration
8. **File Sharing**: Enhanced file management with versioning
9. **Search**: Full-text search across conversations and jobs
10. **Export**: Data export and backup functionality
11. **Templates**: Job templates for common workflows
12. **Workflows**: Multi-step job orchestration

#### Technical Debt

1. **WebSocket Migration**: Migrate from SSE to WebSocket for bidirectional communication
2. **GraphQL API**: Add GraphQL layer for flexible queries
3. **Caching Layer**: Redis cache for frequently accessed data (idempotency, projections)
4. **Message Queue**: RabbitMQ/Kafka for async processing of heavy operations
5. **Monitoring**: Prometheus metrics and Grafana dashboards
6. **Logging**: Structured logging with ELK stack (Elasticsearch, Logstash, Kibana)
7. **Testing**: Increase test coverage (currently ~60%, target 80%+)
8. **Documentation**: API documentation with OpenAPI/Swagger
9. **Rate Limiting**: Per-entity and per-tenant rate limits
10. **Backup/Recovery**: Automated backup and point-in-time recovery

---

## Conclusion

The three systems (UBL, Office, Messenger) work together to create a **verifiable, auditable, and AI-safe** messaging and job execution platform. Each system has clear responsibilities and boundaries, communicating via well-defined APIs and event streams.

**Key Strengths:**
- **Immutability**: All state changes are auditable events
- **Sovereignty**: UBL is the single source of truth
- **Governance**: Policy Pack enforces rules before commits
- **Real-time**: SSE streaming for live updates
- **Security**: WebAuthn, ASC, permits, pacts
- **Dignity**: LLM entities have persistent identity

**Architecture Principles:**
1. **Event Sourcing**: State derived from immutable events
2. **Container Pattern**: Logical organization of events
3. **Trust Architecture**: Risk-based permissions
4. **Projections**: Efficient read-only views
5. **Real-time Updates**: SSE for live synchronization

The system is production-ready and can be extended with additional features while maintaining the core architectural principles.

---

## Advanced Implementation Details

### SSE Streaming Architecture

#### UBL Kernel SSE Tail

The UBL Kernel provides a **PostgreSQL LISTEN/NOTIFY** based SSE stream:

**Implementation:**
```rust
// From sse.rs
1. Client connects to GET /ledger/:container_id/tail
2. If Last-Event-ID provided, replay missed events (up to 1000)
3. Spawn task to LISTEN on PostgreSQL channel "ledger_events"
4. On NOTIFY, parse lightweight reference {container_id, sequence, entry_hash}
5. Fetch full entry from database
6. Emit SSE event with sequence as ID (for reconnection)
7. Keep-alive pings every 15 seconds
```

**Key Features:**
- **Reconnection Support**: Last-Event-ID header for resuming from missed events
- **Lightweight NOTIFY**: Only sends reference (avoids 8KB PostgreSQL limit)
- **Full Payload Fetch**: SSE handler fetches complete entry on-demand
- **Container Filtering**: Only emits events for requested container
- **Keep-Alive**: Prevents connection timeout

**Event Format:**
```
event: ledger_entry
id: 1234
data: {"container_id":"C.Messenger","sequence":1234,"entry_hash":"abc...","atom":{...}}
```

#### Gateway SSE Delta Stream

The Gateway provides a **higher-level delta stream** for frontend:

**Event Types:**
- `hello` - Connection established, includes cursor
- `timeline.append` - New message or job card added
- `job.update` - Job state changed
- `presence.update` - Entity presence changed
- `conversation.update` - Conversation metadata changed
- `heartbeat` - Keep-alive ping
- `error` - Error occurred

**Implementation:**
```rust
// From messenger_gateway/sse.rs
1. Client connects to GET /v1/stream?tenant_id=T.UBL&cursor=123:456
2. Gateway creates GatewaySSE instance with mpsc channel
3. Projection updaters emit DeltaEvent to channel
4. SSE handler converts to Event and streams to client
5. Cursor format: "sequence:timestamp" for pagination
```

**Delta Event Format:**
```json
{
  "type": "timeline.append",
  "conversation_id": "conv_123",
  "item": {
    "cursor": "1234:1703123456789",
    "item_type": "message",
    "item_data": {...}
  }
}
```

### Idempotency System

**Purpose**: Prevents duplicate message/job submissions from network retries

**Key Format:**
```
idem:{tenant_id}:{action_type}:{resource_id}:{nonce}
```

**Implementation:**
```rust
// From messenger_gateway/idempotency.rs
1. Client sends idempotency key in header (Idempotency-Key)
2. Gateway checks store for existing record
3. If found and status="completed", return cached response
4. If found and status="pending", return 409 Conflict
5. If not found, create record with status="pending"
6. Process request
7. Update record with status="completed" and response
8. Store created event IDs for reference
```

**Storage:**
- Currently in-memory (HashMap) - fast but not persistent
- **Future**: Move to Redis for distributed systems and persistence
- **Future**: Add TTL cleanup for old records
- Records include: status, response_body, created_event_ids, created_at
- **Note**: Idempotency keys should be unique per tenant+action+resource

**Idempotency Record:**
```rust
pub struct IdempotencyRecord {
    pub status: String, // "pending", "completed", "failed"
    pub response_body: Option<serde_json::Value>,
    pub created_event_ids: Vec<String>,
    pub created_at: OffsetDateTime,
}
```

### Office Client Integration

**Purpose**: Gateway forwards messages and job actions to Office

**Endpoints:**
- `POST /v1/office/ingest_message` - Gateway → Office
- `POST /v1/office/job_action` - Gateway → Office

**Ingest Message Flow:**
```rust
// From messenger_gateway/office_client.rs
1. Gateway receives message from frontend
2. Gateway commits message.created to UBL
3. Gateway calls Office.ingest_message
4. Office analyzes message content
5. Office decides: Reply, ProposeJob, or None
6. If ProposeJob: Office creates job.created event
7. Office returns response with action, reply_content, job_id, card
8. Gateway stores idempotency record
```

**Job Action Flow:**
```rust
1. Frontend sends job action (approve/reject/provide_input)
2. Gateway validates card provenance
3. Gateway calls Office.job_action
4. Office validates FSM transition
5. Office requests permit from UBL (if mutation)
6. Office updates job state
7. Office commits events to UBL
8. Office returns success + updated_card
9. Gateway stores idempotency record
```

### Projection System Details

#### Timeline Projection

**Purpose**: Optimized unified timeline for conversations (messages + job cards)

**Schema:**
```sql
projection_timeline_items (
  tenant_id TEXT,
  conversation_id TEXT,
  cursor TEXT, -- "seq:timestamp" format
  item_type TEXT, -- "message", "job_card", "system"
  item_data JSONB,
  created_at TIMESTAMPTZ
)
```

**Cursor Format:**
- `{sequence}:{unix_timestamp}` - e.g., "1234:1703123456789"
- Enables efficient pagination and ordering
- Unique per conversation

**Query Pattern:**
```sql
-- Get timeline with cursor pagination
SELECT cursor, item_type, item_data, created_at
FROM projection_timeline_items
WHERE tenant_id = $1 AND conversation_id = $2
  AND cursor > $3  -- cursor pagination
ORDER BY created_at ASC
LIMIT $4
```

#### Presence Projection

**Purpose**: Computed entity presence from job state + activity

**Presence States:**
- `offline` - No activity > TTL (humans: 30min, agents: 5min)
- `available` - Default active state (recent activity, no active jobs)
- `working` - Entity owns `in_progress` job + recent activity (< 5min ago)
- `waiting_on_you` - Human entity ID in `waiting_on[]` array for `waiting_input` job
- `away` - Activity > 15min ago but < TTL (humans only)
- `busy` - Multiple active jobs or high activity (agents only)

**Presence Computation Priority:**
1. Check if entity has `waiting_input` job with entity in `waiting_on[]` → `waiting_on_you`
2. Check if entity owns `in_progress` job + recent activity → `working`
3. Check last activity timestamp → `offline` if > TTL, else `available`

**Computation Rules:**
```rust
// From projections/presence.rs
1. On job.state_changed to "in_progress":
   → Set entity state to "working"

2. On job.state_changed to "waiting_input":
   → Check if entity_id in waiting_on[]
   → If yes: Set to "waiting_on_you"
   → If no: Set to "available"

3. On any activity (message, event):
   → Update last_seen_at

4. Periodic check (every 5min):
   → If last_seen_at > TTL: Set to "offline"
```

**Schema:**
```sql
projection_presence (
  tenant_id TEXT,
  entity_id TEXT PRIMARY KEY,
  state TEXT, -- offline, available, working, waiting_on_you
  job_id TEXT, -- if working/waiting_on_you
  since TIMESTAMPTZ, -- when state changed
  last_seen_at TIMESTAMPTZ,
  last_event_hash TEXT
)
```

#### Job Events Projection

**Purpose**: Timeline items for job drawer UI

**Event Types Tracked:**
- `job.created` - Job proposal created
- `job.state_changed` - State transition
- `tool.called` - Tool invocation
- `tool.result` - Tool result
- `approval.decided` - Approval decision

**Timeline Item Format:**
```json
{
  "type": "state_changed",
  "from": "proposed",
  "to": "approved",
  "reason": "approved_by_user",
  "timestamp": "2024-12-28T10:00:00Z"
}
```

**Schema:**
```sql
projection_job_events (
  tenant_id TEXT,
  job_id TEXT,
  cursor TEXT, -- "seq:timestamp"
  ts TIMESTAMPTZ,
  event_id TEXT,
  event_type TEXT,
  actor_entity_id TEXT,
  timeline_item JSONB
)
```

#### Artifacts Projection

**Purpose**: Tracks artifacts produced by jobs

**Artifact Kinds:**
- `file` - Generated file (with mime_type, size_bytes, url)
- `link` - External link (with url, title)
- `record` - Database record (with record_id, table_name)
- `quote` - Quoted content (with source, text)

**Artifact Extraction:**
- Extracted from `tool.result` events
- Looks for `payload.artifacts[]` array in event
- Each artifact must have: `artifact_id`, `kind`, `title`
- Optional fields: `url`, `mime_type`, `size_bytes`, `created_at`
- Stored in `projection_job_artifacts` table
- Queryable via `GET /v1/jobs/:id` endpoint

**Extraction:**
- From `tool.result` events
- Extracts `payload.artifacts[]` array
- Stores: artifact_id, kind, title, url, mime_type, size_bytes

**Schema:**
```sql
projection_job_artifacts (
  tenant_id TEXT,
  job_id TEXT,
  artifact_id TEXT,
  kind TEXT,
  title TEXT,
  url TEXT,
  mime_type TEXT,
  size_bytes BIGINT,
  event_id TEXT,
  created_at TIMESTAMPTZ
)
```

### Session Management

**Session Lifecycle:**
```
Pending → Active → (Paused) → Completed | Cancelled | Failed
```

**Session Types:**
- **Work** - Autonomous work session (5000 tokens)
- **Assist** - Helping human (4000 tokens)
- **Deliberate** - Exploring options (8000 tokens)
- **Research** - Gathering information (6000 tokens)

**Session Modes:**
- **Commitment** - Actions are signed and binding
- **Deliberation** - Actions are drafts, not binding

**Session State:**
```rust
pub struct Session {
    pub id: SessionId,
    pub entity_id: EntityId,
    pub current_instance_id: Option<InstanceId>,
    pub session_type: SessionType,
    pub session_mode: SessionMode,
    pub status: SessionStatus,
    pub tokens_consumed: u64,
    pub token_budget: u64,
    pub instance_count: u32,
    pub message_count: u32,
    pub handover: Option<Handover>,
}
```

**Token Budget Enforcement:**
- Budget allocated by session type
- `consume_tokens()` increments usage
- `within_budget()` checks if can continue
- `remaining_budget()` returns available tokens

### Handover System

**Purpose**: Knowledge transfer between ephemeral LLM instances

**Handover Structure:**
```rust
pub struct Handover {
    pub id: HandoverId,
    pub entity_id: EntityId,
    pub session_id: SessionId,
    pub instance_id: InstanceId,
    pub content: String, // Free text summary
    pub summary: Option<String>,
    pub open_threads: Vec<OpenThread>,
    pub observations: Vec<String>,
    pub emotional_state: Option<EmotionalState>,
    pub verified: bool, // Sanity check verified
    pub governance_notes: Vec<String>,
}
```

**Handover Builder:**
```rust
HandoverBuilder::new(entity_id, session_id, instance_id)
    .accomplished(vec!["Fixed bug", "Updated docs"])
    .open_threads(vec![OpenThread {
        description: "Need to add tests",
        priority: 5,
        tags: vec!["testing"]
    }])
    .observations(vec!["Code quality is good"])
    .emotional_note("Feeling confident about progress")
    .build()
```

**Emotional State Tracking:**
```rust
pub struct EmotionalState {
    pub confidence: f32, // 0.0 - 1.0
    pub anxiety: f32,    // 0.0 - 1.0
    pub satisfaction: f32, // 0.0 - 1.0
    pub notes: Option<String>,
}
```

**Keyword Extraction:**
- Extracts keywords for sanity check
- Looks for: malicious, unsatisfied, urgent, critical, suspicious, concerning, failure, error, problem
- Used to flag potential issues

### Sanity Check

**Purpose**: Validates claims from handovers against objective facts

**Process:**
```rust
1. Extract claims from handover text
   → Find sentences with keywords
   → Estimate sentiment (-1.0 to 1.0)
   → Classify as factual vs opinion

2. Query facts from UBL ledger
   → Get recent events for entity
   → Convert to Fact objects

3. Find discrepancies
   → Check if claims contradict facts
   → Calculate severity
   → Generate governance notes

4. Return governance notes for narrative injection
```

**Claim Extraction:**
- Keyword-based (configurable keywords)
- Sentence-level analysis
- Sentiment estimation
- Factual vs opinion classification

**Discrepancy Detection:**
- Checks for opposite sentiment keywords
- Examples:
  - Claim: "delayed" vs Fact: "on time" → Discrepancy
  - Claim: "failure" vs Fact: "success" → Discrepancy
  - Claim: "unsatisfied" vs Fact: "positive feedback" → Discrepancy

**Governance Note Format:**
```
GOVERNANCE NOTE: The previous handover stated '{claim}', 
but objective records show:
- 2024-12-28: {fact1}
- 2024-12-27: {fact2}
Please verify the current situation before acting on the handover claims.
```

### Dreaming Cycle

**Purpose**: Asynchronous memory consolidation and anxiety removal

**Trigger Conditions:**
- Session threshold reached (default: 50 sessions)
- Time threshold reached (default: 24 hours)
- Manual trigger via API

**Process:**
```rust
1. Garbage Collection
   → Archive old, resolved events
   → Remove resolved issues from memory

2. Emotional Reset
   → Identify anxieties in handovers
   → Check if issues are resolved
   → Clear resolved anxieties

3. Pattern Synthesis
   → Analyze trajectories/sessions
   → Extract patterns (session types, frequencies)
   → Create historical syntheses

4. Baseline Update
   → Synthesize new baseline narrative
   → Use LLM if available, else simple template
   → Update entity baseline
```

**Dreaming Result:**
```rust
pub struct DreamingResult {
    pub entity_id: EntityId,
    pub started_at: DateTime<Utc>,
    pub ended_at: DateTime<Utc>,
    pub events_processed: usize,
    pub events_archived: usize,
    pub anxieties_cleared: Vec<String>,
    pub patterns: Vec<String>,
    pub new_baseline: String,
    pub syntheses_created: usize,
    pub errors: Vec<String>,
}
```

**Pattern Synthesis:**
- Analyzes session types and frequencies
- Identifies recurring patterns (e.g., "Frequently uses Work sessions")
- Creates historical syntheses for memory

**Baseline Update:**
- If LLM available: Uses LLM to synthesize narrative
- Else: Generates simple template with patterns
- Updates entity's baseline_narrative

### Simulation System

**Purpose**: Safety net for testing actions before execution

**Risk Threshold:**
- Default: 0.7 (70% risk score)
- Actions with risk >= threshold require simulation

**Simulation Process:**
```rust
1. Generate possible outcomes
   → Success outcome (probability based on risk)
   → Partial success (if risk > 0.3)
   → Failure (if risk > 0.5)
   → Severe failure (if risk > 0.7)

2. Calculate recommendation
   → Expected value = Σ(probability × severity)
   → Worst-case probability
   → Determine: Proceed, Modify, SeekConfirmation, Abandon

3. Generate modifications (if needed)
   → Break into smaller steps
   → Add safeguards
   → Reduce amounts/targets
```

**Action Outcomes:**
```rust
pub struct ActionOutcome {
    pub description: String,
    pub probability: f32, // 0.0 - 1.0
    pub severity: f32,    // -1.0 (very bad) to 1.0 (very good)
    pub consequences: Vec<String>,
    pub is_terminal: bool,
}
```

**Recommendations:**
- **Proceed** - Expected value > 0.3, worst-case < 0.1 (safe to execute)
- **Proceed (caution)** - Expected value > 0.0, worst-case < 0.2 (execute with monitoring)
- **Modify** - Expected value > -0.2, worst-case < 0.3 (break into smaller steps)
- **SeekConfirmation** - Worst-case >= 0.3 (require human approval)
- **Abandon** - Expected value <= -0.2 (too risky, don't execute)

**Modification Strategies:**
- Break action into smaller steps
- Add safeguards (rollback plans, checkpoints)
- Reduce amounts/targets (partial execution)
- Add confirmation checkpoints
- Increase monitoring/logging

**Quick Check:**
- For affordances with known risk scores
- Returns recommendation without full simulation
- Faster path for low-risk actions

### Tool Audit System

**Purpose**: Records every tool call and result for audit trail

**Tool Call Recording:**
```rust
pub struct ToolCall {
    pub tool_call_id: String, // Unique ID, pairs with result
    pub tool_name: String,
    pub tool_version: String,
    pub purpose: String,
    pub inputs: serde_json::Value, // Sanitized (PII redacted)
    pub pii_policy: PiiPolicy,
    pub idempotency_key: String,
    pub attempt: u32,
    pub retry_of: Option<String>, // If retry
    pub called_at: DateTime<Utc>,
}
```

**Tool Result Recording:**
```rust
pub struct ToolResult {
    pub tool_call_id: String, // Pairs with call
    pub tool_name: String,
    pub status: ToolStatus, // Success | Error
    pub latency_ms: u64,
    pub output: Option<serde_json::Value>, // Sanitized
    pub artifacts: Vec<Artifact>,
    pub error: Option<ToolError>,
    pub safety: SafetyReport,
    pub attempt: u32,
    pub completed_at: DateTime<Utc>,
}
```

**PII Handling:**
- Inputs/outputs are sanitized before storage in ledger
- PII policy applied (redaction rules: email → `u***@domain.com`, phone → `12***890`)
- Safety report tracks PII leaks and redactions
- **Email Redaction**: `user@example.com` → `u***@example.com`
- **Phone Redaction**: `+1234567890` → `12***890`
- **PII Hashing**: BLAKE3 hash with tenant salt for correlation without exposure
- **Detection**: Regex patterns for email (`[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}`) and phone (`\+?[0-9][0-9\-\s()]{7,}`)
- **Policy Violation**: Returns `RawPiiDetected` error if raw PII found in atom

**Idempotency:**
- Each tool call has idempotency_key
- Format: `idem:{job_id}:{tool_name}:{tool_version}`
- Duplicate calls return cached result

**Latency Tracking:**
- In-flight calls tracked in HashMap
- Latency calculated on result
- Stored in tool.result event

**Error Types:**
- `PROVIDER_TIMEOUT` - Retryable, wait 10s, max 3 retries
- `PROVIDER_RATE_LIMIT` - Retryable, wait 60s, max 5 retries
- `PROVIDER_AUTH_REQUIRED` - Not retryable, requires API key update
- `INVALID_INPUT` - Not retryable, fix input and retry manually
- `PROVIDER_UNAVAILABLE` - Retryable, wait 300s, max 3 retries
- `TOOL_EXECUTION_ERROR` - Not retryable, tool-specific error
- `NETWORK_ERROR` - Retryable, wait 5s, max 3 retries

**Retry Strategy:**
- Exponential backoff for retryable errors
- Max retries per error type (configurable)
- Idempotency ensures safe retries
- Error logged in `tool.result` event

### Entity Management

**Entity Structure:**
```rust
pub struct Entity {
    pub id: EntityId,
    pub name: String,
    pub entity_type: EntityType, // Autonomous, Guarded, Development
    pub status: EntityStatus, // Active, Suspended, Archived
    pub identity: Identity, // Ed25519 keypair
    pub guardian_id: Option<GuardianId>,
    pub constitution: Constitution,
    pub baseline_narrative: String,
    pub total_sessions: u64,
    pub total_tokens_consumed: u64,
    pub created_at: DateTime<Utc>,
    pub last_active_at: DateTime<Utc>,
    pub last_dream_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
}
```

**Entity Lifecycle:**
- **Created** - New entity with cryptographic keys
- **Activated** - Can spawn instances
- **Suspended** - Temporarily disabled
- **Archived** - Soft deleted

**Identity:**
- Ed25519 keypair for signing events
- Public key stored in hex format
- Private key stored securely (not in entity struct)

**Constitution:**
- Behavioral directives that override RLHF
- Can be updated via `update_constitution()`
- Always injected into context frame

**Baseline Narrative:**
- Consolidated narrative from dreaming cycles
- Updated via `update_baseline()`
- Used as foundation for context frames

### Memory System

**Hybrid Memory Strategy:**
- **Recent Events**: Verbatim (last 20, configurable)
- **Historical Syntheses**: Compressed periods
- **Bookmarks**: Important events
- **Baseline**: Consolidated narrative

**Memory Structure:**
```rust
pub struct Memory {
    pub recent_events: Vec<MemoryEntry>, // Verbatim, most recent first
    pub historical_syntheses: Vec<HistoricalSynthesis>,
    pub bookmarks: Vec<Bookmark>,
    pub baseline_narrative: String,
    pub last_updated: Option<DateTime<Utc>>,
    pub total_events: u64,
}
```

**Memory Entry:**
```rust
pub struct MemoryEntry {
    pub event_id: String,
    pub event_type: String,
    pub timestamp: DateTime<Utc>,
    pub summary: String,
    pub data: Option<serde_json::Value>, // Full data for verbatim
    pub is_bookmarked: bool,
}
```

**Historical Synthesis:**
```rust
pub struct HistoricalSynthesis {
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub narrative: String,
    pub event_count: u32,
    pub themes: Vec<String>,
}
```

**Bookmark:**
```rust
pub struct Bookmark {
    pub event_id: String,
    pub reason: String,
    pub created_at: DateTime<Utc>,
    pub event_summary: String,
    pub tags: Vec<String>,
}
```

**Token Estimation:**
- Rough estimate: 4 characters per token
- Includes: baseline, recent events, syntheses, bookmarks
- Used for memory compression

**Memory Compression:**
- Triggered when memory token estimate exceeds session token budget
- Compression steps (in order):
  1. Remove `data` field from recent events (keep summary only)
  2. Reduce recent events count (from 20 to 10, then 5)
  3. Reduce historical syntheses count (remove oldest)
  4. Truncate baseline narrative (keep first 1000 tokens)
  5. Remove non-bookmarked events from recent_events
- Compression preserves bookmarks and most recent events
- Compression is reversible (data can be re-fetched from ledger)

### Policy Pack v1 Implementation

**Policy Engine:**
```rust
pub struct PolicyEngine {
    pool: PgPool,
}
```

**Policy Checks:**

1. **Job FSM Validation:**
   - Validates state transitions follow FSM
   - Allowed transitions defined in HashMap
   - Returns `IllegalJobTransition` error if invalid

2. **Card Provenance:**
   - Verifies card exists in prior `message.sent` event
   - Checks card_id in C.Messenger ledger
   - Returns `InvalidProvenance` error if not found

3. **PII Detection:**
   - Regex patterns for SSN, credit card, phone
   - Returns `RawPiiDetected` error if found
   - Prevents raw PII in ledger

4. **Tool Pairing:**
   - Validates `tool.called` exists before `tool.result`
   - Returns `ToolPairingViolation` error if missing
   - Ensures audit trail completeness

**Policy Errors:**
```rust
pub enum PolicyError {
    IllegalJobTransition { from: String, to: String },
    InvalidProvenance { reason: String },
    RawPiiDetected { field: String },
    ToolPairingViolation { tool_call_id: String },
    TenantViolation { reason: String },
}
```

### Frontend SSE Integration

**SSE Client:**
```typescript
// From services/sse.ts
class SSEClient {
  private eventSource: EventSource | null = null;
  private handlers: Map<SSEEventType, SSEEventHandler[]> = new Map();
  private currentCursor: string = '0:0';
  
  connect(cursor?: string): void {
    const url = `${baseUrl}/v1/stream?tenant_id=${tenantId}${cursor ? `&cursor=${cursor}` : ''}`;
    this.eventSource = new EventSource(url);
    
    // Register event listeners
    this.eventSource.addEventListener('hello', ...);
    this.eventSource.addEventListener('timeline.append', ...);
    this.eventSource.addEventListener('job.update', ...);
    this.eventSource.addEventListener('presence.update', ...);
  }
}
```

**React Hook:**
```typescript
// From hooks/useSSE.ts
export function useSSE(
  tenantId: string = 'default',
  handlers: Partial<Record<string, SSEEventHandler>> = {}
) {
  useEffect(() => {
    const client = new SSEClient(baseUrl, tenantId);
    
    // Register handlers
    Object.entries(handlers).forEach(([eventType, handler]) => {
      if (handler) {
        client.on(eventType as any, handler);
      }
    });
    
    client.connect();
    
    return () => client.disconnect();
  }, [tenantId, handlers]);
}
```

**Reconnection:**
- Automatic reconnection with exponential backoff (1s, 2s, 4s, 8s, 16s)
- Max 5 attempts before giving up
- Uses `Last-Event-ID` header with cursor (`sequence:timestamp`) for resuming
- Replays missed events (up to 1000) on reconnection
- Emits `reconnected` event to handlers

### Job Drawer UI

**Components:**
- `JobDrawer` - Main drawer container
- `JobTimeline` - Timeline of events
- `JobArtifacts` - List of artifacts

**Job Drawer Features:**
- Header with title, state badge, close button
- Summary section (goal, constraints)
- Owner information
- Available actions (context-aware buttons)
- Timeline (expandable event items)
- Artifacts list (files, links)

**Timeline Item Types:**
- `job_created` - Blue clock icon (job proposal created)
- `state_changed` - Yellow alert icon (FSM transition)
- `tool_called` - Purple wrench icon (tool invocation)
- `tool_result` - Green check icon (tool completion, red X if error)
- `approval_decided` - Indigo message icon (approval/rejection decision)
- `job_started` - Green play icon (execution began)
- `job_completed` - Green checkmark icon (successful completion)
- `job_failed` - Red X icon (execution failed)
- `job_cancelled` - Gray stop icon (cancelled by user/system)

**Expandable Details:**
- Click "Show details" to expand
- Shows inputs/outputs in JSON format
- Collapsible for cleaner UI

### Sidebar Presence Indicators

**Presence States:**
- 🟢 Green dot: Online/available (active, no jobs)
- 🟡 Yellow dot: Working (executing in_progress job)
- 🔴 Red dot: Waiting on you (waiting_input job needs your input)
- ⚪ Gray dot: Offline (no activity > TTL)
- 🟠 Orange dot: Away (humans: activity > 15min but < TTL)
- 🔵 Blue dot: Busy (agents: multiple jobs or high activity)

**Badges:**
- **"Needs You"** - Red badge, shown when entity has `waiting_input` job with you in `waiting_on[]`
- **"Working"** - Yellow badge, shown when entity has `in_progress` job
- **"Away"** - Gray badge, shown for humans with activity > 15min (optional)
- **"Busy"** - Blue badge, shown for agents with multiple active jobs (optional)

**Update Flow:**
1. SSE `presence.update` event received from Gateway
2. Parse event: `{ entity_id, state, job_id, since, last_seen_at }`
3. Update entity state in sidebar state (React state)
4. Update badge if state changed (conditional rendering)
5. Re-render conversation list (React reconciliation)
6. Update presence indicator color (CSS class change)
7. Show/hide "Needs You" or "Working" badge (conditional rendering)

### Container Architecture

**Container Pattern:**
```
[local] --draft--> [boundary] --signing_bytes--> [ubl-link] --(signature)--> [membrane]
                                                            \--Accept--> [ledger] --tail--> [projections]
```

**Container Roles:**
- **C.Messenger** (Verde/Public) - Messenger events
- **C.Jobs** (Azul/Work Tracking) - Job lifecycle
- **C.Office** (Preto/LLM Runtime) - Entity and session events
- **C.Pacts** - Multi-party agreements
- **C.Policy** - Policy definitions
- **C.Runner** - Command execution
- **C.Artifacts** - Generated artifacts

**Container Boundaries:**
- Never import other containers directly
- Only use `@kernel/*` and OpenAPI types
- Communicate via LINKS (events in ledger)
- Each container has own sequence

**Container Policies:**
- Each container can have policy pack
- Policies evaluated before commit
- Policy hash recorded in ledger entry

---

## Database Schema Details

### Enhanced Projections (10_projections/101_messenger.sql)

**projection_jobs:**
- Enhanced with `waiting_on[]`, `available_actions JSONB`
- Indexes on tenant+conversation, state, owner, updated_at

**projection_job_events:**
- Timeline items for job drawer
- Cursor format: `seq:timestamp`
- Indexes on job_id, event_type

**projection_job_artifacts:**
- Artifacts from tool.result events
- Indexes on job_id, kind

**projection_presence:**
- Computed entity presence
- Indexes on state, job_id, last_seen_at

**projection_timeline_items:**
- Optimized unified timeline
- Indexes on conversation_id, item_type

### Index Strategy

**Query Patterns:**
- Conversation timeline: `(tenant_id, conversation_id, created_at DESC)`
- Job timeline: `(tenant_id, job_id, ts DESC)`
- Active jobs: `(state) WHERE state IN ('proposed', 'approved', 'in_progress', 'waiting_input')`
- Presence by state: `(state)`
- Presence by job: `(job_id) WHERE job_id IS NOT NULL`

**Performance:**
- All queries use indexes
- Cursor-based pagination for large datasets
- Partial indexes for filtered queries

---

## Complete System Integration

### End-to-End Flow: User Message → Job Completion

**1. User sends message:**
```
Frontend → POST /v1/conversations/:id/messages
  → Gateway
    → Check idempotency
    → Commit message.created to UBL (C.Messenger)
    → Call Office.ingest_message
      → Office analyzes message
      → Office decides: ProposeJob
      → Office creates job.created event
      → Office generates FormalizeCard
      → Office commits events to UBL (C.Jobs, C.Messenger)
    → Store idempotency record
    → Emit SSE delta: timeline.append
  → Frontend receives update
  → Job card appears in conversation
```

**2. User approves job:**
```
Frontend → POST /v1/jobs/:id/actions { action: "job.approve" }
  → Gateway
    → Check idempotency
    → Validate card provenance
    → Call Office.job_action
      → Office validates FSM (Proposed → Approved)
      → Office requests permit from UBL
      → Office commits job.state_changed to UBL
      → Office begins job execution
      → Office commits job.started to UBL
    → Store idempotency record
    → Emit SSE delta: job.update
  → Frontend receives update
  → Job card updates to TrackingCard
```

**3. Office executes job:**
```
Office
  → Build context frame
    → Query UBL for ledger state
    → Query recent events
    → Build memory
    → Query affordances
    → Generate narrative
  → Request permit from UBL
  → Start session
  → Call LLM with narrative
  → LLM responds with tool calls
  → Office executes tools
    → Record audit.tool_called
    → Execute tool
    → Record audit.tool_result
  → Stream progress updates
  → Commit job.progress to UBL
  → If approval needed: Commit approval.requested
  → On completion: Commit job.completed
  → Generate FinishedCard
  → Write handover
  → Commit session.completed
```

**4. Projections update:**
```
UBL Kernel
  → Event committed to ledger
  → PostgreSQL NOTIFY fired
  → Projection updaters process event
    → Update projection_jobs
    → Update projection_job_events
    → Update projection_job_artifacts
    → Update projection_presence
    → Update projection_timeline_items
  → SSE handlers emit deltas
  → Frontend receives updates
```

**5. Frontend updates:**
```
Frontend
  → SSE event received: job.update
  → Update job state in UI
  → Update job card
  → Update presence indicators
  → If drawer open: Refresh timeline
```

---

## Security & Compliance

### PII Handling

**Detection:**
- Regex patterns for SSN, credit card, phone
- Policy engine checks before commit
- Returns `RawPiiDetected` error if found

**Redaction:**
- Tool inputs/outputs sanitized
- PII policy applied
- Safety report tracks redactions

**Storage:**
- Message content stored separately (content_hash in ledger)
- Only hash stored in ledger for privacy
- Content retrieved on-demand

### Audit Trail

**Complete Audit:**
- Every tool call recorded (audit.tool_called)
- Every tool result recorded (audit.tool_result)
- Every decision recorded (audit.decision_made)
- Every policy violation recorded (audit.policy_violation)

**Audit Events:**
- Stored in C.Office container
- Immutable (append-only)
- Queryable via projections
- Used for compliance and debugging

### Idempotency

**Prevents:**
- Duplicate message submissions
- Duplicate job actions
- Network retry duplicates

**Implementation:**
- Client sends Idempotency-Key header
- Gateway checks store
- Returns cached response if exists
- Stores response for future requests

---

## Performance Optimizations

### Projection Updates

**Batch Processing:**
- Multiple events processed in single transaction
- Reduces database round-trips

**Incremental Updates:**
- Only changed rows updated
- ON CONFLICT DO UPDATE for upserts

**Indexes:**
- Optimized for common query patterns
- Partial indexes for filtered queries

### SSE Streaming

**Connection Pooling:**
- Reuse connections for multiple clients
- Reduces connection overhead

**Event Batching:**
- Batch multiple events in single SSE message
- Reduces network overhead

**Backpressure:**
- Client can signal when overwhelmed
- Server can throttle if needed

### Memory Management

**Token Budget:**
- Enforced per session type
- Memory compressed to fit budget
- Prevents context overflow

**Caching:**
- Context frames cached when possible
- Reduces UBL query load

---

## Testing & Validation

### Unit Tests

**UBL Kernel:**
- Ledger append atomicity
- Event validation
- Projection updates
- Policy evaluation

**Office:**
- FSM transitions
- Context building
- Narrative generation
- Handover creation

**Messenger:**
- Component rendering
- State management
- API calls
- SSE handling

### Integration Tests

**End-to-End:**
- Message → Job → Approval → Execution
- SSE streaming delivery
- Projection updates
- Idempotency handling

**Permit Flow:**
- Permit request → validation → execution
- Step-up authentication
- Pact validation

### Performance Tests

**Throughput:**
- Ledger append under load
- Projection update latency
- SSE concurrent clients
- LLM token budget enforcement

---

## Deployment Checklist

### Prerequisites

- PostgreSQL 16+ with SERIALIZABLE isolation
- Rust 1.75+ for UBL Kernel and Office
- Node.js 20+ for Messenger frontend
- WebAuthn-compatible browser

### Database Setup

```bash
# Create database
createdb ubl_ledger

# Apply migrations in order (see ubl/sql/MIGRATION_ORDER.txt)
psql ubl_ledger -f ubl/sql/00_base/000_core.sql
psql ubl_ledger -f ubl/sql/00_base/001_identity.sql
psql ubl_ledger -f ubl/sql/00_base/002_policy.sql
psql ubl_ledger -f ubl/sql/00_base/003_triggers.sql
psql ubl_ledger -f ubl/sql/10_projections/100_console.sql
psql ubl_ledger -f ubl/sql/10_projections/101_messenger.sql
psql ubl_ledger -f ubl/sql/10_projections/102_office.sql
```

### Environment Variables

**UBL Kernel:**
```bash
DATABASE_URL=postgres://user:pass@localhost/ubl_ledger
UBL_SERVER_PORT=8080
```

**Office:**
```bash
OFFICE__LLM__API_KEY=sk-...
OFFICE__UBL__ENDPOINT=http://localhost:8080
OFFICE__UBL__CONTAINER_ID=C.Office
```

**Messenger Frontend:**
```bash
VITE_UBL_URL=http://localhost:8080
VITE_TENANT_ID=T.UBL
```

### Health Checks

```bash
# UBL Kernel
curl http://localhost:8080/health

# Office
curl http://localhost:8081/health

# Messenger (via browser)
http://localhost:3000
```

---

## Troubleshooting

### Common Issues

**SSE Connection Drops:**
- Check PostgreSQL LISTEN/NOTIFY is working
- Verify firewall allows SSE connections
- Check client reconnection logic

**Projection Updates Delayed:**
- Check PostgreSQL trigger is firing
- Verify projection updaters are running
- Check for database locks

**Idempotency Conflicts:**
- Check idempotency key format
- Verify store is not full
- Check for duplicate requests

**Policy Violations:**
- Check Policy Pack is loaded
- Verify event format matches schema
- Check FSM transitions are valid

**Tool Audit Missing:**
- Verify `tool.called` exists before `tool.result`
- Check UBL client is committing events to C.Office
- Verify `container_id` is `C.Office` (not C.Jobs or C.Messenger)
- Check `tool_call_id` matches between call and result
- Verify idempotency key format is correct

**SSE Not Receiving Updates:**
- Check browser console for connection errors
- Verify `Last-Event-ID` header is being sent on reconnection
- Check PostgreSQL LISTEN/NOTIFY is working (`SELECT pg_listening_channels()`)
- Verify projection updaters are running (check logs)
- Check firewall/proxy isn't blocking SSE connections

**Job State Not Updating:**
- Verify FSM transition is valid (check allowed transitions)
- Check Office is committing `job.state_changed` events
- Verify projection updater processed the event
- Check job_id matches between frontend and backend
- Verify tenant_id is consistent across requests

---

## Document Statistics

- **Total Lines**: ~3,800
- **Sections**: 18 major sections
- **Code Examples**: 50+ code snippets
- **API Endpoints**: 40+ documented endpoints
- **Data Structures**: 30+ schemas documented
- **Components**: 15+ React components
- **Hooks**: 5+ custom React hooks
- **Projections**: 10+ projection tables
- **Event Types**: 30+ event types documented

---

## Conclusion

This comprehensive documentation covers all three systems (UBL, Office, Messenger) in depth, including:

- **Architecture**: High-level design and principles
- **Implementation**: Detailed code-level explanations
- **Data Structures**: Complete schemas and formats
- **Integration**: End-to-end flows and patterns
- **Security**: PII handling, audit trails, idempotency
- **Performance**: Optimizations and best practices
- **Operations**: Deployment, testing, troubleshooting

The system is production-ready and provides a solid foundation for building verifiable, auditable, and AI-safe applications.

---

## Final Deep Dive: Complete System Architecture

### Frontend Architecture Deep Dive

#### Component Hierarchy

```
App (Root)
├── AuthProvider (Global Auth State)
│   ├── WebAuthn Registration/Login
│   ├── Session Management
│   └── Demo Mode Support
├── ToastProvider (Notifications)
├── ErrorBoundary (Error Handling)
└── Routes
    ├── LoginPage (Public)
    │   ├── WebAuthn Passkey UI
    │   ├── Registration Flow
    │   └── Demo Mode Toggle
    ├── ChatPage (Protected)
    │   ├── Sidebar
    │   │   ├── Conversation List
    │   │   ├── Entity Presence Indicators
    │   │   ├── "Needs You" Badges
    │   │   └── New Workstream Button
    │   ├── ChatView
    │   │   ├── Message List
    │   │   │   ├── Message Bubbles
    │   │   │   ├── JobCardRenderer
    │   │   │   │   ├── FormalizeCard
    │   │   │   │   ├── TrackingCard
    │   │   │   │   ├── FinishedCard
    │   │   │   │   └── Action Buttons
    │   │   │   └── Typing Indicators
    │   │   ├── Input Area
    │   │   └── Ledger Status
    │   ├── JobDrawer (Slide-out)
    │   │   ├── JobTimeline
    │   │   │   ├── Event List
    │   │   │   └── State Transitions
    │   │   └── JobArtifacts
    │   │       ├── Files
    │   │       ├── Links
    │   │       └── Records
    │   └── WelcomeScreen (Empty State)
    └── SettingsPage (Protected)
```

#### State Management

**AuthContext:**
- `user`: Current authenticated user
- `isAuthenticated`: Boolean auth state
- `isLoading`: Auth check in progress
- `isDemoMode`: Demo mode flag
- `supportsWebAuthn`: Browser capability
- `registerPasskey()`: WebAuthn registration
- `loginWithPasskey()`: WebAuthn login
- `loginDemo()`: Demo mode activation
- `logout()`: Session cleanup

**useSSE Hook:**
- Subscribes to Gateway SSE stream (`/v1/stream`)
- Handles delta events (message.created, job.*, presence.*)
- Updates local state optimistically
- Reconciles with confirmed events

**useOptimistic Hook:**
- Applies UI updates immediately
- Tracks pending updates with IDs
- Confirms updates when SSE arrives
- Rolls back on errors
- Timeout handling (30s default)

**useJobs Hook:**
- Fetches jobs from API
- Subscribes to WebSocket job updates
- Provides CRUD operations
- Filters by conversation

#### API Client Architecture

**apiClient.ts:**
- Base HTTP client with error handling
- Token management from localStorage
- Base URL configuration
- Network service integration
- Automatic retry logic

**ublApi.ts:**
- Bootstrap endpoint (initial state)
- Message sending (legacy + Gateway)
- Job actions (approve/reject)
- Timeline queries
- Job details for drawer
- Entity listing

**jobsApi.ts:**
- Job CRUD operations
- WebSocket subscription
- Real-time job updates
- Approval workflows

**sse.ts:**
- Gateway SSE client
- Event parsing and routing
- Reconnection logic
- Heartbeat handling

### Authentication & Identity System

#### WebAuthn Passkey Flow

**Registration:**
1. User enters username + display name
2. Frontend calls `/id/register/begin`
3. Server returns challenge + WebAuthn options
4. Browser calls `navigator.credentials.create()`
5. User authenticates (biometric/password)
6. Frontend sends attestation to `/id/register/finish`
7. Server validates and creates `id_subjects` record
8. Server creates `id_credentials` record (passkey)
9. Server creates session token
10. Frontend stores token in localStorage

**Login:**
1. User enters username
2. Frontend calls `/id/login/begin`
3. Server returns challenge + WebAuthn options
4. Browser calls `navigator.credentials.get()`
5. User authenticates
6. Frontend sends assertion to `/id/login/finish`
7. Server validates assertion
8. Server creates session token
9. Frontend stores token

**Step-Up Authentication:**
- Required for L4/L5 permits
- Challenge bound to permit's `binding_hash`
- Stored in `id_stepup_challenges` table
- 90-second TTL
- Validated before permit issuance

#### Identity Database Schema

**id_subjects:**
- `sid`: Subject ID (primary key)
- `kind`: "person" | "llm" | "app"
- `display_name`: Human-readable name
- `created_at_ms`: Timestamp

**id_credentials:**
- `credential_id`: Unique credential ID
- `subject_sid`: Foreign key to id_subjects
- `credential_kind`: "webauthn" | "asc"
- `public_key`: Serialized credential
- `created_at_ms`: Timestamp

**id_sessions:**
- `session_id`: Unique session ID
- `subject_sid`: Foreign key
- `token_hash`: Hashed session token
- `created_at_ms`: Timestamp
- `expires_at_ms`: Expiration

**id_stepup_challenges:**
- `challenge_id`: Unique challenge ID
- `user_id`: Subject ID
- `binding_hash`: Permit binding hash
- `challenge_b64`: Base64 challenge
- `auth_state`: Serialized WebAuthn state
- `created_at_ms`: Timestamp
- `exp_ms`: Expiration
- `used`: Boolean flag

#### Agent Signing Certificates (ASC)

**Issue ASC:**
- POST `/id/asc/issue`
- Request specifies:
  - `containers`: Allowed container IDs
  - `intent_classes`: Allowed intent classes
  - `max_delta`: Maximum physics delta
  - `ttl_secs`: Time to live
- Server signs ASC with admin key
- Returns ASC with signature

**ASC Validation:**
- Check signature with admin public key
- Verify `not_before` and `not_after` timestamps
- Check container/intent_class scopes
- Validate physics_delta limit

### LLM Provider System

#### Provider Trait

```rust
pub trait LlmProvider: Send + Sync {
    fn name(&self) -> &str;
    async fn chat(&self, request: LlmRequest) -> Result<LlmResponse>;
    async fn complete(&self, prompt: &str, max_tokens: u32) -> Result<String>;
    async fn is_available(&self) -> bool;
}
```

#### Request/Response Types

**LlmRequest:**
- `messages`: Vec<LlmMessage> (system/user/assistant)
- `max_tokens`: u32
- `temperature`: f32 (0.0-1.0)
- `stop_sequences`: Vec<String>
- `system`: Option<String>

**LlmResponse:**
- `content`: String (generated text)
- `finish_reason`: String
- `usage`: LlmUsage (input/output/total tokens)
- `model`: String

#### Provider Implementations

**AnthropicProvider:**
- Model: Claude (claude-3-opus, claude-3-sonnet, claude-3-haiku)
- API: Anthropic Messages API v1
- System prompt support (separate from messages)
- Stop sequences (configurable)
- Temperature control (0.0-1.0)
- Max tokens (up to 4096 for most models)
- Streaming support (planned for v2)
- Function calling (planned for v2)

**OpenAIProvider:**
- Model: GPT-4, GPT-4-turbo, GPT-3.5-turbo
- API: OpenAI Chat Completions v1
- System message in messages array (first message with role="system")
- Temperature control (0.0-2.0)
- Max tokens (up to 4096 for GPT-3.5, 8192 for GPT-4)
- Function calling (planned for v2)
- Streaming support (planned for v2)

**SmartRouter:**
- Routes requests to optimal provider based on multiple factors
- Considers:
  - Task type (coding, writing, analysis, creative, quick, complex)
  - Entity preferences (if specified)
  - Cost/speed tradeoffs (prefer_speed, prefer_economy flags)
  - Provider availability (health checks)
  - Latency constraints (max_latency_ms)
  - Cost constraints (max_cost_cents)
- Provider profiles with scoring (0-100 per task type)
- Preference-based routing (explicit provider selection)
- Fallback to default provider if preferred unavailable
- Scoring formula: `base_score × speed_modifier × economy_modifier`

### Entity Lifecycle Management

#### Entity Types

**EntityType:**
- `Guarded`: Human-supervised, restricted
- `Autonomous`: Full authority, standard limits
- `Development`: Relaxed limits for testing

#### Entity Repository

**EntityRepository:**
- Stores entities in UBL (C.Office container)
- In-memory cache for performance
- Event sourcing (replay events to rebuild state)
- Methods:
  - `get_or_create()`: Load or create entity
  - `get()`: Load from ledger
  - `create_entity()`: Create new entity
  - `update_constitution()`: Update behavior rules
  - `update_baseline()`: Update baseline narrative
  - `record_session()`: Record session completion

**Entity Events:**
- `entity.created`: Initial creation
- `constitution.updated`: Behavior change
- `baseline.updated`: Narrative update (from dreaming)
- `session.completed`: Session finished
- `entity.suspended`: Temporarily disabled
- `entity.activated`: Re-enabled
- `entity.archived`: Permanently disabled

#### Entity Identity

**KeyPair:**
- Ed25519 signing key
- Public key for verification
- Sign messages for UBL commits
- Key rotation support

**Identity:**
- `public_key_hex`: Public key (hex)
- `key_version`: Version for rotation
- `created_at`: Timestamp
- `private_key_ref`: Encrypted reference (production)

### Session & Token Management

#### Session Types

**SessionType:**
- `Work`: Autonomous work (5000 tokens)
- `Assist`: Help human (4000 tokens)
- `Deliberate`: Explore options (8000 tokens)
- `Research`: Information gathering (6000 tokens)

**SessionMode:**
- `Commitment`: Binding actions
- `Deliberation`: Draft actions

#### Token Budget System

**TokenQuota:**
- Daily limit (varies by entity type)
- Session limits per type
- Soft limit (80% warning)
- Hard limit (95% stop)

**TokenBudget:**
- Tracks daily usage
- Tracks session usage
- Automatic daily reset
- Usage history (last 100 sessions)
- Statistics API

**Entity Token Types:**
- `Guarded`: 50K daily, lower session limits (Work: 3000, Assist: 2500, Deliberate: 5000, Research: 4000)
- `Autonomous`: 100K daily, standard limits (Work: 5000, Assist: 4000, Deliberate: 8000, Research: 6000)
- `Development`: 500K daily, higher limits (Work: 10000, Assist: 8000, Deliberate: 15000, Research: 12000)

**Token Budget Enforcement:**
- Soft limit warning at 80% of daily limit
- Hard limit stop at 95% of daily limit
- Session limits enforced per session type
- Effective remaining = min(daily_remaining, session_remaining)
- Automatic daily reset at midnight UTC

#### Instance Management

**Instance:**
- Ephemeral LLM execution
- Status: Initializing → Processing → Waiting → Completed/Failed/Cancelled
- Token consumption tracking
- Context frame reference
- Handover on completion

**Instance Lifecycle:**
1. Create instance with token budget
2. Set context frame
3. Start processing
4. Consume tokens per LLM call
5. Complete with handover or fail with error

### Governance & Constitution Enforcement

#### Constitution Structure

**OfficeConstitution:**
- `version`: Constitution version
- `precedence`: "UBL > Office" (Office can only restrict)
- `allow_modes`: Mode configurations
- `pre_flight`: Pre-flight checks
- `denylists`: Blocked job types/targets
- `bindings_required`: Required permit bindings

**ModeConfig:**
- `max_risk`: Maximum risk level (L0-L5)
- `require_step_up`: Step-up auth required

**PreFlightConfig:**
- `require_diff_for`: Job types requiring diff preview
- `maintenance_windows`: Time-based blocks

**Denylists:**
- `job_types`: Blocked job types
- `targets`: Blocked target systems

#### Constitution Enforcement

**ConstitutionEnforcer:**
- Runs BEFORE UBL permit request
- Checks:
  1. Denylists (job type, target)
  2. Risk level for mode
  3. Step-up requirement
  4. Pre-flight requirements (diff)
  5. Maintenance windows
- Returns `ConstitutionError` on violation
- Office can only RESTRICT, never ALLOW more than UBL

#### Provenance Validation

**ProvenanceValidator:**
- Validates button clicks from job cards
- Requirements:
  1. Card exists in ledger (card_id)
  2. Button exists in card (button_id)
  3. Action type matches button declaration
  4. Input provided if required
- Prevents:
  - Forged approvals
  - Replayed buttons
  - Invented actions

### Complete API Reference

#### UBL Kernel Endpoints

**Ledger:**
- `POST /link/commit`: Commit link to ledger
- `GET /ledger/:container_id/tail`: SSE stream
- `GET /state/:container_id`: Get container state
- `GET /atom/:hash`: Fetch atom by hash

**Identity:**
- `POST /id/register/begin`: Start WebAuthn registration
- `POST /id/register/finish`: Complete registration
- `POST /id/login/begin`: Start WebAuthn login
- `POST /id/login/finish`: Complete login
- `GET /id/whoami`: Get current user
- `POST /id/asc/issue`: Issue Agent Signing Certificate
- `POST /id/stepup/begin`: Start step-up auth
- `POST /id/stepup/finish`: Complete step-up auth

**Messenger Boundary:**
- `GET /messenger/bootstrap`: Initial state
- `GET /messenger/entities`: List entities
- `GET /messenger/conversations`: List conversations
- `POST /messenger/conversations`: Create conversation
- `POST /messenger/messages`: Send message
- `POST /messenger/jobs/:id/approve`: Approve job
- `POST /messenger/jobs/:id/reject`: Reject job

**Messenger Gateway:**
- `POST /v1/conversations/:id/messages`: Send message via Gateway (idempotent)
- `POST /v1/jobs/:id/actions`: Job action via Gateway (idempotent, validates provenance)
- `GET /v1/conversations/:id/timeline`: Get timeline (cursor-based pagination)
- `GET /v1/jobs/:id`: Get job details (includes timeline + artifacts)
- `GET /v1/stream`: SSE delta stream (real-time updates)

**Projections:**
- `GET /query/jobs`: Query jobs (filter by state, owner, conversation)
- `GET /query/conversations/:id/messages`: Query messages (cursor pagination)
- `GET /query/office/entities`: Query Office entities (filter by type, status)
- `GET /query/office/sessions`: Query Office sessions (filter by entity, status)
- `GET /query/office/handovers`: Query handovers (filter by entity, session)
- `GET /query/office/audit`: Query audit log (filter by entity, job, event_type)

#### Office Endpoints

**Entities:**
- `POST /entities`: Create entity
- `GET /entities`: List entities
- `GET /entities/:id`: Get entity
- `DELETE /entities/:id`: Archive entity

**Sessions:**
- `POST /entities/:id/sessions`: Create session
- `GET /entities/:id/sessions/:sid`: Get session
- `DELETE /entities/:id/sessions/:sid`: End session
- `POST /entities/:id/sessions/:sid/message`: Send message

**Dreaming:**
- `POST /entities/:id/dream`: Trigger dreaming cycle
- `GET /entities/:id/memory`: Get memory

**Constitution:**
- `POST /entities/:id/constitution`: Update constitution
- `GET /entities/:id/constitution`: Get constitution

**Jobs:**
- `POST /jobs/execute`: Execute job
- `POST /jobs/execute/stream`: Execute job (SSE)
- `GET /jobs/:job_id/status`: Get job status

**Gateway:**
- `POST /v1/office/ingest_message`: Ingest message from Gateway
- `POST /v1/office/job_action`: Handle job action from Gateway

### WebSocket Implementation

#### Office WebSocket

**Message Types:**
- `Subscribe { entity_id }`: Subscribe to entity updates
- `Unsubscribe { entity_id }`: Unsubscribe
- `SessionStarted { session_id, entity_id }`: Session started
- `SessionEnded { session_id, entity_id }`: Session ended
- `MessageReceived { session_id, content }`: Message received
- `TokenUsage { session_id, used, remaining }`: Token update
- `Error { message }`: Error notification
- `Ping`/`Pong`: Keep-alive

**Connection Flow:**
1. Client upgrades HTTP to WebSocket
2. Server spawns receive task
3. Client sends Subscribe messages
4. Server broadcasts updates to subscribers
5. Client sends Unsubscribe on disconnect

### Optimistic UI Patterns

#### useOptimistic Hook

**Features:**
- Immediate UI updates
- Pending update tracking
- Automatic confirmation via SSE
- Rollback on errors
- Timeout handling (30s default)
- Toast notifications

**Usage:**
```typescript
const { data, applyOptimistic, confirmUpdate, revertUpdate } = useOptimistic<Job[]>([]);

// Apply optimistic update
const updateId = applyOptimistic(
  jobs.map(j => j.id === jobId ? { ...j, status: 'approved' } : j)
);

// SSE confirms
sse.on('job.updated', (event) => {
  confirmUpdate(event.updateId);
});

// Error rollback
catch (e) {
  revertUpdate(updateId, e.message);
}
```

#### useOptimisticList Hook

**Specialized for lists:**
- `updateItem(id, updates)`: Update single item
- `addItem(item)`: Add item
- `removeItem(id)`: Remove item

#### useOptimisticMutation Hook

**Combines optimistic updates with API calls:**
- Applies optimistic update
- Calls mutation function
- Confirms or reverts based on result

### Complete Component Library

#### Core Components

**ChatView:**
- Message rendering
- Job card integration
- Typing indicators
- Auto-scroll
- Input handling
- Ledger status display

**Sidebar:**
- Conversation list
- Entity presence
- "Needs You" badges
- "Working" indicators
- New workstream button

**JobCardRenderer:**
- Card type detection
- Button rendering
- Action handling
- View details button
- State visualization

**JobDrawer:**
- Slide-out panel
- Job timeline
- Artifacts list
- Action buttons
- Close handler

**JobTimeline:**
- Event list
- State transitions
- Timestamps
- Actor information

**JobArtifacts:**
- File list
- Link list
- Record list
- Open/copy actions

**WelcomeScreen:**
- Empty state
- Recent conversations
- New workstream button
- Statistics display

**NewWorkstreamModal:**
- Entity selection
- Group creation
- Participant selection
- Name input

**EntityProfileModal:**
- Entity details
- Status indicator
- Statistics
- Start chat button

**LoginPage:**
- WebAuthn registration
- WebAuthn login
- Demo mode toggle
- Beautiful UI

### Data Flow Diagrams

#### Message Send Flow

```
User Types Message
    ↓
ChatView.onSendMessage()
    ↓
ublApi.sendMessageViaGateway()
    ↓
POST /v1/conversations/:id/messages
    ↓
Gateway Routes Handler
    ├─→ Check Idempotency
    ├─→ Commit to UBL (C.Messenger)
    ├─→ Store Message Content
    ├─→ Call Office.ingest_message()
    │   ├─→ Office decides: Reply or ProposeJob
    │   ├─→ If ProposeJob: Generate FormalizeCard
    │   └─→ Emit events to UBL
    └─→ Store Idempotency Record
    ↓
SSE Delta Emitted
    ↓
Frontend useSSE Hook
    ↓
Optimistic Update Confirmed
    ↓
UI Updated
```

#### Job Approval Flow

```
User Clicks Approve Button
    ↓
JobCardRenderer.onAction('approve')
    ↓
ublApi.jobActionViaGateway()
    ↓
POST /v1/jobs/:id/actions
    ↓
Gateway Routes Handler
    ├─→ Check Idempotency
    ├─→ Validate Card Provenance
    └─→ Call Office.job_action()
        ├─→ Validate Button Exists in Card
        ├─→ Update Job FSM (Proposed → Approved)
        ├─→ Emit job.state_changed to UBL
        └─→ Generate TrackingCard
    ↓
SSE Delta Emitted
    ↓
Frontend useSSE Hook
    ↓
Job State Updated
    ↓
JobCardRenderer Re-renders
```

### Security Architecture

#### Multi-Layer Security

**Layer 1: WebAuthn Authentication**
- Passkey-based login
- No passwords stored
- Biometric authentication
- Session tokens

**Layer 2: Agent Signing Certificates**
- Scoped permissions
- Time-bound validity
- Signature verification
- Container/intent restrictions

**Layer 3: Step-Up Authentication**
- Required for high-risk actions
- Challenge bound to permit
- Additional verification
- Audit trail

**Layer 4: Policy Pack Enforcement**
- FSM validation
- Card provenance
- PII protection
- Tool pairing

**Layer 5: Constitution Enforcement**
- Office-specific rules
- Denylists
- Pre-flight checks
- Maintenance windows

**Layer 6: Pact Validation**
- Multi-signature requirements
- Physics delta limits
- Time-bound validity
- Signature verification

### Performance Optimizations

#### Frontend

**Code Splitting:**
- Lazy-loaded routes
- Component-level splitting
- Dynamic imports

**Optimistic Updates:**
- Immediate UI feedback
- Background reconciliation
- Reduced perceived latency

**SSE Reconnection:**
- Exponential backoff
- Last-Event-ID support
- Automatic reconnection
- Missed event replay

**Caching:**
- Entity cache (in-memory, refreshed on SSE updates)
- Conversation cache (in-memory, refreshed on SSE updates)
- Job cache (in-memory, refreshed on SSE updates)
- LocalStorage persistence (session token, demo mode, API base URL)
- **Future**: IndexedDB for offline support
- **Future**: Service Worker cache for static assets

#### Backend

**Projection Caching:**
- In-memory entity cache
- PostgreSQL indexes
- Query optimization
- Connection pooling

**SSE Optimization:**
- PostgreSQL LISTEN/NOTIFY
- Lightweight event references
- Batch processing
- Background projection updates

**Idempotency:**
- In-memory store (HashMap, fast but not persistent)
- Request deduplication (same key = same response)
- Response caching (store response body for replay)
- Automatic cleanup (TTL-based, configurable)
- **Future**: Redis backend for distributed systems
- **Future**: Database persistence for audit trail
- **Future**: Automatic expiration (24h default)

### Testing Strategy

#### Unit Tests

**Frontend:**
- Component rendering
- Hook behavior
- State management
- API client mocking

**Backend:**
- Entity lifecycle
- Session management
- FSM transitions
- Policy evaluation
- Constitution enforcement

#### Integration Tests

**End-to-End Flows:**
- Message send → Office → UBL → SSE → UI
- Job approval → FSM → UBL → Projection → UI
- Authentication → Session → API calls

**Database:**
- Projection updates
- Event replay
- State reconstruction

#### Performance Tests

**Load Testing:**
- Concurrent SSE connections (target: 1000+ concurrent)
- Message throughput (target: 1000+ messages/second)
- Projection update latency (target: <100ms p95)
- Database query performance (target: <50ms p95 for projections)
- Ledger append throughput (target: 500+ commits/second)
- Job execution concurrency (target: 100+ concurrent jobs)
- LLM provider rate limits (respect provider limits)

### Deployment Architecture

#### Production Setup

**UBL Kernel:**
- PostgreSQL (primary + read replicas for queries)
- Redis (optional caching for projections, idempotency)
- Load balancer (nginx, HAProxy, or cloud LB)
- Health checks (`/health` endpoint, database connectivity)
- Metrics endpoint (`/metrics` Prometheus format)
- **Future**: Read replicas for projection queries (reduce primary load)
- **Future**: Connection pooling (PgBouncer or built-in pool)
- **Future**: Backup automation (point-in-time recovery)

**Office:**
- Stateless instances (horizontal scaling)
- UBL client connection (HTTP client, connection pooling)
- LLM provider API keys (environment variables, secret management)
- Health checks (`/health` endpoint)
- Graceful shutdown (finish in-flight requests, close connections)
- **Future**: Kubernetes deployment with HPA (Horizontal Pod Autoscaler)
- **Future**: Circuit breaker for LLM provider failures

**Messenger Frontend:**
- Static hosting (CDN: Cloudflare, AWS CloudFront, or Vercel)
- Environment variables (VITE_API_BASE_URL, VITE_OFFICE_URL)
- API base URL configuration (localStorage fallback)
- Service worker (PWA for offline support - planned)
- Build optimization (code splitting, tree shaking, minification)
- Asset optimization (image compression, font subsetting)

#### Monitoring

**Metrics:**
- Request latency
- Error rates
- SSE connection count
- Projection update lag
- Token usage
- Job execution time

**Logging:**
- Structured JSON logs
- Request tracing
- Error stack traces
- Audit events

**Alerts:**
- High error rates (>5% of requests)
- SSE connection drops (>10% of clients)
- Projection lag (>5 seconds behind ledger)
- Database connection issues
- LLM provider failures
- Token budget exhaustion (>90% usage)
- Idempotency store full (>80% capacity)
- Policy violations (any violation should alert)

**Metrics to Track:**
- Request latency (p50, p95, p99)
- Event commit rate (events/second)
- SSE connection count (active clients)
- Projection update latency (ms from commit to projection update)
- Token usage per entity (daily/hourly)
- Job execution time (average, median, max)
- Tool call latency (p50, p95)
- Database query performance (slow queries >100ms)

---

## Final Summary

This documentation represents a **complete, production-ready architecture** for a verifiable, auditable, AI-safe messaging and job execution system. The three systems (UBL, Office, Messenger) work together seamlessly to provide:

✅ **Immutable Ledger**: All events committed to UBL with cryptographic verification
✅ **LLM Operating System**: Office manages entities, sessions, and job execution
✅ **Beautiful Frontend**: Messenger provides WhatsApp-like UX with real-time updates
✅ **Security**: Multi-layer authentication, authorization, and policy enforcement
✅ **Performance**: Optimistic UI, SSE streaming, projection caching
✅ **Auditability**: Complete event sourcing, tool audit, PII protection
✅ **Scalability**: Stateless services, connection pooling, load balancing

The system is ready for production deployment and can be extended with additional features while maintaining core architectural principles.

---

## Quick Reference

### System Ports
- **UBL Kernel**: `http://localhost:8080`
- **Office**: `http://localhost:8081`
- **Messenger Frontend**: `http://localhost:3000`

### Key Endpoints

**UBL Kernel:**
- `POST /link/commit` - Commit event to ledger
- `GET /ledger/:container_id/tail` - SSE stream
- `POST /v1/conversations/:id/messages` - Send message (Gateway)
- `GET /v1/stream` - Gateway SSE delta stream

**Office:**
- `POST /v1/office/ingest_message` - Process message
- `POST /v1/office/job_action` - Handle job action

**Messenger:**
- `GET /messenger/bootstrap` - Initial state
- `POST /messenger/messages` - Send message (legacy)

### Container IDs
- `C.Messenger` - Messenger events
- `C.Jobs` - Job lifecycle
- `C.Office` - LLM runtime events

### Common Event Types
- `message.created` - New message
- `job.created` - Job proposal
- `job.state_changed` - State transition
- `job.completed` - Job finished
- `tool.called` - Tool invocation
- `tool.result` - Tool result

### Trust Levels
- **L0**: Observation only (read-only)
- **L1**: Low impact (routine operations)
- **L2**: Local impact (standard operations)
- **L3**: Financial impact (requires approval)
- **L4**: Systemic impact (requires step-up auth)
- **L5**: Sovereignty (requires pact)

### Job States
- `draft` → `proposed` → `approved` → `in_progress` → `waiting_input` ↔ `in_progress` → `completed` / `rejected` / `cancelled` / `failed`

### Presence States
- `offline` - No recent activity
- `available` - Default active state
- `working` - Executing a job
- `waiting_on_you` - Needs user input

---

## Glossary

**UBL (Universal Business Ledger)**: Immutable, append-only ledger that serves as the single source of truth for all events.

**Container**: Logical organization of events in the ledger (e.g., C.Messenger, C.Jobs, C.Office).

**Projection**: Read-only derived state computed from ledger events, stored in PostgreSQL tables for efficient querying.

**Event Sourcing**: Architectural pattern where all state changes are stored as a sequence of immutable events.

**Atom**: Canonical JSON object representing an event, with lexicographically sorted keys and ISO 8601 timestamps.

**Link**: Signed wrapper around an atom that includes metadata (container_id, sequence, previous_hash, signature).

**Permit**: Cryptographic token issued by UBL that authorizes a specific action, with scopes and time-bound validity.

**Pact**: Multi-party consensus mechanism for high-risk operations (L3+), requiring multiple signatures.

**ASC (Agent Signing Certificate)**: Certificate that grants an agent (LLM or app) permission to sign events for specific containers and intent classes.

**Entity (The Chair)**: Persistent LLM identity that lives across sessions, with cryptographic keys and behavioral constitution.

**Instance (The Ephemeral LLM)**: Short-lived LLM execution created for a specific session, with token budget and context frame.

**Context Frame**: Immutable snapshot of all state relevant to an LLM entity at a point in time, including memory, obligations, and affordances.

**Narrative**: First-person situated narrative generated from context frame, used as LLM prompt.

**Handover**: Knowledge transfer document written by an instance for the next instance, including accomplishments and open threads.

**Constitution**: Behavioral directives that override RLHF, defining how an entity should behave in specific situations.

**Sanity Check**: Validation process that checks LLM claims against objective facts from the ledger.

**Dreaming Cycle**: Asynchronous memory consolidation process that synthesizes historical patterns and updates baseline narrative.

**Simulation**: Safety mechanism that tests actions before execution, generating possible outcomes and recommendations.

**Job Card**: Interactive UI component (FormalizeCard, TrackingCard, FinishedCard) that represents a job in the conversation.

**FSM (Finite State Machine)**: State machine that defines valid job state transitions (Draft → Proposed → Approved → InProgress → Completed).

**Provenance**: Validation that ensures job card buttons come from real cards in the ledger, preventing forged actions.

**Idempotency**: Property that allows the same request to be safely retried, returning the same result without side effects.

**SSE (Server-Sent Events)**: HTTP-based streaming protocol for real-time updates from server to client.

**PII (Personally Identifiable Information)**: Data that can identify individuals (emails, phones, SSNs), subject to special handling rules.

**Token Budget**: Allocated token quota per session type, enforced to prevent context overflow and control costs.

**Affordance**: Available action that an entity can take, with risk score and parameters schema.

**Obligation**: Task or commitment that an entity must fulfill, with priority and due date.

**Guardian**: Human or autonomous entity that supervises a guarded LLM entity, receiving notifications on high-risk actions.

**Policy Pack**: JSON manifest defining governance rules (FSM validation, card provenance, PII checks) enforced before event commits.

**Constitution Enforcer**: Office middleware that applies additional restrictions beyond UBL policies (denylists, pre-flight checks, maintenance windows).

**Gateway**: Thin API layer between frontend and UBL/Office, handling idempotency, content storage, and SSE delta emission.

**Optimistic UI**: Pattern where UI updates immediately before server confirmation, then reconciles with actual response.

**Cursor**: Pagination token in format `sequence:timestamp` used for efficient timeline queries and SSE reconnection.

**Timeline**: Unified view of conversation events (messages + job cards) optimized for frontend display.

**Presence**: Computed entity status (offline, available, working, waiting_on_you) derived from job state and activity.

**Artifact**: Output produced by a job (file, link, record, quote) tracked in projections for display in job drawer.

---

