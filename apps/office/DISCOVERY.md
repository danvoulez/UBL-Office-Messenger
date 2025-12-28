# UBL 2.0 Discovery Document

**Date:** 2024-12-27
**Purpose:** Document findings from UBL-Containers-main analysis for OFFICE implementation

---

## Executive Summary

The Universal Business Ledger (UBL) is a cryptographically-secured, append-only ledger system that separates **meaning** (Mind/TypeScript) from **proof** (Body/Rust). It provides trustworthy business records through:

- **Immutable history** via hash chains and Ed25519 signatures
- **Physical invariants** enforced by a deterministic membrane
- **Sovereign containers** that don't share state
- **Event sourcing** where all state derives from commit history
- **Multi-layer trust** (L0-L5 policy layers)

---

## 1. API Endpoints Available

### Core Ledger Endpoints

| Endpoint | Method | Auth | Purpose |
|----------|--------|------|---------|
| `/health` | GET | None | Server health check |
| `/state/:container_id` | GET | None | Get container state (sequence, balance, last hash) |
| `/link/validate` | POST | Bearer JWT/Cookie | Validate commit against membrane |
| `/link/commit` | POST | Bearer JWT/Cookie | Atomically validate and append to ledger |
| `/ledger/:container_id/tail` | GET | None | SSE stream of ledger events |

### Identity & Authentication (WebAuthn)

| Endpoint | Method | Auth | Purpose |
|----------|--------|------|---------|
| `/id/register/begin` | POST | None | Start passkey registration |
| `/id/register/finish` | POST | None | Complete passkey registration |
| `/id/login/begin` | POST | None | Start passkey login |
| `/id/login/finish` | POST | None | Complete login (returns session) |
| `/id/stepup/begin` | POST | Cookie | Start admin step-up |
| `/id/stepup/finish` | POST | Cookie | Complete step-up |
| `/id/session/token` | POST | Cookie | Exchange session for JWT |
| `/id/whoami` | GET | Cookie/JWT | Get current identity |

### Agent Management (LLM/App Identity)

| Endpoint | Method | Auth | Purpose |
|----------|--------|------|---------|
| `/id/agents` | POST | ASC | Create new LLM or App agent |
| `/id/agents/:sid/asc` | POST | Admin | Issue Agent Signing Certificate |
| `/id/agents/:sid/rotate` | POST | Owner | Rotate agent's public key |

---

## 2. Core Data Structures

### LinkCommit (Mind↔Body Interface)

```rust
pub struct LinkCommit {
    pub version: u8,                    // Protocol version (must = 1)
    pub container_id: String,           // Target container (Hash32)
    pub expected_sequence: u64,         // Causal control
    pub previous_hash: String,          // Hash chain link
    pub atom_hash: String,              // Semantic content hash
    pub intent_class: IntentClass,      // O/C/E/V
    pub physics_delta: i128,            // Physical change
    pub pact: Option<PactProof>,        // Authority proof
    pub author_pubkey: String,          // Ed25519 public key
    pub signature: String,              // Ed25519 signature
}

pub enum IntentClass {
    Observation,  // 0x00 - Δ = 0 (read-only)
    Conservation, // 0x01 - ∑Δ = 0 (transfers)
    Entropy,      // 0x02 - Δ > 0 or < 0 (creation/destruction)
    Evolution,    // 0x03 - Rule changes (L5 pact)
}
```

### LedgerEntry

```rust
pub struct LedgerEntry {
    pub sequence: u64,           // Position in ledger (1-indexed)
    pub entry_hash: String,      // Hash of this entry
    pub link: LinkCommit,        // The original commit
    pub timestamp: i64,          // Unix timestamp
}

pub struct LedgerState {
    pub container_id: String,
    pub sequence: u64,
    pub last_hash: String,
    pub physical_balance: i128,
    pub merkle_root: String,
}
```

### Pact (Authority)

```rust
pub struct Pact {
    pub pact_id: String,
    pub version: u8,
    pub scope: PactScope,        // Container/Namespace/Global
    pub threshold: usize,        // Min signatures
    pub signers: HashSet<String>,
    pub window: TimeWindow,
    pub risk_level: RiskLevel,   // L0-L5
    pub container_id: Option<String>,
}

pub enum RiskLevel {
    L0 = 0,  // Observation
    L1 = 1,  // Low impact
    L2 = 2,  // Local impact
    L3 = 3,  // Financial impact
    L4 = 4,  // Systemic impact
    L5 = 5,  // Sovereignty/Evolution
}
```

---

## 3. Event Types and Affordances

### Intent Classes (What can traverse a Link)

| Class | physics_delta | Pact Required | Use Cases |
|-------|---------------|---------------|-----------|
| Observation (0x00) | Must = 0 | No | Logging, monitoring, audits |
| Conservation (0x01) | balance ≥ 0 | If amount > threshold | Payments, transfers |
| Entropy (0x02) | Any | Yes (creation_authority) | Minting, burning |
| Evolution (0x03) | Any | Yes (L5) | Protocol upgrades |

