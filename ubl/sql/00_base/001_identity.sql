-- ============================================================================
-- UBL Identity Schema - Subjects, Credentials, Sessions, ASC
-- ============================================================================
-- Consolidated from: 000_unified.sql (identity) + 010_sessions.sql + 052_webauthn_stepup.sql
-- Reset: Squash por domínio - instalação do zero

-- ============================================================================
-- SUBJECTS (people, llm, app)
-- ============================================================================

CREATE TABLE IF NOT EXISTS id_subject (
  sid           TEXT PRIMARY KEY,
  kind          TEXT NOT NULL CHECK (kind IN ('person','llm','app')),
  display_name  TEXT NOT NULL,
  status        TEXT NOT NULL DEFAULT 'active',
  created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

COMMENT ON TABLE id_subject IS 'Identity subjects (people, LLMs, apps)';

-- ============================================================================
-- CREDENTIALS (passkey, ed25519, mtls)
-- ============================================================================

CREATE TABLE IF NOT EXISTS id_credential (
  id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  sid           TEXT NOT NULL REFERENCES id_subject(sid) ON DELETE CASCADE,
  credential_kind TEXT NOT NULL CHECK (credential_kind IN ('passkey','ed25519','mtls')),
  credential_id TEXT,
  public_key    BYTEA NOT NULL,
  sign_count    BIGINT DEFAULT 0,
  backup_eligible BOOLEAN,
  backup_state  BOOLEAN,
  transports    TEXT[],
  key_version   INTEGER NOT NULL DEFAULT 1,
  created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  UNIQUE (sid, credential_kind, key_version)
);

CREATE INDEX IF NOT EXISTS ix_id_credential_sid ON id_credential (sid);

COMMENT ON TABLE id_credential IS 'Identity credentials (passkey, ed25519, mtls)';

-- ============================================================================
-- WEBAUTHN CREDENTIALS
-- ============================================================================

CREATE TABLE IF NOT EXISTS id_webauthn_credentials (
  cred_id TEXT PRIMARY KEY,              -- base64url(credentialId)
  user_id TEXT NOT NULL,                 -- subject_id (sid)
  public_key_cbor BYTEA NOT NULL,        -- COSE key bytes
  sign_count BIGINT NOT NULL DEFAULT 0,
  transports JSONB NULL,
  created_at_ms BIGINT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_webauthn_user ON id_webauthn_credentials(user_id);

COMMENT ON TABLE id_webauthn_credentials IS 'WebAuthn credentials (normalized)';

-- ============================================================================
-- CHALLENGES (register/login/stepup)
-- ============================================================================

CREATE TABLE IF NOT EXISTS id_challenge (
  id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  kind         TEXT NOT NULL CHECK (kind IN ('register','login','stepup')),
  sid          TEXT REFERENCES id_subject(sid) ON DELETE SET NULL,
  challenge    BYTEA NOT NULL,
  origin       TEXT NOT NULL,
  expires_at   TIMESTAMPTZ NOT NULL,
  used         BOOLEAN NOT NULL DEFAULT false
);

CREATE INDEX IF NOT EXISTS ix_id_challenge_kind ON id_challenge (kind, expires_at);
CREATE INDEX IF NOT EXISTS ix_id_challenge_sid ON id_challenge (sid);

COMMENT ON TABLE id_challenge IS 'Identity challenges (register/login/stepup)';

-- ============================================================================
-- STEP-UP CHALLENGES (with binding_hash)
-- ============================================================================

CREATE TABLE IF NOT EXISTS id_stepup_challenges (
  challenge_id TEXT PRIMARY KEY,         -- uuid
  user_id TEXT NOT NULL,
  binding_hash TEXT NOT NULL,            -- blake3:... (EXATAMENTE o binding_hash do Permit)
  challenge_b64 TEXT NOT NULL,           -- base64url(challenge bytes)
  auth_state BYTEA NOT NULL,             -- serialized PasskeyAuthentication state
  created_at_ms BIGINT NOT NULL,
  exp_ms BIGINT NOT NULL,
  used BOOLEAN NOT NULL DEFAULT false
);

CREATE INDEX IF NOT EXISTS idx_stepup_binding ON id_stepup_challenges(binding_hash);
CREATE INDEX IF NOT EXISTS idx_stepup_exp ON id_stepup_challenges(exp_ms);
CREATE INDEX IF NOT EXISTS idx_stepup_challenge ON id_stepup_challenges(challenge_b64);

COMMENT ON TABLE id_stepup_challenges IS 'Step-Up challenges com binding_hash para Console L4/L5';

-- ============================================================================
-- SESSIONS (unified: token TEXT PK, session_id UUID for lookup)
-- ============================================================================

CREATE TABLE IF NOT EXISTS id_session (
  token         TEXT PRIMARY KEY,
  sid           TEXT NOT NULL REFERENCES id_subject(sid) ON DELETE CASCADE,
  session_id    UUID DEFAULT gen_random_uuid(),
  flavor        TEXT NOT NULL CHECK (flavor IN ('regular','stepup','user','ict','webauthn')),
  scope         JSONB NOT NULL DEFAULT '{}'::jsonb,
  exp_unix      BIGINT,
  not_before    TIMESTAMPTZ DEFAULT NOW(),
  not_after     TIMESTAMPTZ,
  created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS ix_id_session_sid ON id_session (sid);
CREATE INDEX IF NOT EXISTS ix_id_session_exp ON id_session (exp_unix);
CREATE INDEX IF NOT EXISTS ix_id_session_uuid ON id_session (session_id);

COMMENT ON TABLE id_session IS 'Identity sessions (unified: token TEXT PK, session_id UUID)';

-- ============================================================================
-- AGENT SIGNING CERTIFICATES (ASC)
-- ============================================================================

CREATE TABLE IF NOT EXISTS id_asc (
  asc_id       UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  sid          TEXT NOT NULL REFERENCES id_subject(sid) ON DELETE CASCADE,
  public_key   BYTEA NOT NULL,
  scopes       JSONB NOT NULL,
  not_before   TIMESTAMPTZ NOT NULL,
  not_after    TIMESTAMPTZ NOT NULL,
  signature    BYTEA NOT NULL,
  created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS ix_id_asc_sid ON id_asc (sid, not_before, not_after);

COMMENT ON TABLE id_asc IS 'Agent Signing Certificates (ASC) for LLM/App agents';

-- ============================================================================
-- KEY REVOCATIONS
-- ============================================================================

CREATE TABLE IF NOT EXISTS id_key_revocation (
  sid          TEXT NOT NULL REFERENCES id_subject(sid) ON DELETE CASCADE,
  key_version  INTEGER NOT NULL,
  revoked_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  PRIMARY KEY (sid, key_version)
);

COMMENT ON TABLE id_key_revocation IS 'Key revocations for credentials';

-- ============================================================================
-- CLEANUP FUNCTIONS
-- ============================================================================

CREATE OR REPLACE FUNCTION cleanup_expired_sessions() RETURNS void AS $$
BEGIN
  DELETE FROM id_session WHERE exp_unix < EXTRACT(EPOCH FROM NOW());
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION cleanup_expired_sessions IS 'Clean up expired sessions (run via cron or background task)';


