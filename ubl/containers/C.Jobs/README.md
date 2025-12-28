![C.Jobs • * Azul (Work Tracking)](https://img.shields.io/badge/C.Jobs-*%20Azul%20(Work%20Tracking)-blue)

# 🔵 C.Jobs — Você está aqui

**Path:** `containers/C.Jobs`  
**Role/Cor:** Azul (Work Tracking)  
**Zona:** LAB 256 (API) + 8GB (dev)  

## Credenciais necessárias
- **Passkey (ubl-id)**: usuário logado
- **LLM agent (escopo limitado)**: opcional para assinar como usuário

## Função
Container para tracking de jobs (tarefas) e aprovações. Gerencia o ciclo de vida completo de jobs:
- Criação de jobs
- Execução e progresso
- Aprovações necessárias
- Conclusão e resultados

## Entradas permitidas (Inbound)
- requests do cliente/serviços confiáveis (Messenger, Office)
- SSE do ledger

## Saídas permitidas (Outbound)
- kernel (signing_bytes/validate/commit)
- outros containers via LINKS (nunca por import)

## Dados que passam por aqui
- Drafts de jobs, Links, Eventos do ledger (job.*, approval.*)

## Eventos Suportados

### Job Events
- `job.created` - Novo job criado
- `job.started` - Job iniciado
- `job.progress` - Atualização de progresso
- `job.completed` - Job concluído
- `job.cancelled` - Job cancelado

### Approval Events
- `approval.requested` - Aprovação solicitada
- `approval.decided` - Decisão de aprovação tomada

## Intent Classes

| Event | Intent Class | Physics Delta |
|-------|-------------|---------------|
| `job.created` | Observation | 0 |
| `job.started` | Observation | 0 |
| `job.progress` | Observation | 0 |
| `job.completed` | Observation or Entropy | 0 or +value |
| `job.cancelled` | Observation | 0 |
| `approval.requested` | Observation | 0 |
| `approval.decided` | Observation | 0 |

## Policy
- **Risk Level**: L3 (jobs podem envolver dinheiro)
- **Trust Level**: L2-L3 (team member action, supervisor approval)

## Done if…
- README principal passa nos critérios de Done if e testes verdes.
- Todos eventos de job são commitados corretamente
- Queries funcionam via projections
- Aprovações seguem pacts definidos

## Dicas
- Nunca importe outro container; somente `@kernel/*` e tipos do OpenAPI.
- Jobs são commitados por Messenger ou Office, não diretamente por C.Jobs
- State é sempre derivado de projections

## Mapa da Fronteira
```
[local] --draft--> [boundary] --signing_bytes--> [ubl-link] --(signature)--> [membrane]
                                                            \--Accept--> [ledger] --tail--> [projections]
```

