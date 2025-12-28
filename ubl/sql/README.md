![sql • * —](https://img.shields.io/badge/sql-*%20—-lightgrey)

# sql — Você está aqui

**Path:** `sql`  
**Role/Cor:** —  
**Zona:** LAB 256 (DB)  

## Credenciais necessárias
- Passkey/keys conforme a operação.


## Função
DDL do ledger/idempotency/observability/DR (Postgres).

## Migrations

| File | Purpose |
|------|---------|
| `000_unified.sql` | Initial unified schema |
| `001_ledger.sql` | Append-only ledger with SSE triggers |
| `002_idempotency.sql` | Idempotency keys |
| `003_observability.sql` | Metrics and logs |
| `004_disaster_recovery.sql` | DR procedures |
| `005_atoms.sql` | **Atom storage for projections** |
| `006_projections.sql` | **Projection tables (jobs, messages, approvals)** |
| `007_pacts.sql` | **Pact registry (authority & consensus)** |
| `010_sessions.sql` | Session management |

## Entradas permitidas (Inbound)
- Inputs da devops/build ou chamadas internas


## Saídas permitidas (Outbound)
- Artefatos para containers ou kernel


## Dados que passam por aqui
- Configs, manifests, scripts

## Dicas
- Nunca versionar segredos. Use placeholders e vars de ambiente.

---
_Navegação:_ [Resumo](../../SUMMARY.md  ) · [Guia](GUIDE.md)