### Affordances by Risk Level

- **L0**: Read ledger, query state, validate commits
- **L1-L2**: Submit conservation links (transfers)
- **L3-L4**: Entropy operations (creation/destruction)
- **L5**: Evolution operations (governance changes)

---

## 4. Trust Architecture (L0-L5)

```
L5  SOVEREIGNTY / EVOLUTION     Multi-sig board, all signers required
L4  SYSTEMIC IMPACT             Senior management, supermajority
L3  FINANCIAL IMPACT            Department heads, approval required
L2  LOCAL IMPACT                Team members, policy-dependent
L1  LOW IMPACT                  Any verified user, no pact
L0  OBSERVATION                 Unauthenticated, read-only
```

---

## 5. Event Sourcing Model

```
State = rehydrate(all_events)

[E₀: "init", Δ=0] → hash₀
    ↓
[E₁: "transfer", Δ=-100] → hash₁
    ↓
[E₂: "transfer", Δ=+50] → hash₂
    ↓
[E₃: "mint", Δ=+1000] → hash₃

state = {
    sequence: 4,
    last_hash: hash₃,
    physical_balance: 0 + (-100) + 50 + 1000 = 950,
}
```

### Commit Flow

1. Intent Formation (Mind)
2. Canonicalization (ubl-atom)
3. Hashing (ubl-kernel, BLAKE3)
4. Signing (Ed25519)
5. Link Creation (ubl-link)
6. HTTP Transport (POST /link/commit)
7. Validation (ubl-membrane V1-V8)
8. Append (PostgreSQL SERIALIZABLE)
9. Receipt Return
10. SSE Broadcast

---

## 6. Rust Kernel Modules

| Module | Purpose | Key Functions |
|--------|---------|---------------|
| ubl-atom | Deterministic JSON | `canonicalize()` |
| ubl-kernel | Cryptography | `hash_atom()`, `sign()`, `verify()` |
| ubl-link | Mind↔Body interface | `signing_bytes()`, `LinkCommit` |
| ubl-ledger | Append-only history | `append()`, `merkle_root()` |
| ubl-membrane | Physics validation | `validate()`, `decide()` |
| ubl-pact | Authority & consensus | `validate_pact()` |
| ubl-policy-vm | TDLN executor | `evaluate()` |
| ubl-runner-core | Execution & receipts | `ExecutionJob`, `ExecutionReceipt` |
| ubl-server | HTTP API + WebAuthn | Axum routes |

---

## 7. Container Structures

### C.Policy (Blue - Admin)
- Policy evaluation and TDLN translation
- Requires step-up for admin operations

### C.Runner (Black - Execution)
- Isolated execution of jobs
- Sandbox with timeout, memory limits, network isolation

### C.Messenger (Green - Communication)
- Event distribution and notifications
- Routes events to authorized containers

### C.Pacts (Blue - Governance)
- Authority and consensus definitions
- Stores pact registry

### C.Artifacts (Green - Storage)
- Long-term artifact storage
- Content-addressed retrieval

---

## 8. Integration Points for OFFICE

### Must Consume from UBL:

1. **Event Sourcing**
   - Subscribe to SSE streams via `/ledger/:id/tail`
   - Replay events to reconstruct state
   - Publish LLM actions as events

2. **Trust Architecture**
   - Consume policy chains (L1-L6)
   - Implement policy pinning
   - Validate cryptographic signatures

3. **Affordances System**
   - Discover affordances via UBL API
   - Execute actions through affordances
   - Simulate before execute

4. **Agreement Management**
   - Create agreements for L3+ actions
   - Validate terms via TDLN
   - Track obligations

5. **Trajectory System**
   - Register session trajectories
   - Analyze causality
   - Pattern extraction for dreaming

---

## 9. Gaps and Considerations

### Found:
- No explicit "trajectory" tracking in current kernel
- Policy VM uses simple rules, WASM support planned
- WebAuthn fully implemented for humans
- Agent (LLM) identity via Ed25519 ASC

### Recommendations:
- Implement trajectory as custom events in ledger
- Use simple policy evaluation until WASM ready
- Store constitution as events (versionable)
- Use handovers as specialized events

---

## 10. Key Files Referenced

- `kernel/rust/ubl-atom/src/lib.rs` - Canonicalization
- `kernel/rust/ubl-kernel/src/lib.rs` - Cryptography
- `kernel/rust/ubl-link/src/lib.rs` - Link commit
- `kernel/rust/ubl-ledger/src/lib.rs` - Ledger operations
- `kernel/rust/ubl-membrane/src/lib.rs` - Validation
- `kernel/rust/ubl-pact/src/lib.rs` - Authority
- `kernel/rust/ubl-server/src/main.rs` - HTTP API
- `docs/blueprint/openapi/ubl.openapi.yaml` - API spec
- `manifests/routes.json` - Container routing
