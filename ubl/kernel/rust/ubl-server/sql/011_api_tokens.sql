-- Minimal table to log issued tokens (optional auditing)
CREATE TABLE IF NOT EXISTS id_api_token (
  jti TEXT PRIMARY KEY,
  sid TEXT NOT NULL REFERENCES id_session(sid) ON DELETE CASCADE,
  aud TEXT NOT NULL,
  scope JSONB NOT NULL,
  flavor TEXT NOT NULL,
  issued_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  expires_at TIMESTAMPTZ NOT NULL
);
