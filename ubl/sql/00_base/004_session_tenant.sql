-- ============================================================================
-- Add tenant_id to id_session for Zona Schengen
-- ============================================================================
-- This enables session-level tenant context, so authenticated users
-- can perform actions within their tenant without re-authentication.

-- Add tenant_id column to id_session
ALTER TABLE id_session 
ADD COLUMN IF NOT EXISTS tenant_id TEXT REFERENCES id_tenant(tenant_id);

-- Add default_tenant_id column to id_subject
ALTER TABLE id_subject
ADD COLUMN IF NOT EXISTS default_tenant_id TEXT REFERENCES id_tenant(tenant_id);

-- Index for efficient session queries by tenant
CREATE INDEX IF NOT EXISTS ix_id_session_tenant ON id_session (tenant_id);

COMMENT ON COLUMN id_session.tenant_id IS 'Tenant context for Zona Schengen - no re-auth needed within tenant';
COMMENT ON COLUMN id_subject.default_tenant_id IS 'Default tenant for new sessions';
