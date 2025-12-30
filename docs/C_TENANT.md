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


---

# Implementation Summary

C.Tenant is the multi-tenancy container for UBL, enabling organization-based isolation and invite-based membership.

## Files Created/Modified

### New Files

1. **[ubl/containers/C.Tenant/README.md](ubl/containers/C.Tenant/README.md)**
   - Container specification
   - Event definitions
   - API documentation

2. **[ubl/sql/00_base/002_tenant.sql](ubl/sql/00_base/002_tenant.sql)**
   - `id_tenant` - Organizations table
   - `id_tenant_member` - User ↔ Tenant relationship (many-to-many with roles)
   - `id_invite_code` - Invite codes for joining
   - ALTER TABLE `id_subject` ADD `default_tenant_id`
   - Helper functions: `generate_invite_code()`, `is_invite_valid()`, `use_invite_code()`

3. **[ubl/kernel/rust/ubl-server/src/tenant/mod.rs](ubl/kernel/rust/ubl-server/src/tenant/mod.rs)**
   - Module exports

4. **[ubl/kernel/rust/ubl-server/src/tenant/types.rs](ubl/kernel/rust/ubl-server/src/tenant/types.rs)**
   - `Tenant`, `TenantMember`, `InviteCode` structs
   - `TenantStatus`, `MemberRole`, `InviteStatus` enums
   - Request/Response types

5. **[ubl/kernel/rust/ubl-server/src/tenant/db.rs](ubl/kernel/rust/ubl-server/src/tenant/db.rs)**
   - CRUD operations using dynamic sqlx queries
   - `create_tenant()`, `get_tenant()`, `add_member()`, `get_member_role()`, `list_members()`
   - `get_user_tenant()`, `create_invite()`, `use_invite()`, `get_invite()`

6. **[ubl/kernel/rust/ubl-server/src/tenant/routes.rs](ubl/kernel/rust/ubl-server/src/tenant/routes.rs)**
   - HTTP endpoints with session authentication
   - `POST /tenant` - Create tenant (user becomes owner)
   - `GET /tenant` - Get current user's tenant
   - `GET /tenant/members` - List members
   - `POST /tenant/invite` - Create invite code (owner/admin only)
   - `POST /tenant/join` - Join with invite code

### Modified Files

1. **[ubl/kernel/rust/ubl-server/src/main.rs](ubl/kernel/rust/ubl-server/src/main.rs)**
   - Added `mod tenant;`
   - Added `.merge(tenant::tenant_routes().with_state(pool.clone()))`

2. **[apps/messenger/frontend/src/pages/OnboardingPage.tsx](apps/messenger/frontend/src/pages/OnboardingPage.tsx)**
   - Updated to use new `/tenant` and `/tenant/join` APIs
   - Added invite code display after creation with copy button
   - Better error handling for join failures

## API Reference

### POST /tenant
Create a new tenant. User becomes owner.

**Request:**
```json
{
  "name": "Acme Corp"
}
```

**Response:**
```json
{
  "tenant": {
    "tenant_id": "tenant_a1b2c3d4e5f6",
    "name": "Acme Corp",
    "slug": "acme-corp-a1b2c",
    "status": "active",
    "settings": {},
    "created_by": "sid_...",
    "created_at": "2024-01-15T12:00:00Z"
  },
  "invite_code": "ABCD-1234"
}
```

### GET /tenant
Get current user's tenant.

**Response:**
```json
{
  "tenant": { ... },
  "role": "owner"
}
```

### POST /tenant/join
Join a tenant with invite code.

**Request:**
```json
{
  "code": "ABCD-1234"
}
```

**Response:**
```json
{
  "tenant": { ... }
}
```

### GET /tenant/members
List tenant members.

**Response:**
```json
{
  "members": [
    {
      "tenant_id": "tenant_...",
      "sid": "sid_...",
      "role": "owner",
      "joined_at": "2024-01-15T12:00:00Z",
      "display_name": "John Doe",
      "kind": "person"
    }
  ]
}
```

### POST /tenant/invite
Create a new invite code (owner/admin only).

**Request:**
```json
{
  "max_uses": 10,
  "expires_hours": 168
}
```

**Response:**
```json
{
  "invite": {
    "code": "WXYZ-5678",
    "tenant_id": "tenant_...",
    "expires_at": "2024-01-22T12:00:00Z",
    "max_uses": 10,
    "uses": 0,
    "status": "active"
  }
}
```

## Database Schema

```sql
-- Tenants (Organizations)
CREATE TABLE id_tenant (
  tenant_id   TEXT PRIMARY KEY,
  name        TEXT NOT NULL,
  slug        TEXT UNIQUE NOT NULL,
  status      TEXT DEFAULT 'active',
  settings    JSONB DEFAULT '{}',
  created_by  TEXT NOT NULL,
  created_at  TIMESTAMPTZ DEFAULT NOW()
);

-- Membership (many-to-many with roles)
CREATE TABLE id_tenant_member (
  tenant_id   TEXT REFERENCES id_tenant,
  sid         TEXT REFERENCES id_subject,
  role        TEXT DEFAULT 'member',
  joined_at   TIMESTAMPTZ DEFAULT NOW(),
  PRIMARY KEY (tenant_id, sid)
);

-- Invite Codes
CREATE TABLE id_invite_code (
  code        TEXT PRIMARY KEY,  -- XXXX-XXXX
  tenant_id   TEXT REFERENCES id_tenant,
  created_by  TEXT NOT NULL,
  expires_at  TIMESTAMPTZ NOT NULL,
  max_uses    INT DEFAULT 1,
  uses        INT DEFAULT 0,
  status      TEXT DEFAULT 'active'
);

-- User's default tenant
ALTER TABLE id_subject ADD default_tenant_id TEXT REFERENCES id_tenant;
```

## User Flow

1. **Registration** → User registers via WebAuthn passkey
2. **Onboarding** → User is redirected to OnboardingPage
3. **Choice**:
   - **Create Organization** → Enters name → Gets invite code to share
   - **Join with Invite** → Enters code → Joins as member
4. **Access** → User can now access the messenger within their tenant context

## Next Steps

1. **Apply migrations**: Run `002_tenant.sql` on the database
2. **Test API endpoints**: Use the frontend onboarding flow
3. **Update projections**: Modify `messenger_gateway/routes.rs` to extract `tenant_id` from session
4. **Add tenant context**: Pass `tenant_id` through all commands/queries

## Migration Command

```bash
psql $DATABASE_URL -f ubl/sql/00_base/002_tenant.sql
```
