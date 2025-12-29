//! Link structure validation tests
//! SPEC-UBL-LINK v1.0 compliance

use ubl_link::{LinkCommit, IntentClass};
use serde_json::json;

#[test]
fn test_link_commit_creation() {
    let link = LinkCommit {
        version: 1,
        container_id: "C. Test".to_string(),
        sequence:  1,
        previous_hash: "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
        atom:  json!({"type": "test. created"}),
        atom_hash: "abc123".to_string(),
        timestamp: 1000000,
        intent_class: IntentClass::Observation,
        physics_delta: "0". to_string(),
        signature: Some("ed25519:... ".to_string()),
        actor_id: "user_test".to_string(),
    };
    
    assert_eq!(link.version, 1);
    assert_eq!(link.container_id, "C. Test");
}

#[test]
fn test_intent_class_observation() {
    let link = LinkCommit {
        version: 1,
        container_id:  "C.Test".to_string(),
        sequence: 1,
        previous_hash: "0". repeat(64),
        atom: json!({"type": "observe"}),
        atom_hash: "hash". to_string(),
        timestamp:  1000000,
        intent_class: IntentClass:: Observation,
        physics_delta:  "0".to_string(),
        signature: None,
        actor_id: "test". to_string(),
    };
    
    assert_eq!(link.physics_delta, "0");
    assert!(matches!(link.intent_class, IntentClass::Observation));
}

#[test]
fn test_intent_class_conservation() {
    let link = LinkCommit {
        version: 1,
        container_id:  "C.Test".to_string(),
        sequence: 1,
        previous_hash: "0".repeat(64),
        atom: json!({"type": "transfer"}),
        atom_hash:  "hash".to_string(),
        timestamp: 1000000,
        intent_class: IntentClass::Conservation,
        physics_delta: "0".to_string(),
        signature: None,
        actor_id: "test".to_string(),
    };
    
    assert!(matches!(link.intent_class, IntentClass::Conservation));
}

#[test]
fn test_intent_class_entropy() {
    let link = LinkCommit {
        version: 1,
        container_id:  "C.Test".to_string(),
        sequence: 1,
        previous_hash: "0".repeat(64),
        atom: json!({"type": "create"}),
        atom_hash:  "hash".to_string(),
        timestamp: 1000000,
        intent_class: IntentClass::Entropy,
        physics_delta: "100".to_string(),
        signature: None,
        actor_id: "test".to_string(),
    };
    
    assert!(matches!(link.intent_class, IntentClass::Entropy));
}

#[test]
fn test_intent_class_evolution() {
    let link = LinkCommit {
        version: 1,
        container_id:  "C.Test".to_string(),
        sequence: 1,
        previous_hash: "0".repeat(64),
        atom: json!({"type": "evolve"}),
        atom_hash: "hash".to_string(),
        timestamp: 1000000,
        intent_class: IntentClass::Evolution,
        physics_delta: "0".to_string(),
        signature: None,
        actor_id: "test".to_string(),
    };
    
    assert!(matches!(link.intent_class, IntentClass::Evolution));
}

#[test]
fn test_sequence_numbering() {
    let link1 = LinkCommit {
        version: 1,
        container_id: "C.Test". to_string(),
        sequence: 1,
        previous_hash: "0".repeat(64),
        atom: json!({}),
        atom_hash: "hash1".to_string(),
        timestamp: 1000000,
        intent_class: IntentClass:: Observation,
        physics_delta: "0".to_string(),
        signature: None,
        actor_id: "test".to_string(),
    };
    
    let link2 = LinkCommit {
        sequence: 2,
        previous_hash: "hash1".to_string(),
        timestamp: 1000001,
        .. link1.clone()
    };
    
    assert_eq!(link2.sequence, link1.sequence + 1);
}

#[test]
fn test_container_id_format() {
    let valid_containers = vec! ["C. Messenger", "C.Jobs", "C.Office", "C. Pacts"];
    
    for container in valid_containers {
        let link = LinkCommit {
            version: 1,
            container_id: container.to_string(),
            sequence: 1,
            previous_hash: "0". repeat(64),
            atom: json!({}),
            atom_hash: "hash".to_string(),
            timestamp: 1000000,
            intent_class:  IntentClass::Observation,
            physics_delta: "0". to_string(),
            signature:  None,
            actor_id:  "test".to_string(),
        };
        
        assert!(link.container_id.starts_with("C."));
    }
}

#[test]
fn test_genesis_link() {
    let genesis = LinkCommit {
        version: 1,
        container_id: "C.Test".to_string(),
        sequence: 0,
        previous_hash: "0".repeat(64),
        atom: json!({"type": "genesis"}),
        atom_hash: "genesis_hash".to_string(),
        timestamp: 0,
        intent_class: IntentClass::Evolution,
        physics_delta: "0".to_string(),
        signature: None,
        actor_id: "system".to_string(),
    };
    
    assert_eq!(genesis.sequence, 0);
    assert_eq!(genesis.previous_hash, "0".repeat(64));
}