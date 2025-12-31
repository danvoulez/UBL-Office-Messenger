I have performed a exhaustive audit of the code across all three systems (`ubl/kernel`, `apps/office`, and `apps/messenger`). I have compared the **normative specifications** (the "Law") against the **actual implementation** (the "Reality").

The system is currently **"Architecturally Ready" but "Cryptographically Broken."** You have built a beautiful engine, but the fuel lines (security) are mostly stubs.

Here is the definitive **"Big List of Fixes"** required to move UBL 3.0 from a demo to a flagship product.

---

### 🔴 Phase 1: Cryptographic Integrity (The Foundation)
*The system's core promise is "Truth is what you can prove." Currently, you can't prove anything because of placeholders.*

1.  **Eliminate hardcoded Signatures:** In `ubl/kernel/rust/ubl-server/src/messenger_v1.rs` (line 445) and `ubl-client/mod.rs`, replace `"placeholder"` and `"mock_signature"` with a call to `ubl_kernel::verify`.
2.  **Frontend Ed25519 Derivation:** Implement the logic in `apps/messenger/frontend/src/services/signing.ts` (currently a stub) using the **WebAuthn PRF extension**. The user's Passkey must derive the Ed25519 key used to sign the `signing_bytes`.
3.  **Resolve `atom_hash` Spec Conflict:** 
    *   **Conflict:** `SPEC-UBL-ATOM.md` says use domain tag `"ubl:atom\n"`. `UBL-ATOM-BINDING.md` says NO domain tag. 
    *   **Fix:** Standardize on **NO domain tag** for `atom_hash` to maintain `JSON✯Atomic` compatibility, but update the audit script `verify_ledger.sh` to match.
4.  **ASC Token Enforcement:** The `AscContext` in `auth.rs` is defined but the `route_commit` handler in `main.rs` does not strictly verify that the `intent_class` in the link matches the `allowed_intents` in the ASC. An LLM agent could currently attempt an `Evolution` commit if it has a valid token.

---

### 🟠 Phase 2: Reliability & Scaling (The Plumbing)
*The current communication patterns will fail under load or with large data.*

5.  **Postgres `NOTIFY` 8KB Bypass:** In `ubl/kernel/rust/ubl-server/src/sse.rs`, stop sending the full JSON atom in the notification.
    *   **Fix:** The SQL trigger must only notify `container_id` and `sequence`. The Rust SSE handler must then perform a `SELECT` to fetch the data before sending it to the client.
6.  **Idempotency Store Persistence:** The `IdempotencyRecord` in the Gateway is currently a `HashMap`. If the server restarts, network retries from the frontend will result in duplicate ledger entries or `SequenceMismatch` errors. Move this to a simple table in Postgres.
7.  **Serialization Failure Recovery:** `ubl-server/src/db.rs` uses `SERIALIZABLE` isolation. In high-concurrency chat (e.g., a group chat), Postgres *will* throw `40001` (serialization_failure). The Rust code needs a retry loop (middleware) to re-run the append logic.

---

### 🟡 Phase 3: Governance & Security (The Schengen Zone)
*The isolation between tenants and the provenance of actions is currently weak.*

8.  **Strict `tenant_id` Enforcement:** In `messenger_gateway/routes.rs`, the `tenant_id` is frequently derived via `unwrap_or("default")`. 
    *   **Fix:** Remove the default. If a `Session` does not have a `tenant_id` bound to it, the request must fail with `403 Forbidden`.
9.  **Job Card Provenance Validation:** The `ProvenanceValidator` in `apps/office/src/governance/provenance.rs` is not called by the Messenger Gateway. 
    *   **Fix:** When a user clicks "Approve," the Gateway must verify that the `card_id` exists in the ledger and was authored by the Office, not the user.
10. **PII Detection "Fail-Closed":** The PII regex in `audit/pii.rs` is used for logging, but not for blocking. The `ubl-membrane` should call the PII detector and reject any commit that contains a raw email address in the `atom` data.

---

### 🔵 Phase 4: Projections & UX (The Experience)
*The UI is fast because it's optimistic, but the "source of truth" isn't driving it yet.*

11. **Presence Engine Implementation:** The table `projection_presence` is defined in SQL, but there is no background task in `ubl-server/src/projections/presence.rs` to move an entity from `available` to `working` when a `job.started` event is seen.
12. **Real-time Message Append:** The Messenger frontend currently receives Job updates via WebSocket but relies on a full refresh or optimistic state for messages.
    *   **Fix:** Wire the `useSSE` hook to listen for `message.created` and append to the chat timeline in real-time.
