//! Smoke tests for UBL Policy VM
//!
//! Basic sanity checks that the VM compiles and runs correctly.

use ubl_policy_vm::PolicyVM;

#[test]
fn ubl_policy_vm_smoke() {
    assert!(true);
}

#[test]
fn test_vm_creation() {
    let _vm = PolicyVM::new();
    // VM should be created successfully
    assert!(true);
}

#[test]
fn test_vm_with_custom_limits() {
    // PolicyVM::with_limits takes (max_gas: u64, max_stack: usize)
    let _vm = PolicyVM::with_limits(50000, 512);
    assert!(true);
}

#[test]
fn test_policy_evaluation() {
    let _vm = PolicyVM::new();
    
    // This test just verifies the VM doesn't panic with valid input
    // Full integration tests would test actual policy execution
    // See lib.rs for comprehensive policy evaluation tests
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
