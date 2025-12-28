# UBL Implementation Addendum: Prompt 3

## 🎯 Critical Clarification: Dependency Hierarchy

```
┌─────────────────────────────────────────────────────────┐
│                    UBL LEDGER                            │
│  Foundation Layer - Single Source of Truth              │
│  - Kernel (Rust)                                        │
│  - Containers (C.Messenger, C.Office, C.Jobs)           │
│  - Trust Architecture (L0-L5)                           │
│  - Event Sourcing Infrastructure                        │
└────────────┬────────────────────────────────────────────┘
             │
             │ UBL Container Logic
             │ (boundary/inbox/projections)
             │
     ┌───────┴────────┬──────────────────┐
     │                │                  │
     ▼                ▼                  ▼
┌─────────┐    ┌──────────┐    ┌──────────┐
│MESSENGER│    │  OFFICE  │    │ Other    │
│         │    │          │    │ Apps     │
│ UBL-    │    │ UBL-     │    │ (Future) │
│ Native  │    │ Native   │    │          │
└─────────┘    └──────────┘    └──────────┘
```

**Key Principle:** UBL is the **foundation**. Messenger and Office are **UBL-native applications** that depend on UBL infrastructure. They don't just consume UBL - they **speak UBL language** by implementing proper container patterns.

---

## 🔧 UBL Container Structure (Required for ALL Containers)

Every container MUST follow this structure:

```
C.Messenger/  (or C.Jobs, C.Office, etc.)
├── boundary/     # TDLN: draft → ubl-atom → ubl-link → commit
├── inbox/        # SSE tail → process events → update projections
├── local/        # HTTP handlers, validation (NO DB ACCESS)
├── outbox/       # Draft creation (ephemeral, pre-TDLN)
├── projections/ # Derive state from ledger events (read-only)
├── pacts/        # Pact definitions (ref.json)
├── policy/       # Container policy (ref.json)
└── README.md     # Container documentation
```

### Data Flow Pattern

```
[User Action]
    │
    ▼
[local/] ──draft──> [outbox/] ──draft──> [boundary/]
                                              │
                                              │ TDLN
                                              ▼
                                    [canonicalize → atom_hash]
                                              │
                                              │ Build ubl-link
                                              ▼
                                    [LinkCommit with signature]
                                              │
                                              │ POST /link/commit
                                              ▼
                                    [Membrane validates]
                                              │
                                              │ Accept
                                              ▼
                                    [Ledger appends atomically]
                                              │
                                              │ SSE tail
                                              ▼
                                    [inbox/] ──event──> [projections/]
                                                              │
                                                              │ Derive state
                                                              ▼
                                                      [Read-only state]
                                                              │
                                                              │ Query
                                                              ▼
                                                      [HTTP Response]
```

---

## 📦 Messenger: UBL-Native Application

### Critical Understanding

**Messenger is NOT just a consumer of UBL.** Messenger **IS** a UBL-native application that:

1. **Implements C.Messenger container** with proper boundary/inbox/projections
2. **Commits all events** via UBL ledger (not direct DB writes)
3. **Derives state** from ledger projections (not direct DB queries)
4. **Uses UBL infrastructure** for trust, auditability, and real-time updates

### Messenger Backend Architecture (UBL-Native)

