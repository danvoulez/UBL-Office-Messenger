# UBL 3.0 Implementation Roadmap

**Status**: Core Complete, All 15 Security Fixes Applied (2025-12-31)

## 🎯 Current Status Overview

### ✅ Foundation Complete
- ✅ Three systems properly organized and named
- ✅ UBL core ledger operational
- ✅ Office core LLM runtime operational  
- ✅ UBL Messenger basic UI and backend structure
- ✅ No code duplication
- ✅ Documentation structure in place

### ✅ Security & Integrity (Fixes Applied)
- ✅ Real Ed25519 cryptographic signatures
- ✅ WebAuthn PRF for client-side signing
- ✅ Multi-tenant isolation (tenant_id in all atoms)
- ✅ PII detection fail-closed
- ✅ FSM validation at Gateway and Membrane
- ✅ Serialization conflict retry
- ✅ ASC token enforcement

### ⏳ Core Features In Progress
- ⏳ **Job System** - Cards, execution, approval workflow
- ⏳ **WebSocket** - Real-time updates
- ⏳ **Complete Integration** - End-to-end workflows

---

## 📋 Detailed Implementation Checklist

### Phase 1: Foundation (Week 1) 🔴 CRITICAL

#### 1.1 UBL: C.Jobs Container
**Priority**: 🔴 CRITICAL - Blocks everything else

**Tasks**:
- [ ] Create `ubl/containers/C.Jobs/` directory structure
  ```
  C.Jobs/
  ├── boundary/README.md
  ├── inbox/README.md
  ├── outbox/README.md
  ├── local/README.md
  ├── pacts/ref.json
  ├── policy/ref.json
  ├── projections/README.md
  └── README.md
  ```
- [ ] Define job event types in UBL kernel
- [ ] Implement job queries: `/jobs/:id/events`, `/jobs/:id/receipt`
- [ ] Implement conversation queries: `/conversations/:id/events`
- [ ] Create job approval pacts

**Files to Create**:
- `ubl/containers/C.Jobs/` (entire directory)
- `ubl/kernel/rust/ubl-ledger/src/job_events.rs` (new)
- `ubl/kernel/rust/ubl-server/src/api/jobs.rs` (new)

---

#### 1.2 Office: JobExecutor Module
**Priority**: 🔴 CRITICAL - Enables job execution

**Tasks**:
- [ ] Create `apps/office/src/job_executor/` module
  ```
  job_executor/
  ├── mod.rs
  ├── executor.rs        # Main JobExecutor struct
  ├── progress.rs        # Progress streaming
  └── context.rs         # Conversation context building
  ```
- [ ] Implement `JobExecutor::execute_job()`
- [ ] Implement progress streaming
- [ ] Integrate with Office session system

**Files to Create**:
- `apps/office/src/job_executor/mod.rs`
- `apps/office/src/job_executor/executor.rs`
- `apps/office/src/job_executor/progress.rs`
- `apps/office/src/job_executor/context.rs`

**API Endpoints to Add**:
- `POST /jobs/execute`
- `GET /jobs/:id/status`
- `POST /jobs/:id/pause`
- `POST /jobs/:id/resume`

---

#### 1.3 Office: ApprovalManager Module
**Priority**: 🔴 CRITICAL - Enables approval workflow

**Tasks**:
- [ ] Create `apps/office/src/approval_manager/` module
  ```
  approval_manager/
  ├── mod.rs
  ├── manager.rs        # ApprovalManager struct
  └── decision.rs       # Approval decision handling
  ```
- [ ] Implement approval request/response flow
- [ ] Integrate with MessengerClient

**Files to Create**:
- `apps/office/src/approval_manager/mod.rs`
- `apps/office/src/approval_manager/manager.rs`
- `apps/office/src/approval_manager/decision.rs`

**API Endpoints to Add**:
- `POST /approvals`
- `GET /approvals/:id`
- `POST /approvals/:id/decide`

---

#### 1.4 Office: MessengerClient Module
**Priority**: 🔴 CRITICAL - Enables Office ↔ Messenger communication

**Tasks**:
- [ ] Create `apps/office/src/messenger_client/` module
  ```
  messenger_client/
  ├── mod.rs
  ├── client.rs         # HTTP client
  ├── cards.rs          # Card sending
  └── websocket.rs      # WebSocket client (optional)
  ```
- [ ] Implement HTTP client for Messenger API
- [ ] Implement card sending
- [ ] Implement progress update sending

