# UBL 3.0 Architecture

**Three independent systems working together to realize the LogLine Foundation vision**

## Overview

UBL 3.0 consists of three separate, independently deployable systems that communicate via APIs, WebSockets, and event streams:

```
┌─────────────────────────────────────────────────────┐
│                  UBL MESSENGER                      │
│  WhatsApp UI + Cards + Humanos & Agentes           │
│  (Frontend Beautiful + Backend Smart)               │
└────────────┬────────────────────────────────────────┘
│ API/WS
↓
┌─────────────────────────────────────────────────────┐
│                       OFFICE                        │
│  LLM Runtime + Governança + Context Management     │
│  (Dignidade para entidades efêmeras)                │
└────────────┬────────────────────────────────────────┘
│ Ledger Events
↓
┌─────────────────────────────────────────────────────┐
│                       UBL                           │
│  Append-only + Containers + Trust Architecture     │
│  (Source of Truth Imutável)                         │
└─────────────────────────────────────────────────────┘
```

## System 1: UBL Messenger

**Location**: `apps/messenger/`

**Purpose**: User-facing WhatsApp-like interface for professional collaboration

**Components**:
- **Frontend** (`apps/messenger/frontend/`): React/TypeScript UI
- **Backend** (`apps/messenger/backend/`): Rust API server

**Key Features**:
- Conversations (direct and group)
- Messages with rich content
- Job cards (initiation, progress, completion, approval)
- Real-time updates via WebSocket
- Human and AI agent participants

**APIs**:
- HTTP REST API for CRUD operations
- WebSocket for real-time events
- Integrates with Office for LLM execution
- Publishes events to UBL

## System 2: Office

**Location**: `apps/office/`

**Purpose**: LLM Operating System - Runtime for LLM entities with dignity

**Components**:
- Entity management
- Session handling
- Context frame builder
- Narrator
- Governance (Sanity Check, Constitution, Dreaming Cycle)
- Simulation

**Key Features**:
- Context preparation before LLM invocation
- Narrative generation
- Session handovers
- Psychological governance
- Job execution engine
- Approval management

**APIs**:
- Entity CRUD
- Session management
- Job execution
- Approval workflows
- Context building
- Affordances discovery

## System 3: UBL

**Location**: `ubl/`

**Purpose**: Immutable event-sourced ledger - Single source of truth

**Components**:
- Kernel (Rust)
- Containers (C.Messenger, C.Office, C.Jobs, etc.)
- Trust architecture (L0-L5)
- Event streaming
- Cryptographic proofs

**Key Features**:
- Append-only ledger
- Container-based organization
- Trust levels and pacts
- Event sourcing
- Receipts and verification

**APIs**:
- Link commits
- Event queries
- State projections
- Affordances
- Receipt verification

## Communication Patterns

### UBL Messenger ↔ Office
- **HTTP**: Job execution requests, approval decisions
- **WebSocket**: Progress updates, real-time status

### UBL Messenger ↔ UBL
- **HTTP**: Event publishing, state queries
- **SSE**: Event subscriptions

### Office ↔ UBL
- **HTTP**: Event publishing, state queries, affordances
- **SSE**: Event subscriptions for context building

## Data Flow Example: Creating a Proposal

1. **UBL Messenger**: User creates job "Create Proposal for Client ABC"
2. **UBL Messenger → UBL**: Publishes `job.created` event
3. **UBL Messenger → Office**: Requests job execution
4. **Office**: Builds context frame, generates narrative
5. **Office → UBL**: Publishes `session.started` event
6. **Office**: LLM executes job, gathers data, calculates prices
7. **Office → UBL Messenger**: Sends progress updates via WebSocket
8. **Office**: Needs approval → creates approval request
9. **Office → UBL Messenger**: Sends approval card
10. **UBL Messenger**: User clicks approve
11. **UBL Messenger → UBL**: Publishes `approval.decided` event
12. **UBL Messenger → Office**: Notifies approval decision
13. **Office**: Resumes execution, completes job
14. **Office → UBL**: Publishes `job.completed` event
15. **Office → UBL Messenger**: Sends completion card with artifacts

## Deployment

Each system can be deployed independently:

```bash
# Deploy UBL
cd ubl
docker build -t ubl .
docker run -p 8080:8080 ubl

# Deploy Office
cd apps/office
docker build -t office .
docker run -p 8081:8081 office

# Deploy UBL Messenger
cd apps/messenger
# Frontend
cd frontend && npm run build && serve -s dist
# Backend
cd backend && cargo build --release && ./target/release/messenger
```