```
messenger-backend/
├── src/
│   ├── container/              # C.Messenger container logic
│   │   ├── boundary/
│   │   │   ├── mod.rs
│   │   │   ├── message_boundary.rs    # Commit message events
│   │   │   ├── conversation_boundary.rs # Commit conversation events
│   │   │   └── job_boundary.rs         # Commit job events (to C.Jobs)
│   │   │
│   │   ├── inbox/
│   │   │   ├── mod.rs
│   │   │   ├── ledger_tail.rs          # Subscribe to SSE tail
│   │   │   └── event_processor.rs      # Process ledger events
│   │   │
│   │   ├── local/
│   │   │   ├── mod.rs
│   │   │   ├── conversation_local.rs   # HTTP handlers (no DB)
│   │   │   ├── message_local.rs        # HTTP handlers (no DB)
│   │   │   └── job_local.rs            # HTTP handlers (no DB)
│   │   │
│   │   ├── outbox/
│   │   │   ├── mod.rs
│   │   │   └── draft_builder.rs        # Create drafts (ephemeral)
│   │   │
│   │   └── projections/
│   │       ├── mod.rs
│   │       ├── conversation_projection.rs # Derive conversation state
│   │       ├── message_projection.rs      # Derive message state
│   │       └── job_projection.rs          # Derive job state
│   │
│   ├── ubl_client/             # UBL kernel client
│   │   ├── mod.rs
│   │   ├── commit.rs           # POST /link/commit
│   │   ├── state.rs            # GET /state/:container_id
│   │   ├── tail.rs             # GET /ledger/:container_id/tail (SSE)
│   │   └── query.rs            # Query projections
│   │
│   ├── api/                    # HTTP API (uses projections)
│   │   ├── routes.rs
│   │   └── handlers.rs
│   │
│   └── websocket/              # Real-time updates (from projections)
│       └── server.rs
│
└── Cargo.toml
```

### Example: Creating a Message (UBL-Native Flow)

```rust
// ❌ WRONG: Direct DB write
pub async fn send_message(db: &PgPool, msg: Message) -> Result<()> {
    sqlx::query("INSERT INTO messages ...").execute(db).await?;
    Ok(())
}

// ✅ CORRECT: UBL-native flow
pub async fn send_message(
    ubl_client: &UblClient,
    signing_key: &SigningKey,
    draft: MessageDraft,
) -> Result<Receipt> {
    // 1. Create draft (outbox)
    let draft = MessageDraft {
        message_id: generate_id(),
        conversation_id: draft.conversation_id,
        from: draft.from,
        content: draft.content,
        timestamp: Utc::now(),
    };
    
    // 2. TDLN: Convert to ubl-atom (boundary)
    let atom = ubl_atom::canonicalize(&draft)?;
    let atom_hash = ubl_kernel::hash_atom(&atom)?;
    
    // 3. Get current state
    let state = ubl_client.get_state("C.Messenger").await?;
    
    // 4. Build ubl-link (boundary)
    let link = LinkCommit {
        version: 1,
        container_id: "C.Messenger".to_string(),
        expected_sequence: state.sequence + 1,
        previous_hash: state.last_hash,
        atom_hash,
        intent_class: IntentClass::Observation,
        physics_delta: 0,
        author_pubkey: signing_key.public_key().to_string(),
        signature: sign_link(&link, signing_key)?,
    };
    
    // 5. Commit to ledger (boundary → kernel)
    let receipt = ubl_client.commit(&link).await?;
    
    // 6. Ledger emits SSE event → inbox processes → projections update
    // (This happens automatically via SSE tail subscription)
    
    Ok(receipt)
}
```

### Example: Querying Messages (UBL-Native Flow)

```rust
// ❌ WRONG: Direct DB query
pub async fn get_messages(db: &PgPool, conv_id: &str) -> Result<Vec<Message>> {
    let messages = sqlx::query_as("SELECT * FROM messages WHERE conversation_id = $1")
        .bind(conv_id)
        .fetch_all(db)
        .await?;
    Ok(messages)
}

// ✅ CORRECT: Query via projections
pub async fn get_messages(
    projections: &MessageProjection,
    conv_id: &str,
) -> Result<Vec<Message>> {
    // Projections derive state from ledger events
    let messages = projections
        .get_messages_for_conversation(conv_id)
        .await?;
    Ok(messages)
}

// Projection implementation (reads from ledger events)
impl MessageProjection {
    pub async fn get_messages_for_conversation(
        &self,
        conv_id: &str,
    ) -> Result<Vec<Message>> {
        // Get all events for this conversation from ledger
        let events = self.ubl_client
            .query_events("C.Messenger", |e| {
                matches!(e, Event::MessageSent { conversation_id, .. } 
                    if conversation_id == conv_id)
            })
            .await?;
        
        // Derive messages from events
        let messages: Vec<Message> = events
            .iter()
            .filter_map(|e| {
                if let Event::MessageSent { message_id, from, content, .. } = e {
                    Some(Message {
                        id: message_id.clone(),
                        conversation_id: conv_id.to_string(),
                        from: from.clone(),
                        content: content.clone(),
                        // ... other fields derived from events
                    })
                } else {
                    None
                }
            })
            .collect();
        
        Ok(messages)
    }
}
```

