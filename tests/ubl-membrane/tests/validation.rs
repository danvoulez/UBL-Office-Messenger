//! Membrane validation tests
//! SPEC-UBL-MEMBRANE v1.0 compliance

use ubl_membrane::{validate, LedgerState, MembraneError};
use ubl_link: :{LinkCommit, IntentClass};
use serde_json:: json;

fn create_test_link(sequence: u64, previous_hash: &str) -> LinkCommit {
    LinkCommit {
        version: 1,
        container_id: "C.Test".to_string(),
        sequence,
        previous_hash: previous_hash.to_string(),
        atom: json!({"type": "test"}),
        atom_hash: "test_hash".to_string(),
        timestamp: 1000000,
        intent_class: IntentClass::Observation,
        physics_delta: "0".to_string(),
        signature: Some("ed25519:valid_signature".to_string()),
        actor_id: "test_user".to_string(),
    }
}

#[test]
fn test_valid_link_passes() {
    let state = LedgerState {
        container_id: "C.Test". to_string(),
        last_hash: "prev_hash".to_string(),
        next_sequence: 1,
        physical_balance: 0,
    };
    
    let link = create_test_link(1, "prev_hash");
    
    let result = validate(&link, &state);
    assert!(result.is_ok());
}

#[test]
fn test_wrong_version_fails() {
    let state = LedgerState {
        container_id: "C.Test".to_string(),
        last_hash: "prev_hash". to_string(),
        next_sequence: 1,
        physical_balance: 0,
    };
    
    let mut link = create_test_link(1, "prev_hash");
    link.version = 999;
    
    let result = validate(&link, &state);
    assert!(matches!(result, Err(MembraneError:: InvalidVersion)));
}

#[test]
fn test_sequence_mismatch_fails() {
    let state = LedgerState {
        container_id: "C.Test".to_string(),
        last_hash: "prev_hash".to_string(),
        next_sequence: 5,
        physical_balance: 0,
    };
    
    let link = create_test_link(1, "prev_hash"); // Wrong sequence
    
    let result = validate(&link, &state);
    assert!(matches!(result, Err(MembraneError:: SequenceMismatch)));
}

#[test]
fn test_reality_drift_fails() {
    let state = LedgerState {
        container_id: "C.Test".to_string(),
        last_hash: "correct_hash".to_string(),
        next_sequence: 1,
        physical_balance: 0,
    };
    
    let link = create_test_link(1, "wrong_hash"); // Wrong previous hash
    
    let result = validate(&link, &state);
    assert!(matches!(result, Err(MembraneError:: RealityDrift)));
}

#[test]
fn test_container_mismatch_fails() {
    let state = LedgerState {
        container_id: "C.Test".to_string(),
        last_hash: "prev_hash".to_string(),
        next_sequence: 1,
        physical_balance: 0,
    };
    
    let mut link = create_test_link(1, "prev_hash");
    link.container_id = "C.Wrong".to_string();
    
    let result = validate(&link, &state);
    assert!(matches!(result, Err(MembraneError::InvalidTarget)));
}

#[test]
fn test_observation_requires_zero_delta() {
    let state = LedgerState {
        container_id: "C.Test".to_string(),
        last_hash: "prev_hash".to_string(),
        next_sequence:  1,
        physical_balance: 0,
    };
    
    let mut link = create_test_link(1, "prev_hash");
    link.intent_class = IntentClass::Observation;
    link.physics_delta = "100".to_string(); // Non-zero delta for observation
    
    let result = validate(&link, &state);
    assert!(matches!(result, Err(MembraneError:: PhysicsViolation { .. })));
}

#[test]
fn test_conservation_zero_sum() {
    let state = LedgerState {
        container_id: "C.Test".to_string(),
        last_hash: "prev_hash".to_string(),
        next_sequence:  1,
        physical_balance: 100,
    };
    
    let mut link = create_test_link(1, "prev_hash");
    link.intent_class = IntentClass::Conservation;
    link.physics_delta = "0".to_string(); // Zero-sum transfer
    
    let result = validate(&link, &state);
    assert!(result.is_ok());
}

#[test]
fn test_entropy_increases_balance() {
    let state = LedgerState {
        container_id: "C.Test".to_string(),
        last_hash: "prev_hash".to_string(),
        next_sequence:  1,
        physical_balance: 100,
    };
    
    let mut link = create_test_link(1, "prev_hash");
    link.intent_class = IntentClass:: Entropy;
    link.physics_delta = "50".to_string(); // Creates value
    
    let result = validate(&link, &state);
    assert!(result.is_ok());
}

#[test]
fn test_genesis_link_validation() {
    let state = LedgerState {
        container_id: "C.Test".to_string(),
        last_hash: "0".repeat(64),
        next_sequence: 0,
        physical_balance: 0,
    };
    
    let genesis = LinkCommit {
        version: 1,
        container_id:  "C.Test".to_string(),
        sequence: 0,
        previous_hash: "0".repeat(64),
        atom: json!({"type": "genesis"}),
        atom_hash:  "genesis". to_string(),
        timestamp:  0,
        intent_class: IntentClass::Evolution,
        physics_delta: "0". to_string(),
        signature:  Some("ed25519:sig".to_string()),
        actor_id: "system".to_string(),
    };
    
    let result = validate(&genesis, &state);
    assert!(result.is_ok());
}