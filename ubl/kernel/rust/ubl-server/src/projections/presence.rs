//! Presence Projection
//!
//! Computes entity presence from job state + activity.
//! Rules:
//! - offline: No activity > TTL (humans: 30min, agents: 5min)
//! - waiting_on_you: Human in waiting_on[] for waiting_input job
//! - working: Entity owns in_progress job + recent activity
//! - available: Default

use sqlx::PgPool;
use time::{OffsetDateTime, Duration};
use tracing::info;

/// Presence projection handler
pub struct PresenceProjection {
    pool: PgPool,
}

impl PresenceProjection {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Update presence for an entity based on activity
    pub async fn update_activity(
        &self,
        tenant_id: &str,
        entity_id: &str,
        entry_hash: &str,
    ) -> Result<(), sqlx::Error> {
        let now = OffsetDateTime::now_utc();

        // Update last_seen_at
        sqlx::query!(
            r#"
            INSERT INTO projection_presence (
                tenant_id, entity_id, state, since, last_seen_at, last_event_hash
            ) VALUES ($1, $2, 'available', $3, $3, $4)
            ON CONFLICT (entity_id) DO UPDATE SET
                last_seen_at = EXCLUDED.last_seen_at,
                last_event_hash = EXCLUDED.last_event_hash
            "#,
            tenant_id,
            entity_id,
            now,
            entry_hash
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Recompute presence based on job state
    pub async fn recompute_from_job(
        &self,
        tenant_id: &str,
        entity_id: &str,
        job_id: Option<&str>,
        job_state: Option<&str>,
        waiting_on: Option<Vec<String>>,
    ) -> Result<(), sqlx::Error> {
        let now = OffsetDateTime::now_utc();
        let state = if let Some(state_str) = job_state {
            match state_str {
                "in_progress" => "working",
                "waiting_input" => {
                    // Check if this entity is in waiting_on
                    if let Some(ref waiting) = waiting_on {
                        if waiting.contains(&entity_id.to_string()) {
                            "waiting_on_you"
                        } else {
                            "available"
                        }
                    } else {
                        "available"
                    }
                }
                _ => "available",
            }
        } else {
            "available"
        };

        sqlx::query!(
            r#"
            INSERT INTO projection_presence (
                tenant_id, entity_id, state, job_id, since, last_seen_at, last_event_hash
            ) VALUES ($1, $2, $3, $4, $5, $5, $6)
            ON CONFLICT (entity_id) DO UPDATE SET
                state = EXCLUDED.state,
                job_id = EXCLUDED.job_id,
                since = CASE WHEN projection_presence.state != EXCLUDED.state 
                    THEN EXCLUDED.since ELSE projection_presence.since END,
                last_seen_at = EXCLUDED.last_seen_at,
                last_event_hash = EXCLUDED.last_event_hash
            "#,
            tenant_id,
            entity_id,
            state,
            job_id,
            now,
            "recompute"
        )
        .execute(&self.pool)
        .await?;

        info!("👤 Presence updated: {} -> {}", entity_id, state);
        Ok(())
    }

    /// Check and update offline status based on TTL
    pub async fn check_offline(
        &self,
        tenant_id: &str,
        entity_id: &str,
        ttl_minutes: i64,
    ) -> Result<(), sqlx::Error> {
        let cutoff = OffsetDateTime::now_utc() - Duration::minutes(ttl_minutes);

        sqlx::query!(
            r#"
            UPDATE projection_presence
            SET state = 'offline'
            WHERE tenant_id = $1 AND entity_id = $2
              AND last_seen_at < $3
              AND state != 'offline'
            "#,
            tenant_id,
            entity_id,
            cutoff
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get presence for an entity
    pub async fn get_presence(
        &self,
        tenant_id: &str,
        entity_id: &str,
    ) -> Result<Option<serde_json::Value>, sqlx::Error> {
        let row = sqlx::query!(
            r#"
            SELECT state, job_id, since, last_seen_at
            FROM projection_presence
            WHERE tenant_id = $1 AND entity_id = $2
            "#,
            tenant_id,
            entity_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| {
            serde_json::json!({
                "entity_id": entity_id,
                "state": r.state,
                "job_id": r.job_id,
                "since": r.since.to_string(),
                "last_seen_at": r.last_seen_at.to_string(),
            })
        }))
    }
}

