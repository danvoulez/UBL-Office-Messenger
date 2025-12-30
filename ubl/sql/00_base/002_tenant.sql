-- ============================================================================
-- UBL Tenant Schema - Multi-tenancy Support
-- ============================================================================
-- New file: 002_tenant.sql
-- Adds: id_tenant, id_tenant_member, id_invite_code

-- ============================================================================
-- TENANTS (organizations)
-- ============================================================================

CREATE TABLE IF NOT EXISTS id_tenant (
  tenant_id     TEXT PRIMARY KEY,
  name          TEXT NOT NULL,
  slug          TEXT UNIQUE NOT NULL,
  status        TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active','suspended','deleted')),
  settings      JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_by    TEXT NOT NULL,  -- sid of creator
  created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS ix_id_tenant_slug ON id_tenant (slug);
CREATE INDEX IF NOT EXISTS ix_id_tenant_status ON id_tenant (status);

COMMENT ON TABLE id_tenant IS 'Organizations/Tenants for multi-tenancy';

-- ============================================================================
-- TENANT MEMBERS (user ↔ tenant relationship)
-- ============================================================================

CREATE TABLE IF NOT EXISTS id_tenant_member (
  tenant_id     TEXT NOT NULL REFERENCES id_tenant(tenant_id) ON DELETE CASCADE,
  sid           TEXT NOT NULL REFERENCES id_subject(sid) ON DELETE CASCADE,
  role          TEXT NOT NULL DEFAULT 'member' CHECK (role IN ('owner','admin','member')),
  joined_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  PRIMARY KEY (tenant_id, sid)
);

CREATE INDEX IF NOT EXISTS ix_id_tenant_member_sid ON id_tenant_member (sid);
CREATE INDEX IF NOT EXISTS ix_id_tenant_member_role ON id_tenant_member (tenant_id, role);

COMMENT ON TABLE id_tenant_member IS 'Tenant membership (many-to-many with roles)';

-- ============================================================================
-- INVITE CODES
-- ============================================================================

CREATE TABLE IF NOT EXISTS id_invite_code (
  code          TEXT PRIMARY KEY,  -- format: XXXX-XXXX
  tenant_id     TEXT NOT NULL REFERENCES id_tenant(tenant_id) ON DELETE CASCADE,
  created_by    TEXT NOT NULL,     -- sid of creator
  expires_at    TIMESTAMPTZ NOT NULL,
  max_uses      INT NOT NULL DEFAULT 1,
  uses          INT NOT NULL DEFAULT 0,
  status        TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active','expired','revoked')),
  created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS ix_id_invite_code_tenant ON id_invite_code (tenant_id);
CREATE INDEX IF NOT EXISTS ix_id_invite_code_status ON id_invite_code (status, expires_at);

COMMENT ON TABLE id_invite_code IS 'Invite codes for joining tenants';

-- ============================================================================
-- UPDATE id_subject TO INCLUDE default_tenant_id
-- ============================================================================
-- This allows quick lookup of user's primary tenant

DO $$
BEGIN
  IF NOT EXISTS (
    SELECT 1 FROM information_schema.columns 
    WHERE table_name = 'id_subject' AND column_name = 'default_tenant_id'
  ) THEN
    ALTER TABLE id_subject ADD COLUMN default_tenant_id TEXT REFERENCES id_tenant(tenant_id) ON DELETE SET NULL;
    CREATE INDEX IF NOT EXISTS ix_id_subject_tenant ON id_subject (default_tenant_id);
  END IF;
END $$;

-- ============================================================================
-- HELPER FUNCTIONS
-- ============================================================================

-- Generate invite code in format XXXX-XXXX
CREATE OR REPLACE FUNCTION generate_invite_code() RETURNS TEXT AS $$
DECLARE
  chars TEXT := 'ABCDEFGHJKLMNPQRSTUVWXYZ23456789';
  result TEXT := '';
  i INT;
BEGIN
  FOR i IN 1..8 LOOP
    IF i = 5 THEN
      result := result || '-';
    END IF;
    result := result || substr(chars, floor(random() * length(chars) + 1)::int, 1);
  END LOOP;
  RETURN result;
END;
$$ LANGUAGE plpgsql;

-- Check if invite code is valid
CREATE OR REPLACE FUNCTION is_invite_valid(p_code TEXT) RETURNS BOOLEAN AS $$
BEGIN
  RETURN EXISTS (
    SELECT 1 FROM id_invite_code 
    WHERE code = p_code 
      AND status = 'active'
      AND expires_at > NOW()
      AND uses < max_uses
  );
END;
$$ LANGUAGE plpgsql;

-- Use invite code (increment uses, return tenant_id)
CREATE OR REPLACE FUNCTION use_invite_code(p_code TEXT) RETURNS TEXT AS $$
DECLARE
  v_tenant_id TEXT;
BEGIN
  UPDATE id_invite_code 
  SET uses = uses + 1
  WHERE code = p_code 
    AND status = 'active'
    AND expires_at > NOW()
    AND uses < max_uses
  RETURNING tenant_id INTO v_tenant_id;
  
  RETURN v_tenant_id;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION generate_invite_code IS 'Generate random invite code XXXX-XXXX';
COMMENT ON FUNCTION is_invite_valid IS 'Check if invite code is valid and usable';
COMMENT ON FUNCTION use_invite_code IS 'Use invite code and return tenant_id';
