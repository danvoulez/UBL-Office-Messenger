-- ============================================================================
-- UBL Policy Schema - Pacts + Policy Engine
-- ============================================================================
-- Consolidated from: 007_pacts.sql + 008_policy_engine.sql
-- Reset: Squash por domínio - instalação do zero

-- ============================================================================
-- PACTS (SPEC-UBL-PACT v1.0)
-- ============================================================================
-- Authority and Consensus Rules

CREATE TABLE IF NOT EXISTS pact (
  -- Primary key
  pact_id         TEXT        PRIMARY KEY,
  
  -- Protocol version
  version         SMALLINT    NOT NULL DEFAULT 1,
  
  -- Scope: 'container', 'namespace', 'global'
  scope_type      TEXT        NOT NULL,
  scope_value     TEXT,       -- container_id or namespace prefix, null for global
  
  -- Intent classes this pact governs (array)
  intent_classes  TEXT[]      NOT NULL,
  
  -- Signature threshold
  threshold       SMALLINT    NOT NULL,
  
  -- Authorized signers (public keys)
  signers         TEXT[]      NOT NULL,
  
  -- Time window
  not_before      BIGINT      NOT NULL,
  not_after       BIGINT      NOT NULL,
  
  -- Risk level (0-5)
  risk_level      SMALLINT    NOT NULL,
  
  -- Metadata
  created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  created_by      TEXT        NOT NULL,
  
  -- Constraints
  CONSTRAINT valid_threshold CHECK (threshold > 0 AND threshold <= array_length(signers, 1)),
  CONSTRAINT valid_risk_level CHECK (risk_level >= 0 AND risk_level <= 5),
  CONSTRAINT valid_scope CHECK (
    (scope_type = 'global' AND scope_value IS NULL) OR
    (scope_type IN ('container', 'namespace') AND scope_value IS NOT NULL)
  ),
  CONSTRAINT valid_window CHECK (not_after > not_before)
);

-- Index for looking up pacts by scope
CREATE INDEX IF NOT EXISTS ix_pact_scope ON pact (scope_type, scope_value);

-- Index for looking up pacts by risk level
CREATE INDEX IF NOT EXISTS ix_pact_risk ON pact (risk_level);

COMMENT ON TABLE pact IS 'SPEC-UBL-PACT v1.0: Pact registry (authority & consensus)';

-- ============================================================================
-- PACT SIGNATURES
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

COMMENT ON TABLE pact_signatures IS 'Pact signatures collected for authorization';

-- ============================================================================
-- POLICY ENGINE (SPEC-UBL-POLICY v1.0)
-- ============================================================================

