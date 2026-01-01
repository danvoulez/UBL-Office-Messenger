# UBL Roadmap 2026

> Documento criado após **marco histórico**: Auth WebAuthn funcionando end-to-end pela primeira vez.
> Data: 1 de Janeiro de 2026
> Atualizado após **análise forense** da arquitetura.

---

## 🏛️ O Que Este Projeto É

> **Não é um SaaS comum, nem apenas um Chatbot.**

A estrutura `ubl-kernel` (ledger, atom, policy) + `office` (dreaming, constitution) indica que estamos construindo um **Sistema Operacional para Agentes Autônomos Multi-Tenant**.

### A Filosofia Central: Zero Trust Entre Colaboradores

```
┌─────────────────────────────────────────────────────────────────┐
│  "O sistema não confia em NENHUM dos dois,                      │
│   porque os dois são malucos e desesperados,                    │
│   e o sistema precisa ser PERMANENTE"                           │
└─────────────────────────────────────────────────────────────────┘
```

**O projeto resolve:**
1. **Imutabilidade e Confiança** (`ubl-ledger`, `ubl-pact`) - Nada é apagado
2. **Governança de IA** (`office/governance`, `constitution.rs`) - Regras que ninguém burla
3. **Execução Isolada** (`runner` em outro computador) - Separação física de responsabilidades

**O método convencional (único caminho):**
- LLM propõe rascunho
- Humano autoriza com passkey
- Sistema executa (nem LLM nem humano podem pular etapas)

**Feito em parceria humano/LLM → Mantido em parceria → Não confia em nenhum dos dois**

### Componentes e Suas Razões de Existir

| Componente | O que parece | O que realmente é |
|------------|--------------|-------------------|
| **Dreaming** | "Código zuado" | Consolidação de memória (como sono no cérebro) - organiza, limpa, prioriza |
| **Runner** | "Incompleto" | Propositalmente limitado e em outro computador - a limitação É a segurança |
| **Policy-VM** | "Overengineering?" | Perguntar ao autor original |
| **Constitution** | "Filosofia demais" | Regras imutáveis que governam o sistema |

---

## 🎯 Status Atual

### ✅ Conquistado (Fundação Sólida)
- **WebAuthn/Passkey** funcionando com discoverable credentials
- **Sessões** com SID string persistindo corretamente  
- **Multi-tenant** com onboarding e invite codes reais
- **Fluxo completo**: Login → Onboarding → App
- **Zona Schengen**: Contexto de tenant na sessão
- **IAM robusto**: Step-up auth, device credentials - raro em MVPs
- **Observability config**: 44 arquivos prontos para deploy
- **SQL migrations**: 36 arquivos, schema maduro

### ⚠️ Análise Forense - Riscos Identificados
| Área | Status | Risco |
|------|--------|-------|
| MCP Gateway | 🔴 Incompleto | "Filósofo numa caixa" - pensa mas não age |
| Event Sourcing | 🔴 Ausente | Banco usa UPDATE/DELETE (mutável) |
| UI Mocks | 🟠 Presente | Frontend promete mais do que backend entrega |
| Projections | 🟠 Fracas | Write-heavy mas UI read-heavy |
| Policy-VM | 🟡 Verificar | Complexidade desnecessária? |
| Runner | 🟡 Incompleto | Estratégia MCP-first pode substituir |

### 🔧 Stack Operacional
| Componente | Status | Porta |
|------------|--------|-------|
| UBL Kernel | ✅ Healthy | :8080 |
| Office LLM | ✅ Healthy | :8081 |
| Messenger | ✅ Running | :3000 |
| PostgreSQL | ✅ Running | :5432 |

---

## 📋 Roadmap por Prioridade

### 🔴 P0 - Crítico (Esta Semana)

#### 1. MCP Gateway no Office (PRIMEIRO!)
**Filosofia**: Office é um **orquestrador de MCPs**, não reimplementa tools.

