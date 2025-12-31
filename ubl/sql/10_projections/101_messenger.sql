-- ============================================================================
-- UBL Messenger Projections - v1.0
-- ============================================================================
-- Consolidated from: 040_messenger_v1.sql + 041_messenger_projections_complete.sql + 006_projections.sql (messenger parts)
-- Reset: Squash por domínio - instalação do zero

-- ============================================================================
-- MESSAGE CONTENT STORAGE
-- ============================================================================
-- The ledger stores only content_hash for privacy.
-- Actual content is stored here for retrieval.

CREATE TABLE IF NOT EXISTS message_content (
  message_id      TEXT PRIMARY KEY,
  content         TEXT NOT NULL,
  content_hash    TEXT NOT NULL,
  created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_message_content_hash ON message_content(content_hash);

COMMENT ON TABLE message_content IS 'Stores actual message content. Ledger stores only hash for privacy.';

-- ============================================================================
-- CONVERSATION PROJECTION
-- ============================================================================
-- Derived from conversation.* events

CREATE TABLE IF NOT EXISTS projection_conversations (
  conversation_id TEXT PRIMARY KEY,
  name            TEXT,
  is_group        BOOLEAN NOT NULL DEFAULT false,
  participants    TEXT[] NOT NULL DEFAULT '{}',
  created_by      TEXT NOT NULL,
  created_at      TIMESTAMPTZ NOT NULL,
  last_message_id TEXT,
  last_message_at TIMESTAMPTZ,
  last_event_hash TEXT NOT NULL,
  last_event_seq  BIGINT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_conv_participants ON projection_conversations USING GIN(participants);
CREATE INDEX IF NOT EXISTS idx_conv_last_message ON projection_conversations(last_message_at DESC);

COMMENT ON TABLE projection_conversations IS 'Derived state from conversation.* events in C.Messenger';

-- ============================================================================
-- ENTITY PROJECTION (for Messenger visibility)
-- ============================================================================
-- Derived from entity.registered events

CREATE TABLE IF NOT EXISTS projection_entities (
  entity_id       TEXT PRIMARY KEY,
  display_name    TEXT NOT NULL,
  entity_type     TEXT NOT NULL,
  avatar_hash     TEXT,
  status          TEXT DEFAULT 'online',
  registered_at   TIMESTAMPTZ NOT NULL,
  last_event_hash TEXT NOT NULL,
  last_event_seq  BIGINT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_entity_type ON projection_entities(entity_type);

COMMENT ON TABLE projection_entities IS 'Derived state from entity.registered events visible to Messenger';

-- ============================================================================
-- MESSAGE PROJECTION (from 006_projections.sql)
-- ============================================================================
-- Messages (latest state, not full history)

CREATE TABLE IF NOT EXISTS projection_messages (
  message_id      TEXT PRIMARY KEY,
  conversation_id TEXT NOT NULL,
  from_id         TEXT NOT NULL,
  content_hash    TEXT NOT NULL,
  timestamp       TIMESTAMPTZ NOT NULL,
  message_type    TEXT NOT NULL DEFAULT 'text',
  read_by         TEXT[] DEFAULT '{}',
  
  -- Source tracking
  last_event_hash TEXT NOT NULL,
  last_event_seq  BIGINT NOT NULL
);

CREATE INDEX IF NOT EXISTS ix_proj_messages_conversation ON projection_messages(conversation_id, timestamp DESC);

COMMENT ON TABLE projection_messages IS 'Derived state from message.* events (latest state, not full history)';

-- ============================================================================
-- JOB PROJECTION (Enhanced - from 041_messenger_projections_complete.sql)
-- ============================================================================
-- Tracks job state, owner, available_actions

CREATE TABLE IF NOT EXISTS projection_jobs (
  tenant_id TEXT NOT NULL,
  job_id TEXT PRIMARY KEY,
  conversation_id TEXT NOT NULL,
  title TEXT NOT NULL,
  goal TEXT NOT NULL,
  state TEXT NOT NULL, -- draft, proposed, approved, in_progress, waiting_input, completed, rejected, cancelled, failed
  owner_entity_id TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL,
  waiting_on TEXT[], -- entities waiting for input
  last_activity_at TIMESTAMPTZ,
  available_actions JSONB, -- buttons available for current state
  last_event_hash TEXT NOT NULL,
  last_event_seq BIGINT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_proj_jobs_tenant_conv ON projection_jobs(tenant_id, conversation_id);
CREATE INDEX IF NOT EXISTS idx_proj_jobs_state ON projection_jobs(state) WHERE state IN ('proposed', 'approved', 'in_progress', 'waiting_input');
CREATE INDEX IF NOT EXISTS idx_proj_jobs_owner ON projection_jobs(owner_entity_id);
CREATE INDEX IF NOT EXISTS idx_proj_jobs_updated ON projection_jobs(updated_at DESC);

COMMENT ON TABLE projection_jobs IS 'Enhanced job projection with state, owner, available_actions';

-- ============================================================================
-- JOB EVENTS TIMELINE (for drawer)
-- ============================================================================
-- Timeline items for job drawer, reconstructible by job_id

CREATE TABLE IF NOT EXISTS projection_job_events (
  tenant_id TEXT NOT NULL,
  job_id TEXT NOT NULL,
  cursor TEXT NOT NULL, -- seq:timestamp format for ordering
  ts TIMESTAMPTZ NOT NULL,
  event_id TEXT NOT NULL,
  event_type TEXT NOT NULL, -- job.created, job.state_changed, tool.called, tool.result, approval.decided
  actor_entity_id TEXT NOT NULL,
  timeline_item JSONB NOT NULL, -- rendered timeline item
  PRIMARY KEY (tenant_id, job_id, cursor)
);

CREATE INDEX IF NOT EXISTS idx_proj_job_events_job ON projection_job_events(tenant_id, job_id, ts DESC);
CREATE INDEX IF NOT EXISTS idx_proj_job_events_type ON projection_job_events(event_type);

COMMENT ON TABLE projection_job_events IS 'Timeline items for job drawer, reconstructible by job_id';

-- ============================================================================
-- JOB ARTIFACTS
-- ============================================================================
-- Artifacts produced by jobs (from tool.result events)

CREATE TABLE IF NOT EXISTS projection_job_artifacts (
  tenant_id TEXT NOT NULL,
  job_id TEXT NOT NULL,
  artifact_id TEXT NOT NULL,
  kind TEXT NOT NULL, -- file, link, record, quote
  title TEXT NOT NULL,
  url TEXT,
  mime_type TEXT,
  size_bytes BIGINT,
  event_id TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  PRIMARY KEY (tenant_id, job_id, artifact_id)
);

CREATE INDEX IF NOT EXISTS idx_proj_artifacts_job ON projection_job_artifacts(tenant_id, job_id);
CREATE INDEX IF NOT EXISTS idx_proj_artifacts_kind ON projection_job_artifacts(kind);

COMMENT ON TABLE projection_job_artifacts IS 'Artifacts produced by jobs from tool.result events';

-- ============================================================================
-- ENTITY PRESENCE
-- ============================================================================
-- Computed presence state from job state + activity

CREATE TABLE IF NOT EXISTS projection_presence (
  tenant_id TEXT NOT NULL,
  entity_id TEXT PRIMARY KEY,
  state TEXT NOT NULL, -- offline, available, working, waiting_on_you
  job_id TEXT, -- if working/waiting_on_you, which job
  since TIMESTAMPTZ NOT NULL,
  last_seen_at TIMESTAMPTZ NOT NULL,
  last_event_hash TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_proj_presence_state ON projection_presence(state);
CREATE INDEX IF NOT EXISTS idx_proj_presence_job ON projection_presence(job_id) WHERE job_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_proj_presence_last_seen ON projection_presence(last_seen_at DESC);

COMMENT ON TABLE projection_presence IS 'Computed entity presence from job state + activity';

-- ============================================================================
-- TIMELINE ITEMS (Optimized)
-- ============================================================================
-- Optimized timeline view for conversations (messages + job cards)

CREATE TABLE IF NOT EXISTS projection_timeline_items (
  tenant_id TEXT NOT NULL,
  conversation_id TEXT NOT NULL,
  cursor TEXT NOT NULL, -- seq:timestamp
  item_type TEXT NOT NULL, -- message, job_card, system
  item_data JSONB NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  PRIMARY KEY (tenant_id, conversation_id, cursor)
);

CREATE INDEX IF NOT EXISTS idx_proj_timeline_conv ON projection_timeline_items(tenant_id, conversation_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_proj_timeline_type ON projection_timeline_items(item_type);

COMMENT ON TABLE projection_timeline_items IS 'Optimized timeline view for conversations';

-- ============================================================================
-- APPROVALS PROJECTION (from 006_projections.sql)
-- ============================================================================

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

CREATE INDEX IF NOT EXISTS ix_proj_approvals_job ON projection_approvals(job_id);
CREATE INDEX IF NOT EXISTS ix_proj_approvals_status ON projection_approvals(status);

COMMENT ON TABLE projection_approvals IS 'Pending approvals for jobs';

-- ============================================================================
-- PROJECTION STATE TRACKING
-- ============================================================================

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

COMMENT ON TABLE projection_state IS 'Projection rebuild tracking';


