# 🎯 UBL Flagship Trinity

> **Truth is not what you say. Truth is what you can prove.**

The UBL Flagship Trinity is a complete system for building **verifiable**, **auditable**, and **AI-safe** applications.

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                        UBL FLAGSHIP TRINITY                         │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  ┌─────────────┐   ┌─────────────┐   ┌─────────────────────────┐   │
│  │  MESSENGER  │   │   OFFICE    │   │       UBL LEDGER        │   │
│  │  (Frontend) │   │ (LLM Exec)  │   │       (Kernel)          │   │
│  │             │   │             │   │                         │   │
│  │ • Chat UI   │   │ • Jobs      │   │ • Append-only ledger    │   │
│  │ • Job Cards │   │ • LLM calls │   │ • Containers            │   │
│  │ • WebAuthn  │   │ • Permits   │   │ • Pacts & Policy        │   │
│  └──────┬──────┘   └──────┬──────┘   │ • WebAuthn ID           │   │
│         │                 │          │ • Console API           │   │
│         │    HTTP/WS      │   ASC    │                         │   │
│         └────────────────►│─────────►│                         │   │
│                           │          └─────────────────────────┘   │
│                           │                     │                  │
│                           │              ┌──────┴──────┐           │
│                           │              │  PostgreSQL │           │
│                           │              │ (socket-only)│          │
│                           │              └─────────────┘           │
└─────────────────────────────────────────────────────────────────────┘
```

## 📁 Repository Structure

```
OFFICE-main/
├── ubl/                    # UBL Core (Ledger + Kernel)
│   ├── kernel/rust/        # Rust implementation
│   ├── specs/              # Formal specifications
│   ├── containers/         # Logical container definitions
│   ├── manifests/          # Jobs, policies, offices
│   ├── sql/                # Database migrations
│   ├── infra/              # Infrastructure configs
│   │   └── secpack/        # Security configurations
│   ├── scripts/            # Utility scripts
│   └── clients/            # CLI and SDK
│
├── apps/                   # Applications
│   ├── office/             # Office (LLM Operating System)
│   │   └── src/
│   │       ├── middleware/ # Constitution enforcement
│   │       └── ubl_client/ # HTTP client for UBL
│   └── messenger/          # UBL Messenger (React)
│       └── frontend/
│
├── docs/                   # Documentation
│   ├── WIRING_GUIDE.md     # Integration guide
│   ├── ARCHITECTURE.md     # System architecture
│   ├── adrs/               # Architecture Decision Records
│   └── archive/            # Historical documents
│
└── .github/workflows/      # CI/CD with passkey auth
```

## 🚀 Quick Start

### 1. Start the Kernel

```bash
cd ubl/kernel/rust
cargo build --release
./target/release/ubl-server
```

### 2. Apply Database Migrations

```bash
psql -U ubl_kernel -d ubl_ledger -f ubl/sql/030_console_complete.sql
```

### 3. Start the Messenger

```bash
cd apps/messenger/frontend
npm install
npm run dev
```

### 4. Test the Flow

```bash
./ubl/scripts/test_console_flow.sh http://localhost:8080
```

## 🔐 Security Model

| Layer | Protection |
|-------|------------|
| **Network** | WireGuard mesh, iptables/pf |
| **Database** | Unix socket only, append-only triggers |
| **Auth** | WebAuthn passkeys, ASC tokens |
| **LLM** | Constitution middleware, no DB access |
| **Jobs** | Permits (L0-L5), Pacts for Evolution/Entropy |

### Risk Levels

| Level | Description | Requires |
|-------|-------------|----------|
| L0-L2 | Read/Write | Permit |
| L3 | Sensitive | Permit + Approval |
| L4 | High-risk | Permit + Passkey Step-up |
| L5 | Critical | Permit + Pact (multi-sig) |

## 📚 Documentation

- [Wiring Guide](docs/WIRING_GUIDE.md) - How to connect all components
- [Architecture](docs/ARCHITECTURE.md) - System design
- [Specs](ubl/specs/) - Formal UBL specifications
- [ADRs](docs/adrs/) - Architecture Decision Records
- [Runbook](docs/RUNBOOK.md) - Operations guide

## 🧪 Testing

```bash
# Unit tests
cd ubl/kernel/rust && cargo test

# Console flow test
./ubl/scripts/test_console_flow.sh

# E2E smoke test
./ubl/scripts/e2e_smoke.sh
```

## 📜 License

MIT License - See [LICENSE](LICENSE)

---

**UBL World** — *Where truth is proven, not claimed.*
