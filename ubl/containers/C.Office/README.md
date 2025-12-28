![C.Office • * Preto (LLM Runtime)](https://img.shields.io/badge/C.Office-*%20Preto%20(LLM%20Runtime)-black)

# ⬛ C.Office — Container de Entidades LLM

**Path:** `containers/C.Office`  
**Role/Cor:** Preto (LLM Runtime)  
**Zona:** Fora das LABs (acesso controlado via Gateway)  

## Filosofia

> O Office é um órgão **subordinado** ao UBL.
> Pode legislar (via Constitution) mas apenas dentro dos limites que o UBL permite.
> 
> **Dever do Office:**
> - Implementar o que o UBL NÃO oferece (Context, Narrator, Sanity, Dreaming, etc.)
> - NUNCA reimplementar o que o UBL JÁ oferece (Ledger, Identity, Permits, etc.)

## Credenciais necessárias
- **ASC (Agent Service Credential)**: emitido pelo UBL para o Office
- **Passkey (ubl-id)**: para operações L3+ (via step-up)

## Função

Container para eventos de **Entidades LLM** e suas sessões:
- Criação e gestão de Entities (Chair)
- Sessões e Handovers
- Constitution e Baselines
- Audit trail de decisões

## Eventos Suportados

### Entity Events
| Event | Description | Intent Class | Δ |
|-------|-------------|--------------|---|
| `entity.created` | Nova entidade criada | Observation | 0 |
| `entity.activated` | Entidade ativada | Observation | 0 |
| `entity.suspended` | Entidade suspensa | Observation | 0 |
| `entity.archived` | Entidade arquivada | Observation | 0 |

### Constitution Events
| Event | Description | Intent Class | Δ |
|-------|-------------|--------------|---|
| `constitution.updated` | Regras comportamentais atualizadas | Observation | 0 |
| `baseline.updated` | Narrativa base atualizada (Dreaming) | Observation | 0 |

### Session Events
| Event | Description | Intent Class | Δ |
|-------|-------------|--------------|---|
| `session.started` | Sessão LLM iniciada | Observation | 0 |
| `session.completed` | Sessão concluída com handover | Observation | 0 |
| `session.tokens_used` | Tokens consumidos na sessão | Observation | 0 |

### Audit Events
| Event | Description | Intent Class | Δ |
|-------|-------------|--------------|---|
| `audit.tool_called` | LLM chamou uma tool | Observation | 0 |
| `audit.tool_result` | Resultado da tool | Observation | 0 |
| `audit.decision_made` | LLM tomou decisão | Observation | 0 |
| `audit.policy_violation` | Política violada | Observation | 0 |

## Entradas permitidas (Inbound)
- Messenger (via jobs)
- LLM providers (respostas)
- UBL SSE (eventos do ledger)

## Saídas permitidas (Outbound)
- `/v1/policy/permit` (pedir autorização)
- `/v1/commands/issue` (registar comando para Runner)
- `/link/commit` (gravar eventos)
- LLM providers (prompts)

## Mapa da Fronteira
```
              ┌───────────────────────────────────────────────────────────────┐
              │                        OFFICE                                  │
              │                                                                │
              │  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐    │
              │  │ Constitution │    │   Context    │    │  Governance  │    │
              │  │   Enforcer   │    │   Builder    │    │   (Sanity,   │    │
              │  │              │    │   + Narrator │    │   Dreaming)  │    │
              │  └──────────────┘    └──────────────┘    └──────────────┘    │
              │         │                   │                   │             │
              │         └───────────────────┼───────────────────┘             │
              │                             │                                  │
              │                      ┌──────┴──────┐                          │
              │                      │ Job Executor│                          │
              │                      └──────┬──────┘                          │
              │                             │                                  │
              └─────────────────────────────┼──────────────────────────────────┘
                                            │
                              ┌─────────────┼─────────────┐
                              │             │             │
                              ▼             ▼             ▼
                     ┌────────────┐  ┌────────────┐  ┌────────────┐
                     │ /v1/policy │  │ /link/     │  │ /v1/       │
                     │ /permit    │  │ commit     │  │ commands/  │
                     └────────────┘  └────────────┘  │ issue      │
                                                     └────────────┘
                                            │
                                            ▼
                                     ┌────────────┐
                                     │ UBL KERNEL │
                                     │  (Ledger)  │
                                     └────────────┘
```

## Módulos do Office

O Office implementa a **Universal Historical Specification**:

| Módulo | Função | UBL oferece? |
|--------|--------|--------------|
| `entity/` | Entity (Chair), Instance, Guardian | ❌ Não |
| `context/` | ContextFrame, Builder, Narrator, Memory | ❌ Não |
| `session/` | Session, Handover, Modes, TokenBudget | ❌ Não |
| `governance/` | Sanity, Constitution, Dreaming, Simulation | ❌ Não |
| `audit/` | AuditEvent, ToolAudit | ❌ Não |
| `job_executor/` | JobExecutor, ConversationContext | ❌ Não |
| `ubl_client/` | HTTP client para UBL Gateway | (usa UBL) |
| `middleware/` | Constitution Enforcer, Permit Middleware | (usa UBL) |

## Done if…
- [ ] Todos eventos entity.* são commitados para C.Office
- [ ] Projections `/query/office/entities` funcionam
- [ ] Handovers são armazenados e recuperáveis
- [ ] Constitution enforcement bloqueia violações
- [ ] Dreaming cycle consolida memória

