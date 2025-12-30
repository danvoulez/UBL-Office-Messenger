-- SPEC-UBL-LEDGER v1.0 — Postgres DDL (append-only, verificável)
CREATE TABLE IF NOT EXISTS ledger_entry (
  -- Primary key
  container_id   TEXT        NOT NULL,
  sequence       BIGINT      NOT NULL,
  
  -- Hashes (causal chain)
  link_hash      TEXT        NOT NULL,
  previous_hash  TEXT        NOT NULL,
  entry_hash     TEXT        NOT NULL,
  
  -- Timestamp
  ts_unix_ms     BIGINT      NOT NULL,
  
  -- Metadata (full link commit)
  metadata       JSONB       DEFAULT '{}'::jsonb,
  
  PRIMARY KEY (container_id, sequence)
);

-- Indexes for queries
CREATE INDEX IF NOT EXISTS ix_ledger_entry_container_seq ON ledger_entry (container_id, sequence DESC);
CREATE INDEX IF NOT EXISTS ix_ledger_link_hash ON ledger_entry (link_hash);
CREATE INDEX IF NOT EXISTS ix_ledger_entry_hash ON ledger_entry (entry_hash);

-- NOTIFY trigger for SSE tail (PR10)
CREATE OR REPLACE FUNCTION notify_ledger_event() RETURNS trigger AS $$
BEGIN
  PERFORM pg_notify('ledger_events', row_to_json(NEW)::text);
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_ledger_events ON ledger_entry;
CREATE TRIGGER trg_ledger_events
AFTER INSERT ON ledger_entry
FOR EACH ROW EXECUTE FUNCTION notify_ledger_event();

-- Proibir UPDATE/DELETE
CREATE OR REPLACE FUNCTION forbid_mutation() RETURNS trigger AS $$
BEGIN
  RAISE EXCEPTION 'ledger_entry is append-only';
END $$ LANGUAGE plpgsql;

DO $$ BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgname = 'ledger_no_update') THEN
    CREATE TRIGGER ledger_no_update BEFORE UPDATE ON ledger_entry
      FOR EACH ROW EXECUTE PROCEDURE forbid_mutation();
  END IF;
  IF NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgname = 'ledger_no_delete') THEN
    CREATE TRIGGER ledger_no_delete BEFORE DELETE ON ledger_entry
      FOR EACH ROW EXECUTE PROCEDURE forbid_mutation();
  END IF;
END $$;

-- Inserção segura (pseudo)
-- BEGIN;
--   SELECT sequence, entry_hash FROM ledger_entry WHERE container_id=$1 ORDER BY sequence DESC LIMIT 1 FOR UPDATE;
--   -- checar previous_hash == entry_hash e sequence == last+1
--   INSERT INTO ledger_entry(...) VALUES (...);
-- COMMIT;
