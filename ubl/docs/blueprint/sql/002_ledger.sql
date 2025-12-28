-- 002_ledger.sql (reviewed)
CREATE TABLE IF NOT EXISTS ledger_entry (
  container_id text NOT NULL,
  sequence bigint NOT NULL,
  link_hash text NOT NULL,
  previous_hash text NOT NULL,
  entry_hash text NOT NULL,
  ts_unix_ms bigint NOT NULL,
  metadata jsonb DEFAULT '{}'::jsonb,
  PRIMARY KEY (container_id, sequence)
);

-- Append-only guard (no UPDATE/DELETE)
CREATE OR REPLACE FUNCTION prevent_ledger_mutation() RETURNS trigger AS $$
BEGIN
  RAISE EXCEPTION 'ledger_entry is append-only';
END; $$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_ledger_block_update ON ledger_entry;
CREATE TRIGGER trg_ledger_block_update
BEFORE UPDATE OR DELETE ON ledger_entry
FOR EACH ROW EXECUTE FUNCTION prevent_ledger_mutation();

-- Notify SSE
CREATE OR REPLACE FUNCTION notify_ledger_event() RETURNS trigger AS $$
BEGIN
  PERFORM pg_notify('ledger_events', row_to_json(NEW)::text);
  RETURN NEW;
END; $$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_ledger_events ON ledger_entry;
CREATE TRIGGER trg_ledger_events AFTER INSERT ON ledger_entry
FOR EACH ROW EXECUTE FUNCTION notify_ledger_event();

-- Helpful indexes
CREATE INDEX IF NOT EXISTS idx_ledger_entry_hash ON ledger_entry(entry_hash);
CREATE INDEX IF NOT EXISTS idx_ledger_container_seq ON ledger_entry(container_id, sequence DESC);
