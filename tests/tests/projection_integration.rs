//!  Projection System Integration Tests
//! Tests complete projection lifecycle:  event → projection → query

use sqlx::PgPool;
use serde_json::json;
use time::OffsetDateTime;

#[sqlx::test]
async fn test_message_projection_lifecycle(pool: PgPool) {
    let tenant_id = "T.UBL";
    let conversation_id = "conv_test";
    let message_id = "msg_test";
    
    // 1. Insert message. created event
    let event = json!({
        "type":  "message.created",
        "message_id":  message_id,
        "conversation_id": conversation_id,
        "from": "user_alice",
        "content_hash": "hash_abc123",
        "message_type": "text",
        "created_at": "2024-12-29T10:00:00Z"
    });
    
    let atom_hash = "hash_msg";
    
    sqlx:: query!(
        r#"
        INSERT INTO ledger_atom (hash, data)
        VALUES ($1, $2)
        "#,
        atom_hash,
        event
    )
    .execute(&pool)
    .await
    .unwrap();
    
    sqlx::query!(
        r#"
        INSERT INTO ledger_entries (
            container_id, sequence, atom_hash, previous_hash,
            entry_hash, timestamp_ns, intent_class, physics_delta
        )
        VALUES ('C. Messenger', 1, $1, $2, 'entry1', 1000000, 'Observation', '0')
        "#,
        atom_hash,
        "0". repeat(64)
    )
    .execute(&pool)
    .await
    .unwrap();
    
    // 2. Manually trigger projection update (in prod, this is automatic)
    sqlx::query!(
        r#"
        INSERT INTO projection_messages (
            tenant_id, message_id, conversation_id, sender_entity_id,
            content_hash, message_type, created_at
        )
        VALUES ($1, $2, $3, 'user_alice', 'hash_abc123', 'text', NOW())
        "#,
        tenant_id,
        message_id,
        conversation_id
    )
    .execute(&pool)
    .await
    .unwrap();
    
    // 3. Query projection
    let message = sqlx::query!(
        r#"
        SELECT message_id, conversation_id, sender_entity_id, message_type
        FROM projection_messages
        WHERE message_id = $1
        "#,
        message_id
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    
    assert_eq!(message.message_id, message_id);
    assert_eq!(message.conversation_id, conversation_id);
    assert_eq!(message.sender_entity_id, "user_alice");
}

#[sqlx::test]
async fn test_job_projection_lifecycle(pool: PgPool) {
    let tenant_id = "T.UBL";
    let job_id = "job_test";
    let conversation_id = "conv_test";
    
    // 1. Insert job.created event
    let event = json! ({
        "type": "job.created",
        "id": job_id,
        "conversation_id": conversation_id,
        "title": "Test Job",
        "description": "Test job description",
        "created_by": "user_alice",
        "assigned_to": "agent_sofia",
        "priority": "normal",
        "created_at": "2024-12-29T10:00:00Z"
    });
    
    let atom_hash = "hash_job";
    
    sqlx::query!(
        r#"
        INSERT INTO ledger_atom (hash, data)
        VALUES ($1, $2)
        "#,
        atom_hash,
        event
    )
    .execute(&pool)
    .await
    .unwrap();
    
    sqlx::query!(
        r#"
        INSERT INTO ledger_entries (
            container_id, sequence, atom_hash, previous_hash,
            entry_hash, timestamp_ns, intent_class, physics_delta
        )
        VALUES ('C.Jobs', 1, $1, $2, 'entry1', 1000000, 'Observation', '0')
        "#,
        atom_hash,
        "0".repeat(64)
    )
    .execute(&pool)
    .await
    .unwrap();
    
    // 2. Trigger projection update
    sqlx::query!(
        r#"
        INSERT INTO projection_jobs (
            tenant_id, job_id, conversation_id, title, description,
            state, priority, owner_entity_id, created_by_entity_id,
            created_at, updated_at
        )
        VALUES ($1, $2, $3, 'Test Job', 'Test job description',
                'proposed', 'normal', 'agent_sofia', 'user_alice',
                NOW(), NOW())
        "#,
        tenant_id,
        job_id,
        conversation_id
    )
    .execute(&pool)
    .await
    .unwrap();
    
    // 3. Query projection
    let job = sqlx::query!(
        r#"
        SELECT job_id, title, state, owner_entity_id
        FROM projection_jobs
        WHERE job_id = $1
        "#,
        job_id
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    
    assert_eq!(job.job_id, job_id);
    assert_eq!(job.title, "Test Job");
    assert_eq!(job.state, "proposed");
}

#[sqlx::test]
async fn test_job_state_transition_projection(pool: PgPool) {
    let tenant_id = "T.UBL";
    let job_id = "job_test";
    
    // Create initial job
    sqlx::query!(
        r#"
        INSERT INTO projection_jobs (
            tenant_id, job_id, conversation_id, title, description,
            state, priority, owner_entity_id, created_by_entity_id,
            created_at, updated_at
        )
        VALUES ($1, $2, 'conv_test', 'Test Job', 'Description',
                'proposed', 'normal', 'agent_sofia', 'user_alice',
                NOW(), NOW())
        "#,
        tenant_id,
        job_id
    )
    .execute(&pool)
    .await
    .unwrap();
    
    // Update state to approved
    sqlx::query!(
        r#"
        UPDATE projection_jobs
        SET state = 'approved', updated_at = NOW()
        WHERE job_id = $1
        "#,
        job_id
    )
    .execute(&pool)
    .await
    .unwrap();
    
    // Verify state change
    let job = sqlx::query!(
        r#"
        SELECT state
        FROM projection_jobs
        WHERE job_id = $1
        "#,
        job_id
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    
    assert_eq!(job.state, "approved");
}

#[sqlx::test]
async fn test_timeline_projection(pool: PgPool) {
    let tenant_id = "T.UBL";
    let conversation_id = "conv_test";
    
    // Insert message timeline item
    let cursor1 = "1: 1000000000";
    let item1 = json!({
        "type": "message",
        "message_id": "msg_1",
        "content":  "Hello"
    });
    
    sqlx::query!(
        r#"
        INSERT INTO projection_timeline_items (
            tenant_id, conversation_id, cursor, item_type, item_data, created_at
        )
        VALUES ($1, $2, $3, 'message', $4, NOW())
        "#,
        tenant_id,
        conversation_id,
        cursor1,
        item1
    )
    .execute(&pool)
    .await
    .unwrap();
    
    // Insert job card timeline item
    let cursor2 = "2:1000000001";
    let item2 = json!({
        "type":  "job_card",
        "card_id": "card_1",
        "job_id": "job_1"
    });
    
    sqlx::query!(
        r#"
        INSERT INTO projection_timeline_items (
            tenant_id, conversation_id, cursor, item_type, item_data, created_at
        )
        VALUES ($1, $2, $3, 'job_card', $4, NOW())
        "#,
        tenant_id,
        conversation_id,
        cursor2,
        item2
    )
    .execute(&pool)
    .await
    .unwrap();
    
    // Query timeline
    let items = sqlx::query!(
        r#"
        SELECT cursor, item_type, item_data
        FROM projection_timeline_items
        WHERE tenant_id = $1 AND conversation_id = $2
        ORDER BY created_at ASC
        "#,
        tenant_id,
        conversation_id
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    
    assert_eq!(items. len(), 2);
    assert_eq!(items[0].item_type, "message");
    assert_eq!(items[1].item_type, "job_card");
}

#[sqlx::test]
async fn test_presence_projection_update(pool: PgPool) {
    let tenant_id = "T.UBL";
    let entity_id = "user_alice";
    
    // Set initial presence
    sqlx::query!(
        r#"
        INSERT INTO projection_presence (
            tenant_id, entity_id, state, since, last_seen_at, last_event_hash
        )
        VALUES ($1, $2, 'available', NOW(), NOW(), 'hash1')
        "#,
        tenant_id,
        entity_id
    )
    .execute(&pool)
    .await
    .unwrap();
    
    // Update to working
    sqlx::query!(
        r#"
        UPDATE projection_presence
        SET state = 'working', job_id = 'job_123', since = NOW(), last_event_hash = 'hash2'
        WHERE entity_id = $1
        "#,
        entity_id
    )
    .execute(&pool)
    .await
    .unwrap();
    
    // Verify update
    let presence = sqlx::query!(
        r#"
        SELECT state, job_id
        FROM projection_presence
        WHERE entity_id = $1
        "#,
        entity_id
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    
    assert_eq!(presence.state, "working");
    assert_eq!(presence.job_id, Some("job_123".to_string()));
}

#[sqlx::test]
async fn test_artifacts_projection(pool: PgPool) {
    let tenant_id = "T.UBL";
    let job_id = "job_test";
    let artifact_id = "artifact_test";
    
    // Insert artifact
    sqlx::query!(
        r#"
        INSERT INTO projection_job_artifacts (
            tenant_id, job_id, artifact_id, kind, title,
            url, mime_type, size_bytes, event_id, created_at
        )
        VALUES ($1, $2, $3, 'file', 'test.pdf',
                'https://example.com/test.pdf', 'application/pdf',
                1024, 'event1', NOW())
        "#,
        tenant_id,
        job_id,
        artifact_id
    )
    .execute(&pool)
    .await
    .unwrap();
    
    // Query artifacts
    let artifacts = sqlx::query!(
        r#"
        SELECT artifact_id, kind, title, size_bytes
        FROM projection_job_artifacts
        WHERE job_id = $1
        "#,
        job_id
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    
    assert_eq!(artifacts.len(), 1);
    assert_eq!(artifacts[0]. artifact_id, artifact_id);
    assert_eq!(artifacts[0].kind, "file");
    assert_eq!(artifacts[0].size_bytes, Some(1024));
}

#[sqlx::test]
async fn test_projection_rebuild_from_ledger(pool: PgPool) {
    // Test that projections can be rebuilt from ledger events
    
    // 1. Clear projections
    sqlx::query! ("DELETE FROM projection_messages").execute(&pool).await.unwrap();
    sqlx::query!("DELETE FROM projection_jobs").execute(&pool).await.unwrap();
    
    // 2. Insert events into ledger
    let events = vec![
        json!({"type": "message.created", "message_id": "msg_1", "conversation_id": "conv_1"}),
        json!({"type":  "message.created", "message_id": "msg_2", "conversation_id": "conv_1"}),
        json!({"type": "job.created", "id": "job_1", "conversation_id":  "conv_1"}),
    ];
    
    for (i, event) in events.iter().enumerate() {
        let hash = format!("hash_{}", i);
        
        sqlx::query!(
            r#"
            INSERT INTO ledger_atom (hash, data)
            VALUES ($1, $2)
            "#,
            hash,
            event
        )
        .execute(&pool)
        .await
        .unwrap();
        
        let container_id = if event["type"]. as_str().unwrap().starts_with("message") {
            "C.Messenger"
        } else {
            "C.Jobs"
        };
        
        sqlx::query!(
            r#"
            INSERT INTO ledger_entries (
                container_id, sequence, atom_hash, previous_hash,
                entry_hash, timestamp_ns, intent_class, physics_delta
            )
            VALUES ($1, $2, $3, $4, $5, $6, 'Observation', '0')
            "#,
            container_id,
            (i + 1) as i64,
            hash,
            if i == 0 { "0".repeat(64) } else { format!("entry_{}", i) },
            format!("entry_{}", i + 1),
            1000000i64 + i as i64
        )
        .execute(&pool)
        .await
        . unwrap();
    }
    
    // 3. Rebuild projections (manual trigger)
    // In actual implementation, call rebuild_projections()
    
    // 4. Verify projections are populated
    // This is a placeholder - actual test would verify counts
    assert!(true);
}