**Files to Create**:
- `apps/office/src/messenger_client/mod.rs`
- `apps/office/src/messenger_client/client.rs`
- `apps/office/src/messenger_client/cards.rs`

---

#### 1.5 UBL Messenger Backend: Job Module
**Priority**: 🔴 CRITICAL - Enables job management

**Tasks**:
- [ ] Create `apps/messenger/backend/src/job/` module
  ```
  job/
  ├── mod.rs
  ├── job.rs            # Job entity
  ├── lifecycle.rs      # State machine
  ├── repository.rs     # Persistence + UBL
  └── routes.rs         # HTTP endpoints
  ```
- [ ] Implement job CRUD operations
- [ ] Implement job lifecycle (created→running→done)
- [ ] Publish events to UBL

**Files to Create**:
- `apps/messenger/backend/src/job/mod.rs`
- `apps/messenger/backend/src/job/job.rs`
- `apps/messenger/backend/src/job/lifecycle.rs`
- `apps/messenger/backend/src/job/repository.rs`
- `apps/messenger/backend/src/job/routes.rs`

**API Endpoints to Add**:
- `POST /api/jobs`
- `GET /api/jobs`
- `GET /api/jobs/:id`
- `PATCH /api/jobs/:id`
- `POST /api/jobs/:id/start`
- `POST /api/jobs/:id/complete`
- `POST /api/jobs/:id/cancel`
- `POST /api/jobs/:id/approve`
- `POST /api/jobs/:id/reject`
- `POST /api/jobs/:id/progress`

---

### Phase 2: Integration (Week 2) 🟡 HIGH PRIORITY

#### 2.1 UBL Messenger Backend: Card Module
**Priority**: 🟡 HIGH - Enables card rendering

**Tasks**:
- [ ] Create `apps/messenger/backend/src/card/` module
  ```
  card/
  ├── mod.rs
  ├── card_types.rs      # JobInit, JobProgress, JobComplete, Approval
  ├── renderer.rs       # Generate card JSON
  └── actions.rs        # Handle button clicks
  ```
- [ ] Define card types (JobInit, JobProgress, JobComplete, Approval)
- [ ] Implement card rendering
- [ ] Implement action handling

**Files to Create**:
- `apps/messenger/backend/src/card/mod.rs`
- `apps/messenger/backend/src/card/card_types.rs`
- `apps/messenger/backend/src/card/renderer.rs`
- `apps/messenger/backend/src/card/actions.rs`

**API Endpoints to Add**:
- `GET /api/conversations/:id/cards`
- `POST /api/cards/:id/action`

---

#### 2.2 UBL Messenger Frontend: Job Cards
**Priority**: 🟡 HIGH - Enables UI interaction

**Tasks**:
- [ ] Create `apps/messenger/frontend/src/components/cards/` directory
  ```
  cards/
  ├── JobInitCard.tsx
  ├── JobProgressCard.tsx
  ├── JobCompleteCard.tsx
  ├── ApprovalCard.tsx
  └── CardRenderer.tsx
  ```
- [ ] Implement 4 card types
- [ ] Implement card actions (buttons)
- [ ] Integrate with message list

**Files to Create**:
- `apps/messenger/frontend/src/components/cards/JobInitCard.tsx`
- `apps/messenger/frontend/src/components/cards/JobProgressCard.tsx`
- `apps/messenger/frontend/src/components/cards/JobCompleteCard.tsx`
- `apps/messenger/frontend/src/components/cards/ApprovalCard.tsx`
- `apps/messenger/frontend/src/components/cards/CardRenderer.tsx`

---

#### 2.3 UBL Messenger: WebSocket Module
**Priority**: 🟡 HIGH - Enables real-time updates

**Tasks**:
- [ ] Create `apps/messenger/backend/src/websocket/` module
  ```
  websocket/
  ├── mod.rs
  ├── server.rs          # WebSocket server
  ├── rooms.rs           # Room management
  └── broadcast.rs      # Event broadcasting
  ```
- [ ] Implement WebSocket server
- [ ] Implement room management
- [ ] Implement event broadcasting

**Files to Create**:
- `apps/messenger/backend/src/websocket/mod.rs`
- `apps/messenger/backend/src/websocket/server.rs`
- `apps/messenger/backend/src/websocket/rooms.rs`
- `apps/messenger/backend/src/websocket/broadcast.rs`

**Frontend Hook**:
- [ ] Create `apps/messenger/frontend/src/hooks/useWebSocket.ts`

