-- ============================================================================
-- UBL-FIX: Idempotency & Causality Guards
-- ============================================================================
-- Diamond Checklist: Prevent message duplication and event replay attacks
-- This migration adds client-side idempotency keys and strengthens causal guards

-- ============================================================================
-- 1. MESSAGES IDEMPOTENCY (Diamond Checklist #7)
-- ============================================================================
-- Add client_msg_id for client-side deduplication
ALTER TABLE projection_messages 
  ADD COLUMN IF NOT EXISTS client_msg_id TEXT;

-- UBL-FIX: Unique constraint prevents duplicate messages from client retries
CREATE UNIQUE INDEX IF NOT EXISTS uq_messages_conv_client
  ON projection_messages(conversation_id, client_msg_id)
  WHERE client_msg_id IS NOT NULL;

-- UBL-FIX: Index for causal guard performance
CREATE INDEX IF NOT EXISTS ix_messages_last_event_seq
  ON projection_messages(last_event_seq)
  WHERE last_event_seq IS NOT NULL;

COMMENT ON COLUMN projection_messages.client_msg_id IS 'Client-generated UUID for idempotent message submission';

-- ============================================================================
-- 2. CONVERSATIONS CAUSALITY
-- ============================================================================
-- Ensure last_event_seq exists for causal guards
ALTER TABLE projection_conversations
  ADD COLUMN IF NOT EXISTS last_event_seq BIGINT DEFAULT 0;

CREATE INDEX IF NOT EXISTS ix_conversations_last_event_seq
  ON projection_conversations(last_event_seq);

-- ============================================================================
-- 3. JOBS CAUSALITY (Diamond Checklist #1, #3)
-- ============================================================================
-- Verify last_event_seq columns exist
-- Note: These likely already exist from 101_messenger.sql, but we ensure they're here

-- For old projection_jobs table
DO $$ 
BEGIN
  IF NOT EXISTS (
    SELECT 1 FROM information_schema.columns 
    WHERE table_name = 'projection_jobs' 
    AND column_name = 'last_event_seq'
    AND table_schema = current_schema()
  ) THEN
    ALTER TABLE projection_jobs ADD COLUMN last_event_seq BIGINT DEFAULT 0;
  END IF;
END $$;

CREATE INDEX IF NOT EXISTS ix_jobs_last_event_seq
  ON projection_jobs(last_event_seq);

-- ============================================================================
-- 4. JOB EVENTS CAUSALITY
-- ============================================================================
-- Add sequence tracking if missing
DO $$ 
BEGIN
  IF NOT EXISTS (
    SELECT 1 FROM information_schema.columns 
    WHERE table_name = 'projection_job_events' 
    AND column_name = 'last_event_seq'
    AND table_schema = current_schema()
  ) THEN
    ALTER TABLE projection_job_events ADD COLUMN last_event_seq BIGINT;
  END IF;
END $$;

-- ============================================================================
-- 5. ENTITIES CAUSALITY
-- ============================================================================
-- Ensure causality for entity updates
DO $$ 
BEGIN
  IF NOT EXISTS (
    SELECT 1 FROM information_schema.columns 
    WHERE table_name = 'projection_entities' 
    AND column_name = 'last_event_seq'
    AND table_schema = current_schema()
  ) THEN
    ALTER TABLE projection_entities ADD COLUMN last_event_seq BIGINT DEFAULT 0;
  END IF;
END $$;

CREATE INDEX IF NOT EXISTS ix_entities_last_event_seq
  ON projection_entities(last_event_seq);

-- ============================================================================
-- 6. TIMELINE CAUSALITY
-- ============================================================================
-- Timeline items should be append-only, but add seq for tracking
DO $$ 
BEGIN
  IF NOT EXISTS (
    SELECT 1 FROM information_schema.columns 
    WHERE table_name = 'projection_timeline_items' 
    AND column_name = 'last_event_seq'
    AND table_schema = current_schema()
  ) THEN
    ALTER TABLE projection_timeline_items ADD COLUMN last_event_seq BIGINT;
  END IF;
END $$;

-- ============================================================================
-- 7. APPROVALS CAUSALITY
-- ============================================================================
-- Already has last_event_seq from 101_messenger.sql, just ensure index
CREATE INDEX IF NOT EXISTS ix_approvals_last_event_seq
  ON projection_approvals(last_event_seq);

-- ============================================================================
-- 8. PRESENCE CAUSALITY (Diamond Checklist #3)
-- ============================================================================
-- Ensure presence updates are causal
DO $$ 
BEGIN
  IF NOT EXISTS (
    SELECT 1 FROM information_schema.columns 
    WHERE table_name = 'projection_presence' 
    AND column_name = 'last_event_seq'
    AND table_schema = current_schema()
  ) THEN
    ALTER TABLE projection_presence ADD COLUMN last_event_seq BIGINT DEFAULT 0;
  END IF;
END $$;

CREATE INDEX IF NOT EXISTS ix_presence_last_event_seq
  ON projection_presence(last_event_seq);

-- ============================================================================
-- VERIFICATION QUERY
-- ============================================================================
-- Run this to verify all tables have causality guards:
-- 
-- SELECT table_name, column_name 
-- FROM information_schema.columns 
-- WHERE column_name = 'last_event_seq' 
--   AND table_schema = current_schema()
--   AND table_name LIKE 'projection_%'
-- ORDER BY table_name;
