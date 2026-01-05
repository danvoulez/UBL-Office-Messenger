//! C.Jobs Projection — Job state derived from job.* events

use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use time::OffsetDateTime;
use tracing::{info, error};

/// Job record in projection
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Job {
    pub job_id: String,
    pub conversation_id: String,
    pub title: String,
    pub description: String,
    pub status: String,
    pub priority: String,
    pub assigned_to: Option<String>,
    pub created_by: String,
    pub created_at: OffsetDateTime,
    pub started_at: Option<OffsetDateTime>,
    pub completed_at: Option<OffsetDateTime>,
    pub cancelled_at: Option<OffsetDateTime>,
    pub progress: Option<i32>,
    pub progress_message: Option<String>,
    pub result_summary: Option<String>,
    pub result_artifacts: Option<serde_json::Value>,
    pub estimated_duration_seconds: Option<i32>,
    pub estimated_value: Option<f64>,
    pub last_event_hash: String,
    pub last_event_seq: i64,
}

/// Approval record in projection
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Approval {
    pub approval_id: String,
    pub job_id: String,
    pub action: String,
    pub reason: String,
    pub requested_by: String,
    pub requested_at: OffsetDateTime,
    pub status: String,
    pub decided_by: Option<String>,
    pub decided_at: Option<OffsetDateTime>,
    pub decision: Option<String>,
    pub decision_reason: Option<String>,
    pub last_event_hash: String,
    pub last_event_seq: i64,
}

/// Jobs projection handler
pub struct JobsProjection {
    pool: PgPool,
}

impl JobsProjection {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Process a job event and update projection
    pub async fn process_event(
        &self,
        event_type: &str,
        atom: &serde_json::Value,
        entry_hash: &str,
        sequence: i64,
    ) -> Result<(), sqlx::Error> {
        match event_type {
            "job.created" => self.handle_job_created(atom, entry_hash, sequence).await,
            "job.started" => self.handle_job_started(atom, entry_hash, sequence).await,
            "job.progress" => self.handle_job_progress(atom, entry_hash, sequence).await,
            "job.completed" => self.handle_job_completed(atom, entry_hash, sequence).await,
            "job.cancelled" => self.handle_job_cancelled(atom, entry_hash, sequence).await,
            "approval.requested" => self.handle_approval_requested(atom, entry_hash, sequence).await,
            "approval.decided" => self.handle_approval_decided(atom, entry_hash, sequence).await,
            _ => {
                info!("Unknown job event type: {}", event_type);
                Ok(())
            }
        }
    }

    async fn handle_job_created(
        &self,
        atom: &serde_json::Value,
        entry_hash: &str,
        sequence: i64,
    ) -> Result<(), sqlx::Error> {
        let job_id = atom["id"].as_str().unwrap_or_default();
        let conversation_id = atom["conversation_id"].as_str().unwrap_or_default();
        let title = atom["title"].as_str().unwrap_or_default();
        let description = atom["description"].as_str().unwrap_or_default();
        let priority = atom["priority"].as_str().unwrap_or("normal");
        let assigned_to = atom["assigned_to"].as_str();
        let created_by = atom["created_by"].as_str().unwrap_or_default();
        let created_at = atom["created_at"].as_str().unwrap_or_default();
        let estimated_duration = atom["estimated_duration_seconds"].as_i64().map(|v| v as i32);
        let estimated_value = atom["estimated_value"].as_f64();

        sqlx::query(
            r#"
            INSERT INTO projection_jobs (
                job_id, conversation_id, title, description, status, priority,
                assigned_to, created_by, created_at, estimated_duration_seconds,
                estimated_value, last_event_hash, last_event_seq
            ) VALUES ($1, $2, $3, $4, 'pending', $5, $6, $7, $8::timestamptz, $9, $10, $11, $12)
            ON CONFLICT (job_id) DO UPDATE SET
                title = EXCLUDED.title,
                description = EXCLUDED.description,
                priority = EXCLUDED.priority,
                assigned_to = EXCLUDED.assigned_to,
                last_event_hash = EXCLUDED.last_event_hash,
                last_event_seq = EXCLUDED.last_event_seq
            "#
        )
        .bind(job_id)
        .bind(conversation_id)
        .bind(title)
        .bind(description)
        .bind(priority)
        .bind(assigned_to)
        .bind(created_by)
        .bind(created_at)
        .bind(estimated_duration)
        .bind(estimated_value)
        .bind(entry_hash)
        .bind(sequence)
        .execute(&self.pool)
        .await?;

        info!("📋 Job created: {} - {}", job_id, title);
        Ok(())
    }

