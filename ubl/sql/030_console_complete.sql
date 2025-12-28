-- ============================================================================
-- UBL Console Complete Schema - v1.1
-- Includes: permits, commands, receipts, pacts, pact_signatures
-- ============================================================================
-- Run with: psql -U ubl_kernel -d ubl_ledger -f 030_console_complete.sql
-- ============================================================================

-- Enable required extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- ============================================================================
-- PERMITS TABLE
-- Stores issued permits for audit trail
-- ============================================================================
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

-- ============================================================================
-- COMMANDS TABLE
-- Stores commands issued for Runner execution
-- ============================================================================
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

CREATE INDEX IF NOT EXISTS idx_commands_pending ON commands(tenant_id, target, pending) 
    WHERE pending = 1;
CREATE INDEX IF NOT EXISTS idx_commands_job_id ON commands(job_id);

-- ============================================================================
-- RECEIPTS TABLE
-- Stores execution receipts from Runner
-- ============================================================================
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

CREATE INDEX IF NOT EXISTS idx_receipts_tenant ON receipts(tenant_id);
CREATE INDEX IF NOT EXISTS idx_receipts_status ON receipts(status);

-- ============================================================================
-- PACTS TABLE
-- Stores pact definitions for Evolution/Entropy governance
-- ============================================================================
CREATE TABLE IF NOT EXISTS pact (
    pact_id         TEXT PRIMARY KEY,
    version         SMALLINT NOT NULL DEFAULT 1,
    scope_type      TEXT NOT NULL CHECK (scope_type IN ('global', 'container', 'namespace')),
    scope_value     TEXT,  -- container_id for 'container', prefix for 'namespace'
    intent_classes  TEXT[] NOT NULL,  -- ['Evolution'], ['Entropy'], etc.
    threshold       SMALLINT NOT NULL CHECK (threshold >= 1),
    signers         TEXT[] NOT NULL,  -- List of authorized signer SIDs
    not_before      BIGINT NOT NULL,  -- Unix ms
    not_after       BIGINT NOT NULL,  -- Unix ms
    risk_level      SMALLINT NOT NULL DEFAULT 5,  -- L0-L5
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by      TEXT NOT NULL,
    
    CONSTRAINT pact_time_valid CHECK (not_after > not_before)
);

CREATE INDEX IF NOT EXISTS idx_pact_scope ON pact(scope_type, scope_value);
CREATE INDEX IF NOT EXISTS idx_pact_active ON pact(not_before, not_after);

-- ============================================================================
-- PACT_SIGNATURES TABLE
-- Stores signatures collected for pacts
-- ============================================================================
CREATE TABLE IF NOT EXISTS pact_signatures (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    pact_id         TEXT NOT NULL REFERENCES pact(pact_id),
    signer          TEXT NOT NULL,
    signature       TEXT NOT NULL,  -- Ed25519 signature (hex)
    atom_hash       TEXT NOT NULL,  -- Hash of the atom being authorized
    signed_at       BIGINT NOT NULL,
    verified        BOOLEAN NOT NULL DEFAULT false,
    verified_at     BIGINT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    UNIQUE(pact_id, signer, atom_hash)
);

CREATE INDEX IF NOT EXISTS idx_pact_sig_pact ON pact_signatures(pact_id);
CREATE INDEX IF NOT EXISTS idx_pact_sig_verified ON pact_signatures(pact_id, verified) 
    WHERE verified = true;

-- ============================================================================
-- ID_AGENTS TABLE (for ASC validation)
-- Stores agent identities for LLMs and Apps
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
-- ASC TABLE (Agent Signing Certificates)
-- Stores issued ASCs for audit
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
CREATE INDEX IF NOT EXISTS idx_asc_active ON asc_tokens(not_before, not_after) 
    WHERE revoked = false;

-- ============================================================================
-- APPEND-ONLY TRIGGERS (for ledger tables)
-- ============================================================================

