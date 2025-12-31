-- ============================================================================
-- UBL Triggers - NOTIFY tail + Immutability Guards
-- ============================================================================
-- Consolidated from: 060_notify_fix.sql + 999_ubl_tail_notify.sql + ledger guards
-- Reset: Payload MINÚSCULO (cid:seq) para evitar limite 8KB do PostgreSQL NOTIFY

-- ============================================================================
-- NOTIFY TRIGGER (SSE tail - payload cid:seq apenas)
-- ============================================================================

CREATE OR REPLACE FUNCTION ubl_tail_notify() RETURNS trigger AS $$
BEGIN
  -- Payload MINÚSCULO: 'container_id:sequence' (sempre < 100 bytes)
  PERFORM pg_notify('ubl_tail', NEW.container_id || ':' || NEW.sequence::text);
  RETURN NEW;
END; $$ LANGUAGE plpgsql;

DO $$ 
BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgname = 'trg_tail_notify') THEN
    CREATE TRIGGER trg_tail_notify
    AFTER INSERT ON ledger_entry
    FOR EACH ROW EXECUTE FUNCTION ubl_tail_notify();
  END IF;
END $$;

COMMENT ON FUNCTION ubl_tail_notify IS 
  'Sends lightweight reference via pg_notify (cid:seq) to avoid 8KB limit. SSE handler fetches full payload.';

-- ============================================================================
-- IMMUTABILITY GUARDS (ledger_entry)
-- ============================================================================

CREATE OR REPLACE FUNCTION forbid_mutation() RETURNS trigger AS $$
BEGIN
  RAISE EXCEPTION 'ledger_entry is append-only';
END $$ LANGUAGE plpgsql;

DO $$ 
BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgname = 'ledger_no_update') THEN
    CREATE TRIGGER ledger_no_update BEFORE UPDATE ON ledger_entry
      FOR EACH ROW EXECUTE FUNCTION forbid_mutation();
  END IF;
  
  IF NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgname = 'ledger_no_delete') THEN
    CREATE TRIGGER ledger_no_delete BEFORE DELETE ON ledger_entry
      FOR EACH ROW EXECUTE FUNCTION forbid_mutation();
  END IF;
END $$;

-- ============================================================================
-- IMMUTABILITY GUARDS (ledger_atom)
-- ============================================================================

CREATE OR REPLACE FUNCTION forbid_atom_mutation() RETURNS trigger AS $$
BEGIN
  RAISE EXCEPTION 'ledger_atom is append-only (content-addressed)';
END $$ LANGUAGE plpgsql;

DO $$ 
BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgname = 'atom_no_update') THEN
    CREATE TRIGGER atom_no_update BEFORE UPDATE ON ledger_atom
      FOR EACH ROW EXECUTE FUNCTION forbid_atom_mutation();
  END IF;
  
  IF NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgname = 'atom_no_delete') THEN
    CREATE TRIGGER atom_no_delete BEFORE DELETE ON ledger_atom
      FOR EACH ROW EXECUTE FUNCTION forbid_atom_mutation();
  END IF;
END $$;


