# C.Tenant Design Document

## Visão Geral

O **C.Tenant** é o container UBL responsável pela multitenancy (multi-inquilinos) do sistema. Ele permite que múltiplas organizações compartilhem a mesma infraestrutura mantendo isolamento lógico completo.

---

## 1. Problema que Resolve

### Antes do C.Tenant
- `tenant_id` era hardcoded como `"default"` em todo o código
- Não havia isolamento entre usuários de diferentes organizações
- Não havia mecanismo de convite/ingresso
- Todas as queries retornavam dados de todos os usuários

### Depois do C.Tenant
- Cada usuário pertence a um tenant específico
- Dados são isolados por `tenant_id` em todas as projections
- Convites com códigos únicos (XXXX-XXXX)
- Hierarquia de papéis: Owner → Admin → Member

---

## 2. Modelo de Dados

### Tabelas

```
┌─────────────────┐       ┌────────────────────┐       ┌─────────────────┐
│   id_tenant     │       │  id_tenant_member  │       │   id_subject    │
├─────────────────┤       ├────────────────────┤       ├─────────────────┤
│ tenant_id (PK)  │◄──────│ tenant_id (PK,FK)  │       │ sid (PK)        │
│ name            │       │ sid (PK,FK)        │──────►│ display_name    │
│ slug (UNIQUE)   │       │ role               │       │ kind            │
│ status          │       │ joined_at          │       │ default_tenant  │──┐
│ settings (JSONB)│       └────────────────────┘       └─────────────────┘  │
│ created_by      │                                                          │
│ created_at      │◄─────────────────────────────────────────────────────────┘
└─────────────────┘

┌──────────────────┐
│  id_invite_code  │
├──────────────────┤
│ code (PK)        │  ← formato XXXX-XXXX
│ tenant_id (FK)   │
│ created_by       │
│ expires_at       │
│ max_uses         │
│ uses             │
│ status           │
└──────────────────┘
```

### Relacionamentos

- **id_tenant** 1:N **id_tenant_member**: Um tenant tem muitos membros
- **id_subject** N:M **id_tenant** (via id_tenant_member): Um usuário pode pertencer a múltiplos tenants
- **id_subject.default_tenant_id**: Referência rápida ao tenant principal do usuário
- **id_invite_code** N:1 **id_tenant**: Cada código pertence a um tenant

---

## 3. Fluxo do Usuário

```
                    ┌─────────────┐
                    │   Registro  │
                    │  (WebAuthn) │
                    └──────┬──────┘
                           │
                           ▼
                    ┌─────────────┐
                    │  Onboarding │
                    └──────┬──────┘
                           │
              ┌────────────┴────────────┐
              │                         │
              ▼                         ▼
     ┌────────────────┐       ┌────────────────┐
     │ Criar Organiz. │       │ Entrar c/Código│
     └────────┬───────┘       └────────┬───────┘
              │                        │
              ▼                        ▼
     ┌────────────────┐       ┌────────────────┐
     │  Vira OWNER    │       │  Vira MEMBER   │
     │  Recebe código │       │                │
     └────────┬───────┘       └────────┬───────┘
              │                        │
              └────────────┬───────────┘
                           │
                           ▼
                    ┌─────────────┐
                    │  Messenger  │
                    │ (dentro do  │
                    │   tenant)   │
                    └─────────────┘
```

---

## 4. API Reference

### Endpoints

| Método | Rota | Descrição | Auth | Role Mínimo |
|--------|------|-----------|------|-------------|
| POST | `/tenant` | Criar tenant | ✓ | - (novo) |
| GET | `/tenant` | Obter meu tenant | ✓ | member |
| GET | `/tenant/members` | Listar membros | ✓ | member |
| POST | `/tenant/invite` | Criar código | ✓ | admin |
| POST | `/tenant/join` | Usar código | ✓ | - (novo) |

### Códigos de Convite

Formato: `XXXX-XXXX` (caracteres: A-Z sem O/I, 2-9 sem 0/1)

