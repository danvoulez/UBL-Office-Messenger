-- ============================================================================
-- UBL Core Schema - Ledger, Idempotency, Observability, Atoms
-- ============================================================================
-- Consolidated from: 001_ledger.sql + 002_idempotency.sql + 003_observability.sql + 005_atoms.sql
-- Reset: Squash por domínio - instalação do zero

-- ============================================================================
-- EXTENSIONS
-- ============================================================================
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- ============================================================================
-- LEDGER (SPEC-UBL-LEDGER v1.0)
-- ============================================================================
-- Append-only ledger with causal chain verification

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

COMMENT ON TABLE ledger_entry IS 'SPEC-UBL-LEDGER v1.0: Append-only ledger with causal chain verification';

-- ============================================================================
-- ATOM STORAGE (SPEC-UBL-ATOM v1.0)
-- ============================================================================
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

COMMENT ON TABLE ledger_atom IS 'SPEC-UBL-ATOM v1.0: Atom storage for projections (content-addressed)';

-- ============================================================================
-- IDEMPOTENCY
-- ============================================================================
-- Idempotency keys for safe retries

CREATE TABLE IF NOT EXISTS idempotency_key (
  container_id   CHAR(64),
  idem_key       TEXT,
  payload_hash   CHAR(64),
  response_json  JSONB,
  created_at     TIMESTAMPTZ DEFAULT (NOW() AT TIME ZONE 'UTC'),
  ttl_seconds    INTEGER DEFAULT 86400,
  PRIMARY KEY(container_id, idem_key)
);

COMMENT ON TABLE idempotency_key IS 'Idempotency keys for safe retries';

-- ============================================================================
-- OBSERVABILITY
-- ============================================================================
-- Views/dashboard config (placeholder - extend as needed)

-- TODO: Add metrics/logs tables as needed


