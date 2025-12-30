# SPEC-UBL-SCHENGEN v1.0 — Zona de Confiança por Cascata

## Conceito

A **Zona Schengen** do UBL é um modelo de autorização em cascata onde:
- Uma vez autenticado e dentro de um tenant, o usuário já tem autorização base
- Ações subsequentes não precisam de re-autenticação completa
- Mas **toda ação mantém peso criptográfico** (assinatura Ed25519)

```
┌─────────────────────────────────────────────────────────────────────┐
│                           FRONTEIRA                                  │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │                   ZONA SCHENGEN (Tenant)                      │   │
│  │                                                               │   │
│  │   ┌─────────┐    ┌─────────┐    ┌─────────┐    ┌─────────┐  │   │
│  │   │ Mensagem│    │ Job     │    │ Arquivo │    │ Config  │  │   │
│  │   │ ✓ leve  │    │ ✓ leve  │    │ ✓ leve  │    │ 🔐 peso │  │   │
│  │   └─────────┘    └─────────┘    └─────────┘    └─────────┘  │   │
│  │                                                               │   │
│  │   Dentro: Session Token + tenant_id ✓                        │   │
│  │   Tudo é assinado Ed25519, mas auth já foi feita             │   │
│  │                                                               │   │
│  └──────────────────────────────────────────────────────────────┘   │
│                                                                      │
│   🔐 Entrada: WebAuthn Passkey (verificação biométrica)             │
│   🔐 Step-Up: Para ações L4-L5 (admin, delete, transfer)            │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Níveis de Segurança Existentes

### 1. Fronteira (WebAuthn)

**Arquivo:** `/ubl/kernel/rust/ubl-server/src/id_routes.rs`

```rust
// Login com passkey - verificação biométrica
#[post("/id/login/finish")]
async fn login_finish(webauthn_response) {
    // Valida assinatura do passkey
    // Cria sessão com token
    session = Session::new_regular(sid);  // 1 hora
    // Guarda em id_session
}
```

**Quando acontece:** Primeira entrada no sistema

---

### 2. Sessão Regular (Bearer Token)

**Arquivo:** `/ubl/kernel/rust/ubl-server/src/auth/session.rs`

```rust
pub struct Session {
    pub token: String,      // UUID aleatório
    pub sid: Uuid,          // Subject ID
    pub flavor: SessionFlavor,  // Regular ou StepUp
    pub exp_unix: i64,      // 1 hora para Regular
}

impl Session {
    pub fn new_regular(sid: Uuid) -> Self {
        // Expira em 1 hora
        // scope: {} (vazio - acesso básico)
    }
}
```

**O que permite:**
- Leitura de dados do tenant
- Enviar mensagens
- Ver jobs
- Ações do dia-a-dia

---

### 3. Step-Up Authentication (Ações Críticas)

**Arquivo:** `/ubl/kernel/rust/ubl-server/src/auth/session.rs`

```rust
impl Session {
    pub fn new_stepup(sid: Uuid) -> Self {
        // Expira em 10 minutos
        // scope: {"role": "admin"}
    }
}
```

**Arquivo:** `/apps/office/src/middleware/constitution.rs`

```rust
pub struct ModeConfig {
    pub max_risk: String,      // L0-L5
    pub require_step_up: bool, // Para admin = true
}

// Operator (L0-L2): NÃO precisa step-up
// Admin (L3-L5): PRECISA step-up
```

**Quando é exigido:**
- Risk Level L4-L5
- Deletar recursos
- Mudar permissões
- Transferir ownership
- Deploy em produção

---

### 4. ASC (Agent Signing Certificate)

**Arquivo:** `/ubl/kernel/rust/ubl-server/src/auth.rs`

```rust
pub struct AscContext {
    pub sid: String,
    pub containers: Vec<String>,     // ["C.Messenger", "C.Jobs"]
    pub intent_classes: Vec<String>, // ["Observation", "Reaction"]
    pub max_delta: Option<i128>,     // Limite de physics_delta
}