-- Deny UPDATE/DELETE on permits (audit trail)
CREATE OR REPLACE FUNCTION deny_modify_permits() RETURNS trigger AS $$
BEGIN
    RAISE EXCEPTION 'Permits are append-only for audit purposes';
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS permits_no_modify ON permits;
CREATE TRIGGER permits_no_modify 
    BEFORE UPDATE OR DELETE ON permits
    FOR EACH STATEMENT 
    EXECUTE FUNCTION deny_modify_permits();

-- Deny UPDATE/DELETE on receipts (audit trail)
CREATE OR REPLACE FUNCTION deny_modify_receipts() RETURNS trigger AS $$
BEGIN
    RAISE EXCEPTION 'Receipts are append-only for audit purposes';
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS receipts_no_modify ON receipts;
CREATE TRIGGER receipts_no_modify 
    BEFORE UPDATE OR DELETE ON receipts
    FOR EACH STATEMENT 
    EXECUTE FUNCTION deny_modify_receipts();

-- Deny UPDATE/DELETE on pact_signatures
CREATE OR REPLACE FUNCTION deny_modify_pact_signatures() RETURNS trigger AS $$
BEGIN
    RAISE EXCEPTION 'Pact signatures are append-only';
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS pact_signatures_no_modify ON pact_signatures;
CREATE TRIGGER pact_signatures_no_modify 
    BEFORE UPDATE OR DELETE ON pact_signatures
    FOR EACH STATEMENT 
    EXECUTE FUNCTION deny_modify_pact_signatures();

-- ============================================================================
-- SEED DATA: Default Pact for Evolution (requires 3 human signers)
-- ============================================================================
INSERT INTO pact (pact_id, scope_type, intent_classes, threshold, signers, not_before, not_after, risk_level, created_by)
VALUES (
    'pact:evolution:global:v1',
    'global',
    ARRAY['Evolution'],
    3,  -- Requires 3 signatures
    ARRAY['ubl:sid:person:admin1', 'ubl:sid:person:admin2', 'ubl:sid:person:admin3'],
    0,  -- Always valid from
    9999999999999,  -- Far future
    5,  -- L5
    'system'
) ON CONFLICT (pact_id) DO NOTHING;

-- Default Pact for Entropy (requires 2 human signers)
INSERT INTO pact (pact_id, scope_type, intent_classes, threshold, signers, not_before, not_after, risk_level, created_by)
VALUES (
    'pact:entropy:global:v1',
    'global',
    ARRAY['Entropy'],
    2,  -- Requires 2 signatures
    ARRAY['ubl:sid:person:admin1', 'ubl:sid:person:admin2', 'ubl:sid:person:admin3'],
    0,
    9999999999999,
    5,  -- L5
    'system'
) ON CONFLICT (pact_id) DO NOTHING;

-- ============================================================================
-- GRANTS
-- ============================================================================

-- Kernel has full access
GRANT ALL ON permits TO ubl_kernel;
GRANT ALL ON commands TO ubl_kernel;
GRANT ALL ON receipts TO ubl_kernel;
GRANT ALL ON pact TO ubl_kernel;
GRANT ALL ON pact_signatures TO ubl_kernel;
GRANT ALL ON id_agents TO ubl_kernel;
GRANT ALL ON asc_tokens TO ubl_kernel;

-- Read-only role for projections/queries
GRANT SELECT ON permits TO ubl_readonly;
GRANT SELECT ON commands TO ubl_readonly;
GRANT SELECT ON receipts TO ubl_readonly;
GRANT SELECT ON pact TO ubl_readonly;
GRANT SELECT ON pact_signatures TO ubl_readonly;

-- ============================================================================
-- VERIFICATION
-- ============================================================================
DO $$
BEGIN
    RAISE NOTICE '✅ Console schema v1.1 installed';
    RAISE NOTICE '   - permits (append-only)';
    RAISE NOTICE '   - commands';
    RAISE NOTICE '   - receipts (append-only)';
    RAISE NOTICE '   - pact + pact_signatures';
    RAISE NOTICE '   - id_agents + asc_tokens';
    RAISE NOTICE '   - Default Evolution/Entropy pacts seeded';
END $$;

