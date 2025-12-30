# Implementação da Zona Schengen - Checklist

## Status Atual

A base já existe! O UBL implementa corretamente:

✅ **WebAuthn Passkey** - Fronteira forte
✅ **Session Regular/StepUp** - Dois níveis de acesso  
✅ **ASC Validation** - Certificados para agentes
✅ **Ed25519 Verification** - Toda ação é assinada
✅ **Constitution Rules** - L0-L5 com step-up
✅ **C.Tenant** - Isolamento por organização

## O Que Falta

### 1. Propagar tenant_id na Sessão

**Arquivo:** `ubl/kernel/rust/ubl-server/src/auth/session.rs`

```rust
// Adicionar tenant_id ao struct Session
pub struct Session {
    pub token: String,
    pub sid: Uuid,
    pub tenant_id: Option<String>,  // ← NOVO
    pub flavor: SessionFlavor,
    pub scope: serde_json::Value,
    pub exp_unix: i64,
}
```

**Arquivo:** `ubl/sql/00_base/001_identity.sql`

```sql
-- Atualizar id_session para incluir tenant_id
ALTER TABLE id_session 
ADD COLUMN tenant_id TEXT REFERENCES id_tenant(tenant_id);
```

### 2. Extrair tenant_id dos Headers

**Arquivo:** `ubl/kernel/rust/ubl-server/src/messenger_gateway/routes.rs`

```rust
// Mudar de:
let tenant_id = "default"; // TODO: Extract from session

// Para:
let tenant_id = get_session(&pool, &headers)
    .await
    .and_then(|s| s.tenant_id)
    .ok_or((StatusCode::FORBIDDEN, "No tenant context"))?;
```

### 3. Assinatura Client-Side

**Arquivo:** `apps/messenger/frontend/src/services/signing.ts` (NOVO)

```typescript
import { sign } from '@noble/ed25519';
import { canonicalize } from './canonicalize';

// Derivar chave Ed25519 do passkey via PRF extension
export async function signLink(link: LinkDraft): Promise<string> {
    const session = getSession();
    if (!session?.privateKey) {
        throw new Error('No signing key in session');
    }
    
    const signingData = {
        version: link.version,
        container_id: link.container_id,
        expected_sequence: link.expected_sequence,
        previous_hash: link.previous_hash,
        atom_hash: link.atom_hash,
        intent_class: link.intent_class,
        physics_delta: link.physics_delta,
        pact: link.pact,
    };
    
    const canonical = canonicalize(signingData);
    const signature = await sign(session.privateKey, canonical);
    
    return Buffer.from(signature).toString('hex');
}
```

### 4. UI de Step-Up

**Arquivo:** `apps/messenger/frontend/src/components/StepUpModal.tsx` (NOVO)

```tsx
export const StepUpModal: React.FC<{
    isOpen: boolean;
    onSuccess: (stepupToken: string) => void;
    onCancel: () => void;
    action: string;
}> = ({ isOpen, onSuccess, onCancel, action }) => {
    
    const handleStepUp = async () => {
        // 1. Pede challenge ao servidor
        const { challenge } = await api.post('/id/stepup/begin');
        
        // 2. Usa WebAuthn para assinar
        const assertion = await navigator.credentials.get({
            publicKey: {
                challenge: base64ToBuffer(challenge),
                allowCredentials: [...],
            }
        });
        
        // 3. Envia para servidor validar
        const { token } = await api.post('/id/stepup/finish', {
            response: assertion
        });
        
        onSuccess(token);
    };
    
    return (
        <Modal isOpen={isOpen}>
            <h2>Confirmação Necessária</h2>
            <p>A ação "{action}" requer verificação adicional.</p>
            <Button onClick={handleStepUp}>
                Confirmar com Passkey
            </Button>
            <Button variant="ghost" onClick={onCancel}>
                Cancelar
            </Button>
        </Modal>
    );
};
```

### 5. Middleware de Risk Level

**Arquivo:** `apps/messenger/frontend/src/hooks/useSecureAction.ts` (NOVO)

```typescript
export function useSecureAction() {
    const [needsStepUp, setNeedsStepUp] = useState(false);
    const [pendingAction, setPendingAction] = useState<() => Promise<void>>();
    
    const executeSecure = async (
        action: () => Promise<void>,
        riskLevel: 'L0' | 'L1' | 'L2' | 'L3' | 'L4' | 'L5'
    ) => {
        // L0-L2: Executa direto (Zona Schengen)
        if (['L0', 'L1', 'L2'].includes(riskLevel)) {
            return action();
        }
        
        // L3+: Verifica se tem step-up válido
        const session = getSession();
        if (session?.stepUpExpires && session.stepUpExpires > Date.now()) {
            return action();
        }
        
        // Precisa step-up
        setPendingAction(() => action);
        setNeedsStepUp(true);
    };
    
    return { executeSecure, needsStepUp, setNeedsStepUp, pendingAction };
}
```

---

## Tabela de Risk Levels

| Level | Descrição | Auth Necessária | Exemplos |
|-------|-----------|-----------------|----------|
| L0 | Leitura | Session | Ver mensagens, ver jobs |
| L1 | Escrita reversível | Session | Enviar mensagem, criar job |
| L2 | Escrita não-reversível | Session | Aprovar job, arquivar |
| L3 | Admin | Step-Up | Convidar membros, config |
| L4 | Destrutivo | Step-Up | Deletar recursos, revogar |
| L5 | Crítico | Step-Up | Deletar tenant, transfer owner |

---

## Arquivos a Modificar

### Backend

1. `auth/session.rs` - Adicionar tenant_id
2. `auth/session_db.rs` - Salvar/recuperar tenant_id  
3. `id_routes.rs` - Setar tenant_id no login
4. `messenger_gateway/routes.rs` - Extrair tenant_id da sessão
5. `projections/*.rs` - Usar tenant_id em todas queries

### Frontend

1. `services/signing.ts` - Assinatura Ed25519 client-side
2. `components/StepUpModal.tsx` - UI de step-up
3. `hooks/useSecureAction.ts` - Middleware de risk level
4. `context/AuthContext.tsx` - Guardar tenant_id

### SQL

1. `001_identity.sql` - Adicionar tenant_id à id_session

---

## Prioridades

| # | Item | Impacto | Esforço |
|---|------|---------|---------|
| 1 | tenant_id na sessão | Alto | Médio |
| 2 | Propagar em routes | Alto | Baixo |
| 3 | Assinatura client-side | Alto | Alto |
| 4 | UI Step-Up | Médio | Médio |
| 5 | Risk level middleware | Médio | Baixo |

---

## Fluxo Final Desejado

```
Usuário                Frontend              UBL                 Ledger
   │                      │                    │                    │
   │──Login (passkey)────►│                    │                    │
   │                      │──verify passkey───►│                    │
   │                      │◄──session+tenant───│                    │
   │                      │                    │                    │
   │──Enviar mensagem────►│                    │                    │
   │  (L1, automático)    │──sign Ed25519─────►│                    │
   │                      │◄──verify sig──────►│──append entry─────►│
   │◄─────ok──────────────│                    │                    │
   │                      │                    │                    │
   │──Deletar membro─────►│                    │                    │
   │  (L4, step-up)       │                    │                    │
   │◄─passkey needed──────│                    │                    │
   │──touch passkey──────►│                    │                    │
   │                      │──verify stepup────►│                    │
   │                      │──sign Ed25519─────►│                    │
   │                      │◄──verify sig──────►│──append entry─────►│
   │◄─────ok──────────────│                    │                    │
```
