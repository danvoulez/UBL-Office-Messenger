-- ============================================================================
-- UBL Office Projections - v1.0
-- ============================================================================
-- Consolidated from: 070_office_projections.sql
-- Reset: Squash por domínio - instalação do zero
-- C.Office Projections: LLM entities, sessions, handovers, and audit log

-- ============================================================================
-- ENTITIES (The "Chair" - persistent LLM identity)
-- ============================================================================

CREATE TABLE IF NOT EXISTS office_entities (
  entity_id       TEXT PRIMARY KEY,
  name            TEXT NOT NULL,
  entity_type     TEXT NOT NULL DEFAULT 'autonomous',
  public_key      TEXT NOT NULL,
  status          TEXT NOT NULL DEFAULT 'active',
  constitution    JSONB,
  baseline_narrative TEXT,
  total_sessions  BIGINT NOT NULL DEFAULT 0,
  total_tokens_used BIGINT NOT NULL DEFAULT 0,
  created_at_ms   BIGINT NOT NULL,
  updated_at_ms   BIGINT NOT NULL,
  entry_hash      TEXT,
  sequence        BIGINT
);

CREATE INDEX IF NOT EXISTS idx_office_entities_status ON office_entities(status);
CREATE INDEX IF NOT EXISTS idx_office_entities_type ON office_entities(entity_type);

COMMENT ON TABLE office_entities IS 'LLM Entity projections - the "Chair" (persistent identity)';

-- ============================================================================
-- SESSIONS (Ephemeral LLM session instances)
-- ============================================================================

CREATE TABLE IF NOT EXISTS office_sessions (
  session_id      TEXT PRIMARY KEY,
  entity_id       TEXT NOT NULL REFERENCES office_entities(entity_id),
  session_type    TEXT NOT NULL DEFAULT 'chat',
  mode            TEXT NOT NULL DEFAULT 'assisted',
  token_budget    BIGINT NOT NULL DEFAULT 100000,
  tokens_used     BIGINT,
  duration_ms     BIGINT,
  status          TEXT NOT NULL DEFAULT 'active',
  started_at_ms   BIGINT NOT NULL,
  completed_at_ms BIGINT
);

CREATE INDEX IF NOT EXISTS idx_office_sessions_entity ON office_sessions(entity_id);
CREATE INDEX IF NOT EXISTS idx_office_sessions_status ON office_sessions(status);
CREATE INDEX IF NOT EXISTS idx_office_sessions_started ON office_sessions(started_at_ms DESC);

COMMENT ON TABLE office_sessions IS 'LLM Session projections - "Who sits in the Chair" (ephemeral)';

-- ============================================================================
-- HANDOVERS (Knowledge transfer between sessions)
-- ============================================================================

CREATE TABLE IF NOT EXISTS office_handovers (
  handover_id     TEXT PRIMARY KEY,
  entity_id       TEXT NOT NULL REFERENCES office_entities(entity_id),
  session_id      TEXT NOT NULL REFERENCES office_sessions(session_id),
  content         JSONB NOT NULL,
  created_at_ms   BIGINT NOT NULL,
  entry_hash      TEXT,
  sequence        BIGINT
);

CREATE INDEX IF NOT EXISTS idx_office_handovers_entity ON office_handovers(entity_id);
CREATE INDEX IF NOT EXISTS idx_office_handovers_session ON office_handovers(session_id);
CREATE INDEX IF NOT EXISTS idx_office_handovers_created ON office_handovers(created_at_ms DESC);

COMMENT ON TABLE office_handovers IS 'Knowledge transfer between sessions';

-- ============================================================================
-- AUDIT LOG (Tool calls, decisions, policy violations)
-- ============================================================================

CREATE TABLE IF NOT EXISTS office_audit_log (
  audit_id        TEXT PRIMARY KEY,
  entity_id       TEXT NOT NULL,
  session_id      TEXT NOT NULL,
  job_id          TEXT,
  trace_id        TEXT NOT NULL,
  event_type      TEXT NOT NULL,
  event_data      JSONB NOT NULL,
  created_at_ms   BIGINT NOT NULL,
  entry_hash      TEXT,
  sequence        BIGINT
);

