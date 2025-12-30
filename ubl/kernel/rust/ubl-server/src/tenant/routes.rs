//! # UBL Tenant Routes
//!
//! HTTP endpoints for tenant management:
//! - POST /tenant - Create new tenant
//! - GET /tenant - Get current user's tenant
//! - GET /tenant/members - List tenant members
//! - POST /tenant/invite - Create invite code
//! - POST /tenant/join - Join tenant with invite code

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    routing::{get, post},
    Json, Router,
};
use serde_json::json;
use sqlx::PgPool;
use tracing::{error, info};

use super::db;
use super::types::*;

// ============================================================================
// SESSION HELPER
// ============================================================================

/// Simple user info from session
struct UserSession {
    sid: String,
}

/// Extract SID from session token
async fn get_session(pool: &PgPool, headers: &HeaderMap) -> Option<UserSession> {
    // Extract token from Authorization header or cookie
    let token = headers.get("authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .map(|s| s.to_string())
        .or_else(|| {
            headers.get("cookie")
                .and_then(|h| h.to_str().ok())
                .and_then(|c| {
                    c.split(';')
                        .find_map(|part| {
                            let mut kv = part.trim().splitn(2, '=');
                            match (kv.next(), kv.next()) {
                                (Some("session"), Some(v)) => Some(v.to_string()),
                                (Some("ubl_session"), Some(v)) => Some(v.to_string()),
                                _ => None
                            }
                        })
                })
        })?;
    
    // Validate session and get SID
    let session = crate::auth::session_db::get_valid(pool, &token).await.ok()??;
    
    Some(UserSession {
        sid: session.sid.to_string(),
    })
}

// ============================================================================
// HANDLERS
// ============================================================================

/// POST /tenant - Create new tenant
async fn create_tenant(
    State(pool): State<PgPool>,
    headers: HeaderMap,
    Json(req): Json<CreateTenantRequest>,
) -> Result<Json<CreateTenantResponse>, (StatusCode, Json<serde_json::Value>)> {
    let session = get_session(&pool, &headers).await.ok_or_else(|| {
        (StatusCode::UNAUTHORIZED, Json(json!({ "error": "Authentication required" })))
    })?;
    let sid = &session.sid;
    
    // Check if user already has a tenant
    if let Ok(Some(existing)) = db::get_user_tenant(&pool, sid).await {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "User already has a tenant", "tenant_id": existing }))
        ));
    }
    
    // Create tenant
    let tenant = db::create_tenant(&pool, &req.name, sid)
        .await
        .map_err(|e| {
            error!("Failed to create tenant: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Failed to create tenant" })))
        })?;
    
    // Add creator as owner
    db::add_member(&pool, &tenant.tenant_id, sid, MemberRole::Owner)
        .await
        .map_err(|e| {
            error!("Failed to add owner: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Failed to add owner" })))
        })?;
    
    // Create initial invite code
    let invite = db::create_invite(&pool, &tenant.tenant_id, sid, 100, 24 * 30) // 30 days, 100 uses
        .await
        .map_err(|e| {
            error!("Failed to create invite: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Failed to create invite" })))
        })?;
    
    info!("🏢 Tenant created: {} by {}", tenant.name, sid);
    
    Ok(Json(CreateTenantResponse {
        tenant,
        invite_code: invite.code,
    }))
}

/// GET /tenant - Get current user's tenant
async fn get_my_tenant(
    State(pool): State<PgPool>,
    headers: HeaderMap,
) -> Result<Json<GetTenantResponse>, (StatusCode, Json<serde_json::Value>)> {
    let session = get_session(&pool, &headers).await.ok_or_else(|| {
        (StatusCode::UNAUTHORIZED, Json(json!({ "error": "Authentication required" })))
    })?;
    let sid = &session.sid;
    
    // Get user's default tenant
    let tenant_id = db::get_user_tenant(&pool, sid)
        .await
        .map_err(|e| {
            error!("Failed to get user tenant: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Database error" })))
        })?;
    
    let tenant_id = match tenant_id {
        Some(id) => id,
        None => return Err((
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "User has no tenant" }))
        ))
    };
    
    // Get tenant details
    let tenant = db::get_tenant(&pool, &tenant_id)
        .await
        .map_err(|e| {
            error!("Failed to get tenant: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Database error" })))
        })?
        .ok_or_else(|| (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Tenant not found" }))
        ))?;
    
    // Get user's role
    let role = db::get_member_role(&pool, &tenant_id, sid)
        .await
        .map_err(|e| {
            error!("Failed to get member role: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Database error" })))
        })?
        .unwrap_or(MemberRole::Member);
    
    Ok(Json(GetTenantResponse { tenant, role }))
}

