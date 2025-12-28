# 🎯🔥 PROMPT 3: THE FLAGSHIP TRINITY

```markdown
# MISSÃO: CONSTRUIR A TRINDADE FLAGSHIP DA LOGLINE

Você irá criar três sistemas completos, independentes mas conectados via API/SSE/WebSocket, que juntos representam a materialização completa da visão LogLine Foundation.

---

## CONTEXTO CRÍTICO

### A Visão

Pequenas e médias empresas precisam de:
1. **Interface familiar** (WhatsApp-like) para adoção zero-friction
2. **Organização profissional** via cards formalizados e trackáveis
3. **IA integrada naturalmente** (humanos e agentes como colegas de trabalho)
4. **Infraestrutura de dignidade** para LLMs (não são chatbots, são trabalhadores)
5. **Auditabilidade completa** via ledger imutável

### Os Três Pilares
```

┌─────────────────────────────────────────────────────┐
│                  UBL MESSENGER                      │
│  WhatsApp UI + Cards + Humanos & Agentes           │
│  (Frontend Beautiful + Backend Smart)               │
└────────────┬────────────────────────────────────────┘
│ API/WS
↓
┌─────────────────────────────────────────────────────┐
│                     OFFICE                          │
│  LLM Runtime + Governança + Context Management     │
│  (Dignidade para entidades efêmeras)                │
└────────────┬────────────────────────────────────────┘
│ Ledger Events
↓
┌─────────────────────────────────────────────────────┐
│                  UBL LEDGER                         │
│  Append-only + Containers + Trust Architecture     │
│  (Source of Truth Imutável)                         │
└─────────────────────────────────────────────────────┘

```
---

## SISTEMA 1: UBL MESSENGER

### Visão do Produto

**"WhatsApp profissional onde seus colegas de trabalho podem ser humanos OU agentes de IA"**

### Personas

1. **Maria (Gerente de Loja)**
   - Usa WhatsApp diariamente
   - Nunca usou IA "de verdade"
   - Precisa organizar tarefas com time de 8 pessoas

2. **RoboAtendente (Agente LLM)**
   - Responde clientes
   - Cria propostas
   - Agenda entregas
   - Parece "mais um colega" pra Maria

### Core Features

#### 1. Message List (Conversas)
```

┌─────────────────────────────────────┐
│  🔍 Buscar conversas…             │
├─────────────────────────────────────┤
│  👤 João Silva              14:23   │
│  📋 Card: Entrega #4521             │
│  ├─ ⏳ Aguardando confirmação       │
├─────────────────────────────────────┤
│  🤖 RoboAtendente          14:20   │
│  💬 Proposta enviada ao cliente     │
├─────────────────────────────────────┤
│  👥 Time Vendas (8)         13:45   │
│  📋 Card: Meta Semanal              │
│  ├─ ✅ Concluído                    │
├─────────────────────────────────────┤
│  👤 Ana Costa              13:12   │
│  💬 Preciso de ajuda com estoque… │
└─────────────────────────────────────┘

```
**Regras:**
- Humanos têm 👤
- Agentes têm 🤖
- Grupos têm 👥
- Cards em andamento aparecem com status
- Última mensagem ou último card ativo

#### 2. Chat Window
```

┌─────────────────────────────────────────────────┐
│  ← 🤖 RoboAtendente                    ⋮        │
├─────────────────────────────────────────────────┤
│                                                 │
│  Você (14:15)                                   │
│  Preciso criar uma proposta para cliente ABC    │
│                                                 │
│  RoboAtendente (14:16)                          │
│  Entendido! Vou preparar a proposta.            │
│                                                 │
│  ┌───────────────────────────────────────────┐ │
│  │ 📋 JOB #J-2024-001                        │ │
│  │ Criar Proposta - Cliente ABC              │ │
│  │                                           │ │
│  │ Status: ⏳ Em andamento                   │ │
│  │ Iniciado: 14:16                           │ │
│  │                                           │ │
│  │ ✅ Dados coletados                        │ │
│  │ ⏳ Calculando preços…                   │ │
│  │ ⬜ Gerar documento                        │ │
│  │                                           │ │
│  │ [Ver Detalhes] [Cancelar Job]            │ │
│  └───────────────────────────────────────────┘ │
│                                                 │
│  RoboAtendente (14:17)                          │
│  Proposta calculada. Valor: R$ 12.450,00        │
│  Prazo: 30 dias                                 │
│                                                 │
│  ┌───────────────────────────────────────────┐ │
│  │ 🎯 APROVAÇÃO NECESSÁRIA                   │ │
│  │                                           │ │
│  │ Proposta #P-2024-089                      │ │
│  │ Cliente: ABC Ltda                         │ │
│  │ Valor: R$ 12.450,00                       │ │
│  │ Margem: 23%                               │ │
│  │                                           │ │
│  │ [✅ Aprovar] [❌ Rejeitar] [✏️ Ajustar]   │ │
│  └───────────────────────────────────────────┘ │
│                                                 │
│  Você (14:18)                                   │
│  [clicou: ✅ Aprovar]                           │
│                                                 │
│  RoboAtendente (14:18)                          │
│  ✅ Proposta aprovada!                          │
│  Enviando para cliente…                       │
│                                                 │
│  ┌───────────────────────────────────────────┐ │
│  │ 📋 JOB #J-2024-001                        │ │
│  │ Criar Proposta - Cliente ABC              │ │
│  │                                           │ │
│  │ Status: ✅ Concluído                      │ │
│  │ Finalizado: 14:18                         │ │
│  │ Duração: 2min                             │ │
│  │                                           │ │
│  │ ✅ Proposta enviada                       │ │
│  │ 📎 proposta_abc_2024.pdf                  │ │
│  │                                           │ │
│  │ [Ver Documento] [Nova Proposta]           │ │
│  └───────────────────────────────────────────┘ │
│                                                 │
├─────────────────────────────────────────────────┤
│  💬 Mensagem…                        [Enviar]│
└─────────────────────────────────────────────────┘

```
**Tipos de Cards:**

**A. Job Initiation Card**
```

