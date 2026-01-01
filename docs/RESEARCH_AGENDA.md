# 🔬 Research Agenda - UBL 2026

> **Objetivo**: Validar decisões arquiteturais com pesquisa profunda antes de implementar.
> **Data**: 2026-01-01
> **Status**: Aguardando pesquisa

---

## 📋 Temas para Deep Research

### 1. 🔌 MCP (Model Context Protocol)

**O que já decidimos:**
- Office será um gateway/orquestrador de MCPs
- MCPs rodam em Docker containers

**Perguntas para pesquisar:**

| # | Pergunta | Por que importa |
|---|----------|-----------------|
| 1.1 | Qual a arquitetura ideal para um MCP Gateway/Hub? | Evitar reinventar a roda |
| 1.2 | Como fazer health check e restart de MCPs? | Resiliência |
| 1.3 | Existe padrão para agregação de tools de múltiplos MCPs? | Prompt management |
| 1.4 | Como lidar com rate limits e quotas entre MCPs? | Custos, throttling |
| 1.5 | Melhores práticas de segurança para MCPs (sandboxing)? | Produção segura |
| 1.6 | Streaming vs request/response para tool calls? | UX, latência |
| 1.7 | Quais MCPs são production-ready vs experimental? | Confiabilidade |
| 1.8 | MCP Registry/Discovery - existe padrão? | Extensibilidade |

**Recursos para pesquisar:**
- [ ] https://modelcontextprotocol.io (spec oficial)
- [ ] https://github.com/modelcontextprotocol (org oficial)
- [ ] Docker MCP Catalog
- [ ] Anthropic blog posts sobre MCP
- [ ] Implementações: Claude Desktop, Cursor, Continue.dev

---

### 2. 📜 Event Sourcing & CQRS

**O que já decidimos:**
- Banco deve ser append-only
- Queries por projections

**Perguntas para pesquisar:**

| # | Pergunta | Por que importa |
|---|----------|-----------------|
| 2.1 | Event Store: build own vs use existing (EventStoreDB, Marten)? | Build vs buy |
| 2.2 | PostgreSQL como event store - é viável em 2026? | Simplificar stack |
| 2.3 | Schema de eventos: JSON vs Protobuf vs Avro? | Performance, evolução |
| 2.4 | Snapshot strategy - quando e como? | Performance de replay |
| 2.5 | Event versioning e schema evolution? | Manutenção longo prazo |
| 2.6 | Projections: síncronas vs assíncronas? | Consistência vs latência |
| 2.7 | Outbox pattern para garantir delivery? | Reliability |
| 2.8 | CQRS com um banco vs múltiplos? | Complexidade operacional |

**Recursos para pesquisar:**
- [ ] Martin Fowler - Event Sourcing
- [ ] Greg Young - CQRS/ES talks
- [ ] EventStoreDB docs
- [ ] Marten (PostgreSQL ES para .NET - patterns aplicáveis)
- [ ] Axon Framework patterns
- [ ] "Designing Data-Intensive Applications" - Kleppmann

---

### 3. 🔐 Auth & Identity (WebAuthn/Passkeys)

**O que já funciona:**
- WebAuthn discoverable credentials ✅
- Sessões com SID string ✅
- Multi-tenant com invite codes ✅

**Perguntas para pesquisar:**

| # | Pergunta | Por que importa |
|---|----------|-----------------|
| 3.1 | PRF extension - quão suportado está em 2026? | Client-side signing |
| 3.2 | Passkey sync (iCloud, Google) - implicações? | UX, security model |
| 3.3 | Backup/recovery de passkeys - best practices? | Usuário perdeu device |
| 3.4 | Multi-device registration - flows recomendados? | Onboarding segundo device |
| 3.5 | Session management - JWT vs opaque tokens? | Stateless vs stateful |
| 3.6 | Refresh token rotation - patterns 2026? | Security vs UX |
| 3.7 | Step-up auth - quando exigir re-auth? | Ações sensíveis |
| 3.8 | FIDO Alliance guidelines atuais? | Compliance, best practices |

**Recursos para pesquisar:**
- [ ] WebAuthn spec (W3C)
- [ ] FIDO Alliance whitepapers
- [ ] passkeys.dev
- [ ] webauthn.io
- [ ] Apple/Google/Microsoft passkey docs

---

### 4. 🤖 LLM Integration Patterns

**O que já decidimos:**
- Claude via Anthropic API
- Tools via MCP

**Perguntas para pesquisar:**

| # | Pergunta | Por que importa |
|---|----------|-----------------|
| 4.1 | Streaming vs batch responses - tradeoffs 2026? | UX, custos |
| 4.2 | Context window management - strategies? | Custo, qualidade |
| 4.3 | Tool use: parallel vs sequential execution? | Latência |
| 4.4 | Caching de respostas LLM - quando faz sentido? | Custos |
| 4.5 | Fallback entre modelos (Claude ↔ GPT)? | Resiliência |
| 4.6 | Prompt versioning e A/B testing? | Melhoria contínua |
| 4.7 | Observability específica para LLM? | Debug, custos |
| 4.8 | Safety/guardrails - patterns atuais? | Produção responsável |

**Recursos para pesquisar:**
- [ ] Anthropic API docs e cookbooks
- [ ] LangChain/LangGraph patterns
- [ ] Instructor (structured outputs)
- [ ] Anthropic prompt engineering guide
- [ ] OpenAI best practices (aplicável)

