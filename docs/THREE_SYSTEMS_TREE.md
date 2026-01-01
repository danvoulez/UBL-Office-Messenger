# 🌐 Mapa Completo do Projeto

**Gerado em:** 2026-01-01  
**Total:** ~800 arquivos (sem node_modules/target/.git)

---

## 📊 VISÃO MACRO - O PROJETO COMPLETO

```
╔══════════════════════════════════════════════════════════════════════════════╗
║                         MONOREPO: 800 ARQUIVOS                               ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║  ubl/                    431 arquivos (54%)  ← O PROTOCOLO                   ║
║  ├── kernel/rust/        ~100 rs            ← ubl-server, crates             ║
║  ├── sql/                 36 sql            ← Migrations                     ║
║  ├── containers/          80+ md/json       ← C.Jobs, C.Messenger, etc       ║
║  ├── specs/               10 md             ← Specs do protocolo             ║
║  ├── clients/             30 ts             ← CLI + SDK                      ║
║  ├── infra/               40 sh/yaml        ← Docker, WireGuard, MinIO       ║
║  ├── manifests/           10 yaml/json      ← Policies, constitutions        ║
║  └── runner/              10 ts/sh          ← Executor isolado               ║
║                                                                              ║
║  apps/                   198 arquivos (25%)  ← APLICAÇÕES                    ║
║  ├── messenger/           98 tsx/ts         ← Frontend React                 ║
║  └── office/              83 rs             ← Backend LLM/MCP                ║
║                                                                              ║
║  tests/                  140 arquivos (18%)  ← TESTES                        ║
║  ├── __tests__/           12 tsx            ← Unit tests React               ║
║  ├── tests/*.rs           35 rs             ← Integration Rust               ║
║  ├── *.sh                 15 sh             ← Shell test scripts             ║
║  ├── docker-compose.*      6 yml            ← Test environments              ║
║  └── Playwright            3 ts             ← E2E tests                      ║
║                                                                              ║
║  observability/           44 arquivos (5%)   ← MONITORING                    ║
║  ├── grafana/             10 json           ← Dashboards                     ║
║  ├── prometheus/           5 yml            ← Alertas e rules                ║
║  ├── loki/promtail/        5 yml            ← Logs                           ║
║  └── *.md                 10 md             ← Runbooks                       ║
║                                                                              ║
║  docs/                    30 arquivos        ← DOCUMENTAÇÃO                  ║
║  ├── adrs/                 5 md             ← Architecture decisions         ║
║  ├── devops/               4 md             ← Runbooks                       ║
║  └── *.md                 20 md             ← Specs, roadmaps                ║
║                                                                              ║
║  contracts/                6 json            ← SCHEMAS                       ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

---

## 🎯 PRIORIDADES BASEADAS NA ANÁLISE

### O QUE ESTÁ MADURO ✅
| Área | Arquivos | Status |
|------|----------|--------|
| **UBL Kernel** (`ubl/kernel/rust/ubl-server/`) | 56 rs | ✅ WebAuthn, sessions, tenant OK |
| **SQL Migrations** (`ubl/sql/`) | 36 sql | ✅ Schema completo |
| **Frontend Messenger** (`apps/messenger/`) | 66 tsx | ✅ Login/onboarding OK |
| **LLM Providers** (`apps/office/src/llm/`) | 7 rs | ✅ 4 providers |
| **Observability config** | 44 arquivos | ✅ Pronto pra deploy |

### O QUE ESTÁ PARCIAL 🔄
| Área | Arquivos | Status |
|------|----------|--------|
| **MCP** (`apps/office/src/mcp/`) | 12 rs | 🔄 Client existe, Gateway não |
| **Job Executor** (`apps/office/src/job_executor/`) | 6 rs | 🔄 Estrutura OK, falta wiring |
| **Projections** (`ubl/.../projections/`) | 11 rs | 🔄 Código existe, falta usar |
| **UBL Containers** (`ubl/containers/`) | 80 md/json | 🔄 Schemas, sem implementação |

### O QUE FALTA ❌
| Área | Existe? | Próximo |
|------|---------|---------|
| **Append-only events** | ❌ Não | P0 - Criar `ubl_events` table |
| **MCP Gateway** | ❌ Só client | P0 - Agregar MCPs externos |
| **Office-Plan** | ❌ Spec só | P1 - Implementar spec |
| **UI real de jobs** | ❌ Mocks | P2 - Conectar ao backend |

---

## 📁 DETALHAMENTO POR CAMADA

### 1. UBL (431 arquivos) - O Protocolo

```
ubl/
├── kernel/                          # 🔥 CORE - O que roda
│   ├── rust/                        # Workspace Cargo
│   │   ├── Cargo.toml               # Workspace root
│   │   ├── ubl-server/              # ⚡ O SERVIDOR PRINCIPAL
│   │   │   ├── src/
│   │   │   │   ├── main.rs          # Entry point :8080
│   │   │   │   ├── auth/            # 🔐 Sessões (FUNCIONA ✅)
│   │   │   │   ├── tenant/          # 🏢 Multi-tenant (FUNCIONA ✅)
│   │   │   │   ├── identity/        # 🪪 WebAuthn (FUNCIONA ✅)
│   │   │   │   ├── projections/     # 📊 Views (existe, usar mais)
│   │   │   │   ├── messenger_gateway/ # 📨 Gateway (existe)
│   │   │   │   └── id_routes.rs     # Rotas auth
│   │   │   └── sql/                 # Migrations locais
│   │   │
│   │   ├── ubl-atom/                # 📦 Atoms (imutáveis)
│   │   ├── ubl-ledger/              # 📒 Ledger lib
│   │   ├── ubl-link/                # 🔗 Links entre atoms
│   │   ├── ubl-membrane/            # 🛡️ Boundaries
│   │   ├── ubl-pact/                # 🤝 Contratos
│   │   ├── ubl-policy-vm/           # 📜 Policy engine
│   │   └── ubl-runner-core/         # ⚙️ Runner base
│   │
│   ├── openapi/                     # OpenAPI spec
│   └── tests/                       # Golden tests
│
├── sql/                             # 📝 MIGRATIONS PRINCIPAIS
│   ├── 00_base/                     # Core tables
│   │   ├── 000_core.sql             # ubl_ledger, etc
│   │   ├── 001_identity.sql         # id_identity, id_credential
│   │   ├── 002_tenant.sql           # id_tenant, id_invite_code
│   │   └── 004_session_tenant.sql   # Session+tenant link
│   ├── 10_projections/              # Views
│   │   ├── 100_console.sql
│   │   ├── 101_messenger.sql
│   │   └── 102_office.sql
│   └── 99_legacy/                   # Old migrations
│
├── containers/                      # 📦 CONTAINER SPECS (não código)
│   ├── C.Jobs/                      # Container de jobs
│   │   ├── EVENT_TYPES.md           # Tipos de eventos
│   │   ├── pacts/ref.json           # Contratos
│   │   └── policy/ref.json          # Políticas
│   ├── C.Messenger/                 # Container messenger
│   ├── C.Office/                    # Container office
│   ├── C.Artifacts/                 # Artefatos
│   ├── C.Pacts/                     # Pacts
│   ├── C.Policy/                    # Policies
│   ├── C.Runner/                    # Runner sandboxed
│   └── C.Tenant/                    # Tenants
│
├── specs/                           # 📚 ESPECIFICAÇÕES
│   ├── PHILOSOPHY.md                # Filosofia UBL
│   ├── ubl-atom/SPEC-UBL-ATOM.md    # Spec atoms
│   ├── ubl-ledger/SPEC-UBL-LEDGER.md
│   ├── ubl-link/SPEC-UBL-LINK.md
│   ├── ubl-llm/SPEC-UBL-LLM.md      # 🔥 Spec LLM
│   ├── ubl-membrane/SPEC-UBL-MEMBRANE.md
│   ├── ubl-pact/SPEC-UBL-PACT.md
│   ├── ubl-policy/SPEC-UBL-POLICY.md
│   └── ubl-runner/SPEC-UBL-RUNNER.md
│
├── clients/                         # 🖥️ CLIENTS
│   ├── cli/                         # CLI TypeScript
│   │   ├── src/cmds/                # Comandos
│   │   │   ├── atom.ts              # ubl atom
│   │   │   ├── commit.ts            # ubl commit
│   │   │   ├── doctor.ts            # ubl doctor
│   │   │   ├── id.ts                # ubl id
│   │   │   └── tail.ts              # ubl tail
│   │   └── src/utils/               # Utils
│   ├── ts/sdk/                      # SDK TypeScript
│   └── types/                       # Type definitions
│
├── infra/                           # 🏗️ INFRAESTRUTURA
│   ├── docker-compose.stack.yml     # Stack completo
│   ├── lab-256/                     # Config LAB-256
│   ├── lab-512/                     # Config LAB-512
│   ├── postgres/                    # Postgres scripts
│   ├── minio/                       # MinIO config
│   ├── wireguard/                   # VPN config
│   └── secpack/                     # Security hardening
│
├── manifests/                       # 📋 MANIFESTOS
│   ├── containers.json              # Container registry
│   ├── offices.yaml                 # Office config
│   ├── policies.json                # Policies
│   └── office.constitution.yaml     # 🔥 Constitution do Office
│
├── runner/                          # ⚙️ RUNNER ISOLADO
│   ├── pull_only.ts                 # Pull jobs
│   ├── crypto.ts                    # Signing
│   └── executors/                   # Sandbox executors
│
└── mind/                            # 🧠 ABAC/Agreements
    ├── src/abac.ts                  # Attribute-based access
    └── src/agreements.ts            # Agreement logic
