# 🎯 SPEC: Office-Plan System

**Data:** 2026-01-01  
**Status:** PROPOSTA  
**Escopo:** Messenger + Office + UBL Kernel (harmônico)  
**Objetivo:** Sistema de planejamento que dá liberdade ao LLM e confiança ao usuário

---

## 🌐 VISÃO DOS 3 SISTEMAS

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                         OFFICE-PLAN: 3 SISTEMAS                                 │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                 │
│  ┌─────────────────┐      ┌─────────────────┐      ┌─────────────────┐         │
│  │   MESSENGER     │      │     OFFICE      │      │   UBL KERNEL    │         │
│  │   (Frontend)    │      │   (Cérebro)     │      │   (Verdade)     │         │
│  │   :3000         │      │   :8081         │      │   :8080         │         │
│  ├─────────────────┤      ├─────────────────┤      ├─────────────────┤         │
│  │                 │      │                 │      │                 │         │
│  │ • Renderiza     │      │ • Cria planos   │      │ • Persiste      │         │
│  │   cards         │      │ • Executa LLM   │      │   eventos       │         │
│  │ • Captura       │      │ • Gerencia      │      │ • Garante       │         │
│  │   interações    │      │   progresso     │      │   imutabilidade │         │
│  │ • Mostra        │      │ • Orquestra     │      │ • Projeta       │         │
│  │   progresso     │      │   tools         │      │   estado        │         │
│  │                 │      │                 │      │                 │         │
│  └────────┬────────┘      └────────┬────────┘      └────────┬────────┘         │
│           │                        │                        │                   │
│           │    WebSocket           │    HTTP + Events       │                   │
│           │◄──────────────────────►│◄──────────────────────►│                   │
│           │    (cards, progress)   │    (commits, queries)  │                   │
│                                                                                 │
└─────────────────────────────────────────────────────────────────────────────────┘
```

---

## 📦 RESPONSABILIDADES POR SISTEMA

### 1. MESSENGER (Frontend React)

**Responsabilidade:** UX do plano - mostrar, interagir, atualizar em tempo real

| Componente | Arquivo Atual | O que Adicionar |
|------------|---------------|-----------------|
| PlanCard | `components/` (novo) | Renderiza OfficePlan como card interativo |
| PlanProgress | `components/` (novo) | Barra de progresso + lista de items |
| PlanActions | `components/` (novo) | Botões: Aprovar, Ajustar, Pausar, Cancelar |
| WebSocket handler | `hooks/useWebSocket.ts` | Receber `plan.updated` events |

**Novo componente: PlanCard**
```tsx
interface PlanCardProps {
  plan: OfficePlan;
  stage: 'formalize' | 'tracking' | 'finished';
  onApprove: () => void;
  onAdjust: (changes: string) => void;
  onPause: () => void;
}

// Renderiza:
// - Goals com acceptance criteria
// - Items com status (✅ 🔄 ⬚)
// - Items descobertos pelo LLM marcados como "🆕"
// - Scope changes pendentes com Approve/Reject
// - Progresso geral (X/Y items, Z%)
```

**WebSocket Events que Messenger escuta:**
```typescript
type PlanEvent = 
  | { type: 'plan.created'; plan: OfficePlan }
  | { type: 'plan.item.updated'; item_id: string; status: ItemStatus }
  | { type: 'plan.item.discovered'; item: PlanItem }
  | { type: 'plan.scope_change.requested'; change: ScopeChange }
  | { type: 'plan.progress'; percent: number; current_item: string }
  | { type: 'plan.completed'; summary: PlanSummary }
  | { type: 'plan.paused'; reason: string }
  | { type: 'plan.error'; error: string };
```

---

### 2. OFFICE (Backend Rust - Cérebro)

**Responsabilidade:** Criar, gerenciar e executar planos

| Componente | Arquivo | Função |
|------------|---------|--------|
| OfficePlan | `plan/types.rs` (novo) | Estrutura do plano |
| PlanBuilder | `plan/builder.rs` (novo) | Cria plano a partir do pedido |
| PlanExecutor | `plan/executor.rs` (novo) | Executa items do plano |
| PlanEvents | `plan/events.rs` (novo) | Eventos UBL para o plano |
| NativeTools | `mcp/native_server.rs` | `plan_get`, `plan_update` |

**Estruturas Core:**

```rust
// plan/types.rs

