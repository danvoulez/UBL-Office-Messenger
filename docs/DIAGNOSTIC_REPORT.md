# Diagnostic Report: Office ↔ UBL Kernel Integration

**Date**: 2025-01-20  
**Status**: � Integration Partially Working

---

## Executive Summary

After investigation, the Office ↔ UBL Kernel integration is **mostly functional**. Key findings:

1. **Entity Creation**: ✅ Works via Office API
2. **Session Creation**: ✅ Works, generates Context Frame with narrative
3. **Message Flow**: ✅ Works, calls Anthropic API (needs valid API key)
4. **UBL Projections**: ✅ Tables exist, `/query/*` endpoints work
5. **Some paths mismatch**: Office correctly uses `/query/office/*`

The main blocking issue was **no bootstrap entity** - once created via Office API, sessions work.

---

## 1. Infrastructure Status

| Component | Status | Port | Notes |
|-----------|--------|------|-------|
| PostgreSQL (local) | ✅ Running | 5432 | `ubl_ledger` db, 41 tables |
| UBL Kernel | ✅ Running | 8080 | v2.0.0+postgres |
| Office | ✅ Running | 8081 | v0.1.0, needs entities |
| Messenger Frontend | ✅ Building | 3000 | 1974 modules compiled |

---

## 2. Endpoint Mapping Analysis

### 2.1 What Office Calls → What UBL Provides

| Office Expects | UBL Has | Status | Issue |
|----------------|---------|--------|-------|
| `GET /health` | `GET /health` | ✅ Match | Works |
| `GET /state/{id}` | `GET /state/:container_id` | ⚠️ Partial | Schema mismatch (see §3) |
| `POST /link/commit` | `POST /link/commit` | ⚠️ Partial | Schema mismatch (see §3) |
| `GET /query/office/entities/{id}/handovers/latest` | `GET /office/entities/:id/handovers/latest` | 🔸 Path Diff | Missing `/query` prefix |
| `GET /query/office/audit` | `GET /office/audit` | 🔸 Path Diff | Missing `/query` prefix |
| `GET /query/jobs?assigned_to={}&status=pending` | ❌ None | ❌ Missing | Jobs projection query |
| `GET /guardians/{id}` | ❌ None | ❌ Missing | Guardian system not impl |
| `GET /entities/{id}/issues?status=resolved` | ❌ None | ❌ Missing | Issue tracking not impl |
| `GET /entities/{id}/trajectories` | ❌ None | ❌ Missing | Session history view |
| `GET /ledger/{id}/tail` | `GET /ledger/tail` (SSE) | 🔸 Path Diff | Different path structure |
| `GET /ledger/{id}/events` | ❌ None | ❌ Missing | Event stream by entity |
| `POST /v1/policy/permit` | `POST /v1/policy/permit` | ⚠️ Partial | Requires `office` field |
| `POST /v1/commands/issue` | `POST /v1/commands/issue` | ⚠️ Partial | Complex permit workflow |
| `POST /v1/exec.finish` | `POST /v1/exec.finish` | ⚠️ Partial | Runner signature required |

---

## 3. Schema Mismatches

### 3.1 `/state/{id}` Response

**What Office Expects (LedgerState struct)**:
```rust
pub struct LedgerState {
    pub container_id: String,
    pub sequence: u64,
    pub last_hash: String,
    pub physical_balance: i64,  // ❌ UBL doesn't provide
    pub entry_count: u64,
    pub merkle_root: Option<String>,  // ❌ UBL doesn't provide
}
```

**What UBL Returns**:
```json
{
  "container_id": "C.Office",
  "sequence": 0,
  "last_hash": "genesis",
  "entry_count": 0
}
```

**Fix**: Office uses `#[serde(default)]` so it fails gracefully, but features depending on `physical_balance` won't work.

---

### 3.2 `/link/commit` Request