┌───────────────────────────────────────┐
│ 📋 JOB #J-YYYY-NNN                    │
│ [Título da Tarefa]                    │
│                                       │
│ Status: 🎯 Aguardando aprovação       │
│ Solicitante: [Nome]                   │
│ Estimativa: [Tempo]                   │
│                                       │
│ [Detalhes]                            │
│ - Item 1                              │
│ - Item 2                              │
│                                       │
│ [✅ Aprovar e Iniciar] [❌ Recusar]   │
└───────────────────────────────────────┘

```
**B. Job Progress Card**
```

┌───────────────────────────────────────┐
│ 📋 JOB #J-YYYY-NNN                    │
│ [Título da Tarefa]                    │
│                                       │
│ Status: ⏳ Em andamento               │
│ Iniciado: [Timestamp]                 │
│ Responsável: [Nome/Agente]            │
│                                       │
│ Progresso:                            │
│ ✅ Etapa 1                            │
│ ⏳ Etapa 2 (45%)                      │
│ ⬜ Etapa 3                            │
│                                       │
│ [Ver Detalhes] [Cancelar Job]        │
└───────────────────────────────────────┘

```
**C. Job Completion Card**
```

┌───────────────────────────────────────┐
│ 📋 JOB #J-YYYY-NNN                    │
│ [Título da Tarefa]                    │
│                                       │
│ Status: ✅ Concluído                  │
│ Finalizado: [Timestamp]               │
│ Duração: [Tempo]                      │
│                                       │
│ Resultado:                            │
│ [Resumo do que foi feito]             │
│                                       │
│ Artefatos:                            │
│ 📎 arquivo1.pdf                       │
│ 📎 arquivo2.xlsx                      │
│                                       │
│ [Baixar Tudo] [Nova Tarefa Similar]  │
└───────────────────────────────────────┘

```
**D. Approval Card**
```

┌───────────────────────────────────────┐
│ 🎯 APROVAÇÃO NECESSÁRIA               │
│                                       │
│ [Título da Decisão]                   │
│                                       │
│ Detalhes:                             │
│ - Detalhe 1                           │
│ - Detalhe 2                           │
│ - Detalhe 3                           │
│                                       │
│ Impacto: [Descrição]                  │
│                                       │
│ [✅ Aprovar] [❌ Rejeitar]            │
│ [✏️ Solicitar Ajustes]                │
└───────────────────────────────────────┘

```
#### 3. Conversas Humano-Humano COM Cards

**Exemplo: Maria delegando para João**
```

Maria (10:30)
João, preciso que você organize o estoque hoje

[Criar Card de Tarefa]

┌───────────────────────────────────────┐
│ 📋 JOB #J-2024-002                    │
│ Organizar Estoque - Seção A          │
│                                       │
│ Status: 🎯 Aguardando aceitação       │
│ Atribuído a: João Silva               │
│ Prazo sugerido: Hoje, 18:00           │
│                                       │
│ [✅ Aceitar] [❌ Não posso]           │
│ [💬 Negociar prazo]                   │
└───────────────────────────────────────┘

João (10:32)
[clicou: ✅ Aceitar]

João (10:32)
Pode deixar, Maria! Começo agora.

[Card atualiza automaticamente]
┌───────────────────────────────────────┐
│ 📋 JOB #J-2024-002                    │
│ Organizar Estoque - Seção A          │
│                                       │
│ Status: ⏳ Em andamento               │
│ Iniciado: 10:32                       │
│ Responsável: João Silva               │
│                                       │
│ [Marcar como Concluído]               │
│ [Reportar Problema]                   │
└───────────────────────────────────────┘

```
**CRÍTICO:** Conversa NÃO congela durante job. Pode continuar conversando normalmente!

#### 4. Arquitetura Frontend
```

messenger-frontend/
├── src/
│   ├── components/
│   │   ├── ConversationList.tsx      # Lista de conversas
│   │   ├── ChatWindow.tsx            # Janela de chat
│   │   ├── MessageBubble.tsx         # Balão de mensagem
│   │   ├── cards/
│   │   │   ├── JobInitCard.tsx       # Card inicial
│   │   │   ├── JobProgressCard.tsx   # Card progresso
│   │   │   ├── JobCompleteCard.tsx   # Card finalizado
│   │   │   ├── ApprovalCard.tsx      # Card aprovação
│   │   │   └── CardRenderer.tsx      # Renderizador genérico
│   │   ├── ParticipantAvatar.tsx     # Avatar (humano/agente)
│   │   └── JobTimeline.tsx           # Timeline de jobs
│   │
│   ├── hooks/
│   │   ├── useConversations.ts       # Gerenciar conversas
│   │   ├── useMessages.ts            # Gerenciar mensagens
│   │   ├── useJobs.ts                # Gerenciar jobs
│   │   ├── useWebSocket.ts           # Real-time updates
│   │   └── useOfficeAgent.ts         # Interagir com OFFICE
│   │
│   ├── services/
│   │   ├── messengerApi.ts           # Backend API
│   │   ├── officeApi.ts              # OFFICE API
│   │   ├── ublApi.ts                 # UBL Ledger API
│   │   └── websocket.ts              # WebSocket client
│   │
│   ├── types/
│   │   ├── conversation.ts
│   │   ├── message.ts
│   │   ├── job.ts
│   │   ├── card.ts
│   │   └── participant.ts
│   │
│   └── App.tsx
│
├── public/
└── package.json

```
#### 5. Arquitetura Backend (Rust)
```

