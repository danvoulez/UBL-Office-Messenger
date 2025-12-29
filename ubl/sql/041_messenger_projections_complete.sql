-- C.Messenger Complete Projections
-- Version: 1.0
-- Date: 2024-12-28
--
-- Adds missing projection tables for Gateway:
-- - projection_jobs (enhanced)
-- - projection_job_events (timeline for drawer)
-- - projection_job_artifacts (artifacts produced)
-- - projection_presence (entity presence)
-- - projection_timeline_items (optimized timeline view)

-- ============================================================================
-- Enhanced Job Projection
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

-- ============================================================================
-- Job Events Timeline (for drawer)
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

-- ============================================================================
-- Job Artifacts
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

-- ============================================================================
-- Entity Presence
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

-- ============================================================================
-- Timeline Items (Optimized)
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

-- ============================================================================
-- Comments
-- ============================================================================

COMMENT ON TABLE projection_jobs IS 'Enhanced job projection with state, owner, available_actions';
COMMENT ON TABLE projection_job_events IS 'Timeline items for job drawer, reconstructible by job_id';
COMMENT ON TABLE projection_job_artifacts IS 'Artifacts produced by jobs from tool.result events';
COMMENT ON TABLE projection_presence IS 'Computed entity presence from job state + activity';
COMMENT ON TABLE projection_timeline_items IS 'Optimized timeline view for conversations';