```

### 2. APPS (198 arquivos) - As Aplicações

```
apps/
├── messenger/                       # 📱 FRONTEND
│   ├── frontend/
│   │   ├── src/
│   │   │   ├── pages/               # 📑 4 páginas
│   │   │   │   ├── LoginPage.tsx    # ✅ WebAuthn OK
│   │   │   │   ├── OnboardingPage.tsx # ✅ Tenant OK
│   │   │   │   ├── ChatPage.tsx     # 🔄 Precisa jobs reais
│   │   │   │   └── SettingsPage.tsx
│   │   │   ├── components/          # 🧩 21 componentes
│   │   │   │   ├── cards/           # Cards de jobs
│   │   │   │   ├── modals/          # Modais
│   │   │   │   └── ui/              # Design system
│   │   │   ├── services/            # 🔌 APIs
│   │   │   │   ├── apiClient.ts     # ✅ Auth header OK
│   │   │   │   └── ublApi.ts        # UBL calls
│   │   │   ├── hooks/               # 🪝 React hooks
│   │   │   └── context/             # 🔄 Providers
│   │   ├── public/                  # Assets
│   │   └── Dockerfile
│   └── *.md                         # Docs específicas
│
└── office/                          # 🧠 BACKEND LLM
    ├── src/
    │   ├── main.rs                  # Entry :8081
    │   ├── llm/                     # 🤖 Providers (OK)
    │   │   ├── anthropic.rs         # Claude
    │   │   ├── openai.rs            # GPT
    │   │   ├── gemini.rs            # Gemini
    │   │   └── router.rs            # Router
    │   ├── mcp/                     # 🔧 MCP (FOCO P0)
    │   │   ├── client.rs            # MCP client
    │   │   ├── protocol.rs          # Protocol types
    │   │   ├── registry.rs          # Tool registry
    │   │   └── transport.rs         # Stdio/HTTP
    │   ├── job_executor/            # ⚙️ Jobs
    │   ├── ubl_client/              # 📡 UBL client
    │   └── governance/              # ⚖️ Constitution
    └── config/
        ├── development.toml
        └── production.toml
