# UBL Containers

**Universal Business Ledger — Trustworthy Business Records with Cryptographic Proof**

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![PostgreSQL](https://img.shields.io/badge/postgres-16%2B-blue.svg)](https://www.postgresql.org/)
[![License](https://img.shields.io/badge/license-Apache%202.0-green.svg)](LICENSE)

> Every business transaction recorded with mathematical proof. Auditable forever.

---

## 🎯 What is UBL?

UBL is an **append-only ledger** for business operations where every record is:

- **Signed** — Cryptographic proof of authorship (Ed25519)
- **Chained** — Causally linked to previous records (BLAKE3)
- **Immutable** — Cannot be modified or deleted, only appended
- **Auditable** — Any record can be independently verified

### Use Cases

- **Financial transactions** — Transfers, payments, settlements
- **Compliance records** — Audit trails, regulatory filings
- **Multi-party agreements** — Contracts, approvals, signatures
- **AI agent operations** — LLM actions with cryptographic accountability

---

## ⚡ Quick Start

### Prerequisites

- Rust 1.75+ (`rustup`, `cargo`)
- PostgreSQL 16+
- Node.js 20+ (optional, for TypeScript components)

### Clone & Build

```bash
git clone https://github.com/danvoulez/UBL-Containers.git
cd UBL-Containers

# Set up PostgreSQL
createdb ubl_dev
psql ubl_dev -f sql/000_unified.sql

# Build Rust kernel
cd kernel/rust
cargo build --release
```

### Run Server

```bash
export DATABASE_URL=postgres://localhost:5432/ubl_dev
cargo run -p ubl-server
# Server runs on http://localhost:8080
```

### Health Check

```bash
curl http://localhost:8080/health
# {"status":"healthy","version":"2.0.0+postgres"}
```

---

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      UBL Server v2.0                        │
│                   (Axum + PostgreSQL)                       │
└─────────────────────────────────────────────────────────────┘
                             │
           ┌─────────────────┼─────────────────┐
           │                 │                 │
           ▼                 ▼                 ▼
    ┌──────────┐      ┌──────────┐     ┌──────────┐
    │  Ledger  │      │    SSE   │     │    ID    │
    │  (db.rs) │      │ (sse.rs) │     │(id_*.rs) │
    └──────────┘      └──────────┘     └──────────┘
                             │
                      ┌──────────────┐
                      │ PostgreSQL   │
                      │ SERIALIZABLE │
                      └──────────────┘
```

### Layers

| Layer | Technology | Role |
|-------|------------|------|
| **Mind** | TypeScript | Business logic & semantics |
| **Body** | Rust | Validation & cryptography |
| **Link** | HTTP/JSON | Interface between Mind & Body |
| **Storage** | PostgreSQL | SERIALIZABLE append-only |

---

## 📦 Repository Structure

```
UBL-Containers/
├── kernel/rust/              # Core engine (Rust)
│   ├── ubl-atom/            # JSON canonicalization
│   ├── ubl-kernel/          # BLAKE3 + Ed25519 cryptography
│   ├── ubl-link/            # Mind↔Body interface
│   ├── ubl-membrane/        # Physics validation
│   ├── ubl-ledger/          # Append-only data structure
│   ├── ubl-pact/            # Authority & consensus
│   ├── ubl-policy-vm/       # TDLN executor
│   ├── ubl-runner-core/     # Isolated execution
│   └── ubl-server/          # HTTP API + WebAuthn + Identity
├── mind/                    # Semantic orchestration (TypeScript)
├── clients/                 # CLI and SDK
├── specs/                   # Frozen specifications (v1.0)
├── sql/                     # PostgreSQL schemas
├── containers/              # Container definitions
└── docs/                    # Documentation
```

---

## 📡 API Reference

### Core Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/health` | GET | Server health |
| `/state/:container_id` | GET | Container state |
| `/link/validate` | POST | Validate commit |
| `/link/commit` | POST | Append to ledger |
| `/ledger/:container_id/tail` | GET | SSE stream |

### Identity (WebAuthn)

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/id/register/begin` | POST | Start passkey registration |
| `/id/register/finish` | POST | Complete registration |
| `/id/login/begin` | POST | Start passkey login |
| `/id/login/finish` | POST | Complete login |
| `/id/whoami` | GET | Current identity |

### Agents (LLM/App)

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/id/agents` | POST | Create LLM/App agent |
| `/id/agents/:sid/asc` | POST | Issue Agent Signing Certificate |
| `/id/agents/:sid/rotate` | POST | Rotate agent key |

---

## 🔒 Security

- **Cryptography:** Ed25519 signatures, BLAKE3 hashing
- **Database:** SERIALIZABLE isolation, append-only
- **WebAuthn:** Rate limiting, counter rollback detection, HttpOnly cookies
- **Agent Auth:** Ed25519 + Agent Signing Certificates (ASC)

---

## 🧪 Testing

```bash
cd kernel/rust
cargo test --workspace
# ✅ 43+ tests passing
```

---

## 📚 Documentation

- [ARCHITECTURE.md](ARCHITECTURE.md) — System design
- [PHILOSOPHY.md](PHILOSOPHY.md) — Principles & rationale
- [CONTRIBUTING.md](CONTRIBUTING.md) — How to contribute
- [specs/](specs/) — Frozen specifications (v1.0)

---

## 📄 License

Apache 2.0 — See [LICENSE](LICENSE) for details.

---

## 🤝 Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

---

**Made with ❤️ for trustworthy business operations**
