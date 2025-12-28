-- 052_webauthn_stepup.sql
-- Step-Up challenges com binding_hash para Console L4/L5

-- Credenciais WebAuthn (já pode existir em outra tabela, mas aqui fica normalizado)
CREATE TABLE IF NOT EXISTS id_webauthn_credentials (
  cred_id TEXT PRIMARY KEY,              -- base64url(credentialId)
  user_id TEXT NOT NULL,                 -- subject_id (sid)
  public_key_cbor BYTEA NOT NULL,        -- COSE key bytes
  sign_count BIGINT NOT NULL DEFAULT 0,
  transports JSONB NULL,
  created_at_ms BIGINT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_webauthn_user ON id_webauthn_credentials(user_id);

-- Challenges efêmeros de Step-Up COM binding_hash
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