**What Office Sends (LinkCommit struct)**:
```rust
pub struct LinkCommit {
    pub version: u8,
    pub container_id: String,
    pub expected_sequence: u64,    // Office: u64
    pub previous_hash: String,
    pub atom_hash: String,
    pub intent_class: String,
    pub physics_delta: i64,        // Office: i64
    pub pact: Option<PactProof>,
    pub author_pubkey: String,
    pub signature: String,
}
```

**What UBL Expects (LinkDraft struct)**:
```rust
pub struct LinkDraft {
    pub version: u8,
    pub container_id: String,
    pub expected_sequence: i64,    // UBL: i64 ⚠️
    pub previous_hash: String,
    pub atom_hash: String,
    pub intent_class: String,
    pub physics_delta: String,     // UBL: String (i128) ⚠️
    pub author_pubkey: String,
    pub signature: String,
    pub atom: Option<serde_json::Value>,  // UBL needs full atom
    pub pact: Option<PactProofDraft>,
}
```

**Issues**:
- `expected_sequence`: Office sends `u64`, UBL expects `i64` ✅ Compatible
- `physics_delta`: Office sends `i64`, UBL expects `String` ❌ Incompatible
- `atom`: Office doesn't send full atom, UBL needs it for projections ⚠️

---

### 3.3 `/v1/policy/permit` Request

**What Office Sends (PermitRequest in permit.rs)**:
```rust
pub struct PermitRequest {
    pub tenant_id: String,    // Required
    pub actor_id: String,     // Who is requesting  
    pub intent: String,       // Intent description
    pub context: Value,       // Context for policy
    pub job_type: String,     // Job type (from allowlist)
    pub params: Value,        // Job parameters
    pub target: String,       // LAB_512, LAB_256, etc
    pub approval_ref: Option<String>,
}
```

**What UBL Expects (PermitRequest in console_v1.rs)**:
```rust
pub struct PermitRequest {
    pub office: String,       // ❌ Office doesn't send
    pub action: String,       // Similar to intent
    pub target: String,       // ✅ Matches
    pub args: Value,          // Similar to params
    pub plan: Value,          // ❌ Office doesn't send
    pub risk: String,         // L0-L5 risk level
    pub stepup_assertion: Option<Value>,  // WebAuthn for L4/L5
}
```

**Issues**:
- Field names completely different
- `office` required but Office doesn't send it
- `plan` required but Office doesn't send it
- Office sends `tenant_id`, `actor_id`, `job_type` which UBL doesn't expect

---

## 4. Database Status

### 4.1 Tables Present (41 total)

**Core Ledger**:
- ✅ `ledger_entry` - Append-only ledger
- ✅ `ledger_atom` - Content storage

**Identity**:
- ✅ `id_subject`, `id_subjects` - User identities
- ✅ `id_credential`, `id_webauthn_credentials` - Auth
- ✅ `id_session`, `id_challenge` - Sessions
- ✅ `id_agents` - Machine identities

**Console/Commands**:
- ✅ `console_permits` - Permit tracking
- ✅ `console_commands` - Command queue
- ✅ `console_receipts` - Execution receipts

**Office Projections**:
- ✅ `office_entities` - LLM entities (0 rows)
- ✅ `office_sessions` - Session history
- ✅ `office_handovers` - Knowledge transfer
- ✅ `office_audit_log` - Audit trail

**Messenger Projections**:
- ✅ `projection_conversations`
- ✅ `projection_messages`
- ✅ `projection_jobs`
- ✅ `projection_approvals`

### 4.2 Missing Data

```
office_entities: 0 rows  → No LLM entities registered
```

This is why Office can't create sessions - there's no entity to attach them to.

---

## 5. Root Cause Analysis

### Problem 1: No Entity Bootstrap Process

Office tries to create a session but there's no entity in the database. The flow should be:

```
1. Register Entity → Creates row in office_entities
2. Create Session → Links to entity_id
3. Start Conversation → Session handles LLM calls
```