```
┌─────────────────────────────────────────────────────────────────┐
│                    OFFICE = MCP GATEWAY                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌──────────┐     ┌──────────────┐     ┌───────────────────┐   │
│  │ Messenger│────>│    OFFICE    │────>│   MCP Servers     │   │
│  │          │     │  (Gateway)   │     │                   │   │
│  └──────────┘     └──────────────┘     │ ┌───────────────┐ │   │
│                          │             │ │ playwright    │ │   │
│                          │             │ │ (browser)     │ │   │
│                   ┌──────┴──────┐      │ ├───────────────┤ │   │
│                   │   Claude    │      │ │ filesystem    │ │   │
│                   │   (LLM)     │      │ │ (files)       │ │   │
│                   └─────────────┘      │ ├───────────────┤ │   │
│                                        │ │ postgres      │ │   │
│                                        │ │ (database)    │ │   │
│                                        │ ├───────────────┤ │   │
│                                        │ │ fetch         │ │   │
│                                        │ │ (http)        │ │   │
│                                        │ ├───────────────┤ │   │
│                                        │ │ slack/email   │ │   │
│                                        │ │ (comms)       │ │   │
│                                        │ ├───────────────┤ │   │
│                                        │ │ github        │ │   │
│                                        │ │ (code)        │ │   │
│                                        │ └───────────────┘ │   │
│                                        └───────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

**MCPs Externos Prioritários (Docker ready!):**

| MCP Server | Tools | Uso | Docker Image |
|------------|-------|-----|--------------|
| **playwright** | 22 | Browser automation, scraping, testing | `mcp/playwright` |
| **filesystem** | 5 | Read/write files, list dirs | `mcp/filesystem` |
| **postgres** | 4 | Query database (read-only!) | `mcp/postgres` |
| **fetch** | 1 | HTTP requests, APIs | `mcp/fetch` |
| **github** | 15+ | PRs, issues, code | `mcp/github` |
| **slack** | 8 | Mensagens, channels | `mcp/slack` |
| **gmail** | 5 | Email read/send | Community |
| **google-drive** | 6 | Docs, sheets | Community |
| **puppeteer** | 10 | Alt. browser automation | `mcp/puppeteer` |

**Tarefas**:
- [ ] Criar `McpGateway` struct no Office
- [ ] Config: `mcp_servers.toml` com lista de MCPs
- [ ] Spawn MCPs como processos Docker
- [ ] Proxy tool calls do Claude para MCPs
- [ ] Agregar tools de todos MCPs no prompt
- [ ] Health check e restart de MCPs

**Exemplo de config:**
```toml
# config/mcp_servers.toml

[[servers]]
name = "playwright"
image = "mcp/playwright"
enabled = true
capabilities = ["browser", "scraping", "testing"]

[[servers]]
name = "filesystem"
image = "mcp/filesystem"
enabled = true
mounts = ["/workspace:/workspace:ro"]
capabilities = ["files"]

[[servers]]
name = "postgres"
image = "mcp/postgres"
enabled = true
env = { DATABASE_URL = "${DATABASE_URL}" }
read_only = true  # IMPORTANTE: só SELECT!

[[servers]]
name = "github"
image = "mcp/github"
enabled = true
env = { GITHUB_TOKEN = "${GITHUB_TOKEN}" }
```

#### 2. Append-Only Event Store
**Problema**: Banco atual usa UPDATE/DELETE (mutável).  
**Solução**: Migrar para append-only com projections.

```
┌─────────────────────────────────────────────────────────┐
│  ARQUITETURA APPEND-ONLY                                │
│                                                         │
│  ┌─────────┐    ┌─────────────┐    ┌────────────────┐  │
│  │ Command │───>│ Event Store │───>│ Projections    │  │
│  │ (Write) │    │ (Append)    │    │ (Materialized) │  │
│  └─────────┘    └─────────────┘    └────────────────┘  │
│                       │                    │           │
│                       │                    ▼           │
│                       │            ┌──────────────┐    │
│                       └───────────>│ Read Models  │    │
│                                    └──────────────┘    │
└─────────────────────────────────────────────────────────┘
```

**Tarefas**:
- [ ] Criar tabela `events` (append-only, imutável)
- [ ] Schema: `event_id, stream_id, event_type, payload, created_at`
- [ ] Criar materialized views para queries rápidas
- [ ] Migrar `id_session`, `id_tenant`, `id_credential` para event-sourced
- [ ] Implementar snapshot strategy para performance

#### 2. Remover Mocks da UI
**Problema**: Dados fake hardcoded no frontend.

**Arquivos a limpar**:
- [ ] `apps/messenger/frontend/src/constants/index.ts` - INITIAL_ENTITIES, INITIAL_CONVERSATIONS, INITIAL_MESSAGES
- [ ] `ChatPage.tsx` - Remover fallback para dados mock
- [ ] `Sidebar.tsx` - Usar dados reais da API
- [ ] `WelcomeScreen.tsx` - Conectar com backend

**Arquivos a verificar**:
- [ ] Verificar todos os `isDemoMode` e remover lógica de fallback
- [ ] Remover `loginDemo()` do useAuth (ou manter apenas para dev)

---

### 🟠 P1 - Alta Prioridade (Próximas 2 Semanas)

#### 3. UI Polish
**Problemas identificados**:
- [ ] Header: Mostrar nome do tenant ao lado do avatar
- [ ] Profile: Mostrar "voulezvous.clube@gmail.com" com role (Owner)
- [ ] Sidebar: Espaçamento inconsistente
- [ ] Avatar: Usar iniciais ou foto real, não placeholder genérico
- [ ] Responsividade: Mobile não testado

**Melhorias de UX**:
- [ ] Loading states consistentes
- [ ] Empty states (sem conversas, sem mensagens)
- [ ] Error states com retry
- [ ] Toast notifications consistentes
- [ ] Skeleton loaders

#### 4. Office Tools (LLM Backend)
**Status atual**: Office roda mas não tem tools conectadas.

**Tools a implementar**:
```rust
// Ferramentas que o LLM pode usar
- [ ] read_document(doc_id) -> String
- [ ] search_documents(query) -> Vec<DocResult>
- [ ] send_email(to, subject, body) -> Result
- [ ] create_task(title, assignee) -> Task
- [ ] query_database(sql) -> QueryResult  // Read-only!
- [ ] call_api(endpoint, method, body) -> Response
```

**Arquitetura**:
```
┌──────────────┐     ┌──────────────┐     ┌─────────────┐
│  Messenger   │────>│    Office    │────>│  LLM API    │
│  (Frontend)  │     │  (Tools+RAG) │     │ (Anthropic) │
└──────────────┘     └──────────────┘     └─────────────┘
                            │
                     ┌──────┴──────┐
                     │   Tools     │
                     │ - Email     │
                     │ - Docs      │
                     │ - Tasks     │
                     └─────────────┘