// CRÍTICO: LLM NUNCA pode fazer Entropy/Evolution
fn is_llm_agent(sid: &str) -> bool {
    sid.contains(":llm:")
}
```

**Para:** Agentes LLM e Apps que assinam commits automaticamente

---

### 5. Assinatura Ed25519 (SEMPRE)

**Arquivo:** `/ubl/kernel/rust/ubl-server/src/main.rs`

```rust
// TODA ação no ledger precisa de assinatura
// INDEPENDENTE do nível de sessão

let signing_data = json!({
    "version": link.version,
    "container_id": link.container_id,
    "expected_sequence": link.expected_sequence,
    "previous_hash": link.previous_hash,
    "atom_hash": link.atom_hash,
    "intent_class": link.intent_class,
    "physics_delta": link.physics_delta,
    "pact": link.pact,
});

let signing_bytes = ubl_atom::canonicalize(&signing_data)?;

// ✅ Verifica assinatura Ed25519
verify_signature(&link.author_pubkey, &signing_bytes, &link.signature)?;
```

**Resultado:** Toda ação é criptograficamente verificável, mesmo sendo "leve" dentro da Zona Schengen.

---

## Fluxo Completo

```
┌──────────────────────────────────────────────────────────────────────┐
│  USUÁRIO                                                              │
└─────────┬────────────────────────────────────────────────────────────┘
          │
          │ 1. Login (WebAuthn Passkey)
          ▼
┌──────────────────────────────────────────────────────────────────────┐
│  🔐 FRONTEIRA - Verificação Biométrica                               │
│     - Face ID / Touch ID / YubiKey                                   │
│     - Cria Session Token (1 hora)                                    │
│     - Define tenant_id                                               │
└─────────┬────────────────────────────────────────────────────────────┘
          │
          │ 2. Ações normais (bearer token)
          ▼
┌──────────────────────────────────────────────────────────────────────┐
│  🟢 ZONA SCHENGEN - Operações L0-L2                                  │
│                                                                       │
│  Cada ação:                                                          │
│  ┌─────────────────────────────────────────────────────────────────┐ │
│  │ 1. Verifica Session Token (válido? expirou?)                    │ │
│  │ 2. Extrai tenant_id da sessão                                   │ │
│  │ 3. Prepara Link (container, atom, intent_class)                 │ │
│  │ 4. ✍️  ASSINA com Ed25519 (peso criptográfico)                  │ │
│  │ 5. POST /link/commit                                            │ │
│  │ 6. UBL verifica assinatura (SEMPRE)                             │ │
│  │ 7. Appenda no ledger imutável                                   │ │
│  └─────────────────────────────────────────────────────────────────┘ │
│                                                                       │
│  Exemplos:                                                           │
│  - Enviar mensagem → Observation, delta=0                            │
│  - Criar job → Observation, delta=0                                  │
│  - Aprovar job → Reaction, delta>0                                   │
│                                                                       │
└─────────┬────────────────────────────────────────────────────────────┘
          │
          │ 3. Ação crítica (L4-L5)
          ▼
