//! # UBL Tenant Database Operations
//!
//! CRUD operations for tenants, members, and invite codes.
//! Uses dynamic queries for database-independent compilation.

use sqlx::{PgPool, Row};
use time::OffsetDateTime;
use tracing::{info, warn};

use super::types::*;

// ============================================================================
// TENANT OPERATIONS
// ============================================================================

/// Generate a URL-friendly slug from tenant name
fn slugify(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// Create a new tenant
pub async fn create_tenant(
    pool: &PgPool,
    name: &str,
    created_by: &str,
) -> Result<Tenant, sqlx::Error> {
    let tenant_id = format!("tenant_{}", &uuid::Uuid::new_v4().to_string().replace("-", "")[..12]);
    let base_slug = slugify(name);
    let slug = format!("{}-{}", base_slug, &tenant_id[7..12]); // Add unique suffix
    
    let row = sqlx::query(
        r#"
        INSERT INTO id_tenant (tenant_id, name, slug, created_by)
        VALUES ($1, $2, $3, $4)
        RETURNING tenant_id, name, slug, status, settings, created_by, created_at
        "#
    )
    .bind(&tenant_id)
    .bind(name)
    .bind(&slug)
    .bind(created_by)
    .fetch_one(pool)
    .await?;
    
    info!("🏢 Tenant created: {} ({})", name, tenant_id);
    
    Ok(Tenant {
        tenant_id: row.get("tenant_id"),
        name: row.get("name"),
        slug: row.get("slug"),
        status: TenantStatus::Active,
        settings: row.try_get::<serde_json::Value, _>("settings").unwrap_or_else(|_| serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
    })
}

/// Get tenant by ID
pub async fn get_tenant(pool: &PgPool, tenant_id: &str) -> Result<Option<Tenant>, sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT tenant_id, name, slug, status, settings, created_by, created_at
        FROM id_tenant
        WHERE tenant_id = $1 AND status != 'deleted'
        "#
    )
    .bind(tenant_id)
    .fetch_optional(pool)
    .await?;
    
    Ok(row.map(|r| {
        let status_str: String = r.get("status");
        Tenant {
            tenant_id: r.get("tenant_id"),
            name: r.get("name"),
            slug: r.get("slug"),
            status: match status_str.as_str() {
                "active" => TenantStatus::Active,
                "suspended" => TenantStatus::Suspended,
                _ => TenantStatus::Deleted,
            },
            settings: r.try_get::<serde_json::Value, _>("settings").unwrap_or_else(|_| serde_json::json!({})),
            created_by: r.get("created_by"),
            created_at: r.get("created_at"),
        }
    }))
}

// ============================================================================
// MEMBER OPERATIONS
// ============================================================================

/// Add member to tenant
pub async fn add_member(
    pool: &PgPool,
    tenant_id: &str,
    sid: &str,
    role: MemberRole,
) -> Result<(), sqlx::Error> {
    let role_str = role.to_string();
    
    sqlx::query(
        r#"
        INSERT INTO id_tenant_member (tenant_id, sid, role)
        VALUES ($1, $2, $3)
        ON CONFLICT (tenant_id, sid) DO UPDATE SET role = EXCLUDED.role
        "#
    )
    .bind(tenant_id)
    .bind(sid)
    .bind(&role_str)
    .execute(pool)
    .await?;
    
    // Update user's default tenant if they don't have one
    sqlx::query(
        r#"
        UPDATE id_subject 
        SET default_tenant_id = $1 
        WHERE sid = $2 AND default_tenant_id IS NULL
        "#
    )
    .bind(tenant_id)
    .bind(sid)
    .execute(pool)
    .await?;
    
    info!("👤 Member added: {} to {} as {}", sid, tenant_id, role_str);
    
    Ok(())
}

/// Get member's role in tenant
pub async fn get_member_role(
    pool: &PgPool,
    tenant_id: &str,
    sid: &str,
) -> Result<Option<MemberRole>, sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT role FROM id_tenant_member
        WHERE tenant_id = $1 AND sid = $2
        "#
    )
    .bind(tenant_id)
    .bind(sid)
    .fetch_optional(pool)
    .await?;
    
    Ok(row.map(|r| {
        let role: String = r.get("role");
        match role.as_str() {
            "owner" => MemberRole::Owner,
            "admin" => MemberRole::Admin,
            _ => MemberRole::Member,
        }
    }))
}

