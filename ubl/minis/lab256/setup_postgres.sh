#!/usr/bin/env bash
set -euo pipefail
brew list postgresql@16 >/dev/null 2>&1 || brew install postgresql@16
brew services start postgresql@16
# init db (idempotente)
psql postgres -tc "SELECT 1 FROM pg_database WHERE datname='ubl_dev'" | grep -q 1 || createdb ubl_dev
psql -d ubl_dev -tc "CREATE EXTENSION IF NOT EXISTS pgcrypto;"
echo "✅ Postgres pronto (ubl_dev)"