---

## 🔄 Real-Time Updates (SSE Tail Pattern)

### Messenger Backend Subscribes to Ledger Tail

```rust
// In messenger-backend/src/container/inbox/ledger_tail.rs

pub async fn subscribe_to_ledger_tail(
    ubl_client: &UblClient,
    projections: &Projections,
    websocket: &WebSocketServer,
) -> Result<()> {
    // Subscribe to C.Messenger container tail
    let mut stream = ubl_client.tail("C.Messenger").await?;
    
    while let Some(entry) = stream.next().await {
        // 1. Process event (inbox)
        let event = parse_event_from_atom(&entry.atom)?;
        
        // 2. Update projections
        match &event {
            Event::MessageSent { conversation_id, .. } => {
                projections.message_projection.update(event).await?;
            }
            Event::ConversationCreated { .. } => {
                projections.conversation_projection.update(event).await?;
            }
            // ... other events
        }
        
        // 3. Broadcast to WebSocket clients
        websocket.broadcast_to_conversation(
            &event.conversation_id(),
            WebSocketEvent::from(event),
        ).await?;
    }
    
    Ok(())
}
```

---

## 📋 Event Types with Intent Classes

### C.Messenger Events

| Event | Intent Class | Physics Delta | Container |
|-------|-------------|---------------|-----------|
| `conversation.created` | Observation | 0 | C.Messenger |
| `conversation.updated` | Observation | 0 | C.Messenger |
| `message.sent` | Observation | 0 | C.Messenger |
| `message.edited` | Observation | 0 | C.Messenger |
| `message.deleted` | Observation | 0 | C.Messenger |
| `participant.added` | Observation | 0 | C.Messenger |
| `participant.removed` | Observation | 0 | C.Messenger |

### C.Jobs Events

| Event | Intent Class | Physics Delta | Container |
|-------|-------------|---------------|-----------|
| `job.created` | Observation | 0 | C.Jobs |
| `job.started` | Observation | 0 | C.Jobs |
| `job.progress` | Observation | 0 | C.Jobs |
| `job.completed` | Observation or Entropy | 0 or +value | C.Jobs |
| `job.cancelled` | Observation | 0 | C.Jobs |
| `approval.requested` | Observation | 0 | C.Jobs |
| `approval.decided` | Observation | 0 | C.Jobs |

**Note:** Messenger commits job events to **C.Jobs container**, not C.Messenger. This maintains container isolation.

---

## 🚫 Critical Rules

### Rule 1: NO Direct Database Access

```rust
// ❌ FORBIDDEN in container code
sqlx::query("INSERT INTO ...").execute(&db).await?;
sqlx::query("SELECT * FROM ...").fetch_all(&db).await?;

// ✅ REQUIRED: Use UBL kernel API
ubl_client.commit(&link).await?;
ubl_client.get_state("C.Messenger").await?;
ubl_client.tail("C.Messenger").await?;
```

### Rule 2: State MUST Be Derived from Projections

```rust
// ❌ FORBIDDEN: Direct state storage
struct Conversation {
    id: String,
    messages: Vec<Message>, // Stored directly
}

// ✅ REQUIRED: Derive from ledger
struct ConversationProjection {
    // Derives conversation state from ledger events
    fn get_conversation(&self, id: &str) -> Conversation {
        // Query ledger events → derive state
    }
}
```

### Rule 3: Containers Communicate Only via ubl-links