/// List members of a tenant
pub async fn list_members(
    pool: &PgPool,
    tenant_id: &str,
) -> Result<Vec<TenantMember>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT m.tenant_id, m.sid, m.role, m.joined_at, s.display_name, s.kind
        FROM id_tenant_member m
        JOIN id_subject s ON m.sid = s.sid
        WHERE m.tenant_id = $1
        ORDER BY m.joined_at ASC
        "#
    )
    .bind(tenant_id)
    .fetch_all(pool)
    .await?;
    
    Ok(rows.into_iter().map(|r| {
        let role: String = r.get("role");
        TenantMember {
            tenant_id: r.get("tenant_id"),
            sid: r.get("sid"),
            role: match role.as_str() {
                "owner" => MemberRole::Owner,
                "admin" => MemberRole::Admin,
                _ => MemberRole::Member,
            },
            joined_at: r.get("joined_at"),
            display_name: r.try_get("display_name").ok(),
            kind: r.try_get("kind").ok(),
        }
    }).collect())
}

/// Get user's default tenant
pub async fn get_user_tenant(pool: &PgPool, sid: &str) -> Result<Option<String>, sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT default_tenant_id FROM id_subject
        WHERE sid = $1
        "#
    )
    .bind(sid)
    .fetch_optional(pool)
    .await?;
    
    Ok(row.and_then(|r| r.try_get::<Option<String>, _>("default_tenant_id").ok().flatten()))
}

// ============================================================================
// INVITE CODE OPERATIONS
// ============================================================================

/// Generate random invite code (XXXX-XXXX)
fn generate_code() -> String {
    use rand::Rng;
    let chars: Vec<char> = "ABCDEFGHJKLMNPQRSTUVWXYZ23456789".chars().collect();
    let mut rng = rand::thread_rng();
    
    let mut code = String::new();
    for i in 0..8 {
        if i == 4 {
            code.push('-');
        }
        code.push(chars[rng.gen_range(0..chars.len())]);
    }
    code
}

/// Create a new invite code
pub async fn create_invite(
    pool: &PgPool,
    tenant_id: &str,
    created_by: &str,
    max_uses: i32,
    expires_hours: i32,
) -> Result<InviteCode, sqlx::Error> {
    let code = generate_code();
    let expires_at = OffsetDateTime::now_utc() + time::Duration::hours(expires_hours as i64);
    
    let row = sqlx::query(
        r#"
        INSERT INTO id_invite_code (code, tenant_id, created_by, expires_at, max_uses)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING code, tenant_id, created_by, expires_at, max_uses, uses, status, created_at
        "#
    )
    .bind(&code)
    .bind(tenant_id)
    .bind(created_by)
    .bind(expires_at)
    .bind(max_uses)
    .fetch_one(pool)
    .await?;
    
    info!("🎟️ Invite created: {} for {}", code, tenant_id);
    
    Ok(InviteCode {
        code: row.get("code"),
        tenant_id: row.get("tenant_id"),
        created_by: row.get("created_by"),
        expires_at: row.get("expires_at"),
        max_uses: row.get("max_uses"),
        uses: row.get("uses"),
        status: InviteStatus::Active,
        created_at: row.get("created_at"),
    })
}

/// Validate and use an invite code
pub async fn use_invite(pool: &PgPool, code: &str) -> Result<Option<String>, sqlx::Error> {
    // Normalize code (uppercase, ensure dash)
    let code_normalized = code.to_uppercase().replace(' ', "");
    
    let row = sqlx::query(
        r#"
        UPDATE id_invite_code 
        SET uses = uses + 1
        WHERE code = $1 
          AND status = 'active'
          AND expires_at > NOW()
          AND uses < max_uses
        RETURNING tenant_id
        "#
    )
    .bind(&code_normalized)
    .fetch_optional(pool)
    .await?;
    
    match &row {
        Some(r) => {
            let tenant_id: String = r.get("tenant_id");
            info!("🎟️ Invite used: {} → tenant {}", code_normalized, tenant_id);
            Ok(Some(tenant_id))
        }
        None => {
            warn!("🎟️ Invalid invite: {}", code_normalized);
            Ok(None)
        }
    }
}

/// Get invite code info
pub async fn get_invite(pool: &PgPool, code: &str) -> Result<Option<InviteCode>, sqlx::Error> {
    let code_normalized = code.to_uppercase().replace(' ', "");
    
    let row = sqlx::query(
        r#"
        SELECT code, tenant_id, created_by, expires_at, max_uses, uses, status, created_at
        FROM id_invite_code
        WHERE code = $1
        "#
    )
    .bind(&code_normalized)
    .fetch_optional(pool)
    .await?;
    
    Ok(row.map(|r| {
        let status_str: String = r.get("status");
        InviteCode {
            code: r.get("code"),
            tenant_id: r.get("tenant_id"),
            created_by: r.get("created_by"),
            expires_at: r.get("expires_at"),
            max_uses: r.get("max_uses"),
            uses: r.get("uses"),
            status: match status_str.as_str() {
                "active" => InviteStatus::Active,
                "expired" => InviteStatus::Expired,
                _ => InviteStatus::Revoked,
            },
            created_at: r.get("created_at"),
        }
    }))
}
