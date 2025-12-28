-- UBL Pact Registry — Authority and Consensus Rules
-- SPEC-UBL-PACT v1.0

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

-- No mutations allowed after creation (pacts are immutable)
CREATE OR REPLACE FUNCTION forbid_pact_mutation() RETURNS trigger AS $$
BEGIN
  RAISE EXCEPTION 'pacts are immutable - create a new pact instead';
END $$ LANGUAGE plpgsql;

DO $$ BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgname = 'pact_no_update') THEN
    CREATE TRIGGER pact_no_update BEFORE UPDATE ON pact
      FOR EACH ROW EXECUTE PROCEDURE forbid_pact_mutation();
  END IF;
END $$;

-- Note: DELETE is allowed for expired pacts cleanup

-- Example: Insert a global pact for Evolution (L5)
-- INSERT INTO pact (pact_id, scope_type, intent_classes, threshold, signers, not_before, not_after, risk_level, created_by)
-- VALUES (
--   'evolution_global_001',
--   'global',
--   ARRAY['Evolution'],
--   2,
--   ARRAY['pubkey1_hex', 'pubkey2_hex', 'pubkey3_hex'],
--   0,
--   9999999999999,
--   5,
--   'system'
-- );

