//! # C.Office Projections
//!
//! Projections for LLM entities, sessions, and audit events.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;
use tracing::{debug, error, info};

/// Office Projection Handler
pub struct OfficeProjection {
    pool: PgPool,
}

impl OfficeProjection {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Process an event and update projections
    pub async fn process_event(
        &self,
        event_type: &str,
        atom: &Value,
        entry_hash: &str,
        sequence: i64,
    ) -> anyhow::Result<()> {
        match event_type {
            "entity.created" => self.handle_entity_created(atom, entry_hash, sequence).await,
            "entity.activated" => self.handle_entity_status_change(atom, "active").await,
            "entity.suspended" => self.handle_entity_status_change(atom, "suspended").await,
            "entity.archived" => self.handle_entity_status_change(atom, "archived").await,
            "constitution.updated" => self.handle_constitution_updated(atom).await,
            "baseline.updated" => self.handle_baseline_updated(atom).await,
            "session.started" => self.handle_session_started(atom).await,
            "session.completed" => self.handle_session_completed(atom, entry_hash, sequence).await,
            t if t.starts_with("audit.") => self.handle_audit_event(event_type, atom, entry_hash, sequence).await,
            t if t.starts_with("governance.") => self.handle_governance_event(event_type, atom, entry_hash, sequence).await,
            _ => {
                debug!("Ignoring unknown office event type: {}", event_type);
                Ok(())
            }
        }
    }

    async fn handle_entity_created(&self, atom: &Value, entry_hash: &str, sequence: i64) -> anyhow::Result<()> {
        let entity_id = atom.get("entity_id").and_then(|v| v.as_str()).unwrap_or("");
        let name = atom.get("name").and_then(|v| v.as_str()).unwrap_or("");
        let entity_type = atom.get("entity_type").and_then(|v| v.as_str()).unwrap_or("autonomous");
        let public_key = atom.get("public_key").and_then(|v| v.as_str()).unwrap_or("");
        let constitution = atom.get("constitution").cloned().unwrap_or_else(|| serde_json::json!({}));
        let ts_ms = atom.get("ts_ms").and_then(|v| v.as_i64()).unwrap_or(0);

        sqlx::query(
            r#"
            INSERT INTO office_entities (entity_id, name, entity_type, public_key, constitution, status, created_at_ms, updated_at_ms, entry_hash, sequence)
            VALUES ($1, $2, $3, $4, $5, 'active', $6, $6, $7, $8)
            ON CONFLICT (entity_id) DO NOTHING
            "#,
        )
        .bind(entity_id)
        .bind(name)
        .bind(entity_type)
        .bind(public_key)
        .bind(&constitution)
        .bind(ts_ms)
        .bind(entry_hash)
        .bind(sequence)
        .execute(&self.pool)
        .await?;

        info!("✅ Office projection: entity.created {}", entity_id);
        Ok(())
    }

    async fn handle_entity_status_change(&self, atom: &Value, new_status: &str) -> anyhow::Result<()> {
        let entity_id = atom.get("entity_id").and_then(|v| v.as_str()).unwrap_or("");
        let ts_ms = atom.get("ts_ms").and_then(|v| v.as_i64()).unwrap_or(0);

        sqlx::query(
            r#"
            UPDATE office_entities 
            SET status = $1, updated_at_ms = $2
            WHERE entity_id = $3
            "#,
        )
        .bind(new_status)
        .bind(ts_ms)
        .bind(entity_id)
        .execute(&self.pool)
        .await?;

        info!("✅ Office projection: entity status -> {} for {}", new_status, entity_id);
        Ok(())
    }

    async fn handle_constitution_updated(&self, atom: &Value) -> anyhow::Result<()> {
        let entity_id = atom.get("entity_id").and_then(|v| v.as_str()).unwrap_or("");
        let constitution = atom.get("constitution").cloned().unwrap_or_else(|| serde_json::json!({}));
        let ts_ms = atom.get("ts_ms").and_then(|v| v.as_i64()).unwrap_or(0);

        sqlx::query(
            r#"
            UPDATE office_entities 
            SET constitution = $1, updated_at_ms = $2
            WHERE entity_id = $3
            "#,
        )
        .bind(&constitution)
        .bind(ts_ms)
        .bind(entity_id)
        .execute(&self.pool)
        .await?;

        info!("✅ Office projection: constitution.updated for {}", entity_id);
        Ok(())
    }

    async fn handle_baseline_updated(&self, atom: &Value) -> anyhow::Result<()> {
        let entity_id = atom.get("entity_id").and_then(|v| v.as_str()).unwrap_or("");
        let baseline = atom.get("baseline").and_then(|v| v.as_str()).unwrap_or("");
        let ts_ms = atom.get("ts_ms").and_then(|v| v.as_i64()).unwrap_or(0);

        sqlx::query(
            r#"
            UPDATE office_entities 
            SET baseline_narrative = $1, updated_at_ms = $2
            WHERE entity_id = $3
            "#,
        )
        .bind(baseline)
        .bind(ts_ms)
        .bind(entity_id)
        .execute(&self.pool)
        .await?;

        info!("✅ Office projection: baseline.updated for {}", entity_id);
        Ok(())
    }