```

### 3. TESTS (140 arquivos) - Testes

```
tests/
├── __tests__/                       # Jest/Vitest
│   ├── components/                  # React component tests
│   ├── integration/                 # Integration tests
│   └── e2e/                         # E2E tests
├── tests/                           # Rust integration
│   ├── golden_path.rs               # Happy path
│   ├── chaos_monkey.rs              # Chaos testing
│   ├── multi_tenant.rs              # Multi-tenant
│   └── *.rs                         # 35+ test files
├── *.sh                             # Shell scripts
│   ├── 01-foundation.sh
│   ├── 02-golden-paths.sh
│   ├── run-e2e-tests.sh
│   └── run-integration-tests.sh
└── docker-compose.*.yml             # Test environments
```

### 4. OBSERVABILITY (44 arquivos) - Monitoring

```
observability/
├── grafana/
│   └── provisioning/
│       ├── dashboards/
│       │   ├── office-runtime.json
│       │   ├── system-overview.json
│       │   └── ubl-kernel.json
│       └── datasources/
├── prometheus/
│   ├── prometheus.yml
│   └── alerts/
│       └── cryptography.yml
├── loki/
│   └── loki-config.yml
├── promtail/
│   └── promtail-config.yml
├── alertmanager/
│   └── alertmanager.yml
└── runbooks/
    └── *.md
