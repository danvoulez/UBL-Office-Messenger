//! Gateway Projection Management
//!
//! Manages projection updates triggered by UBL ledger events.
//! Gateway subscribes to SSE tail and updates projections accordingly.

use sqlx::PgPool;
use tracing::info;

/// Job data from projection (Fix #6: Job Card Provenance)
#[derive(Debug, sqlx::FromRow)]
pub struct JobProjection {
    pub tenant_id: String,
    pub job_id: String,
    pub conversation_id: String,
    pub title: String,
    pub goal: String,
    pub state: String,
    pub owner_entity_id: Option<String>,
    pub waiting_on: Option<Vec<String>>,
}

/// Gateway projection manager
pub struct GatewayProjections {
    pool: PgPool,
}

impl GatewayProjections {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Fix #6: Get a job by ID with tenant isolation
    pub async fn get_job(&self, job_id: &str, tenant_id: &str) -> Result<Option<JobProjection>, sqlx::Error> {
        let job = sqlx::query_as::<_, JobProjection>(
            r#"
            SELECT tenant_id, job_id, conversation_id, title, goal, state, owner_entity_id, waiting_on
            FROM projection_jobs
            WHERE job_id = $1 AND tenant_id = $2
            "#,
        )
        .bind(job_id)
        .bind(tenant_id)
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(job)
    }

    /// Process a ledger event and update projections
    /// This is called by the SSE tail subscription handler
    pub async fn process_event(
        &self,
        container_id: &str,
        event_type: &str,
        atom: &serde_json::Value,
        entry_hash: &str,
        sequence: i64,
    ) -> Result<(), sqlx::Error> {
        info!("📊 Gateway projection: {} {} seq={}", container_id, event_type, sequence);

        // Delegate to existing projection handlers
        match container_id {
            "C.Messenger" => {
                // Use existing MessagesProjection
                // This will be handled by the main projection system
                Ok(())
            }
            "C.Jobs" => {
                // Use existing JobsProjection
                // This will be handled by the main projection system
                Ok(())
            }
            _ => Ok(()),
        }
    }
}

