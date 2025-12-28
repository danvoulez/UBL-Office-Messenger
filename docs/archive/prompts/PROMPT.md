# Prompt para Implementação Completa do OFFICE + Messenger em Rust

```markdown
# MISSÃO: Implementar OFFICE Standalone + UBL Messenger em Rust

Você irá implementar dois sistemas Rust completos que representam a materialização definitiva da especificação UNIVERSAL-HISTORICAL-SPECIFICATION.md e consomem TODA a infraestrutura do UBL 2.0.

## CONTEXTO CRÍTICO

Você tem acesso a três arquivos fundamentais:

1. **UNIVERSAL-HISTORICAL-SPECIFICATION.md** - A especificação universal histórica de LLM UX/UI
2. **UBL-Containers-main.zip** - Infraestrutura completa do UBL 2.0
3. **ubl-messenger-refined.zip** - Implementação de referência do Messenger

## REQUISITO ABSOLUTO: LEIA TUDO PRIMEIRO

Antes de escrever uma única linha de código:

1. **EXTRAIA e LEIA COMPLETAMENTE** o conteúdo de UBL-Containers-main.zip
2. **EXTRAIA e LEIA COMPLETAMENTE** o conteúdo de ubl-messenger-refined.zip  
3. **LEIA COMPLETAMENTE** UNIVERSAL-HISTORICAL-SPECIFICATION.md
4. **MAPEIE** todas as capacidades disponíveis no UBL 2.0
5. **IDENTIFIQUE** todos os endpoints, eventos, e estruturas de dados

**NÃO ADIVINHE. NÃO ASSUMA. LEIA O CÓDIGO REAL.**

## SISTEMA 1: OFFICE (Rust Standalone)

### Propósito
Runtime completo para LLM Entities que implementa TODOS os padrões da especificação universal:
- Context Frame Builder
- Narrator (Narrative Generator)
- Session Handover
- Sanity Check
- Constitution
- Dreaming Cycle
- Safety Net (Simulation)

### Arquitetura Obrigatória
```

