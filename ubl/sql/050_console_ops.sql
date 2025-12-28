-- 050_console_ops.sql
-- Console API operational tables (Permits, Commands, Receipts)
-- These are OPERATIONAL (mutable, reconstructible), not ledger tables.

-- =============================================================================
-- PERMITS
-- =============================================================================

CREATE TABLE IF NOT EXISTS console_permits (
  jti TEXT PRIMARY KEY,
  office TEXT NOT NULL,
  action TEXT NOT NULL,
  target TEXT NOT NULL,
  args_json JSONB NOT NULL,
  risk TEXT NOT NULL,
  plan_hash TEXT NOT NULL,
  nonce TEXT NOT NULL,
  issued_at_ms BIGINT NOT NULL,
  exp_ms BIGINT NOT NULL,
  binding_hash TEXT NOT NULL,
  approver TEXT NOT NULL,
  sig TEXT NOT NULL,
  used BOOLEAN NOT NULL DEFAULT false
);

CREATE INDEX IF NOT EXISTS idx_console_permits_exp ON console_permits(exp_ms);
CREATE INDEX IF NOT EXISTS idx_console_permits_office ON console_permits(office);
CREATE INDEX IF NOT EXISTS idx_console_permits_binding ON console_permits(binding_hash);

-- =============================================================================
-- COMMANDS
-- =============================================================================

CREATE TABLE IF NOT EXISTS console_commands (
  command_id TEXT PRIMARY KEY,
  permit_jti TEXT NOT NULL REFERENCES console_permits(jti),
  office TEXT NOT NULL,
  action TEXT NOT NULL,
  target TEXT NOT NULL,
  args_json JSONB NOT NULL,
  risk TEXT NOT NULL,
  plan_hash TEXT NOT NULL,
  binding_hash TEXT NOT NULL,
  pending BOOLEAN NOT NULL DEFAULT true,
  created_at_ms BIGINT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_console_commands_pending ON console_commands(pending);
CREATE INDEX IF NOT EXISTS idx_console_commands_office ON console_commands(office);

-- =============================================================================
-- RECEIPTS
-- =============================================================================

CREATE TABLE IF NOT EXISTS console_receipts (
  command_id TEXT PRIMARY KEY REFERENCES console_commands(command_id),
  permit_jti TEXT NOT NULL,
  runner_id TEXT NOT NULL,
  status TEXT NOT NULL,  -- "OK" | "ERROR"
  logs_hash TEXT NOT NULL,
  ret_json JSONB NOT NULL,
  sig_runner TEXT NOT NULL,
  finished_at_ms BIGINT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_console_receipts_runner ON console_receipts(runner_id);
CREATE INDEX IF NOT EXISTS idx_console_receipts_status ON console_receipts(status);

-- =============================================================================
-- RUNNERS REGISTRY
-- =============================================================================

CREATE TABLE IF NOT EXISTS ubl_runners (
  runner_id TEXT PRIMARY KEY,
  pubkey_ed25519 TEXT NOT NULL,  -- hex-encoded 32-byte public key
  is_active BOOLEAN NOT NULL DEFAULT true,
  zone TEXT NOT NULL DEFAULT 'LAB_512',
  updated_at_ms BIGINT NOT NULL
);

-- Seed default runners
INSERT INTO ubl_runners (runner_id, pubkey_ed25519, is_active, zone, updated_at_ms)
VALUES 
  ('LAB_512', '0000000000000000000000000000000000000000000000000000000000000000', true, 'LAB_512', 0),
  ('LAB_256', '0000000000000000000000000000000000000000000000000000000000000000', true, 'LAB_256', 0)
ON CONFLICT (runner_id) DO NOTHING;

-- NOTE: Update the pubkey_ed25519 values with real keys before production!