┌──────────────────────────────────────────────────────────────────────┐
│  🔴 STEP-UP REQUIRED - Verificação Adicional                         │
│                                                                       │
│  Constitution Rule:                                                  │
│  ┌─────────────────────────────────────────────────────────────────┐ │
│  │ admin:                                                          │ │
│  │   max_risk: "L5"                                                │ │
│  │   require_step_up: true   ← 🔐                                  │ │
│  └─────────────────────────────────────────────────────────────────┘ │
│                                                                       │
│  Processo:                                                           │
│  1. UI pede passkey novamente                                        │
│  2. Cria Session StepUp (10 min)                                     │
│  3. Executa ação com flavor=stepup                                   │
│                                                                       │
│  Exemplos:                                                           │
│  - Deletar tenant → Evolution, requer step-up                        │
│  - Revogar chave → Entropy, requer step-up                           │
│  - Deploy prod → Risk L5, requer step-up                             │
│                                                                       │
└──────────────────────────────────────────────────────────────────────┘
```

---

## O Que Já Existe no Código

### ✅ Implementado

| Componente | Arquivo | Status |
|------------|---------|--------|
| WebAuthn Login | `id_routes.rs` | ✅ Completo |
| Session Regular | `auth/session.rs` | ✅ Completo |
| Session StepUp | `auth/session.rs` | ✅ Completo |
| Session DB | `auth/session_db.rs` | ✅ Completo |
| ASC Validation | `auth.rs` | ✅ Completo |
| Ed25519 Verify | `main.rs` | ✅ Completo |
| Constitution | `middleware/constitution.rs` | ✅ Completo |
| Risk Levels L0-L5 | `constitution.rs` | ✅ Completo |
| LLM Restrictions | `auth.rs` | ✅ Completo |

### 🔄 Precisa Propagar

| Componente | Arquivo | Status |
|------------|---------|--------|
| tenant_id na sessão | `id_session` | 🔄 Parcial |
| tenant_id nos commits | `messenger_gateway` | 🔄 Hardcoded |
| Step-up UI | Frontend | 🔄 Falta |

---

## Como Garantir Peso Criptográfico Sem Friction

### Princípio

> "Segurança invisível para o usuário, mas auditável para o sistema"

### Implementação Atual

```rust
// No frontend (React):
const sendMessage = async (content) => {
    // 1. Prepara o atom
    const atom = {
        type: "message.created",
        content_hash: blake3(content),
        from: session.sid,
        // ...
    };
    
    // 2. Assina localmente (chave no navegador via WebAuthn)
    const signature = await signWithPasskey(atom);
    
    // 3. Envia para UBL
    await api.post('/link/commit', {
        ...link,
        signature,
        author_pubkey: pubkey,
    });
};
```

### O Que Acontece

1. **Usuário não vê nada** (sessão válida, token bearer)
2. **Sistema assina automaticamente** (chave derivada do passkey)
3. **UBL verifica Ed25519** (não confia em ninguém)
4. **Ledger registra tudo** (imutável, auditável)

---

## Melhorias Sugeridas

### 1. Sessão com Tenant Context

```rust
// Atual
pub struct Session {
    pub token: String,
    pub sid: Uuid,
    pub flavor: SessionFlavor,
    pub scope: serde_json::Value,
    pub exp_unix: i64,
}

// Proposto
pub struct Session {
    pub token: String,
    pub sid: Uuid,
    pub tenant_id: Option<String>,  // ← ADICIONAR
    pub flavor: SessionFlavor,
    pub scope: serde_json::Value,
    pub exp_unix: i64,
}
```

### 2. Assinatura Client-Side

Atualmente a assinatura pode ser "placeholder" em dev. Para produção:

```typescript
// Frontend deve usar WebAuthn PRF extension
// ou derivar chave Ed25519 do passkey

import { sign } from '@noble/ed25519';

const signLink = async (link) => {
    const keyPair = await deriveFromPasskey();
    const canonicalBytes = canonicalize(link);
    return sign(keyPair.privateKey, canonicalBytes);
};
```

### 3. Audit Trail Automático

```sql
-- Toda ação tem:
-- 1. entry_hash (único)
-- 2. previous_hash (chain)
-- 3. signature (Ed25519)
-- 4. author_pubkey
-- 5. timestamp

-- Query de auditoria:
SELECT 
    container_id,
    sequence,
    entry_hash,
    atom_data->>'type' as event_type,
    author_pubkey,
    signature,
    ts_unix_ms
FROM ledger_entry
WHERE container_id = 'C.Messenger'
  AND atom_data->>'tenant_id' = $1
ORDER BY sequence;
```

---

## Resumo: Zona Schengen + Peso Criptográfico

| Aspecto | Antes da Zona | Dentro da Zona | Step-Up |
|---------|---------------|----------------|---------|
| **Verificação** | Passkey biométrico | Token bearer | Passkey novamente |
| **Duração** | Uma vez | 1 hora | 10 minutos |
| **UX** | Toque/Face | Invisível | Toque/Face |
| **Assinatura Ed25519** | ✅ | ✅ | ✅ |
| **No Ledger** | ✅ | ✅ | ✅ |
| **Auditável** | ✅ | ✅ | ✅ |

### Conclusão

O UBL já implementa a Zona Schengen corretamente:
- **Fronteira forte** (WebAuthn Passkey)
- **Interior fluido** (Bearer token, sem re-auth)
- **Peso criptográfico em tudo** (Ed25519 em cada commit)
- **Step-up quando necessário** (L4-L5, admin)

O que falta é:
1. Propagar `tenant_id` consistentemente
2. Implementar assinatura client-side em produção
3. UI de step-up no frontend