```rust
// ❌ FORBIDDEN: Direct container imports
use crate::container::c_jobs::Job; // NO!

// ✅ REQUIRED: Communicate via ledger
// Messenger commits job.created event to C.Jobs container
let link = LinkCommit {
    container_id: "C.Jobs", // Target container
    // ... rest of link
};
ubl_client.commit(&link).await?;
```

---

## 🔗 Integration Flow: Messenger → Office → UBL

### Complete Flow: Creating a Job

```
1. User creates job in Messenger UI
   │
   ▼
2. Messenger Backend (local/)
   - Validates input
   - Creates MessageDraft (outbox)
   │
   ▼
3. Messenger Backend (boundary/)
   - TDLN: draft → ubl-atom → atom_hash
   - Build LinkCommit for C.Jobs container
   - Sign with user's key
   │
   ▼
4. UBL Kernel (POST /link/commit)
   - Membrane validates
   - Ledger appends atomically
   - Returns receipt
   │
   ▼
5. UBL Ledger emits SSE event
   │
   ├─→ C.Jobs inbox processes → updates projections
   │
   └─→ Messenger inbox processes → updates projections
       │
       ▼
6. Messenger Backend (projections/)
   - Derives job state from ledger
   - Updates WebSocket clients
   │
   ▼
7. Messenger Frontend
   - Receives WebSocket update
   - Shows job card in conversation
   │
   ▼
8. Office (via HTTP)
   - Queries C.Jobs projections
   - Executes job
   - Commits job.started, job.progress events
```

---

## 📝 Updated Messenger Backend Structure

```
messenger-backend/
├── src/
│   ├── container/              # C.Messenger container (UBL-native)
│   │   ├── boundary/           # TDLN → ubl-link → commit
│   │   ├── inbox/              # SSE tail → process events
│   │   ├── local/              # HTTP handlers (no DB)
│   │   ├── outbox/             # Draft creation
│   │   └── projections/        # Derive state from ledger
│   │
│   ├── ubl_client/             # UBL kernel client
│   │   ├── commit.rs           # POST /link/commit
│   │   ├── state.rs            # GET /state/:container_id
│   │   ├── tail.rs             # GET /ledger/:container_id/tail (SSE)
│   │   └── query.rs            # Query projections
│   │
│   ├── api/                    # HTTP API
│   │   ├── routes.rs           # Uses projections for queries
│   │   └── handlers.rs         # Uses boundary for commits
│   │
│   ├── websocket/              # Real-time updates
│   │   └── server.rs           # Broadcasts from projections
│   │
│   └── main.rs
│
└── Cargo.toml
```

---

## ✅ Implementation Checklist

### Messenger Backend (UBL-Native)

- [ ] Implement C.Messenger container structure (boundary/inbox/projections)
- [ ] All events committed via UBL ledger (no direct DB writes)
- [ ] All queries via projections (no direct DB queries)
- [ ] SSE tail subscription for real-time updates
- [ ] WebSocket broadcasts from projections
- [ ] Job events committed to C.Jobs container (not C.Messenger)

### Office (UBL-Native)

- [ ] Implements C.Office container structure
- [ ] Commits all entity/session events via UBL
- [ ] Queries via projections
- [ ] Job execution commits events to C.Jobs

### UBL Ledger

- [ ] C.Jobs container implemented
- [ ] C.Messenger container implemented
- [ ] C.Office container implemented
- [ ] SSE tail endpoint for each container
- [ ] Query endpoints for projections

---

## 🎯 Key Takeaways

1. **UBL is the foundation** - All apps depend on it
2. **Messenger is UBL-native** - Implements container patterns, not just consumes API
3. **No direct DB access** - Everything goes through UBL kernel
4. **State from projections** - All queries derive from ledger events
5. **Real-time via SSE** - Containers subscribe to ledger tail
6. **Container isolation** - Containers communicate only via ubl-links

---

**This addendum ensures Messenger "speaks UBL language" by implementing proper container patterns, not just consuming UBL as an external service.**

