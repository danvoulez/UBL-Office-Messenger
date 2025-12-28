# LAB 512 - Development Environment

Local development setup for UBL 2.0.

## Quick Start

```bash
# Start PostgreSQL
docker-compose up -d

# Check logs
docker-compose logs -f postgres

# Stop
docker-compose down

# Reset database (destroys all data!)
docker-compose down -v
docker-compose up -d
```

## Connection

```bash
# From host
psql postgres://ubl_dev:dev_password_local_only@localhost:5432/ubl_dev

# From Rust server
DATABASE_URL=postgres://ubl_dev:dev_password_local_only@localhost:5432/ubl_dev cargo run -p ubl-server
```

## Environment Variables

Create `.env` in kernel/rust/:

```bash
DATABASE_URL=postgres://ubl_dev:dev_password_local_only@localhost:5432/ubl_dev
RUST_LOG=ubl_server=debug,info
```

## Migrations

Migrations run automatically on container start from `/sql/` directory:

1. `001_ledger.sql` - Core ledger tables (append-only)
2. `002_idempotency.sql` - Idempotency keys
3. `003_observability.sql` - Metrics and tracing
4. `004_disaster_recovery.sql` - Backup metadata

## Testing

```bash
# Run tests with real database
cd kernel/rust
DATABASE_URL=postgres://ubl_dev:dev_password_local_only@localhost:5432/ubl_dev cargo test

# Run server
DATABASE_URL=postgres://ubl_dev:dev_password_local_only@localhost:5432/ubl_dev cargo run -p ubl-server
```

## Health Check

```bash
# Check if database is ready
docker-compose ps

# Manual connection test
psql postgres://ubl_dev:dev_password_local_only@localhost:5432/ubl_dev -c "SELECT version();"
```

## Cleanup

```bash
# Stop and remove containers
docker-compose down

# Stop and remove all data
docker-compose down -v

# Remove images
docker-compose down --rmi all -v
```

## LAB 512 vs LAB 256

- **LAB 512** (Development): Local Docker, relaxed security, full logs
- **LAB 256** (Production): Cloud/bare metal, strict security, encrypted backups

Tomorrow we'll configure LAB 256 with:
- TLS/SSL encryption
- Connection pooling
- Replication
- Automated backups
- Monitoring
