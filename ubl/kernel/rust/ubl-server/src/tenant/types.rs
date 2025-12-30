//! # UBL Tenant Types
//!
//! Types for multi-tenancy support

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

// ============================================================================
// TENANT
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tenant {
    pub tenant_id: String,
    pub name: String,
    pub slug: String,
    pub status: TenantStatus,
    pub settings: serde_json::Value,
    pub created_by: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TenantStatus {
    Active,
    Suspended,
    Deleted,
}

impl Default for TenantStatus {
    fn default() -> Self {
        Self::Active
    }
}

// ============================================================================
// TENANT MEMBER
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantMember {
    pub tenant_id: String,
    pub sid: String,
    pub role: MemberRole,
    #[serde(with = "time::serde::rfc3339")]
    pub joined_at: OffsetDateTime,
    // Joined fields from id_subject
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MemberRole {
    Owner,
    Admin,
    Member,
}

impl Default for MemberRole {
    fn default() -> Self {
        Self::Member
    }
}

impl std::fmt::Display for MemberRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MemberRole::Owner => write!(f, "owner"),
            MemberRole::Admin => write!(f, "admin"),
            MemberRole::Member => write!(f, "member"),
        }
    }
}

// ============================================================================
// INVITE CODE
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InviteCode {
    pub code: String,
    pub tenant_id: String,
    pub created_by: String,
    #[serde(with = "time::serde::rfc3339")]
    pub expires_at: OffsetDateTime,
    pub max_uses: i32,
    pub uses: i32,
    pub status: InviteStatus,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum InviteStatus {
    Active,
    Expired,
    Revoked,
}

impl Default for InviteStatus {
    fn default() -> Self {
        Self::Active
    }
}

// ============================================================================
// REQUEST/RESPONSE TYPES
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateTenantRequest {
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct CreateTenantResponse {
    pub tenant: Tenant,
    pub invite_code: String,
}

#[derive(Debug, Deserialize)]
pub struct JoinTenantRequest {
    pub code: String,
}

#[derive(Debug, Serialize)]
pub struct JoinTenantResponse {
    pub tenant: Tenant,
}

#[derive(Debug, Serialize)]
pub struct GetTenantResponse {
    pub tenant: Tenant,
    pub role: MemberRole,
}

#[derive(Debug, Serialize)]
pub struct ListMembersResponse {
    pub members: Vec<TenantMember>,
}

#[derive(Debug, Deserialize)]
pub struct CreateInviteRequest {
    pub max_uses: Option<i32>,
    pub expires_hours: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct CreateInviteResponse {
    pub invite: InviteCode,
}
