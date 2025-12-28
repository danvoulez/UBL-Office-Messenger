# 🚀 TRINITY RUNBOOK — Local Development

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
cd ubl/kernel/rust/ubl-server

# Copy env
cp .env.example .env

# Run migrations (first time)
sqlx database setup

# Start server
cargo run --release

# Verify
curl http://localhost:8080/health
# → {"status":"ok","version":"2.0.0"}
```

---

## 3️⃣ Start Office

```bash
cd office/office

# Copy env
cp .env.example .env

# Start server
cargo run --release

# Verify
curl http://localhost:8787/health
# → {"status":"ok"}
```

---

## 4️⃣ Start Messenger Backend

```bash
cd ubl-messenger/backend

# Copy env
cp .env.example .env

# Start server
cargo run --release

# Verify
curl http://localhost:4000/health
# → {"status":"ok"}
```

---

## 5️⃣ Start Messenger Frontend

```bash
cd ubl-messenger/frontend

# Install deps (first time)
npm install

# Start dev server
npm run dev

# Open browser
open http://localhost:5173
```

---

## 🧪 Quick Smoke Test

```bash
# Health checks
curl -s localhost:8080/health | jq .  # UBL
curl -s localhost:8787/health | jq .  # Office
curl -s localhost:4000/health | jq .  # Messenger

# All should return {"status":"ok"} or similar
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
| UBL Server | `ubl/kernel/rust/ubl-server/.env` |
| Office | `office/office/.env` |
| Messenger Backend | `ubl-messenger/backend/.env` |
| Messenger Frontend | `ubl-messenger/frontend/.env` |

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
| UBL Server | 8080 |
| Office | 8787 |
| Messenger Backend | 4000 |
| Messenger Frontend | 5173 |



