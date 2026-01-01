//! Session Manager - Centralized session lifecycle
//!
//! Phase 5: Unified session management across UBL Kernel.
//! Sessions are:
//! - Created after successful authentication
//! - Validated on each request
//! - Refreshed periodically
//! - Stored in database with proper indexing
//!
//! Note: Uses runtime queries (not compile-time checked) because
//! the id_sessions table may not exist yet during initial builds.

use std::sync::Arc;
use sqlx::{PgPool, Row};
use time::OffsetDateTime;
use uuid::Uuid;

use super::config::config;
use super::error::{IdentityError, IdentityResult};

/// Session information
#[derive(Debug, Clone)]
pub struct SessionInfo {
    /// Session token (opaque, used in Authorization header)
    pub token: String,
    /// Subject ID (SID) that owns this session
    pub sid: Uuid,
    /// Session kind (regular, stepup, machine)
    pub kind: SessionKind,
    /// When the session was created
    pub created_at: OffsetDateTime,
    /// When the session expires
    pub expires_at: OffsetDateTime,
    /// Last activity timestamp
    pub last_activity: OffsetDateTime,
    /// Optional: tenant context
    pub tenant_id: Option<String>,
}

/// Kind of session
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionKind {
    /// Regular user session
    Regular,
    /// Step-up session (elevated privileges, shorter TTL)
    StepUp,
    /// Machine/service session
    Machine,
}

impl SessionKind {
    /// Get the session kind as a string
    pub fn as_str(&self) -> &'static str {
        match self {
            SessionKind::Regular => "regular",
            SessionKind::StepUp => "stepup",
            SessionKind::Machine => "machine",
        }
    }

    /// Parse from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "regular" => Some(SessionKind::Regular),
            "stepup" => Some(SessionKind::StepUp),
            "machine" => Some(SessionKind::Machine),
            _ => None,
        }
    }
}

/// Session Manager - handles creation, validation, and lifecycle of sessions
pub struct SessionManager {
    pool: Arc<PgPool>,
}

impl SessionManager {
    /// Create a new SessionManager
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    /// Create a new session for a subject
    pub async fn create(
        &self,
        sid: Uuid,
        kind: SessionKind,
        tenant_id: Option<String>,
    ) -> IdentityResult<SessionInfo> {
        let cfg = config();
        
        // Determine TTL based on session kind
        let ttl_hours = match kind {
            SessionKind::Regular => cfg.session.regular_ttl_hours,
            SessionKind::StepUp => cfg.session.stepup_ttl_hours,
            SessionKind::Machine => cfg.session.regular_ttl_hours * 24, // Machines get longer TTL
        };

        let now = OffsetDateTime::now_utc();
        let expires_at = now + time::Duration::hours(ttl_hours as i64);
        
        // Generate secure token
        let token = self.generate_token();

        // Store in database
        sqlx::query!(
            r#"
            INSERT INTO id_sessions (token, sid, kind, created_at, expires_at, last_activity, tenant_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
            token,
            sid,
            kind.as_str(),
            now,
            expires_at,
            now,
            tenant_id
        )
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

        Ok(SessionInfo {
            token,
            sid,
            kind,
            created_at: now,
            expires_at,
            last_activity: now,
            tenant_id,
        })
    }

    /// Validate a session token
    /// Returns the session info if valid, error otherwise
    pub async fn validate(&self, token: &str) -> IdentityResult<SessionInfo> {
        let row = sqlx::query!(
            r#"
            SELECT sid, kind, created_at, expires_at, last_activity, tenant_id
            FROM id_sessions
            WHERE token = $1
            "#,
            token
        )
        .fetch_optional(self.pool.as_ref())
        .await
        .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

        let row = row.ok_or(IdentityError::SessionNotFound)?;

        let now = OffsetDateTime::now_utc();
        
        // Check expiration
        if row.expires_at < now {
            // Clean up expired session
            let _ = self.revoke(token).await;
            return Err(IdentityError::SessionExpired);
        }

        let kind = SessionKind::from_str(&row.kind)
            .unwrap_or(SessionKind::Regular);

        Ok(SessionInfo {
            token: token.to_string(),
            sid: row.sid,
            kind,
            created_at: row.created_at,
            expires_at: row.expires_at,
            last_activity: row.last_activity,
            tenant_id: row.tenant_id,
        })
    }

    /// Touch a session (update last_activity)
    pub async fn touch(&self, token: &str) -> IdentityResult<()> {
        let now = OffsetDateTime::now_utc();
        
        sqlx::query!(
            r#"
            UPDATE id_sessions
            SET last_activity = $1
            WHERE token = $2
            "#,
            now,
            token
        )
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Revoke a session
    pub async fn revoke(&self, token: &str) -> IdentityResult<()> {
        sqlx::query!(
            r#"
            DELETE FROM id_sessions
            WHERE token = $1
            "#,
            token
        )
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Revoke all sessions for a subject
    pub async fn revoke_all(&self, sid: Uuid) -> IdentityResult<u64> {
        let result = sqlx::query!(
            r#"
            DELETE FROM id_sessions
            WHERE sid = $1
            "#,
            sid
        )
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

        Ok(result.rows_affected())
    }

    /// List active sessions for a subject
    pub async fn list_for_subject(&self, sid: Uuid) -> IdentityResult<Vec<SessionInfo>> {
        let now = OffsetDateTime::now_utc();
        
        let rows = sqlx::query!(
            r#"
            SELECT token, kind, created_at, expires_at, last_activity, tenant_id
            FROM id_sessions
            WHERE sid = $1 AND expires_at > $2
            ORDER BY last_activity DESC
            "#,
            sid,
            now
        )
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|row| SessionInfo {
                token: row.token,
                sid,
                kind: SessionKind::from_str(&row.kind).unwrap_or(SessionKind::Regular),
                created_at: row.created_at,
                expires_at: row.expires_at,
                last_activity: row.last_activity,
                tenant_id: row.tenant_id,
            })
            .collect())
    }

    /// Cleanup expired sessions (should be called periodically)
    pub async fn cleanup_expired(&self) -> IdentityResult<u64> {
        let now = OffsetDateTime::now_utc();
        
        let result = sqlx::query!(
            r#"
            DELETE FROM id_sessions
            WHERE expires_at < $1
            "#,
            now
        )
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

        tracing::info!("Session cleanup: removed {} expired sessions", result.rows_affected());
        Ok(result.rows_affected())
    }

    /// Generate a cryptographically secure session token
    fn generate_token(&self) -> String {
        use rand::RngCore;
        let mut bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut bytes);
        format!("ses_{}", hex::encode(bytes))
    }
}