/// Um plano de trabalho
pub struct OfficePlan {
    pub id: String,
    pub job_id: String,
    pub conversation_id: String,
    pub created_at: DateTime<Utc>,
    
    /// Pedido original do usuário (imutável)
    pub original_request: String,
    
    /// Goals (o contrato com o usuário)
    pub goals: Vec<PlanGoal>,
    
    /// Items de trabalho (mutável pelo LLM)
    pub items: Vec<PlanItem>,
    
    /// Status geral
    pub status: PlanStatus,
    
    /// Mudanças de escopo pendentes
    pub pending_scope_changes: Vec<ScopeChange>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PlanGoal {
    pub id: String,
    pub description: String,
    pub acceptance_criteria: Vec<String>,
    pub status: GoalStatus,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PlanItem {
    pub id: String,
    pub goal_id: String,
    pub title: String,
    pub status: ItemStatus,
    pub added_by: AddedBy,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub output: Option<String>,  // O que foi produzido
    pub children: Vec<String>,   // Sub-items
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum ItemStatus {
    Todo,
    Doing,
    Done,
    Blocked,
    Skipped,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum AddedBy {
    Original,      // Veio do plano aprovado
    LlmDiscovered, // LLM descobriu durante execução
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ScopeChange {
    pub id: String,
    pub change_type: ScopeChangeType,
    pub reason: String,
    pub impact: String,
    pub requested_at: DateTime<Utc>,
    pub decision: Option<ScopeDecision>,
}
```

**Ferramentas Nativas para o LLM:**

```rust
// Ferramenta: plan_get
// LLM usa para ver o plano atual
{
    "name": "plan_get",
    "description": "Get the current plan with all goals and items",
    "input_schema": {
        "type": "object",
        "properties": {
            "plan_id": { "type": "string" }
        },
        "required": ["plan_id"]
    }
}

// Ferramenta: plan_update
// LLM usa para atualizar progresso e adicionar items
{
    "name": "plan_update",
    "description": "Update plan: mark items done, add discovered items, request scope changes",
    "input_schema": {
        "type": "object",
        "properties": {
            "plan_id": { "type": "string" },
            "updates": {
                "type": "array",
                "items": {
                    "oneOf": [
                        {
                            "type": "object",
                            "properties": {
                                "action": { "const": "mark_done" },
                                "item_id": { "type": "string" },
                                "output": { "type": "string" }
                            }
                        },
                        {
                            "type": "object", 
                            "properties": {
                                "action": { "const": "add_item" },
                                "goal_id": { "type": "string" },
                                "title": { "type": "string" },
                                "parent_item_id": { "type": "string" }
                            }
                        },
                        {
                            "type": "object",
                            "properties": {
                                "action": { "const": "mark_blocked" },
                                "item_id": { "type": "string" },
                                "reason": { "type": "string" }
                            }
                        },
                        {
                            "type": "object",
                            "properties": {
                                "action": { "const": "request_scope_change" },
                                "change_type": { "type": "string" },
                                "reason": { "type": "string" },
                                "impact": { "type": "string" }
                            }
                        }
                    ]
                }
            }
        },
        "required": ["plan_id", "updates"]
    }
}
```

---

### 3. UBL KERNEL (Backend Rust - Verdade)

**Responsabilidade:** Persistir eventos do plano, projetar estado, garantir auditoria

| Componente | Arquivo | Função |
|------------|---------|--------|
| Plan Events Schema | `contracts/plan_event.schema.json` (novo) | Schema dos eventos |
| Plan Projection | `projections/` (novo) | Projetar estado do plano |
| Container | containers | Um container por job/plan |

**Eventos UBL (imutáveis):**

```json
// plan.created
{
  "type": "plan.created",
  "plan_id": "plan_abc123",
  "job_id": "job_xyz789",
  "original_request": "Refatora o auth pra ficar mais modular",
  "goals": [
    {
      "id": "goal_1",
      "description": "Organizar código em módulos",
      "acceptance_criteria": ["identity/ existe", "config centralizado"]
    }
  ],
  "items": [
    { "id": "item_1", "goal_id": "goal_1", "title": "Criar identity/config.rs" }
  ],
  "timestamp": "2026-01-01T12:00:00Z"
}

// plan.approved
{
  "type": "plan.approved",
  "plan_id": "plan_abc123",
  "approved_by": "ubl:sid:dan123...",
  "timestamp": "2026-01-01T12:01:00Z"
}

// plan.item.started
{
  "type": "plan.item.started",
  "plan_id": "plan_abc123",
  "item_id": "item_1",
  "timestamp": "2026-01-01T12:02:00Z"
}

// plan.item.completed
{
  "type": "plan.item.completed",
  "plan_id": "plan_abc123",
  "item_id": "item_1",
  "output": "Created file with 42 lines",
  "timestamp": "2026-01-01T12:05:00Z"
}

// plan.item.discovered
{
  "type": "plan.item.discovered",
  "plan_id": "plan_abc123",
  "item": {
    "id": "item_5",
    "goal_id": "goal_1",
    "title": "Criar identity/types.rs",
    "added_by": "llm_discovered"
  },
  "reason": "Encontrei tipos duplicados que precisam de home",
  "timestamp": "2026-01-01T12:06:00Z"
}

// plan.scope_change.requested
{
  "type": "plan.scope_change.requested",
  "plan_id": "plan_abc123",
  "change": {
    "id": "scope_1",
    "change_type": "add_goal",
    "reason": "Descobri que também precisa de refatorar o frontend",
    "impact": "Adiciona 3-4 items, +2h estimado"
  },
  "timestamp": "2026-01-01T12:10:00Z"
}

// plan.scope_change.decided
{
  "type": "plan.scope_change.decided",
  "plan_id": "plan_abc123",
  "change_id": "scope_1",
  "approved": true,
  "decided_by": "ubl:sid:dan123...",
  "timestamp": "2026-01-01T12:11:00Z"
}

// plan.completed
{
  "type": "plan.completed",
  "plan_id": "plan_abc123",
  "summary": {
    "goals_completed": 3,
    "items_original": 6,
    "items_discovered": 2,
    "items_completed": 8,
    "items_skipped": 0,
    "duration_seconds": 1200
  },
  "timestamp": "2026-01-01T12:20:00Z"
}
```

**Projeção de Estado:**

```rust
// UBL projeta o estado atual do plano a partir dos eventos
pub struct PlanProjection {
    pub plan_id: String,
    pub status: PlanStatus,
    pub goals: Vec<ProjectedGoal>,
    pub items: Vec<ProjectedItem>,
    pub progress_percent: u8,
    pub current_item: Option<String>,
}

// Query endpoint
GET /containers/{container_id}/projections/plan
→ Retorna PlanProjection atual
```

---

## 🔄 FLUXO COMPLETO (3 SISTEMAS)

```
┌──────────────────────────────────────────────────────────────────────────────────┐
│                              FLUXO HARMÔNICO                                     │
├──────────────────────────────────────────────────────────────────────────────────┤
│                                                                                  │
│  1. USUÁRIO PEDE                                                                 │
│  ──────────────────                                                              │
│  Messenger → Office: "Refatora o auth"                                           │
│                                                                                  │
│  2. OFFICE CRIA PLANO                                                            │
│  ──────────────────────                                                          │
│  Office (PlanBuilder):                                                           │
│    - Analisa pedido                                                              │
│    - Cria OfficePlan com goals e items                                           │
│    - Gera FormalizeCard                                                          │
│                                                                                  │
│  Office → UBL: commit { type: "plan.created", ... }                              │
│  Office → Messenger (WS): { type: "plan.created", plan }                         │
│                                                                                  │
│  3. USUÁRIO APROVA                                                               │
│  ───────────────────                                                             │
│  Messenger → Office: POST /plan/{id}/approve                                     │
│  Office → UBL: commit { type: "plan.approved", ... }                             │
│  Office → Messenger (WS): { type: "plan.approved" }                              │
│                                                                                  │
│  4. OFFICE EXECUTA                                                               │
│  ───────────────────                                                             │
│  Para cada item:                                                                 │
│    Office → UBL: commit { type: "plan.item.started" }                            │
│    Office → Messenger (WS): { type: "plan.progress", current_item }              │
│                                                                                  │
│    Office (LLM + Tools):                                                         │
│      - Executa item                                                              │
│      - Pode usar plan_update para:                                               │
│        - Marcar done                                                             │
│        - Adicionar sub-items descobertos                                         │
│        - Pedir scope change                                                      │
│                                                                                  │
│    Office → UBL: commit { type: "plan.item.completed" }                          │
│    Office → Messenger (WS): { type: "plan.item.updated" }                        │
│                                                                                  │
│  5. SE LLM DESCOBRE ALGO                                                         │
│  ─────────────────────────                                                       │
│  LLM usa tool: plan_update({ action: "add_item", ... })                          │
│  Office → UBL: commit { type: "plan.item.discovered" }                           │
│  Office → Messenger (WS): { type: "plan.item.discovered" }                       │
│  (Usuário vê "🆕" no item, mas não precisa aprovar)                              │
│                                                                                  │
│  6. SE LLM QUER MUDAR ESCOPO                                                     │
│  ─────────────────────────────                                                   │
│  LLM usa tool: plan_update({ action: "request_scope_change", ... })              │
│  Office → UBL: commit { type: "plan.scope_change.requested" }                    │
│  Office → Messenger (WS): { type: "plan.scope_change.requested" }                │
│  (Usuário vê modal: "LLM quer adicionar X. Aprovar?")                            │
│                                                                                  │
│  Messenger → Office: POST /plan/{id}/scope/{change_id}/decide                    │
│  Office → UBL: commit { type: "plan.scope_change.decided" }                      │
│                                                                                  │
│  7. PLANO COMPLETA                                                               │
│  ───────────────────                                                             │
│  Office → UBL: commit { type: "plan.completed", summary }                        │
│  Office → Messenger (WS): { type: "plan.completed" }                             │
│  Messenger: Mostra FinishedCard com resumo                                       │
│                                                                                  │
└──────────────────────────────────────────────────────────────────────────────────┘
```

---

## 📡 CONTRATOS ENTRE SISTEMAS

### Messenger ↔ Office

**WebSocket (Office → Messenger):**
```typescript
// Connection
ws://office:8081/ws?token={session_token}

// Events Office envia:
interface PlanCreatedEvent {
  type: 'plan.created';
  plan: OfficePlan;
  card: FormalizeCard;
}

interface PlanProgressEvent {
  type: 'plan.progress';
  plan_id: string;
  percent: number;
  current_item: string;
  items_done: number;
  items_total: number;
}

interface PlanItemUpdatedEvent {
  type: 'plan.item.updated';
  plan_id: string;
  item: PlanItem;
}

interface PlanScopeChangeEvent {
  type: 'plan.scope_change.requested';
  plan_id: string;
  change: ScopeChange;
}
```

**HTTP (Messenger → Office):**
```
POST /plan/{plan_id}/approve
POST /plan/{plan_id}/reject
POST /plan/{plan_id}/adjust
  Body: { adjustments: string }

POST /plan/{plan_id}/pause
POST /plan/{plan_id}/resume
POST /plan/{plan_id}/cancel

POST /plan/{plan_id}/scope/{change_id}/decide
  Body: { approved: boolean, comment?: string }

GET /plan/{plan_id}
  → OfficePlan
```

### Office ↔ UBL Kernel

**Commits (Office → UBL):**
```rust
// Office usa UblClient para commitar eventos
ubl_client.commit(LinkCommit {
    container_id: plan.container_id,
    intent_class: "observation",  // Planos são observações
    atom_hash: hash_of(plan_event),
    // ...
})
```

**Queries (Office → UBL):**
```
GET /containers/{id}/projections/plan
  → PlanProjection

GET /containers/{id}/events?type=plan.*
  → Vec<PlanEvent>
```

---

## 🗂️ ARQUIVOS A CRIAR

### Messenger (TypeScript)
```
apps/messenger/src/
├── components/
│   ├── plan/
│   │   ├── PlanCard.tsx           # Card principal do plano
│   │   ├── PlanProgress.tsx       # Barra + lista de items
│   │   ├── PlanGoal.tsx           # Renderiza um goal
│   │   ├── PlanItem.tsx           # Renderiza um item
│   │   ├── ScopeChangeModal.tsx   # Modal para aprovar scope change
│   │   └── index.ts
├── hooks/
│   └── usePlan.ts                 # Hook para gerenciar estado do plano
├── types/
│   └── plan.ts                    # Tipos TypeScript do plano
```

### Office (Rust)
```
apps/office/src/
├── plan/
│   ├── mod.rs                     # Re-exports
│   ├── types.rs                   # OfficePlan, PlanItem, etc.
│   ├── builder.rs                 # PlanBuilder - cria plano do pedido
│   ├── executor.rs                # PlanExecutor - executa items
│   ├── events.rs                  # Eventos UBL
│   └── tools.rs                   # plan_get, plan_update handlers
├── mcp/
│   └── native_server.rs           # Adicionar plan tools
├── api/
│   └── plan.rs                    # HTTP endpoints
```

### UBL Kernel (Rust)
```
ubl/kernel/rust/ubl-server/src/
├── projections/
│   ├── mod.rs
│   └── plan.rs                    # PlanProjection

contracts/
└── plan_event.schema.json         # Schema dos eventos
```

---

## 📊 MÉTRICAS DE SUCESSO

| Métrica | Objetivo |
|---------|----------|
| Tempo até aprovação | Usuário aprova plano em <30s |
| Confiança do usuário | >90% dos planos aprovados sem ajuste |
| Flexibilidade LLM | LLM adiciona items descobertos sem travar |
| Visibilidade | Usuário sempre sabe o que está acontecendo |
| Auditoria | 100% das ações no UBL |
| Intervenção | Usuário pode pausar/ajustar a qualquer momento |

---

## 🚀 FASES DE IMPLEMENTAÇÃO

### Fase 1: Fundação (3 dias)
**Baixo risco, estruturas core**

| Sistema | O que fazer |
|---------|-------------|
| Office | Criar `plan/types.rs`, `plan/events.rs` |
| UBL | Criar `contracts/plan_event.schema.json` |
| Messenger | Criar `types/plan.ts` |

### Fase 2: Criação de Plano (2 dias)
**Office cria plano, Messenger mostra**

| Sistema | O que fazer |
|---------|-------------|
| Office | Criar `plan/builder.rs`, endpoint `POST /plan` |
| Messenger | Criar `PlanCard.tsx` para FormalizeCard |

### Fase 3: Execução (3 dias)
**Office executa, progresso em tempo real**

| Sistema | O que fazer |
|---------|-------------|
| Office | Criar `plan/executor.rs`, WS events |
| Office | Implementar `plan_get`, `plan_update` tools |
| Messenger | Criar `PlanProgress.tsx`, `usePlan.ts` |
| UBL | Criar projeção básica |

### Fase 4: Scope Changes (2 dias)
**LLM pede mudanças, usuário decide**

| Sistema | O que fazer |
|---------|-------------|
| Office | Lógica de scope change |
| Messenger | `ScopeChangeModal.tsx` |
| UBL | Eventos de scope change |

### Fase 5: Polish (2 dias)
**Testes, edge cases, UX refinements**

---

## ✅ PRÓXIMOS PASSOS

1. [ ] Aprovar esta spec
2. [ ] Criar branch `feature/office-plan`
3. [ ] Implementar Fase 1 em paralelo nos 3 sistemas
4. [ ] Testar integração Messenger ↔ Office
5. [ ] Testar integração Office ↔ UBL
6. [ ] E2E test completo
7. [ ] Deploy

---

## 📝 DECISÕES TOMADAS

| Decisão | Escolha | Razão |
|---------|---------|-------|
| LLM pode adicionar items? | Sim, sem aprovação | Liberdade para trabalhar |
| LLM pode mudar goals? | Precisa aprovação | Goals são o contrato |
| Onde vive o estado? | UBL (eventos) + Office (cache) | Verdade no UBL, performance no Office |
| Formato de eventos | JSON imutável | Compatível com UBL existente |
| WebSocket ou polling? | WebSocket | Tempo real é essencial |