    async fn handle_job_started(
        &self,
        atom: &serde_json::Value,
        entry_hash: &str,
        sequence: i64,
    ) -> Result<(), sqlx::Error> {
        let job_id = atom["job_id"].as_str().unwrap_or_default();
        let started_at = atom["started_at"].as_str().unwrap_or_default();
        let tenant_id = atom.get("tenant_id").and_then(|v| v.as_str()).unwrap_or("default");
        let now = time::OffsetDateTime::now_utc();

        // Update old table (Diamond Checklist #2: causal ordering)
        let _ = sqlx::query(
            r#"
            UPDATE projection_jobs
            SET status = 'running', started_at = $2::timestamptz,
                last_event_hash = $3, last_event_seq = $4
            WHERE job_id = $1 AND last_event_seq < $4
            "#
        )
        .bind(job_id)
        .bind(started_at)
        .bind(entry_hash)
        .bind(sequence)
        .execute(&self.pool)
        .await;

        // Update new table (Diamond Checklist #2: causal ordering)
        let _ = sqlx::query(
            r#"
            UPDATE projection_jobs
            SET state = 'in_progress', updated_at = $2, last_activity_at = $2,
                last_event_hash = $3, last_event_seq = $4
            WHERE tenant_id = $5 AND job_id = $1 AND last_event_seq < $4
            "#
        )
        .bind(job_id)
        .bind(now)
        .bind(entry_hash)
        .bind(sequence)
        .bind(tenant_id)
        .execute(&self.pool)
        .await;

        info!("🚀 Job started: {}", job_id);
        Ok(())
    }

    async fn handle_job_progress(
        &self,
        atom: &serde_json::Value,
        entry_hash: &str,
        sequence: i64,
    ) -> Result<(), sqlx::Error> {
        let job_id = atom["job_id"].as_str().unwrap_or_default();
        let progress = atom["progress"].as_i64().unwrap_or(0) as i32;
        let message = atom["message"].as_str();

        sqlx::query(
            r#"
            UPDATE projection_jobs
            SET progress = $2, progress_message = $3,
                last_event_hash = $4, last_event_seq = $5
            WHERE job_id = $1 AND last_event_seq < $5
            "#
        )
        .bind(job_id)
        .bind(progress)
        .bind(message)
        .bind(entry_hash)
        .bind(sequence)
        .execute(&self.pool)
        .await?;

        info!("📊 Job progress: {} - {}%", job_id, progress);
        Ok(())
    }

    async fn handle_job_completed(
        &self,
        atom: &serde_json::Value,
        entry_hash: &str,
        sequence: i64,
    ) -> Result<(), sqlx::Error> {
        let job_id = atom["job_id"].as_str().unwrap_or_default();
        let completed_at = atom["completed_at"].as_str().unwrap_or_default();
        let summary = atom["result"]["summary"].as_str();
        let artifacts = atom["result"]["artifacts"].clone();
        let tenant_id = atom.get("tenant_id").and_then(|v| v.as_str()).unwrap_or("default");
        let now = time::OffsetDateTime::now_utc();

        // Update old table (Diamond Checklist #2)
        let _ = sqlx::query(
            r#"
            UPDATE projection_jobs
            SET status = 'completed', completed_at = $2::timestamptz,
                progress = 100, result_summary = $3, result_artifacts = $4,
                last_event_hash = $5, last_event_seq = $6
            WHERE job_id = $1 AND last_event_seq < $6
            "#
        )
        .bind(job_id)
        .bind(completed_at)
        .bind(summary)
        .bind(artifacts)
        .bind(entry_hash)
        .bind(sequence)
        .execute(&self.pool)
        .await;

        // Update new table (Diamond Checklist #2)
        let _ = sqlx::query(
            r#"
            UPDATE projection_jobs
            SET state = 'completed', updated_at = $2, last_activity_at = $2,
                last_event_hash = $3, last_event_seq = $4
            WHERE tenant_id = $5 AND job_id = $1 AND last_event_seq < $4
            "#
        )
        .bind(job_id)
        .bind(now)
        .bind(entry_hash)
        .bind(sequence)
        .bind(tenant_id)
        .execute(&self.pool)
        .await;

        info!("✅ Job completed: {}", job_id);
        Ok(())
    }

