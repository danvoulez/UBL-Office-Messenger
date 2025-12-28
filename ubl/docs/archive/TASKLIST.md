# UBL 2.0 - Tasklist de Implementação

**Data:** 2025-12-25  
**Status:** LAB 512 em configuração

---

## 🎯 Prioridades de Desenvolvimento

### ✅ COMPLETO - Foundation (Chain 1)

- [x] **ubl-atom** - Hashing canônico com BLAKE3
- [x] **ubl-kernel** - Hash binding e link commits
- [x] **ubl-link** - Assinatura Ed25519 e commits
- [x] **ubl-membrane** - Physics invariants (thermodynamics)
- [x] **ubl-ledger** - Interface de ledger append-only
- [x] **ubl-pact** - Agreement verification
- [x] **ubl-policy-vm** - Policy engine com ABAC
- [x] **ubl-runner-core** - Container orchestration
- [x] **ubl-server** - HTTP API com 6 rotas
- [x] **ubl-cortex** (TypeScript) - ABAC minds/agreements
- [x] 43+ testes passando
- [x] Documentação completa
- [x] Monorepo consolidado

### 🔄 EM PROGRESSO - Persistence (Chain 2)

#### LAB 512 - Development Environment

**Configuração com Container (SPEC):**

1. **PostgreSQL 16 em Docker Compose**
   - [ ] Remover instalação Homebrew (conflito)
   - [ ] Docker daemon rodando
   - [ ] Container postgres:16-alpine
   - [ ] Volume persistente para dados
   - [ ] Auto-migrations via init-db.sh
   - [ ] Health check com pg_isready
   - [ ] Network bridge ubl-dev

2. **Database Setup**
   - [ ] Database: `ubl_dev`
   - [ ] User: `ubl_dev` / Password: `dev_password_local_only`
   - [ ] Port: 5432 (localhost)
   - [ ] Connection string no .env

3. **Migrations Automáticas**
   - [ ] 001_ledger.sql - Tabela append-only com 14 colunas
   - [ ] 002_idempotency.sql - Idempotency keys
   - [ ] 003_observability.sql - Metrics/tracing
   - [ ] 004_disaster_recovery.sql - Backup metadata
   - [ ] schema_version table para tracking

4. **Integração com ubl-server**
   - [ ] Adicionar sqlx no Cargo.toml
   - [ ] Implementar append() com PostgreSQL
   - [ ] Implementar tail() com LISTEN/NOTIFY
   - [ ] Connection pool
   - [ ] Transações SERIALIZABLE
   - [ ] FOR UPDATE lock no last sequence

5. **Testing**
   - [ ] Integration tests com banco real
   - [ ] Append tests
   - [ ] Tail tests
   - [ ] Concurrency tests
   - [ ] Health check endpoint

### 📋 PLANEJADO - LAB 256 Production (Chain 2 + 3)

**Amanhã - Configuração Completa:**

1. **PostgreSQL Production**
   - [ ] Cloud/Bare Metal setup
   - [ ] TLS/SSL encryption
   - [ ] Connection pooling (PgBouncer)
   - [ ] Master-Replica replication
   - [ ] Automated backups (WAL)
   - [ ] Point-in-time recovery
   - [ ] Monitoring (Prometheus/Grafana)

2. **Security Hardening**
   - [ ] mTLS entre zonas
   - [ ] WebAuthn/Passkey (PR28-29)
   - [ ] Step-up admin (10min TTL)
   - [ ] Session management
   - [ ] Rate limiting
   - [ ] Audit logging

3. **WireGuard Networking**
   - [ ] Zone isolation (PR30)
   - [ ] LAB 256 (API) network
   - [ ] LAB 512 (Sandbox) network
   - [ ] Firewall rules (iptables.sh)
   - [ ] Service discovery

4. **MinIO Artifacts**
   - [ ] Object storage setup
   - [ ] Bucket policies
   - [ ] Versioning enabled
   - [ ] Lifecycle policies
   - [ ] Integration with ubl-server

### 📋 PLANEJADO - Features Avançadas (Chain 3-5)

#### Security (Chain 3)
- [ ] WebAuthn implementation (PR28)
- [ ] Passkey support (PR29)
- [ ] Admin step-up auth
- [ ] Session expiry (10min)
- [ ] Remover "mock" signatures

#### Artifacts (Chain 4)
- [ ] WASM execution sandbox
- [ ] Artifact storage (MinIO)
- [ ] Proof verification
- [ ] Runner coordination

