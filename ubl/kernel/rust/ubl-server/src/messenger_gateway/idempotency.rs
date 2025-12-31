//! Idempotency Management (Fix #4: Persistent)
//!
//! Prevents duplicate actions using tenant-scoped idempotency keys.
//! Format: `idem:{tenant_id}:{action_type}:{resource_id}:{nonce}`
//!
//! FIXED: Now uses Postgres instead of in-memory HashMap to survive restarts.

use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use time::OffsetDateTime;
use tracing::{debug, warn};

/// Persistent idempotency store backed by Postgres
#[derive(Clone)]
pub struct IdempotencyStore {
    pool: PgPool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdempotencyRecord {
    pub status: String, // "pending", "completed", "failed"
    pub response_body: Option<serde_json::Value>,
    pub created_event_ids: Vec<String>,
    pub created_at: OffsetDateTime,
}

impl IdempotencyStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Check if idempotency key exists and return cached response
    pub async fn check(&self, key: &str) -> Option<IdempotencyRecord> {
        let result = sqlx::query_as::<_, IdempotencyRow>(
            r#"
            SELECT status, response_body, event_ids, created_at
            FROM gateway_idempotency
            WHERE idem_key = $1
            "#,
        )
        .bind(key)
        .fetch_optional(&self.pool)
        .await;

        match result {
            Ok(Some(row)) => {
                debug!(key = %key, "Idempotency key found");
                Some(IdempotencyRecord {
                    status: row.status,
                    response_body: row.response_body,
                    created_event_ids: row.event_ids,
                    created_at: row.created_at,
                })
            }
            Ok(None) => None,
            Err(e) => {
                warn!(key = %key, error = %e, "Failed to check idempotency key");
                None
            }
        }
    }

    /// Store idempotency record
    pub async fn store(&self, key: String, tenant_id: &str, record: IdempotencyRecord) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO gateway_idempotency (idem_key, tenant_id, status, response_body, event_ids)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (idem_key) DO UPDATE SET
                status = EXCLUDED.status,
                response_body = EXCLUDED.response_body,
                event_ids = EXCLUDED.event_ids
            "#,
        )
        .bind(&key)
        .bind(tenant_id)
        .bind(&record.status)
        .bind(&record.response_body)
        .bind(&record.created_event_ids)
        .execute(&self.pool)
        .await?;
        
        debug!(key = %key, status = %record.status, "Stored idempotency record");
        Ok(())
    }

    /// Update status of existing record
    pub async fn update_status(&self, key: &str, status: &str, response: Option<serde_json::Value>, event_ids: Vec<String>) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE gateway_idempotency 
            SET status = $2, response_body = $3, event_ids = $4
            WHERE idem_key = $1
            "#,
        )
        .bind(key)
        .bind(status)
        .bind(&response)
        .bind(&event_ids)
        .execute(&self.pool)
        .await?;
        
        debug!(key = %key, status = %status, "Updated idempotency record");
        Ok(())
    }

    /// Generate idempotency key
    pub fn generate_key(
        tenant_id: &str,
        action_type: &str,
        resource_id: &str,
        nonce: &str,
    ) -> String {
        format!("idem:{}:{}:{}:{}", tenant_id, action_type, resource_id, nonce)
    }

    /// Cleanup old records (call periodically)
    pub async fn cleanup_old(&self, max_age_hours: i64) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            r#"
            DELETE FROM gateway_idempotency
            WHERE created_at < NOW() - INTERVAL '1 hour' * $1
            "#,
        )
        .bind(max_age_hours)
        .execute(&self.pool)
        .await?;
        
        let count = result.rows_affected();
        if count > 0 {
            debug!(count = count, "Cleaned up old idempotency records");
        }
        Ok(count)
    }
}

/// Internal row type for sqlx
#[derive(sqlx::FromRow)]
struct IdempotencyRow {
    status: String,
    response_body: Option<serde_json::Value>,
    event_ids: Vec<String>,
    created_at: OffsetDateTime,
}
