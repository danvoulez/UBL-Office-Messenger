-- UBL Projections — Materialized views derived from ledger events
-- CRITICAL: These are READ-ONLY derived state. The ledger is the source of truth.

-- =============================================================================
-- C.Jobs Projection — Active jobs derived from job.* events
-- =============================================================================

CREATE TABLE IF NOT EXISTS projection_jobs (
  job_id         TEXT        PRIMARY KEY,
  conversation_id TEXT       NOT NULL,
  title          TEXT        NOT NULL,
  description    TEXT        NOT NULL,
  status         TEXT        NOT NULL DEFAULT 'pending',
  priority       TEXT        NOT NULL DEFAULT 'normal',
  assigned_to    TEXT,
  created_by     TEXT        NOT NULL,
  created_at     TIMESTAMPTZ NOT NULL,
  started_at     TIMESTAMPTZ,
  completed_at   TIMESTAMPTZ,
  cancelled_at   TIMESTAMPTZ,
  progress       INT         DEFAULT 0,
  progress_message TEXT,
  result_summary TEXT,
  result_artifacts JSONB     DEFAULT '[]'::jsonb,
  estimated_duration_seconds INT,
  estimated_value DECIMAL,
  
  -- Source tracking
  last_event_hash TEXT       NOT NULL,
  last_event_seq  BIGINT     NOT NULL
);

CREATE INDEX IF NOT EXISTS ix_proj_jobs_conversation ON projection_jobs (conversation_id);
CREATE INDEX IF NOT EXISTS ix_proj_jobs_status ON projection_jobs (status);
CREATE INDEX IF NOT EXISTS ix_proj_jobs_assigned ON projection_jobs (assigned_to);

-- =============================================================================
-- C.Jobs Approvals Projection — Pending approvals
-- =============================================================================

CREATE TABLE IF NOT EXISTS projection_approvals (
  approval_id    TEXT        PRIMARY KEY,
  job_id         TEXT        NOT NULL REFERENCES projection_jobs(job_id),
  action         TEXT        NOT NULL,
  reason         TEXT        NOT NULL,
  requested_by   TEXT        NOT NULL,
  requested_at   TIMESTAMPTZ NOT NULL,
  status         TEXT        NOT NULL DEFAULT 'pending',
  decided_by     TEXT,
  decided_at     TIMESTAMPTZ,
  decision       TEXT,
  decision_reason TEXT,
  
  -- Source tracking
  last_event_hash TEXT       NOT NULL,
  last_event_seq  BIGINT     NOT NULL
);

CREATE INDEX IF NOT EXISTS ix_proj_approvals_job ON projection_approvals (job_id);
CREATE INDEX IF NOT EXISTS ix_proj_approvals_status ON projection_approvals (status);

-- =============================================================================
-- C.Messenger Projection — Messages (latest state, not full history)
-- =============================================================================

CREATE TABLE IF NOT EXISTS projection_messages (
  message_id     TEXT        PRIMARY KEY,
  conversation_id TEXT       NOT NULL,
  from_id        TEXT        NOT NULL,
  content_hash   TEXT        NOT NULL,
  timestamp      TIMESTAMPTZ NOT NULL,
  message_type   TEXT        NOT NULL DEFAULT 'text',
  read_by        TEXT[]      DEFAULT '{}',
  
  -- Source tracking
  last_event_hash TEXT       NOT NULL,
  last_event_seq  BIGINT     NOT NULL
);

CREATE INDEX IF NOT EXISTS ix_proj_messages_conversation ON projection_messages (conversation_id, timestamp DESC);

-- =============================================================================
-- Projection rebuild tracking
-- =============================================================================

CREATE TABLE IF NOT EXISTS projection_state (
  container_id   TEXT        PRIMARY KEY,
  last_sequence  BIGINT      NOT NULL DEFAULT 0,
  last_hash      TEXT        NOT NULL DEFAULT '0x00',
  last_rebuild   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Initialize container states
INSERT INTO projection_state (container_id, last_sequence, last_hash)
VALUES 
  ('C.Jobs', 0, '0x00'),
  ('C.Messenger', 0, '0x00')
ON CONFLICT (container_id) DO NOTHING;

