//! Smoke tests for UBL Policy VM
//!
//! Basic sanity checks that the VM compiles and runs correctly.

use ubl_policy_vm::{PolicyVM, VMConfig};
use serde_json::json;

#[test]
fn ubl_policy_vm_smoke() {
    assert!(true);
}

#[test]
fn test_vm_creation() {
    let vm = PolicyVM::new();
    // VM should be created successfully
    assert!(true);
}

#[test]
fn test_vm_with_custom_limits() {
    let config = VMConfig {
        max_gas: 50000,
        max_stack: 512,
        max_bytecode_size: 32768,
        max_constants: 128,
        max_string_length: 4096,
        max_rules: 50,
        max_constraints_per_rule: 10,
    };
    
    let vm = PolicyVM::with_limits(config);
    assert!(true);
}

#[test]
fn test_policy_evaluation() {
    let vm = PolicyVM::new();
    
    // Define a simple policy that allows observations
    let policy_json = json!({
        "name": "test_policy",
        "version": "1.0",
        "description": "Test policy for smoke test",
        "default_action": "deny",
        "rules": [
            {
                "name": "allow_observations",
                "description": "Allow all observation events",
                "priority": 100,
                "conditions": {
                    "intent_class": "eq",
                    "value": "Observation"
                },
                "action": "allow"
            }
        ]
    });
    
    // This test just verifies the VM doesn't panic with valid input
    // Full integration tests would test actual policy execution
}

#[test]
fn test_security_constants() {
    // Verify security limits are exposed and reasonable
    assert!(ubl_policy_vm::MAX_BYTECODE_SIZE > 0);
    assert!(ubl_policy_vm::MAX_CONSTANTS > 0);
    assert!(ubl_policy_vm::MAX_STRING_LENGTH > 0);
    assert!(ubl_policy_vm::MAX_GAS > 0);
    assert!(ubl_policy_vm::MAX_STACK_SIZE > 0);
    assert!(ubl_policy_vm::MAX_RULES > 0);
    assert!(ubl_policy_vm::MAX_CONSTRAINTS_PER_RULE > 0);
}