```

---

## 🎯 MATRIZ DE DECISÃO: O QUE FAZER AGORA?

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        PRIORIDADES DE IMPLEMENTAÇÃO                         │
├───────────────────┬───────────────┬─────────────────┬───────────────────────┤
│      ITEM         │   IMPACTO     │    ESFORÇO      │      DEPENDÊNCIA      │
├───────────────────┼───────────────┼─────────────────┼───────────────────────┤
│ MCP Gateway       │    ALTO       │     MÉDIO       │ Nenhuma               │
│ (office/mcp/)     │ Habilita todo │  3-5 dias       │                       │
│                   │ ecossistema   │                 │                       │
├───────────────────┼───────────────┼─────────────────┼───────────────────────┤
│ Append-only       │    ALTO       │     BAIXO       │ Nenhuma               │
│ events table      │ Fundação      │  1 dia          │                       │
├───────────────────┼───────────────┼─────────────────┼───────────────────────┤
│ Wire jobs UI      │    MÉDIO      │     MÉDIO       │ MCP Gateway           │
│ (messenger)       │ UX completo   │  2-3 dias       │                       │
├───────────────────┼───────────────┼─────────────────┼───────────────────────┤
│ Office-Plan       │    MÉDIO      │     ALTO        │ Events + MCP          │
│ (SPEC completa)   │ Feature       │  5-7 dias       │                       │
├───────────────────┼───────────────┼─────────────────┼───────────────────────┤
│ Observability     │    BAIXO      │     BAIXO       │ Docker Compose        │
│ (já configurado)  │ Produção      │  1 dia          │                       │
└───────────────────┴───────────────┴─────────────────┴───────────────────────┘

RECOMENDAÇÃO:
  Semana 1: MCP Gateway + Append-only events
  Semana 2: Wire jobs UI + Office-Plan básico
```

---

## 📁 ARQUIVOS CHAVE POR OBJETIVO

### Se quiser: **Adicionar novo MCP**
→ `apps/office/src/mcp/config.rs` + `registry.rs`

### Se quiser: **Adicionar nova rota no UBL**
→ `ubl/kernel/rust/ubl-server/src/main.rs` (mount)
→ Criar novo `*_routes.rs`

### Se quiser: **Nova migration SQL**
→ `ubl/sql/00_base/` (criar próximo número)

### Se quiser: **Novo componente React**
→ `apps/messenger/frontend/src/components/`

### Se quiser: **Novo tipo de evento**
→ `ubl/containers/C.*/EVENT_TYPES.md` (spec)
→ `ubl/kernel/rust/ubl-server/src/projections/` (handler)

### Se quiser: **Deploy observability**
→ `observability/docker-compose.observability.yml`

---

# 🧠 ÁRVORE DETALHADA: CÓDIGO QUE EXECUTA

Abaixo está o detalhamento dos 3 sistemas principais (código que roda):

---

## 📱 MESSENGER (Frontend React) - `:3000`

**66 arquivos** | React + Vite + TypeScript + Tailwind