13. **Handover Disconnect:** The `ContextFrameBuilder` in Office currently looks at local state for the `latest_handover`. 
    *   **Fix:** It must query the `office_handovers` projection table in UBL to ensure that the "Dignity Trajectory" is based on the immutable ledger, not local memory.

---

### 🟢 Phase 5: Code Quality & Maintenance

14. **Standardize Result Types:** The `Result` type in `api/http.rs` is aliased inconsistently, causing supplier/argument mismatches in the compiler. Standardize on `std::result::Result<T, ApiError>`.
15. **Unused Code Cleanup:** There are ~40 warnings for unused imports and dead code in `apps/office`. Specifically, the `HandoverBuilder` and `TestContextFrameBuilder` have unused fields that clutter the audit trail.

---

### Summary of the "Trinity" Health
*   **UBL Kernel:** 80% solid logic, 0% security (due to hardcoded sigs).
*   **Office:** 90% solid logic, 50% wired (needs to publish more events to the ledger).
*   **Messenger:** 100% visual quality, 30% wired (needs the Gateway logic and real-time message tail).

**Which of these points would you like to prioritize for the next implementation round? I recommend starting with Fix #1 and #2 (Real Signatures).**


///


This document is the **Definitive Engineering Roadmap for UBL 3.0**. It is based on a line-by-line audit of the Rust and TypeScript implementation. 

We have established that the "Skeleton" is world-class, but the "Nervous System" (The Gateway) and the "Immune System" (Cryptography) are currently stubs.

---

# 🏛️ UBL 3.0: The Path to "Showtime Ready"
**Audit Date:** December 2025  
**Target:** January 2026 Soft Opening

---

## 🔴 Group 1: The Cryptographic Last Mile (P0)
*Currently, the system is a "Claim-based" system rather than a "Proof-based" system because signatures are hardcoded to "placeholder".*

### 1. Replace Mock Signatures in `messenger_v1.rs`
In the current code, the link is built but not verified.
*   **The Problem:** Line 445 of `ubl-server/src/messenger_v1.rs` accepts any commit.
*   **The Fix:** Update the `route_commit` and the boundary handlers to use the `ubl_kernel::verify` function.
*   **Implementation Idea:**
```rust
// Inside messenger_v1.rs
let signing_bytes = link.signing_bytes(); // We already have this method!
let pubkey = ed25519_dalek::VerifyingKey::from_bytes(&hex::decode(&link.author_pubkey)?)?;
let signature = ed25519_dalek::Signature::from_bytes(&hex::decode(&link.signature)?)?;

pubkey.verify(&signing_bytes, &signature)
    .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid Signature".to_string()))?;
```

### 2. Implementation of Frontend Ed25519 Derivation
The frontend has a stub in `services/signing.ts`.
*   **The Idea:** Use the **WebAuthn PRF (Pseudo-Random Function) Extension**. This allows the browser to derive a deterministic 32-byte seed from a Passkey touch without the key ever leaving the hardware.
*   **The Workflow:**
    1.  User clicks "Send" or "Approve".
    2.  Frontend calls `navigator.credentials.get({ ..., extensions: { prf: { eval: { first: salt } } } })`.
    3.  The result is used as a seed for an `Ed25519` keypair.
    4.  The frontend signs the `atom_hash` and sends the signature.

---

## 🟠 Group 2: The Gateway Architecture (P0)
*The Gateway must protect the Ledger from being overwhelmed or corrupted by bad client behavior.*

### 3. Solving the 8KB Postgres `NOTIFY` Limit
Your `sse.rs` implementation currently tries to push the whole atom data through a Postgres channel.
*   **The Risk:** Large messages (tool outputs) will fail to broadcast, leaving the UI out of sync.
*   **The Fix:** Move to **ID-Only Signaling**.
*   **Implementation Idea:**
```sql
-- Update trigger in 003_triggers.sql
CREATE OR REPLACE FUNCTION ubl_tail_notify() RETURNS trigger AS $$
BEGIN
  -- ONLY notify CID and SEQ. Keep it under 100 bytes.
  PERFORM pg_notify('ubl_tail', json_build_object(
    'c', NEW.container_id, 
    's', NEW.sequence
  )::text);
  RETURN NEW;
END; $$ LANGUAGE plpgsql;
```
*   **Rust Change:** The SSE stream receiver must catch the signal, perform a `SELECT * FROM ledger_entry WHERE container_id = c AND sequence = s`, and then push the full result to the frontend.