    async fn handle_session_started(&self, atom: &Value) -> anyhow::Result<()> {
        let entity_id = atom.get("entity_id").and_then(|v| v.as_str()).unwrap_or("");
        let session_id = atom.get("session_id").and_then(|v| v.as_str()).unwrap_or("");
        let session_type = atom.get("session_type").and_then(|v| v.as_str()).unwrap_or("chat");
        let mode = atom.get("mode").and_then(|v| v.as_str()).unwrap_or("assisted");
        let token_budget = atom.get("token_budget").and_then(|v| v.as_i64()).unwrap_or(100000);
        let ts_ms = atom.get("ts_ms").and_then(|v| v.as_i64()).unwrap_or(0);

        sqlx::query(
            r#"
            INSERT INTO office_sessions (session_id, entity_id, session_type, mode, token_budget, started_at_ms, status)
            VALUES ($1, $2, $3, $4, $5, $6, 'active')
            ON CONFLICT (session_id) DO NOTHING
            "#,
        )
        .bind(session_id)
        .bind(entity_id)
        .bind(session_type)
        .bind(mode)
        .bind(token_budget)
        .bind(ts_ms)
        .execute(&self.pool)
        .await?;

        // Update entity stats
        sqlx::query(
            r#"
            UPDATE office_entities 
            SET total_sessions = total_sessions + 1, updated_at_ms = $1
            WHERE entity_id = $2
            "#,
        )
        .bind(ts_ms)
        .bind(entity_id)
        .execute(&self.pool)
        .await?;

        info!("✅ Office projection: session.started {} for {}", session_id, entity_id);
        Ok(())
    }

    async fn handle_session_completed(&self, atom: &Value, entry_hash: &str, sequence: i64) -> anyhow::Result<()> {
        let entity_id = atom.get("entity_id").and_then(|v| v.as_str()).unwrap_or("");
        let session_id = atom.get("session_id").and_then(|v| v.as_str()).unwrap_or("");
        let tokens_used = atom.get("tokens_used").and_then(|v| v.as_i64()).unwrap_or(0);
        let duration_ms = atom.get("duration_ms").and_then(|v| v.as_i64()).unwrap_or(0);
        let handover = atom.get("handover").cloned();
        let ts_ms = atom.get("ts_ms").and_then(|v| v.as_i64()).unwrap_or(0);

        // Update session
        sqlx::query(
            r#"
            UPDATE office_sessions 
            SET tokens_used = $1, duration_ms = $2, completed_at_ms = $3, status = 'completed'
            WHERE session_id = $4
            "#,
        )
        .bind(tokens_used)
        .bind(duration_ms)
        .bind(ts_ms)
        .bind(session_id)
        .execute(&self.pool)
        .await?;

        // Store handover if present
        if let Some(ref handover_data) = handover {
            sqlx::query(
                r#"
                INSERT INTO office_handovers (handover_id, entity_id, session_id, content, created_at_ms, entry_hash, sequence)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                "#,
            )
            .bind(format!("handover_{}", &entry_hash[..8]))
            .bind(entity_id)
            .bind(session_id)
            .bind(handover_data)
            .bind(ts_ms)
            .bind(entry_hash)
            .bind(sequence)
            .execute(&self.pool)
            .await?;
        }

        // Update entity token usage
        sqlx::query(
            r#"
            UPDATE office_entities 
            SET total_tokens_used = total_tokens_used + $1, updated_at_ms = $2
            WHERE entity_id = $3
            "#,
        )
        .bind(tokens_used)
        .bind(ts_ms)
        .bind(entity_id)
        .execute(&self.pool)
        .await?;

        info!("✅ Office projection: session.completed {} with {} tokens", session_id, tokens_used);
        Ok(())
    }

    async fn handle_audit_event(&self, event_type: &str, atom: &Value, entry_hash: &str, sequence: i64) -> anyhow::Result<()> {
        let entity_id = atom.get("entity_id").and_then(|v| v.as_str()).unwrap_or("");
        let session_id = atom.get("session_id").and_then(|v| v.as_str()).unwrap_or("");
        let job_id = atom.get("job_id").and_then(|v| v.as_str());
        let trace_id = atom.get("trace_id").and_then(|v| v.as_str()).unwrap_or("");
        let ts_ms = atom.get("ts_ms").and_then(|v| v.as_i64()).unwrap_or(0);

        sqlx::query(
            r#"
            INSERT INTO office_audit_log (audit_id, entity_id, session_id, job_id, trace_id, event_type, event_data, created_at_ms, entry_hash, sequence)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
        )
        .bind(format!("audit_{}_{}", &entry_hash[..8], sequence))
        .bind(entity_id)
        .bind(session_id)
        .bind(job_id)
        .bind(trace_id)
        .bind(event_type)
        .bind(atom)
        .bind(ts_ms)
        .bind(entry_hash)
        .bind(sequence)
        .execute(&self.pool)
        .await?;

        debug!("✅ Office projection: {} for {}", event_type, entity_id);
        Ok(())
    }

    async fn handle_governance_event(&self, event_type: &str, atom: &Value, entry_hash: &str, sequence: i64) -> anyhow::Result<()> {
        // Governance events also go to audit log
        self.handle_audit_event(event_type, atom, entry_hash, sequence).await
    }
}

// =============================================================================
// Query Types
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityRow {
    pub entity_id: String,
    pub name: String,
    pub entity_type: String,
    pub public_key: String,
    pub status: String,
    pub constitution: Option<Value>,
    pub baseline_narrative: Option<String>,
    pub total_sessions: i64,
    pub total_tokens_used: i64,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRow {
    pub session_id: String,
    pub entity_id: String,
    pub session_type: String,
    pub mode: String,
    pub token_budget: i64,
    pub tokens_used: Option<i64>,
    pub duration_ms: Option<i64>,
    pub status: String,
    pub started_at_ms: i64,
    pub completed_at_ms: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandoverRow {
    pub handover_id: String,
    pub entity_id: String,
    pub session_id: String,
    pub content: Value,
    pub created_at_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditRow {
    pub audit_id: String,
    pub entity_id: String,
    pub session_id: String,
    pub job_id: Option<String>,
    pub trace_id: String,
    pub event_type: String,
    pub event_data: Value,
    pub created_at_ms: i64,
}

