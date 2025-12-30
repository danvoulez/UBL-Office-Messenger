# 📚 UBL 3.0 Documentation

> Complete navigation for UBL 3.0 documentation.

---

## 🎯 Start Here

| Document | Description |
|----------|-------------|
| [README](../README.md) | Project overview and quick start |
| [ARCHITECTURE](ARCHITECTURE.md) | System design (Messenger + Office + UBL Kernel) |
| [THREE_SYSTEMS_OVERVIEW](THREE_SYSTEMS_OVERVIEW.md) | Deep-dive into all three systems |
| [WIRING_GUIDE](WIRING_GUIDE.md) | How to connect Messenger → Office → Kernel |

## 🔐 Security & Authorization

| Document | Description |
|----------|-------------|
| [SPEC_UBL_SCHENGEN](SPEC_UBL_SCHENGEN.md) | Zona Schengen - Authorization cascade |
| [SCHENGEN_IMPLEMENTATION_CHECKLIST](SCHENGEN_IMPLEMENTATION_CHECKLIST.md) | Implementation progress |
| [WHY_SO_COMPLEX](WHY_SO_COMPLEX.md) | Why the system is designed this way |

## 🏢 Multi-Tenancy (C.Tenant)

| Document | Description |
|----------|-------------|
| [C_TENANT](C_TENANT.md) | Design + implementation |

## 📊 Status & Progress

| Document | Description |
|----------|-------------|
| [STATUS](STATUS.md) | Current implementation status |
| [ROADMAP](ROADMAP.md) | Future plans |
| [RUNBOOK](RUNBOOK.md) | Operations and troubleshooting |

---

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
| [ADR-003](adrs/ADR-003-Three-Systems.md) | Three Independent Systems |
| [ADR-004](adrs/ADR-004-Zona-Schengen.md) | Zona Schengen (Authorization) |
| [ADR-005](adrs/ADR-005-Append-Only-Ledger.md) | Append-Only Ledger |

---

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
| `00_base/002_tenant.sql` | Multi-tenancy tables |
| `00_base/003_policy.sql` | Policy tables |
| `00_base/004_session_tenant.sql` | Session context |
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
| `run_golden_test.sh` | UBL 3.0 integration tests |

## 📱 Applications

| App | Location | Description |
|-----|----------|-------------|
| Messenger | `apps/messenger/frontend/` | React chat UI |
| Office | `apps/office/` | LLM Operating System |
| CLI | `ubl/clients/cli/` | Command-line interface |
| SDK | `ubl/clients/ts/sdk/` | TypeScript SDK |

---

## 🗃️ Archive

Historical documents (for reference only):

| Category | Location |
|----------|----------|
| Status Reports | `docs/archive/status/` |
| Session Logs | `docs/archive/sessions/` |
| Original Prompts | `docs/archive/prompts/` |

---

## 🗺️ Directory Structure

```
docs/
├── INDEX.md                  ← You are here
├── ARCHITECTURE.md           ← System design
├── THREE_SYSTEMS_OVERVIEW.md ← Deep-dive all systems  
├── WIRING_GUIDE.md           ← Integration guide
│
├── SPEC_UBL_SCHENGEN.md      ← Authorization spec
├── SCHENGEN_...CHECKLIST.md  ← Implementation checklist
├── WHY_SO_COMPLEX.md         ← System philosophy
│
├── C_TENANT.md               ← Multi-tenancy
│
├── STATUS.md                 ← Current status
├── ROADMAP.md                ← Future plans
├── RUNBOOK.md                ← Operations
│
├── adrs/                     ← Architecture decisions
├── devops/                   ← DevOps guides
└── archive/                  ← Historical docs
```

---

*Last updated: 2025-12-30*