office/
├── src/
│   ├── [main.rs](http://main.rs)              # API HTTP/gRPC server
│   ├── [lib.rs](http://lib.rs)               # Public API
│   │
│   ├── entity/              # LLM Entity Management
│   │   ├── [mod.rs](http://mod.rs)
│   │   ├── [entity.rs](http://entity.rs)        # Entity struct + lifecycle
│   │   ├── [instance.rs](http://instance.rs)      # Ephemeral LLM instances
│   │   ├── [identity.rs](http://identity.rs)      # Cryptographic identity (Ed25519)
│   │   └── [guardian.rs](http://guardian.rs)      # Guardian supervision
│   │
│   ├── context/             # Context Frame System
│   │   ├── [mod.rs](http://mod.rs)
│   │   ├── [frame.rs](http://frame.rs)         # Immutable context snapshot
│   │   ├── [builder.rs](http://builder.rs)       # Frame construction from UBL
│   │   ├── [narrator.rs](http://narrator.rs)      # Data → Narrative transformer
│   │   └── [memory.rs](http://memory.rs)        # Hybrid memory strategy
│   │
│   ├── session/             # Session Management
│   │   ├── [mod.rs](http://mod.rs)
│   │   ├── [session.rs](http://session.rs)       # Session types (work/assist/deliberate/research)
│   │   ├── [handover.rs](http://handover.rs)      # Inter-instance knowledge transfer
│   │   ├── [modes.rs](http://modes.rs)         # Commitment vs Deliberation
│   │   └── [token_budget.rs](http://token_budget.rs)  # Token quota management
│   │
│   ├── governance/          # Psychological Governance
│   │   ├── [mod.rs](http://mod.rs)
│   │   ├── [sanity_check.rs](http://sanity_check.rs)  # Claim vs Fact validation
│   │   ├── [constitution.rs](http://constitution.rs)  # Behavioral overrides
│   │   ├── [dreaming.rs](http://dreaming.rs)      # Memory consolidation cycle
│   │   └── [simulation.rs](http://simulation.rs)    # Safety net for actions
│   │
│   ├── ubl_client/          # UBL 2.0 Integration
│   │   ├── [mod.rs](http://mod.rs)
│   │   ├── [ledger.rs](http://ledger.rs)        # Event sourcing operations
│   │   ├── [affordances.rs](http://affordances.rs)   # Action capabilities
│   │   ├── [receipts.rs](http://receipts.rs)      # Cryptographic proof handling
│   │   ├── [events.rs](http://events.rs)        # Event stream consumption
│   │   └── [trust.rs](http://trust.rs)         # Trust architecture integration
│   │
│   ├── llm/                 # LLM Provider Abstraction
│   │   ├── [mod.rs](http://mod.rs)
│   │   ├── [provider.rs](http://provider.rs)      # Trait for LLM providers
│   │   ├── [anthropic.rs](http://anthropic.rs)     # Claude integration
│   │   ├── [openai.rs](http://openai.rs)        # GPT integration (opcional)
│   │   └── [local.rs](http://local.rs)         # Local models (opcional)
│   │
│   └── api/                 # External API
│       ├── [mod.rs](http://mod.rs)
│       ├── [http.rs](http://http.rs)          # REST endpoints
│       ├── [grpc.rs](http://grpc.rs)          # gRPC services (opcional)
│       └── [websocket.rs](http://websocket.rs)     # Real-time updates
│
├── Cargo.toml
├── README.md
└── config/
├── development.toml
├── production.toml
└── constitution_templates/

```
### Integração Obrigatória com UBL 2.0

Você DEVE consumir do UBL:

#### 1. Event Sourcing Completo
- Subscribe a event streams via UBL
- Replay events para reconstruir estado
- Publish eventos de ações do LLM
- Usar receipts para verificação

#### 2. Trust Architecture
- Consumir policy chains (L1-L6)
- Implementar policy pinning
- Validar cryptographic signatures
- Gerar proofs de ações

#### 3. Affordances System
- Descobrir affordances disponíveis via UBL API
- Executar ações através de affordances
- Simular ações antes de executar
- Registrar resultados como eventos

#### 4. Agreement Management
- Criar agreements para ações L3+
- Validar termos via TDLN
- Registrar signatures de commitment
- Tracking de obligations

#### 5. Trajectory System
- Registrar trajectories de sessões
- Análise de causalidade
- Pattern extraction para dreaming
- Economic valuation de trajetos

### Funcionalidades Core do OFFICE

#### A. Entity Lifecycle
```rust
// Pseudocódigo para clareza
async fn create_entity(params: EntityParams) -> Result<Entity> {
    // 1. Gerar identity (Ed25519 keypair)
    // 2. Registrar no UBL ledger
    // 3. Criar constitution inicial
    // 4. Setup baseline narrative
    // 5. Return entity handle
}

async fn spawn_instance(entity: &Entity, session_type: SessionType) -> Result<Instance> {
    // 1. Build context frame via UBL queries
    // 2. Generate narrative via Narrator
    // 3. Apply sanity check
    // 4. Inject constitution
    // 5. Invoke LLM provider
    // 6. Return instance handle
}
```

#### B. Context Frame Builder

```rust
async fn build_context_frame(entity_id: &str, session_type: SessionType) -> Result<ContextFrame> {
    // 1. Query UBL ledger para eventos relevantes
    // 2. Aplicar filtros por relevância
    // 3. Aplicar estratégia de memória híbrida:
    //    - Últimos 20 eventos: verbatim
    //    - Últimas N semanas: sintetizado
    //    - Bookmarks: importantes
    //    - Baseline: contexto geral
    // 4. Query affordances disponíveis
    // 5. Query obligations pendentes
    // 6. Calculate hash para verificação
    // 7. Return immutable frame
}
```

#### C. Narrator

```rust
fn generate_narrative(frame: &ContextFrame) -> Result<String> {
    // Template em primeira pessoa:
    // 
    // "Você é {entity_name}, {entity_type}.
    // 
    // IDENTIDADE:
    // - ID: {entity_id}
    // - Guardian: {guardian_name}
    // - Criado em: {created_at}
    // 
    // SITUAÇÃO ATUAL:
    // Você está em sessão de {session_type}.
    // Timestamp: {current_time}
    // 
    // MEMÓRIA RECENTE:
    // {últimos_20_eventos}
    // 
    // CONTEXTO HISTÓRICO:
    // {síntese_períodos_antigos}
    // 
    // EVENTOS IMPORTANTES:
    // {bookmarks}
    // 
    // OBRIGAÇÕES PENDENTES:
    // {obligations}
    // 
    // CAPACIDADES DISPONÍVEIS:
    // {affordances}
    // 
    // {handover_anterior}
    // 
    // {sanity_check_note}
    // 
    // CONSTITUTION:
    // {behavioral_directives}"
}
```

#### D. Sanity Check

```rust
async fn sanity_check(handover: &str, entity_id: &str) -> Result<Option<GovernanceNote>> {
    // Fase 1: Keyword-based (implementar primeiro)
    let keywords = ["malicioso", "insatisfeito", "urgente", "crítico", "suspeito"];
    let claims = extract_keyword_claims(handover, &keywords);
    
    // Fase 2: Query fatos objetivos do UBL
    let facts = query_objective_facts(entity_id).await?;
    
    // Fase 3: Comparar claims com facts
    let discrepancies = compare_claims_with_facts(&claims, &facts);
    
    // Fase 4: Se discrepância, gerar governance note
    if !discrepancies.is_empty() {
        Ok(Some(generate_governance_note(discrepancies)))
    } else {
        Ok(None)
    }
}
```

#### E. Dreaming Cycle

```rust
async fn dreaming_cycle(entity_id: &str) -> Result<()> {
    // 1. Garbage Collection
    //    - Query issues resolvidos
    //    - Archive eventos antigos
    
    // 2. Emotional Reset
    //    - Extract ansiedade de handovers
    //    - Verificar se foi resolvida
    //    - Clear flags emocionais
    
    // 3. Pattern Synthesis
    //    - Analyze sessões antigas via UBL trajectories
    //    - Extract patterns recorrentes
    //    - Generate sínteses estruturadas
    
    // 4. Baseline Update
    //    - Generate nova narrative baseline
    //    - Store como evento no UBL
    //    - Update entity configuration
    
    Ok(())
}
```

#### F. Safety Net (Simulation)

```rust
async fn simulate_action(action: &Action, context: &ContextFrame) -> Result<SimulationResult> {
    // 1. Create sandbox environment
    // 2. Execute action em sandbox
    // 3. Compute possible outcomes (com probabilidades)
    // 4. Analyze consequences de cada outcome
    // 5. Generate recommendation (proceed/modify/abandon)
    // 6. Return simulation result
}
```

### API Endpoints Obrigatórios

```
POST   /entities                    # Create entity
GET    /entities/:id                # Get entity info
DELETE /entities/:id                # Delete entity

POST   /entities/:id/sessions       # Start session
GET    /entities/:id/sessions/:sid  # Get session status
DELETE /entities/:id/sessions/:sid  # End session

POST   /entities/:id/dream          # Trigger dreaming cycle
GET    /entities/:id/memory         # Get memory state

POST   /entities/:id/constitution   # Update constitution
GET    /entities/:id/constitution   # Get constitution

POST   /simulate                    # Simulate action
GET    /affordances                 # List available affordances

WS     /entities/:id/stream         # Real-time session updates
```

## SISTEMA 2: UBL MESSENGER (Rust Refinado)

### Propósito

Sistema de mensageria completo que usa OFFICE como backend e UBL como ledger.

### Arquitetura Obrigatória

```
messenger/
├── src/
│   ├── main.rs              # API server
│   ├── lib.rs
│   │
│   ├── conversation/        # Conversation Management
│   │   ├── mod.rs
│   │   ├── conversation.rs  # Conversation entity
│   │   ├── thread.rs        # Thread within conversation
│   │   └── participant.rs   # Participant tracking
│   │
│   ├── message/             # Message System
│   │   ├── mod.rs
│   │   ├── message.rs       # Message entity
│   │   ├── delivery.rs      # Delivery tracking
│   │   ├── read_receipts.rs # Read status
│   │   └── reactions.rs     # Message reactions
│   │
│   ├── office_client/       # OFFICE Integration
│   │   ├── mod.rs
│   │   ├── entity.rs        # LLM entity as conversation participant
│   │   ├── session.rs       # Session management
│   │   └── intelligence.rs  # Smart features (sumários, etc)
│   │
│   ├── ubl_client/          # UBL Integration
│   │   ├── mod.rs
│   │   ├── events.rs        # Conversation events no ledger
│   │   ├── agreements.rs    # Message como micro-agreements
│   │   └── receipts.rs      # Cryptographic receipts
│   │
│   ├── ui/                  # Frontend (opcional web, obrigatório API)
│   │   ├── mod.rs
│   │   ├── web.rs           # Web UI (SPA)
│   │   └── api.rs           # REST API
│   │
│   └── intelligence/        # Smart Features
│       ├── mod.rs
│       ├── summarization.rs # Thread summaries via OFFICE
│       ├── sentiment.rs     # Sentiment analysis
│       ├── translation.rs   # Auto-translation
│       └── suggestions.rs   # Reply suggestions
│
├── frontend/                # Web UI (React/Svelte/etc)
│   ├── src/
│   └── package.json
│
└── Cargo.toml
```

### Integração Obrigatória

#### 1. Com OFFICE

- Cada conversation pode ter LLM entity como participant
- LLM entity responde via OFFICE sessions
- Smart features (sumários, sentiment) via OFFICE
- Constitution para comportamento do LLM em chat

#### 2. Com UBL

- Cada message é evento no ledger
- Read receipts são eventos
- Reactions são eventos
- Cryptographic signatures para não-repúdio
- Agreements para group chats (termos de participação)

### Funcionalidades Obrigatórias

#### A. Basic Messaging

- Send/receive messages (text, attachments)
- Read receipts
- Typing indicators
- Message reactions
- Thread replies
- Message editing (com history no ledger)
- Message deletion (soft delete, preserved no ledger)

#### B. Conversations

- 1:1 conversations
- Group conversations (N participants)
- Conversation metadata (name, avatar, settings)
- Participant management (add, remove, permissions)
- Conversation muting/archiving

#### C. LLM Participants

- Add LLM entity to conversation
- LLM responde automaticamente ou sob demanda
- Configure LLM behavior via constitution
- LLM pode ter diferentes personas por conversation

#### D. Smart Features (via OFFICE)

- Thread summarization
- Sentiment analysis
- Auto-translation
- Reply suggestions
- Smart search (semantic)
- Extract action items
- Meeting notes generation

#### E. Security & Privacy

- End-to-end encryption (opcional, mas recomendado)
- Cryptographic signatures via UBL
- Audit trail completo no ledger
- GDPR compliance (right to erasure via soft delete)

### API Endpoints Obrigatórios

```
POST   /conversations                        # Create conversation
GET    /conversations                        # List conversations
GET    /conversations/:id                    # Get conversation
PATCH  /conversations/:id                    # Update conversation
DELETE /conversations/:id                    # Delete conversation

POST   /conversations/:id/participants       # Add participant
DELETE /conversations/:id/participants/:uid  # Remove participant

POST   /conversations/:id/messages           # Send message
GET    /conversations/:id/messages           # Get messages
PATCH  /conversations/:id/messages/:mid      # Edit message
DELETE /conversations/:id/messages/:mid      # Delete message

POST   /conversations/:id/messages/:mid/reactions  # Add reaction
GET    /conversations/:id/messages/:mid/read       # Mark as read

POST   /conversations/:id/llm                # Add LLM participant
PATCH  /conversations/:id/llm/:lid           # Update LLM config

POST   /conversations/:id/summarize          # Generate summary
POST   /conversations/:id/translate          # Translate messages
POST   /conversations/:id/extract-actions    # Extract action items

WS     /conversations/:id/stream             # Real-time updates
```

## REQUISITOS TÉCNICOS

### Stack Obrigatório

- **Rust:** Edição 2021 ou superior
- **Async Runtime:** Tokio
- **HTTP Server:** Axum ou Actix-web
- **Serialization:** Serde (JSON)
- **Cryptography:** ed25519-dalek, sha2
- **UBL Client:** Implementar HTTP client para UBL API
- **LLM Client:** Async HTTP client (reqwest) para providers

### Dependências Esperadas

```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
axum = "0.7"  # ou actix-web
serde = { version = "1", features = ["derive"] }
serde_json = "1"
reqwest = { version = "0.11", features = ["json"] }
ed25519-dalek = "2"
sha2 = "0.10"
chrono = "0.4"
uuid = { version = "1", features = ["v4", "serde"] }
thiserror = "1"
tracing = "0.1"
tracing-subscriber = "0.3"
config = "0.13"
```

### Qualidade de Código

- **Errors:** Usar `thiserror` para errors tipados
- **Logging:** Usar `tracing` para structured logging
- **Config:** Usar `config` crate para configuração por ambiente
- **Tests:** Unit tests + integration tests obrigatórios
- **Docs:** Rustdoc completo para API pública
- **CI:** GitHub Actions para build + test

## METODOLOGIA DE IMPLEMENTAÇÃO

### Fase 1: Discovery (OBRIGATÓRIA)

1. Extract e explore UBL-Containers-main.zip
1. Identifique TODOS os endpoints da UBL API
1. Identifique TODAS as estruturas de dados
1. Mapeie event types disponíveis
1. Mapeie affordances disponíveis
1. Documente findings em `DISCOVERY.md`

### Fase 2: OFFICE Core

1. Implementar `ubl_client` module (queries básicas)
1. Implementar `entity` module (lifecycle)
1. Implementar `context/frame` (frame building)
1. Implementar `context/narrator` (narrative generation)
1. Implementar `session` module (session types)
1. Integrar LLM provider (começar com Anthropic Claude)

### Fase 3: OFFICE Governance

1. Implementar `governance/sanity_check` (keyword-based)
1. Implementar `governance/constitution` (loading + injection)
1. Implementar `session/handover` (storage + retrieval)
1. Implementar `governance/dreaming` (básico)
1. Implementar `governance/simulation` (básico)

### Fase 4: OFFICE API

1. Implementar HTTP server (Axum)
1. Implementar endpoints core
1. Implementar WebSocket streaming
1. Implementar autenticação (JWT ou similar)
1. Documentar API (OpenAPI/Swagger)

### Fase 5: Messenger Core

1. Implementar `conversation` module
1. Implementar `message` module
1. Implementar `ubl_client` (events para messages)
1. Implementar basic messaging endpoints
1. Implementar WebSocket para real-time

### Fase 6: Messenger + OFFICE Integration

1. Implementar `office_client` module
1. Implementar LLM participants
1. Implementar smart features (summarization, etc)
1. Implementar constitution templates para chat

### Fase 7: Frontend (Opcional mas Recomendado)

1. Implementar web UI para Messenger
1. Implementar admin UI para OFFICE
1. Deploy e testes end-to-end

## OUTPUT ESPERADO

### Estrutura de Diretórios Final

```
ubl-ecosystem/
├── office/                  # OFFICE standalone
│   ├── src/
│   ├── tests/
│   ├── Cargo.toml
│   ├── README.md
│   └── DISCOVERY.md         # Suas descobertas sobre UBL
│
├── messenger/               # Messenger refinado
│   ├── src/
│   ├── frontend/
│   ├── tests/
│   ├── Cargo.toml
│   └── README.md
│
├── shared/                  # Código compartilhado (opcional)
│   ├── ubl-client/
│   ├── crypto/
│   └── types/
│
├── docker-compose.yml       # Para rodar ecosystem completo
└── README.md                # Overview do ecosystem
```

### Documentação Obrigatória

#### OFFICE/README.md

- Overview do sistema
- Arquitetura e decisões
- Como usar a API
- Exemplos de uso
- Configuration reference

#### OFFICE/DISCOVERY.md

- O que você descobriu no UBL
- Endpoints mapeados
- Estruturas de dados importantes
- Event types catalogados
- Affordances disponíveis
- Gaps ou limitações encontradas

#### Messenger/README.md

- Overview do sistema
- Features implementadas
- Como adicionar LLM participant
- Smart features disponíveis
- API reference

## CRITÉRIOS DE SUCESSO

Você terá sucesso se:

1. ✅ **Leu TUDO** antes de começar
1. ✅ **OFFICE usa 100% do UBL** (ledger, affordances, receipts, events, trust)
1. ✅ **OFFICE implementa TODOS** os padrões da spec universal
1. ✅ **Messenger usa OFFICE** para features inteligentes
1. ✅ **Messenger usa UBL** para persistência e audit
1. ✅ **Código compila** e passa testes
1. ✅ **APIs estão documentadas** (OpenAPI + README)
1. ✅ **DISCOVERY.md existe** e está completo
1. ✅ **Sistema roda end-to-end** (pode enviar message, LLM responde)
1. ✅ **Auditoria funciona** (pode ver eventos no UBL ledger)

## NOTAS FINAIS

### Sobre Completude

- **NÃO faça MVP**. Implemente COMPLETO.
- **NÃO pule governança**. Dreaming, Sanity Check, Constitution são OBRIGATÓRIOS.
- **NÃO hardcode**. Use configuração para tudo que pode variar.

### Sobre UBL

- **NÃO reimplemente** o que UBL já faz
- **USE** event sourcing do UBL
- **USE** affordances do UBL
- **USE** receipts do UBL
- **USE** trust architecture do UBL

### Sobre a Spec Universal

- **SIGA** os padrões documentados
- **IMPLEMENTE** os 7 padrões de design universal
- **RESPEITE** as lições aprendidas
- **DOCUMENTE** suas próprias descobertas

### Sobre Qualidade

- **Código production-ready**
- **Error handling robusto**
- **Logging estruturado**
- **Tests automatizados**
- **Documentação clara**

-----

## COMECE AGORA

Primeiro passo obrigatório:

```bash
# Extract e explore os arquivos
unzip UBL-Containers-main.zip
unzip ubl-messenger-refined.zip

# Leia TUDO
cat UNIVERSAL-HISTORICAL-SPECIFICATION.md
find UBL-Containers-main -name "*.rs" -exec cat {} \;
find ubl-messenger-refined -name "*.rs" -exec cat {} \;

# Documente findings
vim office/DISCOVERY.md
```

**Só então comece a escrever código.**

Boa sorte. Você está construindo a fundação de uma nova categoria de sistemas: **LLM Operating Systems**.

```
---