```
apps/messenger/frontend/src/
├── App.tsx                          # Raiz do app
├── index.tsx                        # Entry point
├── types.ts                         # Tipos globais
├── constants.tsx                    # Constantes
├── vite-env.d.ts                    # Tipos Vite
│
├── 🧩 components/                   # UI Components (21)
│   ├── BridgeConfig.tsx
│   ├── ChatView.tsx                 # View principal do chat
│   ├── ErrorBoundary.tsx
│   ├── JobArtifacts.tsx
│   ├── JobCard.tsx
│   ├── JobDrawer.tsx
│   ├── JobTimeline.tsx
│   ├── Sidebar.tsx
│   ├── VirtualizedList.tsx
│   ├── WelcomeScreen.tsx
│   │
│   ├── 🃏 cards/                    # Cards de jobs
│   │   ├── index.ts
│   │   ├── AcceptanceCard.tsx       # Card de aprovação
│   │   ├── JobCardRenderer.tsx
│   │   └── LiveProgressCard.tsx     # Progresso em tempo real
│   │
│   ├── 🪟 modals/                   # Modais
│   │   ├── index.ts
│   │   ├── index.tsx
│   │   ├── EntityProfileModal.tsx
│   │   ├── NewWorkstreamModal.tsx
│   │   └── TaskCreationModal.tsx
│   │
│   └── 🎨 ui/                       # Design system
│       ├── index.ts
│       ├── Avatar.tsx
│       ├── Badge.tsx
│       ├── Button.tsx
│       ├── GhostCard.tsx
│       ├── HoldButton.tsx
│       ├── Input.tsx
│       ├── MessageStatus.tsx
│       ├── Modal.tsx
│       ├── Spinner.tsx
│       ├── SyncStatus.tsx
│       └── ThoughtStream.tsx
│
├── 📑 pages/                        # Páginas (4)
│   ├── ChatPage.tsx                 # Chat principal
│   ├── LoginPage.tsx                # WebAuthn login ✅
│   ├── OnboardingPage.tsx           # Setup tenant ✅
│   └── SettingsPage.tsx
│
├── 🔌 services/                     # APIs e serviços (8)
│   ├── apiClient.ts                 # HTTP client com auth ✅
│   ├── ublApi.ts                    # Chamadas ao UBL
│   ├── jobsApi.ts                   # Chamadas de jobs
│   ├── crypto.ts                    # Crypto utils
│   ├── eventBus.ts                  # Event bus local
│   ├── ledger.ts                    # Ledger client
│   ├── network.ts                   # Network utils
│   └── sse.ts                       # Server-sent events
│
├── 🪝 hooks/                        # React hooks (5)
│   ├── useAuth.ts                   # Auth state
│   ├── useJobs.ts                   # Jobs state
│   ├── useSSE.ts                    # SSE connection
│   ├── useOptimistic.ts             # Optimistic updates
│   └── useAudioEngine.ts            # Audio feedback
│
├── 🔄 context/                      # React context (5)
│   ├── AuthContext.tsx              # Auth provider ✅
│   ├── ThemeContext.tsx             # Theme provider
│   ├── NotificationContext.tsx      # Notifications
│   ├── OnboardingContext.tsx        # Onboarding state
│   └── ProtocolContext.tsx          # Protocol state
│
├── 🎨 theme/
│   └── ThemeProvider.tsx
│
├── 📚 lib/
│   ├── cn.ts                        # classNames helper
│   └── toast.tsx                    # Toast notifications
│
├── 📈 observability/                # Observability (3)
│   ├── index.ts
│   ├── metrics.ts
│   └── tracing.ts
│
└── 🔒 utils/
    └── security.ts                  # Security utils
```

---

## 🧠 OFFICE (Backend Rust - Cérebro) - `:8081`

**83 arquivos** | Rust + Axum + LLM + MCP

