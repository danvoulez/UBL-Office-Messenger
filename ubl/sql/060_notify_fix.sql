-- 060_notify_fix.sql
-- FIX: Postgres NOTIFY 8KB limit (Gemini P0 #2)
--
-- Problem: row_to_json(NEW) can exceed 8KB, causing silent NOTIFY failure
-- Solution: Send only reference (container_id, sequence), SSE handler fetches full payload

-- Drop old trigger if exists
DROP TRIGGER IF EXISTS notify_ledger_insert ON ledger_entry;

-- New function: send only reference
CREATE OR REPLACE FUNCTION notify_ledger_ref() RETURNS trigger AS $$
BEGIN
  -- Only send container_id and sequence (always < 100 bytes)
  PERFORM pg_notify('ledger_events', json_build_object(
    'container_id', NEW.container_id,
    'sequence', NEW.sequence,
    'entry_hash', NEW.entry_hash
  )::text);
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Recreate trigger with new function
CREATE TRIGGER notify_ledger_insert
  AFTER INSERT ON ledger_entry
  FOR EACH ROW
  EXECUTE FUNCTION notify_ledger_ref();

-- Index for fast lookup by container + sequence
CREATE INDEX IF NOT EXISTS idx_ledger_container_seq 
  ON ledger_entry (container_id, sequence DESC);

COMMENT ON FUNCTION notify_ledger_ref IS 
  'Sends lightweight reference via pg_notify to avoid 8KB limit. SSE handler fetches full payload.';

