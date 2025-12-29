//! Gateway Projection Management
//!
//! Manages projection updates triggered by UBL ledger events.
//! Gateway subscribes to SSE tail and updates projections accordingly.

use sqlx::PgPool;
use tracing::info;

/// Gateway projection manager
pub struct GatewayProjections {
    pool: PgPool,
}

impl GatewayProjections {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
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

