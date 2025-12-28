-- UBL ID Unified Schema (PR28 + WebAuthn sessions)
-- Drop and recreate for dev

-- Subjects (people, llm, app)
CREATE TABLE IF NOT EXISTS id_subject (
  sid           text PRIMARY KEY,
  kind          text NOT NULL CHECK (kind IN ('person','llm','app')),
  display_name  text NOT NULL,
  status        text NOT NULL DEFAULT 'active',
  created_at    timestamptz NOT NULL DEFAULT now()
);

-- Credentials (passkey, ed25519, mtls)
CREATE TABLE IF NOT EXISTS id_credential (
  id            uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  sid           text NOT NULL REFERENCES id_subject(sid) ON DELETE CASCADE,
  credential_kind text NOT NULL CHECK (credential_kind IN ('passkey','ed25519','mtls')),
  credential_id text,
  public_key    bytea NOT NULL,
  sign_count    bigint DEFAULT 0,
  backup_eligible boolean,
  backup_state boolean,
  transports    text[],
  key_version   integer NOT NULL DEFAULT 1,
  created_at    timestamptz NOT NULL DEFAULT now(),
  UNIQUE (sid, credential_kind, key_version)
);

-- Challenges (register/login/stepup)
CREATE TABLE IF NOT EXISTS id_challenge (
  id           uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  kind         text NOT NULL CHECK (kind IN ('register','login','stepup')),
  sid          text REFERENCES id_subject(sid) ON DELETE SET NULL,
  challenge    bytea NOT NULL,
  origin       text NOT NULL,
  expires_at   timestamptz NOT NULL,
  used         boolean NOT NULL DEFAULT false
);

-- Sessions (unified: token TEXT PK for webauthn, session_id UUID for lookup)
CREATE TABLE IF NOT EXISTS id_session (
  token         text PRIMARY KEY,
  sid           text NOT NULL REFERENCES id_subject(sid) ON DELETE CASCADE,
  session_id    uuid DEFAULT gen_random_uuid(),
  flavor        text NOT NULL CHECK (flavor IN ('regular','stepup','user','ict','webauthn')),
  scope         jsonb NOT NULL DEFAULT '{}'::jsonb,
  exp_unix      bigint,
  not_before    timestamptz DEFAULT now(),
  not_after     timestamptz,
  created_at    timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS ix_id_session_sid ON id_session (sid);
CREATE INDEX IF NOT EXISTS ix_id_session_exp ON id_session (exp_unix);
CREATE INDEX IF NOT EXISTS ix_id_session_uuid ON id_session (session_id);

-- Agent Signing Certificates (ASC)
CREATE TABLE IF NOT EXISTS id_asc (
  asc_id       uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  sid          text NOT NULL REFERENCES id_subject(sid) ON DELETE CASCADE,
  public_key   bytea NOT NULL,
  scopes       jsonb NOT NULL,
  not_before   timestamptz NOT NULL,
  not_after    timestamptz NOT NULL,
  signature    bytea NOT NULL,
  created_at   timestamptz NOT NULL DEFAULT now()
);

-- Key revocations
CREATE TABLE IF NOT EXISTS id_key_revocation (
  sid          text NOT NULL REFERENCES id_subject(sid) ON DELETE CASCADE,
  key_version  integer NOT NULL,
  revoked_at   timestamptz NOT NULL DEFAULT now(),
  PRIMARY KEY (sid, key_version)
);

-- Ledger
CREATE TABLE IF NOT EXISTS ledger_entry (
  id            bigserial PRIMARY KEY,
  container_id  text NOT NULL,
  sequence      bigint NOT NULL,
  link_hash     text NOT NULL,
  previous_hash text NOT NULL,
  entry_hash    text NOT NULL,
  ts_unix_ms    bigint NOT NULL,
  metadata      jsonb NOT NULL DEFAULT '{}',
  intent_class  text,
  physics_delta jsonb,
  created_at    timestamptz NOT NULL DEFAULT now(),
  UNIQUE (container_id, sequence)
);
CREATE INDEX IF NOT EXISTS ix_ledger_container ON ledger_entry (container_id);

-- Notify trigger for SSE
CREATE OR REPLACE FUNCTION notify_ledger_change() RETURNS trigger AS $$
BEGIN
  PERFORM pg_notify('ledger_' || NEW.container_id, row_to_json(NEW)::text);
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS ledger_notify ON ledger_entry;
CREATE TRIGGER ledger_notify AFTER INSERT ON ledger_entry
FOR EACH ROW EXECUTE FUNCTION notify_ledger_change();
