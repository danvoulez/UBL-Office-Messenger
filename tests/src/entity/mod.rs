// Existing entity code...

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_entity_new() {
        let entity = Entity::new(
            "entity_123".to_string(),
            "Test Entity".to_string(),
            EntityType::Autonomous,
        );
        
        assert_eq!(entity.id, "entity_123");
        assert_eq!(entity.name, "Test Entity");
        assert_eq!(entity.entity_type, EntityType::Autonomous);
        assert_eq!(entity.status, EntityStatus::Active);
        assert_eq!(entity.total_sessions, 0);
        assert_eq!(entity.total_tokens_consumed, 0);
    }
    
    #[test]
    fn test_entity_types() {
        let guarded = Entity::new("e1".to_string(), "Test". to_string(), EntityType::Guarded);
        let autonomous = Entity::new("e2".to_string(), "Test".to_string(), EntityType::Autonomous);
        let dev = Entity::new("e3". to_string(), "Test".to_string(), EntityType::Development);
        
        assert!(matches!(guarded.entity_type, EntityType::Guarded));
        assert!(matches!(autonomous.entity_type, EntityType::Autonomous));
        assert!(matches!(dev.entity_type, EntityType:: Development));
    }
    
    #[test]
    fn test_entity_identity() {
        let identity = Identity::generate();
        
        assert_eq!(identity.public_key_hex. len(), 64);
        assert_eq!(identity.key_version, 1);
    }
    
    #[test]
    fn test_entity_signing() {
        let identity = Identity::generate();
        let message = b"test message";
        
        let signature = identity.sign(message);
        
        assert!(signature.starts_with("ed25519:"));
        assert!(signature.len() > 10);
    }
}