    async fn handle_job_cancelled(
        &self,
        atom: &serde_json::Value,
        entry_hash: &str,
        sequence: i64,
    ) -> Result<(), sqlx::Error> {
        let job_id = atom["job_id"].as_str().unwrap_or_default();
        let cancelled_at = atom["cancelled_at"].as_str().unwrap_or_default();
        let tenant_id = atom.get("tenant_id").and_then(|v| v.as_str()).unwrap_or("default");
        let now = time::OffsetDateTime::now_utc();

        // Update old table (Diamond Checklist #2)
        let _ = sqlx::query(
            r#"
            UPDATE projection_jobs
            SET status = 'cancelled', cancelled_at = $2::timestamptz,
                last_event_hash = $3, last_event_seq = $4
            WHERE job_id = $1 AND last_event_seq < $4
            "#
        )
        .bind(job_id)
        .bind(cancelled_at)
        .bind(entry_hash)
        .bind(sequence)
        .execute(&self.pool)
        .await;

        // Update new table (Diamond Checklist #2)
        let _ = sqlx::query(
            r#"
            UPDATE projection_jobs
            SET state = 'cancelled', updated_at = $2, last_activity_at = $2,
                last_event_hash = $3, last_event_seq = $4
            WHERE tenant_id = $5 AND job_id = $1 AND last_event_seq < $4
            "#
        )
        .bind(job_id)
        .bind(now)
        .bind(entry_hash)
        .bind(sequence)
        .bind(tenant_id)
        .execute(&self.pool)
        .await;

        info!("🚫 Job cancelled: {}", job_id);
        Ok(())
    }

    async fn handle_approval_requested(
        &self,
        atom: &serde_json::Value,
        entry_hash: &str,
        sequence: i64,
    ) -> Result<(), sqlx::Error> {
        let approval_id = atom["id"].as_str().unwrap_or_default();
        let job_id = atom["job_id"].as_str().unwrap_or_default();
        let action = atom["action"].as_str().unwrap_or_default();
        let reason = atom["reason"].as_str().unwrap_or_default();
        let requested_by = atom["requested_by"].as_str().unwrap_or_default();
        let requested_at = atom["requested_at"].as_str().unwrap_or_default();

        sqlx::query(
            r#"
            INSERT INTO projection_approvals (
                approval_id, job_id, action, reason, requested_by, requested_at,
                status, last_event_hash, last_event_seq
            ) VALUES ($1, $2, $3, $4, $5, $6::timestamptz, 'pending', $7, $8)
            ON CONFLICT (approval_id) DO NOTHING
            "#
        )
        .bind(approval_id)
        .bind(job_id)
        .bind(action)
        .bind(reason)
        .bind(requested_by)
        .bind(requested_at)
        .bind(entry_hash)
        .bind(sequence)
        .execute(&self.pool)
        .await?;

        // Update job status (Diamond Checklist #2)
        sqlx::query(
            r#"
            UPDATE projection_jobs
            SET status = 'awaiting_approval',
                last_event_hash = $2, last_event_seq = $3
            WHERE job_id = $1 AND last_event_seq < $3
            "#
        )
        .bind(job_id)
        .bind(entry_hash)
        .bind(sequence)
        .execute(&self.pool)
        .await?;

        info!("🔔 Approval requested for job: {} - {}", job_id, action);
        Ok(())
    }

