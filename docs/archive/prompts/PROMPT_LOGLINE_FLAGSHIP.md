FULL PROMPT — LogLine Flagship: UBL Messenger + Office + UBL Ledger
===================================================================

You are building the **flagship product trio** of LogLine:

1.  **UBL Messenger** — a functioning, beautiful WhatsApp-like frontend for SMBs where **humans and AI agents appear as coworkers**.
2.  **Office** — a robust LLM runtime infrastructure that makes the simple Messenger experience possible by providing the **right tools + right information + right format + right timing**, and guarantees “**trajectory of dignity at work**” for temporary LLM workers.
3.  **UBL Ledger + Container Logic** — the **mandatory** append-only truth layer and computable governance substrate used by both Messenger and Office.

These three pieces must be **separated**, but designed from day 1 to be wired together through APIs and event streams (REST + SSE + WS). No “tight coupling through shared memory.” Everything important becomes ledger events.

Non-negotiable principles
-------------------------

*   **UBL-first truth:** Any meaningful state is derivable from the ledger. Projections/caches are allowed but must be rebuildable.
*   **Everything has an ID:** `trace_id`, `tenant_id`, `conversation_id`, `entity_id`, and especially `job_id`.
*   **Jobs are formal:** Work is tracked as formal jobs with lifecycle events.
*   **Cards are the UX for jobs:** Cards formalize, track, and close jobs.
*   **Chat never freezes:** Cards never block conversation. Chat remains fully available during jobs.
*   **Human and agent parity:** Humans can also use job cards and buttons. It’s not “agent-only.”
*   **Governance is computable:** Policies live as code/contract logic. Decisions are recorded.
*   **Dignity trajectory:** Agents have continuity—context, identity, commitments, and accountability across messages/jobs—without leaking private tenant data.

* * *

A) UBL Messenger — Product requirements
=======================================

Core experience
---------------

Build a WhatsApp-style UI:

*   Left: conversation list with avatars, last message snippet, unread badge
*   Top: active conversation header (participant name, status)
*   Center: message timeline (human + agent messages)
*   Bottom: composer with attachments + quick actions

### Coworkers model

Conversations can include:

*   **Human coworkers** (employees, customers, vendors)
*   **Agent coworkers** (LLM workers with roles like “Ops Assistant”, “Billing Agent”, “Scheduler Agent”)

Agents must look like coworkers:

*   Profile: name, role, capability badges
*   Presence states: `available`, `working`, `waiting_on_you`, `offline`
*   “Working…” indicator tied to a job

Job card system (mandatory)
---------------------------

Every job has `job_id` and appears in chat as a **card**. There are three card phases:

1.  **Formalize card** (job proposal / job draft)
    *   Fields: `job_id`, title, goal, proposed owner (human/agent), inputs needed, expected outputs, constraints, SLA/priority
    *   Must include buttons:
        *   **Agree / Approve**
        *   **Disagree / Reject**
        *   **Request changes**
        *   **Ask in chat** (never blocks; opens chat with a prefilled question)
    *   Must link to ledger-backed job creation event
2.  **Tracking card** (in progress)
    *   Fields: `job_id`, current status, step list, latest event, blockers, “waiting on”
    *   Must include buttons:
        *   **Confirm / Acknowledge**
        *   **Dispute / Escalate**
        *   **Provide missing input**
        *   **Pause / Cancel**
        *   **Ask in chat**
3.  **Finished card** (done / failed / cancelled)
    *   Fields: `job_id`, outcome summary, artifacts produced (files/links), next recommended action
    *   Must include buttons:
        *   **Accept outcome**
        *   **Dispute**
        *   **Create follow-up job**
        *   **Ask in chat**

### Cards are not the only messages

Regular chat messages continue normally and can happen between card events.

Messages: types
---------------

Messenger must support message types:

*   `text`
*   `rich_card` (job cards)
*   `artifact` (links, files, receipts, quotes)
*   `system_event` (join/leave, policy notice)

SMB readiness
-------------

*   Multi-tenant: every action scoped by `tenant_id`
*   Contacts imported/created per tenant
*   Simple onboarding: “Create workspace → invite coworker → add agent coworker”
*   Clean professional vibe: “small business-grade,” not toy.

UI must be event-driven
-----------------------

Messenger is a projection surface:

*   It **subscribes** to job/message updates via **SSE or WebSocket**
*   It **sends commands** via REST or WS
*   It can cache locally but must reconcile with event stream

* * *

B) Office — Runtime requirements
================================

Office is the **LLM worker infrastructure** that makes Messenger feel simple and reliable.

Office responsibilities
-----------------------

Office must:

1.  Turn messages + context into **structured intent**
2.  Decide whether to:
    *   reply normally in chat
    *   propose a job
    *   continue a job
    *   request missing inputs
    *   escalate to a human
3.  Run jobs as a **controlled state machine**
4.  Use tools safely and record tool usage
5.  Emit **job events** and **message outputs** as ledger spans
6.  Enforce governance and policy gates

“Trajectory of dignity”
-----------------------

Office must maintain a coherent work trajectory for each agent:

*   Agent identity: `entity_id`, role, allowed tools, memory scope
*   Work continuity: job histories, decisions, constraints, approvals
*   Explainability: short rationale stored as event metadata (not long essays)
*   Accountability: who approved what, when, and why
*   Boundaries: never exceed tenant scope; never hallucinate authority; never fabricate completed work

Office architecture (conceptual modules)
----------------------------------------

Office should be built as modules that can evolve:

*   **Context Builder:** assembles “frame” for the LLM (tenant, conversation, job state, affordances)
*   **Narrator:** turns frame into a short, consistent system narrative
*   **Planner:** identifies intents and job candidates
*   **Governance Gate:** policies/constraints checks
*   **Tool Router:** executes allowed tools, records traces
*   **Job Engine:** job lifecycle + state transitions + idempotency
*   **Emitter:** writes all outputs to UBL ledger + pushes to streams

Office must support two output modes
------------------------------------

*   **Chat reply:** plain message (fast)
*   **Job mode:** propose/update/close job cards (structured)

Job state machine
-----------------

Minimum states:

*   `draft` → `proposed` → `approved` → `in_progress` → (`waiting_input` ↔ `in_progress`) → `completed`
*   Failure exits: `rejected`, `cancelled`, `failed`

Every transition must be recorded as a ledger event with:

*   `job_id`
*   `actor` (human/agent/system)
*   `prev_state`, `next_state`
*   `reason_code`
*   optional `artifact_refs`

* * *

C) UBL Ledger + Container Logic — Mandatory substrate
=====================================================

Ledger semantics
----------------

*   Append-only events (“spans” / NDJSON / JSON✯Atomic)
*   Deterministic hashing + signing
*   Idempotency: every command has an idempotency key
*   Observability: trace IDs across all services

Container logic role
--------------------

*   Validation of events against schemas/contracts
*   Policy enforcement hooks (pre/post)
*   Projection support (build queryable views)
*   Minimal “computable governance” primitives (rate limits, access control, approval rules)

Shared event taxonomy (minimum)
-------------------------------

Define a stable schema for:

*   `message.sent`
*   `message.delivered`
*   `conversation.created`
*   `entity.registered`
*   `job.created`
*   `job.proposed`
*   `job.approved`
*   `job.state_changed`
*   `job.artifact_attached`
*   `job.completed`
*   `policy.violation`
*   `tool.called`
*   `tool.result`

Both Messenger and Office must write and read from this taxonomy.

* * *

D) Wiring: the three pieces and their interfaces
================================================

Separation of concerns
----------------------

*   **Messenger**: UI + thin gateway API (auth/session, subscriptions, posting user actions)
*   **Office**: intelligence runtime + job engine + tool execution + governance
*   **UBL**: ledger + verification + container logic + projections

No piece should require the others’ internal DB schemas. Communication happens through:

*   REST commands
*   SSE/WS streams
*   Ledger reads (query projections)
*   Ledger writes (append events)

API and streaming contracts
---------------------------

### 1) Messenger Gateway API (used by frontend)

Commands (REST or WS):

*   `POST /v1/conversations`
*   `POST /v1/conversations/{id}/messages` (text or card actions)
*   `POST /v1/jobs/{job_id}/actions` (approve/reject/provide\_input/cancel)
*   `GET /v1/conversations/{id}`
*   `GET /v1/conversations/{id}/messages`
*   `GET /v1/jobs/{job_id}`

Streams:

*   `GET /v1/stream` (SSE): emits conversation updates, message updates, job updates
*   or `WS /v1/ws`

Messenger gateway does NOT decide job logic. It forwards to Office and persists to UBL.

### 2) Office API (used by Messenger gateway)

Commands:

*   `POST /v1/office/ingest_message`  
    Input: tenant, conversation, sender entity, message payload, trace\_id  
    Output: either chat reply event or job card event(s)
*   `POST /v1/office/job_action`  
    Approve/reject/inputs; returns new job state + next card
*   `GET /v1/office/affordances?entity_id=...&tenant_id=...`  
    Returns tool/action capabilities for UI hints

Office must always emit outputs as UBL events; responses can be “ack + ids.”

### 3) UBL API

*   Append events (validated)
*   Query projections by tenant/conversation/job
*   Stream ledger tail or projection updates (SSE/WS)

* * *

E) UX rules for cards + buttons
===============================

Buttons are always present
--------------------------

Every card must have **explicit options**. Default minimum:

*   Approve / Reject
*   Ask in chat (opens chat)
*   Provide input (opens input UI)
*   Escalate (marks dispute + opens chat)

Card actions are messages too
-----------------------------

When a user presses a button:

*   Messenger emits a normal “action message” in the chat timeline (readable)
*   System also emits the formal action event to ledger

Chat-first conflict resolution
------------------------------

If the user rejects/disputes:

*   Office must ask clarifying questions in chat
*   It may re-propose an updated job card

* * *

F) Security, tenancy, and identity
==================================

Tenancy
-------

Every request includes `tenant_id`. Strict isolation.

*   No cross-tenant memory
*   No cross-tenant projections
*   Agent identity is tenant-scoped (even if “same model,” it’s a new entity per tenant)

Identity
--------

Entities:

*   `human`: user accounts, coworkers, customers
*   `agent`: Office-managed LLM workers

Every action is attributed:

*   `actor_entity_id`
*   `actor_type`
*   `auth_context` (session, device)

* * *

G) Acceptance criteria (what “done” looks like)
===============================================

End-to-end flow must work
-------------------------

1.  Human messages an agent coworker
2.  Office proposes a job via a formalize card
3.  Human approves with one button
4.  Office runs the job, posts tracking updates as cards
5.  Office posts a finished card with artifact(s)
6.  Human accepts or disputes
7.  Entire flow is reconstructible from UBL ledger events

Observability
-------------

*   Every request has `trace_id`
*   You can follow trace across Messenger → Office → UBL events
*   Job timeline is queryable by `job_id`

No freezing
-----------

At every point, user can continue chatting even while job runs.

* * *

H) Build constraints (how to keep it sane)
==========================================

*   Start minimal, but don’t break the architecture.
*   Projections are allowed; source-of-truth is ledger.
*   Prefer SSE for streaming first (WS optional later).
*   Keep UI beautiful but not at the cost of correctness.

* * *

Deliverable focus
=================

Your job is to design the system so the **three parts are separate**, but:

*   share the same event model
*   can be wired cleanly via APIs + streaming
*   and demonstrate the job-card lifecycle with approvals and dignity trajectory.

Do not handwave “later we’ll add ledger.” Ledger is mandatory now. Every job and card must reference a `job_id` and be backed by ledger events.

* * *

If you want, next I can give you the **tightest “v1 flagship slice”** (one measurable deliverable) that proves all three pieces are genuinely wired: **one agent coworker, one conversation, one job lifecycle, SSE updates, ledger reconstruction by job\_id**.

## Prompt:
YES

## Response:
Thought for 14s

Perfect. Here’s the **tightest v1 “flagship slice”** that proves the whole thesis: **Messenger UI + Office runtime + UBL ledger** are _separate_, _wired_, and **reconstructible by `job_id`**.

V1 Flagship Slice
-----------------

**Single measurable deliverable:**

> From the Messenger UI, a human messages an agent coworker → Office proposes a **Formalize Job Card** → user clicks **Approve** → Office emits **Tracking** updates → Office emits **Finished** card → you can reconstruct the whole job from UBL by `job_id`.

### Proof of Done

You have all 4 of these:

1.  UI shows **three cards** for the same `job_id`: `formalize → tracking → finished`
2.  Card buttons work: **Approve** changes the job state and triggers subsequent cards
3.  UI receives updates live via **SSE** (or WS) without refresh
4.  A single `GET /jobs/{job_id}` (or ledger query) returns the full ordered event chain for that job

* * *

The smallest “real wiring” architecture
=======================================

### Piece 1 — UBL (truth)

*   Accepts append-only events
*   Provides:
    *   **append** endpoint
    *   **query** by `tenant_id`, `conversation_id`, `job_id`
    *   **stream tail** (SSE/WS) for live updates

### Piece 2 — Office (brains + dignity trajectory)

*   Receives “user said X” and “user clicked Y”
*   Outputs:
    *   one of: normal chat reply **or** job-card event(s)
*   Writes every output to UBL

### Piece 3 — Messenger (beautiful WhatsApp clone)

*   UI never talks to Office directly
*   UI talks to a thin **Messenger Gateway**
*   Messenger Gateway:
    *   forwards inbound actions to Office
    *   reads UBL for history/projection
    *   streams updates to UI via SSE/WS

**Important:** the Messenger Gateway is part of “Messenger” (product surface). Office stays a separate runtime.

* * *

Minimal APIs (don’t add more than this)
=======================================

Messenger Gateway (what the UI uses)
------------------------------------

1.  `POST /v1/conversations/:id/messages`
    *   body: `{ tenant_id, actor_entity_id, text | card_action, trace_id }`
*   behavior: forward to Office ingest, return `{ accepted: true }`
    2.  `GET /v1/conversations/:id/timeline?tenant_id=...`
    *   returns message list + cards (projection)
    3.  `GET /v1/jobs/:job_id?tenant_id=...`
    *   returns the job event chain (projection)
    4.  `GET /v1/stream?tenant_id=...`
    *   SSE that emits: `message`, `job_update`, `presence`

Office (what Messenger Gateway uses)
------------------------------------

1.  `POST /v1/office/ingest_message`
    *   input: `{ tenant_id, conversation_id, actor_entity_id, text, trace_id }`
*   output: `{ emitted_event_ids: [...] }` (Office always writes to UBL)
    2.  `POST /v1/office/job_action`
    *   input: `{ tenant_id, job_id, actor_entity_id, action, payload?, trace_id }`
*   output: `{ emitted_event_ids: [...] }`

UBL (truth layer)
-----------------

1.  `POST /v1/ledger/append`
2.  `GET /v1/ledger/query?...`
3.  `GET /v1/ledger/stream?...` (SSE tail)

* * *

Minimal event model (this is the spine)
=======================================

All events share:

*   `event_id`, `ts`, `tenant_id`, `trace_id`
*   `actor_entity_id`, `conversation_id`
*   optional `job_id`

### Required event types for v1

1.  `message.sent`
2.  `job.created` _(creates `job_id`)_
3.  `job.proposed` _(Formalize Card payload)_
4.  `job.approved` _(from button click)_
5.  `job.state_changed` _(to in\_progress / waiting\_input / etc)_
6.  `job.progress` _(Tracking Card payload)_
7.  `job.completed` _(Finished Card payload)_

That’s it. Don’t overdesign v1.

* * *

UI requirements (minimal but “flagship-feel”)
=============================================

### Conversation list

*   Agent coworkers and humans in the same list
*   Agent rows show:
    *   role badge
    *   presence (“Working…” if a job is active)

### Timeline rendering

Render these three card components:

1.  **FormalizeCard(job\_id)** with buttons:
    *   Approve
*   Reject
*   Request changes
*   Ask in chat
    2.  **TrackingCard(job\_id)** with buttons:
    *   Acknowledge
*   Provide input
*   Dispute
*   Cancel
*   Ask in chat
    3.  **FinishedCard(job\_id)** with buttons:
    *   Accept outcome
*   Dispute
*   Follow-up job
*   Ask in chat

### Non-freezing rule

Even while TrackingCard exists:

*   input box stays active
*   user can continue chatting
*   card actions just emit events; they never lock UI

* * *

Office behavior (v1 “smart enough”)
===================================

For v1, Office can be simple and still legitimate:

*   If a message looks like work (“do X”, “please”, “need”, “can you”, “schedule”, “send”, “call”, etc.)
    *   Office emits a **job.proposed** with a tight Formalize payload
*   When approved:
    *   emit `job.state_changed → in_progress`
    *   emit 1–2 `job.progress` updates
    *   emit `job.completed` with a short outcome + artifact refs (even if mock artifact refs for v1)

**Key:** Office must be the only place that “decides” job transitions. Messenger just displays + forwards clicks.

* * *

Build checklist (one screen)
============================

1.  **Bring up UBL** (ledger API + stream)
2.  **Bring up Office** (ingest\_message + job\_action; writes to UBL)
3.  **Bring up Messenger Gateway** (timeline query + SSE stream; forwards to Office)
4.  **Point the WhatsApp-like UI** to the Messenger Gateway:
    *   send message → gateway → office → ubl
    *   receive SSE updates → render cards
5.  Run the golden path:
    *   message agent coworker
    *   approve Formalize card
    *   watch Tracking card update via SSE
    *   see Finished card
6.  Verify reconstruction:
    *   open `/v1/jobs/{job_id}` and confirm ordered event chain matches what UI showed

* * *

If you want, next message I’ll give you the **exact v1 payload shapes** for the 3 cards (the JSON contracts), so there’s zero ambiguity when wiring UI ↔ gateway ↔ Office ↔ UBL.

