//! Token Manager - ASC (Authorization Scope Certificate) lifecycle
//!
//! Phase 5: Unified ASC management.
//! ASCs are:
//! - Issued after authentication to grant specific permissions
//! - Scoped to containers, intent classes, and delta limits
//! - Time-bounded with not_before/not_after
//! - Signed by the issuing subject

use std::sync::Arc;
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

use super::error::{IdentityError, IdentityResult};

/// ASC Scopes - what the certificate allows
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AscScopes {
    /// Allowed containers (empty = all)
    #[serde(default)]
    pub containers: Vec<String>,
    /// Allowed intent classes (empty = all)
    #[serde(default)]
    pub intent_classes: Vec<String>,
    /// Maximum physics delta per operation
    #[serde(default)]
    pub max_delta: Option<i64>,
}

impl Default for AscScopes {
    fn default() -> Self {
        Self {
            containers: vec![],
            intent_classes: vec![],
            max_delta: None,
        }
    }
}

/// ASC (Authorization Scope Certificate)
#[derive(Debug, Clone)]
pub struct Asc {
    /// Unique ASC ID
    pub asc_id: Uuid,
    /// Subject ID that owns this ASC
    pub sid: Uuid,
    /// Subject kind (person, machine, etc.)
    pub owner_kind: String,
    /// Public key for signature verification
    pub public_key: String,
    /// Scopes granted by this ASC
    pub scopes: AscScopes,
    /// Not valid before this time
    pub not_before: OffsetDateTime,
    /// Not valid after this time
    pub not_after: OffsetDateTime,
    /// Signature over the certificate
    pub signature: String,
}

/// Request to issue a new ASC
pub struct IssueAscRequest {
    /// Subject ID requesting the ASC
    pub sid: Uuid,
    /// Subject kind
    pub owner_kind: String,
    /// Public key
    pub public_key: String,
    /// Requested scopes
    pub scopes: AscScopes,
    /// TTL in hours (default: 24)
    pub ttl_hours: Option<u64>,
}

/// Token Manager - handles ASC lifecycle
pub struct TokenManager {
    pool: Arc<PgPool>,
}

impl TokenManager {
    /// Create a new TokenManager
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    /// Issue a new ASC
    pub async fn issue(&self, request: IssueAscRequest) -> IdentityResult<Asc> {
        let now = OffsetDateTime::now_utc();
        let ttl_hours = request.ttl_hours.unwrap_or(24);
        let not_after = now + time::Duration::hours(ttl_hours as i64);
        
        let asc_id = Uuid::new_v4();
        let scopes_json = serde_json::to_value(&request.scopes)
            .map_err(|e| IdentityError::SerializationError(e.to_string()))?;

        // Generate signature placeholder (in production, this would be signed)
        let signature = self.generate_signature(&asc_id, &request.sid, &scopes_json);

        // Store in database
        sqlx::query!(
            r#"
            INSERT INTO id_asc (asc_id, sid, public_key, scopes, not_before, not_after, signature)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
            asc_id,
            request.sid,
            request.public_key,
            scopes_json,
            now,
            not_after,
            signature
        )
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

        Ok(Asc {
            asc_id,
            sid: request.sid,
            owner_kind: request.owner_kind,
            public_key: request.public_key,
            scopes: request.scopes,
            not_before: now,
            not_after,
            signature,
        })
    }

    /// Validate an ASC by ID
    /// Returns Ok if valid, Err with reason if invalid
    pub async fn validate(&self, asc_id: Uuid) -> IdentityResult<Asc> {
        let now = OffsetDateTime::now_utc();
        
        let row = sqlx::query!(
            r#"
            SELECT a.asc_id, a.sid, a.public_key, a.scopes, a.not_before, a.not_after, a.signature,
                   s.kind as owner_kind
            FROM id_asc a
            JOIN id_subjects s ON a.sid = s.sid
            WHERE a.asc_id = $1
            "#,
            asc_id
        )
        .fetch_optional(self.pool.as_ref())
        .await
        .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

        let row = row.ok_or(IdentityError::AscNotFound)?;

        // Check time bounds
        if row.not_before > now {
            return Err(IdentityError::AscNotYetValid);
        }
        if row.not_after < now {
            return Err(IdentityError::AscExpired);
        }

        // Parse scopes
        let scopes: AscScopes = serde_json::from_value(row.scopes)
            .map_err(|e| IdentityError::SerializationError(e.to_string()))?;

        Ok(Asc {
            asc_id: row.asc_id,
            sid: row.sid,
            owner_kind: row.owner_kind,
            public_key: row.public_key,
            scopes,
            not_before: row.not_before,
            not_after: row.not_after,
            signature: row.signature,
        })
    }