```

#### 5. Office Observability
**Métricas necessárias**:
- [ ] `llm_requests_total` - Total de chamadas ao LLM
- [ ] `llm_latency_seconds` - Latência por request
- [ ] `llm_tokens_used` - Tokens consumidos (cost tracking)
- [ ] `tool_calls_total` - Uso de cada tool
- [ ] `tool_errors_total` - Erros por tool

**Tracing**:
- [ ] Trace ID propagado do Messenger → Office → LLM
- [ ] Spans para cada tool call
- [ ] Logs estruturados com contexto

**Dashboards Grafana**:
- [ ] Office Overview (requests, latency, errors)
- [ ] LLM Cost Dashboard (tokens por tenant)
- [ ] Tool Usage Dashboard

---

### 🟡 P2 - Média Prioridade (Próximo Mês)

#### 6. Email Integration (SMTP)
**Credenciais já no .env** - pronto para wiring.

- [ ] Serviço de email no Office
- [ ] Templates: Invite code, Welcome, Notifications
- [ ] Queue para envio assíncrono
- [ ] Retry com backoff

#### 7. Settings Page
- [ ] Editar perfil (nome, avatar)
- [ ] Editar tenant (nome) - apenas owner
- [ ] Gerar novos invite codes
- [ ] Listar membros do tenant
- [ ] Remover membros (owner only)

#### 8. Conversations Reais
- [ ] API para criar conversation
- [ ] API para enviar mensagem
- [ ] API para listar mensagens (paginated)
- [ ] WebSocket/SSE para real-time updates
- [ ] Typing indicators

#### 9. Jobs/Workflow System
- [ ] Cards de aprovação no chat
- [ ] Job state machine
- [ ] Audit trail de decisões
- [ ] Notificações de jobs pendentes

---

### 🟢 P3 - Backlog

#### 10. Security Hardening
- [ ] Rate limiting por tenant
- [ ] CSRF protection
- [ ] Input validation/sanitization
- [ ] SQL injection prevention (sqlx já ajuda)
- [ ] XSS prevention no frontend
- [ ] Audit log de ações sensíveis

#### 11. Performance
- [ ] Connection pooling otimizado
- [ ] Query optimization (EXPLAIN ANALYZE)
- [ ] CDN para assets estáticos
- [ ] Service worker para offline
- [ ] Lazy loading de componentes

#### 12. DevOps
- [ ] Docker Compose para produção
- [ ] CI/CD pipeline (GitHub Actions)
- [ ] Automated tests (unit, integration, e2e)
- [ ] Database migrations versioned
- [ ] Blue-green deployment

#### 13. Multi-tenant Isolation
- [ ] Row-level security no PostgreSQL
- [ ] Tenant ID em todas as queries
- [ ] Separate schemas por tenant (enterprise)
- [ ] Data export por tenant

---

## 🔌 Ecossistema MCP (Model Context Protocol)

> **Filosofia**: Não reimplementar. Orquestrar. O momentum está lá fora.

### MCPs Externos - Catálogo Curado

#### 🥇 Tier 1 - Essenciais (Integrar Primeiro)

| MCP | Maintainer | Tools | Por que é essencial |
|-----|------------|-------|---------------------|
| **playwright** | Microsoft | 22 | Browser automation completa. Scraping, testing, screenshots. |
| **filesystem** | Anthropic | 5 | Ler/escrever arquivos. Básico. |
| **fetch** | Anthropic | 1 | HTTP requests para qualquer API. |
| **postgres** | Community | 4 | Query database. Modo read-only! |
| **memory** | Anthropic | 4 | Knowledge graph persistente. |

#### 🥈 Tier 2 - Alta Utilidade

| MCP | Maintainer | Tools | Caso de uso |
|-----|------------|-------|-------------|
| **github** | GitHub | 15+ | PRs, issues, code review, commits |
| **slack** | Slack | 8 | Integração com workspace Slack |
| **gmail** | Community | 5 | Enviar/ler emails |
| **google-drive** | Community | 6 | Docs, Sheets, apresentações |
| **notion** | Community | 10+ | Docs, databases, wikis |
| **linear** | Community | 8 | Issue tracking, sprints |

#### 🥉 Tier 3 - Específicos

| MCP | Maintainer | Tools | Caso de uso |
|-----|------------|-------|-------------|
| **puppeteer** | Community | 10 | Alt. a Playwright |
| **brave-search** | Brave | 2 | Web search |
| **exa** | Exa | 3 | Semantic web search |
| **aws** | Community | 20+ | AWS services |
| **stripe** | Community | 10+ | Payments |
| **twilio** | Community | 5 | SMS, calls |
| **docker** | Docker | 8 | Container management |
| **kubernetes** | Community | 15+ | K8s management |

### Arquitetura MCP Gateway

```rust
// apps/office/src/mcp/gateway.rs

