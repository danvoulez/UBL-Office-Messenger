![C.Tenant • 🟠 Laranja (Organization)](https://img.shields.io/badge/C.Tenant-🟠%20Laranja%20(Organization)-orange)

# 🟠 C.Tenant — Organization Management

**Path:** `containers/C.Tenant`  
**Role/Cor:** Laranja (Organization)  
**Zona:** LAB 256 (API)  

## Credenciais necessárias
- **Passkey (ubl-id)**: usuário logado

## Função
Container para gestão de tenants (organizações) e seus membros:
- Criação de tenants
- Geração de invite codes
- Join de novos membros
- Roles (owner, admin, member)

## Entradas permitidas (Inbound)
- Requests de usuários autenticados
- SSE do ledger

## Saídas permitidas (Outbound)
- kernel (signing_bytes/validate/commit)

## Dados que passam por aqui
- Tenant metadata, Member lists, Invite codes

## Eventos Suportados

### Tenant Events
- `tenant.created` - Nova organização criada
- `tenant.updated` - Metadata atualizada
- `tenant.deleted` - Organização removida (soft delete)

### Member Events
- `tenant.member.invited` - Convite gerado
- `tenant.member.joined` - Membro entrou via invite
- `tenant.member.left` - Membro saiu
- `tenant.member.role_changed` - Role alterada

## Intent Classes

| Event | Intent Class | Physics Delta |
|-------|-------------|---------------|
| `tenant.created` | Observation | 0 |
| `tenant.updated` | Observation | 0 |
| `tenant.member.invited` | Observation | 0 |
| `tenant.member.joined` | Observation | 0 |
| `tenant.member.role_changed` | Observation | 0 |

## Policy
- **Risk Level**: L2 (tenant management)
- **Trust Level**: L2 (owner/admin action)

## Data Model

```
Tenant
├── tenant_id: TEXT (PK)
├── name: TEXT
├── slug: TEXT (unique, URL-friendly)
├── status: active | suspended | deleted
├── created_by: TEXT (sid)
├── created_at: TIMESTAMPTZ

TenantMember
├── tenant_id: TEXT (FK)
├── sid: TEXT (FK → id_subject)
├── role: owner | admin | member
├── joined_at: TIMESTAMPTZ

InviteCode
├── code: TEXT (PK) - formato XXXX-XXXX
├── tenant_id: TEXT (FK)
├── created_by: TEXT (sid)
├── expires_at: TIMESTAMPTZ
├── max_uses: INT
├── uses: INT
├── status: active | expired | revoked
```

## API Routes

```
POST   /tenant              → Criar tenant (retorna invite code)
GET    /tenant              → Meu tenant atual
GET    /tenant/members      → Listar membros
POST   /tenant/invite       → Gerar novo invite code
POST   /tenant/join         → Entrar via invite code
DELETE /tenant/member/:sid  → Remover membro (owner/admin)
```

## Done if…
- Usuário pode criar tenant e receber invite code
- Usuário pode entrar em tenant via invite code
- Session inclui tenant_id
- Queries filtram por tenant_id correto