    /// Validate an ASC against specific requirements
    pub async fn validate_for_action(
        &self,
        asc_id: Uuid,
        container: &str,
        intent_class: &str,
        delta: i64,
    ) -> IdentityResult<Asc> {
        let asc = self.validate(asc_id).await?;

        // Check container scope
        if !asc.scopes.containers.is_empty() {
            let allowed = asc.scopes.containers.iter().any(|c| {
                c == "*" || container.starts_with(c)
            });
            if !allowed {
                return Err(IdentityError::AscScopeViolation(
                    format!("Container '{}' not in scope", container)
                ));
            }
        }

        // Check intent class scope
        if !asc.scopes.intent_classes.is_empty() {
            let allowed = asc.scopes.intent_classes.iter().any(|i| {
                i == "*" || i == intent_class
            });
            if !allowed {
                return Err(IdentityError::AscScopeViolation(
                    format!("Intent class '{}' not in scope", intent_class)
                ));
            }
        }

        // Check delta
        if let Some(max_delta) = asc.scopes.max_delta {
            if delta > max_delta {
                return Err(IdentityError::AscScopeViolation(
                    format!("Delta {} exceeds max {}", delta, max_delta)
                ));
            }
        }

        Ok(asc)
    }

    /// Revoke an ASC (sets not_after to now)
    pub async fn revoke(&self, asc_id: Uuid) -> IdentityResult<()> {
        let now = OffsetDateTime::now_utc();
        
        sqlx::query!(
            r#"
            UPDATE id_asc
            SET not_after = $1
            WHERE asc_id = $2
            "#,
            now,
            asc_id
        )
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Revoke all ASCs for a subject
    pub async fn revoke_all(&self, sid: Uuid) -> IdentityResult<u64> {
        let now = OffsetDateTime::now_utc();
        
        let result = sqlx::query!(
            r#"
            UPDATE id_asc
            SET not_after = $1
            WHERE sid = $2 AND not_after > $1
            "#,
            now,
            sid
        )
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

        Ok(result.rows_affected())
    }

    /// List active ASCs for a subject
    pub async fn list_for_subject(&self, sid: Uuid) -> IdentityResult<Vec<Asc>> {
        let now = OffsetDateTime::now_utc();
        
        let rows = sqlx::query!(
            r#"
            SELECT a.asc_id, a.public_key, a.scopes, a.not_before, a.not_after, a.signature,
                   s.kind as owner_kind
            FROM id_asc a
            JOIN id_subjects s ON a.sid = s.sid
            WHERE a.sid = $1 AND a.not_before <= $2 AND a.not_after > $2
            ORDER BY a.not_after DESC
            "#,
            sid,
            now
        )
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

        let mut ascs = Vec::with_capacity(rows.len());
        for row in rows {
            let scopes: AscScopes = serde_json::from_value(row.scopes)
                .map_err(|e| IdentityError::SerializationError(e.to_string()))?;
            
            ascs.push(Asc {
                asc_id: row.asc_id,
                sid,
                owner_kind: row.owner_kind,
                public_key: row.public_key,
                scopes,
                not_before: row.not_before,
                not_after: row.not_after,
                signature: row.signature,
            });
        }

        Ok(ascs)
    }

    /// Generate signature for ASC (placeholder - in production use proper signing)
    fn generate_signature(&self, asc_id: &Uuid, sid: &Uuid, scopes: &serde_json::Value) -> String {
        use sha2::{Sha256, Digest};
        
        let data = format!("{}:{}:{}", asc_id, sid, scopes);
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        let result = hasher.finalize();
        hex::encode(result)
    }
}