```
apps/office/src/
├── main.rs                          # Entry point
├── lib.rs                           # Library exports
├── types.rs                         # Tipos globais
├── asc.rs                           # ASC (?)
├── http_unix.rs                     # Unix socket HTTP
│
├── 🤖 llm/                          # LLM Providers (7)
│   ├── mod.rs
│   ├── provider.rs                  # Trait Provider
│   ├── router.rs                    # Router entre providers
│   ├── anthropic.rs                 # Claude
│   ├── openai.rs                    # GPT
│   ├── gemini.rs                    # Gemini
│   └── local.rs                     # Modelos locais
│
├── 🔧 mcp/                          # Model Context Protocol (12)
│   ├── mod.rs
│   ├── protocol.rs                  # MCP protocol types
│   ├── client.rs                    # MCP client
│   ├── transport.rs                 # Stdio/HTTP transport
│   ├── registry.rs                  # Tool registry
│   ├── unified_registry.rs          # Registry unificado
│   ├── tool_executor.rs             # Executor de tools
│   ├── config.rs                    # Config MCPs
│   ├── builtin.rs                   # Tools built-in
│   ├── native.rs                    # Tools nativos
│   ├── native_server.rs             # Servidor MCP nativo
│   └── prompts.rs                   # Prompts de MCPs
│
├── ⚙️ job_executor/                  # Execução de Jobs (6)
│   ├── mod.rs
│   ├── executor.rs                  # Job executor principal
│   ├── fsm.rs                       # State machine
│   ├── types.rs                     # Tipos de job
│   ├── cards.rs                     # Cards de output
│   └── conversation_context.rs      # Contexto de conversa
│
├── ✅ task/                          # Execução de Tasks (5)
│   ├── mod.rs
│   ├── executor.rs
│   ├── fsm.rs
│   ├── types.rs
│   └── cards.rs
│
├── 👤 entity/                        # Entidades (6)
│   ├── mod.rs
│   ├── entity.rs                    # Base entity
│   ├── guardian.rs                  # Entity guardian
│   ├── identity.rs                  # Identity entity
│   ├── instance.rs                  # Instance
│   └── repository.rs                # Entity repo
│
├── ⚖️ governance/                    # Governança (6)
│   ├── mod.rs
│   ├── constitution.rs              # Regras constitucionais
│   ├── dreaming.rs                  # Modo "sonho"
│   ├── provenance.rs                # Proveniência
│   ├── sanity_check.rs              # Sanity checks
│   └── simulation.rs                # Simulações
│
├── 🎫 session/                       # Sessões LLM (5)
│   ├── mod.rs
│   ├── session.rs                   # Session state
│   ├── handover.rs                  # Handover entre sessões
│   ├── modes.rs                     # Modos de sessão
│   └── token_budget.rs              # Budget de tokens
│
├── 📡 ubl_client/                    # Cliente UBL (7)
│   ├── mod.rs
│   ├── ledger.rs                    # Ledger client
│   ├── events.rs                    # Event types
│   ├── identity_events.rs           # Identity events
│   ├── trust.rs                     # Trust verification
│   ├── receipts.rs                  # Receipts
│   └── affordances.rs               # Affordances
│
├── 🔄 context/                       # Context building (5)
│   ├── mod.rs
│   ├── builder.rs                   # Context builder
│   ├── frame.rs                     # Context frame
│   ├── memory.rs                    # Memory management
│   └── narrator.rs                  # Context narrator
│
├── 📝 audit/                         # Auditoria (4)
│   ├── mod.rs
│   ├── events.rs                    # Audit events
│   ├── pii.rs                       # PII handling
│   └── tool_audit.rs                # Tool auditing
│
├── 🌐 api/                           # HTTP API (5)
│   ├── mod.rs
│   ├── http.rs                      # HTTP routes
│   ├── mcp.rs                       # MCP routes
│   ├── websocket.rs                 # WebSocket
│   └── task_routes.rs               # Task routes
│
├── 🛡️ middleware/                    # Middleware (3)
│   ├── mod.rs
│   ├── constitution.rs              # Constitution check
│   └── permit.rs                    # Permission check
│
├── 🚀 routes/                        # Routes extras (3)
│   ├── mod.rs
│   ├── deploy.rs                    # Deploy routes
│   └── ws.rs                        # WebSocket routes
│
└── 📈 observability/                 # Observability (3)
    ├── mod.rs
    ├── metrics.rs
    └── tracing.rs
```

---

## ⚡ UBL KERNEL (Backend Rust - Verdade) - `:8080`

**56 arquivos** | Rust + Axum + PostgreSQL + WebAuthn

