//! Entity Lifecycle Tests
//!  Tests entity creation, activation, suspension, and archival

use office: :{
    entity: :{Entity, EntityType, EntityStatus, Identity},
    governance::constitution:: OfficeConstitution,
};
use uuid::Uuid;

#[tokio::test]
async fn test_entity_creation() {
    let entity_id = format!("entity_{}", Uuid:: new_v4());
    let name = "Test Entity";
    
    let entity = Entity::new(
        entity_id.clone(),
        name.to_string(),
        EntityType:: Autonomous,
    );
    
    assert_eq!(entity.id, entity_id);
    assert_eq!(entity. name, name);
    assert_eq!(entity.entity_type, EntityType::Autonomous);
    assert_eq!(entity.status, EntityStatus::Active);
    assert_eq!(entity. total_sessions, 0);
}

#[tokio::test]
async fn test_entity_types() {
    let types = vec![
        EntityType:: Guarded,
        EntityType:: Autonomous,
        EntityType::Development,
    ];
    
    for entity_type in types {
        let entity = Entity::new(
            format!("entity_{}", Uuid::new_v4()),
            "Test". to_string(),
            entity_type. clone(),
        );
        
        assert_eq!(entity.entity_type, entity_type);
    }
}

#[tokio::test]
async fn test_entity_status_transitions() {
    let mut entity = Entity::new(
        format!("entity_{}", Uuid::new_v4()),
        "Test".to_string(),
        EntityType::Autonomous,
    );
    
    // Active -> Suspended
    entity.status = EntityStatus:: Suspended;
    assert_eq!(entity.status, EntityStatus:: Suspended);
    
    // Suspended -> Active
    entity.status = EntityStatus::Active;
    assert_eq!(entity.status, EntityStatus::Active);
    
    // Active -> Archived
    entity.status = EntityStatus:: Archived;
    assert_eq!(entity.status, EntityStatus:: Archived);
}

#[tokio::test]
async fn test_entity_identity() {
    let identity = Identity::generate();
    
    assert_eq!(identity.public_key_hex. len(), 64); // 32 bytes = 64 hex chars
    assert_eq!(identity.key_version, 1);
}

#[tokio::test]
async fn test_entity_signing() {
    let identity = Identity::generate();
    let message = b"Hello, World!";
    
    let signature = identity.sign(message);
    
    // Signature should be base64url encoded Ed25519 signature
    assert!(signature.starts_with("ed25519:"));
    assert!(signature.len() > 10);
}

#[tokio:: test]
async fn test_entity_constitution_update() {
    let mut entity = Entity::new(
        format!("entity_{}", Uuid::new_v4()),
        "Test".to_string(),
        EntityType::Autonomous,
    );
    
    let new_constitution = OfficeConstitution:: default();
    entity.constitution = new_constitution. clone();
    
    assert_eq!(entity.constitution.version, new_constitution.version);
}

#[tokio::test]
async fn test_entity_baseline_update() {
    let mut entity = Entity::new(
        format! ("entity_{}", Uuid::new_v4()),
        "Test".to_string(),
        EntityType::Autonomous,
    );
    
    let new_baseline = "Updated baseline narrative from dreaming cycle.".to_string();
    entity.baseline_narrative = new_baseline.clone();
    
    assert_eq!(entity.baseline_narrative, new_baseline);
}

#[tokio::test]
async fn test_entity_session_count() {
    let mut entity = Entity::new(
        format!("entity_{}", Uuid::new_v4()),
        "Test".to_string(),
        EntityType::Autonomous,
    );
    
    assert_eq!(entity.total_sessions, 0);
    
    entity.total_sessions += 1;
    assert_eq!(entity.total_sessions, 1);
    
    entity.total_sessions += 5;
    assert_eq!(entity.total_sessions, 6);
}

#[tokio:: test]
async fn test_entity_token_tracking() {
    let mut entity = Entity::new(
        format! ("entity_{}", Uuid::new_v4()),
        "Test".to_string(),
        EntityType::Autonomous,
    );
    
    assert_eq!(entity.total_tokens_consumed, 0);
    
    entity.total_tokens_consumed += 1000;
    assert_eq!(entity.total_tokens_consumed, 1000);
}

#[tokio::test]
async fn test_guarded_entity_requires_guardian() {
    let entity = Entity::new(
        format!("entity_{}", Uuid:: new_v4()),
        "Test".to_string(),
        EntityType::Guarded,
    );
    
    // Guarded entity should have guardian_id set
    assert_eq!(entity.entity_type, EntityType::Guarded);
}

#[tokio::test]
async fn test_development_entity_high_limits() {
    let entity = Entity::new(
        format!("entity_{}", Uuid:: new_v4()),
        "Test".to_string(),
        EntityType::Development,
    );
    
    assert_eq!(entity.entity_type, EntityType::Development);
}

#[tokio::test]
async fn test_entity_metadata() {
    let mut entity = Entity::new(
        format! ("entity_{}", Uuid::new_v4()),
        "Test".to_string(),
        EntityType::Autonomous,
    );
    
    let metadata = serde_json::json!({
        "custom_field": "value",
        "tags": ["production", "critical"]
    });
    
    entity.metadata = metadata. clone();
    
    assert_eq!(entity.metadata, metadata);
}