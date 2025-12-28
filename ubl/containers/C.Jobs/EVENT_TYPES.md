# C.Jobs Event Types Specification

**Status:** NORMATIVE  
**Version:** 1.0  
**Date:** 2024-12-27  
**Governed by:** SPEC-UBL-ATOM v1.0, SPEC-UBL-CORE v1.0  
**Container:** C.Jobs

---

## Overview

This document defines the canonical event atom schemas for the C.Jobs container. All events MUST conform to SPEC-UBL-ATOM v1.0 canonicalization rules.

**Critical Rules:**
- All keys MUST be sorted lexicographically (UTF-8 byte order)
- No container_id, signature, sequence, or policy fields
- All values MUST be canonical JSON types
- Timestamps MUST be ISO 8601 strings (UTC)
- Numbers MUST be finite (no NaN, Infinity)

---

## Event Type: `job.created`

**Intent Class:** `Observation`  
**Physics Delta:** `0`  
**Purpose:** Record the creation of a new job

### Canonical Atom Schema

```json
{
  "assigned_to": "string",
  "conversation_id": "string",
  "created_at": "string",
  "created_by": "string",
  "description": "string",
  "estimated_duration_seconds": 0,
  "estimated_value": 0.0,
  "id": "string",
  "priority": "string",
  "title": "string",
  "type": "job.created"
}
```

### Field Definitions

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `type` | string | ✅ | Always `"job.created"` |
| `id` | string | ✅ | Unique job identifier (e.g., `"job_2024_001"`) |
| `conversation_id` | string | ✅ | ID of conversation where job was created |
| `title` | string | ✅ | Job title |
| `description` | string | ❌ | Job description (optional) |
| `created_by` | string | ✅ | User/agent ID who created the job |
| `assigned_to` | string | ✅ | User/agent ID assigned to execute the job |
| `priority` | string | ❌ | Priority level: `"low"`, `"normal"`, `"high"`, `"urgent"` |
| `estimated_duration_seconds` | integer | ❌ | Estimated duration in seconds |
| `estimated_value` | number | ❌ | Estimated monetary value (if applicable) |
| `created_at` | string | ✅ | ISO 8601 timestamp (UTC) |

### Example Canonical Atom

```json
{
  "assigned_to": "agent_robofab",
  "conversation_id": "conv_123",
  "created_at": "2024-12-27T10:00:00Z",
  "created_by": "user_maria",
  "description": "Create proposal for client ABC",
  "estimated_duration_seconds": 300,
  "estimated_value": 0.0,
  "id": "job_2024_001",
  "priority": "normal",
  "title": "Create Proposal - Client ABC",
  "type": "job.created"
}
```

---

## Event Type: `job.started`

**Intent Class:** `Observation`  
**Physics Delta:** `0`  
**Purpose:** Record that job execution has started

### Canonical Atom Schema

```json
{
  "id": "string",
  "started_at": "string",
  "type": "job.started"
}
```

### Field Definitions

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `type` | string | ✅ | Always `"job.started"` |
| `id` | string | ✅ | Job identifier |
| `started_at` | string | ✅ | ISO 8601 timestamp (UTC) |

### Example Canonical Atom

```json
{
  "id": "job_2024_001",
  "started_at": "2024-12-27T10:02:15Z",
  "type": "job.started"
}
```

---

## Event Type: `job.progress`

**Intent Class:** `Observation`  
**Physics Delta:** `0`  
**Purpose:** Record job execution progress

### Canonical Atom Schema

```json
{
  "current_step": "string",
  "id": "string",
  "percent_complete": 0,
  "step_description": "string",
  "total_steps": 0,
  "type": "job.progress"
}
```

### Field Definitions

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `type` | string | ✅ | Always `"job.progress"` |
| `id` | string | ✅ | Job identifier |
| `current_step` | string | ✅ | Current step identifier |
| `step_description` | string | ❌ | Human-readable step description |
| `total_steps` | integer | ❌ | Total number of steps |
| `percent_complete` | integer | ❌ | Completion percentage (0-100) |

### Example Canonical Atom

```json
{
  "current_step": "step_2",
  "id": "job_2024_001",
  "percent_complete": 45,
  "step_description": "Calculating prices",
  "total_steps": 5,
  "type": "job.progress"
}
```

---

## Event Type: `job.completed`

**Intent Class:** `Observation` or `Entropy`  
**Physics Delta:** `0` or `+value`  
**Purpose:** Record successful job completion

### Canonical Atom Schema

```json
{
  "artifacts": ["string"],
  "completed_at": "string",
  "duration_seconds": 0,
  "id": "string",
  "result_summary": "string",
  "type": "job.completed",
  "value_created": 0.0
}
```

### Field Definitions

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `type` | string | ✅ | Always `"job.completed"` |
| `id` | string | ✅ | Job identifier |
| `completed_at` | string | ✅ | ISO 8601 timestamp (UTC) |
| `duration_seconds` | integer | ✅ | Total execution time in seconds |
| `result_summary` | string | ❌ | Human-readable result summary |
| `artifacts` | array[string] | ❌ | List of artifact IDs/files created |
| `value_created` | number | ❌ | Monetary value created (if applicable) |

**Note:** If `value_created > 0`, use `IntentClass::Entropy` with `physics_delta = value_created`. Otherwise use `IntentClass::Observation` with `physics_delta = 0`.

### Example Canonical Atom

