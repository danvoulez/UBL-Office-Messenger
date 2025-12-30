-- SPEC-UBL-ATOM v1.0 — Atom Storage for Projections
-- Atoms contain the semantic content referenced by ubl-links.
-- They are stored separately and linked by atom_hash.

CREATE TABLE IF NOT EXISTS ledger_atom (
  -- Primary key is the atom hash (content-addressed)
  atom_hash      TEXT        PRIMARY KEY,
  
  -- Container that this atom belongs to
  container_id   TEXT        NOT NULL,
  
  -- The actual atom content (canonical JSON)
  atom_data      JSONB       NOT NULL,
  
  -- Timestamp of storage
  ts_unix_ms     BIGINT      NOT NULL,
  
  -- Atom type (extracted from atom_data.type)
  atom_type      TEXT        GENERATED ALWAYS AS (atom_data->>'type') STORED
);

-- Index for querying by container
CREATE INDEX IF NOT EXISTS ix_ledger_atom_container ON ledger_atom (container_id);

-- Index for querying by atom type
CREATE INDEX IF NOT EXISTS ix_ledger_atom_type ON ledger_atom (atom_type);

-- Combined index for container + type (projection queries)
CREATE INDEX IF NOT EXISTS ix_ledger_atom_container_type ON ledger_atom (container_id, atom_type);

-- No mutations allowed
CREATE OR REPLACE FUNCTION forbid_atom_mutation() RETURNS trigger AS $$
BEGIN
  RAISE EXCEPTION 'ledger_atom is append-only (content-addressed)';
END $$ LANGUAGE plpgsql;

DO $$ BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgname = 'atom_no_update') THEN
    CREATE TRIGGER atom_no_update BEFORE UPDATE ON ledger_atom
      FOR EACH ROW EXECUTE PROCEDURE forbid_atom_mutation();
  END IF;
  IF NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgname = 'atom_no_delete') THEN
    CREATE TRIGGER atom_no_delete BEFORE DELETE ON ledger_atom
      FOR EACH ROW EXECUTE PROCEDURE forbid_atom_mutation();
  END IF;
END $$;