---

### 5. 📊 Observability Stack

**O que temos:**
- Prometheus + Grafana (básico)
- Jaeger (tracing)
- Loki (logs)

**Perguntas para pesquisar:**

| # | Pergunta | Por que importa |
|---|----------|-----------------|
| 5.1 | OpenTelemetry - é o padrão definitivo em 2026? | Vendor lock-in |
| 5.2 | Tracing distribuído para LLM chains? | Debug de tool calls |
| 5.3 | Métricas de negócio vs técnicas - como separar? | Dashboards úteis |
| 5.4 | Alerting inteligente - patterns? | Reduzir noise |
| 5.5 | Cost monitoring para LLM/cloud? | Budget control |
| 5.6 | SLOs/SLIs - como definir para AI apps? | Reliability |

**Recursos para pesquisar:**
- [ ] OpenTelemetry docs
- [ ] Google SRE book (SLOs)
- [ ] Datadog/New Relic patterns (conceitos, não vendor)
- [ ] LangSmith/Langfuse (LLM observability específico)

---

### 6. 🎨 Frontend Architecture

**O que temos:**
- React + Vite
- Tailwind
- Framer Motion

**Perguntas para pesquisar:**

| # | Pergunta | Por que importa |
|---|----------|-----------------|
| 6.1 | React Server Components - aplicável? | Performance |
| 6.2 | State management 2026 - Zustand? Jotai? Context? | Simplicidade vs poder |
| 6.3 | Real-time UI - WebSocket vs SSE vs polling? | Chat, updates |
| 6.4 | Optimistic updates - patterns? | UX responsiva |
| 6.5 | Design system - build vs adopt (Radix, Shadcn)? | Velocidade |
| 6.6 | Accessibility - WCAG 2.2 requirements? | Compliance |
| 6.7 | Mobile: PWA vs React Native vs responsive? | Escopo |

**Recursos para pesquisar:**
- [ ] React docs (Server Components)
- [ ] Shadcn/ui (patterns)
- [ ] TanStack Query (data fetching)
- [ ] Vercel/Next.js patterns (aplicável a Vite)

---

### 7. 🏗️ Infrastructure & DevOps

**O que temos:**
- Docker Compose local
- PostgreSQL

**Perguntas para pesquisar:**

| # | Pergunta | Por que importa |
|---|----------|-----------------|
| 7.1 | Kubernetes vs simpler (Fly.io, Railway)? | Complexidade vs controle |
| 7.2 | Database: managed vs self-hosted? | Ops burden |
| 7.3 | CI/CD: GitHub Actions patterns 2026? | Automação |
| 7.4 | Feature flags - qual sistema? | Safe deploys |
| 7.5 | Secrets management - Vault vs cloud native? | Security |
| 7.6 | Multi-region - quando faz sentido? | Latência, compliance |

---

## 🎯 Priorização de Pesquisa

| Prioridade | Tema | Impacto | Urgência |
|------------|------|---------|----------|
| 🔴 1 | MCP Gateway | Alto - define arquitetura Office | Imediato |
| 🔴 2 | Event Sourcing | Alto - define modelo de dados | Imediato |
| 🟠 3 | LLM Patterns | Médio - já funciona básico | Semana 2 |
| 🟠 4 | Observability | Médio - produção | Semana 2 |
| 🟡 5 | Frontend | Baixo - funciona | Semana 3 |
| 🟡 6 | Auth (avançado) | Baixo - funciona | Quando precisar |
| 🟢 7 | Infra/DevOps | Baixo - local ok | Pré-produção |

---

## 📝 Template de Deep Research

Para cada tema, a pesquisa deve responder:

```markdown
## Tema: [Nome]

### 1. Estado da Arte (2026)
- O que é considerado best practice hoje?
- Quais são as ferramentas/libs dominantes?
- O que mudou nos últimos 12 meses?

### 2. Opções Viáveis
| Opção | Prós | Contras | Adoção |
|-------|------|---------|--------|
| A | ... | ... | Alta/Média/Baixa |
| B | ... | ... | Alta/Média/Baixa |

### 3. Recomendação
- Escolha: [X]
- Razão: ...
- Riscos: ...

### 4. Implementação de Referência
- Links para código/exemplos
- Libs específicas
- Configuração recomendada

### 5. O que NÃO fazer
- Anti-patterns identificados
- Erros comuns
```

---

## 🔗 Recursos Gerais

### Onde pesquisar
- **Hacker News** - discussões técnicas recentes
- **Reddit** (r/programming, r/rust, r/typescript)
- **GitHub Trending** - o que está crescendo
- **ThoughtWorks Tech Radar**
- **InfoQ** - arquitetura
- **Martin Fowler's blog**

### Ferramentas de Deep Research
- **Perplexity** - pesquisa com fontes
- **Claude** - análise profunda (nós!)
- **ChatGPT Deep Research** - se disponível
- **Google Scholar** - papers acadêmicos

---

## ✅ Próximos Passos

1. [ ] Escolher 1-2 temas prioritários para pesquisar primeiro
2. [ ] Fazer deep research (Perplexity, docs oficiais, exemplos)
3. [ ] Trazer findings para cá
4. [ ] Validar decisões juntos
5. [ ] Implementar com confiança

---

*"Measure twice, cut once."*

*Última atualização: 2026-01-01*
