# C.Office — Event Types

## Entity Lifecycle

### entity.created
```json
{
  "type": "entity.created",
  "entity_id": "entity_abc123",
  "name": "Claude Assistant",
  "entity_type": "autonomous",
  "public_key": "ed25519:abc...",
  "constitution": {
    "version": 1,
    "directives": [],
    "denylists": []
  },
  "ts_ms": 1703123456789
}
```

### entity.activated
```json
{
  "type": "entity.activated",
  "entity_id": "entity_abc123",
  "ts_ms": 1703123456789
}
```

### entity.suspended
```json
{
  "type": "entity.suspended",
  "entity_id": "entity_abc123",
  "reason": "Excessive policy violations",
  "ts_ms": 1703123456789
}
```

### entity.archived
```json
{
  "type": "entity.archived",
  "entity_id": "entity_abc123",
  "ts_ms": 1703123456789
}
```

## Constitution Events

### constitution.updated
```json
{
  "type": "constitution.updated",
  "entity_id": "entity_abc123",
  "constitution": {
    "version": 2,
    "directives": [
      {
        "id": "no-financial-advice",
        "rule": "Never provide financial investment advice",
        "severity": "block"
      }
    ],
    "denylists": ["competitor_names"]
  },
  "updated_by": "guardian_xyz",
  "ts_ms": 1703123456789
}
```

### baseline.updated
```json
{
  "type": "baseline.updated",
  "entity_id": "entity_abc123",
  "baseline": "Core narrative synthesized from 50 sessions...",
  "sessions_synthesized": 50,
  "dreaming_cycle_id": "dream_123",
  "ts_ms": 1703123456789
}
```

## Session Events

### session.started
```json
{
  "type": "session.started",
  "entity_id": "entity_abc123",
  "session_id": "sess_xyz789",
  "session_type": "chat",
  "mode": "assisted",
  "token_budget": 100000,
  "handover_received": "Previous session context...",
  "ts_ms": 1703123456789
}
```

### session.completed
```json
{
  "type": "session.completed",
  "entity_id": "entity_abc123",
  "session_id": "sess_xyz789",
  "tokens_used": 45000,
  "duration_ms": 180000,
  "handover": {
    "summary": "Session summary for next instance...",
    "key_facts": ["fact1", "fact2"],
    "pending_tasks": []
  },
  "ts_ms": 1703123456789
}
```

## Audit Events

### audit.tool_called
```json
{
  "type": "audit.tool_called",
  "entity_id": "entity_abc123",
  "session_id": "sess_xyz789",
  "job_id": "job_456",
  "trace_id": "trace_abc",
  "tool": {
    "name": "web_search",
    "input": {"query": "weather forecast"},
    "risk_level": "L1"
  },
  "ts_ms": 1703123456789
}
```

### audit.tool_result
```json
{
  "type": "audit.tool_result",
  "entity_id": "entity_abc123",
  "session_id": "sess_xyz789",
  "job_id": "job_456",
  "trace_id": "trace_abc",
  "tool": {
    "name": "web_search",
    "success": true,
    "duration_ms": 1500
  },
  "ts_ms": 1703123456789
}
```

### audit.decision_made
```json
{
  "type": "audit.decision_made",
  "entity_id": "entity_abc123",
  "session_id": "sess_xyz789",
  "decision": {
    "type": "propose_job",
    "rationale": "User requested file organization",
    "confidence": 0.95,
    "alternatives_considered": ["chat_reply", "escalate"]
  },
  "ts_ms": 1703123456789
}
```

### audit.policy_violation
```json
{
  "type": "audit.policy_violation",
  "entity_id": "entity_abc123",
  "session_id": "sess_xyz789",
  "violation": {
    "policy_id": "no-financial-advice",
    "code": "CONSTITUTION_001",
    "triggering_event": "tool.called",
    "message_safe": "Blocked attempt to access financial data"
  },
  "ts_ms": 1703123456789
}
```

## Governance Events

### governance.sanity_check
```json
{
  "type": "governance.sanity_check",
  "entity_id": "entity_abc123",
  "session_id": "sess_xyz789",
  "check": {
    "claim": "User has 5 pending tasks",
    "fact": "User has 3 pending tasks",
    "discrepancy": true,
    "severity": "medium"
  },
  "action": "governance_note_created",
  "ts_ms": 1703123456789
}
```

### governance.dreaming_cycle
```json
{
  "type": "governance.dreaming_cycle",
  "entity_id": "entity_abc123",
  "cycle_id": "dream_123",
  "input": {
    "sessions_count": 50,
    "total_tokens": 2500000,
    "time_range_ms": [1703000000000, 1703123456789]
  },
  "output": {
    "baseline_updated": true,
    "memories_consolidated": 150,
    "anxiety_resolved": ["unfinished_task_123"]
  },
  "ts_ms": 1703123456789
}
```

### governance.simulation
```json
{
  "type": "governance.simulation",
  "entity_id": "entity_abc123",
  "session_id": "sess_xyz789",
  "simulation": {
    "action_proposed": "delete_file",
    "outcomes": [
      {"scenario": "success", "probability": 0.7, "impact": "low"},
      {"scenario": "wrong_file", "probability": 0.2, "impact": "high"},
      {"scenario": "permission_denied", "probability": 0.1, "impact": "none"}
    ],
    "recommendation": "proceed_with_confirmation",
    "risk_score": 0.45
  },
  "ts_ms": 1703123456789
}
```