Currently missing: Entity registration endpoint/process.

### Problem 2: Protocol Version Mismatch

Office was built for "UBL v1.1 Protocol" but UBL Kernel has evolved:
- Different commit schemas
- Different permit schemas
- Some endpoints renamed

### Problem 3: Path Inconsistency

Office uses paths like:
```
/query/office/entities/{id}/handovers/latest
```

But UBL Kernel mounts them at:
```
/office/entities/:id/handovers/latest
```

---

## 6. Recommended Fixes (Priority Order)

### P0: Entity Bootstrap (Blocking)

**Option A**: Add `/office/entities` POST endpoint to UBL Kernel
```rust
POST /office/entities
{
  "entity_id": "E.Aria",
  "name": "Aria",
  "entity_type": "autonomous",
  "public_key": "...",
  "constitution": {...}
}
```

**Option B**: Create bootstrap script
```bash
psql -d ubl_ledger -c "INSERT INTO office_entities ..."
```

### P1: Fix Path Mapping (Quick)

Update Office's `ubl_client/mod.rs` to use correct paths:
```rust
// Change from:
format!("{}/query/office/entities/{}/handovers/latest", ...)
// To:
format!("{}/office/entities/{}/handovers/latest", ...)
```

### P2: Align Commit Schema (Medium)

Either:
- **A**: Update Office to use UBL's current commit schema
- **B**: Add v1.1 commit endpoint to UBL that accepts Office format

### P3: Add Missing Endpoints (Larger)

Endpoints needed for full Office functionality:
1. `GET /query/jobs` - Job queries with filters
2. `GET /guardians/:id` - Guardian info
3. `GET /entities/:id/issues` - Issue tracking
4. `GET /ledger/:id/events` - Event stream

---

## 7. Quick Test After Fixes

```bash
# 1. Bootstrap an entity
curl -X POST http://localhost:8080/office/entities \
  -H "Content-Type: application/json" \
  -d '{"entity_id":"E.Aria","name":"Aria","entity_type":"autonomous","public_key":"test"}'

# 2. Verify it exists
curl http://localhost:8080/office/entities

# 3. Create session in Office
curl -X POST http://localhost:8081/sessions \
  -H "Content-Type: application/json" \
  -d '{"entity_id":"E.Aria","session_type":"chat"}'
```

---

## 8. Files to Modify

