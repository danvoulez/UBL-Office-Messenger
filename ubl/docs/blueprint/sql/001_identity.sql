-- 001_identity.sql (reviewed)
CREATE TABLE IF NOT EXISTS id_subject (
  sid uuid PRIMARY KEY,
  kind text CHECK (kind IN ('person','llm','app')),
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS id_webauthn (
  cred_id text PRIMARY KEY,
  sid uuid NOT NULL REFERENCES id_subject(sid),
  public_key bytea NOT NULL,
  sign_count bigint NOT NULL DEFAULT 0,
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS id_challenge (
  challenge_id uuid PRIMARY KEY,
  purpose text CHECK (purpose IN ('register','login','stepup')) NOT NULL,
  data jsonb NOT NULL,
  sid uuid NULL,
  expires_at timestamptz NOT NULL,
  used_at timestamptz NULL
);
CREATE INDEX IF NOT EXISTS idx_id_challenge_exp ON id_challenge(expires_at);
CREATE INDEX IF NOT EXISTS idx_id_challenge_used ON id_challenge(used_at);

CREATE TABLE IF NOT EXISTS id_session (
  token text PRIMARY KEY,
  sid uuid NOT NULL REFERENCES id_subject(sid),
  flavor text NOT NULL CHECK (flavor IN ('regular','stepup')),
  scope jsonb NOT NULL,
  exp_unix bigint NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_id_session_exp ON id_session(exp_unix);

CREATE TABLE IF NOT EXISTS id_api_token (
  jti uuid PRIMARY KEY,
  sid uuid NOT NULL REFERENCES id_subject(sid),
  aud text NOT NULL,
  scope jsonb NOT NULL,
  exp_unix bigint NOT NULL,
  revoked boolean NOT NULL DEFAULT false,
  created_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_id_api_token_exp ON id_api_token(exp_unix);

CREATE TABLE IF NOT EXISTS id_asc (
  asc_id uuid PRIMARY KEY,
  owner_sid uuid NOT NULL REFERENCES id_subject(sid),
  pubkey text NOT NULL,
  scope jsonb NOT NULL,
  valid_from timestamptz NOT NULL,
  valid_to timestamptz NOT NULL,
  revoked boolean NOT NULL DEFAULT false
);
