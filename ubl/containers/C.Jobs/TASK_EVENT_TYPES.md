````markdown
# C.Jobs Task Event Types Specification

**Status:** NORMATIVE  
**Version:** 1.0  
**Date:** 2024-12-28  
**Governed by:** SPEC-UBL-ATOM v1.0, SPEC-UBL-CORE v1.0  
**Container:** C.Jobs

---

## Overview

This document defines the canonical event atom schemas for **task formalization** events in the C.Jobs container. Tasks are formalized work items created from conversations that require explicit approval and acceptance workflows.

**Key Differences from Jobs:**
- Tasks require explicit **human acceptance** after completion
- Tasks have bidirectional approval (human↔agent can both create drafts)
- Accepted tasks become **official documents** versioned in git

**Critical Rules:**
- All keys MUST be sorted lexicographically (UTF-8 byte order)
- No container_id, signature, sequence, or policy fields
- All values MUST be canonical JSON types
- Timestamps MUST be ISO 8601 strings (UTC)
- Numbers MUST be finite (no NaN, Infinity)

---

## Event Type: `task.created`

**Intent Class:** `Observation`  
**Physics Delta:** `0`  
**Purpose:** Record the creation of a new task draft

### Canonical Atom Schema

```json
{
  "assigned_to": "string",
  "attachment_count": 0,
  "conversation_id": "string",
  "created_by": "string",
  "deadline": "string",
  "description": "string",
  "estimated_cost": "string",
  "priority": "string",
  "task_id": "string",
  "timestamp": "string",
  "title": "string",
  "type": "task.created"
}
```

### Field Definitions

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `type` | string | ✅ | Always `"task.created"` |
| `task_id` | string | ✅ | Unique task identifier (e.g., `"task_2024_001"`) |
| `conversation_id` | string | ✅ | ID of conversation where task was created |
| `title` | string | ✅ | Task title |
| `description` | string | ❌ | Task description (optional) |
| `priority` | string | ❌ | Priority level: `"low"`, `"normal"`, `"high"`, `"critical"` |
| `deadline` | string | ❌ | ISO 8601 timestamp (UTC) for deadline |
| `estimated_cost` | string | ❌ | Estimated cost as string (e.g., "R$ 500,00") |
| `created_by` | string | ✅ | User/agent SID who created the task |
| `assigned_to` | string | ✅ | User/agent SID assigned to approve/execute |
| `attachment_count` | integer | ✅ | Number of attachments |
| `timestamp` | string | ✅ | ISO 8601 timestamp (UTC) |

### Example Canonical Atom

```json
{
  "assigned_to": "agent_robofab",
  "attachment_count": 2,
  "conversation_id": "conv_123",
  "created_by": "user_maria",
  "deadline": "2024-12-31T23:59:59Z",
  "description": "Create quarterly financial report for board review",
  "estimated_cost": "R$ 500,00",
  "priority": "high",
  "task_id": "task_2024_001",
  "timestamp": "2024-12-28T10:00:00Z",
  "title": "Quarterly Financial Report Q4 2024",
  "type": "task.created"
}
```

---

## Event Type: `task.approved`

**Intent Class:** `Observation`  
**Physics Delta:** `0`  
**Purpose:** Record that a task draft was approved by the other party

### Canonical Atom Schema

```json
{
  "approved_by": "string",
  "task_id": "string",
  "timestamp": "string",
  "type": "task.approved"
}
```

### Field Definitions

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `type` | string | ✅ | Always `"task.approved"` |
| `task_id` | string | ✅ | Task identifier |
| `approved_by` | string | ✅ | User/agent SID who approved |
| `timestamp` | string | ✅ | ISO 8601 timestamp (UTC) |

---

## Event Type: `task.rejected`

**Intent Class:** `Observation`  
**Physics Delta:** `0`  
**Purpose:** Record that a task draft was rejected

### Canonical Atom Schema

```json
{
  "reason": "string",
  "rejected_by": "string",
  "task_id": "string",
  "timestamp": "string",
  "type": "task.rejected"
}
```

### Field Definitions

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `type` | string | ✅ | Always `"task.rejected"` |
| `task_id` | string | ✅ | Task identifier |
| `rejected_by` | string | ✅ | User/agent SID who rejected |
| `reason` | string | ✅ | Rejection reason |
| `timestamp` | string | ✅ | ISO 8601 timestamp (UTC) |

---

## Event Type: `task.started`

**Intent Class:** `Observation`  
**Physics Delta:** `0`  
**Purpose:** Record that task execution has started

### Canonical Atom Schema

```json
{
  "task_id": "string",
  "timestamp": "string",
  "type": "task.started"
}
```

---

## Event Type: `task.progress`

**Intent Class:** `Observation`  
**Physics Delta:** `0`  
**Purpose:** Record task execution progress

### Canonical Atom Schema

```json
{
  "current_step": "string",
  "progress": 0,
  "task_id": "string",
  "timestamp": "string",
  "type": "task.progress"
}
```