/// GET /tenant/members - List tenant members
async fn list_members(
    State(pool): State<PgPool>,
    headers: HeaderMap,
) -> Result<Json<ListMembersResponse>, (StatusCode, Json<serde_json::Value>)> {
    let session = get_session(&pool, &headers).await.ok_or_else(|| {
        (StatusCode::UNAUTHORIZED, Json(json!({ "error": "Authentication required" })))
    })?;
    let sid = &session.sid;
    
    // Get user's tenant
    let tenant_id = db::get_user_tenant(&pool, sid)
        .await
        .map_err(|e| {
            error!("Failed to get user tenant: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Database error" })))
        })?
        .ok_or_else(|| (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "User has no tenant" }))
        ))?;
    
    let members = db::list_members(&pool, &tenant_id)
        .await
        .map_err(|e| {
            error!("Failed to list members: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Database error" })))
        })?;
    
    Ok(Json(ListMembersResponse { members }))
}

/// POST /tenant/invite - Create new invite code
async fn create_invite(
    State(pool): State<PgPool>,
    headers: HeaderMap,
    Json(req): Json<CreateInviteRequest>,
) -> Result<Json<CreateInviteResponse>, (StatusCode, Json<serde_json::Value>)> {
    let session = get_session(&pool, &headers).await.ok_or_else(|| {
        (StatusCode::UNAUTHORIZED, Json(json!({ "error": "Authentication required" })))
    })?;
    let sid = &session.sid;
    
    // Get user's tenant
    let tenant_id = db::get_user_tenant(&pool, sid)
        .await
        .map_err(|e| {
            error!("Failed to get user tenant: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Database error" })))
        })?
        .ok_or_else(|| (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "User has no tenant" }))
        ))?;
    
    // Check user is owner or admin
    let role = db::get_member_role(&pool, &tenant_id, sid)
        .await
        .map_err(|e| {
            error!("Failed to get member role: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Database error" })))
        })?;
    
    match role {
        Some(MemberRole::Owner) | Some(MemberRole::Admin) => {}
        _ => return Err((
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "Only owner or admin can create invites" }))
        ))
    }
    
    let max_uses = req.max_uses.unwrap_or(100);
    let expires_hours = req.expires_hours.unwrap_or(24 * 7); // Default: 7 days
    
    let invite = db::create_invite(&pool, &tenant_id, sid, max_uses, expires_hours)
        .await
        .map_err(|e| {
            error!("Failed to create invite: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Failed to create invite" })))
        })?;
    
    Ok(Json(CreateInviteResponse { invite }))
}

/// POST /tenant/join - Join tenant with invite code
async fn join_tenant(
    State(pool): State<PgPool>,
    headers: HeaderMap,
    Json(req): Json<JoinTenantRequest>,
) -> Result<Json<JoinTenantResponse>, (StatusCode, Json<serde_json::Value>)> {
    let session = get_session(&pool, &headers).await.ok_or_else(|| {
        (StatusCode::UNAUTHORIZED, Json(json!({ "error": "Authentication required" })))
    })?;
    let sid = &session.sid;
    
    // Check if user already has a tenant
    if let Ok(Some(existing)) = db::get_user_tenant(&pool, sid).await {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "User already has a tenant", "tenant_id": existing }))
        ));
    }
    
    // Use invite code
    let tenant_id = db::use_invite(&pool, &req.code)
        .await
        .map_err(|e| {
            error!("Failed to use invite: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Database error" })))
        })?
        .ok_or_else(|| (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Invalid or expired invite code" }))
        ))?;
    
    // Add user as member
    db::add_member(&pool, &tenant_id, sid, MemberRole::Member)
        .await
        .map_err(|e| {
            error!("Failed to add member: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Failed to join tenant" })))
        })?;
    
    // Get tenant details
    let tenant = db::get_tenant(&pool, &tenant_id)
        .await
        .map_err(|e| {
            error!("Failed to get tenant: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Database error" })))
        })?
        .ok_or_else(|| (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Tenant not found" }))
        ))?;
    
    info!("👤 User {} joined tenant {}", sid, tenant.name);
    
    Ok(Json(JoinTenantResponse { tenant }))
}

// ============================================================================
// ROUTER
// ============================================================================

/// Build the tenant router
pub fn tenant_routes() -> Router<PgPool> {
    Router::new()
        .route("/tenant", post(create_tenant).get(get_my_tenant))
        .route("/tenant/members", get(list_members))
        .route("/tenant/invite", post(create_invite))
        .route("/tenant/join", post(join_tenant))
}