```json
{
  "artifacts": ["artifact_proposal_abc_2024.pdf"],
  "completed_at": "2024-12-27T10:05:30Z",
  "duration_seconds": 195,
  "id": "job_2024_001",
  "result_summary": "Proposal created and sent to client",
  "type": "job.completed",
  "value_created": 0.0
}
```

---

## Event Type: `job.cancelled`

**Intent Class:** `Observation`  
**Physics Delta:** `0`  
**Purpose:** Record job cancellation

### Canonical Atom Schema

```json
{
  "cancelled_at": "string",
  "cancelled_by": "string",
  "id": "string",
  "reason": "string",
  "type": "job.cancelled"
}
```

### Field Definitions

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `type` | string | ✅ | Always `"job.cancelled"` |
| `id` | string | ✅ | Job identifier |
| `cancelled_at` | string | ✅ | ISO 8601 timestamp (UTC) |
| `cancelled_by` | string | ✅ | User/agent ID who cancelled |
| `reason` | string | ❌ | Cancellation reason |

### Example Canonical Atom

```json
{
  "cancelled_at": "2024-12-27T10:10:00Z",
  "cancelled_by": "user_maria",
  "id": "job_2024_001",
  "reason": "Client requested changes",
  "type": "job.cancelled"
}
```

---

## Event Type: `approval.requested`

**Intent Class:** `Observation`  
**Physics Delta:** `0`  
**Purpose:** Record that approval is required for a job action

### Canonical Atom Schema

```json
{
  "approval_id": "string",
  "details": ["string"],
  "impact": "string",
  "job_id": "string",
  "requested_at": "string",
  "requested_by": "string",
  "title": "string",
  "type": "approval.requested"
}
```

### Field Definitions

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `type` | string | ✅ | Always `"approval.requested"` |
| `approval_id` | string | ✅ | Unique approval identifier |
| `job_id` | string | ✅ | Job identifier |
| `title` | string | ✅ | Approval title |
| `details` | array[string] | ✅ | List of detail strings |
| `impact` | string | ✅ | Impact description |
| `requested_by` | string | ✅ | User/agent ID requesting approval |
| `requested_at` | string | ✅ | ISO 8601 timestamp (UTC) |

### Example Canonical Atom

```json
{
  "approval_id": "approval_2024_001",
  "details": [
    "Proposal value: R$ 12.450,00",
    "Margin: 23%",
    "Client: ABC Ltda"
  ],
  "impact": "Financial impact above threshold",
  "job_id": "job_2024_001",
  "requested_at": "2024-12-27T10:04:00Z",
  "requested_by": "agent_robofab",
  "title": "Approve Proposal - Client ABC",
  "type": "approval.requested"
}
```

---

## Event Type: `approval.decided`

**Intent Class:** `Observation`  
**Physics Delta:** `0`  
**Purpose:** Record approval decision

### Canonical Atom Schema

```json
{
  "approval_id": "string",
  "decided_at": "string",
  "decided_by": "string",
  "decision": "string",
  "job_id": "string",
  "reason": "string",
  "type": "approval.decided"
}
```

### Field Definitions

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `type` | string | ✅ | Always `"approval.decided"` |
| `approval_id` | string | ✅ | Approval identifier |
| `job_id` | string | ✅ | Job identifier |
| `decision` | string | ✅ | Decision: `"approved"`, `"rejected"`, or `"request_changes"` |
| `decided_by` | string | ✅ | User/agent ID who made the decision |
| `decided_at` | string | ✅ | ISO 8601 timestamp (UTC) |
| `reason` | string | ❌ | Decision reason (optional) |

### Example Canonical Atom

```json
{
  "approval_id": "approval_2024_001",
  "decided_at": "2024-12-27T10:05:00Z",
  "decided_by": "user_maria",
  "decision": "approved",
  "job_id": "job_2024_001",
  "reason": "Proposal looks good",
  "type": "approval.decided"
}
```

---

## Canonicalization Rules

All event atoms MUST follow SPEC-UBL-ATOM v1.0:

1. **Key Ordering:** All object keys sorted lexicographically (UTF-8 byte order)
2. **No Whitespace:** JSON compact (no spaces, newlines, trailing commas)
3. **Finite Numbers:** No NaN, Infinity, or -Infinity
4. **UTF-8 Normalized:** Strings in NFC normalization
5. **No Metadata:** No container_id, signature, sequence, or policy fields

## Intent Class Mapping

| Event Type | Intent Class | Physics Delta |
|------------|-------------|---------------|
| `job.created` | Observation | 0 |
| `job.started` | Observation | 0 |
| `job.progress` | Observation | 0 |
| `job.completed` | Observation or Entropy | 0 or +value_created |
| `job.cancelled` | Observation | 0 |
| `approval.requested` | Observation | 0 |
| `approval.decided` | Observation | 0 |

## Validation

Before committing any event atom:

1. ✅ Validate JSON structure matches schema
2. ✅ Validate all required fields present
3. ✅ Validate field types match schema
4. ✅ Validate timestamp format (ISO 8601 UTC)
5. ✅ Validate no prohibited fields (container_id, signature, etc.)
6. ✅ Canonicalize JSON (sort keys, compact)
7. ✅ Generate atom_hash using BLAKE3

## Versioning

This specification is version 1.0 and follows UBL v1.0 FROZEN specifications. Any changes require:
- Version bump
- Migration guide
- Backward compatibility considerations

---

**This specification ensures all C.Jobs events are permanent, canonical, and verifiable according to UBL standards.**