Propriedades:
- `max_uses`: Quantas vezes pode ser usado
- `expires_at`: Quando expira
- `uses`: Quantas vezes já foi usado
- `status`: active | expired | revoked

---

## 5. Implementação Ideal vs Atual

### O que foi implementado ✅

1. **Schema SQL completo** (002_tenant.sql)
   - Tabelas com constraints corretos
   - Funções helper em PL/pgSQL
   - Índices para performance

2. **Módulo Rust** (src/tenant/)
   - types.rs: Structs e enums serializáveis
   - db.rs: CRUD com queries dinâmicas
   - routes.rs: Endpoints HTTP com autenticação
   - mod.rs: Exports

3. **Frontend** (OnboardingPage.tsx)
   - UI para criar/entrar em tenant
   - Exibição de código de convite com copy
   - Demo mode fallback

### O que falta para produção 🔄

1. **Propagar tenant_id nas projections**
   ```rust
   // Atual (hardcoded)
   let tenant_id = "default";
   
   // Ideal (extraído da sessão)
   let tenant_id = session.tenant_id.as_deref()
       .ok_or((StatusCode::FORBIDDEN, "No tenant"))?;
   ```

2. **Adicionar tenant_id em todas as queries**
   ```sql
   -- Antes
   SELECT * FROM projection_jobs WHERE job_id = $1;
   
   -- Depois
   SELECT * FROM projection_jobs 
   WHERE tenant_id = $1 AND job_id = $2;
   ```

3. **Eventos no ledger** (não apenas projection)
   ```json
   {
     "type": "tenant.created",
     "tenant_id": "tenant_abc123",
     "name": "Acme Corp",
     "created_by": "sid_xyz"
   }
   ```

4. **Mudar de tenant** (para usuários multi-tenant)
   - `PUT /tenant/switch` → Muda `default_tenant_id`
   - UI no header para seleção

---

## 6. Decisões de Design

### Por que não usar UUID para tenant_id?

Usamos `tenant_abc123` (prefixo + sufixo) porque:
1. Legibilidade em logs e debug
2. Prefixo identifica o tipo de entidade
3. Consistente com outros IDs do sistema (sid_, job_, msg_)

### Por que convites em vez de auto-join?

1. **Segurança**: Apenas quem tem o código pode entrar
2. **Controle**: Admin sabe quantas pessoas convidou
3. **Auditoria**: Log de quem usou cada código
4. **Expiração**: Códigos temporários por design

### Por que um tenant "default" para migração?

Para não quebrar dados existentes:
1. Usuários sem tenant → pertencem a "default"
2. Dados sem tenant_id → assumem "default"
3. Migração gradual possível

---

## 7. Segurança

### Isolamento de Dados

```sql
-- Toda query DEVE incluir tenant_id
-- Nunca confiar em parâmetros do cliente para tenant_id
-- Sempre extrair da sessão autenticada

-- ❌ ERRADO
SELECT * FROM messages WHERE id = $1;

-- ✅ CERTO
SELECT * FROM messages 
WHERE tenant_id = $1 AND id = $2;
-- $1 vem da sessão, $2 vem do request
```

### Hierarquia de Papéis

```
OWNER (1 por tenant)
  └── Pode: TUDO + deletar tenant + transferir ownership
  
ADMIN
  └── Pode: convidar, remover membros, configurar
  
MEMBER
  └── Pode: usar o sistema, ver dados do tenant
```

---

## 8. Próximos Passos

1. [ ] Aplicar migrations em produção
2. [ ] Atualizar todas as projections com tenant_id
3. [ ] Adicionar eventos tenant.* ao ledger
4. [ ] Criar UI de gerenciamento de membros
5. [ ] Implementar switch de tenant
6. [ ] Adicionar rate limiting por tenant

---

## 9. Referências

- **SPEC-UBL-TENANT v1.0** (este documento)
- **Container README**: `/ubl/containers/C.Tenant/README.md`
- **Schema SQL**: `/ubl/sql/00_base/002_tenant.sql`
- **Implementação Rust**: `/ubl/kernel/rust/ubl-server/src/tenant/`