    async fn handle_approval_decided(
        &self,
        atom: &serde_json::Value,
        entry_hash: &str,
        sequence: i64,
    ) -> Result<(), sqlx::Error> {
        let approval_id = atom["approval_id"].as_str().unwrap_or_default();
        let decided_by = atom["decided_by"].as_str().unwrap_or_default();
        let decided_at = atom["decided_at"].as_str().unwrap_or_default();
        let decision = atom["decision"].as_str().unwrap_or_default();
        let decision_reason = atom["reason"].as_str();

        // Update approval (Diamond Checklist #2)
        sqlx::query(
            r#"
            UPDATE projection_approvals
            SET status = $2, decided_by = $3, decided_at = $4::timestamptz,
                decision = $5, decision_reason = $6,
                last_event_hash = $7, last_event_seq = $8
            WHERE approval_id = $1 AND last_event_seq < $8
            "#
        )
        .bind(approval_id)
        .bind(decision)
        .bind(decided_by)
        .bind(decided_at)
        .bind(decision)
        .bind(decision_reason)
        .bind(entry_hash)
        .bind(sequence)
        .execute(&self.pool)
        .await?;

        // If approved, update job status back to running
        // If rejected, update to cancelled
        let new_status = if decision == "approved" { "running" } else { "rejected" };
        
        // Diamond Checklist #2: causal ordering with JOIN
        sqlx::query(
            r#"
            UPDATE projection_jobs
            SET status = $3, last_event_hash = $4, last_event_seq = $5
            FROM projection_approvals
            WHERE projection_jobs.job_id = projection_approvals.job_id
              AND projection_approvals.approval_id = $1
              AND ($2 = 'approved' OR $2 = 'rejected')
              AND projection_jobs.last_event_seq < $5
            "#
        )
        .bind(approval_id)
        .bind(decision)
        .bind(new_status)
        .bind(entry_hash)
        .bind(sequence)
        .execute(&self.pool)
        .await?;

        info!("⚖️ Approval decided: {} - {}", approval_id, decision);
        Ok(())
    }

    /// Query jobs by conversation
    pub async fn get_jobs_by_conversation(&self, conversation_id: &str) -> Result<Vec<Job>, sqlx::Error> {
        sqlx::query_as::<_, Job>(
            r#"
            SELECT job_id, conversation_id, 
                   COALESCE(title, '') as title, 
                   COALESCE(description, '') as description, 
                   COALESCE(status, 'pending') as status, 
                   COALESCE(priority, 'normal') as priority,
                   assigned_to, 
                   COALESCE(created_by, '') as created_by, 
                   created_at, started_at, completed_at,
                   cancelled_at, progress, progress_message, result_summary,
                   result_artifacts, estimated_duration_seconds,
                   estimated_value,
                   last_event_hash, last_event_seq
            FROM projection_jobs
            WHERE conversation_id = $1
            ORDER BY created_at DESC
            "#
        )
        .bind(conversation_id)
        .fetch_all(&self.pool)
        .await
    }

    /// Query job by ID
    pub async fn get_job(&self, job_id: &str) -> Result<Option<Job>, sqlx::Error> {
        sqlx::query_as::<_, Job>(
            r#"
            SELECT job_id, conversation_id, 
                   COALESCE(title, '') as title, 
                   COALESCE(description, '') as description, 
                   COALESCE(status, 'pending') as status, 
                   COALESCE(priority, 'normal') as priority,
                   assigned_to, 
                   COALESCE(created_by, '') as created_by, 
                   created_at, started_at, completed_at,
                   cancelled_at, progress, progress_message, result_summary,
                   result_artifacts, estimated_duration_seconds,
                   estimated_value,
                   last_event_hash, last_event_seq
            FROM projection_jobs
            WHERE job_id = $1
            "#
        )
        .bind(job_id)
        .fetch_optional(&self.pool)
        .await
    }

    /// Query pending approvals for a job
    pub async fn get_pending_approvals(&self, job_id: &str) -> Result<Vec<Approval>, sqlx::Error> {
        sqlx::query_as!(
            Approval,
            r#"
            SELECT approval_id, job_id, action, reason, requested_by, requested_at,
                   status, decided_by, decided_at, decision, decision_reason,
                   last_event_hash, last_event_seq
            FROM projection_approvals
            WHERE job_id = $1 AND status = 'pending'
            ORDER BY requested_at DESC
            "#,
            job_id
        )
        .fetch_all(&self.pool)
        .await
    }
}

