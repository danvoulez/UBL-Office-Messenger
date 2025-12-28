-- Policy Engine Storage
-- SPEC-UBL-POLICY v1.0
--
-- Tables for storing policy definitions and container mappings

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

-- Create index for querying by version
CREATE INDEX IF NOT EXISTS ix_policy_definitions_version 
    ON policy_definitions (policy_id, version);

-- Container to policy mappings
CREATE TABLE IF NOT EXISTS container_policies (
    container_id TEXT PRIMARY KEY,
    policy_id TEXT NOT NULL REFERENCES policy_definitions(policy_id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create index for reverse lookup (which containers use a policy)
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

-- Create indexes for querying audit log
CREATE INDEX IF NOT EXISTS ix_policy_evaluations_container 
    ON policy_evaluations (container_id, created_at DESC);
CREATE INDEX IF NOT EXISTS ix_policy_evaluations_policy 
    ON policy_evaluations (policy_id, created_at DESC);
CREATE INDEX IF NOT EXISTS ix_policy_evaluations_decision 
    ON policy_evaluations (decision, created_at DESC);

-- Function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Triggers for updated_at
DROP TRIGGER IF EXISTS policy_definitions_updated_at ON policy_definitions;
CREATE TRIGGER policy_definitions_updated_at
    BEFORE UPDATE ON policy_definitions
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS container_policies_updated_at ON container_policies;
CREATE TRIGGER container_policies_updated_at
    BEFORE UPDATE ON container_policies
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Insert default policies for known containers
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

-- Comment on tables
COMMENT ON TABLE policy_definitions IS 'SPEC-UBL-POLICY v1.0: Policy definitions with compiled rules';
COMMENT ON TABLE container_policies IS 'SPEC-UBL-POLICY v1.0: Container to policy mappings';
COMMENT ON TABLE policy_evaluations IS 'SPEC-UBL-POLICY v1.0: Audit log of policy evaluations';



