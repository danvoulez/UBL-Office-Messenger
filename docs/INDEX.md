# 📚 UBL Documentation Index

> Complete navigation for the UBL Flagship Trinity documentation.

## 🎯 Start Here

| Document | Description |
|----------|-------------|
| [README](../README.md) | Project overview and quick start |
| [WIRING_GUIDE](WIRING_GUIDE.md) | How to connect Messenger → Office → Kernel |
| [ARCHITECTURE](ARCHITECTURE.md) | System design and component overview |
| [RUNBOOK](RUNBOOK.md) | Operations and troubleshooting |

## 📐 Specifications

All formal specifications are in `ubl/specs/`:

| Spec | Description | Status |
|------|-------------|--------|
| [SPEC-UBL-CORE](../ubl/specs/ubl-core/SPEC-UBL-CORE.md) | Core ontology, containers, physics | v1.0 ✅ |
| [SPEC-UBL-ATOM](../ubl/specs/ubl-atom/SPEC-UBL-ATOM.md) | Canonical data format | v1.0 ✅ |
| [SPEC-UBL-LINK](../ubl/specs/ubl-link/SPEC-UBL-LINK.md) | Commit envelope structure | v1.0 ✅ |
| [SPEC-UBL-PACT](../ubl/specs/ubl-pact/SPEC-UBL-PACT.md) | Multi-signature governance | v1.0 ✅ |
| [SPEC-UBL-POLICY](../ubl/specs/ubl-policy/SPEC-UBL-POLICY.md) | TDLN translation rules | v1.0 ✅ |
| [SPEC-UBL-MEMBRANE](../ubl/specs/ubl-membrane/SPEC-UBL-MEMBRANE.md) | Physical validation layer | v1.0 ✅ |
| [SPEC-UBL-LEDGER](../ubl/specs/ubl-ledger/SPEC-UBL-LEDGER.md) | Append-only history | v1.0 ✅ |
| [SPEC-UBL-RUNNER](../ubl/specs/ubl-runner/SPEC-UBL-RUNNER.md) | Isolated job execution | v1.0 ✅ |
| [SPEC-UBL-LLM](../ubl/specs/ubl-llm/SPEC-UBL-LLM.md) | LLM access layer | v1.0 ✅ |

## 🏛️ Architecture Decision Records (ADRs)

| ADR | Title |
|-----|-------|
| [ADR-001](adrs/ADR-UBL-Console-001.v1.md) | Console API v1.1 |
| [ADR-002](adrs/ADR-UBL-Registry-002.md) | Office Git Registry |

## 🔧 Infrastructure

| Document | Location |
|----------|----------|
| [SecPack README](../ubl/infra/secpack/README.md) | Security configurations |
| [Lab 256 Setup](../ubl/infra/lab-256/) | Gateway zone |
| [Lab 512 Setup](../ubl/infra/lab-512/README.md) | Sandbox zone |
| [PostgreSQL](../ubl/infra/postgres/) | Database scripts |
| [WireGuard](../ubl/infra/wireguard/) | VPN configuration |
| [MinIO](../ubl/infra/minio/) | Object storage |

## 📦 Manifests

Configuration files in `ubl/manifests/`:

| Manifest | Description |
|----------|-------------|
| [containers.json](../ubl/manifests/containers.json) | Container definitions |
| [jobs.allowlist.yaml](../ubl/manifests/jobs.allowlist.yaml) | Job type permissions |
| [offices.yaml](../ubl/manifests/offices.yaml) | Office registry |
| [policies.json](../ubl/manifests/policies.json) | Policy definitions |
| [routes.json](../ubl/manifests/routes.json) | API routing |
| [asc.schema.json](../ubl/manifests/policy/asc.schema.json) | ASC token schema |

## 🗄️ Database

SQL migrations in `ubl/sql/`:

| Directory | Description |
|-----------|-------------|
| `00_base/000_core.sql` | Core ledger tables |
| `00_base/001_identity.sql` | Identity tables |
| `00_base/002_policy.sql` | Policy tables |
| `00_base/003_triggers.sql` | NOTIFY triggers |
| `10_projections/100_console.sql` | Console projections |
| `10_projections/101_messenger.sql` | Messenger projections |
| `10_projections/102_office.sql` | Office projections |

See `ubl/sql/MIGRATION_ORDER.txt` for apply order.

## 🧪 Scripts

Utility scripts in `ubl/scripts/`:

| Script | Description |
|--------|-------------|
| `test_console_flow.sh` | E2E console flow test |
| `e2e_smoke.sh` | Full smoke test |
| `verify_ledger.sh` | Ledger integrity check |

## 📱 Applications

| App | Location | Description |
|-----|----------|-------------|
| Messenger | `apps/messenger/frontend/` | React chat UI |
| CLI | `ubl/clients/cli/` | Command-line interface |
| SDK | `ubl/clients/ts/sdk/` | TypeScript SDK |

## 🗃️ Archive

Historical documents (for reference only):

| Category | Location |
|----------|----------|
| Audits | `docs/archive/audits/` |
| Status Reports | `docs/archive/status/` |
| Original Prompts | `docs/archive/prompts/` |

---

## 🗺️ Quick Navigation

```
docs/
├── INDEX.md            ← You are here
├── WIRING_GUIDE.md     ← Start here for integration
├── ARCHITECTURE.md     ← System design
├── ROADMAP.md          ← Future plans
├── RUNBOOK.md          ← Operations
├── STATUS.md           ← Current status
├── adrs/               ← Architecture decisions
└── archive/            ← Historical docs
```

---

*Last updated: 2025-12-30*