pub struct McpGateway {
    servers: HashMap<String, McpServerHandle>,
    tool_registry: ToolRegistry,
}

pub struct McpServerHandle {
    name: String,
    process: Child,            // Docker container
    stdin: ChildStdin,         // JSON-RPC in
    stdout: BufReader<ChildStdout>,  // JSON-RPC out
    tools: Vec<ToolDefinition>,
    status: ServerStatus,
}

impl McpGateway {
    /// Start all configured MCP servers
    pub async fn start_all(&mut self, config: &McpConfig) -> Result<()>;
    
    /// Get all available tools from all servers
    pub fn all_tools(&self) -> Vec<ToolDefinition>;
    
    /// Route tool call to correct MCP server
    pub async fn call_tool(&self, name: &str, params: Value) -> Result<ToolResult>;
    
    /// Health check all servers
    pub async fn health_check(&self) -> HealthReport;
}
```

### MCP Protocol (JSON-RPC 2.0)

```json
// Initialize
{"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"capabilities": {}}}

// List tools
{"jsonrpc": "2.0", "id": 2, "method": "tools/list"}

// Call tool
{"jsonrpc": "2.0", "id": 3, "method": "tools/call", "params": {
    "name": "browser_navigate",
    "arguments": {"url": "https://example.com"}
}}
```

### Docker Compose para MCPs

```yaml
# docker-compose.mcp.yml

version: '3.8'

services:
  playwright:
    image: mcp/playwright
    stdin_open: true
    networks:
      - mcp-internal

  filesystem:
    image: mcp/filesystem
    stdin_open: true
    volumes:
      - ./workspace:/workspace:ro
    networks:
      - mcp-internal

  postgres-mcp:
    image: mcp/postgres
    stdin_open: true
    environment:
      - DATABASE_URL=${DATABASE_URL}
      - READ_ONLY=true
    networks:
      - mcp-internal

  fetch:
    image: mcp/fetch
    stdin_open: true
    networks:
      - mcp-internal

networks:
  mcp-internal:
    driver: bridge
```

### Segurança MCP

| Concern | Mitigação |
|---------|-----------|
| File access | Mounts read-only, paths whitelisted |
| Database | READ_ONLY=true, só SELECT |
| Network | Internal network, no external access |
| Secrets | Env vars injetadas, não no config |
| Resource limits | Docker memory/cpu limits |
| Audit | Log todas as tool calls |

---

## 🏗️ Arquitetura Futura

### Event Sourcing Schema
```sql
-- Immutable event store
CREATE TABLE events (
    event_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    stream_id TEXT NOT NULL,  -- e.g., "tenant:abc123" or "user:xyz"
    stream_version BIGINT NOT NULL,
    event_type TEXT NOT NULL,  -- e.g., "TenantCreated", "MessageSent"
    payload JSONB NOT NULL,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    
    UNIQUE (stream_id, stream_version)
);

-- Only INSERT allowed, no UPDATE/DELETE
CREATE RULE events_immutable AS ON UPDATE TO events DO INSTEAD NOTHING;
CREATE RULE events_no_delete AS ON DELETE TO events DO INSTEAD NOTHING;

-- Projections (materialized from events)
CREATE MATERIALIZED VIEW tenant_summary AS
SELECT 
    payload->>'tenant_id' as tenant_id,
    payload->>'name' as name,
    COUNT(*) FILTER (WHERE event_type = 'MemberJoined') as member_count,
    MIN(created_at) as created_at
FROM events
WHERE stream_id LIKE 'tenant:%'
GROUP BY payload->>'tenant_id', payload->>'name';
```

### Office Tool Interface
```rust
#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters(&self) -> serde_json::Value;
    
    async fn execute(
        &self, 
        params: serde_json::Value,
        context: &ToolContext,
    ) -> Result<ToolOutput, ToolError>;
}

