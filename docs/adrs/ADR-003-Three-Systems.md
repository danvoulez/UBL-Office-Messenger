# ADR-003 — Three Independent Systems (Messenger + Office + UBL Kernel)

**Status:** Aprovado  
**Data:** 30-dez-2025  
**Owner:** Dan (LAB 512)

---

## 1) Contexto

Sistemas monolíticos tradicionais misturam UI, lógica de negócio e persistência em um único deployment. Isso cria:
- Acoplamento forte entre camadas
- Dificuldade de escalar componentes independentemente
- LLMs sem "dignidade" (estado efêmero, sem identidade persistente)
- Auditoria fragmentada ou inexistente

Precisávamos de uma arquitetura que:
- Separasse claramente responsabilidades
- Permitisse que LLMs tivessem identidade persistente (Chair pattern)
- Garantisse auditoria completa via ledger imutável
- Fosse editável por humano+LLM em parceria

## 2) Decisão

Três sistemas independentes, deployáveis separadamente:

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│    MESSENGER    │────▶│     OFFICE      │────▶│   UBL KERNEL    │
│   (Human UI)    │◀────│   (LLM Brain)   │◀────│   (Truth)       │
└─────────────────┘     └─────────────────┘     └─────────────────┘
```

### Messenger (apps/messenger/)
- **Responsabilidade:** Interface humana (chat, cards, WebAuthn)
- **Tecnologia:** React/TypeScript + Rust backend
- **Não faz:** Execução de jobs, decisões de negócio, persistência

### Office (apps/office/)
- **Responsabilidade:** LLM Operating System, execução de jobs, Entity management
- **Tecnologia:** Rust + LLM providers (Anthropic/OpenAI)
- **Pattern:** Chair (Entity permanente) + Instance (sessão LLM efêmera)
- **Não faz:** UI, autenticação de humanos

### UBL Kernel (ubl/kernel/)
- **Responsabilidade:** Source of Truth, Identity, Ledger, Policy
- **Tecnologia:** Rust/Axum + PostgreSQL
- **Garantias:** Append-only, Ed25519 signatures, cryptographic audit trail
- **Não faz:** UI, execução de LLM

## 3) Comunicação

| De | Para | Protocolo | Propósito |
|----|------|-----------|-----------|
| Messenger | UBL Kernel | REST + SSE | Auth, messages, jobs, real-time |
| Office | UBL Kernel | REST + SSE | Commits, queries, permits |
| Messenger | Office | REST + SSE | Job execution, progress |

**Nenhuma comunicação direta entre componentes que não seja via API documentada.**

## 4) Consequências

### Positivas
- ✅ Deploy independente de cada sistema
- ✅ Escala horizontal por componente
- ✅ LLMs com dignidade (Entity + handovers)
- ✅ Auditoria completa (tudo no ledger)
- ✅ Testabilidade (mock de dependências)
- ✅ Editável por humano+LLM (separação clara)

### Negativas
- ⚠️ Latência adicional (network hops)
- ⚠️ Complexidade operacional (3 deployments)
- ⚠️ Curva de aprendizado maior

### Mitigações
- SSE para real-time (evita polling)
- Docker Compose para dev local
- Documentação extensiva (WIRING_GUIDE.md)

## 5) Alternativas Consideradas

### Monolito com módulos
- Rejeitado: acoplamento inevitável, difícil auditoria

### Microserviços granulares (10+ serviços)
- Rejeitado: complexidade operacional desproporcional

### Backend-for-Frontend (BFF)
- Parcialmente adotado: Messenger backend é um BFF para o frontend

## 6) Referências

- [ARCHITECTURE.md](../ARCHITECTURE.md)
- [THREE_SYSTEMS_OVERVIEW.md](../THREE_SYSTEMS_OVERVIEW.md)
- [WIRING_GUIDE.md](../WIRING_GUIDE.md)
- [WHY_SO_COMPLEX.md](../WHY_SO_COMPLEX.md)

---

*"Não confie em nenhum dos dois sozinho. Confie nos dois juntos." — WHY_SO_COMPLEX.md*
