#!/bin/bash
# Init script for UBL development database
# Runs automatically when container starts

set -e

echo "🔧 Initializing UBL development database..."

# Run migrations in order
psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" <<-EOSQL
    -- Create schema version table
    CREATE TABLE IF NOT EXISTS schema_version (
        version INT PRIMARY KEY,
        applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        description TEXT
    );

    -- Check if migrations already applied
    DO \$\$
    DECLARE
        v_count INT;
    BEGIN
        SELECT COUNT(*) INTO v_count FROM schema_version WHERE version = 1;
        
        IF v_count = 0 THEN
            RAISE NOTICE 'Running migration 001_ledger.sql...';
        END IF;
    END \$\$;
EOSQL

# Run migration files
for sql_file in /sql/00*.sql; do
    if [ -f "$sql_file" ]; then
        echo "📄 Running $(basename $sql_file)..."
        psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" -f "$sql_file"
    fi
done

# Record migrations
psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" <<-EOSQL
    INSERT INTO schema_version (version, description) 
    VALUES 
        (1, '001_ledger.sql - Core ledger tables'),
        (2, '002_idempotency.sql - Idempotency keys'),
        (3, '003_observability.sql - Metrics and tracing'),
        (4, '004_disaster_recovery.sql - Backup metadata')
    ON CONFLICT (version) DO NOTHING;
EOSQL

echo "✅ Database initialization complete!"
echo ""
echo "Connection details:"
echo "  Host: localhost"
echo "  Port: 5432"
echo "  Database: ubl_dev"
echo "  User: ubl_dev"
echo "  Password: dev_password_local_only"
echo ""
echo "Connection string:"
echo "  postgres://ubl_dev:dev_password_local_only@localhost:5432/ubl_dev"