### Field Definitions

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `type` | string | ✅ | Always `"task.progress"` |
| `task_id` | string | ✅ | Task identifier |
| `progress` | integer | ✅ | Completion percentage (0-100) |
| `current_step` | string | ✅ | Current step description |
| `timestamp` | string | ✅ | ISO 8601 timestamp (UTC) |

---

## Event Type: `task.completed`

**Intent Class:** `Observation` or `Entropy`  
**Physics Delta:** `0` or `+value`  
**Purpose:** Record successful task completion (awaiting acceptance)

### Canonical Atom Schema

```json
{
  "artifact_count": 0,
  "duration_seconds": 0,
  "git_commit_hash": "string",
  "success": true,
  "summary": "string",
  "task_id": "string",
  "timestamp": "string",
  "type": "task.completed"
}
```

### Field Definitions

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `type` | string | ✅ | Always `"task.completed"` |
| `task_id` | string | ✅ | Task identifier |
| `success` | boolean | ✅ | Whether task completed successfully |
| `summary` | string | ✅ | Human-readable result summary |
| `artifact_count` | integer | ✅ | Number of artifacts produced |
| `duration_seconds` | integer | ✅ | Total execution time in seconds |
| `git_commit_hash` | string | ❌ | Git commit hash if versioned |
| `timestamp` | string | ✅ | ISO 8601 timestamp (UTC) |

---

## Event Type: `task.accepted`

**Intent Class:** `Observation` or `Entropy`  
**Physics Delta:** `0` or `+value`  
**Purpose:** Record final human acceptance of completed task

### Canonical Atom Schema

```json
{
  "accepted_by": "string",
  "feedback": "string",
  "task_id": "string",
  "timestamp": "string",
  "type": "task.accepted"
}
```

### Field Definitions

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `type` | string | ✅ | Always `"task.accepted"` |
| `task_id` | string | ✅ | Task identifier |
| `accepted_by` | string | ✅ | Human SID who accepted |
| `feedback` | string | ❌ | Optional acceptance feedback |
| `timestamp` | string | ✅ | ISO 8601 timestamp (UTC) |

**Note:** This is a critical event. After acceptance, the task becomes an official document and is versioned in git.

---

## Event Type: `task.disputed`

**Intent Class:** `Observation`  
**Physics Delta:** `0`  
**Purpose:** Record human dispute of completed task

### Canonical Atom Schema

```json
{
  "disputed_by": "string",
  "reason": "string",
  "task_id": "string",
  "timestamp": "string",
  "type": "task.disputed"
}
```

### Field Definitions

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `type` | string | ✅ | Always `"task.disputed"` |
| `task_id` | string | ✅ | Task identifier |
| `disputed_by` | string | ✅ | Human SID who disputed |
| `reason` | string | ✅ | Dispute reason |
| `timestamp` | string | ✅ | ISO 8601 timestamp (UTC) |

---

## Event Type: `task.cancelled`

**Intent Class:** `Observation`  
**Physics Delta:** `0`  
**Purpose:** Record task cancellation

### Canonical Atom Schema

```json
{
  "cancelled_by": "string",
  "reason": "string",
  "task_id": "string",
  "timestamp": "string",
  "type": "task.cancelled"
}
```

---

## Intent Class Mapping

| Event Type | Intent Class | Physics Delta |
|------------|-------------|---------------|
| `task.created` | Observation | 0 |
| `task.approved` | Observation | 0 |
| `task.rejected` | Observation | 0 |
| `task.started` | Observation | 0 |
| `task.progress` | Observation | 0 |
| `task.completed` | Observation or Entropy | 0 or +value_created |
| `task.accepted` | Observation or Entropy | 0 or +value_created |
| `task.disputed` | Observation | 0 |
| `task.cancelled` | Observation | 0 |

---

## Task Lifecycle State Machine

```
                    ┌──────────┐
                    │  Draft   │
                    └────┬─────┘
                         │
            ┌────────────┼────────────┐
            │ approve    │ reject     │ cancel
            ▼            ▼            ▼
       ┌─────────┐  ┌─────────┐  ┌───────────┐
       │ Approved│  │ Rejected│  │ Cancelled │
       └────┬────┘  └─────────┘  └───────────┘
            │ start
            ▼
       ┌─────────┐
       │ Running │◄─────┐
       └────┬────┘      │ resume
            │           │
    ┌───────┼───────┐   │
    │pause  │complete│  │
    ▼       ▼       ▼   │
┌────────┐ ┌─────────┐  │
│ Paused │─┘│Completed│  │
└────────┘  └────┬────┘  
                 │
        ┌────────┴────────┐
        │ accept          │ dispute
        ▼                 ▼
   ┌──────────┐     ┌──────────┐
   │ Accepted │     │ Disputed │
   └──────────┘     └──────────┘
```

---

## Relationship with Jobs

Tasks extend the job system with:
1. **Explicit draft approval** - Both parties must agree before execution
2. **Human acceptance** - Completion requires explicit human sign-off
3. **Document versioning** - Accepted tasks are committed to git

Tasks use the same C.Jobs container but have a more formal lifecycle.

---

**This specification ensures all task events are permanent, canonical, and verifiable according to UBL standards.**
````
