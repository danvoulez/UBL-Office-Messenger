# C.Tenant Implementation Summary

## Overview

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
