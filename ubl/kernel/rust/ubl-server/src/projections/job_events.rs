//! Job Events Projection — Timeline items for job drawer
//!
//! Builds timeline items from job events for the job drawer UI.
//! Events: job.created, job.state_changed, tool.called, tool.result, approval.decided

use sqlx::PgPool;
use time::OffsetDateTime;
use tracing::{info, error};

/// Job events projection handler
pub struct JobEventsProjection {
    pool: PgPool,
}

impl JobEventsProjection {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Process a job event and add timeline item
    pub async fn process_event(
        &self,
        event_type: &str,
        atom: &serde_json::Value,
        entry_hash: &str,
        sequence: i64,
        tenant_id: &str,
    ) -> Result<(), sqlx::Error> {
        let job_id = atom.get("job_id")
            .or_else(|| atom.get("id"))
            .and_then(|v| v.as_str())
            .unwrap_or_default();

        if job_id.is_empty() {
            return Ok(());
        }

        let ts = OffsetDateTime::now_utc();
        let cursor = format!("{}:{}", sequence, ts.unix_timestamp());

        let timeline_item = self.build_timeline_item(event_type, atom, &ts)?;
        let actor_entity_id = atom.get("actor")
            .and_then(|a| a.get("entity_id"))
            .and_then(|v| v.as_str())
            .unwrap_or("system")
            .to_string();

        sqlx::query!(
            r#"
            INSERT INTO projection_job_events (
                tenant_id, job_id, cursor, ts, event_id, event_type,
                actor_entity_id, timeline_item
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (tenant_id, job_id, cursor) DO UPDATE SET
                timeline_item = EXCLUDED.timeline_item
            "#,
            tenant_id,
            job_id,
            cursor,
            ts,
            entry_hash,
            event_type,
            actor_entity_id,
            timeline_item
        )
        .execute(&self.pool)
        .await?;

        info!("📋 Job event timeline: {} {} seq={}", job_id, event_type, sequence);
        Ok(())
    }

    fn build_timeline_item(
        &self,
        event_type: &str,
        atom: &serde_json::Value,
        ts: &OffsetDateTime,
    ) -> Result<serde_json::Value, sqlx::Error> {
        let item = match event_type {
            "job.created" => serde_json::json!({
                "type": "job_created",
                "title": atom.get("title").and_then(|v| v.as_str()).unwrap_or("Job created"),
                "description": atom.get("description").and_then(|v| v.as_str()),
                "timestamp": ts.to_string(),
            }),
            "job.state_changed" => serde_json::json!({
                "type": "state_changed",
                "from": atom.get("from_state").and_then(|v| v.as_str()),
                "to": atom.get("to_state").and_then(|v| v.as_str()),
                "reason": atom.get("reason").and_then(|v| v.as_str()),
                "timestamp": ts.to_string(),
            }),
            "tool.called" => serde_json::json!({
                "type": "tool_called",
                "tool_name": atom.get("tool_name").and_then(|v| v.as_str()),
                "purpose": atom.get("purpose").and_then(|v| v.as_str()),
                "timestamp": ts.to_string(),
            }),
            "tool.result" => serde_json::json!({
                "type": "tool_result",
                "tool_name": atom.get("tool_name").and_then(|v| v.as_str()),
                "status": atom.get("status").and_then(|v| v.as_str()),
                "success": atom.get("status").and_then(|v| v.as_str()) == Some("success"),
                "timestamp": ts.to_string(),
            }),
            "approval.decided" => serde_json::json!({
                "type": "approval_decided",
                "decision": atom.get("decision").and_then(|v| v.as_str()),
                "reason": atom.get("reason").and_then(|v| v.as_str()),
                "timestamp": ts.to_string(),
            }),
            _ => serde_json::json!({
                "type": "unknown",
                "event_type": event_type,
                "timestamp": ts.to_string(),
            }),
        };

        Ok(item)
    }

    /// Get timeline items for a job
    pub async fn get_timeline(
        &self,
        tenant_id: &str,
        job_id: &str,
        limit: i64,
    ) -> Result<Vec<serde_json::Value>, sqlx::Error> {
        let rows = sqlx::query!(
            r#"
            SELECT timeline_item
            FROM projection_job_events
            WHERE tenant_id = $1 AND job_id = $2
            ORDER BY ts DESC
            LIMIT $3
            "#,
            tenant_id,
            job_id,
            limit
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.timeline_item).collect())
    }
}

