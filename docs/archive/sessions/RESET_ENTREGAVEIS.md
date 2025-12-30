# Reset de Foco - Entregáveis Implementados

## ✅ Status: COMPLETO

### A) /metrics que funciona (Prometheus text)
- ✅ Implementado em `ubl/kernel/rust/ubl-server/src/metrics.rs`
- ✅ Endpoint: `GET /metrics`
- ✅ Retorna texto Prometheus simples com `ubl_up` e `ubl_build_info`
- **Prova:** `curl -s http://localhost:8080/metrics | head`

### B) SSE "só ID" (cid:seq)
- ✅ Implementado em `ubl/kernel/rust/ubl-server/src/sse.rs`
- ✅ Endpoint: `GET /ledger/tail`
- ✅ Emite apenas `container_id:sequence` (ex: "repo://tenant/ws:42")
- ✅ Trigger Postgres criado em `migrations/999_ubl_tail_notify.sql`
- ✅ Payload MINÚSCULO para evitar limite 8KB do PostgreSQL NOTIFY
- **Prova:** `curl -N http://localhost:8080/ledger/tail | head`

### C) DSN por Unix socket (sem quebrar TCP)
- ✅ Suportado via `DATABASE_URL`:
  - Unix: `postgres:///ubl_dev?host=/var/run/postgresql`
  - TCP: `postgres://ubl:***@127.0.0.1:5432/ubl`
- ✅ `sqlx::PgPoolOptions` respeita o DSN automaticamente
- **Prova:** `psql "postgres:///ubl_dev?host=/var/run/postgresql" -c '\dt'`

### D) JWT "ASC-light" para CLI/LLM
- ✅ Já existe em `ubl/kernel/rust/ubl-server/src/id_session_token.rs`
- ✅ Endpoint: `POST /id/session/token`
- ✅ Suporta step-up para escopo "admin"
- ✅ Retorna Bearer token com Ed25519
- **Prova:** 
  ```bash
  curl -s -X POST http://localhost:8080/id/session/token \
    -H 'content-type: application/json' \
    -d '{"aud":"ubl://cli","scope":["read","write"]}' | jq
  ```

### E) Comandos de Triage
- ✅ Executados - nenhum problema encontrado:
  1. `/metrics` já plugado ✅
  2. Nenhum `pg_notify` com payload grande ✅
  3. Nenhum DSN hardcoded com TCP ✅
  4. Nenhuma referência ao pocket ✅

## 📋 Próximos Passos

1. **Aplicar migration:**
   ```bash
   sqlx migrate run --database-url "postgres://..."
   ```

2. **Testar endpoints:**
   ```bash
   # Metrics
   curl -s http://localhost:8080/metrics | head
   
   # SSE
   curl -N http://localhost:8080/ledger/tail
   
   # JWT Token
   curl -s -X POST http://localhost:8080/id/session/token \
     -H 'content-type: application/json' \
     -d '{"aud":"ubl://cli","scope":["read"]}' | jq
   ```

## 🔒 Segurança

- Unix Socket é OBRIGATÓRIO quando configurado (sem fallback TCP)
- SSE payload mínimo (cid:seq) para evitar limite 8KB
- JWT com step-up para escopos admin
- ASC validation em todos os commits

## 📝 Notas

- O Postgres LISTEN/NOTIFY é feito via trigger (migration 999)
- O TailBus também é notificado diretamente no `route_commit` para garantir
- O endpoint `/metrics` pode ser estendido com prometheus crate se necessário

