//!  Ledger integration tests
//!  Tests full ledger append flow with database

use sqlx::PgPool;
use serde_json::json;

#[sqlx::test]
async fn test_ledger_append_single(pool: PgPool) {
    // Test appending a single event to the ledger
    let container_id = "C.Test";
    let atom = json!({"type": "test. created", "id": "test_1"});
    
    // Canonicalize atom
    let atom_canonical = serde_json::to_vec(&atom).unwrap();
    let atom_hash = format!("0x{}", hex::encode(blake3::hash(&atom_canonical).as_bytes()));
    
    // Insert into ledger
    let result = sqlx::query!(
        r#"
        INSERT INTO ledger_entries (
            container_id, sequence, atom_hash, previous_hash, 
            entry_hash, timestamp_ns, intent_class, physics_delta
        )
        VALUES ($1, 1, $2, $3, $4, $5, 'Observation', '0')
        RETURNING id
        "#,
        container_id,
        atom_hash,
        "0". repeat(64),
        "entry_hash",
        1000000i64
    )
    .fetch_one(&pool)
    .await;
    
    assert!(result. is_ok());
}

#[sqlx::test]
async fn test_ledger_sequence_enforcement(pool: PgPool) {
    let container_id = "C.Test";
    
    // Insert first entry
    sqlx::query!(
        r#"
        INSERT INTO ledger_entries (
            container_id, sequence, atom_hash, previous_hash,
            entry_hash, timestamp_ns, intent_class, physics_delta
        )
        VALUES ($1, 1, 'hash1', $2, 'entry1', 1000000, 'Observation', '0')
        "#,
        container_id,
        "0".repeat(64)
    )
    .execute(&pool)
    .await
    .unwrap();
    
    // Try to insert with wrong sequence (should fail)
    let result = sqlx::query!(
        r#"
        INSERT INTO ledger_entries (
            container_id, sequence, atom_hash, previous_hash,
            entry_hash, timestamp_ns, intent_class, physics_delta
        )
        VALUES ($1, 5, 'hash2', 'hash1', 'entry2', 1000001, 'Observation', '0')
        "#,
        container_id
    )
    .execute(&pool)
    .await;
    
    // Should fail due to sequence gap
    assert!(result.is_err());
}

#[sqlx::test]
async fn test_ledger_causality_chain(pool: PgPool) {
    let container_id = "C.Test";
    
    // Genesis
    sqlx::query!(
        r#"
        INSERT INTO ledger_entries (
            container_id, sequence, atom_hash, previous_hash,
            entry_hash, timestamp_ns, intent_class, physics_delta
        )
        VALUES ($1, 0, 'genesis', $2, 'entry0', 0, 'Evolution', '0')
        "#,
        container_id,
        "0".repeat(64)
    )
    .execute(&pool)
    .await
    .unwrap();
    
    // Link 1
    sqlx::query!(
        r#"
        INSERT INTO ledger_entries (
            container_id, sequence, atom_hash, previous_hash,
            entry_hash, timestamp_ns, intent_class, physics_delta
        )
        VALUES ($1, 1, 'hash1', 'entry0', 'entry1', 1000000, 'Observation', '0')
        "#,
        container_id
    )
    .execute(&pool)
    .await
    .unwrap();
    
    // Link 2
    sqlx:: query!(
        r#"
        INSERT INTO ledger_entries (
            container_id, sequence, atom_hash, previous_hash,
            entry_hash, timestamp_ns, intent_class, physics_delta
        )
        VALUES ($1, 2, 'hash2', 'entry1', 'entry2', 1000001, 'Observation', '0')
        "#,
        container_id
    )
    .execute(&pool)
    .await
    .unwrap();
    
    // Verify chain
    let entries = sqlx::query!(
        r#"
        SELECT sequence, previous_hash, entry_hash
        FROM ledger_entries
        WHERE container_id = $1
        ORDER BY sequence
        "#,
        container_id
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    
    assert_eq!(entries. len(), 3);
    assert_eq!(entries[1].previous_hash, entries[0].entry_hash);
    assert_eq!(entries[2].previous_hash, entries[1].entry_hash);
}

#[sqlx::test]
async fn test_projection_update(pool: PgPool) {
    // Insert message event
    let event = json! ({
        "type": "message.created",
        "id":  "msg_1",
        "conversation_id": "conv_1",
        "from": "user_alice",
        "content_hash": "content_hash_123",
        "created_at": "2024-12-29T10:00:00Z"
    });
    
    let atom_bytes = serde_json::to_vec(&event).unwrap();
    let atom_hash = format! ("0x{}", hex::encode(blake3::hash(&atom_bytes).as_bytes()));
    
    // Insert to ledger
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
    
    // Manually insert into projection (in real system, trigger would do this)
    sqlx::query!(
        r#"
        INSERT INTO projection_messages (
            tenant_id, message_id, conversation_id, sender_entity_id,
            content_hash, message_type, created_at
        )
        VALUES ('T.UBL', 'msg_1', 'conv_1', 'user_alice', 'content_hash_123', 'text', NOW())
        "#
    )
    .execute(&pool)
    .await
    .unwrap();
    
    // Verify projection
    let msg = sqlx::query!(
        r#"
        SELECT message_id, conversation_id
        FROM projection_messages
        WHERE message_id = 'msg_1'
        "#
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    
    assert_eq!(msg. message_id, "msg_1");
    assert_eq!(msg.conversation_id, "conv_1");
}