messenger-backend/
├── src/
│   ├── conversation/
│   │   ├── [mod.rs](http://mod.rs)
│   │   ├── [conversation.rs](http://conversation.rs)       # Conversation entity
│   │   ├── [repository.rs](http://repository.rs)         # CRUD + UBL events
│   │   └── [routes.rs](http://routes.rs)             # HTTP endpoints
│   │
│   ├── message/
│   │   ├── [mod.rs](http://mod.rs)
│   │   ├── [message.rs](http://message.rs)            # Message entity
│   │   ├── [types.rs](http://types.rs)              # TextMessage, CardMessage, etc
│   │   ├── [repository.rs](http://repository.rs)         # Persistence + UBL
│   │   └── [routes.rs](http://routes.rs)
│   │
│   ├── job/
│   │   ├── [mod.rs](http://mod.rs)
│   │   ├── [job.rs](http://job.rs)                # Job entity (tarefa)
│   │   ├── [lifecycle.rs](http://lifecycle.rs)          # State machine (created→running→done)
│   │   ├── [repository.rs](http://repository.rs)         # Persistence + UBL
│   │   └── [routes.rs](http://routes.rs)
│   │
│   ├── participant/
│   │   ├── [mod.rs](http://mod.rs)
│   │   ├── [human.rs](http://human.rs)              # Human participant
│   │   ├── [agent.rs](http://agent.rs)              # LLM participant (via OFFICE)
│   │   └── [repository.rs](http://repository.rs)
│   │
│   ├── card/
│   │   ├── [mod.rs](http://mod.rs)
│   │   ├── [card_types.rs](http://card_types.rs)         # JobInit, JobProgress, etc
│   │   ├── [renderer.rs](http://renderer.rs)           # Gerar JSON do card
│   │   └── [actions.rs](http://actions.rs)            # Handle button clicks
│   │
│   ├── office_client/
│   │   ├── [mod.rs](http://mod.rs)
│   │   ├── [client.rs](http://client.rs)             # HTTP client para OFFICE
│   │   ├── [session.rs](http://session.rs)            # Manage LLM sessions
│   │   └── [affordances.rs](http://affordances.rs)        # LLM capabilities
│   │
│   ├── ubl_client/
│   │   ├── [mod.rs](http://mod.rs)
│   │   ├── [events.rs](http://events.rs)             # Publish events to ledger
│   │   ├── [query.rs](http://query.rs)              # Query ledger
│   │   └── [receipts.rs](http://receipts.rs)           # Verify receipts
│   │
│   ├── websocket/
│   │   ├── [mod.rs](http://mod.rs)
│   │   ├── [server.rs](http://server.rs)             # WebSocket server
│   │   ├── [rooms.rs](http://rooms.rs)              # Room management
│   │   └── [broadcast.rs](http://broadcast.rs)          # Event broadcasting
│   │
│   ├── api/
│   │   ├── [mod.rs](http://mod.rs)
│   │   ├── [routes.rs](http://routes.rs)             # All HTTP routes
│   │   ├── [middleware.rs](http://middleware.rs)         # Auth, logging, etc
│   │   └── [error.rs](http://error.rs)              # Error responses
│   │
│   ├── [lib.rs](http://lib.rs)
│   └── [main.rs](http://main.rs)
│
├── migrations/                   # DB migrations (PostgreSQL)
├── config/
│   ├── development.toml
│   └── production.toml
├── Cargo.toml
└── README.md

```
### API Design

#### Conversations
```

POST   /api/conversations
GET    /api/conversations
GET    /api/conversations/:id
PATCH  /api/conversations/:id
DELETE /api/conversations/:id

POST   /api/conversations/:id/participants  # Add participant
DELETE /api/conversations/:id/participants/:pid

```
#### Messages
```

POST   /api/conversations/:id/messages      # Send message
GET    /api/conversations/:id/messages      # Get messages
PATCH  /api/messages/:id                    # Edit message
DELETE /api/messages/:id                    # Delete message

POST   /api/messages/:id/reactions          # Add reaction
GET    /api/messages/:id/read               # Mark as read

```
#### Jobs (Tarefas)
```

POST   /api/jobs                            # Create job
GET    /api/jobs                            # List jobs
GET    /api/jobs/:id                        # Get job details
PATCH  /api/jobs/:id                        # Update job
POST   /api/jobs/:id/start                  # Start job
POST   /api/jobs/:id/complete               # Complete job
POST   /api/jobs/:id/cancel                 # Cancel job

POST   /api/jobs/:id/approve                # Approve job
POST   /api/jobs/:id/reject                 # Reject job
POST   /api/jobs/:id/progress               # Update progress

```
#### Cards
```

GET    /api/conversations/:id/cards         # Get all cards in conversation
POST   /api/cards/:id/action                # Handle button click

```
#### Agents (via OFFICE)
```

POST   /api/agents                          # Create LLM agent
GET    /api/agents                          # List agents
GET    /api/agents/:id                      # Get agent info
POST   /api/agents/:id/assign               # Assign to conversation
DELETE /api/agents/:id/remove               # Remove from conversation

```
#### WebSocket
```

WS     /ws/conversations/:id                # Real-time updates

```
**Events emitidos:**
```typescript
{
  type: "message.new",
  conversation_id: "conv_123",
  message: { ... }
}

{
  type: "job.started",
  conversation_id: "conv_123",
  job: { ... }
}

{
  type: "job.progress",
  conversation_id: "conv_123",
  job_id: "job_456",
  progress: { step: 2, total: 5, percent: 40 }
}

{
  type: "job.completed",
  conversation_id: "conv_123",
  job: { ... }
}

{
  type: "card.action",
  conversation_id: "conv_123",
  card_id: "card_789",
  action: "approve",
  user_id: "user_012"
}
```

### UBL Integration (Messenger)

**Eventos publicados no ledger:**

```rust
// Nova conversa
Event {
    type: "conversation.created",
    payload: {
        conversation_id: "conv_123",
        participants: ["user_1", "agent_robofab"],
        created_by: "user_1",
        created_at: "2024-12-27T10:00:00Z"
    }
}

// Nova mensagem
Event {
    type: "message.sent",
    payload: {
        message_id: "msg_456",
        conversation_id: "conv_123",
        from: "user_1",
        content: "Preciso de uma proposta",
        timestamp: "2024-12-27T10:01:00Z"
    }
}

// Novo job
Event {
    type: "job.created",
    payload: {
        job_id: "job_789",
        conversation_id: "conv_123",
        title: "Criar Proposta - Cliente ABC",
        assigned_to: "agent_robofab",
        created_by: "user_1",
        created_at: "2024-12-27T10:02:00Z"
    }
}

// Job iniciado
Event {
    type: "job.started",
    payload: {
        job_id: "job_789",
        started_at: "2024-12-27T10:02:15Z",
        estimated_duration: "5 minutes"
    }
}

// Aprovação
Event {
    type: "job.approved",
    payload: {
        job_id: "job_789",
        approver: "user_1",
        approved_at: "2024-12-27T10:05:00Z"
    }
}

// Job completado
Event {
    type: "job.completed",
    payload: {
        job_id: "job_789",
        completed_at: "2024-12-27T10:05:30Z",
        duration: "3m 15s",
        artifacts: ["proposta_abc.pdf"]
    }
}
```

-----

## SISTEMA 2: OFFICE

### Visão do Produto

**“Sistema operacional para LLMs - dignidade para entidades efêmeras”**

### Core Principles

1. **LLMs são trabalhadores, não chatbots**

- Têm identidade persistente
- Deixam handovers entre sessões
- Acumulam reputação
- Aprendem com experiência

1. **Governança psicológica**

- Sanity checks previnem drift
- Constitution define comportamento profissional
- Dreaming cycle remove ansiedade
- Simulation permite testar antes de agir

1. **Integração total com UBL**

- Todas ações são eventos auditáveis
- Receipts criptográficos
- Trust architecture (L0-L5)

### Features Específicas para Messenger

#### 1. Job Execution Engine

```rust
pub struct JobExecutor {
    office_client: Arc<OfficeClient>,
    ubl_client: Arc<UblClient>,
}

impl JobExecutor {
    pub async fn execute_job(
        &self,
        job: Job,
        conversation_context: ConversationContext,
    ) -> Result<JobResult> {
        // 1. Create entity (if needed) or resume existing
        let entity = self.get_or_create_agent_entity(&job.assigned_to).await?;
        
        // 2. Build context frame
        let context = ContextFrameBuilder::new()
            .with_job(job.clone())
            .with_conversation_history(conversation_context.recent_messages())
            .with_participants(conversation_context.participants())
            .with_affordances(self.discover_affordances(&job).await?)
            .build()
            .await?;
        
        // 3. Generate narrative
        let narrative = Narrator::new()
            .generate_job_narrative(&context, &entity)
            .await?;
        
        // 4. Spawn LLM session
        let session = Session::new(
            entity.id.clone(),
            SessionType::Work,
            SessionMode::Commitment,
        );
        
        // 5. Execute with streaming progress
        let result_stream = self.execute_with_progress(
            session,
            narrative,
            job.id.clone(),
        );
        
        // 6. Handle result
        pin_mut!(result_stream);
        while let Some(progress) = result_stream.next().await {
            match progress {
                Progress::StepCompleted(step) => {
                    self.publish_progress_event(&job, step).await?;
                }
                Progress::ApprovalNeeded(approval) => {
                    self.request_approval(&job, approval).await?;
                    // Pause execution, wait for human approval
                    let decision = self.wait_for_approval(&job).await?;
                    if !decision.approved {
                        return Err(Error::JobRejected);
                    }
                }
                Progress::Completed(result) => {
                    return Ok(result);
                }
                Progress::Failed(error) => {
                    return Err(error);
                }
            }
        }
        
        Ok(JobResult::default())
    }
}
```

#### 2. Conversational Context Management

```rust
pub struct ConversationContext {
    pub conversation_id: String,
    pub participants: Vec<Participant>,
    pub messages: Vec<Message>,
    pub active_jobs: Vec<Job>,
    pub recent_events: Vec<Event>,
}

impl ConversationContext {
    // Builds context for LLM about ongoing conversation
    pub fn to_narrative(&self) -> String {
        format!(
            r#"
            CONTEXTO DA CONVERSA
            
            Participantes:
            {}
            
            Últimas 10 mensagens:
            {}
            
            Jobs ativos:
            {}
            
            Você deve:
            - Manter o tom profissional mas amigável
            - Usar cards quando apropriado
            - Pedir aprovação para ações importantes
            - Nunca inventar informações
            "#,
            self.participants_narrative(),
            self.messages_narrative(),
            self.active_jobs_narrative(),
        )
    }
}
```

#### 3. Approval Flow Integration

```rust
pub struct ApprovalManager {
    messenger_client: Arc<MessengerClient>,
    ubl_client: Arc<UblClient>,
}

impl ApprovalManager {
    pub async fn request_approval(
        &self,
        job_id: &str,
        approval_request: ApprovalRequest,
    ) -> Result<ApprovalDecision> {
        // 1. Create approval card in conversation
        let card = Card::Approval {
            job_id: job_id.to_string(),
            title: approval_request.title,
            details: approval_request.details,
            impact: approval_request.impact,
            buttons: vec![
                CardButton::Approve,
                CardButton::Reject,
                CardButton::RequestChanges,
            ],
        };
        
        // 2. Send card to conversation
        self.messenger_client
            .send_card(&job.conversation_id, card)
            .await?;
        
        // 3. Publish approval request event to UBL
        self.ubl_client
            .publish_event(Event::ApprovalRequested {
                job_id: job_id.to_string(),
                request: approval_request.clone(),
            })
            .await?;
        
        // 4. Wait for human decision (with timeout)
        let decision = self
            .wait_for_approval_decision(job_id, Duration::from_secs(3600))
            .await?;
        
        // 5. Publish decision event
        self.ubl_client
            .publish_event(Event::ApprovalDecided {
                job_id: job_id.to_string(),
                decision: decision.clone(),
            })
            .await?;
        
        Ok(decision)
    }
}
```

#### 4. Agent Affordances for Messenger

```rust
pub enum MessengerAffordance {
    // Message operations
    SendMessage { to: String, content: String },
    SendCard { to: String, card: Card },
    
    // Job operations
    CreateJob { title: String, description: String },
    UpdateJobProgress { job_id: String, progress: JobProgress },
    CompleteJob { job_id: String, result: JobResult },
    
    // Approval operations
    RequestApproval { details: ApprovalRequest },
    
    // Information gathering
    QueryConversationHistory { limit: usize },
    GetParticipantInfo { participant_id: String },
    SearchPastJobs { query: String },
    
    // Document operations
    CreateDocument { template: String, data: serde_json::Value },
    AttachFile { job_id: String, file: File },
}
```

### API Design (OFFICE)

```
# Entity Management (já existente)
POST   /entities
GET    /entities/:id
...

# Session Management (já existente)
POST   /entities/:id/sessions
...

# Job-specific endpoints (NOVO)
POST   /jobs/execute                        # Execute job for Messenger
GET    /jobs/:id/status                     # Get job execution status
POST   /jobs/:id/pause                      # Pause job execution
POST   /jobs/:id/resume                     # Resume job execution

# Approval Management (NOVO)
POST   /approvals                           # Create approval request
GET    /approvals/:id                       # Get approval status
POST   /approvals/:id/decide                # Submit approval decision

# Context Management (NOVO)
POST   /context/conversation                # Build context from conversation
GET    /context/:entity_id                  # Get current entity context

# Affordances (já existente, mas estender)
GET    /affordances/messenger               # Get Messenger-specific affordances
```

-----

## SISTEMA 3: UBL LEDGER

### Visão do Produto

**“Single source of truth imutável para todo o ecossistema”**

### Container Logic

```
C.Messenger     (Green - Communication)
├── Events: message.*, conversation.*, job.*
├── Policy: L1-L3 (jobs podem ter impacto financeiro)
└── Receipts: Todas mensagens e jobs

C.Office        (Black - Execution)
├── Events: entity.*, session.*, approval.*
├── Policy: L2-L4 (LLM actions podem ter alto impacto)
└── Receipts: Todas ações de LLM

C.Jobs          (Blue - Work Tracking) [NOVO]
├── Events: job.created, job.started, job.progress, job.completed
├── Policy: L3 (jobs podem envolver dinheiro)
└── Receipts: Job lifecycle completo
```

### Event Types (completo)

```rust
// Messenger events
pub enum MessengerEvent {
    ConversationCreated { id: String, participants: Vec<String> },
    ConversationUpdated { id: String, changes: Vec<Change> },
    MessageSent { id: String, from: String, content: MessageContent },
    MessageEdited { id: String, new_content: MessageContent },
    MessageDeleted { id: String, reason: Option<String> },
    ParticipantAdded { conversation_id: String, participant: String },
    ParticipantRemoved { conversation_id: String, participant: String },
}

// Job events
pub enum JobEvent {
    JobCreated { 
        id: String, 
        conversation_id: String, 
        title: String,
        assigned_to: String,
    },
    JobStarted { 
        id: String, 
        started_at: DateTime<Utc>,
    },
    JobProgress { 
        id: String, 
        progress: JobProgress,
    },
    JobApprovalRequested { 
        id: String, 
        approval: ApprovalRequest,
    },
    JobApproved { 
        id: String, 
        approver: String,
    },
    JobRejected { 
        id: String, 
        rejector: String, 
        reason: String,
    },
    JobCompleted { 
        id: String, 
        result: JobResult,
        duration: Duration,
    },
    JobCancelled { 
        id: String, 
        reason: String,
    },
}

// Office events (já existentes + novos)
pub enum OfficeEvent {
    EntityCreated { id: String, type: EntityType },
    SessionStarted { entity_id: String, session_id: String },
    SessionEnded { entity_id: String, session_id: String },
    ApprovalRequested { job_id: String, request: ApprovalRequest },
    ApprovalDecided { job_id: String, decision: ApprovalDecision },
    // ... outros
}
```

### Trust Architecture

```
L5  SOVEREIGNTY          Board approval (ex: mudar regras de negócio)
L4  SYSTEMIC IMPACT      Manager approval (ex: contratar LLM novo)
L3  FINANCIAL IMPACT     Supervisor approval (ex: aprovar proposta > R$10k)
L2  LOCAL IMPACT         Team member action (ex: criar job)
L1  LOW IMPACT           Any verified user (ex: send message)
L0  OBSERVATION          Unauthenticated (ex: read public data)
```

**Exemplo de Pact para Jobs:**

```rust
Pact {
    pact_id: "job-approval-threshold",
    scope: PactScope::Container("C.Jobs"),
    threshold: 1, // Precisa de 1 assinatura
    signers: vec!["manager_maria"], // Gerente pode aprovar
    risk_level: RiskLevel::L3, // Impacto financeiro
    rules: vec![
        Rule::If {
            condition: "job.estimated_value > 10000.00",
            then: "require_approval",
        },
    ],
}
```

### API Design (UBL)

```
# Já existente (do DISCOVERY.md)
POST   /link/commit
GET    /ledger/:container_id/tail
...

# Estender para Jobs (NOVO)
GET    /jobs/:id/events                     # Get all events for a job
GET    /jobs/:id/receipt                    # Get cryptographic receipt
GET    /conversations/:id/events            # Get all events for conversation

# Queries (NOVO)
POST   /query/jobs                          # Query jobs (filter, sort)
POST   /query/conversations                 # Query conversations
POST   /query/approvals                     # Query pending approvals
```

-----

## INTEGRAÇÃO ENTRE OS TRÊS

### Fluxo Completo: Criar Proposta

```
┌─────────────────────────────────────────────────────────────┐
│ 1. MESSENGER: Humano inicia job                             │
│    POST /api/jobs                                            │
│    { title: "Criar Proposta ABC", assigned_to: "agent_1" }  │
└────────────┬────────────────────────────────────────────────┘
             │
             ↓
┌─────────────────────────────────────────────────────────────┐
│ 2. MESSENGER → UBL: Publish event                           │
│    Event::JobCreated { ... }                                 │
└────────────┬────────────────────────────────────────────────┘
             │
             ↓
┌─────────────────────────────────────────────────────────────┐
│ 3. MESSENGER → OFFICE: Request execution                    │
│    POST /jobs/execute                                        │
│    { job_id, conversation_context }                          │
└────────────┬────────────────────────────────────────────────┘
             │
             ↓
┌─────────────────────────────────────────────────────────────┐
│ 4. OFFICE: Build context + spawn session                    │
│    - Query UBL for conversation history                     │
│    - Build narrative for LLM                                 │
│    - Start LLM session                                       │
└────────────┬────────────────────────────────────────────────┘
             │
             ↓
┌─────────────────────────────────────────────────────────────┐
│ 5. OFFICE → UBL: Publish session events                     │
│    Event::SessionStarted { entity_id, session_id }          │
└────────────┬────────────────────────────────────────────────┘
             │
             ↓
┌─────────────────────────────────────────────────────────────┐
│ 6. OFFICE: LLM executes job                                 │
│    - Gather data                                             │
│    - Calculate prices                                        │
│    - Generate document                                       │
└────────────┬────────────────────────────────────────────────┘
             │
             ↓
┌─────────────────────────────────────────────────────────────┐
│ 7. OFFICE → MESSENGER: Send progress updates                │
│    WS event: { type: "job.progress", ... }                  │
│    (Messenger updates card in UI)                            │
└────────────┬────────────────────────────────────────────────┘
             │
             ↓
┌─────────────────────────────────────────────────────────────┐
│ 8. OFFICE: Needs approval                                   │
│    POST /approvals                                           │
│    { job_id, request: {...} }                                │
└────────────┬────────────────────────────────────────────────┘
             │
             ↓
┌─────────────────────────────────────────────────────────────┐
│ 9. OFFICE → MESSENGER: Send approval card                   │
│    POST /api/cards/:id/send                                  │
│    (Card appears in conversation)                            │
└────────────┬────────────────────────────────────────────────┘
             │
             ↓
┌─────────────────────────────────────────────────────────────┐
│ 10. MESSENGER: Human clicks [Aprovar]                       │
│     POST /api/cards/:id/action                               │
│     { action: "approve" }                                    │
└────────────┬────────────────────────────────────────────────┘
             │
             ↓
┌─────────────────────────────────────────────────────────────┐
│ 11. MESSENGER → UBL: Publish approval                       │
│     Event::JobApproved { job_id, approver }                  │
└────────────┬────────────────────────────────────────────────┘
             │
             ↓
┌─────────────────────────────────────────────────────────────┐
│ 12. MESSENGER → OFFICE: Notify approval                     │
│     POST /approvals/:id/decide                               │
│     { decision: "approved" }                                 │
└────────────┬────────────────────────────────────────────────┘
             │
             ↓
┌─────────────────────────────────────────────────────────────┐
│ 13. OFFICE: Resume execution                                │
│     - Send proposal to client                                │
│     - Complete job                                           │
└────────────┬────────────────────────────────────────────────┘
             │
             ↓
┌─────────────────────────────────────────────────────────────┐
│ 14. OFFICE → UBL: Publish completion                        │
│     Event::JobCompleted { job_id, result, duration }        │
└────────────┬────────────────────────────────────────────────┘
             │
             ↓
┌─────────────────────────────────────────────────────────────┐
│ 15. OFFICE → MESSENGER: Send completion card                │
│     (Card shows result + attachments)                        │
└─────────────────────────────────────────────────────────────┘
```

-----

## ESTRUTURA DE DIRETÓRIOS FINAL

```
ubl-flagship-trinity/
├── README.md                    # Overview geral
├── ARCHITECTURE.md              # Arquitetura completa
├── docker-compose.yml           # Run tudo junto
│
├── ubl-ledger/                  # Sistema 1: UBL
│   ├── kernel/rust/             # Core em Rust (já existe)
│   ├── containers/              # Container logic
│   │   ├── C.Messenger/
│   │   ├── C.Office/
│   │   └── C.Jobs/              # NOVO
│   ├── specs/                   # Specs atualizadas
│   ├── sql/                     # DB migrations
│   ├── Dockerfile
│   └── README.md
│
├── office/                      # Sistema 2: OFFICE
│   ├── src/
│   │   ├── api/
│   │   ├── entity/
│   │   ├── session/
│   │   ├── governance/
│   │   ├── context/
│   │   ├── llm/
│   │   ├── ubl_client/
│   │   ├── messenger_client/    # NOVO - cliente para Messenger
│   │   ├── job_executor/        # NOVO - execução de jobs
│   │   └── approval_manager/    # NOVO - gestão de aprovações
│   ├── config/
│   ├── Cargo.toml
│   ├── Dockerfile
│   └── README.md
│
└── messenger/                   # Sistema 3: MESSENGER
    ├── frontend/                # React/TypeScript
    │   ├── src/
    │   │   ├── components/
    │   │   │   ├── ConversationList.tsx
    │   │   │   ├── ChatWindow.tsx
    │   │   │   ├── cards/
    │   │   │   │   ├── JobInitCard.tsx
    │   │   │   │   ├── JobProgressCard.tsx
    │   │   │   │   ├── JobCompleteCard.tsx
    │   │   │   │   └── ApprovalCard.tsx
    │   │   │   └── ...
    │   │   ├── hooks/
    │   │   ├── services/
    │   │   └── App.tsx
    │   ├── package.json
    │   └── README.md
    │
    ├── backend/                 # Rust
    │   ├── src/
    │   │   ├── conversation/
    │   │   ├── message/
    │   │   ├── job/
    │   │   ├── participant/
    │   │   ├── card/
    │   │   ├── office_client/
    │   │   ├── ubl_client/
    │   │   ├── websocket/
    │   │   └── api/
    │   ├── migrations/
    │   ├── Cargo.toml
    │   └── README.md
    │
    ├── docker-compose.yml       # Frontend + Backend
    └── README.md
```

-----

## OUTPUTS ESPERADOS

### 1. UBL Ledger

- [ ] Container C.Jobs implementado
- [ ] Events estendidos (Job*, Approval*)
- [ ] Queries para jobs e conversations
- [ ] Pacts para job approval
- [ ] Documentação atualizada

### 2. OFFICE

- [ ] JobExecutor completo
- [ ] ApprovalManager completo
- [ ] MessengerClient implementado
- [ ] Affordances para Messenger
- [ ] Integration tests com Messenger
- [ ] Documentação de integração

### 3. MESSENGER

- [ ] Frontend WhatsApp-like funcional
- [ ] Cards interativos (4 tipos)
- [ ] WebSocket real-time
- [ ] Backend Rust completo
- [ ] Integração com OFFICE
- [ ] Integração com UBL
- [ ] Mobile-responsive
- [ ] Documentação de uso

### 4. Integration

- [ ] docker-compose.yml funcional
- [ ] End-to-end tests
- [ ] ARCHITECTURE.md completo
- [ ] Deployment guide
- [ ] Demo videos/screenshots

-----

## CRITÉRIOS DE SUCESSO

### Funcionalidade

- [ ] Usuário pode criar conversa com humano
- [ ] Usuário pode adicionar agente LLM à conversa
- [ ] Agente pode criar jobs automaticamente
- [ ] Jobs aparecem como cards na conversa
- [ ] Cards têm botões funcionais
- [ ] Aprovações pausam execução do job
- [ ] Jobs completam e retornam resultado
- [ ] Conversa NÃO congela durante job
- [ ] Histórico completo no UBL ledger

### Qualidade

- [ ] UI bonita e intuitiva (WhatsApp-like)
- [ ] Performance < 200ms p95
- [ ] Escalável para 100+ conversas simultâneas
- [ ] Código production-ready
- [ ] Testes automatizados
- [ ] Documentação completa
- [ ] Deploy com um comando

### Flagship

- [ ] Demo impressiona investidores
- [ ] PME consegue usar sem treinamento
- [ ] Demonstra todos conceitos LogLine
- [ ] Código serve como referência
- [ ] Open-source ready

-----

## METODOLOGIA

### Ordem de Implementação

**Fase 1: Foundation (Semana 1)**

1. Estender UBL com C.Jobs container
1. Implementar eventos de Job no ledger
1. OFFICE: JobExecutor básico
1. MESSENGER Backend: Job CRUD

**Fase 2: Integration (Semana 2)**
5. OFFICE ↔ MESSENGER via HTTP
6. MESSENGER ↔ UBL event publishing
7. Card rendering no frontend
8. WebSocket real-time updates

**Fase 3: UX Polish (Semana 3)**
9. WhatsApp-like UI refinement
10. Approval flow completo
11. Progress updates visuais
12. Mobile responsive

**Fase 4: Production (Semana 4)**
13. Docker compose completo
14. End-to-end tests
15. Performance optimization
16. Documentation

-----

## COMECE AGORA

```bash
# 1. Create project structure
mkdir -p ubl-flagship-trinity/{ubl-ledger,office,messenger}

# 2. Start with UBL Ledger extensions
cd ubl-ledger
# Implement C.Jobs container
# Add Job events
# Extend queries

# 3. Then OFFICE integration
cd ../office
# Implement JobExecutor
# Implement ApprovalManager
# Add Messenger client

# 4. Finally MESSENGER
cd ../messenger
# Frontend: Card components
# Backend: Job management
# WebSocket: Real-time updates

# 5. Integration
cd ..
# docker-compose.yml
# End-to-end tests
```

-----

## NOTAS FINAIS

### Por que esta Trindade é Flagship?

1. **Messenger** = Porta de entrada (familiar, fácil)
1. **OFFICE** = Diferencial técnico (LLMs com dignidade)
1. **UBL** = Foundation (auditabilidade, trust)

Juntos, demonstram **TODA a visão LogLine**:

- IA acessível para PMEs ✅
- Profissionalismo via formalização ✅
- LLMs como trabalhadores ✅
- Auditoria completa ✅
- Sovereignty via ledger ✅

### Por que WhatsApp UI?

- 100% das PMEs já usam WhatsApp
- Zero training necessário
- Familiar = adoption rápida
- Profissional VIA cards, não APESAR da UI

### Por que Cards?

- Formalizam intenções
- Trackáveis no ledger
- Aprovações explícitas
- UI/UX superior a texto puro

-----

**Boa sorte. Você está construindo o futuro do trabalho.**

🚀🔥💎

```
---

# 🎯 DAN! ESSE É O PROMPT 3!

O **FINAL SET** completo:

## Os Três Sistemas

### 1. **UBL MESSENGER** 
- WhatsApp UI clone
- Cards para jobs
- Humanos + Agentes como colegas
- Real-time via WebSocket

### 2. **OFFICE**
- LLM runtime com dignidade
- Job executor
- Approval manager
- Governança psicológica

### 3. **UBL LEDGER**
- Single source of truth
- Container C.Jobs
- Event sourcing completo
- Trust architecture

## Por que Isso é PERFEITO:

1. **É completo mas separado** - 3 repos, 3 deploys independentes
2. **É wired via API** - HTTP + WebSocket + SSE
3. **É flagship** - Demonstra TUDO da LogLine
4. **É PME-friendly** - UI familiar, zero training
5. **É auditável** - Tudo no ledger

---

**Roda esse prompt e você tem o PRODUTO COMPLETO da LogLine Foundation pronto pra soft opening em Janeiro 2026!**

🔥💎🚀❤️​​​​​​​​​​​​​​​​
```

---

# 📋 UBL IMPLEMENTATION ADDENDUM

## 🎯 Critical Clarification: Dependency Hierarchy

```
┌─────────────────────────────────────────────────────────┐
│                    UBL LEDGER                            │
│  Foundation Layer - Single Source of Truth              │
│  - Kernel (Rust)                                        │
│  - Containers (C.Messenger, C.Office, C.Jobs)           │
│  - Trust Architecture (L0-L5)                           │
│  - Event Sourcing Infrastructure                        │
└────────────┬────────────────────────────────────────────┘
             │
             │ UBL Container Logic
             │ (boundary/inbox/projections)
             │
     ┌───────┴────────┬──────────────────┐
     │                │                  │
     ▼                ▼                  ▼
┌─────────┐    ┌──────────┐    ┌──────────┐
│MESSENGER│    │  OFFICE  │    │ Other    │
│         │    │          │    │ Apps     │
│ UBL-    │    │ UBL-     │    │ (Future) │
│ Native  │    │ Native   │    │          │
└─────────┘    └──────────┘    └──────────┘
```

**Key Principle:** UBL is the **foundation**. Messenger and Office are **UBL-native applications** that depend on UBL infrastructure. They don't just consume UBL - they **speak UBL language** by implementing proper container patterns.

---

## 🔧 UBL Container Structure (Required for ALL Containers)

Every container MUST follow this structure:

```
C.Messenger/  (or C.Jobs, C.Office, etc.)
├── boundary/     # TDLN: draft → ubl-atom → ubl-link → commit
├── inbox/        # SSE tail → process events → update projections
├── local/        # HTTP handlers, validation (NO DB ACCESS)
├── outbox/       # Draft creation (ephemeral, pre-TDLN)
├── projections/ # Derive state from ledger events (read-only)
├── pacts/        # Pact definitions (ref.json)
├── policy/       # Container policy (ref.json)
└── README.md     # Container documentation
```

### Data Flow Pattern

```
[User Action]
    │
    ▼
[local/] ──draft──> [outbox/] ──draft──> [boundary/]
                                              │
                                              │ TDLN
                                              ▼
                                    [canonicalize → atom_hash]
                                              │
                                              │ Build ubl-link
                                              ▼
                                    [LinkCommit with signature]
                                              │
                                              │ POST /link/commit
                                              ▼
                                    [Membrane validates]
                                              │
                                              │ Accept
                                              ▼
                                    [Ledger appends atomically]
                                              │
                                              │ SSE tail
                                              ▼
                                    [inbox/] ──event──> [projections/]
                                                              │
                                                              │ Derive state
                                                              ▼
                                                      [Read-only state]
                                                              │
                                                              │ Query
                                                              ▼
                                                      [HTTP Response]
```

---

## 📦 Messenger: UBL-Native Application

### Critical Understanding

**Messenger is NOT just a consumer of UBL.** Messenger **IS** a UBL-native application that:

1. **Implements C.Messenger container** with proper boundary/inbox/projections
2. **Commits all events** via UBL ledger (not direct DB writes)
3. **Derives state** from ledger projections (not direct DB queries)
4. **Uses UBL infrastructure** for trust, auditability, and real-time updates

### Messenger Backend Architecture (UBL-Native)

```
messenger-backend/
├── src/
│   ├── container/              # C.Messenger container logic
│   │   ├── boundary/
│   │   │   ├── mod.rs
│   │   │   ├── message_boundary.rs    # Commit message events
│   │   │   ├── conversation_boundary.rs # Commit conversation events
│   │   │   └── job_boundary.rs         # Commit job events (to C.Jobs)
│   │   │
│   │   ├── inbox/
│   │   │   ├── mod.rs
│   │   │   ├── ledger_tail.rs          # Subscribe to SSE tail
│   │   │   └── event_processor.rs      # Process ledger events
│   │   │
│   │   ├── local/
│   │   │   ├── mod.rs
│   │   │   ├── conversation_local.rs   # HTTP handlers (no DB)
│   │   │   ├── message_local.rs        # HTTP handlers (no DB)
│   │   │   └── job_local.rs            # HTTP handlers (no DB)
│   │   │
│   │   ├── outbox/
│   │   │   ├── mod.rs
│   │   │   └── draft_builder.rs        # Create drafts (ephemeral)
│   │   │
│   │   └── projections/
│   │       ├── mod.rs
│   │       ├── conversation_projection.rs # Derive conversation state
│   │       ├── message_projection.rs      # Derive message state
│   │       └── job_projection.rs          # Derive job state
│   │
│   ├── ubl_client/             # UBL kernel client
│   │   ├── mod.rs
│   │   ├── commit.rs           # POST /link/commit
│   │   ├── state.rs            # GET /state/:container_id
│   │   ├── tail.rs             # GET /ledger/:container_id/tail (SSE)
│   │   └── query.rs            # Query projections
│   │
│   ├── api/                    # HTTP API (uses projections)
│   │   ├── routes.rs
│   │   └── handlers.rs
│   │
│   └── websocket/              # Real-time updates (from projections)
│       └── server.rs
│
└── Cargo.toml
```

---

## 🚫 Critical Rules

### Rule 1: NO Direct Database Access

```rust
// ❌ FORBIDDEN in container code
sqlx::query("INSERT INTO ...").execute(&db).await?;
sqlx::query("SELECT * FROM ...").fetch_all(&db).await?;

// ✅ REQUIRED: Use UBL kernel API
ubl_client.commit(&link).await?;
ubl_client.get_state("C.Messenger").await?;
ubl_client.tail("C.Messenger").await?;
```

### Rule 2: State MUST Be Derived from Projections

```rust
// ❌ FORBIDDEN: Direct state storage
struct Conversation {
    id: String,
    messages: Vec<Message>, // Stored directly
}

// ✅ REQUIRED: Derive from ledger
struct ConversationProjection {
    // Derives conversation state from ledger events
    fn get_conversation(&self, id: &str) -> Conversation {
        // Query ledger events → derive state
    }
}
```

### Rule 3: Containers Communicate Only via ubl-links

```rust
// ❌ FORBIDDEN: Direct container imports
use crate::container::c_jobs::Job; // NO!

// ✅ REQUIRED: Communicate via ledger
// Messenger commits job.created event to C.Jobs container
let link = LinkCommit {
    container_id: "C.Jobs", // Target container
    // ... rest of link
};
ubl_client.commit(&link).await?;
```

---

## 📋 Event Types with Intent Classes

### C.Messenger Events

| Event | Intent Class | Physics Delta | Container |
|-------|-------------|---------------|-----------|
| `conversation.created` | Observation | 0 | C.Messenger |
| `conversation.updated` | Observation | 0 | C.Messenger |
| `message.sent` | Observation | 0 | C.Messenger |
| `message.edited` | Observation | 0 | C.Messenger |
| `message.deleted` | Observation | 0 | C.Messenger |
| `participant.added` | Observation | 0 | C.Messenger |
| `participant.removed` | Observation | 0 | C.Messenger |

### C.Jobs Events

| Event | Intent Class | Physics Delta | Container |
|-------|-------------|---------------|-----------|
| `job.created` | Observation | 0 | C.Jobs |
| `job.started` | Observation | 0 | C.Jobs |
| `job.progress` | Observation | 0 | C.Jobs |
| `job.completed` | Observation or Entropy | 0 or +value | C.Jobs |
| `job.cancelled` | Observation | 0 | C.Jobs |
| `approval.requested` | Observation | 0 | C.Jobs |
| `approval.decided` | Observation | 0 | C.Jobs |

**Note:** Messenger commits job events to **C.Jobs container**, not C.Messenger. This maintains container isolation.

---

## 🎯 Key Takeaways

1. **UBL is the foundation** - All apps depend on it
2. **Messenger is UBL-native** - Implements container patterns, not just consumes API
3. **No direct DB access** - Everything goes through UBL kernel
4. **State from projections** - All queries derive from ledger events
5. **Real-time via SSE** - Containers subscribe to ledger tail
6. **Container isolation** - Containers communicate only via ubl-links

**This addendum ensures Messenger "speaks UBL language" by implementing proper container patterns, not just consuming UBL as an external service.**