## Prompt:
YES

## Response:
Awesome. Here are the **exact v1 payload shapes** (contracts) for:

*   **Event envelope (UBL)**
*   **Three cards**: Formalize → Tracking → Finished
*   **Button actions** (what UI sends when clicked)
*   **Timeline items** (what Messenger Gateway returns to UI)

I’m giving this in **TypeScript-style shapes** (readable JSON contracts). You can enforce with JSON Schema later, but this is enough to wire today.

* * *

0) Shared primitives
--------------------

```ts
type ISO8601 = string;          // e.g. "2025-12-27T14:12:05.123Z"
type UUID = string;            // any stable ID; UUIDv7 recommended but not required
type TenantId = string;
type TraceId = string;
type ConversationId = string;
type EntityId = string;
type JobId = string;
type EventId = string;

type ActorType = "human" | "agent" | "system";

type Severity = "info" | "success" | "warning" | "error";

type JobState =
  | "draft"
  | "proposed"
  | "approved"
  | "in_progress"
  | "waiting_input"
  | "completed"
  | "rejected"
  | "cancelled"
  | "failed";
```

* * *

1) UBL Event Envelope (everything in the ledger looks like this)
----------------------------------------------------------------

```ts
type UblEvent<TType extends string, TPayload extends object> = {
  event_id: EventId;
  event_type: TType;              // e.g. "job.proposed"
  ts: ISO8601;

  tenant_id: TenantId;
  trace_id: TraceId;

  conversation_id: ConversationId;
  job_id?: JobId;                 // present for all job-* events
  actor: {
    entity_id: EntityId;
    actor_type: ActorType;
  };

  // Optional linking / ordering helpers:
  causation_id?: EventId;         // "this event was caused by that event"
  correlation_id?: string;        // group events (optional)
  seq?: number;                   // projection sequence number (optional)

  payload: TPayload;

  // Optional verification / signature placeholders (v1 can be empty):
  integrity?: {
    hash?: string;
    signature?: string;
    signer_pubkey?: string;
  };
};
```

* * *

2) Card message wrapper (how cards appear in the chat timeline)
---------------------------------------------------------------

Cards are emitted as a normal message event, but with `message.kind = "card"`.

```ts
type MessageSentPayload = {
  message_id: UUID;
  kind: "text" | "card" | "system";

  // timeline rendering
  body_text?: string;              // for kind="text" or "system"
  card?: JobCard;                  // for kind="card"

  // UX sugar (optional)
  preview?: {
    title?: string;
    subtitle?: string;
    severity?: Severity;
  };
};
```

So a **card in the UI** is always a `message.sent` event whose payload includes `card`.

* * *

3) JobCard union (three card types)
-----------------------------------

```ts
type JobCard = FormalizeCard | TrackingCard | FinishedCard;

type CardBase = {
  card_id: UUID;                   // unique per card instance
  job_id: JobId;
  card_type: "job.formalize" | "job.tracking" | "job.finished";
  version: "v1";

  title: string;                   // short: "Send invoice", "Schedule meeting"
  summary?: string;                // 1–2 lines max (UI-friendly)

  state: JobState;                 // aligns with job state at time of card
  created_at: ISO8601;

  // these two let UI keep chat open + contextual
  conversation_id: ConversationId;
  tenant_id: TenantId;

  // primary owner / assignee for the job
  owner: {
    entity_id: EntityId;
    display_name: string;
    actor_type: "human" | "agent";
  };

  // who created the card (agent/human/system)
  author: {
    entity_id: EntityId;
    display_name: string;
    actor_type: ActorType;
  };

  buttons: CardButton[];           // REQUIRED: always present
};

type CardButton = {
  button_id: UUID;                 // stable for this card instance
  label: string;                   // "Approve", "Reject", ...
  action: CardAction;              // what to send when clicked
  style?: "primary" | "secondary" | "danger";
  requires_input?: boolean;        // if true, UI opens input modal
  confirm?: {
    title: string;
    body?: string;
  };
};

type CardAction =
  | { type: "job.approve"; job_id: JobId }
  | { type: "job.reject"; job_id: JobId; reason_code?: string }
  | { type: "job.request_changes"; job_id: JobId }
  | { type: "job.provide_input"; job_id: JobId; input_schema?: InputSchema }
  | { type: "job.ack"; job_id: JobId }
  | { type: "job.dispute"; job_id: JobId; reason_code?: string }
  | { type: "job.cancel"; job_id: JobId }
  | { type: "chat.ask"; job_id?: JobId; prompt_text: string };
```

### InputSchema (minimal v1)

```ts
type InputSchema = {
  fields: Array<{
    key: string;                   // "invoice_number"
    label: string;                 // "Invoice number"
    type: "string" | "number" | "boolean" | "select" | "multiline";
    required?: boolean;
    options?: Array<{ value: string; label: string }>; // for select
    placeholder?: string;
  }>;
};
```

* * *

4) The three cards
==================

4.1) Formalize Card (job proposal / draft)
------------------------------------------

```ts
type FormalizeCard = CardBase & {
  card_type: "job.formalize";

  // job definition (tight + structured)
  job: {
    job_id: JobId;
    goal: string;                  // “Send invoice #123 to ACME”
    description?: string;          // optional detail, still short
    priority?: "low" | "normal" | "high" | "urgent";
    due_at?: ISO8601;

    inputs_needed?: Array<{
      key: string;
      label: string;
      status: "missing" | "provided";
      value_preview?: string;      // never large or sensitive
    }>;

    expected_outputs?: Array<{
      kind: "message" | "file" | "link" | "record";
      description: string;
    }>;

    constraints?: Array<string>;   // “Do not contact customer by phone”
    sla_hint?: string;             // “ETA ~10 minutes”
  };

  // OPTIONAL: what Office plans to do (very short)
  plan_hint?: Array<string>;       // e.g. ["Collect invoice number", "Generate PDF", "Send email"]
};
```

**Mandatory v1 buttons for Formalize**

*   Approve
*   Reject
*   Request changes
*   Ask in chat

* * *

4.2) Tracking Card (job in progress)
------------------------------------

```ts
type TrackingCard = CardBase & {
  card_type: "job.tracking";

  progress: {
    percent?: number;              // 0–100 (optional)
    status_line: string;           // “Waiting for invoice number”
    current_step?: string;         // “Step 2/3”
    blockers?: Array<string>;
    waiting_on?: Array<{
      entity_id: EntityId;
      display_name: string;
    }>;

    steps?: Array<{
      key: string;                 // "collect_input"
      label: string;               // “Collect invoice number”
      state: "todo" | "doing" | "done" | "blocked";
      updated_at?: ISO8601;
    }>;

    last_update_at?: ISO8601;
  };

  // Optional quick links to artifacts-in-progress
  artifacts_preview?: ArtifactRef[];
};
```

**Mandatory v1 buttons for Tracking**

*   Acknowledge (ack)
*   Provide input
*   Dispute
*   Cancel
*   Ask in chat

* * *

4.3) Finished Card (done/failed/cancelled/rejected)
---------------------------------------------------

```ts
type FinishedCard = CardBase & {
  card_type: "job.finished";

  outcome: {
    result: "completed" | "failed" | "cancelled" | "rejected";
    summary: string;               // 1–3 lines, human readable
    completed_at?: ISO8601;
    failure_reason?: string;       // if failed
  };

  artifacts: ArtifactRef[];        // can be empty but usually present
  next_actions?: Array<{
    label: string;                 // “Create follow-up job”
    suggested_action: CardAction;  // usually job.follow_up or chat.ask
  }>;
};

type ArtifactRef = {
  artifact_id: UUID;
  kind: "file" | "link" | "record" | "quote";
  title: string;
  url?: string;                    // if link/file hosted somewhere
  mime_type?: string;              // if file
  size_bytes?: number;
  // optional ledger pointer
  event_id?: EventId;
};
```

**Mandatory v1 buttons for Finished**

*   Accept outcome
*   Dispute
*   Follow-up job
*   Ask in chat

For “Accept outcome”, v1 can map it to `job.ack` (or define `job.accept` if you want—optional).

* * *

5) Job lifecycle events (separate from message.sent)
====================================================

These are the **job authority events** Office writes. Messenger shows them as cards by emitting/deriving `message.sent` card messages, but the job state must still be real.

5.1) job.created
----------------

```ts
type JobCreated = UblEvent<"job.created", {
  job_id: JobId;
  title: string;
  conversation_id: ConversationId;
  owner_entity_id: EntityId;       // who is responsible (agent/human)
}>;
```

5.2) job.proposed
-----------------

```ts
type JobProposed = UblEvent<"job.proposed", {
  job_id: JobId;
  proposed_card: FormalizeCard;     // embed the exact card payload
}>;
```

5.3) job.approved / job.rejected
--------------------------------

```ts
type JobApproved = UblEvent<"job.approved", {
  job_id: JobId;
}>;
type JobRejected = UblEvent<"job.rejected", {
  job_id: JobId;
  reason_code?: string;
  reason_text?: string;
}>;
```

5.4) job.state\_changed
-----------------------

```ts
type JobStateChanged = UblEvent<"job.state_changed", {
  job_id: JobId;
  prev_state: JobState;
  next_state: JobState;
  reason_code?: string;
  note?: string;
}>;
```

5.5) job.progress
-----------------

```ts
type JobProgress = UblEvent<"job.progress", {
  job_id: JobId;
  tracking_card: TrackingCard;      // embed the exact tracking card payload
}>;
```

5.6) job.completed
------------------

```ts
type JobCompleted = UblEvent<"job.completed", {
  job_id: JobId;
  finished_card: FinishedCard;      // embed the exact finished card payload
}>;
```

* * *

6) What UI sends when user clicks a button (Gateway inbound)
============================================================

UI sends a **CardAction submission**. Gateway forwards to Office.

```ts
type CardActionSubmit = {
  tenant_id: TenantId;
  trace_id: TraceId;

  conversation_id: ConversationId;
  actor_entity_id: EntityId;        // user who clicked
  actor_type: "human";              // in UI it’s usually human

  card_id: UUID;
  button_id: UUID;

  action: CardAction;

  // present only if action.requires_input or provide_input
  input?: Record<string, string | number | boolean>;
};
```

* * *

7) What Messenger Gateway returns to UI for a conversation timeline
===================================================================

```ts
type TimelineResponse = {
  tenant_id: TenantId;
  conversation_id: ConversationId;

  items: TimelineItem[];
  next_cursor?: string;
};

type TimelineItem = {
  kind: "message";
  ts: ISO8601;
  event_id: EventId;

  sender: {
    entity_id: EntityId;
    display_name: string;
    actor_type: "human" | "agent" | "system";
  };

  message: {
    message_id: UUID;
    kind: "text" | "card" | "system";
    body_text?: string;
    card?: JobCard;
  };
};
```

* * *

8) SSE event shape (UI live updates)
====================================

```ts
type SseEnvelope =
  | { type: "timeline.append"; tenant_id: TenantId; conversation_id: ConversationId; item: TimelineItem }
  | { type: "job.update"; tenant_id: TenantId; job_id: JobId; state: JobState }
  | { type: "presence.update"; tenant_id: TenantId; entity_id: EntityId; presence: "available" | "working" | "waiting_on_you" | "offline" };
```

* * *

Tiny example (end-to-end feel)
------------------------------

*   Human: “Can you send invoice 123 to ACME?”
*   Office writes:
    *   `job.created` (job\_id=J1)
    *   `job.proposed` with FormalizeCard(J1)
    *   `message.sent` with card FormalizeCard(J1)
*   User clicks Approve:
    *   UI sends `CardActionSubmit { action: job.approve, job_id: J1 }`
*   Office writes:
    *   `job.approved`
    *   `job.state_changed → in_progress`
    *   `job.progress` with TrackingCard(J1)
    *   `message.sent` with card TrackingCard(J1)
    *   `job.completed` with FinishedCard(J1)
    *   `message.sent` with card FinishedCard(J1)

UI shows 3 cards, chat never freezes, and `/jobs/J1` returns the whole chain.

* * *

If you say “YES” again, I’ll do the next extremely useful step: **the default button sets + default copy + default card layouts** (so the UI looks flagship immediately without design bikeshedding).

## Prompt:
YES

## Response:
Alright — here’s the **default button sets + default copy + default layout rules** so the UI ships “flagship-feel” immediately, without bikeshedding. This is opinionated on purpose.

* * *

1) Default button sets (exact, consistent across the app)
=========================================================

A) FormalizeCard buttons (job proposal)
---------------------------------------

Order matters (left→right / top→bottom depending on UI):

1.  **Approve** (primary)
    *   label: `"Approve"`
*   action: `job.approve`
*   style: `"primary"`
    2.  **Reject** (danger)
    *   label: `"Reject"`
*   action: `job.reject`
*   style: `"danger"`
*   confirm: `{ title: "Reject this job?", body: "Office will stop and ask what you want instead." }`
    3.  **Request changes** (secondary)
    *   label: `"Request changes"`
*   action: `job.request_changes`
*   style: `"secondary"`
*   requires\_input: true
*   input\_schema: one multiline field `changes_request`
    4.  **Ask in chat** (secondary)
    *   label: `"Ask in chat"`
*   action: `chat.ask` with a prefilled prompt like:  
    `"Can you clarify what should change about this job?"`

✅ **Always include all 4.** Even if the job is simple.

* * *

B) TrackingCard buttons (job in progress)
-----------------------------------------

1.  **Acknowledge** (primary)
    *   label: `"Got it"`
*   action: `job.ack`
*   style: `"primary"`
    2.  **Provide input** (secondary)
    *   label: `"Provide info"`
*   action: `job.provide_input`
*   style: `"secondary"`
*   requires\_input: true
*   input\_schema: derived from job’s missing inputs; fallback to one multiline `info`
    3.  **Dispute** (danger)
    *   label: `"Dispute"`
*   action: `job.dispute`
*   style: `"danger"`
*   requires\_input: true (multiline `dispute_reason`)
*   confirm: `{ title: "Dispute this update?", body: "Office will pause and ask for clarification." }`
    4.  **Cancel** (danger, smaller)
    *   label: `"Cancel"`
*   action: `job.cancel`
*   style: `"danger"`
*   confirm: `{ title: "Cancel this job?", body: "Office will stop work on this job." }`
    5.  **Ask in chat** (secondary)
    *   label: `"Ask in chat"`
*   action: `chat.ask` prompt:  
    `"Quick question about job {{job_id}}: …"`

✅ Tracking has **5 buttons** so the user always has a safe escape hatch.

* * *

C) FinishedCard buttons (job done)
----------------------------------

1.  **Accept** (primary)
    *   label: `"Accept"`
*   action: `job.ack`
*   style: `"primary"`
    2.  **Dispute** (danger)
    *   label: `"Dispute"`
*   action: `job.dispute`
*   style: `"danger"`
*   requires\_input: true (`dispute_reason`)
    3.  **Follow-up job** (secondary)
    *   label: `"Follow-up"`
*   action: `chat.ask` (v1) with prompt:  
    `"Create a follow-up job for this outcome: …"`  
    _(Later you can add `job.follow_up` as a first-class action.)_
    4.  **Ask in chat** (secondary)
    *   label: `"Ask in chat"`
*   action: `chat.ask`

✅ Finished stays light: accept or challenge, plus next-step.

* * *

2) Default copy (the “voice” of Office + cards)
===============================================

Card title rules
----------------

*   **Verb-first**: “Send invoice”, “Schedule meeting”, “Update inventory”
*   ≤ 36 chars when possible

Summary rules
-------------

*   1–2 lines, plain language
*   Avoid “AI talk” (“I will attempt to…”)
*   Prefer certainty or ask for missing info

### Default FormalizeCard copy template

*   title: `"{Verb} {Object}"`
*   summary: `"Here’s the job proposal. Approve to start, or request changes."`

**Goal line** (inside card):

*   `"Goal: {goal}"`

**SLA hint** (optional):

*   `"ETA: ~{x} min"`

**Constraints** (if any):

*   `"Constraints: {constraint1}; {constraint2}"`

### Default TrackingCard copy template

*   title: `"{Title}"`
*   summary: `"In progress. You can keep chatting while I work."`

**Status line:**

*   `"Status: {status_line}"`

**Waiting on:**

*   `"Waiting on: {name}"` (only if waiting\_input)

### Default FinishedCard copy template

*   title: `"{Title}"`
*   summary: `"Done. Review the outcome below."`  
    (If failed: `"I couldn’t finish. See why below."`)

**Outcome summary:**

*   1–3 lines max
*   Include what was produced + where it went

* * *

3) Default layout rules (so it looks tight in WhatsApp UI)
==========================================================

FormalizeCard layout (compact and scannable)
--------------------------------------------

Top row:

*   **Title** (left)
*   **State pill** (right): “PROPOSED”

Body blocks (in this order):

1.  **Goal** (always)
2.  **Owner** (who will do it)
3.  **Inputs needed** (only if any)
4.  **Expected outputs** (optional)
5.  **Constraints** (optional)
6.  **ETA / due** (optional)

Buttons row:

*   Primary first, then danger/secondary
*   On mobile: 2 per row (Approve | Reject) then (Request changes | Ask in chat)

TrackingCard layout
-------------------

Top row:

*   Title + state pill: “IN PROGRESS” / “WAITING”

Body:

1.  **Status line** (bold)
2.  **Steps** list (up to 5 steps, collapse after)
3.  **Blockers** (if any)
4.  **Artifacts preview** (if any)

Buttons:

*   Primary (“Got it”) always visible
*   Others may be behind “More” on very small screens, but **Dispute** must remain visible.

FinishedCard layout
-------------------

Top row:

*   Title + pill: “DONE” / “FAILED”

Body:

1.  **Outcome summary** (bold)
2.  **Artifacts** list (each with icon + title + open)
3.  **Next actions** (optional)