pub struct ToolContext {
    pub tenant_id: String,
    pub user_sid: String,
    pub trace_id: String,
}
```

---

## 📅 Timeline Sugerida

| Semana | Foco | Entregáveis |
|--------|------|-------------|
| 1 | **Foundation** | Append-only events + MCP Gateway básico |
| 1 | **Wiring** | Remover mocks, conectar UI ao backend real |
| 2 | **MCP Expansion** | Mais MCPs (github, postgres), verificar policy-vm |
| 3 | **Office Plan** | Spec implementada, tools nativas |
| 4 | **Observability** | Métricas de MCPs, tracing, dashboards |
| 5+ | **Polish** | Email, settings, jobs, security |

---

## 🔥 Lições da Análise Forense

### O Que Está PRONTO (Não Mexer)
- ✅ Identity/Auth - Maduro, refatorado várias vezes
- ✅ SQL Migrations - Schema versionado, bem estruturado
- ✅ Observability config - Pronto para produção
- ✅ LLM Providers - 4 providers funcionando
- ✅ Dreaming - Consolidação de memória, manter!
- ✅ Runner isolado - A limitação é a feature, não bug

### O Que FALTA (Foco Imediato)
- ❌ **MCP Gateway** - O cérebro precisa de mãos (ferramentas)
- ❌ **Append-only events** - Fundação para auditoria imutável
- ❌ **Wiring UI-Backend** - Remover a "mentira" dos mocks

### O Que QUESTIONAR (Decisões Pendentes)
- 🤔 **policy-vm** - Perguntar ao autor original sobre propósito
- 🤔 **Container specs** - 80 JSONs sem código que os usa (migrar para código?)

### Princípio Operacional
> **"Pare de desenhar o mapa e comece a construir as estradas."**
> 
> A fase de Arquitetura/Design está completa.
> Agora é Wiring/Integration agressiva.
>
> **MAS**: Respeitar a separação física (Runner) que é o diferencial do projeto.

---

## 🎯 Princípios MCP-First

1. **Não reimplemente** - Se existe MCP, use
2. **Docker always** - MCPs rodam em containers isolados
3. **Read-only default** - Database, filesystem = read-only até precisar
4. **Aggregate tools** - Claude vê todas as tools de todos os MCPs
5. **Log everything** - Cada tool call é um evento auditável
6. **Fail gracefully** - MCP down? Degrada, não quebra

---

## 🎯 Definition of Done

Para cada feature:
- [ ] Código implementado e testado
- [ ] Sem fallback para demo/mock
- [ ] Dados persistidos no PostgreSQL
- [ ] Métricas expostas para Prometheus
- [ ] Logs estruturados com trace_id
- [ ] Documentação atualizada

---

## 📝 Notas

### O que NÃO fazer
- ❌ Atalhos que criam dívida técnica
- ❌ Demo mode em produção
- ❌ UUIDs onde strings fazem mais sentido
- ❌ Mutable state onde events são melhor

### Princípios
- ✅ Append-only first
- ✅ Observability built-in
- ✅ Multi-tenant by design
- ✅ Real persistence, no mocks

---

*Última atualização: 2026-01-01*
*Autor: Copilot + Dan*