```
ubl/kernel/rust/ubl-server/src/
├── main.rs                          # Entry point
├── db.rs                            # Database pool
├── crypto.rs                        # Crypto utils
├── keystore.rs                      # Key management
├── webauthn_store.rs                # WebAuthn credentials
├── tracing.rs                       # Tracing setup
├── metrics.rs                       # Metrics
├── sse.rs                           # Server-sent events
├── snapshots.rs                     # Snapshots
├── rate_limit.rs                    # Rate limiting
├── otel_tracing.rs                  # OpenTelemetry
├── pact_db.rs                       # Pacts database
├── policy_registry.rs               # Policy registry
│
├── 🔐 auth/                          # Autenticação (4)
│   ├── session.rs                   # Session struct ✅ (String SID)
│   ├── session_db.rs                # Session DB ops ✅
│   └── require_stepup.rs            # Step-up auth
│
├── 🪪 identity/                      # Identidade (6)
│   ├── mod.rs
│   ├── challenge.rs                 # Auth challenges
│   ├── config.rs                    # Identity config
│   ├── error.rs                     # Error types
│   ├── session.rs                   # Identity session
│   └── token.rs                     # Token management
│
├── id_routes.rs                     # Identity routes (WebAuthn) ✅
├── id_db.rs                         # Identity database
├── id_ledger.rs                     # Identity ledger
├── id_session_token.rs              # Session tokens
├── middleware_require_stepup.rs     # Stepup middleware
│
├── 🏢 tenant/                        # Multi-tenancy (4)
│   ├── mod.rs
│   ├── db.rs                        # Tenant DB ✅
│   ├── routes.rs                    # Tenant routes ✅
│   └── types.rs                     # Tenant types
│
├── 📊 projections/                   # Projeções (11)
│   ├── mod.rs
│   ├── routes.rs                    # Projection routes
│   ├── rebuild.rs                   # Rebuild projections
│   ├── jobs.rs                      # Job projections
│   ├── job_events.rs                # Job event handling
│   ├── messages.rs                  # Message projections
│   ├── timeline.rs                  # Timeline
│   ├── artifacts.rs                 # Artifacts
│   ├── office.rs                    # Office projections
│   └── presence.rs                  # Presence tracking
│
├── 📨 messenger_gateway/             # Gateway Messenger (6)
│   ├── mod.rs
│   ├── routes.rs                    # Gateway routes
│   ├── sse.rs                       # SSE for messenger
│   ├── office_client.rs             # Office HTTP client
│   ├── idempotency.rs               # Idempotency
│   └── projections.rs               # Gateway projections
│
├── 📜 policy/                        # Políticas (2)
│   ├── mod.rs
│   └── policies.rs                  # Policy definitions
│
├── console_v1.rs                    # Console API v1
├── messenger_v1.rs                  # Messenger API v1
├── registry_v1.rs                   # Registry API v1
├── repo_routes.rs                   # Repo routes
└── integrate_repo_routes.rs         # Route integration
```

---

## 📡 Fluxo de Dados

```
┌─────────────────┐        ┌─────────────────┐        ┌─────────────────┐
│   MESSENGER     │        │     OFFICE      │        │   UBL KERNEL    │
│   React :3000   │        │   Rust :8081    │        │   Rust :8080    │
├─────────────────┤        ├─────────────────┤        ├─────────────────┤
│                 │        │                 │        │                 │
│  LoginPage      │──────▶ │                 │        │  id_routes      │
│  apiClient      │        │                 │──────▶ │  auth/session   │
│  ublApi         │        │  ubl_client     │        │  tenant/        │
│                 │        │  ledger.rs      │        │                 │
│  ChatPage       │◀─ WS ─▶│  api/websocket  │        │                 │
│  useSSE         │◀─ SSE ─│                 │◀─ SSE ─│  sse.rs         │
│                 │        │                 │        │                 │
│  JobCard        │        │  job_executor   │        │  projections/   │
│  cards/         │◀───────│  task/          │──────▶ │  jobs.rs        │
│                 │        │  mcp/           │        │                 │
│                 │        │  llm/           │        │                 │
└─────────────────┘        └─────────────────┘        └─────────────────┘
     [UI/UX]                   [LLM/MCP]              [Verdade/Auth]
```

---

## ✅ Status Atual

### Funcionando:
- [x] WebAuthn discoverable credentials
- [x] Auto-login após registro
- [x] Session com SID string (não UUID)
- [x] Multi-tenant real (PostgreSQL)
- [x] Invite codes funcionais
- [x] Frontend conectado ao backend real

### Próximos Passos (P0):
- [ ] MCP Gateway no Office
- [ ] Append-only event store
- [ ] Office-Plan system (SPEC_OFFICE_PLAN.md)
- [ ] Remover mocks do UI
- [ ] Observability completo

---

## 📚 Documentação Relacionada

- [ROADMAP_2026.md](ROADMAP_2026.md) - Roadmap completo
- [RESEARCH_AGENDA.md](RESEARCH_AGENDA.md) - Agenda de pesquisa
- [SPEC_OFFICE_PLAN.md](SPEC_OFFICE_PLAN.md) - Spec do sistema de planos
- [THREE_SYSTEMS_OVERVIEW.md](THREE_SYSTEMS_OVERVIEW.md) - Overview original
- [ARCHITECTURE.md](ARCHITECTURE.md) - Arquitetura geral