Buttons:

*   Accept (primary)
*   Dispute (danger)
*   Follow-up (secondary)
*   Ask in chat (secondary)

* * *

4) Default card fields (what always appears, even in v1)
========================================================

Every card shows:

*   `job_id` (short form like `J1…9K` clickable “copy”)
*   owner name (“Owner: Ana (Agent)”)
*   timestamp (“Created 14:12”)

Every card is clickable to open:

*   “Job details” drawer (v1 can just show the raw event chain)

* * *

5) Default “presence” UX for agents (coworker dignity)
======================================================

In conversation list:

*   `available`: green dot + “Available”
*   `working`: small spinner + “Working on {{job\_id\_short}}”
*   `waiting_on_you`: amber dot + “Needs your input”
*   `offline`: gray dot

In header:

*   If `working`, show a small chip: “Job: {{title}}”

Presence updates come from SSE `presence.update` OR derived from job state transitions.

* * *

6) Default fallback behaviors (so it never breaks)
==================================================

*   If card has unknown fields: render title/summary/buttons only (never crash)
*   If `input_schema` missing but `requires_input=true`: show a single multiline “Details”
*   If artifacts list empty: hide artifacts block cleanly
*   If SSE disconnects: UI keeps working with polling every 10s

* * *

7) Default “golden path” UX script (what v1 should feel like)
=============================================================

1.  Human: “Send invoice 123 to ACME”
2.  Office proposes Formalize card:
    *   Goal filled
    *   Inputs show “Invoice number: missing” if not found
3.  Human clicks **Approve**
4.  Tracking card appears immediately (“In progress…”) — chat remains open
5.  If missing info: Tracking says “Waiting for invoice number” and button **Provide info**
6.  Human provides info; Tracking updates
7.  Finished card arrives with artifact link (“Invoice PDF” + “Email sent to billing@acme.com”)

* * *

If you want one more “no ambiguity” piece, tell me **which v1 job you want as the demo** (invoicing, scheduling, support ticket, onboarding). If you don’t care, I’ll pick **“Schedule a meeting + send invite”** because it demonstrates waiting\_input + artifact links cleanly.

## Prompt:
YES

## Response:
Locked in: **V1 demo job = “Schedule a meeting + send invite”**. It hits _everything_ you need for flagship: missing info → waiting state → button-driven input → artifacts (calendar link) → finished card — all while chat stays live.

Below is the **exact golden-path script + the actual example payloads** (events + cards + button submissions). You can copy/paste these into tests or seed data.

* * *

V1 Demo Scenario
----------------

**Human** (Dan) messages **Agent coworker** (“Office Scheduler”) in Messenger:

> “Can you schedule a 30-min call with Maria next week?”

Office doesn’t guess. It creates a job proposal, asks for missing inputs via the **card system**, and completes with a calendar artifact.

### The missing inputs we _expect_ in v1

*   Maria’s email (or contact)
*   Preferred days/times or timezone confirmation
*   Meeting title
*   Location/link (Google Meet / Zoom)

* * *

IDs used in this example (stable for the whole flow)
====================================================

```json
{
  "tenant_id": "tnt_acme_001",
  "conversation_id": "cnv_9f2a",
  "human_entity_id": "ent_human_dan",
  "agent_entity_id": "ent_agent_scheduler",
  "job_id": "job_sched_4c1b",
  "trace_id": "trc_20251227_001"
}
```

* * *

1) Human sends the message (UBL `message.sent`)
===============================================

```json
{
  "event_id": "evt_0001",
  "event_type": "message.sent",
  "ts": "2025-12-27T10:15:00.000Z",
  "tenant_id": "tnt_acme_001",
  "trace_id": "trc_20251227_001",
  "conversation_id": "cnv_9f2a",
  "actor": { "entity_id": "ent_human_dan", "actor_type": "human" },
  "payload": {
    "message_id": "msg_0001",
    "kind": "text",
    "body_text": "Can you schedule a 30-min call with Maria next week?"
  }
}
```

* * *

2) Office creates + proposes the job (UBL `job.created`, `job.proposed`)
========================================================================

### 2.1 `job.created`

```json
{
  "event_id": "evt_0002",
  "event_type": "job.created",
  "ts": "2025-12-27T10:15:01.000Z",
  "tenant_id": "tnt_acme_001",
  "trace_id": "trc_20251227_001",
  "conversation_id": "cnv_9f2a",
  "job_id": "job_sched_4c1b",
  "actor": { "entity_id": "ent_agent_scheduler", "actor_type": "agent" },
  "payload": {
    "job_id": "job_sched_4c1b",
    "title": "Schedule call with Maria",
    "conversation_id": "cnv_9f2a",
    "owner_entity_id": "ent_agent_scheduler"
  }
}
```

### 2.2 `job.proposed` (embeds the FormalizeCard)

```json
{
  "event_id": "evt_0003",
  "event_type": "job.proposed",
  "ts": "2025-12-27T10:15:02.000Z",
  "tenant_id": "tnt_acme_001",
  "trace_id": "trc_20251227_001",
  "conversation_id": "cnv_9f2a",
  "job_id": "job_sched_4c1b",
  "actor": { "entity_id": "ent_agent_scheduler", "actor_type": "agent" },
  "payload": {
    "job_id": "job_sched_4c1b",
    "proposed_card": {
      "card_id": "card_formalize_001",
      "job_id": "job_sched_4c1b",
      "card_type": "job.formalize",
      "version": "v1",
      "title": "Schedule call with Maria",
      "summary": "Here’s the job proposal. Approve to start, or request changes.",
      "state": "proposed",
      "created_at": "2025-12-27T10:15:02.000Z",
      "conversation_id": "cnv_9f2a",
      "tenant_id": "tnt_acme_001",
      "owner": {
        "entity_id": "ent_agent_scheduler",
        "display_name": "Office Scheduler",
        "actor_type": "agent"
      },
      "author": {
        "entity_id": "ent_agent_scheduler",
        "display_name": "Office Scheduler",
        "actor_type": "agent"
      },
      "job": {
        "job_id": "job_sched_4c1b",
        "goal": "Schedule a 30-minute call with Maria next week and send an invite",
        "priority": "normal",
        "inputs_needed": [
          { "key": "maria_contact", "label": "Maria email/contact", "status": "missing" },
          { "key": "time_window", "label": "Preferred days/times", "status": "missing" },
          { "key": "timezone", "label": "Timezone confirmation", "status": "missing", "value_preview": "Europe/Lisbon?" },
          { "key": "meeting_link", "label": "Meeting link type", "status": "missing", "value_preview": "Google Meet?" }
        ],
        "expected_outputs": [
          { "kind": "record", "description": "Calendar event created" },
          { "kind": "link", "description": "Invite link sent to Maria" }
        ],
        "constraints": ["Don’t email Maria until you approve the details"],
        "sla_hint": "ETA ~5 minutes after details"
      },
      "plan_hint": ["Collect missing details", "Create calendar invite", "Send invite to Maria"],
      "buttons": [
        {
          "button_id": "btn_approve_001",
          "label": "Approve",
          "style": "primary",
          "action": { "type": "job.approve", "job_id": "job_sched_4c1b" }
        },
        {
          "button_id": "btn_reject_001",
          "label": "Reject",
          "style": "danger",
          "confirm": { "title": "Reject this job?", "body": "Office will stop and ask what you want instead." },
          "action": { "type": "job.reject", "job_id": "job_sched_4c1b" }
        },
        {
          "button_id": "btn_changes_001",
          "label": "Request changes",
          "style": "secondary",
          "requires_input": true,
          "action": { "type": "job.request_changes", "job_id": "job_sched_4c1b" }
        },
        {
          "button_id": "btn_askchat_001",
          "label": "Ask in chat",
          "style": "secondary",
          "action": { "type": "chat.ask", "job_id": "job_sched_4c1b", "prompt_text": "What should change about this job proposal?" }
        }
      ]
    }
  }
}
```

### 2.3 Messenger renders it as a card message (UBL `message.sent`)

_(This is what actually appears in the chat timeline.)_

```json
{
  "event_id": "evt_0004",
  "event_type": "message.sent",
  "ts": "2025-12-27T10:15:02.200Z",
  "tenant_id": "tnt_acme_001",
  "trace_id": "trc_20251227_001",
  "conversation_id": "cnv_9f2a",
  "job_id": "job_sched_4c1b",
  "actor": { "entity_id": "ent_agent_scheduler", "actor_type": "agent" },
  "payload": {
    "message_id": "msg_0002",
    "kind": "card",
    "card": { "$ref": "FormalizeCard above (card_formalize_001)" }
  }
}
```

* * *

3) User clicks “Approve” (UI → Gateway inbound)
===============================================

```json
{
  "tenant_id": "tnt_acme_001",
  "trace_id": "trc_20251227_001",
  "conversation_id": "cnv_9f2a",
  "actor_entity_id": "ent_human_dan",
  "actor_type": "human",
  "card_id": "card_formalize_001",
  "button_id": "btn_approve_001",
  "action": { "type": "job.approve", "job_id": "job_sched_4c1b" }
}
```

Office writes to UBL:

### 3.1 `job.approved` + `job.state_changed`

```json
{
  "event_id": "evt_0005",
  "event_type": "job.approved",
  "ts": "2025-12-27T10:15:05.000Z",
  "tenant_id": "tnt_acme_001",
  "trace_id": "trc_20251227_001",
  "conversation_id": "cnv_9f2a",
  "job_id": "job_sched_4c1b",
  "actor": { "entity_id": "ent_human_dan", "actor_type": "human" },
  "payload": { "job_id": "job_sched_4c1b" }
}
```

```json
{
  "event_id": "evt_0006",
  "event_type": "job.state_changed",
  "ts": "2025-12-27T10:15:05.050Z",
  "tenant_id": "tnt_acme_001",
  "trace_id": "trc_20251227_001",
  "conversation_id": "cnv_9f2a",
  "job_id": "job_sched_4c1b",
  "actor": { "entity_id": "ent_agent_scheduler", "actor_type": "agent" },
  "payload": {
    "job_id": "job_sched_4c1b",
    "prev_state": "proposed",
    "next_state": "in_progress",
    "reason_code": "approved_by_user"
  }
}
```

* * *

4) Office posts TrackingCard, but it’s missing info → WAITING\_INPUT
====================================================================

### 4.1 `job.state_changed` → `waiting_input`

```json
{
  "event_id": "evt_0007",
  "event_type": "job.state_changed",
  "ts": "2025-12-27T10:15:06.000Z",
  "tenant_id": "tnt_acme_001",
  "trace_id": "trc_20251227_001",
  "conversation_id": "cnv_9f2a",
  "job_id": "job_sched_4c1b",
  "actor": { "entity_id": "ent_agent_scheduler", "actor_type": "agent" },
  "payload": {
    "job_id": "job_sched_4c1b",
    "prev_state": "in_progress",
    "next_state": "waiting_input",
    "reason_code": "missing_required_inputs",
    "note": "Need contact + preferred times before creating invite."
  }
}
```

### 4.2 `job.progress` with TrackingCard (embeds the card)

```json
{
  "event_id": "evt_0008",
  "event_type": "job.progress",
  "ts": "2025-12-27T10:15:06.200Z",
  "tenant_id": "tnt_acme_001",
  "trace_id": "trc_20251227_001",
  "conversation_id": "cnv_9f2a",
  "job_id": "job_sched_4c1b",
  "actor": { "entity_id": "ent_agent_scheduler", "actor_type": "agent" },
  "payload": {
    "job_id": "job_sched_4c1b",
    "tracking_card": {
      "card_id": "card_tracking_001",
      "job_id": "job_sched_4c1b",
      "card_type": "job.tracking",
      "version": "v1",
      "title": "Schedule call with Maria",
      "summary": "In progress. You can keep chatting while I work.",
      "state": "waiting_input",
      "created_at": "2025-12-27T10:15:06.200Z",
      "conversation_id": "cnv_9f2a",
      "tenant_id": "tnt_acme_001",
      "owner": { "entity_id": "ent_agent_scheduler", "display_name": "Office Scheduler", "actor_type": "agent" },
      "author": { "entity_id": "ent_agent_scheduler", "display_name": "Office Scheduler", "actor_type": "agent" },
      "progress": {
        "status_line": "Waiting for: Maria’s email + preferred days/times",
        "waiting_on": [{ "entity_id": "ent_human_dan", "display_name": "Dan" }],
        "steps": [
          { "key": "collect_inputs", "label": "Collect details", "state": "blocked" },
          { "key": "create_invite", "label": "Create calendar invite", "state": "todo" },
          { "key": "send_invite", "label": "Send invite to Maria", "state": "todo" }
        ],
        "last_update_at": "2025-12-27T10:15:06.200Z"
      },
      "artifacts_preview": [],
      "buttons": [
        { "button_id": "btn_ack_001", "label": "Got it", "style": "primary", "action": { "type": "job.ack", "job_id": "job_sched_4c1b" } },
        {
          "button_id": "btn_input_001",
          "label": "Provide info",
          "style": "secondary",
          "requires_input": true,
          "action": {
            "type": "job.provide_input",
            "job_id": "job_sched_4c1b",
            "input_schema": {
              "fields": [
                { "key": "maria_email", "label": "Maria email", "type": "string", "required": true, "placeholder": "maria@company.com" },
                { "key": "time_window", "label": "Preferred days/times", "type": "multiline", "required": true, "placeholder": "Tue–Thu, 14:00–17:00" },
                { "key": "timezone", "label": "Timezone", "type": "string", "required": true, "placeholder": "Europe/Lisbon" },
                { "key": "meeting_link", "label": "Meeting link", "type": "select", "required": true, "options": [
                  { "value": "google_meet", "label": "Google Meet" },
                  { "value": "zoom", "label": "Zoom" }
                ]}
              ]
            }
          }
        },
        {
          "button_id": "btn_dispute_001",
          "label": "Dispute",
          "style": "danger",
          "requires_input": true,
          "confirm": { "title": "Dispute this update?", "body": "Office will pause and ask for clarification." },
          "action": { "type": "job.dispute", "job_id": "job_sched_4c1b" }
        },
        {
          "button_id": "btn_cancel_001",
          "label": "Cancel",
          "style": "danger",
          "confirm": { "title": "Cancel this job?", "body": "Office will stop work on this job." },
          "action": { "type": "job.cancel", "job_id": "job_sched_4c1b" }
        },
        {
          "button_id": "btn_askchat_002",
          "label": "Ask in chat",
          "style": "secondary",
          "action": { "type": "chat.ask", "job_id": "job_sched_4c1b", "prompt_text": "Quick question about the scheduling job—what should I assume?" }
        }
      ]
    }
  }
}
```

### 4.3 Messenger emits it as a card message (`message.sent`)

```json
{
  "event_id": "evt_0009",
  "event_type": "message.sent",
  "ts": "2025-12-27T10:15:06.300Z",
  "tenant_id": "tnt_acme_001",
  "trace_id": "trc_20251227_001",
  "conversation_id": "cnv_9f2a",
  "job_id": "job_sched_4c1b",
  "actor": { "entity_id": "ent_agent_scheduler", "actor_type": "agent" },
  "payload": {
    "message_id": "msg_0003",
    "kind": "card",
    "card": { "$ref": "TrackingCard above (card_tracking_001)" }
  }
}
```

* * *

5) User clicks “Provide info” and submits the modal (UI → Gateway inbound)
==========================================================================

```json
{
  "tenant_id": "tnt_acme_001",
  "trace_id": "trc_20251227_001",
  "conversation_id": "cnv_9f2a",
  "actor_entity_id": "ent_human_dan",
  "actor_type": "human",
  "card_id": "card_tracking_001",
  "button_id": "btn_input_001",
  "action": { "type": "job.provide_input", "job_id": "job_sched_4c1b" },
  "input": {
    "maria_email": "maria@acme.com",
    "time_window": "Tue–Thu, 14:00–17:00",
    "timezone": "Europe/Lisbon",
    "meeting_link": "google_meet"
  }
}
```

Office now flips back to `in_progress`, creates invite, and finishes.

* * *

6) Office completes the job: Tracking update → Finished card
============================================================

### 6.1 `job.state_changed` waiting\_input → in\_progress

```json
{
  "event_id": "evt_0010",
  "event_type": "job.state_changed",
  "ts": "2025-12-27T10:15:20.000Z",
  "tenant_id": "tnt_acme_001",
  "trace_id": "trc_20251227_001",
  "conversation_id": "cnv_9f2a",
  "job_id": "job_sched_4c1b",
  "actor": { "entity_id": "ent_agent_scheduler", "actor_type": "agent" },
  "payload": {
    "job_id": "job_sched_4c1b",
    "prev_state": "waiting_input",
    "next_state": "in_progress",
    "reason_code": "inputs_received"
  }
}
```

### 6.2 `job.progress` (Tracking card shows “Creating invite…”)

_(You can reuse the same TrackingCard type, with updated steps.)_

### 6.3 `job.completed` embeds FinishedCard

