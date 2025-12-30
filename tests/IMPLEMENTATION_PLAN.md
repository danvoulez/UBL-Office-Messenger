# Plano de Implementação - Prompts 1, 2 e 3

## 📋 Visão Geral

Implementação em 9 etapas baseadas nos prompts:

### Etapa 1: Limpeza (Prompt 1 - Seção 0)
- [ ] Remover rotas HTTP do Runner
- [ ] Remover server.rs do Runner
- [ ] Remover runner_proxy.rs do Office (se existir)
- [ ] Remover sse_payload.rs do UBL Server

### Etapa 2: SSE Simplificado (Prompt 1 - Seção 1.1)
- [ ] Criar novo sse.rs que emite apenas `{sequence}`
- [ ] Integrar broadcast channel no append do ledger
- [ ] Testar SSE com apenas IDs

### Etapa 3: Office como Front-Door (Prompt 1 - Seção 2.1)
- [ ] Criar types.rs no Office
- [ ] Criar asc.rs (validação SID/ASC)
- [ ] Criar routes/ws.rs (ws/test, ws/build)
- [ ] Criar routes/deploy.rs
- [ ] Atualizar main.rs do Office

### Etapa 4: Runner Pull-Only (Prompt 1 - Seção 2.2)
- [ ] Converter Runner para pull-only (sem HTTP)
- [ ] Implementar handlers para ws/test, ws/build, deploy
- [ ] Implementar commit_observation

### Etapa 5: Schemas Canônicos (Prompt 1 - Seção 2.3)
- [ ] Criar contracts/ubl/atoms/*.schema.json
- [ ] Validar schemas JSON

### Etapa 6: CLI Atualizado (Prompt 1 - Seção 2.4)
- [ ] Criar commands/ws-test.ts
- [ ] Atualizar index.ts do CLI

### Etapa 7: Tipos TypeScript (Prompt 2 - Seção 1)
- [ ] Criar clients/types/ubl/atoms/*.d.ts
- [ ] Criar clients/types/ubl/index.d.ts

### Etapa 8: Testes de Integração (Prompt 2 - Seção 2-3)
- [ ] Criar lib.rs no Office
- [ ] Criar tests/office_integration.rs
- [ ] Adicionar dev-dependencies no Cargo.toml

### Etapa 9: Unix Socket (Prompt 3 - Opcional)
- [ ] Configurar Postgres via Unix Socket
- [ ] Adicionar axum-server ao UBL Server
- [ ] Criar http_unix.rs no Office
- [ ] Configurar Nginx e PM2

## 🎯 Ordem de Execução Recomendada

1. **Etapa 1** (Limpeza) - Base limpa
2. **Etapa 2** (SSE) - Funcionalidade core
3. **Etapa 3** (Office) - Front-door principal
4. **Etapa 4** (Runner) - Backend pull-only
5. **Etapa 5** (Schemas) - Contratos
6. **Etapa 6** (CLI) - Interface de usuário
7. **Etapa 7** (Types TS) - Tipos compartilhados
8. **Etapa 8** (Tests) - Validação
9. **Etapa 9** (Unix Socket) - Opcional, produção

## ✅ Status Atual

- [ ] Etapa 1: Limpeza
- [ ] Etapa 2: SSE Simplificado
- [ ] Etapa 3: Office Front-Door
- [ ] Etapa 4: Runner Pull-Only
- [ ] Etapa 5: Schemas
- [ ] Etapa 6: CLI
- [ ] Etapa 7: Types TS
- [ ] Etapa 8: Tests
- [ ] Etapa 9: Unix Socket