CREATE INDEX IF NOT EXISTS idx_office_audit_entity ON office_audit_log(entity_id);
CREATE INDEX IF NOT EXISTS idx_office_audit_session ON office_audit_log(session_id);
CREATE INDEX IF NOT EXISTS idx_office_audit_job ON office_audit_log(job_id) WHERE job_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_office_audit_type ON office_audit_log(event_type);
CREATE INDEX IF NOT EXISTS idx_office_audit_created ON office_audit_log(created_at_ms DESC);

COMMENT ON TABLE office_audit_log IS 'Audit trail of LLM actions, decisions, and policy violations';

-- ============================================================================
-- PRESENCE (Fix #10: Real-time entity presence tracking)
-- ============================================================================
-- Computable presence from ledger state + heartbeats
-- Status: available, working, waiting_on_you, offline, dreaming

CREATE TABLE IF NOT EXISTS office_presence (
  entity_id       TEXT PRIMARY KEY REFERENCES office_entities(entity_id),
  status          TEXT NOT NULL DEFAULT 'offline',
  current_job_id  TEXT,                      -- What they're working on
  current_activity TEXT,                     -- "Thinking", "Calling tool X", etc.
  last_heartbeat_ms BIGINT NOT NULL,         -- Last heartbeat timestamp
  last_action_ms  BIGINT,                    -- Last action timestamp
  session_id      TEXT,                      -- Current session if any
  updated_at_ms   BIGINT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_office_presence_status ON office_presence(status);
CREATE INDEX IF NOT EXISTS idx_office_presence_heartbeat ON office_presence(last_heartbeat_ms);

COMMENT ON TABLE office_presence IS 'Real-time entity presence derived from ledger events';

-- Function to update presence from events
-- Called by projector on relevant events
CREATE OR REPLACE FUNCTION update_entity_presence(
  p_entity_id TEXT,
  p_status TEXT,
  p_job_id TEXT DEFAULT NULL,
  p_activity TEXT DEFAULT NULL,
  p_session_id TEXT DEFAULT NULL
) RETURNS VOID AS $$
BEGIN
  INSERT INTO office_presence (entity_id, status, current_job_id, current_activity, last_heartbeat_ms, last_action_ms, session_id, updated_at_ms)
  VALUES (p_entity_id, p_status, p_job_id, p_activity, EXTRACT(EPOCH FROM NOW()) * 1000, EXTRACT(EPOCH FROM NOW()) * 1000, p_session_id, EXTRACT(EPOCH FROM NOW()) * 1000)
  ON CONFLICT (entity_id) DO UPDATE SET
    status = p_status,
    current_job_id = COALESCE(p_job_id, office_presence.current_job_id),
    current_activity = p_activity,
    last_action_ms = EXTRACT(EPOCH FROM NOW()) * 1000,
    session_id = COALESCE(p_session_id, office_presence.session_id),
    updated_at_ms = EXTRACT(EPOCH FROM NOW()) * 1000;
END;
$$ LANGUAGE plpgsql;

-- Function to mark offline entities (no heartbeat in last 30s)
CREATE OR REPLACE FUNCTION mark_stale_entities_offline() RETURNS INTEGER AS $$
DECLARE
  stale_count INTEGER;
BEGIN
  WITH updated AS (
    UPDATE office_presence
    SET status = 'offline', current_activity = NULL, current_job_id = NULL
    WHERE status != 'offline' 
      AND last_heartbeat_ms < (EXTRACT(EPOCH FROM NOW()) * 1000 - 30000)
    RETURNING entity_id
  )
  SELECT COUNT(*) INTO stale_count FROM updated;
  
  RETURN stale_count;
END;
$$ LANGUAGE plpgsql;