```json
{
  "event_id": "evt_0012",
  "event_type": "job.completed",
  "ts": "2025-12-27T10:15:40.000Z",
  "tenant_id": "tnt_acme_001",
  "trace_id": "trc_20251227_001",
  "conversation_id": "cnv_9f2a",
  "job_id": "job_sched_4c1b",
  "actor": { "entity_id": "ent_agent_scheduler", "actor_type": "agent" },
  "payload": {
    "job_id": "job_sched_4c1b",
    "finished_card": {
      "card_id": "card_finished_001",
      "job_id": "job_sched_4c1b",
      "card_type": "job.finished",
      "version": "v1",
      "title": "Schedule call with Maria",
      "summary": "Done. Review the outcome below.",
      "state": "completed",
      "created_at": "2025-12-27T10:15:40.000Z",
      "conversation_id": "cnv_9f2a",
      "tenant_id": "tnt_acme_001",
      "owner": { "entity_id": "ent_agent_scheduler", "display_name": "Office Scheduler", "actor_type": "agent" },
      "author": { "entity_id": "ent_agent_scheduler", "display_name": "Office Scheduler", "actor_type": "agent" },
      "outcome": {
        "result": "completed",
        "summary": "Created a 30-minute invite and sent it to maria@acme.com for Tue–Thu, 14:00–17:00 (Europe/Lisbon).",
        "completed_at": "2025-12-27T10:15:40.000Z"
      },
      "artifacts": [
        {
          "artifact_id": "art_cal_001",
          "kind": "link",
          "title": "Calendar invite (Google Meet)",
          "url": "https://calendar.example/invite/abc123",
          "event_id": "evt_0012"
        }
      ],
      "next_actions": [
        {
          "label": "Create follow-up job",
          "suggested_action": {
            "type": "chat.ask",
            "job_id": "job_sched_4c1b",
            "prompt_text": "Create a follow-up job related to this meeting (agenda, notes template, reminders)."
          }
        }
      ],
      "buttons": [
        { "button_id": "btn_accept_001", "label": "Accept", "style": "primary", "action": { "type": "job.ack", "job_id": "job_sched_4c1b" } },
        { "button_id": "btn_dispute_002", "label": "Dispute", "style": "danger", "requires_input": true, "action": { "type": "job.dispute", "job_id": "job_sched_4c1b" } },
        { "button_id": "btn_followup_001", "label": "Follow-up", "style": "secondary", "action": { "type": "chat.ask", "job_id": "job_sched_4c1b", "prompt_text": "Make a follow-up job based on this outcome." } },
        { "button_id": "btn_askchat_003", "label": "Ask in chat", "style": "secondary", "action": { "type": "chat.ask", "job_id": "job_sched_4c1b", "prompt_text": "Any question about the meeting invite?" } }
      ]
    }
  }
}
```

And Messenger shows it as a timeline card:

```json
{
  "event_id": "evt_0013",
  "event_type": "message.sent",
  "ts": "2025-12-27T10:15:40.050Z",
  "tenant_id": "tnt_acme_001",
  "trace_id": "trc_20251227_001",
  "conversation_id": "cnv_9f2a",
  "job_id": "job_sched_4c1b",
  "actor": { "entity_id": "ent_agent_scheduler", "actor_type": "agent" },
  "payload": {
    "message_id": "msg_0005",
    "kind": "card",
    "card": { "$ref": "FinishedCard above (card_finished_001)" }
  }
}
```

* * *

What this proves (flagship logic, not just UI)
----------------------------------------------

*   **Cards are formal** (every card tied to `job_id`)
*   **Buttons create real transitions** (approve/input/dispute/cancel)
*   **Chat never freezes** (cards live inside the timeline, not a modal workflow)
*   **Ledger is truth** (the entire narrative is reconstructible from events by `job_id`)
*   **Office gives dignity** (clear responsibility, approvals, constraints, attribution)

* * *

If you want to push this from “demo-complete” to “shockingly real” with almost no extra work, the very next upgrade is adding **`tool.called`** + **`tool.result`** events around “create calendar invite” — so the audit trail shows _exactly_ what the agent did, when, and with what inputs.

## Prompt:
YES

## Response:
Great — adding `tool.called` + `tool.result` is the _exact_ move that turns this from “nice UX” into **computable, auditable work**.

Below are the **contracts + default redaction rules + a complete example** for the scheduling job (calendar invite tool).

* * *

1) Event contracts
------------------

### 1.1 `tool.called` (Office → UBL)

This event records **intent to execute** a tool, with inputs in a **safe form**.

```json
{
  "event_id": "evt_0020",
  "event_type": "tool.called",
  "ts": "2025-12-27T10:15:24.000Z",
  "tenant_id": "tnt_acme_001",
  "trace_id": "trc_20251227_001",
  "conversation_id": "cnv_9f2a",
  "job_id": "job_sched_4c1b",
  "actor": { "entity_id": "ent_agent_scheduler", "actor_type": "agent" },
  "payload": {
    "tool_call_id": "tcall_001",
    "tool_name": "calendar.create_invite",
    "tool_version": "v1",
    "purpose": "Create a 30-minute meeting invite with Google Meet",
    "inputs": {
      "start_window": "Tue–Thu, 14:00–17:00",
      "timezone": "Europe/Lisbon",
      "duration_minutes": 30,
      "attendees": [
        {
          "label": "Maria",
          "email_redacted": "m***@acme.com",
          "email_hash": "blake3:8b6c...e19a"
        }
      ],
      "meeting_link": "google_meet",
      "title": "Call with Maria",
      "notes_hint": "Short check-in call"
    },
    "pii_policy": {
      "redactions_applied": ["email_redacted"],
      "hashes_applied": ["email_hash"],
      "raw_pii_stored": false
    },
    "idempotency_key": "idem:job_sched_4c1b:calendar.create_invite:v1"
  }
}
```

### 1.2 `tool.result` (Office → UBL)

This event records **what happened**: success/failure, outputs, and pointers to artifacts.

```json
{
  "event_id": "evt_0021",
  "event_type": "tool.result",
  "ts": "2025-12-27T10:15:29.000Z",
  "tenant_id": "tnt_acme_001",
  "trace_id": "trc_20251227_001",
  "conversation_id": "cnv_9f2a",
  "job_id": "job_sched_4c1b",
  "actor": { "entity_id": "ent_agent_scheduler", "actor_type": "agent" },
  "payload": {
    "tool_call_id": "tcall_001",
    "tool_name": "calendar.create_invite",
    "status": "success",
    "latency_ms": 4870,
    "output": {
      "calendar_provider": "google",
      "event_id": "gcal_evt_7f91",
      "meeting_link": "https://meet.google.com/abc-defg-hij",
      "invite_url": "https://calendar.example/invite/abc123",
      "scheduled_time": {
        "start": "2025-12-30T14:30:00.000Z",
        "end": "2025-12-30T15:00:00.000Z",
        "timezone": "Europe/Lisbon"
      },
      "attendees_count": 2
    },
    "artifacts": [
      {
        "artifact_id": "art_cal_001",
        "kind": "link",
        "title": "Calendar invite (Google Meet)",
        "url": "https://calendar.example/invite/abc123",
        "mime_type": "text/html",
        "event_id": "evt_0021"
      }
    ],
    "safety": {
      "pii_leak_detected": false,
      "redaction_summary": ["attendee emails not stored raw"]
    }
  }
}
```

* * *

2) Minimal payload schema (so everyone implements it the same way)
------------------------------------------------------------------

### `tool.called.payload`

*   `tool_call_id` (stable id for call/result pairing)
*   `tool_name`, `tool_version`
*   `purpose` (human readable)
*   `inputs` (sanitized)
*   `pii_policy` (what you did)
*   `idempotency_key` (so retries don’t duplicate side effects)

### `tool.result.payload`

*   `tool_call_id`, `tool_name`
*   `status`: `"success" | "error"`
*   `latency_ms`
*   `output` (sanitized)
*   `artifacts[]`
*   optional `error` object if failed:
    *   `error_code`, `message_safe`, `retryable`

* * *

3) PII rules (v1 defaults that keep you safe)
---------------------------------------------

**Rule 1 — Never store raw attendee emails/phones in the ledger.**  
Store:

*   `email_redacted` (m\*\*\*@acme.com)
*   `email_hash` (BLAKE3 over canonical lowercase email + tenant salt)

**Rule 2 — Store URLs that are meant to be shared.**  
Meeting links and invite links are “artifacts.” That’s fine.

**Rule 3 — If a tool needs raw PII, it gets it at execution time only.**  
The raw PII can live:

*   in Messenger session memory / secure store
*   or an encrypted vault reference  
    But **ledger only gets a pointer + hash**, not the raw.

* * *

4) Full “shockingly real” flow patch (where these events sit)
-------------------------------------------------------------

When Office receives the missing inputs and transitions back to `in_progress`, the sequence becomes:

1.  `job.state_changed` waiting\_input → in\_progress
2.  `tool.called` calendar.create\_invite
3.  `tool.result` (success)
4.  `job.progress` (TrackingCard updates steps: “Create invite ✅”)
5.  `job.completed` (FinishedCard includes artifact link)

That’s it. Now anyone can audit:

*   **who** approved
*   **what** tool ran
*   **with which sanitized inputs**
*   **what** it produced
*   **what** the user accepted/disputed

* * *

5) One small extra that’s worth it (still v1)
---------------------------------------------

Add a tiny field to `job.progress` / TrackingCard:

*   `last_tool_call_id?: string`  
    So the UI can show: “Created invite (details)” and open a job drawer that lists tool calls/results for that job.

* * *

If you want, next “YES” and I’ll give you the **failure path** (tool error → retry rules → user-facing card update wording) so the system stays calm and professional when Google/Zoom fails.

## Prompt:
YES

## Response:
Cool — here’s the **failure path** that keeps everything calm, auditable, and “SMB professional,” while preserving your core rule: **chat never freezes** and **cards always provide options**.

You’ll get:

*   The **state machine behavior**
*   The **exact events** (`tool.result` error, `job.state_changed`, `job.progress` card update)
*   The **retry rules** (idempotency + backoff + escalation)
*   The **default UI copy** (no drama, no “AI excuses”)
*   The **buttons** that keep the user in control

* * *

1) Failure state machine (minimal, correct)
===========================================

When a tool fails inside a job:

### If retryable (network/timeouts/429):

`in_progress → in_progress` (keep state)  
…but TrackingCard updates with “temporary issue + retry schedule” and buttons.

### If blocked by missing permission/invalid inputs:

`in_progress → waiting_input`  
TrackingCard asks for the exact missing permission/input.

### If hard failure (tool down, invalid request repeatedly):

`in_progress → failed` (or keep in `in_progress` but “paused”)  
I recommend: **failed** only after retries are exhausted.

* * *

2) Retry rules (v1 defaults)
============================

Idempotency
-----------

Every tool call includes:

*   `idempotency_key = idem:{job_id}:{tool_name}:{tool_version}`

So retrying won’t create duplicates if the provider actually succeeded but the response got lost.

Backoff schedule
----------------

*   Attempt 1: immediate
*   Attempt 2: +10s
*   Attempt 3: +60s
*   Attempt 4: +5m  
    Then **escalate to user** (choose new action, switch tool, or cancel).

Circuit breaker (tenant-scoped)
-------------------------------

If a provider fails repeatedly (e.g., 10 failures in 5 minutes), Office:

*   stops auto-retrying
*   asks user to choose: “Try later / Switch tool / Cancel”

* * *

3) Event contracts for errors
=============================

3.1 `tool.result` with error (Office → UBL)
-------------------------------------------

```json
{
  "event_id": "evt_0031",
  "event_type": "tool.result",
  "ts": "2025-12-27T10:15:29.000Z",
  "tenant_id": "tnt_acme_001",
  "trace_id": "trc_20251227_001",
  "conversation_id": "cnv_9f2a",
  "job_id": "job_sched_4c1b",
  "actor": { "entity_id": "ent_agent_scheduler", "actor_type": "agent" },
  "payload": {
    "tool_call_id": "tcall_001",
    "tool_name": "calendar.create_invite",
    "status": "error",
    "latency_ms": 4870,
    "error": {
      "error_code": "PROVIDER_TIMEOUT",
      "message_safe": "Calendar provider didn’t respond in time.",
      "retryable": true,
      "suggested_wait_seconds": 10
    },
    "safety": {
      "pii_leak_detected": false,
      "redaction_summary": ["attendee emails not stored raw"]
    }
  }
}
```

### Error codes (v1 set)

*   `PROVIDER_TIMEOUT` (retryable)
*   `PROVIDER_RATE_LIMIT` (retryable)
*   `PROVIDER_AUTH_REQUIRED` (not retryable until fixed)
*   `INVALID_INPUT` (not retryable until fixed)
*   `PROVIDER_UNAVAILABLE` (retryable, but circuit break sooner)
*   `UNKNOWN` (retryable once, then treat as unavailable)

* * *

4) How Office reacts (exact sequence)
=====================================

Case A: Retryable error (timeout/429)
-------------------------------------

### 4.1 Keep job state, emit a progress card update

You _can_ keep state `in_progress`. TrackingCard communicates retry.

**Optional**: emit `job.state_changed` with same state (no-op). Not required.

### 4.2 Emit `job.progress` with a TrackingCard that shows retry plan

```json
{
  "event_id": "evt_0032",
  "event_type": "job.progress",
  "ts": "2025-12-27T10:15:29.100Z",
  "tenant_id": "tnt_acme_001",
  "trace_id": "trc_20251227_001",
  "conversation_id": "cnv_9f2a",
  "job_id": "job_sched_4c1b",
  "actor": { "entity_id": "ent_agent_scheduler", "actor_type": "agent" },
  "payload": {
    "job_id": "job_sched_4c1b",
    "tracking_card": {
      "card_id": "card_tracking_002",
      "job_id": "job_sched_4c1b",
      "card_type": "job.tracking",
      "version": "v1",
      "title": "Schedule call with Maria",
      "summary": "In progress. You can keep chatting while I work.",
      "state": "in_progress",
      "created_at": "2025-12-27T10:15:29.100Z",
      "conversation_id": "cnv_9f2a",
      "tenant_id": "tnt_acme_001",
      "owner": { "entity_id": "ent_agent_scheduler", "display_name": "Office Scheduler", "actor_type": "agent" },
      "author": { "entity_id": "ent_agent_scheduler", "display_name": "Office Scheduler", "actor_type": "agent" },
      "progress": {
        "status_line": "Calendar provider timed out. Retrying in ~10 seconds.",
        "blockers": ["Temporary provider timeout"],
        "steps": [
          { "key": "collect_inputs", "label": "Collect details", "state": "done" },
          { "key": "create_invite", "label": "Create calendar invite", "state": "doing" },
          { "key": "send_invite", "label": "Send invite to Maria", "state": "todo" }
        ],
        "last_update_at": "2025-12-27T10:15:29.100Z"
      },
      "buttons": [
        { "button_id": "btn_ack_002", "label": "Got it", "style": "primary", "action": { "type": "job.ack", "job_id": "job_sched_4c1b" } },
        {
          "button_id": "btn_retry_now_001",
          "label": "Retry now",
          "style": "secondary",
          "action": { "type": "chat.ask", "job_id": "job_sched_4c1b", "prompt_text": "Retry creating the invite now." }
        },
        {
          "button_id": "btn_switch_tool_001",
          "label": "Switch to Zoom",
          "style": "secondary",
          "action": { "type": "job.request_changes", "job_id": "job_sched_4c1b" }
        },
        {
          "button_id": "btn_dispute_003",
          "label": "Dispute",
          "style": "danger",
          "requires_input": true,
          "action": { "type": "job.dispute", "job_id": "job_sched_4c1b" }
        },
        {
          "button_id": "btn_cancel_002",
          "label": "Cancel",
          "style": "danger",
          "confirm": { "title": "Cancel this job?", "body": "Office will stop work on this job." },
          "action": { "type": "job.cancel", "job_id": "job_sched_4c1b" }
        }
      ]
    }
  }
}
```

**Why these buttons?**

*   “Retry now” gives control without forcing them into settings.
*   “Switch tool” is the practical escape hatch.
*   Dispute/cancel are always there.

* * *

Case B: Auth required / permission missing
------------------------------------------

Example: Google calendar not connected.

### 4.1 Change to waiting\_input

```json
{
  "event_id": "evt_0033",
  "event_type": "job.state_changed",
  "ts": "2025-12-27T10:15:29.150Z",
  "tenant_id": "tnt_acme_001",
  "trace_id": "trc_20251227_001",
  "conversation_id": "cnv_9f2a",
  "job_id": "job_sched_4c1b",
  "actor": { "entity_id": "ent_agent_scheduler", "actor_type": "agent" },
  "payload": {
    "job_id": "job_sched_4c1b",
    "prev_state": "in_progress",
    "next_state": "waiting_input",
    "reason_code": "provider_auth_required",
    "note": "Calendar account not connected."
  }
}
```

### 4.2 TrackingCard asks for connection

TrackingCard status line becomes:

*   “Calendar isn’t connected. Connect it to continue.”

Buttons include:

*   “Connect calendar” (opens settings / OAuth link)
*   “Use Zoom instead”
*   “Cancel”
*   “Ask in chat”

_(In v1, “Connect calendar” can just be a link artifact or deep link.)_

* * *

Case C: Invalid input
---------------------

Example: user provided malformed email or impossible time window.

Office moves to `waiting_input` and the TrackingCard shows:

*   what’s invalid
*   exactly what to provide instead
*   a “Provide info” modal button with fields prefilled

* * *

Case D: Retries exhausted → fail gracefully
-------------------------------------------

After 4 attempts (or circuit breaker), Office ends it cleanly:

### 4.1 `job.state_changed` → failed

```json
{
  "event_id": "evt_0039",
  "event_type": "job.state_changed",
  "ts": "2025-12-27T10:21:00.000Z",
  "tenant_id": "tnt_acme_001",
  "trace_id": "trc_20251227_001",
  "conversation_id": "cnv_9f2a",
  "job_id": "job_sched_4c1b",
  "actor": { "entity_id": "ent_agent_scheduler", "actor_type": "agent" },
  "payload": {
    "job_id": "job_sched_4c1b",
    "prev_state": "in_progress",
    "next_state": "failed",
    "reason_code": "provider_unavailable_retries_exhausted",
    "note": "Calendar provider kept failing after retries."
  }
}
```

### 4.2 `job.completed` with FinishedCard result=failed

FinishedCard copy stays professional and actionable:

*   Summary: “I couldn’t finish because the calendar provider is unavailable.”
*   Artifacts: optional “error report” (safe) or “suggested next steps”
*   Buttons:
    *   “Try later” (follow-up job / schedule retry)
    *   “Switch tool”
    *   “Dispute”
    *   “Ask in chat”

* * *

5) Default UI copy for failures (ready-made)
============================================

### Retryable (timeout/429)

*   **Status line:** “Calendar provider timed out. Retrying in ~10 seconds.”
*   **Optional small note:** “You can keep chatting while I work.”

### Auth required

*   “Calendar isn’t connected. Connect it to continue.”

### Invalid input

*   “I can’t use that email/time window. Please confirm the details.”

### Retries exhausted

*   “I couldn’t finish because the calendar provider is unavailable right now.”

No “AI apology spirals.” No “I tried my best.” Just clear next steps.

* * *

6) Tiny spec addition that makes audit _amazing_
================================================

Add `attempt` counters:

### In `tool.called.payload`

*   `attempt: 1..N`
*   `retry_of_tool_call_id?: string`

### In `tool.result.payload`

*   `attempt`
*   `retryable`

This makes it trivial to show in the Job drawer:

*   Attempt 1 failed (timeout)
*   Attempt 2 succeeded  
    etc.

* * *

If you want to keep building momentum: next “YES” and I’ll give you the **Job Drawer spec** (how the UI shows the full job timeline from the ledger: state changes + tool calls/results + cards), because that’s where the “LogLine vibe” becomes undeniable.

## Prompt:
YES

## Response:
Awesome — the **Job Drawer** is where this becomes unmistakably “LogLine flagship”: the user can _see the whole trajectory_ (cards + state + tools + approvals) without leaving chat.

Below is a **complete Job Drawer spec**: layout, data contract, rendering rules, and the exact event mapping from the ledger.

* * *

Job Drawer Spec (V1)
====================

Purpose
-------

A **side panel / bottom sheet** opened by tapping:

*   any job card
*   the job chip in the header (“Job: Schedule call…”)
*   the short `job_id` label in a card

It shows the job’s **single source of truth**:

*   lifecycle (proposed → approved → …)
*   approvals/disputes
*   tool calls and results
*   artifacts produced
*   a compact event trail

It never blocks chat; it’s a viewer + action surface.

* * *

1) UI layout (WhatsApp-native but professional)
===============================================

### Header

*   **Job title** (e.g., “Schedule call with Maria”)
*   **State pill** (PROPOSED / IN PROGRESS / WAITING / DONE / FAILED)
*   **Owner** (Office Scheduler)
*   **Job ID** (short, tap to copy)
*   **Last updated** timestamp

### Sections (in this order)

1.  **Summary**
    *   Goal (1–2 lines)
*   Constraints (chips)
*   Due / ETA (if present)
*   “Created by” + time
    2.  **Actions**
    *   A row of context-aware buttons (derived from current state)
    *   If `proposed`: Approve / Reject / Request changes
    *   If `waiting_input`: Provide info / Cancel
    *   If `in_progress`: Cancel / Dispute
    *   If `completed`: Accept / Dispute / Follow-up
        3.  **Timeline**  
    A vertical event timeline (most recent at top by default):
    *   State changes
*   Tool calls/results (expandable)
*   Card emissions
*   Disputes / approvals  
    Each item shows: icon, title, timestamp, actor
    4.  **Artifacts**  
    List of artifacts with open/copy:
    *   Calendar link
*   Meeting link
*   Files  
    Each artifact shows which event produced it
    5.  **Audit (compact)**  
    A collapsible “raw trail” view:
    *   event\_id (short)
*   event\_type
*   ts
*   trace\_id (short)
*   “Open details” (shows JSON)

V1: keep “Open details” simple (JSON pretty print).

* * *

2) Data contract: `GET /v1/jobs/{job_id}` (Gateway → UI)
========================================================

This is a **projection** built from the UBL ledger event chain.

```ts
type JobDrawerResponse = {
  tenant_id: string;
  conversation_id: string;
  job_id: string;

  title: string;
  goal: string;

  state: JobState;
  owner: { entity_id: string; display_name: string; actor_type: "human" | "agent" };

  created_at: ISO8601;
  updated_at: ISO8601;

  constraints?: string[];
  due_at?: ISO8601;
  priority?: "low" | "normal" | "high" | "urgent";

  // Current available actions (so UI doesn’t invent logic)
  available_actions: CardButton[];     // same shape as card buttons

  // Timeline items already normalized for UI
  timeline: JobTimelineItem[];

  // Derived artifacts across the chain
  artifacts: ArtifactRef[];

  // Raw ledger slice (optional in v1 but recommended)
  raw_events?: Array<{
    event_id: string;
    event_type: string;
    ts: ISO8601;
    actor: { entity_id: string; actor_type: ActorType };
    trace_id: string;
    payload: object;
  }>;
};
```

* * *

3) Timeline item normalization
==============================

```ts
type JobTimelineItem =
  | {
      kind: "state";
      ts: ISO8601;
      actor_name: string;
      prev_state: JobState;
      next_state: JobState;
      reason_code?: string;
      note?: string;
      event_id: string;
    }
  | {
      kind: "approval";
      ts: ISO8601;
      actor_name: string;
      action: "approved" | "rejected" | "acknowledged" | "cancelled" | "disputed";
      reason?: string;
      event_id: string;
    }
  | {
      kind: "tool";
      ts: ISO8601;
      actor_name: string;
      tool_name: string;
      status: "called" | "success" | "error";
      latency_ms?: number;
      purpose?: string;
      inputs_preview?: Record<string, string | number | boolean>;
      output_preview?: Record<string, string | number | boolean>;
      error_safe?: { error_code: string; message_safe: string; retryable: boolean };
      tool_call_id: string;
      attempt?: number;
      event_id: string;
    }
  | {
      kind: "card";
      ts: ISO8601;
      actor_name: string;
      card_type: "job.formalize" | "job.tracking" | "job.finished";
      title: string;
      summary?: string;
      card_id: string;
      event_id: string;
    }
  | {
      kind: "message";
      ts: ISO8601;
      actor_name: string;
      body_text: string;
      event_id: string;
    };
```

* * *

4) Mapping: UBL events → drawer timeline items
==============================================

Here’s the authoritative mapping:

### `job.state_changed` → `timeline.kind = "state"`

*   Use `prev_state`, `next_state`, `reason_code`, `note`

### `job.approved` → `approval: approved`

### `job.rejected` → `approval: rejected`

### `job.completed` → `card: job.finished` AND (optional) `state` already “completed”

### `job.progress` → `card: job.tracking` (if it embeds card)

### `job.proposed` → `card: job.formalize`

### `tool.called` → `tool: called`

*   show `purpose`, `inputs` preview (sanitized), `attempt`

### `tool.result`

*   if success: `tool: success`, show `output` preview + artifacts
*   if error: `tool: error`, show `error_safe`, retryable

### `message.sent` with `kind=text` and `job_id` present (optional)

*   show as `message` if you want richer context in the drawer
*   otherwise skip in v1

* * *

5) Rendering rules (so it’s clean)
==================================

Icons (v1)
----------

*   state: ⇄
*   approval: ✓ / ✕ / ! (dispute)
*   tool: 🔧 (called), ✅ (success), ⚠️ (error)
*   card: 🧾
*   artifact: 🔗 / 📄

Collapse rules
--------------

*   Show last **12 timeline items** by default
*   “Show all” expands
*   Tool items are expandable:
    *   collapsed: “Created invite — success (4.8s)”
    *   expanded: inputs preview, outputs preview, tool\_call\_id, event\_id

Preview redaction rules
-----------------------

*   Never show raw email/phone in inputs preview
*   Emails show as `m***@acme.com`
*   Show `email_hash` if present
*   Keep previews under ~6 fields

* * *

6) Available actions derivation (important)
===========================================

UI must not guess which buttons exist. The drawer uses:

*   `available_actions` from the gateway projection (which itself is derived from current state + last card)

### Default action sets by state (v1)

*   `proposed`: Approve / Reject / Request changes / Ask in chat
*   `waiting_input`: Provide info / Cancel / Ask in chat
*   `in_progress`: Got it / Dispute / Cancel / Ask in chat
*   `completed`: Accept / Dispute / Follow-up / Ask in chat
*   `failed`: Try later / Switch tool / Cancel / Ask in chat

“Try later” can be implemented as `chat.ask` in v1:

> “Create a follow-up job to retry in 30 minutes.”

* * *

7) One example: how the drawer looks for the scheduling job
===========================================================

### Header

*   “Schedule call with Maria” \[DONE\]
*   Owner: Office Scheduler
*   Job: `job_sched_4c1b` (tap to copy)

### Timeline items (top-down)

*   ✅ Tool: calendar.create\_invite — success (4.8s)
*   🔧 Tool: calendar.create\_invite — called
*   ⇄ State: waiting\_input → in\_progress (inputs\_received)
*   🧾 Card: Tracking — “Waiting for Maria email + preferred times”
*   ⇄ State: in\_progress → waiting\_input (missing\_required\_inputs)
*   ✓ Approved by Dan
*   🧾 Card: Formalize — “Approve to start…”

### Artifacts

*   “Calendar invite (Google Meet)” \[Open\] \[Copy link\]

* * *

8) Tiny but huge: the “Job permalink”
=====================================

Every job drawer has a “Copy link” button that copies:

*   `https://messenger.yourdomain/jobs/{job_id}`  
    This deep-links directly into the drawer inside the conversation.

Even in local dev, it’s gold.

* * *

If you want, the next “YES” and I’ll give you the **presence derivation rules** (exactly how agents become “Working / Waiting on you / Available” purely from ledger + job state, so presence is computable and not a random UI flag).

## Prompt:
YES

## Response:
Love it. Here are **presence derivation rules** that are **100% computable from ledger events + job state**, no random flags. This makes agent “coworker dignity” real: presence is an _audit-backed signal_, not vibes.

* * *

Presence Derivation Spec (V1)
=============================

Goal
----

For each `entity_id` (human or agent) in a `tenant_id`, produce:

```ts
type PresenceState = "available" | "working" | "waiting_on_you" | "offline";

type Presence = {
  tenant_id: string;
  entity_id: string;
  state: PresenceState;

  // optional UI details
  job_id?: string;
  job_title?: string;
  since?: ISO8601;                // when this state began
  detail?: string;                // "Working on Schedule call..."
};
```

Presence is derived from:

*   recent activity (`message.sent`, `tool.called`, `job.state_changed`)
*   job ownership and waiting flags
*   last-seen timestamps
*   optional heartbeat pings (nice-to-have)

* * *

1) Definitions
==============

### “Last seen”

For an entity, define:

*   `last_seen_ts(entity)` = max(ts of any event where actor.entity\_id == entity)

Events that count:

*   `message.sent`
*   `job.*`
*   `tool.*`
*   optional `presence.ping` (if you add later)

### “Active job”

A job is active if its latest state is one of:

*   `proposed`
*   `approved`
*   `in_progress`
*   `waiting_input`

A job is closed if:

*   `completed`
*   `rejected`
*   `cancelled`
*   `failed`

### “Job owner”

Owner comes from the latest known job definition (from `job.created` or embedded cards):

*   `owner_entity_id`

* * *

2) Offline rule (simple + strict)
=================================

If `now - last_seen_ts(entity) > OFFLINE_TTL`, then:

*   `state = offline`

**Default OFFLINE\_TTL:**

*   Humans: **30 minutes**
*   Agents: **5 minutes**

Why different? Agents should look “alive” only when they’re actually doing things.

* * *

3) Waiting-on-you rule (strongest signal)
=========================================

If there exists an **active job** where:

*   job state == `waiting_input`
*   and **the waiting target includes the human** (or the assigned input provider)  
    then for that human:
*   `state = waiting_on_you`
*   `job_id = that job`
*   `detail = "Needs your input"`

How do we know the waiting target in v1?

*   From the latest TrackingCard (`job.progress`) field:
    *   `tracking_card.progress.waiting_on[]`

If `waiting_on` is absent, fallback:

*   treat the last actor who can unblock (usually the human who approved) as the waiting target _only if_ job reason\_code indicates missing inputs. But prefer explicit `waiting_on`.

**Priority:** waiting\_on\_you overrides everything for humans (except offline).

* * *

4) Working rule (for agents + humans)
=====================================

If there exists an **active job** where:

*   job state == `in_progress`
*   AND `owner_entity_id == entity_id`  
    then:
*   `state = working`
*   `job_id = that job`
*   `detail = "Working on {job_title}"`

**Which job if multiple?**  
Pick the one with:

1.  most recent `tool.called` ts
2.  else most recent `job.state_changed` ts
3.  else most recent `job.progress` ts

This yields a stable “currently working on…” label.

* * *

5) Proposed rule (optional but nice)
====================================

If an agent has only `proposed` jobs and no `in_progress`:  
You can either show `available` or `working`.  
For v1, keep it clean:

*   treat `proposed` as **available** (but you can show a small chip: “Has proposals”)

* * *

6) Available rule (default)
===========================

If not offline, not waiting\_on\_you, not working:

*   `state = available`

For humans, available can just be “online” conceptually, but the UI label can remain “Available.”

* * *

7) Presence emission (SSE / WS updates)
=======================================

Presence should update when:

*   `job.state_changed`
*   `job.progress` (waiting\_on changed)
*   `tool.called` / `tool.result` (agent activity)
*   `message.sent` (activity)

### SSE payload

```json
{
  "type": "presence.update",
  "tenant_id": "tnt_acme_001",
  "entity_id": "ent_agent_scheduler",
  "presence": "working",
  "job_id": "job_sched_4c1b",
  "detail": "Working on Schedule call with Maria",
  "since": "2025-12-27T10:15:24.000Z"
}
```

* * *

8) UI mapping rules (so it feels right)
=======================================

Conversation list (agent row)
-----------------------------

*   `working`: show spinner + “Working on {job\_title}”
*   `waiting_on_you`: amber dot + “Needs your input”
*   `available`: green dot + “Available”
*   `offline`: gray dot + “Offline”

Conversation header
-------------------

If the other party is an agent:

*   show a small chip: `Job: {job_title}` when working/waiting\_on\_you
*   clicking opens Job Drawer for that `job_id`

* * *

9) The “dignity” detail: don’t lie with presence
================================================

Presence must never claim “Working” without evidence.  
**Evidence types:**

*   `tool.called` in last X minutes for that job
*   or `job.state_changed` to `in_progress` recently
*   or `job.progress` updates recently

If a job is `in_progress` but there’s no activity beyond a “stale TTL”:

*   degrade to `available` + detail “Paused” (optional)  
    For v1, simplest:
*   if no activity for **AGENT\_STALE\_TTL = 3 minutes**, set agent to `available` unless still `waiting_input`.

* * *

10) Minimal projection algorithm (one pass)
===========================================

For a tenant:

1.  Build job state table keyed by `job_id` (latest state, owner, title, last\_activity\_ts, waiting\_on\[\])
2.  For each entity:
    *   compute last\_seen\_ts from ledger
    *   if offline → offline
    *   else if entity is human and any job waiting\_on includes them → waiting\_on\_you (pick most recent)
    *   else if entity owns any in\_progress job → working (pick most recent)
    *   else available
3.  Emit presence.update when the computed presence differs from previous.

* * *

If you want, next “YES” and I’ll give you the **projection tables** (what the Messenger Gateway should cache as derived views: `jobs_by_conversation`, `timeline_by_conversation`, `presence_by_entity`, `job_event_chain`) with exact keys and indexes, so the implementation stays fast and rebuildable.

## Prompt:
YES

## Response:
Yes — here are the **projection tables/views** the Messenger Gateway should maintain. They’re designed to be:

*   **Fast** (UI queries are O(1) by conversation/job/entity)
*   **Rebuildable** (can be regenerated from the ledger tail)
*   **Wireable** (SSE can emit deltas from these projections)
*   **Minimal** (v1 only, no fat)

I’m describing them as logical tables. You can implement them as:

*   Postgres tables,
*   Redis hashes + lists,
*   in-memory maps with periodic snapshot,
*   or UBL container projections.

The key is: **keys + indexes + update rules** are stable.

* * *

0) Projection philosophy (v1)
=============================

*   Ledger is append-only truth.
*   Gateway maintains projections as **read-optimized views**.
*   Every projection record includes:
    *   `tenant_id`
    *   `updated_at`
    *   `source_event_id` (last applied)
    *   optional `seq` (monotonic per-tenant)

* * *

1) `projection_conversations`
=============================

**Purpose:** drive the conversation list (WhatsApp left sidebar).

**Primary key:** `(tenant_id, conversation_id)`

**Fields:**

```ts
{
  tenant_id: string;
  conversation_id: string;

  title: string;                       // "Maria", "Ops Team"
  participant_entity_ids: string[];    // includes humans + agents

  last_item_ts?: ISO8601;
  last_preview?: {
    kind: "text" | "card" | "system";
    text?: string;                     // short snippet
    job_id?: string;
    card_type?: "job.formalize" | "job.tracking" | "job.finished";
  };

  unread_count_by_entity?: Record<EntityId, number>; // optional v1

  // presence hints for list rows:
  agent_presence?: {
    entity_id: string;
    state: "available" | "working" | "waiting_on_you" | "offline";
    job_id?: string;
    detail?: string;
  }[];

  updated_at: ISO8601;
  source_event_id: string;
}
```

**Indexes:**

*   `(tenant_id, last_item_ts DESC)`
*   `(tenant_id, participant_entity_ids)` (optional depending on DB)

**Update triggers:**

*   `message.sent` → update `last_item_ts` + `last_preview`
*   presence changes → update embedded `agent_presence` (or compute client-side)

* * *

2) `projection_timeline_items`
==============================

