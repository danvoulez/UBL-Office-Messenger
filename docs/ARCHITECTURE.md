# UBL Flagship Trinity Architecture

**Three independent systems working together to realize the LogLine Foundation vision**

## Overview

The UBL Flagship Trinity consists of three separate, independently deployable systems that communicate via APIs, WebSockets, and event streams:

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

**Location**: `ubl-messenger/`

**Purpose**: User-facing WhatsApp-like interface for professional collaboration

**Components**:
- **Frontend** (`ubl-messenger/frontend/`): React/TypeScript UI
- **Backend** (`ubl-messenger/backend/`): Rust API server (target)
- **Backend Node** (`ubl-messenger/backend-node/`): Node.js server (temporary)

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

**Location**: `office/office/`

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
docker run -p 3000:3000 ubl

# Deploy Office
cd office/office
docker build -t office .
docker run -p 8080:8080 office

# Deploy UBL Messenger
cd ubl-messenger
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
# Terminal 1: UBL
cd ubl
cargo run

# Terminal 2: Office
cd office/office
cargo run

# Terminal 3: UBL Messenger Backend
cd ubl-messenger/backend-node  # or ubl-messenger/backend for Rust
npm run server  # or cargo run

# Terminal 4: UBL Messenger Frontend
cd ubl-messenger/frontend
npm run dev
```

## Directory Structure

```
OFFICE-main/
├── ARCHITECTURE.md              # This file
├── README.md                    # Root README
│
├── ubl-messenger/              # System 1: UBL Messenger
│   ├── README.md
│   ├── frontend/               # React UI
│   ├── backend-node/           # Node.js backend (temp)
│   └── backend/                 # Rust backend (target)
│
├── office/                     # System 2: Office
│   ├── office/                 # Office Rust code
│   │   └── README.md
│   └── docker-compose.yml      # Orchestration
│
└── ubl/                        # System 3: UBL
    ├── README.md
    ├── kernel/                 # Core Rust implementation
    ├── containers/             # Container logic
    └── specs/                  # Specifications
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
See `# 🎯🔥 PROMPT 3: THE FLAGSHIP TRINITY.ini` for complete roadmap.

## References

- [Universal Historical Specification](./UNIVERSAL-HISTORICAL-SPECIFICATION.md)
- [LLM UX/UI Specification](./messenger/frontend/LLM%20UX/LLM-UI-UX.md)
- [Office Discovery](./office/office/DISCOVERY.md)
- [UBL Architecture](./UBL-Containers-main/ARCHITECTURE.md)

## License

MIT

