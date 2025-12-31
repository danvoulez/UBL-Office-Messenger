# 🚀 UBL 3.0 RUNBOOK — Local Development

**One screen. Copy/paste. Done.**

---

## Prerequisites

```bash
# macOS
xcode-select --install          # Xcode CLI tools
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh  # Rust
brew install postgresql@15      # Postgres
brew install node               # Node.js (for frontend)
```

---

## 1️⃣ Start Postgres

```bash
# Start Postgres (brew)
brew services start postgresql@15

# Create database
createdb ubl_ledger

# Verify
psql -d ubl_ledger -c "SELECT 1"
```

---

## 2️⃣ Start UBL Server

```bash
cd ubl/kernel/rust

# Apply migrations (first time)
psql -d ubl_ledger -f ../../../ubl/sql/00_base/000_core.sql
psql -d ubl_ledger -f ../../../ubl/sql/00_base/001_identity.sql
psql -d ubl_ledger -f ../../../ubl/sql/00_base/002_policy.sql
psql -d ubl_ledger -f ../../../ubl/sql/00_base/003_triggers.sql
psql -d ubl_ledger -f ../../../ubl/sql/10_projections/100_console.sql
psql -d ubl_ledger -f ../../../ubl/sql/10_projections/101_messenger.sql
psql -d ubl_ledger -f ../../../ubl/sql/10_projections/102_office.sql

# Start server
DATABASE_URL="postgres://user@localhost/ubl_ledger" cargo run --bin ubl-server

# Verify
curl http://localhost:8080/health
# → {"status":"ok"}
```

---

## 3️⃣ Start Office

```bash
cd apps/office

# Start server
cargo run

# Verify
curl http://localhost:8081/health
# → {"status":"ok"}
```

---

## 4️⃣ Start Messenger Frontend

```bash
cd apps/messenger/frontend

# Install deps (first time)
npm install

# Start dev server
npm run dev

# Open browser
open http://localhost:3000
```

---

## 5️⃣ Verify All Services

```bash
# Health checks
curl -s localhost:8080/health | jq .  # UBL Kernel
curl -s localhost:8081/health | jq .  # Office
curl -s localhost:3000                 # Messenger Frontend

# All should return {"status":"ok"} or HTML
```

---

## 🧪 Quick Smoke Test

```bash
# WebAuthn register test
curl -s -X POST http://localhost:8080/id/register/begin \
  -H "Content-Type: application/json" \
  -d '{"username":"test"}' | jq .

# Should return WebAuthn options
```

---

## 🐳 Docker (Alternative)

```bash
# From project root
docker compose -f docker-compose.stack.yml up

# All services start automatically
```

---

## 📁 Environment Files

| Service | File |
|---------|------|
| UBL Server | Environment variables (DATABASE_URL, WEBAUTHN_ORIGIN) |
| Office | `apps/office/config/development.toml` |
| Messenger Frontend | `apps/messenger/frontend/.env` (optional) |

---

## 🔧 Troubleshooting

### Postgres won't start
```bash
brew services restart postgresql@15
tail -f /usr/local/var/log/postgresql@15.log
```

### Port already in use
```bash
lsof -i :8080  # Find process
kill -9 <PID>  # Kill it
```

### Rust compilation fails
```bash
rustup update
cargo clean
cargo build
```

### Missing Xcode tools
```bash
xcode-select --install
# Click Install in dialog
```

---

## 🌐 Default Ports

| Service | Port |
|---------|------|
| Postgres | 5432 |
| UBL Kernel | 8080 |
| Office | 8081 |
| Messenger Frontend | 3000 |