**WebSocket Endpoint**:
- `WS /ws/conversations/:id`

---

#### 2.4 Office ↔ UBL Messenger Integration
**Priority**: 🟡 HIGH - Connects systems

**Tasks**:
- [ ] Complete Office client in Messenger backend
- [ ] Complete Messenger client in Office
- [ ] Test job execution flow
- [ ] Test approval flow

**Files to Update**:
- `apps/messenger/backend/src/office_client/mod.rs` (complete implementation)
- `apps/office/src/messenger_client/mod.rs` (complete implementation)

---

#### 2.5 UBL Messenger ↔ UBL Integration
**Priority**: 🟡 HIGH - Event publishing

**Tasks**:
- [ ] Complete UBL client in Messenger backend
- [ ] Implement event publishing (job.*, approval.*)
- [ ] Implement event queries

**Files to Update**:
- `apps/messenger/backend/src/ubl_client/mod.rs` (complete implementation)

---

### Phase 3: UX Polish (Week 3) 🟢 MEDIUM PRIORITY

#### 3.1 Frontend Enhancements
- [ ] WhatsApp-like UI refinement
- [ ] Mobile responsive design
- [ ] Job timeline component
- [ ] Participant avatars (human/agent distinction)

#### 3.2 Backend Enhancements
- [ ] Participant module (`participant/human.rs`, `participant/agent.rs`)
- [ ] Agent management endpoints (`/api/agents`)
- [ ] Conversation search
- [ ] Message search

---

### Phase 4: Production (Week 4) 🟢 LOW PRIORITY

#### 4.1 Testing
- [ ] Unit tests for all modules
- [ ] Integration tests
- [ ] End-to-end tests
- [ ] Performance tests

#### 4.2 Documentation
- [ ] API documentation
- [ ] Deployment guide
- [ ] User guide
- [ ] Developer guide

#### 4.3 Deployment
- [ ] Complete docker-compose.yml
- [ ] Production configuration
- [ ] Monitoring setup
- [ ] Logging setup

---

## 🎯 Critical Path

The **critical path** for getting the core job feature working:

```
1. UBL C.Jobs Container
   ↓
2. Office JobExecutor
   ↓
3. Messenger Backend Job Module
   ↓
4. Messenger Frontend Job Cards
   ↓
5. Integration & WebSocket
```

**Without step 1, nothing else works.**  
**Without steps 2-3, jobs can't execute.**  
**Without step 4, users can't interact.**  
**Without step 5, it's not real-time.**

---

## 📊 Progress Tracking

### UBL
- [x] Core ledger ✅
- [x] Existing containers ✅
- [ ] C.Jobs container ❌
- [ ] Job events ❌
- [ ] Job queries ❌

### Office
- [x] Core runtime ✅
- [x] Entity management ✅
- [x] Session handling ✅
- [x] Context & Narrator ✅
- [x] Governance ✅
- [ ] JobExecutor ❌
- [ ] ApprovalManager ❌
- [ ] MessengerClient ❌
- [ ] Messenger affordances ❌

### UBL Messenger
- [x] Frontend basic UI ✅
- [x] Backend basic structure ✅
- [ ] Job cards ❌
- [ ] Job backend module ❌
- [ ] Card module ❌
- [ ] WebSocket ❌
- [ ] Complete Office client ❌
- [ ] Complete UBL client ❌

---

## 🚀 Quick Start Implementation

To start implementing, follow this order:

```bash
# 1. UBL C.Jobs Container (FOUNDATION)
cd ubl/containers
mkdir -p C.Jobs/{boundary,inbox,outbox,local,pacts,policy,projections}
# Then implement job events in kernel

# 2. Office JobExecutor
cd ../../apps/office/src
mkdir -p job_executor approval_manager messenger_client
# Implement modules

# 3. Messenger Backend Job Module
cd ../../../apps/messenger/backend/src
mkdir -p job card websocket participant
# Implement modules

# 4. Messenger Frontend Cards
cd ../../frontend/src/components
mkdir -p cards
# Implement card components
```

---

## 📝 Notes

- **Current state**: Foundation is solid, core features need implementation
- **Biggest gap**: Job system (the core workflow feature)
- **Estimated effort**: 4 weeks for complete implementation per spec
- **Can start immediately**: UBL C.Jobs container (blocks everything else)

---

**Next Action**: Start with UBL C.Jobs container implementation.

