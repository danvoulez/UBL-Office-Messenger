# C.Messenger Event Types Specification

**Status:** NORMATIVE  
**Version:** 1.0  
**Date:** 2024-12-28  
**Governed by:** SPEC-UBL-ATOM v1.0, SPEC-UBL-CORE v1.0  
**Container:** C.Messenger

---

## Overview

This document defines the canonical event atom schemas for the C.Messenger container. All events MUST conform to SPEC-UBL-ATOM v1.0 canonicalization rules.

**Critical Rules:**
- All keys MUST be sorted lexicographically (UTF-8 byte order)
- No container_id, signature, sequence, or policy fields
- All values MUST be canonical JSON types
- Timestamps MUST be ISO 8601 strings (UTC)
- Numbers MUST be finite (no NaN, Infinity)

---

## Event Type: `conversation.created`

**Intent Class:** `Observation`  
**Physics Delta:** `0`  
**Purpose:** Record the creation of a new conversation/workstream

### Canonical Atom Schema

```json
{
  "created_at": "string",
  "created_by": "string",
  "id": "string",
  "is_group": false,
  "name": "string",
  "participants": ["string"],
  "type": "conversation.created"
}
```

### Field Definitions

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `type` | string | ✅ | Always `"conversation.created"` |
| `id` | string | ✅ | Unique conversation identifier (e.g., `"conv_2024_001"`) |
| `name` | string | ❌ | Conversation/workstream name (for groups) |
| `participants` | array[string] | ✅ | List of participant SIDs |
| `is_group` | boolean | ✅ | Whether this is a group conversation |
| `created_by` | string | ✅ | User SID who created the conversation |
| `created_at` | string | ✅ | ISO 8601 timestamp (UTC) |

### Example Canonical Atom

```json
{
  "created_at": "2024-12-28T10:00:00Z",
  "created_by": "user_joao",
  "id": "conv_2024_001",
  "is_group": true,
  "name": "Strategic Board 🏛️",
  "participants": ["user_joao", "agent_sofia", "user_ana"],
  "type": "conversation.created"
}
```

---

## Event Type: `conversation.participant_added`

**Intent Class:** `Observation`  
**Physics Delta:** `0`  
**Purpose:** Record a new participant joining a conversation

### Canonical Atom Schema

```json
{
  "added_at": "string",
  "added_by": "string",
  "conversation_id": "string",
  "participant_id": "string",
  "type": "conversation.participant_added"
}
```

### Field Definitions

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `type` | string | ✅ | Always `"conversation.participant_added"` |
| `conversation_id` | string | ✅ | Conversation identifier |
| `participant_id` | string | ✅ | SID of added participant |
| `added_by` | string | ✅ | SID who added the participant |
| `added_at` | string | ✅ | ISO 8601 timestamp (UTC) |

---

## Event Type: `message.created`

**Intent Class:** `Observation`  
**Physics Delta:** `0`  
**Purpose:** Record a new message in a conversation

### Canonical Atom Schema

```json
{
  "content_hash": "string",
  "conversation_id": "string",
  "created_at": "string",
  "from": "string",
  "id": "string",
  "message_type": "string",
  "type": "message.created"
}
```

### Field Definitions

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `type` | string | ✅ | Always `"message.created"` |
| `id` | string | ✅ | Unique message identifier (e.g., `"msg_2024_001"`) |
| `conversation_id` | string | ✅ | ID of conversation |
| `from` | string | ✅ | SID of sender |
| `content_hash` | string | ✅ | BLAKE3 hash of message content (privacy) |
| `message_type` | string | ✅ | Type: `"text"`, `"system"`, `"job_card"`, `"file"` |
| `created_at` | string | ✅ | ISO 8601 timestamp (UTC) |

### Example Canonical Atom

```json
{
  "content_hash": "a1b2c3d4e5f6...",
  "conversation_id": "conv_2024_001",
  "created_at": "2024-12-28T10:05:30Z",
  "from": "user_joao",
  "id": "msg_2024_001",
  "message_type": "text",
  "type": "message.created"
}
```

**Note:** The actual message content is stored separately (e.g., in a content-addressed store or the atom_data field). The ledger only stores the hash for privacy.

---

## Event Type: `message.read`

**Intent Class:** `Observation`  
**Physics Delta:** `0`  
**Purpose:** Record that a message was read by a user

### Canonical Atom Schema

```json
{
  "message_id": "string",
  "read_at": "string",
  "read_by": "string",
  "type": "message.read"
}
```

### Field Definitions

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `type` | string | ✅ | Always `"message.read"` |
| `message_id` | string | ✅ | Message identifier |
| `read_by` | string | ✅ | SID of reader |
| `read_at` | string | ✅ | ISO 8601 timestamp (UTC) |

---

## Event Type: `entity.registered`

**Intent Class:** `Observation`  
**Physics Delta:** `0`  
**Purpose:** Record a new entity (person, agent) registration visible to Messenger

### Canonical Atom Schema

```json
{
  "avatar_hash": "string",
  "display_name": "string",
  "entity_type": "string",
  "id": "string",
  "registered_at": "string",
  "type": "entity.registered"
}
```

### Field Definitions

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `type` | string | ✅ | Always `"entity.registered"` |
| `id` | string | ✅ | Entity SID |
| `display_name` | string | ✅ | Display name |
| `entity_type` | string | ✅ | Type: `"person"`, `"agent"`, `"system"` |
| `avatar_hash` | string | ❌ | BLAKE3 hash of avatar (optional) |
| `registered_at` | string | ✅ | ISO 8601 timestamp (UTC) |

---

## Intent Class Mapping

| Event Type | Intent Class | Physics Delta |
|------------|-------------|---------------|
| `conversation.created` | Observation | 0 |
| `conversation.participant_added` | Observation | 0 |
| `message.created` | Observation | 0 |
| `message.read` | Observation | 0 |
| `entity.registered` | Observation | 0 |

---

## Validation

Before committing any event atom:

1. ✅ Validate JSON structure matches schema
2. ✅ Validate all required fields present
3. ✅ Validate field types match schema
4. ✅ Validate timestamp format (ISO 8601 UTC)
5. ✅ Validate no prohibited fields (container_id, signature, etc.)
6. ✅ Canonicalize JSON (sort keys, compact)
7. ✅ Generate atom_hash using BLAKE3

---

## Relationship with Other Containers

- **C.Jobs**: When a job is discussed in a conversation, a `message.created` event with `message_type: "job_card"` is committed to C.Messenger. The job lifecycle is managed in C.Jobs.
- **C.Artifacts**: File attachments are stored via C.Artifacts. C.Messenger stores only the artifact reference hash.

---

**This specification ensures all C.Messenger events are permanent, canonical, and verifiable according to UBL standards.**

