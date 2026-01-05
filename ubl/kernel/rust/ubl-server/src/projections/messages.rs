//! C.Messenger Projection — Message state derived from message.* events

use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use time::OffsetDateTime;
use tracing::info;

/// Message record in projection
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Message {
    pub message_id: String,
    pub conversation_id: String,
    pub from_id: String,
    pub content_hash: String,
    pub timestamp: OffsetDateTime,
    pub message_type: String,
    pub read_by: Vec<String>,
    pub last_event_hash: String,
    pub last_event_seq: i64,
}

/// Messages projection handler
pub struct MessagesProjection {
    pool: PgPool,
}

impl MessagesProjection {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Process a message event and update projection
    pub async fn process_event(
        &self,
        event_type: &str,
        atom: &serde_json::Value,
        entry_hash: &str,
        sequence: i64,
    ) -> Result<(), sqlx::Error> {
        match event_type {
            "message.created" => self.handle_message_created(atom, entry_hash, sequence).await,
            "message.read" => self.handle_message_read(atom, entry_hash, sequence).await,
            _ => {
                info!("Unknown message event type: {}", event_type);
                Ok(())
            }
        }
    }

    async fn handle_message_created(
        &self,
        atom: &serde_json::Value,
        entry_hash: &str,
        sequence: i64,
    ) -> Result<(), sqlx::Error> {
        let message_id = atom["message_id"].as_str().unwrap_or_default();
        let conversation_id = atom["conversation_id"].as_str().unwrap_or_default();
        let from_id = atom["from"].as_str().unwrap_or_default();
        let content_hash = atom["content_hash"].as_str().unwrap_or_default();
        let timestamp = atom["timestamp"].as_str().unwrap_or_default();
        let message_type = atom["message_type"].as_str().unwrap_or("text");

        sqlx::query(
            r#"
            INSERT INTO projection_messages (
                message_id, conversation_id, from_id, content_hash, timestamp,
                message_type, last_event_hash, last_event_seq
            ) VALUES ($1, $2, $3, $4, $5::timestamptz, $6, $7, $8)
            ON CONFLICT (message_id) DO NOTHING
            "#
        )
        .bind(message_id)
        .bind(conversation_id)
        .bind(from_id)
        .bind(content_hash)
        .bind(timestamp)
        .bind(message_type)
        .bind(entry_hash)
        .bind(sequence)
        .execute(&self.pool)
        .await?;

        info!("💬 Message created: {} in {}", message_id, conversation_id);
        Ok(())
    }

    async fn handle_message_read(
        &self,
        atom: &serde_json::Value,
        entry_hash: &str,
        sequence: i64,
    ) -> Result<(), sqlx::Error> {
        let message_id = atom["message_id"].as_str().unwrap_or_default();
        let read_by = atom["read_by"].as_str().unwrap_or_default();

        // Diamond Checklist #2: causal ordering to prevent race conditions
        sqlx::query(
            r#"
            UPDATE projection_messages
            SET read_by = array_append(read_by, $2),
                last_event_hash = $3, last_event_seq = $4
            WHERE message_id = $1 AND NOT ($2 = ANY(read_by)) AND last_event_seq < $4
            "#
        )
        .bind(message_id)
        .bind(read_by)
        .bind(entry_hash)
        .bind(sequence)
        .execute(&self.pool)
        .await?;

        info!("👁️ Message read: {} by {}", message_id, read_by);
        Ok(())
    }

    /// Query messages by conversation
    pub async fn get_messages_by_conversation(
        &self,
        conversation_id: &str,
        limit: i64,
        before_seq: Option<i64>,
    ) -> Result<Vec<Message>, sqlx::Error> {
        let before = before_seq.unwrap_or(i64::MAX);
        
        sqlx::query_as::<_, Message>(
            r#"
            SELECT message_id, conversation_id, from_id, content_hash, timestamp,
                   message_type, COALESCE(read_by, ARRAY[]::text[]) as read_by, 
                   last_event_hash, last_event_seq
            FROM projection_messages
            WHERE conversation_id = $1 AND last_event_seq < $2
            ORDER BY timestamp DESC
            LIMIT $3
            "#
        )
        .bind(conversation_id)
        .bind(before)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
    }

    /// Get unread message count for a user in a conversation
    pub async fn get_unread_count(
        &self,
        conversation_id: &str,
        user_id: &str,
    ) -> Result<i64, sqlx::Error> {
        let count = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) as "count!"
            FROM projection_messages
            WHERE conversation_id = $1 AND NOT ($2 = ANY(read_by))
            "#,
            conversation_id,
            user_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(count)
    }
}