Or use Docker Compose (see `office/docker-compose.yml`)

## Development Setup

### Prerequisites
- Node.js 18+ (for Messenger frontend and temporary backend)
- Rust 1.70+ (for OFFICE and Messenger backend)
- Docker (for UBL Ledger)

### Running Locally

```bash
# Terminal 1: UBL Kernel
cd ubl/kernel/rust
DATABASE_URL="postgres://user@localhost/ubl_ledger" cargo run --bin ubl-server

# Terminal 2: Office
cd apps/office
cargo run

# Terminal 3: UBL Messenger Frontend
cd apps/messenger/frontend
npm run dev
```

## Directory Structure

```
OFFICE-main/
│
├── README.md                           # Project overview
├── THREE_SYSTEMS_OVERVIEW.md           # Complete 4000-line architecture doc
├── WIRING_GUIDE.md                     # Integration guide
├── CONTRIBUTING.md                     # Contribution guidelines
│
├── apps/
│   │
│   ├── messenger/                      # ══════ SYSTEM 1: MESSENGER ══════
│   │   │
│   │   └── frontend/                   # React/TypeScript UI (port 3000)
│   │       ├── package.json
│   │       ├── vite.config.ts
│   │       ├── tsconfig.json
│   │       │
│   │       └── src/
│   │           ├── App.tsx                     # Root component
│   │           ├── index.tsx                   # Entry point
│   │           ├── types.ts                    # TypeScript definitions
│   │           │
│   │           ├── components/
│   │           │   ├── ChatView.tsx            # Main chat interface
│   │           │   ├── Sidebar.tsx             # Conversation list + presence
│   │           │   ├── WelcomeScreen.tsx       # Empty state
│   │           │   ├── JobDrawer.tsx           # Slide-out job details
│   │           │   ├── JobTimeline.tsx         # Event timeline
│   │           │   ├── JobArtifacts.tsx        # Generated artifacts
│   │           │   ├── BridgeConfig.tsx        # API configuration
│   │           │   ├── ErrorBoundary.tsx       # Error handling
│   │           │   │
│   │           │   ├── cards/
│   │           │   │   └── JobCardRenderer.tsx # FormalizeCard, TrackingCard, FinishedCard
│   │           │   │
│   │           │   ├── modals/
│   │           │   │   ├── NewWorkstreamModal.tsx
│   │           │   │   └── EntityProfileModal.tsx
│   │           │   │
│   │           │   └── ui/                     # Design system
│   │           │       ├── Avatar.tsx
│   │           │       ├── Badge.tsx
│   │           │       ├── Button.tsx
│   │           │       ├── Input.tsx
│   │           │       ├── Modal.tsx
│   │           │       ├── Spinner.tsx
│   │           │       ├── HoldButton.tsx      # Long-press actions
│   │           │       ├── GhostCard.tsx       # Loading skeleton
│   │           │       ├── MessageStatus.tsx   # Delivery indicators
│   │           │       ├── SyncStatus.tsx      # Connection status
│   │           │       └── ThoughtStream.tsx   # LLM thinking display
│   │           │
│   │           ├── pages/
│   │           │   ├── LoginPage.tsx           # WebAuthn passkey auth
│   │           │   ├── ChatPage.tsx            # Main application
│   │           │   └── SettingsPage.tsx        # User preferences
│   │           │
│   │           ├── hooks/
│   │           │   ├── useAuth.ts              # Authentication state
│   │           │   ├── useSSE.ts               # Server-sent events
│   │           │   ├── useJobs.ts              # Job management
│   │           │   ├── useOptimistic.ts        # Optimistic UI updates
│   │           │   └── useAudioEngine.ts       # Sound effects
│   │           │
│   │           ├── services/
│   │           │   ├── apiClient.ts            # HTTP client
│   │           │   ├── ublApi.ts               # UBL Kernel API
│   │           │   ├── jobsApi.ts              # Job operations
│   │           │   ├── sse.ts                  # SSE client
│   │           │   ├── ledger.ts               # Ledger helpers
│   │           │   ├── network.ts              # Network status
│   │           │   └── eventBus.ts             # Internal events
│   │           │
│   │           ├── context/
│   │           │   ├── AuthContext.tsx         # Auth provider
│   │           │   ├── ThemeContext.tsx        # Dark/light mode
│   │           │   ├── NotificationContext.tsx # Toast notifications
│   │           │   ├── OnboardingContext.tsx   # First-run experience
│   │           │   └── ProtocolContext.tsx     # UBL protocol state
│   │           │
│   │           ├── lib/
│   │           │   ├── cn.ts                   # Tailwind class merger
│   │           │   └── toast.tsx               # Toast notifications
│   │           │
│   │           ├── observability/
│   │           │   ├── index.ts                # OpenTelemetry setup
│   │           │   ├── metrics.ts              # Frontend metrics
│   │           │   └── tracing.ts              # Distributed tracing
│   │           │
│   │           └── utils/
│   │               └── security.ts             # WebAuthn helpers
│   │
│   └── office/                         # ══════ SYSTEM 2: OFFICE ══════
│       │
│       ├── Cargo.toml                  # Rust dependencies
│       │
│       ├── config/
│       │   ├── development.toml        # Dev settings
│       │   └── production.toml         # Prod settings
│       │
│       └── src/
│           ├── main.rs                 # Entry point (port 8081)
│           ├── lib.rs                  # Library exports
│           ├── types.rs                # Core types
│           ├── asc.rs                  # Agent Signing Certificates
│           ├── http_unix.rs            # Unix socket support
│           │
│           ├── api/
│           │   ├── mod.rs
│           │   ├── http.rs             # HTTP routes
│           │   └── websocket.rs        # WebSocket handler
│           │
│           ├── routes/
│           │   ├── mod.rs
│           │   ├── deploy.rs           # Deployment routes
│           │   └── ws.rs               # WebSocket routes
│           │
│           ├── entity/                 # The Chair (permanent identity)
│           │   ├── mod.rs
│           │   ├── entity.rs           # Entity struct
│           │   ├── instance.rs         # Ephemeral LLM instance
│           │   ├── identity.rs         # Ed25519 keypair
│           │   ├── guardian.rs         # Human supervisor
│           │   └── repository.rs       # Entity storage
│           │
│           ├── session/                # LLM session management
│           │   ├── mod.rs
│           │   ├── session.rs          # Session lifecycle
│           │   ├── handover.rs         # Knowledge transfer
│           │   ├── modes.rs            # Commitment vs Deliberation
│           │   └── token_budget.rs     # Token quota enforcement
│           │
│           ├── context/                # Context frame building
│           │   ├── mod.rs
│           │   ├── builder.rs          # Frame construction
│           │   ├── frame.rs            # Immutable context snapshot
│           │   ├── memory.rs           # Hybrid memory system
│           │   └── narrator.rs         # Narrative generation
│           │
│           ├── governance/             # Behavioral governance
│           │   ├── mod.rs
│           │   ├── constitution.rs     # Behavioral directives
│           │   ├── sanity_check.rs     # Claim validation
│           │   ├── dreaming.rs         # Memory consolidation
│           │   ├── simulation.rs       # Action safety testing
│           │   └── provenance.rs       # Card button validation
│           │
│           ├── job_executor/           # Job execution engine
│           │   ├── mod.rs
│           │   ├── executor.rs         # Main executor
│           │   ├── fsm.rs              # State machine
│           │   ├── cards.rs            # Job card generation
│           │   ├── types.rs            # Job types
│           │   └── conversation_context.rs
│           │
│           ├── llm/                    # LLM provider system
│           │   ├── mod.rs
│           │   ├── provider.rs         # Provider trait
│           │   ├── anthropic.rs        # Claude integration
│           │   ├── openai.rs           # GPT integration
│           │   ├── local.rs            # Local models
│           │   └── router.rs           # Smart routing
│           │
│           ├── audit/                  # Tool audit system
│           │   ├── mod.rs
│           │   ├── tool_audit.rs       # Call/result recording
│           │   ├── events.rs           # Audit events
│           │   └── pii.rs              # PII detection/redaction
│           │
│           ├── middleware/
│           │   ├── mod.rs
│           │   ├── constitution.rs     # Constitution enforcement
│           │   └── permit.rs           # Permit validation
│           │
│           ├── ubl_client/             # UBL Kernel client
│           │   ├── mod.rs
│           │   ├── ledger.rs           # Ledger operations
│           │   ├── events.rs           # Event types
│           │   ├── identity_events.rs  # Identity events
│           │   ├── affordances.rs      # Available actions
│           │   ├── trust.rs            # Trust levels
│           │   └── receipts.rs         # Commit receipts
│           │
│           └── observability/
│               ├── mod.rs
│               ├── metrics.rs          # Prometheus metrics
│               └── tracing.rs          # OpenTelemetry
│
├── ubl/                                # ══════ SYSTEM 3: UBL KERNEL ══════
│   │
│   ├── README.md
│   │
│   ├── kernel/
│   │   │
│   │   ├── rust/                       # Rust implementation (port 8080)
│   │   │   ├── Cargo.toml              # Workspace manifest
│   │   │   │
│   │   │   ├── ubl-atom/               # Canonical JSON (JSON✯Atomic v1.0)
│   │   │   │   ├── Cargo.toml
│   │   │   │   └── src/
│   │   │   │       ├── lib.rs
│   │   │   │       ├── canonical.rs    # Canonicalization
│   │   │   │       └── hash.rs         # BLAKE3 atom_hash()
│   │   │   │
│   │   │   ├── ubl-link/               # Signed event wrapper
│   │   │   │   ├── Cargo.toml
│   │   │   │   └── src/
│   │   │   │       ├── lib.rs
│   │   │   │       └── signing.rs      # Ed25519 signatures
│   │   │   │
│   │   │   ├── ubl-ledger/             # Append-only ledger
│   │   │   │   ├── Cargo.toml
│   │   │   │   └── src/
│   │   │   │       ├── lib.rs
│   │   │   │       ├── entry.rs        # LedgerEntry
│   │   │   │       └── append.rs       # Atomic append
│   │   │   │
│   │   │   ├── ubl-membrane/           # Validation boundary
│   │   │   │   ├── Cargo.toml
│   │   │   │   └── src/
│   │   │   │       ├── lib.rs
│   │   │   │       └── validate.rs     # Entry validation
│   │   │   │
│   │   │   ├── ubl-pact/               # Multi-party consensus
│   │   │   │   ├── Cargo.toml
│   │   │   │   └── src/lib.rs
│   │   │   │
│   │   │   ├── ubl-policy-vm/          # Policy bytecode VM
│   │   │   │   ├── Cargo.toml
│   │   │   │   └── src/lib.rs
│   │   │   │
│   │   │   ├── ubl-runner-core/        # Command execution
│   │   │   │   ├── Cargo.toml
│   │   │   │   └── src/lib.rs
│   │   │   │
│   │   │   └── ubl-server/             # HTTP/SSE server
│   │   │       ├── Cargo.toml
│   │   │       │
│   │   │       └── src/
│   │   │           ├── main.rs         # Entry point
│   │   │           ├── db.rs           # PostgreSQL connection
│   │   │           ├── crypto.rs       # Cryptographic ops
│   │   │           ├── sse.rs          # SSE streaming
│   │   │           ├── metrics.rs      # Prometheus metrics
│   │   │           ├── keystore.rs     # Key management
│   │   │           ├── rate_limit.rs   # Rate limiting
│   │   │           ├── snapshots.rs    # State snapshots
│   │   │           │
│   │   │           ├── auth/
│   │   │           │   ├── session.rs          # Session management
│   │   │           │   ├── session_db.rs       # Session storage
│   │   │           │   └── require_stepup.rs   # Step-up auth
│   │   │           │
│   │   │           ├── messenger_gateway/      # Frontend gateway
│   │   │           │   ├── mod.rs
│   │   │           │   ├── routes.rs           # Gateway routes
│   │   │           │   ├── sse.rs              # Delta streaming
│   │   │           │   ├── idempotency.rs      # Request dedup
│   │   │           │   ├── projections.rs      # Read models
│   │   │           │   └── office_client.rs    # Office integration
│   │   │           │
│   │   │           ├── projections/            # Read models
│   │   │           │   ├── mod.rs
│   │   │           │   ├── jobs.rs             # Job projections
│   │   │           │   ├── messages.rs         # Message projections
│   │   │           │   ├── timeline.rs         # Unified timeline
│   │   │           │   ├── presence.rs         # Entity presence
│   │   │           │   ├── artifacts.rs        # Job artifacts
│   │   │           │   ├── job_events.rs       # Job timeline
│   │   │           │   ├── office.rs           # Office projections
│   │   │           │   ├── rebuild.rs          # Projection rebuild
│   │   │           │   └── routes.rs           # Query routes
│   │   │           │
│   │   │           ├── policy/
│   │   │           │   ├── mod.rs
│   │   │           │   └── policies.rs         # Policy Pack v1
│   │   │           │
│   │   │           ├── id_routes.rs            # Identity routes
│   │   │           ├── id_db.rs                # Identity storage
│   │   │           ├── id_ledger.rs            # Identity ledger
│   │   │           ├── id_session_token.rs     # Session tokens
│   │   │           ├── webauthn_store.rs       # WebAuthn storage
│   │   │           │
│   │   │           ├── console_v1.rs           # Console API
│   │   │           ├── messenger_v1.rs         # Messenger boundary
│   │   │           ├── registry_v1.rs          # Registry API
│   │   │           └── repo_routes.rs          # Repository routes
│   │   │
│   │   ├── openapi/
│   │   │   └── README.md               # OpenAPI specs
│   │   │
│   │   └── tests/
│   │       ├── Cargo.toml
│   │       └── golden_test.rs          # Golden path tests
│   │
│   ├── sql/                            # Database migrations
│   │   ├── MIGRATION_ORDER.txt         # Apply order
│   │   ├── README.md
│   │   │
│   │   ├── 00_base/                    # Core schema
│   │   │   ├── 000_core.sql            # Ledger tables
│   │   │   ├── 001_identity.sql        # Identity tables
│   │   │   ├── 002_policy.sql          # Policy tables
│   │   │   └── 003_triggers.sql        # NOTIFY triggers
│   │   │
│   │   ├── 10_projections/             # Read models
│   │   │   ├── 100_console.sql         # Console projections
│   │   │   ├── 101_messenger.sql       # Messenger projections
│   │   │   └── 102_office.sql          # Office projections
│   │   │
│   │   ├── 060_notify_minimal.sql      # Minimal NOTIFY
│   │   │
│   │   ├── 90_ops/
│   │   │   └── 900_disaster_recovery.sql
│   │   │
│   │   └── 99_legacy/                  # Deprecated (do not use)
│   │       └── ...
│   │
│   ├── containers/                     # Container definitions
│   │   ├── C.Messenger/                # Messenger events
│   │   │   ├── README.md
│   │   │   ├── EVENT_TYPES.md
│   │   │   ├── policy/ref.json
│   │   │   └── pacts/ref.json
│   │   │
│   │   ├── C.Jobs/                     # Job lifecycle
│   │   │   ├── README.md
│   │   │   ├── EVENT_TYPES.md
│   │   │   ├── policy/ref.json
│   │   │   └── pacts/ref.json
│   │   │
│   │   ├── C.Office/                   # LLM runtime events
│   │   │   ├── README.md
│   │   │   ├── EVENT_TYPES.md
│   │   │   ├── policy/ref.json
│   │   │   └── pacts/ref.json
│   │   │
│   │   ├── C.Pacts/                    # Multi-party agreements
│   │   ├── C.Policy/                   # Policy definitions
│   │   ├── C.Runner/                   # Command execution
│   │   └── C.Artifacts/                # Generated artifacts
│   │
│   ├── specs/                          # Philosophy & specs
│   │   ├── PHILOSOPHY.md               # ★ Trust architecture philosophy
│   │   └── ubl-membrane/
│   │       └── SPEC-UBL-MEMBRANE.md
│   │
│   ├── clients/
│   │   ├── cli/                        # Command-line client
│   │   │   ├── package.json
│   │   │   └── src/
│   │   │       ├── index.ts
│   │   │       └── cmds/
│   │   │           ├── atom.ts         # Canonicalize atom
│   │   │           ├── commit.ts       # Commit to ledger
│   │   │           ├── tail.ts         # SSE tail
│   │   │           ├── id.ts           # Identity ops
│   │   │           ├── link.ts         # Link ops
│   │   │           ├── pack.ts         # Pack ops
│   │   │           └── ...
│   │   │
│   │   ├── ts/sdk/                     # TypeScript SDK
│   │   │   ├── package.json
│   │   │   └── src/
│   │   │       ├── index.ts
│   │   │       └── repo.ts
│   │   │
│   │   └── types/                      # Shared TypeScript types
│   │       └── ubl/
│   │           └── index.d.ts
│   │
│   ├── manifests/                      # Configuration manifests
│   │   ├── containers.json
│   │   ├── policies.json
│   │   ├── routes.json
│   │   ├── offices.yaml
│   │   └── policy/
│   │       ├── asc.schema.json
│   │       └── policy_pack_v1.json
│   │
│   ├── mind/                           # ABAC/Agreements (TypeScript)
│   │   ├── package.json
│   │   └── src/
│   │       ├── index.ts
│   │       ├── abac.ts
│   │       └── agreements.ts
│   │
│   ├── runner/                         # Job runner
│   │   ├── package.json
│   │   ├── crypto.ts
│   │   └── pull_only.ts
│   │
│   └── infra/                          # Infrastructure
│       ├── docker-compose.stack.yml
│       ├── postgres/roles.sql
│       └── minio/
│           ├── policy.json
│           └── lifecycle.json
│
├── contracts/                          # JSON Schema contracts
│   ├── link_commit.schema.json
│   ├── identity_event.schema.json
│   ├── ws_receipt.schema.json
│   └── ubl/atoms/
│       ├── deploy.request.schema.json
│       └── ws.receipt.schema.json
│
├── tests/                              # Integration tests
│   ├── Cargo.toml                      # Rust test harness
│   ├── package.json                    # JS/TS tests
│   ├── playwright.config.ts            # E2E config
│   ├── vitest.config.ts                # Unit test config
│   │
│   ├── src/
│   │   ├── lib.rs
│   │   ├── fixtures.rs
│   │   ├── helpers.rs
│   │   └── clients/
│   │       ├── ubl_client.rs
│   │       └── office_client.rs
│   │
│   ├── tests/
│   │   ├── golden_path.rs              # Happy path tests
│   │   ├── diamond_complete.rs         # Diamond test
│   │   ├── entity_lifecycle.rs         # Entity tests
│   │   ├── session_management.rs       # Session tests
│   │   ├── job_execution.rs            # Job tests
│   │   ├── governance.rs               # Governance tests
│   │   ├── policy_enforcement.rs       # Policy tests
│   │   ├── projection_consistency.rs   # Projection tests
│   │   ├── resilience_tests.rs         # Failure tests
│   │   ├── performance.rs              # Perf tests
│   │   └── ...
│   │
│   ├── __tests__/                      # Frontend tests
│   │   ├── components/
│   │   ├── integration/
│   │   └── e2e/
│   │
│   └── docker-compose.*.yml            # Test environments
│
├── observability/                      # Monitoring & alerting
│   ├── docker-compose.observability.yml
│   ├── prometheus.yml
│   ├── alertmanager.yml
│   ├── loki-config.yml
│   ├── promtail-config.yml
│   ├── jaeger-config.yml
│   │
│   ├── grafana/
│   │   └── provisioning/
│   │       ├── datasources/prometheus.yml
│   │       └── dashboards/
│   │           ├── system-overview.json
│   │           ├── ubl-kernel.json
│   │           └── office-runtime.json
│   │
│   ├── prometheus/
│   │   ├── prometheus.yml
│   │   ├── alerts/cryptography.yml
│   │   └── recording-rules/
│   │       ├── latency.yml
│   │       └── throughput.yml
│   │
│   └── runbooks/                       # Operations runbooks
│       └── ...
│
└── docs/                               # Documentation
    ├── ARCHITECTURE.md                 # ★ This file
    ├── WIRING_GUIDE.md                 # Integration wiring
    ├── RUNBOOK.md                      # Local development guide
    ├── ROADMAP.md                      # Implementation status
    ├── STATUS.md                       # Current system status
    │
    ├── adrs/                           # Architecture Decision Records
    │   └── ADR-UBL-Console-001.v1.md
    │
    └── devops/                         # DevOps documentation
        └── ...
```

## Status

### Current State
- ✅ UBL Messenger frontend: React UI functional
- ✅ UBL Messenger backend (Node.js): Basic API working
- ✅ Office: Core LLM runtime implemented
- ✅ UBL: Event sourcing system operational
- 🚧 UBL Messenger backend (Rust): In development
- 🚧 Job cards: UI ready, backend integration pending
- 🚧 WebSocket: Real-time updates pending
- 🚧 Approval workflow: Integration pending

### Next Steps
See [ROADMAP.md](./ROADMAP.md) for complete implementation status.

## References

- [ROADMAP](./ROADMAP.md) - Implementation status and next steps
- [RUNBOOK](./RUNBOOK.md) - Local development guide
- [STATUS](./STATUS.md) - System health overview

## License

MIT