#### Developer Experience (Chain 5)
- [ ] TypeScript SDK (PR32)
- [ ] BLAKE3 via WASM
- [ ] Ed25519 browser signing
- [ ] Zod schemas auto-generated
- [ ] Conformance tests (PR27)
- [ ] Cross-language golden hashes
- [ ] Fuzzing

#### Containers
- [ ] C.Messenger (green/public)
- [ ] C.Policy (blue/admin)
- [ ] C.Artifacts (yellow/storage)
- [ ] C.Runner (black/execution)

---

## 🔧 Estrutura de Diretórios

```
/Users/voulezvous/UBL-2.0-insiders/
├── kernel/rust/          # 9 Rust crates
│   ├── ubl-atom/
│   ├── ubl-kernel/
│   ├── ubl-link/
│   ├── ubl-membrane/
│   ├── ubl-ledger/
│   ├── ubl-pact/
│   ├── ubl-policy-vm/
│   ├── ubl-runner-core/
│   └── ubl-server/
├── mind/typescript/      # TypeScript cortex
│   └── ubl-cortex/
├── sql/                  # Database migrations
│   ├── 001_ledger.sql
│   ├── 002_idempotency.sql
│   ├── 003_observability.sql
│   └── 004_disaster_recovery.sql
├── infra/
│   ├── lab-512/          # 🔄 Development (hoje)
│   │   ├── docker-compose.yml
│   │   ├── init-db.sh
│   │   └── README.md
│   ├── lab-256/          # 📋 Production (amanhã)
│   │   ├── api.service
│   │   ├── tail.service
│   │   └── iptables.sh
│   ├── postgres/         # Backup/restore scripts
│   ├── minio/            # Object storage
│   └── wireguard/        # Network configs
└── containers/           # Future implementations
    ├── C.Messenger/
    ├── C.Policy/
    ├── C.Artifacts/
    └── C.Runner/
```

---

## 📊 Métricas

### Foundation ✅
- **Crates:** 9/9 completos
- **Tests:** 43+ passando
- **Coverage:** 100% specs implementadas
- **Docs:** Completos

### Persistence 🔄
- **Database:** PostgreSQL 16
- **Schema:** 4 migrations
- **Integration:** Em progresso
- **Testing:** Pendente

### Production 📋
- **LAB 256:** Planejado para amanhã
- **Security:** WebAuthn pendente
- **Networking:** WireGuard pendente
- **Monitoring:** Pendente

---

## 🎯 Próximos Passos (Hoje - LAB 512)

1. **Remover PostgreSQL Homebrew**
   ```bash
   brew services stop postgresql@16
   brew uninstall postgresql@16
   ```

2. **Iniciar Docker Daemon**
   - Abrir Docker Desktop ou OrbStack
   - Verificar: `docker ps`

3. **Subir Container PostgreSQL**
   ```bash
   cd /Users/voulezvous/UBL-2.0-insiders/infra/lab-512
   docker-compose up -d
   docker-compose logs -f postgres
   ```

4. **Verificar Migrations**
   ```bash
   docker exec ubl-postgres-dev psql -U ubl_dev -d ubl_dev -c "\dt"
   docker exec ubl-postgres-dev psql -U ubl_dev -d ubl_dev -c "\d ledger_entry"
   ```

5. **Atualizar .env**
   ```bash
   # kernel/rust/.env
   DATABASE_URL=postgres://ubl_dev:dev_password_local_only@localhost:5432/ubl_dev
   ```

6. **Integrar com ubl-server**
   - Adicionar sqlx dependencies
   - Implementar append() com PostgreSQL
   - Implementar tail() com LISTEN/NOTIFY
   - Testes de integração

---

## 🎯 Próximos Passos (Amanhã - LAB 256)

1. **PostgreSQL Production**
   - Cloud setup (AWS RDS / DigitalOcean / Linode)
   - TLS certificates
   - Replication setup
   - Backup configuration

2. **WireGuard Network**
   - Generate keys
   - Configure zones
   - Firewall rules
   - Service routing

3. **MinIO Storage**
   - Deploy MinIO cluster
   - Configure buckets
   - Set policies
   - Integration testing

4. **Security**
   - WebAuthn setup
   - Admin policies
   - Audit logging
   - Rate limiting

---

**Status Atual:** PostgreSQL via Homebrew instalado (temporário)  
**Próximo:** Migrar para Docker Compose conforme SPEC  
**Objetivo:** LAB 512 completo hoje, LAB 256 amanhã