**Purpose:** render the chat timeline quickly.

**Primary key:** `(tenant_id, conversation_id, timeline_cursor)`  
Where `timeline_cursor` is:

*   either `ts+event_id` lexicographically,
*   or a monotonic `seq`.

**Fields:**

```ts
{
  tenant_id: string;
  conversation_id: string;

  cursor: string;               // "2025-12-27T10:15:06.300Z#evt_0009"
  ts: ISO8601;
  event_id: string;

  sender: {
    entity_id: string;
    display_name: string;
    actor_type: "human" | "agent" | "system";
  };

  item: {
    kind: "text" | "card" | "system";
    body_text?: string;
    card?: JobCard;             // exact card payload
  };

  // linkage
  job_id?: string;

  updated_at: ISO8601;
  source_event_id: string;
}
```

**Indexes:**

*   `(tenant_id, conversation_id, ts DESC)`
*   `(tenant_id, conversation_id, job_id, ts DESC)` (for “jump to job”)

**Update triggers:**

*   `message.sent` always inserts one timeline row.
*   Optionally, `job.progress` / `job.completed` can also insert timeline rows **only** via `message.sent` (recommended), so timeline stays consistent.

* * *

3) `projection_jobs`
====================

**Purpose:** quick job header state + job chip in header + job list per conversation.

**Primary key:** `(tenant_id, job_id)`

**Fields:**

```ts
{
  tenant_id: string;
  job_id: string;

  conversation_id: string;

  title: string;
  goal: string;
  state: JobState;

  owner: { entity_id: string; display_name: string; actor_type: "human" | "agent" };

  created_at: ISO8601;
  updated_at: ISO8601;

  priority?: "low" | "normal" | "high" | "urgent";
  due_at?: ISO8601;
  constraints?: string[];

  // waiting signal:
  waiting_on?: Array<{ entity_id: string; display_name: string }>;

  // activity signals:
  last_activity_at?: ISO8601;
  last_tool_call_id?: string;
  last_error_code?: string;

  // current action set (for Job Drawer + maybe timeline quick actions)
  available_actions: CardButton[];

  source_event_id: string;
}
```

**Indexes:**

*   `(tenant_id, conversation_id, updated_at DESC)`
*   `(tenant_id, state, updated_at DESC)`
*   `(tenant_id, owner.entity_id, state)` (for agent working list)

**Update triggers:**

*   `job.created` initializes
*   `job.proposed` fills title/goal/constraints, sets state=proposed
*   `job.approved/rejected/cancelled` updates state + actions
*   `job.state_changed` updates state (+ timestamps)
*   `job.progress` updates `waiting_on`, `last_activity_at`, maybe actions
*   `tool.called/result` updates `last_activity_at`, `last_tool_call_id`, `last_error_code`
*   `job.completed` sets state=completed and actions for Finished

* * *

4) `projection_job_events`
==========================

**Purpose:** reconstruct the job drawer quickly without querying the whole ledger.

**Primary key:** `(tenant_id, job_id, cursor)`  
Again cursor = `ts#event_id` or `seq`.

**Fields:**

```ts
{
  tenant_id: string;
  job_id: string;

  cursor: string;
  ts: ISO8601;
  event_id: string;
  event_type: string;

  actor: { entity_id: string; actor_type: "human" | "agent" | "system" };

  // normalized drawer item:
  timeline_item: JobTimelineItem;

  // optional raw payload snippet for “Open details”
  raw_payload?: object;

  source_event_id: string;
}
```

**Indexes:**

*   `(tenant_id, job_id, ts DESC)`
*   `(tenant_id, job_id, event_type, ts DESC)`

**Update triggers:**

*   any event with `job_id` inserts a normalized job timeline item:
    *   `job.state_changed`, `job.approved`, `tool.called`, `tool.result`, `job.progress`, `job.completed`, etc.

* * *

5) `projection_job_artifacts`
=============================

**Purpose:** fast artifact list for job drawer + finished card.

**Primary key:** `(tenant_id, job_id, artifact_id)`

**Fields:**

```ts
{
  tenant_id: string;
  job_id: string;

  artifact_id: string;
  kind: "file" | "link" | "record" | "quote";
  title: string;
  url?: string;
  mime_type?: string;
  size_bytes?: number;

  produced_by_event_id: string;
  produced_at: ISO8601;

  source_event_id: string;
}
```

**Indexes:**

*   `(tenant_id, job_id, produced_at DESC)`

**Update triggers:**

*   `tool.result` (success) → upsert artifacts
*   `job.completed` → upsert artifacts (if card includes them)

* * *

6) `projection_presence`
========================

**Purpose:** entity presence, computable.

**Primary key:** `(tenant_id, entity_id)`

**Fields:**

```ts
{
  tenant_id: string;
  entity_id: string;

  state: "available" | "working" | "waiting_on_you" | "offline";

  job_id?: string;
  job_title?: string;

  since: ISO8601;
  last_seen_at: ISO8601;

  detail?: string;

  source_event_id: string;
}
```

**Indexes:**

*   `(tenant_id, state)`
*   `(tenant_id, last_seen_at DESC)`

**Update triggers:**

*   Any relevant ledger event updates `last_seen_at` for actor
*   Job state/progress updates recompute presence for:
    *   job owner (agent)
    *   waiting\_on humans

* * *

7) `projection_entities`
========================

**Purpose:** resolve display names/avatars/roles fast.

**Primary key:** `(tenant_id, entity_id)`

**Fields:**

```ts
{
  tenant_id: string;
  entity_id: string;

  actor_type: "human" | "agent";
  display_name: string;
  role?: string;                    // "Scheduler", "Ops"
  avatar_url?: string;

  // agent capabilities summary (for UI badges)
  capabilities?: string[];

  updated_at: ISO8601;
  source_event_id: string;
}
```

**Update triggers:**

*   `entity.registered`, `entity.updated` (if you add)

* * *

8) The projection updater (how to build/rebuild)
================================================

Inputs
------

*   Consume ledger stream in order (`ts#event_id` or `seq`).
*   Maintain per-tenant cursor `projection_cursor`.

Process
-------

For each event:

1.  Apply to relevant projections:
    *   if `message.sent` → timeline + conversation preview
    *   if `job.*` → jobs + job\_events (+ maybe timeline if it’s also a message)
    *   if `tool.*` → job\_events + jobs + artifacts
2.  Recompute presence for affected entities:
    *   actor
    *   job owner
    *   waiting\_on targets
3.  Emit SSE deltas:
    *   `timeline.append`
    *   `job.update`
    *   `presence.update`
    *   `conversation.update` (optional)

Rebuild
-------

To rebuild projections:

*   clear tables
*   replay ledger from genesis for tenant
*   set `projection_cursor` to tail

V1 doesn’t need fancy snapshots, but you can add periodic snapshots later.

* * *

9) Minimal endpoints powered by these projections
=================================================

*   `GET /v1/conversations?tenant_id=...` → `projection_conversations`
*   `GET /v1/conversations/{id}/timeline?tenant_id=...&cursor=...` → `projection_timeline_items`
*   `GET /v1/jobs/{job_id}?tenant_id=...` → join `projection_jobs + projection_job_events + projection_job_artifacts`
*   `GET /v1/presence?tenant_id=...` → `projection_presence`
*   `GET /v1/stream?tenant_id=...` → SSE emitted from deltas

* * *

10) One small “flagship” touch
==============================

In the conversation header, show:

*   “Active job: {title}” if `projection_jobs` has any active job for this conversation
*   Clicking opens drawer (via `projection_jobs.job_id`)

That gives the whole system a “work OS” vibe without clutter.

* * *

If you want to keep the momentum: next “YES” and I’ll give you the **exact SSE delta protocol** (event names, payloads, ordering guarantees, reconnection strategy with cursor) so the UI never gets out of sync.

## Prompt:
YES

## Response:
Perfect. Here’s the **exact SSE delta protocol** for Messenger Gateway → UI: event names, payloads, ordering guarantees, reconnection strategy, and how to avoid desync forever.

This is v1, but it’s already “production-shaped.”

* * *

SSE Delta Protocol (V1)
=======================

Endpoint
--------

`GET /v1/stream?tenant_id=...&cursor=...`

*   `tenant_id` required
*   `cursor` optional (for resume). If omitted, server may:
    *   start from “now” (tail-only), or
    *   send a small “recent backlog” (recommended: last 50 timeline items)

### Headers

*   `Content-Type: text/event-stream`
*   `Cache-Control: no-cache`
*   `Connection: keep-alive`

* * *

1) Message format (standard SSE)
================================

Each SSE frame:

```
id: <cursor>
event: <event_name>
data: <json>
```

*   `id` is **the cursor** the client should store for resume
*   `event` is one of the event names below
*   `data` is JSON

* * *

2) The cursor (the thing that makes reconnection bulletproof)
=============================================================

### Cursor format

Use a monotonic per-tenant sequence:

*   `cursor = "seq:<number>"` (best)  
    If you don’t have per-tenant seq yet:
*   `cursor = "<ISO8601>#<event_id>"`

**Recommendation (strong):** keep a per-tenant `seq` in projections. It makes ordering + resume trivial.

### Ordering guarantee

Gateway guarantees:

*   SSE events are sent in **strict cursor order**
*   any event sent has a cursor **greater than** all previously sent events for that tenant

* * *

3) Event names + payloads
=========================

3.1 `hello`
-----------

Sent immediately after connection.

**event:** `hello`  
**id:** current cursor (tail cursor)

```json
{
  "tenant_id": "tnt_acme_001",
  "server_time": "2025-12-27T10:15:06.400Z",
  "cursor": "seq:18420",
  "capabilities": {
    "supports_resume": true,
    "supports_heartbeat": true
  }
}
```

* * *

3.2 `timeline.append`
---------------------

A new timeline item for a conversation (text or card).

```json
{
  "tenant_id": "tnt_acme_001",
  "conversation_id": "cnv_9f2a",
  "item": {
    "kind": "message",
    "ts": "2025-12-27T10:15:06.300Z",
    "event_id": "evt_0009",
    "sender": {
      "entity_id": "ent_agent_scheduler",
      "display_name": "Office Scheduler",
      "actor_type": "agent"
    },
    "message": {
      "message_id": "msg_0003",
      "kind": "card",
      "card": { "...": "JobCard payload" }
    }
  }
}
```

Client behavior:

*   If UI is currently in that conversation, append to timeline.
*   Otherwise, update conversation list preview + unread count.

* * *

3.3 `conversation.update`
-------------------------

Optional but useful: changes in conversation list metadata.

```json
{
  "tenant_id": "tnt_acme_001",
  "conversation": {
    "conversation_id": "cnv_9f2a",
    "title": "Maria",
    "last_item_ts": "2025-12-27T10:15:06.300Z",
    "last_preview": {
      "kind": "card",
      "text": "Waiting for details…",
      "job_id": "job_sched_4c1b",
      "card_type": "job.tracking"
    }
  }
}
```

Client behavior:

*   update conversation row in list

* * *

3.4 `job.update`
----------------

A job header state changed (used for chips, drawer header, list badges).

```json
{
  "tenant_id": "tnt_acme_001",
  "job": {
    "job_id": "job_sched_4c1b",
    "conversation_id": "cnv_9f2a",
    "title": "Schedule call with Maria",
    "state": "waiting_input",
    "updated_at": "2025-12-27T10:15:06.000Z",
    "waiting_on": [{ "entity_id": "ent_human_dan", "display_name": "Dan" }],
    "available_actions": [ { "...": "CardButton" } ]
  }
}
```

Client behavior:

*   update job drawer if open
*   update header “active job chip” if present
*   update any “job badge” on conversation row

* * *

3.5 `presence.update`
---------------------

Presence changes for coworker list + conversation list.

```json
{
  "tenant_id": "tnt_acme_001",
  "entity_id": "ent_agent_scheduler",
  "presence": {
    "state": "working",
    "job_id": "job_sched_4c1b",
    "job_title": "Schedule call with Maria",
    "since": "2025-12-27T10:15:24.000Z",
    "detail": "Working on Schedule call with Maria"
  }
}
```

* * *

3.6 `artifact.added`
--------------------

Optional (nice for drawer + finished card).  
If you already send artifacts inside `job.update`, you can skip this.

```json
{
  "tenant_id": "tnt_acme_001",
  "job_id": "job_sched_4c1b",
  "artifact": {
    "artifact_id": "art_cal_001",
    "kind": "link",
    "title": "Calendar invite (Google Meet)",
    "url": "https://calendar.example/invite/abc123",
    "produced_at": "2025-12-27T10:15:29.000Z"
  }
}
```

* * *

3.7 `heartbeat`
---------------

Sent periodically to keep the connection alive and give clients a time sync point.

```json
{
  "tenant_id": "tnt_acme_001",
  "server_time": "2025-12-27T10:16:00.000Z"
}
```

**Default interval:** every 15 seconds.

* * *

3.8 `error`
-----------

Used for non-fatal stream errors (auth issues, cursor invalid).

```json
{
  "tenant_id": "tnt_acme_001",
  "code": "CURSOR_TOO_OLD",
  "message": "Resume cursor is no longer available. Please resync.",
  "recommended_action": "resync"
}
```

* * *

4) Reconnection strategy (client-side)
======================================

Client stores the latest `id` cursor received.

### On disconnect:

*   reconnect with: `/v1/stream?tenant_id=...&cursor=<last_cursor>`

### If server responds with `CURSOR_TOO_OLD` or 410:

Client performs a **resync**:

1.  `GET /v1/conversations?tenant_id=...` (refresh list)
2.  For active conversation, `GET /v1/conversations/{id}/timeline?tenant_id=...&cursor=<optional>`
3.  For open drawer, `GET /v1/jobs/{job_id}?tenant_id=...`
4.  Reconnect stream with new cursor returned from `hello`

* * *

5) Server retention requirements (so resume works)
==================================================

To support resume, server must retain a **cursor-addressable delta log**.

V1 approach:

*   store last **N deltas** in memory or Redis per tenant
*   N = enough for ~5 minutes of activity OR ~1000 events (choose your scale)

If cursor older than retention:

*   respond `error CURSOR_TOO_OLD` and force resync.

* * *

6) Exactly-once UI semantics (the anti-duplication rule)
========================================================

Client must treat `event_id` inside `timeline.append.item.event_id` as unique.

*   If already seen, ignore (dedupe)

Same for:

*   `job.update.job.job_id + updated_at` (or include version)
*   `presence.update.entity_id + since` (or include version)

This prevents duplicate inserts when reconnecting.

* * *

7) One subtle rule that saves you later: “SSE emits projections, not raw ledger”
================================================================================

SSE payloads should come from **Messenger Gateway projections**, not raw UBL events.  
Why:

*   UI gets stable shapes (timeline items, job headers, presence)
*   you can change ledger internals without breaking the UI
*   projection can include display names already resolved

Ledger remains available for audit/details in Job Drawer’s “raw” section.

* * *

8) Minimal “wire test” (Proof of Done for SSE)
==============================================

When you run the golden job:

*   you see **timeline.append** for each message/card
*   you see **job.update** as the job transitions
*   you see **presence.update** when agent starts/blocks/completes
*   and reconnecting resumes without losing events

* * *

If you want one more “flagship hardcore” piece: next “YES” and I’ll give you the **REST command contracts** (exact request/response for `POST /messages`, `POST /job actions`, idempotency keys, and how the gateway turns them into Office calls + UBL appends).

## Prompt:
YES

## Response:
Alright — here are the **REST command contracts** (v1) that make the wiring clean: UI → Messenger Gateway → Office → UBL. This includes **idempotency**, **responses**, and **how each request becomes ledger writes + SSE deltas**.

* * *

0) Cross-cutting rules (apply to every endpoint)
================================================

Headers
-------

*   `Authorization: Bearer <token>` _(or your session cookie; v1 can be simple)_
*   `Content-Type: application/json`
*   **`Idempotency-Key: <string>`** _(required for all POSTs that cause writes)_
*   Optional: `X-Trace-Id: <trace_id>` _(if client doesn’t send it in body)_

Idempotency-Key format (recommendation)
---------------------------------------

*   `idem:<tenant_id>:<conversation_id>:<client_action_id>`
*   Or for job actions: `idem:<tenant_id>:job:<job_id>:<action>:<client_action_id>`

**Behavior**

*   Gateway stores: `(tenant_id, idempotency_key) → {status, response_body, created_event_ids}`
*   If the same key arrives again: **return same response**, do not re-execute Office/tools.

Standard error shape
--------------------

```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Missing field: tenant_id",
    "details": { "field": "tenant_id" }
  }
}
```

Common codes:

*   `UNAUTHORIZED`
*   `FORBIDDEN`
*   `NOT_FOUND`
*   `VALIDATION_ERROR`
*   `CONFLICT`
*   `RATE_LIMITED`
*   `UPSTREAM_ERROR`

* * *

1) The Gateway is a command router + projection reader
======================================================

### Gateway responsibilities

*   Accept client commands
*   Validate tenancy/auth
*   Forward to Office (for intelligence + job transitions)
*   Append resulting events to UBL (if Office doesn’t append directly)
*   Update projections
*   Fan out SSE deltas

There are **two valid wiring styles**:

### Style A (recommended): **Office writes to UBL**

*   Gateway calls Office
*   Office writes ledger events
*   Gateway consumes ledger stream and updates projections → SSE

### Style B: **Gateway writes to UBL**

*   Gateway calls Office
*   Office returns “events to emit”
*   Gateway appends them to UBL
*   Projections update → SSE

V1 can do either. The HTTP contracts below work for both.

* * *

2) Commands: UI → Messenger Gateway
===================================

2.1 Create conversation
-----------------------

`POST /v1/conversations`

**Request**