-- Policy definitions
CREATE TABLE IF NOT EXISTS policy_definitions (
  policy_id TEXT PRIMARY KEY,
  version TEXT NOT NULL,
  description TEXT NOT NULL,
  rules JSONB NOT NULL DEFAULT '[]'::jsonb,
  default_deny BOOLEAN NOT NULL DEFAULT true,
  bytecode_hash TEXT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS ix_policy_definitions_version 
  ON policy_definitions (policy_id, version);

-- Container to policy mappings
CREATE TABLE IF NOT EXISTS container_policies (
  container_id TEXT PRIMARY KEY,
  policy_id TEXT NOT NULL REFERENCES policy_definitions(policy_id),
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS ix_container_policies_policy_id 
  ON container_policies (policy_id);

-- Policy evaluation audit log
CREATE TABLE IF NOT EXISTS policy_evaluations (
  id BIGSERIAL PRIMARY KEY,
  evaluation_id TEXT NOT NULL UNIQUE,
  container_id TEXT NOT NULL,
  policy_id TEXT NOT NULL,
  policy_version TEXT NOT NULL,
  actor TEXT NOT NULL,
  intent_type TEXT,
  decision TEXT NOT NULL, -- 'allow' or 'deny'
  intent_class SMALLINT,
  required_pact TEXT,
  deny_reason TEXT,
  evaluation_time_us BIGINT, -- microseconds
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS ix_policy_evaluations_container 
  ON policy_evaluations (container_id, created_at DESC);
CREATE INDEX IF NOT EXISTS ix_policy_evaluations_policy 
  ON policy_evaluations (policy_id, created_at DESC);
CREATE INDEX IF NOT EXISTS ix_policy_evaluations_decision 
  ON policy_evaluations (decision, created_at DESC);

COMMENT ON TABLE policy_definitions IS 'SPEC-UBL-POLICY v1.0: Policy definitions with compiled rules';
COMMENT ON TABLE container_policies IS 'SPEC-UBL-POLICY v1.0: Container to policy mappings';
COMMENT ON TABLE policy_evaluations IS 'SPEC-UBL-POLICY v1.0: Audit log of policy evaluations';

-- ============================================================================
-- TRIGGERS (immutability guards)
-- ============================================================================

-- Pacts are immutable
CREATE OR REPLACE FUNCTION forbid_pact_mutation() RETURNS trigger AS $$
BEGIN
  RAISE EXCEPTION 'pacts are immutable - create a new pact instead';
END $$ LANGUAGE plpgsql;

DO $$ 
BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgname = 'pact_no_update') THEN
    CREATE TRIGGER pact_no_update BEFORE UPDATE ON pact
      FOR EACH ROW EXECUTE FUNCTION forbid_pact_mutation();
  END IF;
END $$;

-- Pact signatures are append-only
CREATE OR REPLACE FUNCTION deny_modify_pact_signatures() RETURNS trigger AS $$
BEGIN
  RAISE EXCEPTION 'Pact signatures are append-only';
END;
$$ LANGUAGE plpgsql;

DO $$ 
BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgname = 'pact_signatures_no_modify') THEN
    CREATE TRIGGER pact_signatures_no_modify 
      BEFORE UPDATE OR DELETE ON pact_signatures
      FOR EACH STATEMENT 
      EXECUTE FUNCTION deny_modify_pact_signatures();
  END IF;
END $$;

-- Update timestamps
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
  NEW.updated_at = NOW();
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DO $$ 
BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgname = 'policy_definitions_updated_at') THEN
    CREATE TRIGGER policy_definitions_updated_at
      BEFORE UPDATE ON policy_definitions
      FOR EACH ROW
      EXECUTE FUNCTION update_updated_at_column();
  END IF;
  
  IF NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgname = 'container_policies_updated_at') THEN
    CREATE TRIGGER container_policies_updated_at
      BEFORE UPDATE ON container_policies
      FOR EACH ROW
      EXECUTE FUNCTION update_updated_at_column();
  END IF;
END $$;

-- ============================================================================
-- SEED DATA
-- ============================================================================

-- Default policies for known containers
INSERT INTO policy_definitions (policy_id, version, description, rules, default_deny)
VALUES 
  ('default_C.Jobs', '1.0', 'Default policy for C.Jobs', '[]'::jsonb, true),
  ('default_C.Messenger', '1.0', 'Default policy for C.Messenger', '[]'::jsonb, true),
  ('default_C.Artifacts', '1.0', 'Default policy for C.Artifacts', '[]'::jsonb, true),
  ('default_C.Pacts', '1.0', 'Default policy for C.Pacts', '[]'::jsonb, true),
  ('default_C.Policy', '1.0', 'Default policy for C.Policy', '[]'::jsonb, true)
ON CONFLICT (policy_id) DO NOTHING;

-- Map containers to their default policies
INSERT INTO container_policies (container_id, policy_id)
VALUES 
  ('C.Jobs', 'default_C.Jobs'),
  ('C.Messenger', 'default_C.Messenger'),
  ('C.Artifacts', 'default_C.Artifacts'),
  ('C.Pacts', 'default_C.Pacts'),
  ('C.Policy', 'default_C.Policy')
ON CONFLICT (container_id) DO NOTHING;


