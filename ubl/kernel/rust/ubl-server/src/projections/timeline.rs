//! Timeline Projection
//!
//! Optimized timeline view for conversations (messages + job cards).
//! Combines messages and job cards in a single sorted view.

use sqlx::PgPool;
use time::OffsetDateTime;
use tracing::info;

/// Timeline projection handler
pub struct TimelineProjection {
    pool: PgPool,
}

impl TimelineProjection {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Add a timeline item (message or job card)
    pub async fn add_item(
        &self,
        tenant_id: &str,
        conversation_id: &str,
        item_type: &str,
        item_data: &serde_json::Value,
        sequence: i64,
    ) -> Result<(), sqlx::Error> {
        let ts = OffsetDateTime::now_utc();
        let cursor = format!("{}:{}", sequence, ts.unix_timestamp());

        sqlx::query!(
            r#"
            INSERT INTO projection_timeline_items (
                tenant_id, conversation_id, cursor, item_type, item_data, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (tenant_id, conversation_id, cursor) DO UPDATE SET
                item_data = EXCLUDED.item_data
            "#,
            tenant_id,
            conversation_id,
            cursor,
            item_type,
            item_data,
            ts
        )
        .execute(&self.pool)
        .await?;

        info!("📜 Timeline item added: {} {} seq={}", conversation_id, item_type, sequence);
        Ok(())
    }

    /// Get timeline for a conversation
    pub async fn get_timeline(
        &self,
        tenant_id: &str,
        conversation_id: &str,
        cursor: Option<&str>,
        limit: i64,
    ) -> Result<Vec<serde_json::Value>, sqlx::Error> {
        let query = if let Some(cursor_val) = cursor {
            // Parse cursor: seq:timestamp
            let parts: Vec<&str> = cursor_val.split(':').collect();
            let seq: i64 = parts.get(0).and_then(|s| s.parse().ok()).unwrap_or(0);
            
            sqlx::query!(
                r#"
                SELECT cursor, item_type, item_data, created_at
                FROM projection_timeline_items
                WHERE tenant_id = $1 AND conversation_id = $2
                  AND cursor > $3
                ORDER BY created_at ASC
                LIMIT $4
                "#,
                tenant_id,
                conversation_id,
                cursor_val,
                limit
            )
        } else {
            sqlx::query!(
                r#"
                SELECT cursor, item_type, item_data, created_at
                FROM projection_timeline_items
                WHERE tenant_id = $1 AND conversation_id = $2
                ORDER BY created_at DESC
                LIMIT $3
                "#,
                tenant_id,
                conversation_id,
                limit
            )
        };

        let rows = query.fetch_all(&self.pool).await?;

        Ok(rows.into_iter().map(|r| {
            serde_json::json!({
                "cursor": r.cursor,
                "item_type": r.item_type,
                "item_data": r.item_data,
                "created_at": r.created_at.to_string(),
            })
        }).collect())
    }
}

