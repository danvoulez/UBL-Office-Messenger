-- ADR-UBL-Registry-002 — Tabelas do Registry multi-tenant

BEGIN;

-- Projects table
CREATE TABLE IF NOT EXISTS registry_projects (
  tenant_id     TEXT NOT NULL,
  project_id    TEXT NOT NULL,
  name          TEXT NOT NULL,
  owners        JSONB NOT NULL,
  visibility    TEXT NOT NULL DEFAULT 'private',
  repo_url      TEXT NOT NULL,
  last_activity BIGINT NOT NULL DEFAULT 0,
  created_at    BIGINT NOT NULL DEFAULT (EXTRACT(EPOCH FROM NOW()) * 1000),
  PRIMARY KEY (tenant_id, project_id)
);

-- Activity log
CREATE TABLE IF NOT EXISTS registry_activity (
  tenant_id  TEXT NOT NULL,
  project_id TEXT NOT NULL,
  ts         BIGINT NOT NULL,
  action     TEXT NOT NULL,           -- push|merge|tag|init
  actor      TEXT NOT NULL,
  ref        TEXT,
  commit     TEXT,
  details    JSONB NOT NULL DEFAULT '{}'::jsonb,
  PRIMARY KEY (tenant_id, project_id, ts)
);

-- Releases
CREATE TABLE IF NOT EXISTS registry_releases (
  tenant_id  TEXT NOT NULL,
  project_id TEXT NOT NULL,
  tag        TEXT NOT NULL,
  commit     TEXT NOT NULL,
  notes_hash TEXT NOT NULL,
  ts         BIGINT NOT NULL,
  manifest   JSONB NOT NULL,
  PRIMARY KEY (tenant_id, project_id, tag)
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_registry_projects_last 
  ON registry_projects(tenant_id, last_activity DESC);

CREATE INDEX IF NOT EXISTS idx_registry_activity_ts   
  ON registry_activity(tenant_id, ts DESC);

CREATE INDEX IF NOT EXISTS idx_registry_releases_ts
  ON registry_releases(tenant_id, ts DESC);

COMMIT;



