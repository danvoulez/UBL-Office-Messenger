![sql • * —](https://img.shields.io/badge/sql-*%20—-lightgrey)

# sql — Migrações Consolidadas

**Path:** `sql`  
**Role/Cor:** —  
**Zona:** LAB 256 (DB)

## Reset: Squash por Domínio

Migrações foram consolidadas para reduzir ruído e drift:
- **00_base/** - Core (ledger, idempotency, observability, atoms, identity, policy, triggers)
- **10_projections/** - Projeções por domínio (console, messenger, office)
- **90_ops/** - Operações (disaster recovery)
- **99_legacy/** - Arquivos antigos (apenas referência histórica)

## Instalação do Zero

```bash
# Instalar todas as migrações em ordem
make db.install

# Ou manualmente:
psql "$DATABASE_URL" -v ON_ERROR_STOP=1 -f $(cat MIGRATION_ORDER.txt | head -1)
# ... (continue para cada arquivo em MIGRATION_ORDER.txt)
```

## Comandos Make

- `make db.nuke` - DROP + CREATE database (dev only)
- `make db.install` - Run migrations in order from MIGRATION_ORDER.txt
- `make db.verify` - Sanity checks (tabelas, triggers, functions)
- `make db.check` - Check if migrations are valid SQL

## Estrutura

```
sql/
├── 00_base/
│   ├── 000_core.sql          # Extensões, tipos, ledger, idempotency, observability
│   ├── 001_identity.sql      # id_subject, id_credential, id_challenge, id_session (+ step-up)
│   ├── 002_policy.sql        # pacts + policy_engine
│   └── 003_triggers.sql      # NOTIFY tail (payload cid:seq), guards (no UPDATE/DELETE)
├── 10_projections/
│   ├── 100_console.sql       # Console v1.1 (permits, commands, receipts, runners)
│   ├── 101_messenger.sql     # Messenger v1.0 (conversations, messages, jobs, presence)
│   └── 102_office.sql        # Office (entities, sessions, handovers, audit)
├── 90_ops/
│   └── 900_disaster_recovery.sql  # Backup/restore/verify
├── 99_legacy/                # Arquivos antigos (não rodar em instalações novas)
├── MIGRATION_ORDER.txt       # Ordem de execução (fonte da verdade)
└── Makefile                  # Comandos de instalação/verificação
```

## Padrões

- Todos os objetos usam `IF NOT EXISTS` e guards idempotentes
- Triggers usam `DO $$ BEGIN ... IF NOT EXISTS ... END $$;`
- Payload NOTIFY é MINÚSCULO (cid:seq) para evitar limite 8KB
- Tabelas append-only têm triggers de proteção (no UPDATE/DELETE)

## Credenciais necessárias

- Passkey/keys conforme a operação.

## Função

DDL do ledger/idempotency/observability/DR (Postgres).

## Upgrade vs. Fresh Install

- **Fresh install**: Usa apenas arquivos novos (00_base, 10_projections, 90_ops)
- **Banco existente**: 
  1. Rode um bridge leve que cria `ubl_schema_version` e marca `2.0-squashed`
  2. Não re-aplique 99_legacy
  3. Use apenas patches incrementais depois do squash (ex.: 91_patches/2_0_1.sql)

## Done if...

- `make db.install` roda limpo em banco vazio
- `\dt` mostra apenas as tabelas finais (sem sobras duplicadas)
- LISTEN/NOTIFY funcionando (payload `container_id:sequence`)
- Projeções aparecem com dados após um commit no ledger
- `verify_ledger_integrity()` retorna válido

---

_Navegação:_ [Resumo](../../SUMMARY.md) · [Guia](GUIDE.md)
