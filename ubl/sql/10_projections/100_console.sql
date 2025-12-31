-- ============================================================================
-- UBL Console Projections - v1.1
-- ============================================================================
-- Consolidated from: 020_console_v1_1.sql + 030_console_complete.sql + 050_console_ops.sql
-- Reset: Squash por domínio - instalação do zero
-- ADR-UBL-Console-001 v1.1 — multi-tenant + envelopes

-- ============================================================================
-- GATEWAY IDEMPOTENCY (Fix #4)
-- ============================================================================
-- Persistent idempotency store for Gateway operations.
-- Replaces in-memory HashMap to survive restarts.
-- TTL-based cleanup via created_at.

CREATE TABLE IF NOT EXISTS gateway_idempotency (
  idem_key        TEXT PRIMARY KEY,
  tenant_id       TEXT NOT NULL,
  status          TEXT NOT NULL CHECK (status IN ('pending', 'completed', 'failed')),
  response_body   JSONB,
  event_ids       TEXT[] NOT NULL DEFAULT '{}',
  created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for cleanup job (remove entries older than 24h)
CREATE INDEX IF NOT EXISTS idx_gateway_idem_created ON gateway_idempotency(created_at);
CREATE INDEX IF NOT EXISTS idx_gateway_idem_tenant ON gateway_idempotency(tenant_id);

COMMENT ON TABLE gateway_idempotency IS 'Fix #4: Persistent idempotency store for Gateway (survives restarts)';

-- ============================================================================
-- PERMITS (ADR-UBL-Console-001 v1.1)
-- ============================================================================
-- Stores issued permits for audit trail

CREATE TABLE IF NOT EXISTS permits (
  jti             TEXT PRIMARY KEY,
  tenant_id       TEXT NOT NULL,
  actor_id        TEXT NOT NULL,
  job_type        TEXT NOT NULL,
  target          TEXT NOT NULL,
  subject_hash    TEXT NOT NULL,
  policy_hash     TEXT NOT NULL,
  approval_ref    TEXT,
  risk            TEXT NOT NULL DEFAULT 'L0',
  exp             BIGINT NOT NULL,
  issued_at       BIGINT NOT NULL,
  used            BOOLEAN NOT NULL DEFAULT false,
  used_at         BIGINT,
  created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_permits_tenant ON permits(tenant_id);
CREATE INDEX IF NOT EXISTS idx_permits_actor ON permits(actor_id);
CREATE INDEX IF NOT EXISTS idx_permits_used ON permits(used) WHERE used = false;
CREATE INDEX IF NOT EXISTS idx_permits_jti_used ON permits(jti, used);

COMMENT ON TABLE permits IS 'ADR-UBL-Console-001 v1.1: Issued permits for audit trail (append-only)';

-- ============================================================================
-- COMMANDS (ADR-UBL-Console-001 v1.1)
-- ============================================================================
-- Stores commands issued for Runner execution

CREATE TABLE IF NOT EXISTS commands (
  jti             TEXT PRIMARY KEY,
  tenant_id       TEXT NOT NULL,
  job_id          TEXT NOT NULL UNIQUE,
  job_type        TEXT NOT NULL,
  params          JSONB NOT NULL,
  subject_hash    TEXT NOT NULL,
  policy_hash     TEXT NOT NULL,
  permit          JSONB NOT NULL,
  target          TEXT NOT NULL,
  office_id       TEXT NOT NULL,
  pending         INT NOT NULL DEFAULT 1,
  issued_at       BIGINT NOT NULL,
  created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_commands_tenant_pending ON commands(tenant_id, pending, issued_at);
CREATE INDEX IF NOT EXISTS idx_commands_target_pending ON commands(target, pending, issued_at);
CREATE INDEX IF NOT EXISTS idx_commands_pending ON commands(tenant_id, target, pending) WHERE pending = 1;
CREATE INDEX IF NOT EXISTS idx_commands_job_id ON commands(job_id);

COMMENT ON TABLE commands IS 'ADR-UBL-Console-001 v1.1: Commands issued for Runner execution';

-- ============================================================================
-- RECEIPTS (ADR-UBL-Console-001 v1.1)
-- ============================================================================
-- Stores execution receipts from Runner

CREATE TABLE IF NOT EXISTS receipts (
  id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  tenant_id       TEXT NOT NULL,
  job_id          TEXT NOT NULL UNIQUE REFERENCES commands(job_id),
  status          TEXT NOT NULL,  -- 'OK', 'ERROR', 'TIMEOUT'
  finished_at     BIGINT NOT NULL,
  logs_hash       TEXT NOT NULL,
  artifacts       JSONB NOT NULL DEFAULT '[]',
  usage           JSONB NOT NULL DEFAULT '{}',
  error           TEXT DEFAULT '',
  created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_receipts_tenant ON receipts(tenant_id, finished_at);
CREATE INDEX IF NOT EXISTS idx_receipts_status ON receipts(status);

COMMENT ON TABLE receipts IS 'ADR-UBL-Console-001 v1.1: Execution receipts from Runner (append-only)';

-- ============================================================================
-- CONSOLE OPERATIONAL TABLES (050_console_ops.sql)
-- ============================================================================
-- These are OPERATIONAL (mutable, reconstructible), not ledger tables.

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

-- ============================================================================
-- RUNNERS REGISTRY
-- ============================================================================

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

-- ============================================================================
-- ID_AGENTS (for ASC validation)
-- ============================================================================

CREATE TABLE IF NOT EXISTS id_agents (
  agent_id        TEXT PRIMARY KEY,  -- e.g., 'ubl:sid:llm:gpt4'
  kind            TEXT NOT NULL CHECK (kind IN ('llm', 'app')),
  display_name    TEXT NOT NULL,
  pubkey          TEXT NOT NULL,  -- Ed25519 public key (hex)
  created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  revoked         BOOLEAN NOT NULL DEFAULT false,
  revoked_at      TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_agents_kind ON id_agents(kind);
CREATE INDEX IF NOT EXISTS idx_agents_active ON id_agents(agent_id) WHERE revoked = false;

-- ============================================================================
-- ASC TOKENS (Agent Signing Certificates)
-- ============================================================================

CREATE TABLE IF NOT EXISTS asc_tokens (
  asc_id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  agent_id        TEXT NOT NULL REFERENCES id_agents(agent_id),
  containers      TEXT[] NOT NULL,
  operations      TEXT[] NOT NULL,
  max_physics_delta BIGINT NOT NULL DEFAULT 0,
  not_before      TIMESTAMPTZ NOT NULL,
  not_after       TIMESTAMPTZ NOT NULL,
  signature       BYTEA NOT NULL,
  issued_by       TEXT NOT NULL,
  revoked         BOOLEAN NOT NULL DEFAULT false,
  revoked_at      TIMESTAMPTZ,
  created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_asc_agent ON asc_tokens(agent_id);
CREATE INDEX IF NOT EXISTS idx_asc_active ON asc_tokens(not_before, not_after) WHERE revoked = false;

-- ============================================================================
-- IMMUTABILITY GUARDS (append-only tables)
-- ============================================================================

CREATE OR REPLACE FUNCTION deny_modify_permits() RETURNS trigger AS $$
BEGIN
  RAISE EXCEPTION 'Permits are append-only for audit purposes';
END;
$$ LANGUAGE plpgsql;

DO $$ 
BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgname = 'permits_no_modify') THEN
    CREATE TRIGGER permits_no_modify 
      BEFORE UPDATE OR DELETE ON permits
      FOR EACH STATEMENT 
      EXECUTE FUNCTION deny_modify_permits();
  END IF;
END $$;

CREATE OR REPLACE FUNCTION deny_modify_receipts() RETURNS trigger AS $$
BEGIN
  RAISE EXCEPTION 'Receipts are append-only for audit purposes';
END;
$$ LANGUAGE plpgsql;

DO $$ 
BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgname = 'receipts_no_modify') THEN
    CREATE TRIGGER receipts_no_modify 
      BEFORE UPDATE OR DELETE ON receipts
      FOR EACH STATEMENT 
      EXECUTE FUNCTION deny_modify_receipts();
  END IF;
END $$;