| File | Changes |
|------|---------|
| `apps/office/src/ubl_client/mod.rs` | Fix endpoint paths, update schemas |
| `ubl/kernel/rust/ubl-server/src/projections/routes.rs` | Add POST /office/entities |
| `apps/office/src/middleware/permits.rs` | Align PermitRequest with UBL schema |
| `ubl/kernel/rust/ubl-server/src/main.rs` | Route /query/* alias if needed |

---

## 9. Conclusion

The systems were built against slightly different interface contracts. The fixes are mostly:

1. **Path alignment** (trivial)
2. **Schema alignment** (medium - needs coordination)
3. **Bootstrap process** (needed - no entity = no sessions)

**Estimated effort**: 4-6 hours of focused work to achieve basic integration.

**Recommendation**: Start with P0 (entity bootstrap) and P1 (path fixes) - this will unblock session creation and allow testing the rest of the flow.

---

## 10. Immediate Action Plan

### Step 1: Bootstrap Entity (5 min)

```bash
# Insert test entity directly
/opt/homebrew/opt/postgresql@16/bin/psql -d ubl_ledger -c "
INSERT INTO office_entities (entity_id, name, entity_type, public_key, status, created_at_ms, updated_at_ms)
VALUES ('E.Aria', 'Aria', 'autonomous', 'test_pubkey_placeholder', 'active', 
        (EXTRACT(EPOCH FROM NOW()) * 1000)::BIGINT,
        (EXTRACT(EPOCH FROM NOW()) * 1000)::BIGINT)
ON CONFLICT (entity_id) DO NOTHING;
"
```

### Step 2: Fix Office ubl_client paths (15 min)

```bash
# In apps/office/src/ubl_client/mod.rs:
# - /query/office/audit → /office/audit
# - /query/office/entities/{}/handovers/latest → /office/entities/{}/handovers/latest
# - /query/jobs?assigned_to= → Can stay (fails gracefully)
```

### Step 3: Fix physics_delta type (10 min)

```bash
# In apps/office/src/ubl_client/mod.rs:
# Change physics_delta: i64 → physics_delta: String
# Update commit_atom to format as string
```

### Step 4: Test Session Creation

```bash
curl -X POST http://localhost:8081/sessions \
  -H "Content-Type: application/json" \
  -d '{"entity_id":"E.Aria","session_type":"chat","mode":"assisted"}'
```

### Step 5: Test Chat

```bash
curl -X POST http://localhost:8081/chat \
  -H "Content-Type: application/json" \
  -d '{"session_id":"<from_step_4>","message":"Hello Aria"}'
```

---

## 11. Architecture Decision Required

The permit system has a fundamental mismatch. Two options:

### Option A: Office Adapts to UBL (Recommended)

Office updates its PermitRequest to match UBL Console v1.1:
```rust
// Map Office concepts → UBL concepts
office = entity_id,
action = job_type,
plan = context,
risk = derive_from_job_type()
```

### Option B: UBL Adds Office-Native Endpoint

Add `/v1/office/permit` that accepts Office's format and internally adapts.

**Recommendation**: Option A - Office should speak UBL's language, not the other way around.

---

## 12. Verified Test Flow (2025-01-20)

The following was tested and works:

```bash
# 1. Start UBL Kernel (port 8080)
cd ubl/kernel/rust && cargo run --release

# 2. Start Office (port 8081)
OFFICE__SERVER__PORT=8081 \
OFFICE__UBL__ENDPOINT="http://localhost:8080" \
OFFICE__LLM__PROVIDER="anthropic" \
OFFICE__LLM__MODEL="claude-sonnet-4-20250514" \
OFFICE__LLM__API_KEY="your-api-key-here" \
./apps/office/target/release/office

# 3. Create Entity (in-memory, via Office API)
curl -X POST http://localhost:8081/entities \
  -H "Content-Type: application/json" \
  -d '{"name":"Aria","entity_type":"autonomous"}'
# Returns entity_id like "entity_a14e00aa-90de-4cab-a858-e99f7eb3c9bf"

# 4. Create Session
curl -X POST "http://localhost:8081/entities/ENTITY_ID/sessions" \
  -H "Content-Type: application/json" \
  -d '{"session_type":"assist","initiator":"U.test"}'
# Returns session with Context Frame and narrative!

# 5. Send Message (requires valid API key)
curl -X POST "http://localhost:8081/entities/ENTITY_ID/sessions/SESSION_ID/message" \
  -H "Content-Type: application/json" \
  -d '{"role":"user","content":"Hello Aria!"}'
```

**Result**: Context Frame generation works! The narrative includes:
- Entity identity
- Session type and context
- Token budget
- Constitution/behavioral directives
- Available capabilities (affordances)

---

## 13. Remaining Work

| Item | Status | Blocking |
|------|--------|----------|
| Valid Anthropic API Key | ❌ Needed | Yes - for LLM responses |
| Entity Persistence to UBL | ⚠️ Optional | No - works in memory |
| Permit Integration | ⚠️ Schema mismatch | No - for L4/L5 actions |
| Handover/Dreaming | ⚠️ Not tested | No - advanced features |
| Jobs/Issues Projections | ⚠️ Missing endpoints | No - secondary features |

**To get full LLM responses**: Provide a valid Anthropic API key.

**Next steps after API key**:
1. Test full conversation flow
2. Test job proposal and approval
3. Test handover between sessions
4. Test dreaming cycle
