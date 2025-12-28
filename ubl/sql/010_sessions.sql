-- Session management for WebAuthn
CREATE TABLE IF NOT EXISTS id_session (
  token      text PRIMARY KEY,
  sid        uuid NOT NULL REFERENCES id_subject(sid) ON DELETE CASCADE,
  flavor     text NOT NULL CHECK (flavor IN ('regular','stepup')),
  scope      jsonb NOT NULL DEFAULT '{}',
  exp_unix   bigint NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS ix_id_session_sid ON id_session (sid);
CREATE INDEX IF NOT EXISTS ix_id_session_exp ON id_session (exp_unix);

-- Clean up expired sessions periodically
-- Could be run via cron or background task
CREATE OR REPLACE FUNCTION cleanup_expired_sessions() RETURNS void AS $$
BEGIN
  DELETE FROM id_session WHERE exp_unix < EXTRACT(EPOCH FROM now());
END;
$$ LANGUAGE plpgsql;
