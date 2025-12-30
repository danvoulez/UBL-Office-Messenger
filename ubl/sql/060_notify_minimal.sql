-- ============================================================================
-- 060_notify_minimal.sql
-- SSE Minimal Payload Trigger
-- 
-- Per Dan's patch: NOTIFY payload must be <100B to avoid PostgreSQL 8KB limit
-- Emits only: {"container_id": "...", "sequence": N}
-- Client fetches full entry via GET /ledger/entry/:container_id/:sequence
-- ============================================================================

-- Drop existing trigger if exists
DROP TRIGGER IF EXISTS trg_ubl_tail ON ledger_entry;
DROP FUNCTION IF EXISTS ubl_notify_minimal();

-- Create minimal notify function
CREATE OR REPLACE FUNCTION ubl_notify_minimal() RETURNS trigger AS $$
BEGIN
  -- Emit minimal payload: only container_id and sequence
  -- This keeps payload well under 8KB PostgreSQL NOTIFY limit
  PERFORM pg_notify(
    'ubl_tail',
    json_build_object(
      'container_id', NEW.container_id,
      'sequence', NEW.sequence
    )::text
  );
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create trigger on ledger_entry insert
CREATE TRIGGER trg_ubl_tail
AFTER INSERT ON ledger_entry
FOR EACH ROW EXECUTE PROCEDURE ubl_notify_minimal();

-- Grant execute to app user
GRANT EXECUTE ON FUNCTION ubl_notify_minimal() TO PUBLIC;

-- Verify trigger exists
DO $$
BEGIN
  IF EXISTS (
    SELECT 1 FROM pg_trigger WHERE tgname = 'trg_ubl_tail'
  ) THEN
    RAISE NOTICE 'SSE minimal trigger created successfully';
  ELSE
    RAISE EXCEPTION 'Failed to create SSE minimal trigger';
  END IF;
END $$;
