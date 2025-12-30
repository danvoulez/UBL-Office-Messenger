//! Link structure validation tests
//! SPEC-UBL-LINK v1.0 compliance
//!
//! Tests the LinkCommit structure as defined in ubl-link crate.
//! Fields per spec:
//! - version: u8
//! - container_id: String  
//! - expected_sequence: u64 (causal control)
//! - previous_hash: String
//! - atom_hash: String
//! - intent_class: IntentClass
//! - physics_delta: i128
//! - pact: Option<PactProof>
//! - author_pubkey: String
//! - signature: String

use ubl_link::{LinkCommit, IntentClass};

/// Helper to create a test LinkCommit with common defaults
fn make_test_link(container_id: &str, expected_sequence: u64, intent_class: IntentClass, physics_delta: i128) -> LinkCommit {
    LinkCommit {
        version: 1,
        container_id: container_id.to_string(),
        expected_sequence,
        previous_hash: "0".repeat(64),
        atom_hash: "test_atom_hash".to_string(),
        intent_class,
        physics_delta,
        pact: None,
        author_pubkey: "ed25519_test_pubkey".to_string(),
        signature: "ed25519_test_signature".to_string(),
    }
}

#[test]
fn test_link_commit_creation() {
    let link = make_test_link("C.Test", 1, IntentClass::Observation, 0);
    
    assert_eq!(link.version, 1);
    assert_eq!(link.container_id, "C.Test");
    assert_eq!(link.expected_sequence, 1);
}

#[test]
fn test_intent_class_observation() {
    let link = make_test_link("C.Test", 1, IntentClass::Observation, 0);
    
    assert_eq!(link.physics_delta, 0);
    assert!(matches!(link.intent_class, IntentClass::Observation));
}

#[test]
fn test_intent_class_conservation() {
    let link = make_test_link("C.Test", 1, IntentClass::Conservation, 0);
    
    assert!(matches!(link.intent_class, IntentClass::Conservation));
}

#[test]
fn test_intent_class_entropy() {
    let link = make_test_link("C.Test", 1, IntentClass::Entropy, 100);
    
    assert!(matches!(link.intent_class, IntentClass::Entropy));
    assert_eq!(link.physics_delta, 100);
}

#[test]
fn test_intent_class_evolution() {
    let link = make_test_link("C.Test", 1, IntentClass::Evolution, 0);
    
    assert!(matches!(link.intent_class, IntentClass::Evolution));
}

#[test]
fn test_sequence_numbering() {
    let link1 = make_test_link("C.Test", 1, IntentClass::Observation, 0);
    
    let link2 = LinkCommit {
        expected_sequence: 2,
        previous_hash: "hash_of_link1".to_string(),
        ..link1.clone()
    };
    
    assert_eq!(link2.expected_sequence, link1.expected_sequence + 1);
}

#[test]
fn test_container_id_format() {
    let valid_containers = vec!["C.Messenger", "C.Jobs", "C.Office", "C.Pacts"];
    
    for container in valid_containers {
        let link = make_test_link(container, 1, IntentClass::Observation, 0);
        assert!(link.container_id.starts_with("C."));
    }
}

#[test]
fn test_genesis_link() {
    let genesis = LinkCommit {
        version: 1,
        container_id: "C.Test".to_string(),
        expected_sequence: 0,
        previous_hash: "0".repeat(64),
        atom_hash: "genesis_hash".to_string(),
        intent_class: IntentClass::Evolution,
        physics_delta: 0,
        pact: None,
        author_pubkey: "system".to_string(),
        signature: "genesis_signature".to_string(),
    };
    
    assert_eq!(genesis.expected_sequence, 0);
    assert_eq!(genesis.previous_hash, "0".repeat(64));
}

#[test]
fn test_signing_bytes_excludes_signature() {
    let link = make_test_link("C.Test", 1, IntentClass::Conservation, -50);
    
    let bytes = link.signing_bytes();
    
    // Verify signing bytes are deterministic
    assert_eq!(bytes, link.signing_bytes());
    
    // Verify signature is NOT in signing bytes (by checking bytes don't contain signature)
    let sig_bytes = link.signature.as_bytes();
    assert!(!bytes.windows(sig_bytes.len()).any(|w| w == sig_bytes));
}

#[test]
fn test_physics_delta_negative() {
    // Conservation can have negative delta (one side of a transfer)
    let link = make_test_link("C.Test", 1, IntentClass::Conservation, -100);
    
    assert_eq!(link.physics_delta, -100);
}

#[test]
fn test_intent_class_byte_values() {
    assert_eq!(IntentClass::Observation.as_byte(), 0x00);
    assert_eq!(IntentClass::Conservation.as_byte(), 0x01);
    assert_eq!(IntentClass::Entropy.as_byte(), 0x02);
    assert_eq!(IntentClass::Evolution.as_byte(), 0x03);
}
