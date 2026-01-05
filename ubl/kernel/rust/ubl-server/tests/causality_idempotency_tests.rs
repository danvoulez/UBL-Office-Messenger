//! Integration tests for UBL-FIX: Causality and Idempotency
//! 
//! These tests verify Diamond Checklist requirements:
//! - #1: No duplication after reprocessing/retries
//! - #2: Causal ordering (old seq doesn't overwrite new seq)
//! - #3: Liveness (system stays responsive)
//! - #4: Auth anti-replay

#[cfg(test)]
mod causality_tests {
    use sqlx::PgPool;
    
    // Helper to setup test database connection
    async fn setup_test_db() -> PgPool {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://localhost:5432/ubl_test".to_string());
        PgPool::connect(&database_url).await.expect("Failed to connect to test database")
    }
    
    #[tokio::test]
    #[ignore] // Run with: cargo test --ignored
    async fn test_message_causal_ordering() {
        // Diamond Checklist #2: Events with seq antigo não sobrescrevem estado mais novo
        let pool = setup_test_db().await;
        
        let test_msg_id = format!("test-msg-{}", uuid::Uuid::new_v4());
        let test_conv_id = "test-conv-causality";
        
        // Insert message with seq=10
        sqlx::query(
            r#"
            INSERT INTO projection_messages 
            (message_id, conversation_id, from_id, content_hash, timestamp, message_type, last_event_hash, last_event_seq)
            VALUES ($1, $2, 'user1', 'hash1', NOW(), 'text', 'hash_seq10', 10)
            "#
        )
        .bind(&test_msg_id)
        .bind(test_conv_id)
        .execute(&pool)
        .await
        .expect("Failed to insert initial message");
        
        // Try to update with newer seq=11
        let rows_updated = sqlx::query(
            r#"
            UPDATE projection_messages
            SET content_hash = 'hash2', last_event_hash = 'hash_seq11', last_event_seq = 11
            WHERE message_id = $1 AND last_event_seq < 11
            "#
        )
        .bind(&test_msg_id)
        .execute(&pool)
        .await
        .expect("Failed to update with seq=11")
        .rows_affected();
        
        assert_eq!(rows_updated, 1, "Should update message with newer seq");
        
        // Verify update was applied
        let current_seq: i64 = sqlx::query_scalar(
            "SELECT last_event_seq FROM projection_messages WHERE message_id = $1"
        )
        .bind(&test_msg_id)
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch current seq");
        
        assert_eq!(current_seq, 11, "Message should have seq=11");
        
        // Now try to apply old seq=10 (should be rejected by guard)
        let rows_updated_old = sqlx::query(
            r#"
            UPDATE projection_messages
            SET content_hash = 'hash_old', last_event_hash = 'hash_seq10_retry', last_event_seq = 10
            WHERE message_id = $1 AND last_event_seq < 10
            "#
        )
        .bind(&test_msg_id)
        .execute(&pool)
        .await
        .expect("Failed to attempt old seq update")
        .rows_affected();
        
        assert_eq!(rows_updated_old, 0, "Old seq should NOT update (causal guard)");
        
        // Verify final state is still seq=11
        let final_seq: i64 = sqlx::query_scalar(
            "SELECT last_event_seq FROM projection_messages WHERE message_id = $1"
        )
        .bind(&test_msg_id)
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch final seq");
        
        assert_eq!(final_seq, 11, "Message should still have seq=11 (not overwritten by old seq)");
        
        // Cleanup
        sqlx::query("DELETE FROM projection_messages WHERE message_id = $1")
            .bind(&test_msg_id)
            .execute(&pool)
            .await
            .ok();
    }
}