```json
{
  "tenant_id": "tnt_acme_001",
  "trace_id": "trc_20251227_010",
  "title": "Maria",
  "participants": [
    { "entity_id": "ent_human_dan" },
    { "entity_id": "ent_agent_scheduler" }
  ]
}
```

**Response (201)**

```json
{
  "conversation_id": "cnv_9f2a",
  "created_event_ids": ["evt_conv_created_01"],
  "cursor": "seq:18420"
}
```

* * *

2.2 Send a message (text or card action message)
------------------------------------------------

`POST /v1/conversations/{conversation_id}/messages`

### A) Text message

**Request**

```json
{
  "tenant_id": "tnt_acme_001",
  "trace_id": "trc_20251227_011",
  "actor_entity_id": "ent_human_dan",
  "kind": "text",
  "body_text": "Can you schedule a 30-min call with Maria next week?"
}
```

**Response (202 Accepted)**  
_(We return fast; updates arrive via SSE.)_

```json
{
  "accepted": true,
  "conversation_id": "cnv_9f2a",
  "client_action": "message.send",
  "created_event_ids": ["evt_0001"],
  "cursor": "seq:18421"
}
```

### B) Card action as a message (optional)

You can also treat button clicks as “messages” in the timeline. Usually better to use `/job_actions` below, but this is allowed for generic actions.

**Request**

```json
{
  "tenant_id": "tnt_acme_001",
  "trace_id": "trc_20251227_012",
  "actor_entity_id": "ent_human_dan",
  "kind": "card_action",
  "job_id": "job_sched_4c1b",
  "card_id": "card_tracking_001",
  "button_id": "btn_input_001",
  "action": { "type": "job.provide_input", "job_id": "job_sched_4c1b" },
  "input": {
    "maria_email": "maria@acme.com",
    "time_window": "Tue–Thu, 14:00–17:00",
    "timezone": "Europe/Lisbon",
    "meeting_link": "google_meet"
  }
}
```

**Response (202)**

```json
{
  "accepted": true,
  "created_event_ids": ["evt_user_action_01"],
  "cursor": "seq:18422"
}
```

* * *

2.3 Job actions (the canonical way to handle buttons)
-----------------------------------------------------

`POST /v1/jobs/{job_id}/actions`

**Request**

```json
{
  "tenant_id": "tnt_acme_001",
  "trace_id": "trc_20251227_013",
  "conversation_id": "cnv_9f2a",

  "actor_entity_id": "ent_human_dan",
  "card_id": "card_formalize_001",
  "button_id": "btn_approve_001",

  "action": { "type": "job.approve", "job_id": "job_sched_4c1b" }
}
```

**Response (202)**

```json
{
  "accepted": true,
  "job_id": "job_sched_4c1b",
  "created_event_ids": ["evt_0005", "evt_0006"],
  "cursor": "seq:18423"
}
```

### With input (Provide info / Dispute / Request changes)

**Request**

```json
{
  "tenant_id": "tnt_acme_001",
  "trace_id": "trc_20251227_014",
  "conversation_id": "cnv_9f2a",
  "actor_entity_id": "ent_human_dan",
  "card_id": "card_tracking_001",
  "button_id": "btn_input_001",
  "action": { "type": "job.provide_input", "job_id": "job_sched_4c1b" },
  "input": {
    "maria_email": "maria@acme.com",
    "time_window": "Tue–Thu, 14:00–17:00",
    "timezone": "Europe/Lisbon",
    "meeting_link": "google_meet"
  }
}
```

**Response**

```json
{
  "accepted": true,
  "job_id": "job_sched_4c1b",
  "created_event_ids": ["evt_0010"],
  "cursor": "seq:18424"
}
```

* * *

3) Reads: UI → Gateway (projections)
====================================

3.1 Conversation list
---------------------

`GET /v1/conversations?tenant_id=...`

**Response**

```json
{
  "tenant_id": "tnt_acme_001",
  "items": [
    {
      "conversation_id": "cnv_9f2a",
      "title": "Maria",
      "last_item_ts": "2025-12-27T10:15:06.300Z",
      "last_preview": { "kind": "card", "text": "Waiting for details…", "job_id": "job_sched_4c1b" },
      "agent_presence": [
        { "entity_id": "ent_agent_scheduler", "state": "waiting_on_you", "job_id": "job_sched_4c1b", "detail": "Needs your input" }
      ]
    }
  ],
  "cursor": "seq:18430"
}
```

3.2 Timeline
------------

`GET /v1/conversations/{conversation_id}/timeline?tenant_id=...&cursor=...`

**Response**

```json
{
  "tenant_id": "tnt_acme_001",
  "conversation_id": "cnv_9f2a",
  "items": [ /* TimelineItem[] */ ],
  "next_cursor": "seq:18430"
}
```

3.3 Job Drawer
--------------

`GET /v1/jobs/{job_id}?tenant_id=...`

**Response**

```json
{
  "tenant_id": "tnt_acme_001",
  "conversation_id": "cnv_9f2a",
  "job_id": "job_sched_4c1b",
  "title": "Schedule call with Maria",
  "goal": "Schedule a 30-minute call with Maria next week and send an invite",
  "state": "waiting_input",
  "owner": { "entity_id": "ent_agent_scheduler", "display_name": "Office Scheduler", "actor_type": "agent" },
  "created_at": "2025-12-27T10:15:01.000Z",
  "updated_at": "2025-12-27T10:15:06.200Z",
  "available_actions": [ /* CardButton[] */ ],
  "timeline": [ /* JobTimelineItem[] */ ],
  "artifacts": [],
  "cursor": "seq:18430"
}
```

* * *

4) How Gateway turns commands into Office calls + ledger writes
===============================================================

4.1 `POST /conversations/:id/messages` (text)
---------------------------------------------

**Flow**

1.  Validate auth + tenant membership
2.  Append `message.sent` to UBL (human message)
3.  Call Office: `POST /v1/office/ingest_message` with the same payload
4.  Office emits job/message/tool events (either writes to UBL itself or returns them)
5.  Gateway projection updater consumes new events → emits SSE deltas:
    *   `timeline.append` (for card messages)
    *   `job.update`
    *   `presence.update`
    *   `conversation.update`

**Gateway response**

*   `202 Accepted` with created\_event\_ids for the **initial human message** (and optionally more if synchronous)

4.2 `POST /jobs/:job_id/actions`
--------------------------------

**Flow**

1.  Validate auth + tenant membership
2.  Append `job.action_submitted` _(optional; can be skipped in v1)_ or just forward
3.  Call Office: `POST /v1/office/job_action`
4.  Office emits `job.approved` / `job.state_changed` / `job.progress` / `tool.called` / etc.
5.  Projections update → SSE deltas

* * *

5) Office contracts (Gateway → Office)
======================================

5.1 Ingest message
------------------

`POST /v1/office/ingest_message`

**Request**

```json
{
  "tenant_id": "tnt_acme_001",
  "trace_id": "trc_20251227_011",
  "conversation_id": "cnv_9f2a",
  "actor_entity_id": "ent_human_dan",
  "kind": "text",
  "body_text": "Can you schedule a 30-min call with Maria next week?"
}
```

**Response (200)**  
Style A (Office writes to UBL):

```json
{
  "ok": true,
  "emitted_event_ids": ["evt_0002", "evt_0003", "evt_0004"],
  "notes": "Proposed a job card"
}
```

Style B (Gateway writes to UBL):

```json
{
  "ok": true,
  "events_to_append": [ /* array of UblEvent envelopes */ ]
}
```

5.2 Job action
--------------

`POST /v1/office/job_action`

**Request**

```json
{
  "tenant_id": "tnt_acme_001",
  "trace_id": "trc_20251227_013",
  "conversation_id": "cnv_9f2a",
  "job_id": "job_sched_4c1b",
  "actor_entity_id": "ent_human_dan",
  "card_id": "card_formalize_001",
  "button_id": "btn_approve_001",
  "action": { "type": "job.approve", "job_id": "job_sched_4c1b" }
}
```

**Response (200)**

```json
{
  "ok": true,
  "emitted_event_ids": ["evt_0005", "evt_0006", "evt_0007", "evt_0008", "evt_0009"]
}
```

* * *

6) UBL append + query contracts (Gateway ↔ UBL)
===============================================

6.1 Append (single or batch)
----------------------------

`POST /v1/ledger/append`

**Request**

```json
{
  "tenant_id": "tnt_acme_001",
  "events": [ /* UblEvent[] */ ]
}
```

**Response**

```json
{
  "ok": true,
  "accepted_event_ids": ["evt_0001", "evt_0002"],
  "cursor": "seq:18421"
}
```

6.2 Query (projection-friendly)
-------------------------------

`GET /v1/ledger/query?tenant_id=...&conversation_id=...&job_id=...&after_cursor=...&limit=...`

**Response**

```json
{
  "tenant_id": "tnt_acme_001",
  "events": [ /* UblEvent[] */ ],
  "next_cursor": "seq:18421"
}
```

* * *

7) Important behavioral guarantees (the stuff that prevents pain later)
=======================================================================

Guarantee A: POST returns fast; UI relies on SSE for the “real story”
---------------------------------------------------------------------

Every command returns:

*   `accepted: true`
*   `cursor`
*   `created_event_ids` (at least the first event)

Then SSE drives the UI with:

*   `timeline.append`
*   `job.update`
*   `presence.update`

Guarantee B: UI never needs to poll if SSE is alive
---------------------------------------------------

Polling is only fallback.

Guarantee C: Idempotency covers retries + double clicks
-------------------------------------------------------

If user taps Approve twice, only one job transition occurs.

Guarantee D: Command validation is strict
-----------------------------------------

Gateway rejects:

*   wrong tenant\_id
*   missing Idempotency-Key
*   actions for job not in that conversation/tenant
*   unknown button\_id for card\_id (optional but recommended)

* * *

8) One “flagship” touch: optimistic UI without lying
====================================================

When user clicks a card button:

*   UI immediately appends a local “action bubble” (“Dan approved the job”)
*   When SSE brings the real event, UI reconciles (dedupe by event\_id)

If the command fails:

*   replace the bubble with “Couldn’t send — retry” (still in chat)

* * *

If you want to go one layer deeper: next “YES” and I’ll give you the **exact minimal event validation rules** (Container Logic checks) that make it impossible to corrupt job state (illegal transitions, wrong tenant, missing job\_id, invalid button provenance) — basically the guardrails that turn this into a real OS.

## Prompt:
YES

## Response:
Alright — here’s the **minimal Container Logic validation layer** that makes it **impossible to corrupt job state**. This is the guardrail set that turns the trio into a real OS.

Think of it as three gates:

1.  **Schema Gate** (is the event well-formed?)
2.  **Authority Gate** (is the actor allowed to do this?)
3.  **State Gate** (is this transition/legal sequence valid?)

Everything below is **deterministic and computable**.

* * *

0) Core invariants (apply to every event)
=========================================

Event envelope required fields
------------------------------

Reject if missing:

*   `event_id`, `event_type`, `ts`
*   `tenant_id`, `trace_id`, `conversation_id`
*   `actor.entity_id`, `actor.actor_type`
*   `payload` must be an object

Tenant isolation
----------------

Reject if:

*   event references any foreign tenant object (job\_id, conversation\_id, entity\_id) not belonging to `tenant_id`

Idempotency (mandatory for commands)
------------------------------------

For all write commands (not just ledger appends), require:

*   `Idempotency-Key` (at gateway level)  
    Ledger may also enforce:
*   `event_id` uniqueness per tenant

* * *

1) Schema Gate (per event type)
===============================

For each `event_type`, enforce strict schema:

### `message.sent`

*   payload must include:
    *   `message_id`
    *   `kind in {text, card, system}`
*   if `kind=text`: require `body_text`
*   if `kind=card`: require `card` and `job_id` present at envelope level **and** `card.job_id` matches

### `job.created`

*   require `job_id`, `title`, `owner_entity_id`, `conversation_id`

### `job.proposed`

*   require `job_id` and `proposed_card`
*   `proposed_card.card_type == job.formalize`
*   `proposed_card.job_id == envelope.job_id`
*   `proposed_card.buttons` non-empty, includes **Approve + Reject** at minimum

### `job.progress`

*   require `tracking_card`
*   `tracking_card.card_type == job.tracking`
*   if `tracking_card.state == waiting_input`: require `tracking_card.progress.waiting_on[]` non-empty

### `job.completed`

*   require `finished_card`
*   `finished_card.card_type == job.finished`
*   `finished_card.outcome.result in {completed, failed, cancelled, rejected}`

### `tool.called`

*   require:
    *   `tool_call_id`, `tool_name`, `idempotency_key`, `inputs`, `pii_policy`
*   require `pii_policy.raw_pii_stored == false`

### `tool.result`

*   require:
    *   `tool_call_id`, `tool_name`, `status in {success,error}`
*   if error: require `error.error_code`, `error.message_safe`, `error.retryable`

**Hard rule:** Unknown fields are allowed only if explicitly “extensible”; otherwise reject (prevents silent drift).

* * *

2) Authority Gate (who can emit what)
=====================================

Actor type rules
----------------

*   `job.*` events that change state (approved/rejected/state\_changed/completed/progress) must be from:
    *   agent owner, **or**
    *   human with appropriate role (e.g., job approver), **or**
    *   system (policy agent)
*   `tool.*` events must be from `actor_type == agent` or `system`

Ownership / role checks (minimal v1)
------------------------------------

Define roles in tenant:

*   `job_approver` (humans)
*   `job_owner` (agents/humans)
*   `admin`

Rules:

*   `job.approved` / `job.rejected` allowed only if actor is in conversation AND has `job_approver` (or is conversation creator/admin)
*   `job.cancel` allowed by:
    *   job owner OR any human participant (SMB-friendly)
*   `job.progress` allowed only by:
    *   job owner (agent) OR system
*   `job.completed` allowed only by:
    *   job owner (agent) OR system

**Conversation membership is mandatory** for any event referencing `conversation_id`.

* * *

3) Provenance Gate (button/card authenticity)
=============================================

This is the “no fake buttons” protection.

Card provenance rule
--------------------

If a user submits a card action (approve/provide\_input/dispute/etc), require that:

1.  There exists a prior ledger event in the same tenant+conversation+job where:
    *   `message.sent.kind == card`
    *   `card.card_id == submitted.card_id`
2.  The referenced `button_id` exists in that exact card payload
3.  The action type matches the button’s declared `action.type`

If any fail → reject action as `INVALID_PROVENANCE`.

This prevents:

*   forging approvals
*   replaying buttons from a different job
*   “invented” actions not offered by the UI

* * *

4) State Gate (illegal transitions blocked)
===========================================

Maintain a canonical state machine:

Allowed transitions:

*   `draft → proposed`
*   `proposed → approved | rejected`
*   `approved → in_progress`
*   `in_progress → waiting_input | completed | failed | cancelled`
*   `waiting_input → in_progress | cancelled | failed`
*   `completed/rejected/cancelled/failed` are terminal (no transitions)

Validation rule
---------------

For any event that implies a transition (`job.approved`, `job.rejected`, `job.state_changed`, `job.completed`, `job.cancelled`):

1.  Fetch current job state (projection or fold ledger)
2.  Compute the implied `next_state`
3.  Reject if `(current_state, next_state)` not in allowed set

**Also enforce consistency:**

*   `job.approved` requires current state == `proposed`
*   `job.completed` requires current state in `{in_progress, waiting_input}`

* * *

5) Tool pairing and sequencing rules
====================================

Tool call/result pairing
------------------------

*   Every `tool.result.tool_call_id` must have a prior `tool.called` with same:
    *   tenant\_id, job\_id, tool\_call\_id, tool\_name
*   `tool.result.ts >= tool.called.ts` (monotonic)

Tool usage allowed only during active work
------------------------------------------

Reject `tool.called` if job state is not in:

*   `{in_progress}` (optionally allow during `waiting_input` only for “lookup” tools, but v1 keep strict)

Idempotent side effects
-----------------------

For tools that create external side effects:

*   `tool.called.idempotency_key` must be unique per job+tool
*   If a duplicate tool.called arrives with same idempotency\_key:
    *   treat as replay: accept but do not re-run (or reject as duplicate, your choice—accepting is smoother)

* * *

6) “No PII in ledger” enforcement (v1 simple but strong)
========================================================

For `tool.called.inputs` and `tool.result.output`:

*   Reject if any field matches known PII patterns **unless redacted**:
    *   raw email addresses
    *   phone numbers
*   Allow only:
    *   redacted versions: `m***@acme.com`
    *   hashes: `blake3:...`
    *   vault references: `vault_ref:...`

This one rule saves you from regret later.

* * *

7) Cross-event consistency rules (the quiet killers)
====================================================

Job ↔ Conversation consistency
------------------------------

*   Once `job.created` establishes `job_id → conversation_id`, reject any later event for that job if `conversation_id` differs.

Card ↔ Job consistency
----------------------

*   Any card embedded inside job events must have matching:
    *   `card.job_id == envelope.job_id`
    *   `card.conversation_id == envelope.conversation_id`
    *   `card.tenant_id == envelope.tenant_id`

Finished implies terminal
-------------------------

If `job.completed.outcome.result == completed`, enforce:

*   job state becomes `completed` and terminal

* * *

8) Minimal deterministic “policy code” outline
==============================================

This is how Container Logic should conceptually run:

```text
on_append(event):
  assert_envelope(event)
  assert_schema(event.event_type, event.payload)
  assert_tenant_isolation(event)

  if event.references_job:
    job = load_job_snapshot(event.tenant_id, event.job_id)   // folded/projection
    assert_conversation_match(job, event.conversation_id)
    assert_actor_authorized(event, job)
    assert_state_transition_legal(event, job.state)
    assert_card_provenance_if_action(event)
    assert_tool_rules_if_tool(event, job.state)

  assert_no_raw_pii(event)

  accept(event)
```

* * *

