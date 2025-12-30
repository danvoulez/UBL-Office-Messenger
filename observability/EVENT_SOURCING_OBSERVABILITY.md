# 📊 Event-Sourced System Observability

## Critical Differences for Event Sourcing

Event-sourced systems have unique observability requirements because: 
- **State is derived** (not directly queryable)
- **Events are immutable** (can't be "fixed")
- **Causality matters** (event order is critical)
- **Projections lag** (eventual consistency)
- **Replay is possible** (but must be tracked)

---

## Must-Have Metrics for Event Sourcing

### 1. Event Stream Health

#### Ledger Append Metrics
```rust
// Event write throughput
ledger_append_total{container_id, status}
ledger_append_rate{container_id}  // events/sec

// Event write latency (CRITICAL for backpressure)
ledger_append_duration_seconds{container_id, p50, p95, p99}

// Append failures (ZERO tolerance)
ledger_append_failures_total{container_id, error_type, causality_violation}

// Sequence gaps (MUST be zero)
ledger_sequence_gaps_total{container_id}

// Hash chain validation
ledger_hash_chain_valid{container_id}  // boolean
ledger_hash_mismatches_total{container_id}
```

#### Event Ordering & Causality
```rust
// Out-of-order events (CRITICAL)
ledger_out_of_order_events_total{container_id}

// Causality violations (previous_hash mismatch)
ledger_causality_violations_total{container_id}

// Concurrent writes (serialization conflicts)
ledger_write_conflicts_total{container_id}

// Retry attempts
ledger_append_retries_total{container_id, retry_count}
```

---

### 2. Projection Health (CRITICAL)

#### Projection Lag
```rust
// Current lag per projection (events behind)
projection_lag_events{projection_name, container_id}

// Time-based lag (seconds behind)
projection_lag_seconds{projection_name, container_id}

// Maximum observed lag
projection_max_lag_events{projection_name}

// Lag percentiles
projection_lag_p50{projection_name}
projection_lag_p95{projection_name}
projection_lag_p99{projection_name}
```

#### Projection Update Performance
```rust
// Update duration (how long to process one event)
projection_update_duration_seconds{projection_name, event_type}

// Batch processing
projection_batch_size{projection_name}
projection_batch_duration_seconds{projection_name}

// Update failures (can cause permanent lag)
projection_update_failures_total{projection_name, event_type, error_type}

// Retry queue depth
projection_retry_queue_depth{projection_name}
```

#### Projection Consistency
```rust
// Consistency checks
projection_consistency_checks_total{projection_name, result}
projection_inconsistencies_detected{projection_name, check_type}

// Rebuild operations
projection_rebuilds_total{projection_name, reason}
projection_rebuild_duration_seconds{projection_name}

// Snapshot age
projection_last_snapshot_age_seconds{projection_name}
```

---

### 3. Event Replay Tracking

#### Replay Operations
```rust
// Replay in progress
event_replay_active{container_id, projection_name}

// Replay progress
event_replay_progress_percent{container_id, projection_name}
event_replay_events_processed{container_id, projection_name}

// Replay performance
event_replay_throughput{container_id}  // events/sec
event_replay_duration_seconds{container_id, projection_name}

// Replay errors
event_replay_errors_total{container_id, event_type, error}
```

---

### 4. Data Integrity Metrics (ZERO tolerance)

#### Ledger Integrity
```rust
// Hash chain validation
ledger_integrity_checks_total{container_id, result}
ledger_integrity_violations_total{container_id, violation_type}

// Cryptographic verification
ledger_signature_verifications_total{container_id, result}
ledger_signature_failures_total{container_id, reason}

// Sequence continuity
ledger_sequence_continuity_checks{container_id}
ledger_sequence_gaps_detected{container_id, gap_size}

// Tamper detection
ledger_tamper_attempts_detected{container_id}
ledger_unauthorized_access_attempts{container_id}
```

#### Event Validation
```rust
// Schema validation
event_schema_validations_total{container_id, event_type, result}
event_schema_violations_total{container_id, event_type, field}

// Business rule validation
event_business_rule_checks_total{container_id, rule_type, result}
event_business_rule_violations_total{container_id, rule_type}

// Idempotency
event_duplicate_detections{container_id, event_id}
event_idempotency_violations{container_id}
```

---

### 5. Event Store Performance

#### Write Performance
```rust
// Write contention
ledger_write_lock_wait_seconds{container_id}
ledger_write_conflicts_per_second{container_id}

// Transaction isolation
ledger_serialization_failures_total{container_id}
ledger_deadlocks_total{container_id}

// WAL (Write-Ahead Log) metrics
postgres_wal_size_bytes
postgres_wal_age_seconds
postgres_checkpoint_duration_seconds
```

#### Read Performance
```rust
// Event stream reads
ledger_stream_reads_total{container_id, direction}
ledger_stream_read_duration_seconds{container_id}

// Tail following (SSE)
ledger_tail_followers{container_id}
ledger_tail_lag_seconds{container_id, follower_id}

// Historical queries
ledger_historical_query_duration_seconds{container_id, range}
ledger_full_scan_duration_seconds{container_id}
```

---

### 6. Container-Specific Metrics

#### Per-Container Health
```rust
// Event volume per container
ledger_events_per_container{container_id, event_type}

// Container growth rate
ledger_container_size_bytes{container_id}
ledger_container_growth_rate_bytes_per_hour{container_id}

// Container activity
ledger_container_write_rate{container_id}
ledger_container_read_rate{container_id}
ledger_container_last_write_seconds_ago{container_id}
```

---

### 7. Snapshot Metrics

#### Snapshot Creation
```rust
// Snapshot operations
projection_snapshot_created_total{projection_name}
projection_snapshot_creation_duration_seconds{projection_name}
projection_snapshot_size_bytes{projection_name}

// Snapshot age
projection_time_since_last_snapshot_seconds{projection_name}
projection_events_since_snapshot{projection_name}

// Snapshot restoration
projection_snapshot_restores_total{projection_name}
projection_snapshot_restore_duration_seconds{projection_name}
```

---

### 8. Eventual Consistency Tracking

#### Consistency Windows
```rust
// Time to consistency
event_to_projection_latency_seconds{container_id, projection_name}

// Stale read detection
projection_stale_reads_total{projection_name}
projection_staleness_seconds{projection_name}

// Read-your-writes violations
read_after_write_consistency_violations{container_id}
```

---

## Critical Alerts for Event Sourcing

### CRITICAL (P0) - Immediate Response

```yaml
# 1.  Ledger Append Failure
- alert: LedgerAppendFailure
  expr: rate(ledger_append_failures_total[1m]) > 0
  severity: critical
  description: "LEDGER APPEND FAILING - Data loss risk!"
  
# 2. Causality Violation
- alert: CausalityViolation
  expr: increase(ledger_causality_violations_total[5m]) > 0
  severity: critical
  description: "Event causality violated - integrity compromised!"

# 3. Hash Chain Broken
- alert: HashChainBroken
  expr: ledger_hash_chain_valid == 0
  severity: critical
  description: "Hash chain integrity broken - possible tampering!"

# 4. Sequence Gap
- alert: SequenceGap
  expr: increase(ledger_sequence_gaps_total[5m]) > 0
  severity: critical
  description: "Sequence gap detected - events missing!"

# 5. Projection Total Failure
- alert: ProjectionTotalFailure
  expr: rate(projection_update_failures_total[5m]) > 0.9
  severity: critical
  description: "Projection completely failing - system unusable!"
```

### HIGH (P1) - < 15 minutes

```yaml
# 6. High Projection Lag
- alert: HighProjectionLag
  expr: projection_lag_events > 1000
  for: 5m
  severity:  high
  description: "Projection lag exceeds 1000 events"

# 7. Projection Inconsistency
- alert: ProjectionInconsistency
  expr: increase(projection_inconsistencies_detected[10m]) > 0
  severity: high
  description: "Projection data inconsistent with ledger"

# 8. Write Conflict Storm
- alert: WriteConflictStorm
  expr: rate(ledger_write_conflicts_total[1m]) > 10
  severity: high
  description: "High write contention - performance degraded"
```

---

## Event Sourcing Dashboards

### Dashboard 1: Ledger Health

**Panels:**
1. Event Append Rate (line graph)
2. Append Latency p95/p99 (line graph)
3. Current Sequence per Container (stat)
4. Append Success Rate (gauge)
5. Causality Violations (counter - MUST be 0)
6. Hash Chain Integrity (boolean indicator)
7. Sequence Gaps (counter - MUST be 0)
8. Write Conflicts (line graph)

### Dashboard 2: Projection Health

**Panels:**
1. Projection Lag (heatmap by projection)
2. Lag Distribution (histogram)
3. Update Throughput (line graph)
4. Update Failures by Type (bar chart)
5. Consistency Check Results (stat)
6. Rebuild Operations (timeline)
7. Snapshot Age (stat per projection)
8. Time to Consistency (line graph)

### Dashboard 3: Event Flow

**Panels:**
1. Event Flow Diagram (Sankey/flow diagram)
   ```
   Ledger → Projection 1 → View 1
         → Projection 2 → View 2
         → Projection 3 → View 3
   ```
2. Events by Type (pie chart)
3. Events by Container (stacked area)
4. Event Processing Pipeline (stages with latency)
5. Backpressure Indicators (gauges)

### Dashboard 4: Data Integrity

**Panels:**
1. Integrity Check Status (stat grid)
2. Validation Failures (counter)
3. Tamper Attempts (counter - alert on any)
4. Signature Verification Rate (line graph)
5. Schema Violations (bar chart by field)
6. Duplicate Event Detection (counter)

---

## Specialized Logging for Event Sourcing

### Structured Event Log Format

```json
{
  "timestamp": "2024-12-29T10:00:00.123Z",
  "level": "INFO",
  "message": "Event appended to ledger",
  "trace_id": "abc123.. .",
  "span_id":  "def456...",
  
  // Event sourcing specific
  "event":  {
    "container_id": "C. Messenger",
    "sequence":  12345,
    "event_type": "message. created",
    "event_id": "evt_2024_001",
    "atom_hash": "abc123...",
    "entry_hash": "def456...",
    "previous_hash": "ghi789...",
    "causality_valid": true,
    "signature_valid": true
  },
  
  // Performance
  "performance": {
    "append_duration_ms": 15,
    "lock_wait_ms": 2,
    "validation_ms": 3,
    "commit_ms": 10
  },
  
  // Context
  "context": {
    "actor": "user_joao",
    "tenant_id": "T.UBL",
    "idempotency_key": "idem_123"
  }
}
```

### Critical Log Patterns to Alert On

```regex
# Causality violation
"causality_valid": false

# Hash mismatch
"signature_valid":false
"hash_mismatch"

# Sequence gap
"sequence_gap_detected"

# Append failure
"ledger_append_failed"
"event_type":"error".*"ledger"

# Projection failure
"projection_update_failed"
"projection_inconsistency"

# Replay issues
"replay_failed"
"replay_stuck"
```

---

## Tracing for Event Sourcing

### Trace Spans to Track

```
┌─────────────────────────────────────────────────────────┐
│ Span:  Message Send (User Action)                       │
│ ├─ Span:  Validate Event                                │
│ ├─ Span: Ledger Append                                 │
│ │  ├─ Span:  Acquire Write Lock                         │
│ │  ├─ Span: Validate Causality                         │
│ │  ├─ Span: Hash Computation                           │
│ │  ├─ Span:  Signature Generation                       │
│ │  └─ Span: Database Commit                            │
│ ├─ Span: Projection Update (async)                     │
│ │  ├─ Span:  Event Dispatch                             │
│ │  ├─ Span:  Projection 1 Update                        │
│ │  ├─ Span:  Projection 2 Update                        │
│ │  └─ Span: Projection 3 Update                        │
│ └─ Span: SSE Notification                              │
└─────────────────────────────────────────────────────────┘
```

### Critical Trace Attributes

```rust
// Event attributes
span.set_attribute("event. container_id", container_id);
span.set_attribute("event.sequence", sequence);
span.set_attribute("event.type", event_type);
span.set_attribute("event.hash", event_hash);
span.set_attribute("event.causality_valid", true);

// Projection attributes
span.set_attribute("projection.name", projection_name);
span.set_attribute("projection.lag_events", lag);
span.set_attribute("projection.update_duration_ms", duration);

// Performance attributes
span.set_attribute("db.lock_wait_ms", lock_wait);
span.set_attribute("db.transaction_isolation", "SERIALIZABLE");
```

---

## Event Sourcing SLIs/SLOs

### Service Level Indicators

| SLI | Target | Measurement |
|-----|--------|-------------|
| **Event Append Success Rate** | 99.99% | `ledger_append_total{status="success"} / ledger_append_total` |
| **Event Append Latency (p95)** | < 100ms | `histogram_quantile(0.95, ledger_append_duration_seconds)` |
| **Projection Lag (p95)** | < 100 events | `histogram_quantile(0.95, projection_lag_events)` |
| **Time to Consistency (p95)** | < 5s | `histogram_quantile(0.95, event_to_projection_latency_seconds)` |
| **Causality Violations** | 0 | `ledger_causality_violations_total` |
| **Hash Chain Integrity** | 100% | `ledger_hash_chain_valid` |
| **Projection Consistency** | 99.9% | `projection_consistency_checks{result="pass"} / projection_consistency_checks_total` |

### Error Budgets

```
Monthly Error Budget:
- Event Append Failures: 43 minutes (99.99% uptime)
- Projection Lag > 1000 events: 43 minutes
- Causality Violations: 0 (ZERO tolerance)
- Data Integrity Issues: 0 (ZERO tolerance)
```

---

## Monitoring Queries

### Prometheus Queries for Event Sourcing

```promql
# Current projection lag
projection_lag_events{projection_name="projection_messages"}

# Event append rate
rate(ledger_append_total[5m])

# Append success rate
sum(rate(ledger_append_total{status="success"}[5m])) 
/ 
sum(rate(ledger_append_total[5m]))

# Projection throughput
rate(projection_update_total[5m])

# Time to consistency
histogram_quantile(0.95, 
  rate(event_to_projection_latency_seconds_bucket[5m])
)

# Write conflicts
rate(ledger_write_conflicts_total[1m])

# Causality violations (MUST be 0)
increase(ledger_causality_violations_total[1h])

# Projection lag trend
deriv(projection_lag_events[5m])  # positive = falling behind
```

---

## Runbook:  Event Sourcing Issues

### Projection Lag Crisis

**Symptoms:** `projection_lag_events > 1000`

**Impact:** Stale data, users see outdated state

**Investigation:**
```bash
# Check projection update rate
curl -s localhost:9090/api/v1/query? query=rate\(projection_update_total\[5m\]\) | jq

# Check for errors
docker logs ubl-kernel 2>&1 | grep "projection_update_failed"

# Check database connections
docker exec postgres psql -U ubl -c "SELECT * FROM pg_stat_activity WHERE application_name LIKE '%projection%';"

# Check event stream
docker exec postgres psql -U ubl -c "SELECT container_id, MAX(sequence) FROM ledger_entries GROUP BY container_id;"
```

**Resolution:**
```bash
# Option 1: Scale projection workers
docker-compose up -d --scale projection-worker=3

# Option 2: Trigger fast-forward
curl -X POST localhost:8080/admin/projections/fast-forward \
  -d '{"projection":  "projection_messages", "target_sequence": 12345}'

# Option 3: Rebuild from snapshot
curl -X POST localhost:8080/admin/projections/rebuild \
  -d '{"projection": "projection_messages", "from_snapshot": true}'
```

### Causality Violation

**Symptoms:** `ledger_causality_violations_total > 0`

**Impact:** CRITICAL - Data integrity compromised

**Investigation:**
```bash
# Find violating event
docker logs ubl-kernel 2>&1 | grep "causality_violation"

# Check hash chain
curl -s localhost:8080/admin/ledger/verify-hash-chain? container_id=C.Messenger | jq

# Check sequence
docker exec postgres psql -U ubl -c "
  SELECT container_id, sequence, entry_hash, previous_hash 
  FROM ledger_entries 
  WHERE container_id = 'C.Messenger' 
  ORDER BY sequence DESC 
  LIMIT 10;
"
```

**Resolution:**
```bash
# STOP ALL WRITES IMMEDIATELY
curl -X POST localhost:8080/admin/ledger/freeze

# Verify extent of damage
./scripts/verify-ledger-integrity.sh

# If isolated:
# - Investigate root cause
# - Fix application bug
# - Resume writes

# If widespread:
# - INCIDENT:  Page on-call lead
# - Restore from backup
# - Replay events from last good state
```

---

## Testing Event Sourcing Observability

### Test Script

```bash
#!/bin/bash
# Test event sourcing observability

# 1. Normal operations
echo "Testing normal event flow..."
for i in {1..100}; do
  curl -X POST localhost:8080/v1/conversations/test/messages \
    -d '{"content": "test '$i'", "idempotency_key": "test_'$i'"}'
done

# Check metrics
curl localhost:9090/api/v1/query?query=ledger_append_total | jq

# 2. Write contention
echo "Testing write contention..."
for i in {1.. 10}; do
  (
    for j in {1..50}; do
      curl -X POST localhost:8080/v1/conversations/test/messages \
        -d '{"content": "concurrent", "idempotency_key":  "conc_'$i'_'$j'"}'
    done
  ) &
done
wait

# Check conflicts
curl localhost:9090/api/v1/query?query=ledger_write_conflicts_total | jq

# 3. Projection lag
echo "Testing projection lag..."
# Stop projection updater
docker-compose pause projection-worker

# Generate events
for i in {1..1000}; do
  curl -X POST localhost:8080/v1/conversations/test/messages \
    -d '{"content": "lag test", "idempotency_key": "lag_'$i'"}'
done

# Check lag
curl localhost:9090/api/v1/query?query=projection_lag_events | jq

# Resume
docker-compose unpause projection-worker

# Watch lag decrease
watch -n 1 'curl -s localhost:9090/api/v1/query?query=projection_lag_events | jq -r ".data.result[0].value[1]"'
```

---

## Summary

Event-sourced systems MUST have these observability items:

### Critical Metrics (MUST have)
✅ Event append success rate (99.99% target)
✅ Causality violations (MUST be 0)
✅ Hash chain integrity (MUST be 100%)
✅ Sequence gaps (MUST be 0)
✅ Projection lag (events & time)
✅ Projection consistency checks
✅ Event replay tracking
✅ Write conflicts & contention
✅ Time to consistency

### Critical Alerts (MUST have)
✅ Ledger append failures
✅ Causality violations
✅ Hash chain broken
✅ Sequence gaps
✅ Projection total failure
✅ High projection lag
✅ Data integrity violations

### Specialized Dashboards
✅ Ledger Health Dashboard
✅ Projection Health Dashboard
✅ Event Flow Visualization
✅ Data Integrity Dashboard

### Zero Tolerance Items
- Causality violations:  **0**
- Hash chain breaks: **0**
- Sequence gaps: **0**
- Data tampering: **0**
- Unauthorized access: **0**

---

**"In event sourcing, you can't fix the past - only observe it perfectly."**