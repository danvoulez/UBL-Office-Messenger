-- ADR-UBL-Console-001 v1.1 — multi-tenant + envelopes
-- Commands and Receipts for Permit → Execute → Receipt flow

BEGIN;

-- Commands table: stores pending jobs waiting for Runner execution
CREATE TABLE IF NOT EXISTS commands (
  jti           TEXT PRIMARY KEY,
  tenant_id     TEXT NOT NULL,
  job_id        TEXT NOT NULL UNIQUE,
  job_type      TEXT NOT NULL,
  params        JSONB NOT NULL,
  subject_hash  TEXT NOT NULL,
  policy_hash   TEXT NOT NULL,
  permit        JSONB NOT NULL,
  target        TEXT NOT NULL,
  office_id     TEXT NOT NULL,
  pending       INT  NOT NULL DEFAULT 1,
  issued_at     BIGINT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_commands_tenant_pending
  ON commands (tenant_id, pending, issued_at);

CREATE INDEX IF NOT EXISTS idx_commands_target_pending
  ON commands (target, pending, issued_at);

-- Receipts table: stores execution results
CREATE TABLE IF NOT EXISTS receipts (
  tenant_id     TEXT NOT NULL,
  job_id        TEXT PRIMARY KEY,
  status        TEXT NOT NULL,
  finished_at   BIGINT NOT NULL,
  logs_hash     TEXT NOT NULL,
  artifacts     JSONB NOT NULL DEFAULT '[]'::jsonb,
  usage         JSONB NOT NULL DEFAULT '{}'::jsonb,
  error         TEXT NOT NULL DEFAULT ''
);

CREATE INDEX IF NOT EXISTS idx_receipts_tenant
  ON receipts (tenant_id, finished_at);

-- Permits table: stores issued permits for audit
CREATE TABLE IF NOT EXISTS permits (
  jti           TEXT PRIMARY KEY,
  tenant_id     TEXT NOT NULL,
  actor_id      TEXT NOT NULL,
  job_type      TEXT NOT NULL,
  target        TEXT NOT NULL,
  subject_hash  TEXT NOT NULL,
  policy_hash   TEXT NOT NULL,
  approval_ref  TEXT,
  exp           BIGINT NOT NULL,
  issued_at     BIGINT NOT NULL,
  used          BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE INDEX IF NOT EXISTS idx_permits_tenant
  ON permits (tenant_id, issued_at);

CREATE INDEX IF NOT EXISTS idx_permits_jti_used
  ON permits (jti, used);

COMMIT;