9) What you get from these rules (why it matters)
=================================================

With these guardrails:

*   nobody can “approve” a job without a real offered button
*   no job can jump states
*   tool actions can’t be forged or orphaned
*   cross-tenant bleed becomes structurally impossible
*   audit trails stay coherent and reconstructible forever

* * *

If you keep going: the next “YES” move is a **single compact “Policy Pack v1”** (a JSON/DSL manifest) that lists these rules as named policies (e.g., `policy.job_fsm`, `policy.card_provenance`, `policy.no_raw_pii`) so Office + Gateway + UBL all share the same enforcement vocabulary.

## Prompt:
YES

## Response:
Absolutely — here’s the **Policy Pack v1**: a compact manifest you can ship as the shared “governance vocabulary” across **UBL + Office + Messenger Gateway**. It names the rules, declares what they apply to, what they enforce, and what happens on violation.

It’s intentionally **small, explicit, and computable**. You can implement it as JSON today; later you can compile it into whatever runtime you prefer.

* * *

Policy Pack v1 (Manifest)
=========================

Design goals
------------

*   **Shared language**: Office and Gateway can “speak” policy names.
*   **Deterministic**: no ambiguous phrasing.
*   **Composable**: each policy can be turned on/off per tenant.
*   **Auditable**: violations produce ledger events (`policy.violation`).

* * *

1) Manifest shape
-----------------

```json
{
  "policy_pack_id": "policy_pack_v1",
  "version": "1.0.0",
  "default_mode": "enforce",
  "policies": [
    {
      "policy_id": "policy.envelope_required_fields",
      "mode": "enforce",
      "applies_to": ["*"],
      "rules": [
        { "type": "require_fields", "fields": ["event_id","event_type","ts","tenant_id","trace_id","conversation_id","actor","payload"] },
        { "type": "require_fields", "fields": ["actor.entity_id","actor.actor_type"] }
      ],
      "on_violation": { "action": "reject", "code": "INVALID_ENVELOPE" }
    }
  ]
}
```

* * *

2) The full Policy Pack v1 (recommended minimum)
================================================

```json
{
  "policy_pack_id": "policy_pack_v1",
  "version": "1.0.0",
  "default_mode": "enforce",

  "policies": [

    {
      "policy_id": "policy.envelope_required_fields",
      "mode": "enforce",
      "applies_to": ["*"],
      "rules": [
        { "type": "require_fields", "fields": ["event_id","event_type","ts","tenant_id","trace_id","conversation_id","actor","payload"] },
        { "type": "require_fields", "fields": ["actor.entity_id","actor.actor_type"] },
        { "type": "require_actor_type_in", "allowed": ["human","agent","system"] }
      ],
      "on_violation": { "action": "reject", "code": "INVALID_ENVELOPE" }
    },

    {
      "policy_id": "policy.tenant_isolation",
      "mode": "enforce",
      "applies_to": ["*"],
      "rules": [
        { "type": "enforce_tenant_scope", "scope_fields": ["conversation_id","job_id","actor.entity_id"] }
      ],
      "on_violation": { "action": "reject", "code": "TENANT_SCOPE_VIOLATION" }
    },

    {
      "policy_id": "policy.event_id_uniqueness",
      "mode": "enforce",
      "applies_to": ["*"],
      "rules": [
        { "type": "unique_per_tenant", "field": "event_id" }
      ],
      "on_violation": { "action": "reject", "code": "DUPLICATE_EVENT_ID" }
    },

    {
      "policy_id": "policy.message_schema",
      "mode": "enforce",
      "applies_to": ["message.sent"],
      "rules": [
        { "type": "require_fields", "fields": ["payload.message_id","payload.kind"] },
        { "type": "require_in", "field": "payload.kind", "allowed": ["text","card","system"] },
        { "type": "require_if", "if": {"field":"payload.kind","eq":"text"}, "then_require": ["payload.body_text"] },
        { "type": "require_if", "if": {"field":"payload.kind","eq":"card"}, "then_require": ["payload.card","job_id"] },
        { "type": "require_if", "if": {"field":"payload.kind","eq":"card"}, "then_assert_equal": [
          { "a": "payload.card.job_id", "b": "job_id" },
          { "a": "payload.card.conversation_id", "b": "conversation_id" },
          { "a": "payload.card.tenant_id", "b": "tenant_id" }
        ]}
      ],
      "on_violation": { "action": "reject", "code": "INVALID_MESSAGE_SCHEMA" }
    },

    {
      "policy_id": "policy.job_schema",
      "mode": "enforce",
      "applies_to": ["job.created","job.proposed","job.progress","job.completed","job.state_changed","job.approved","job.rejected"],
      "rules": [
        { "type": "require_fields", "fields": ["job_id"] }
      ],
      "on_violation": { "action": "reject", "code": "INVALID_JOB_SCHEMA" }
    },

    {
      "policy_id": "policy.job_conversation_lock",
      "mode": "enforce",
      "applies_to": ["job.*","tool.*","message.sent"],
      "rules": [
        { "type": "lock_mapping_on_first_seen", "key": "job_id", "lock_field": "conversation_id" }
      ],
      "on_violation": { "action": "reject", "code": "JOB_CONVERSATION_MISMATCH" }
    },

    {
      "policy_id": "policy.job_fsm",
      "mode": "enforce",
      "applies_to": ["job.approved","job.rejected","job.state_changed","job.completed"],
      "rules": [
        {
          "type": "fsm",
          "entity": "job",
          "state_field": "job.state",
          "transitions": [
            ["draft","proposed"],
            ["proposed","approved"],
            ["proposed","rejected"],
            ["approved","in_progress"],
            ["in_progress","waiting_input"],
            ["in_progress","completed"],
            ["in_progress","failed"],
            ["in_progress","cancelled"],
            ["waiting_input","in_progress"],
            ["waiting_input","failed"],
            ["waiting_input","cancelled"]
          ],
          "terminal_states": ["completed","failed","cancelled","rejected"]
        }
      ],
      "on_violation": { "action": "reject", "code": "ILLEGAL_JOB_TRANSITION" }
    },

    {
      "policy_id": "policy.job_authority",
      "mode": "enforce",
      "applies_to": ["job.approved","job.rejected","job.state_changed","job.progress","job.completed","tool.called","tool.result"],
      "rules": [
        { "type": "require_conversation_membership", "actor_field": "actor.entity_id", "conversation_field": "conversation_id" },

        { "type": "allow_if", "event_types": ["job.approved","job.rejected"], "condition": { "actor_has_role": "job_approver" } },

        { "type": "allow_if", "event_types": ["job.progress","job.completed","tool.called","tool.result"], "condition": { "actor_is_job_owner": true } },

        { "type": "allow_if", "event_types": ["job.state_changed"], "condition": { "actor_is_job_owner_or_system": true } }
      ],
      "on_violation": { "action": "reject", "code": "UNAUTHORIZED_ACTION" }
    },

    {
      "policy_id": "policy.card_provenance",
      "mode": "enforce",
      "applies_to": ["job.approved","job.rejected","job.state_changed"],
      "rules": [
        {
          "type": "require_button_provenance",
          "card_id_field": "payload.card_id",
          "button_id_field": "payload.button_id",
          "action_type_field": "payload.action.type",
          "must_match_prior_card_message": true,
          "prior_event_type": "message.sent",
          "prior_card_kind_path": "payload.kind",
          "prior_card_payload_path": "payload.card"
        }
      ],
      "on_violation": { "action": "reject", "code": "INVALID_PROVENANCE" }
    },

    {
      "policy_id": "policy.tool_pairing",
      "mode": "enforce",
      "applies_to": ["tool.result"],
      "rules": [
        { "type": "require_prior_event", "prior_event_type": "tool.called", "match_on": ["tenant_id","job_id","payload.tool_call_id","payload.tool_name"] },
        { "type": "monotonic_time", "prior_event_type": "tool.called", "match_on": ["payload.tool_call_id"], "require_ts_gte": true }
      ],
      "on_violation": { "action": "reject", "code": "TOOL_ORPHAN_RESULT" }
    },

    {
      "policy_id": "policy.tool_only_during_work",
      "mode": "enforce",
      "applies_to": ["tool.called"],
      "rules": [
        { "type": "require_job_state_in", "allowed": ["in_progress"], "job_state_source": "projection_or_fold" }
      ],
      "on_violation": { "action": "reject", "code": "TOOL_NOT_ALLOWED_IN_STATE" }
    },

    {
      "policy_id": "policy.no_raw_pii",
      "mode": "enforce",
      "applies_to": ["tool.called","tool.result","message.sent","job.*"],
      "rules": [
        { "type": "reject_if_regex_match", "paths": ["payload"], "patterns": ["[A-Z0-9._%+-]+@[A-Z0-9.-]+\\.[A-Z]{2,}"] },
        { "type": "reject_if_regex_match", "paths": ["payload"], "patterns": ["\\+?[0-9][0-9\\-\\s()]{7,}"] },
        { "type": "allow_if_present", "paths": ["payload.pii_policy.redactions_applied","payload.pii_policy.hashes_applied"] }
      ],
      "on_violation": { "action": "reject", "code": "RAW_PII_DETECTED" }
    },

    {
      "policy_id": "policy.policy_violation_event",
      "mode": "enforce",
      "applies_to": ["*"],
      "rules": [
        {
          "type": "emit_on_reject",
          "emit_event_type": "policy.violation",
          "payload_template": {
            "violated_policy_id": "{{policy_id}}",
            "code": "{{code}}",
            "event_type": "{{event_type}}",
            "event_id": "{{event_id}}",
            "message_safe": "{{message_safe}}"
          }
        }
      ],
      "on_violation": { "action": "emit_only" }
    }

  ]
}
```

* * *

3) What `policy.violation` looks like (ledger event)
====================================================

This is emitted when a policy rejects something (or warns, if you add “warn mode” later).

```json
{
  "event_id": "evt_pol_0001",
  "event_type": "policy.violation",
  "ts": "2025-12-27T10:16:10.000Z",
  "tenant_id": "tnt_acme_001",
  "trace_id": "trc_20251227_999",
  "conversation_id": "cnv_9f2a",
  "actor": { "entity_id": "system_policy_agent", "actor_type": "system" },
  "payload": {
    "violated_policy_id": "policy.no_raw_pii",
    "code": "RAW_PII_DETECTED",
    "event_type": "tool.called",
    "event_id": "evt_0020",
    "message_safe": "Raw email detected in payload.inputs.attendees[0].email"
  }
}
```

* * *

4) Operational modes (v1 defaults)
==================================

Each policy has a `mode`:

*   `enforce` (reject)
*   `warn` (accept but emit `policy.violation`)
*   `off` (do nothing)

**V1 recommendation**

*   `no_raw_pii` = enforce
*   `job_fsm` = enforce
*   `card_provenance` = enforce
*   Everything else = enforce (you want correctness early)

* * *

5) How this becomes “shared vocabulary” across the three pieces
===============================================================

### Messenger Gateway

*   Uses the pack to validate inbound commands (especially provenance)
*   Returns user-friendly errors for blocked actions

### Office

*   Reads the same policy ids to decide what it can propose/do
*   E.g. if `tool_only_during_work` is enforced, Office won’t attempt tool calls in `waiting_input`

### UBL / Container Logic

*   Enforces the policies at append time (final gate)
*   Emits `policy.violation` on rejects

That’s the triangle: **UI discipline + runtime discipline + ledger discipline**.

* * *

If you want to crystalize this into a “Flagship v1 spec doc”, next “YES” and I’ll output a **single-page spec** (tight, executive + implementer-friendly) that explains the three pieces, the golden flow, the APIs, the event types, and the policy pack — something you can hand to anyone and they’ll build the right thing.

## Prompt:
YES

## Response:
Flagship v1 Spec — LogLine: UBL Messenger + Office + UBL Ledger
---------------------------------------------------------------

### What we’re building

A **three-piece flagship system** for SMBs:

1.  **UBL Messenger** — a beautiful WhatsApp-like app where **humans and agent coworkers** share the same chat UI.
2.  **Office** — a robust LLM runtime that turns chat into reliable work by proposing and running **formal jobs** with governance.
3.  **UBL Ledger + Container Logic** — the mandatory append-only truth layer + computable validation policies.

These are **separate services**, wired via **REST + SSE (or WS)**, with **ledger-backed reconstruction** by `job_id`.

* * *

Core concept: Job Cards
-----------------------

Work is expressed as a formal **Job** (`job_id`) represented in chat as **Cards**:

1.  **Formalize Card** — proposes a job (approve/reject/request changes)
2.  **Tracking Card** — shows progress / waiting input (provide info/dispute/cancel)
3.  **Finished Card** — outcome + artifacts (accept/dispute/follow-up)

**Non-negotiable UX rule:** chat never freezes; cards never block conversation.

* * *

System architecture
-------------------

### A) Messenger (UI + Gateway)

*   UI talks only to Messenger Gateway
*   Gateway:
    *   accepts user commands
    *   forwards work logic to Office
    *   reads/writes UBL (directly or via Office)
    *   maintains projections for fast reads
    *   streams deltas via SSE

### B) Office (runtime)

*   Ingests messages + button actions
*   Builds context + governs actions
*   Decides: normal reply vs job proposal vs job progression
*   Executes tools (calendar/email/etc.)
*   Emits ledger events (including tool audit)

### C) UBL (ledger + container logic)

*   Append-only events, verifiable
*   Query by tenant/conversation/job
*   Stream tail for projections
*   Enforces Policy Pack v1 (see below)

* * *

Golden path (Proof of Done)
---------------------------

**Demo job:** “Schedule a meeting + send invite”

1.  Human messages agent coworker in Messenger
2.  Office emits `job.created` + `job.proposed` + card message (FormalizeCard)
3.  Human clicks **Approve**
4.  Office runs job: emits TrackingCard updates + tool audit (`tool.called`/`tool.result`)
5.  Office emits FinishedCard with artifact link (calendar invite)
6.  `GET /jobs/{job_id}` reconstructs full chain: approvals → tool calls → outcomes
7.  UI receives live updates via SSE without refresh

* * *

APIs
----

### Messenger Gateway (UI-facing)

**Commands**

*   `POST /v1/conversations`
*   `POST /v1/conversations/{id}/messages` (text)
*   `POST /v1/jobs/{job_id}/actions` (button clicks + optional input)

**Reads**

*   `GET /v1/conversations?tenant_id=...`
*   `GET /v1/conversations/{id}/timeline?tenant_id=...&cursor=...`
*   `GET /v1/jobs/{job_id}?tenant_id=...`

**Stream**

*   `GET /v1/stream?tenant_id=...&cursor=...` (SSE)

### Office (Gateway-facing)

*   `POST /v1/office/ingest_message`
*   `POST /v1/office/job_action`
*   `GET /v1/office/affordances?tenant_id=...&entity_id=...`

### UBL (Ledger)

*   `POST /v1/ledger/append` (batch append)
*   `GET /v1/ledger/query?...`
*   `GET /v1/ledger/stream?...` (tail)

* * *

SSE protocol (Gateway → UI)
---------------------------

SSE frames use:

*   `id: <cursor>`
*   `event: <name>`
*   `data: <json>`

Events:

*   `hello`
*   `timeline.append`
*   `conversation.update`
*   `job.update`
*   `presence.update`
*   `artifact.added` (optional)
*   `heartbeat`
*   `error` (e.g., cursor too old → resync)

Resume: UI reconnects with `cursor=<last_cursor>`.

* * *

Event taxonomy (UBL)
--------------------

All events share envelope:

*   `event_id`, `event_type`, `ts`
*   `tenant_id`, `trace_id`, `conversation_id`
*   `actor { entity_id, actor_type }`
*   optional `job_id`
*   `payload`

**Minimum types**

*   `message.sent` (text, system, card)
*   `job.created`
*   `job.proposed` (embeds FormalizeCard)
*   `job.approved` / `job.rejected`
*   `job.state_changed`
*   `job.progress` (embeds TrackingCard)
*   `job.completed` (embeds FinishedCard)
*   `tool.called` (sanitized inputs + idempotency key)
*   `tool.result` (success/error + artifacts)
*   `policy.violation` (on rejected events)

* * *

Card contracts (v1)
-------------------

Cards are embedded in `message.sent.kind="card"`.

**FormalizeCard**

*   job goal, inputs needed, expected outputs, constraints, ETA
*   buttons: Approve / Reject / Request changes / Ask in chat

**TrackingCard**

*   status line, steps, blockers, waiting\_on
*   buttons: Got it / Provide info / Dispute / Cancel / Ask in chat

**FinishedCard**

*   outcome summary, artifacts, next actions
*   buttons: Accept / Dispute / Follow-up / Ask in chat

All cards must include:

*   `job_id`, `card_id`, `tenant_id`, `conversation_id`, `state`, `owner`, `author`

* * *

Presence (computable from ledger)
---------------------------------

Presence states:

*   `offline` (no activity > TTL)
*   `waiting_on_you` (you’re in `waiting_on[]` for a `waiting_input` job)
*   `working` (you own an `in_progress` job + recent activity)
*   `available` (default)

Presence updates are streamed via `presence.update`.

* * *

Projections (Gateway read model)
--------------------------------

Maintain rebuildable views:

*   `projection_conversations`
*   `projection_timeline_items`
*   `projection_jobs`
*   `projection_job_events` (drawer timeline)
*   `projection_job_artifacts`
*   `projection_presence`
*   `projection_entities`

All rebuildable by replaying ledger tail.

* * *

Policy Pack v1 (Container Logic)
--------------------------------

Minimum enforced policies:

*   Required envelope fields
*   Tenant isolation
*   Event id uniqueness
*   Strict schemas per event type
*   Job ↔ conversation lock
*   Job finite-state machine (illegal transitions rejected)
*   Role/ownership authorization