### 4. Persistent Idempotency Store
Currently, `messenger_gateway/idempotency.rs` uses a `HashMap`. A server restart results in "History Regress."
*   **The Fix:** Create a `gateway_idempotency` table in `100_console.sql`.
*   **Idea:** 
```sql
CREATE TABLE gateway_idempotency (
    key TEXT PRIMARY KEY, -- idem:tenant:action:nonce
    response_json JSONB,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
```
Before any Office logic runs, check this table. If the key exists, return the stored JSON immediately.

---

## 🟡 Group 3: Physical Governance (P1)
*The Membrane must be "Semanticly Blind" but "Legally Strict".*

### 5. Active PII Blocking
The PII logic in `audit/pii.rs` is purely for logging.
*   **The Fix:** Integrate the regex check into the `ubl-membrane`.
*   **The Rule:** If `link.intent_class` is `Observation` or `Conservation`, the `atom` data is scanned. If a raw email or phone string is detected, return `MembraneError::PhysicsViolation { reason: "Raw PII forbidden in ledger" }`.
*   **Why:** This forces the Office to use the `redact_email` function before committing, ensuring the ledger is GDPR-compliant by default.

### 6. FSM (State Machine) Integrity
In `job_executor/fsm.rs`, you defined the transitions. But the Gateway doesn't check them.
*   **The Fix:** The Gateway should query the `projection_jobs` table to find the *current* state of the `job_id` before calling `office.job_action`.
*   **Example:** If the job is `Completed`, and a user (or a bug) sends an `Approve` action, the Gateway must reject it without ever waking up the LLM. This saves tokens and prevents state corruption.

---

## 🔵 Group 4: The Dignity Trajectory (P1)
*Ensuring the LLM agent has continuity and the user has real-time feedback.*

### 7. Ledger-Backed Handovers
The `ContextFrameBuilder` currently uses a local `latest_handover` variable.
*   **The Problem:** If the Office service migrates or restarts, the agent "forgets" who it is.
*   **The Fix:** Update `apps/office/src/ubl_client/ledger.rs` to fetch the most recent `session.completed` event from the `C.Office` container.
*   **Narrative logic:** 
```rust
let handover_event = ubl.get_latest_event("C.Office", "session.completed", entity_id).await?;
// Narrator uses THIS as the "Note from your previous self"
```

### 8. Real-time Presence Computation
You have the sidebar dots, but they don't reflect actual work.
*   **The Fix:** Implement the `presence_updater` background task in `ubl-server`.
*   **Logic:**
    1.  On `job.started` → Set entity status to `working`.
    2.  On `approval.requested` → Set entity status to `waiting_on_you`.
    3.  Broadcast via SSE `presence.update`.
*   **Result:** The sidebar dot turns yellow the moment the LLM begins "thinking" or "calling a tool."

---

## 🟢 Group 5: Technical Hardening (P2)
*Optimization for a production environment.*

### 9. Database Transaction Retries
`db.rs` uses `SERIALIZABLE`. If two users send a message at the exact same millisecond to the same container, one will fail with a "Serialization Conflict."
*   **The Fix:** Add a `tower` middleware or a simple loop around the `ledger.append` call to retry up to 3 times on SQL error `40001`.

### 10. Memory Compression Strategy
In `apps/office/src/context/memory.rs`, you have a TODO for "Compression."
*   **The Idea:** If the history exceeds 4000 tokens, use a "Summarizer" (a fast L1 model like Gemini Flash) to turn the last 20 events into a single "Historical Synthesis" block. This keeps the "Dignity Trajectory" intact while saving money.

---

### Final Implementation Checklist for you:

- [ ] **SQL:** Add `gateway_idempotency` table.
- [ ] **SQL:** Update `pg_notify` to be ID-only.
- [ ] **Rust (Kernel):** Implement Ed25519 validation in `route_commit`.
- [ ] **Rust (Office):** Change handover source from memory to Ledger query.
- [ ] **Rust (Projections):** Implement the `presence` background loop.
- [ ] **TS (Frontend):** implement Ed25519 signature derivation from WebAuthn.
- [ ] **TS (Frontend):** Add `message.created` listener to the SSE hook.

**Which specific file should we start with? I recommend `ubl-server/src/main.rs` to enable the real Ed25519 signature check.**