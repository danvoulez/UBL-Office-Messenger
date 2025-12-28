//! # UBL Policy VM
//!
//! **Title:** SPEC-UBL-POLICY v1.0  
//! **Status:** NORMATIVE  
//! **Change-Control:** STRICT (no retroactive changes)  
//! **Hash:** BLAKE3 | **Signature:** Ed25519  
//! **Freeze-Date:** 2025-12-25  
//! **Governed by:** SPEC-UBL-CORE v1.0, SPEC-UBL-POLICY v1.0  
//!
//! TDLN - Deterministic Translation of Language to Notation
//! Executor WASM determinístico (semantically blind)
//!
//! ## Architecture
//!
//! ```text
//! PolicyDefinition (JSON)
//!         │
//!         ▼
//! ┌───────────────┐
//! │   Compiler    │ → Compiles rules to bytecode
//! └───────────────┘
//!         │
//!         ▼
//! CompiledPolicy (bytecode + constants)
//!         │
//!         ▼
//! ┌───────────────┐
//! │  BytecodeVM   │ → Executes deterministically
//! └───────────────┘
//!         │
//!         ▼
//! PolicyResult (Allow/Deny)
//! ```

#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod bytecode;
pub mod compiler;

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use thiserror::Error;

// Re-exports
pub use bytecode::{
    BytecodeVM, CompiledPolicy, ExecutionContext, PolicyResult,
    BytecodeError, Opcode, Value, VMConfig,
    // Security limits
    MAX_BYTECODE_SIZE, MAX_CONSTANTS, MAX_STRING_LENGTH,
    MAX_GAS, MAX_STACK_SIZE,
    // Intent class constants
    INTENT_CLASS_OBSERVATION, INTENT_CLASS_CONSERVATION,
    INTENT_CLASS_ENTROPY, INTENT_CLASS_EVOLUTION,
};
pub use compiler::{
    PolicyCompiler, PolicyDefinition, PolicyRule, 
    AppliesTo, IntentClassSpec, Constraint,
    create_default_policy, CompilerError,
    MAX_RULES, MAX_CONSTRAINTS_PER_RULE,
};

/// Errors from policy evaluation
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum PolicyError {
    /// Policy not found
    #[error("Policy not found: {0}")]
    PolicyNotFound(String),

    /// Policy execution failed
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    /// Invalid policy bytecode
    #[error("Invalid bytecode")]
    InvalidBytecode,

    /// Timeout during execution
    #[error("Execution timeout")]
    Timeout,
    
    /// Compilation error
    #[error("Compilation error: {0}")]
    CompilationError(String),
}

/// Result type for policy operations
pub type Result<T> = std::result::Result<T, PolicyError>;

/// Translation decision from TDLN (SPEC-UBL-POLICY v1.0 §6)
/// Legacy type for compatibility - use PolicyResult from bytecode module
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TranslationDecision {
    /// Allow the translation with constraints
    Allow {
        /// Intent class permitted
        intent_class: u8,
        /// Pact required (if any)
        required_pact: Option<String>,
        /// Constraints snapshot
        constraints: Vec<ConstraintSnapshot>,
    },
    /// Deny the translation
    Deny {
        /// Reason for denial
        reason: String,
    },
}

impl From<PolicyResult> for TranslationDecision {
    fn from(result: PolicyResult) -> Self {
        match result {
            PolicyResult::Allow { intent_class, required_pact, constraints } => {
                TranslationDecision::Allow {
                    intent_class,
                    required_pact,
                    constraints: constraints.into_iter().map(|c| ConstraintSnapshot {
                        kind: "applied".to_string(),
                        value: c,
                    }).collect(),
                }
            }
            PolicyResult::Deny { reason } => {
                TranslationDecision::Deny { reason }
            }
        }
    }
}

/// A constraint from policy evaluation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConstraintSnapshot {
    /// Type of constraint (e.g., "max_delta", "time_window")
    pub kind: String,
    /// Value of the constraint (JSON-serializable)
    pub value: String,
}

/// Policy definition (legacy - use PolicyDefinition from compiler module)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    /// Policy identifier
    pub policy_id: String,
    
    /// Version
    pub version: String,
    
    /// Hash of the policy bytecode (BLAKE3)
    pub bytecode_hash: String,
    
    /// Compiled bytecode (internal)
    #[serde(skip)]
    pub compiled: Option<CompiledPolicy>,
    
    /// Human-readable description
    pub description: String,
}

/// Policy evaluation context (legacy - use ExecutionContext from bytecode module)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationContext {
    /// Container ID
    pub container_id: String,
    
    /// Actor (public key or entity ID)
    pub actor: String,
    
    /// Intent payload (JSON)
    pub intent: serde_json::Value,
    
    /// Current state (optional)
    pub state: Option<serde_json::Value>,
    
    /// Timestamp
    pub timestamp: i64,
}

impl From<EvaluationContext> for ExecutionContext {
    fn from(ctx: EvaluationContext) -> Self {
        ExecutionContext {
            container_id: ctx.container_id,
            actor: ctx.actor,
            intent: ctx.intent,
            state: ctx.state,
            timestamp: ctx.timestamp,
        }
    }
}

/// Policy VM - executes TDLN policies
pub struct PolicyVM {
    /// Registered policies (policy_id -> CompiledPolicy)
    policies: HashMap<String, CompiledPolicy>,
    /// Bytecode VM for execution
    vm: BytecodeVM,
    /// Compiler for on-the-fly compilation
    compiler: PolicyCompiler,
}

impl PolicyVM {
    /// Create a new policy VM
    pub fn new() -> Self {
        Self {
            policies: HashMap::new(),
            vm: BytecodeVM::default(),
            compiler: PolicyCompiler::new(),
        }
    }

    /// Create with custom limits
    pub fn with_limits(max_gas: u64, max_stack: usize) -> Self {
        Self {
            policies: HashMap::new(),
            vm: BytecodeVM::new(max_gas, max_stack),
            compiler: PolicyCompiler::new(),
        }
    }

    /// Register a compiled policy
    pub fn register_compiled(&mut self, policy: CompiledPolicy) {
        self.policies.insert(policy.policy_id.clone(), policy);
    }

    /// Register a policy definition (compiles it)
    pub fn register(&mut self, definition: &PolicyDefinition) {
        let compiled = self.compiler.compile(definition);
        self.policies.insert(compiled.policy_id.clone(), compiled);
    }

    /// Register a legacy Policy struct
    pub fn register_legacy(&mut self, policy: Policy) {
        if let Some(compiled) = policy.compiled {
            self.policies.insert(policy.policy_id, compiled);
        }
    }

    /// Get a registered policy
    pub fn get_policy(&self, policy_id: &str) -> Option<&CompiledPolicy> {
        self.policies.get(policy_id)
    }

    /// Evaluate a policy (SPEC-UBL-POLICY v1.0 §6)
    pub fn evaluate(
        &self,
        policy_id: &str,
        context: &EvaluationContext,
    ) -> Result<TranslationDecision> {
        let policy = self.policies
            .get(policy_id)
            .ok_or_else(|| PolicyError::PolicyNotFound(policy_id.to_string()))?;

        let exec_ctx = ExecutionContext {
            container_id: context.container_id.clone(),
            actor: context.actor.clone(),
            intent: context.intent.clone(),
            state: context.state.clone(),
            timestamp: context.timestamp,
        };

        let result = self.vm.execute(policy, &exec_ctx)
            .map_err(|e| PolicyError::ExecutionFailed(e.to_string()))?;

        Ok(result.into())
    }

    /// Check if a policy is registered
    pub fn has_policy(&self, policy_id: &str) -> bool {
        self.policies.contains_key(policy_id)
    }

    /// List all registered policy IDs
    pub fn list_policies(&self) -> Vec<&str> {
        self.policies.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for PolicyVM {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_context(intent_type: &str, amount: Option<i64>) -> EvaluationContext {
        let mut intent = json!({"type": intent_type});
        if let Some(amt) = amount {
            intent["amount"] = json!(amt);
        }
        
        EvaluationContext {
            container_id: "test".to_string(),
            actor: "alice".to_string(),
            intent,
            state: None,
            timestamp: 1000,
        }
    }

    #[test]
    fn test_register_and_evaluate() {
        let mut vm = PolicyVM::new();
        
        // Create and register a policy
        let definition = PolicyDefinition {
            policy_id: "test_policy".to_string(),
            version: "1.0".to_string(),
            description: "Test".to_string(),
            rules: vec![
                PolicyRule {
                    rule_id: "allow_observe".to_string(),
                    applies_to: AppliesTo::Global,
                    intent_class: IntentClassSpec::Observation,
                    constraints: vec![
                        Constraint::IntentTypeEquals { value: "observe".to_string() },
                    ],
                    required_pact: None,
                },
            ],
            default_deny: true,
        };
        
        vm.register(&definition);
        
        // Evaluate - should allow
        let context = make_context("observe", None);
        let decision = vm.evaluate("test_policy", &context).unwrap();
        
        match decision {
            TranslationDecision::Allow { intent_class, .. } => {
                assert_eq!(intent_class, 0x00);
            }
            _ => panic!("Expected Allow"),
        }
    }

    #[test]
    fn test_default_deny() {
        let mut vm = PolicyVM::new();
        
        let definition = PolicyDefinition {
            policy_id: "strict".to_string(),
            version: "1.0".to_string(),
            description: "Strict policy".to_string(),
            rules: vec![
                PolicyRule {
                    rule_id: "only_observe".to_string(),
                    applies_to: AppliesTo::Global,
                    intent_class: IntentClassSpec::Observation,
                    constraints: vec![
                        Constraint::IntentTypeEquals { value: "observe".to_string() },
                    ],
                    required_pact: None,
                },
            ],
            default_deny: true,
        };
        
        vm.register(&definition);
        
        // Unknown intent - should deny
        let context = make_context("hack", None);
        let decision = vm.evaluate("strict", &context).unwrap();
        
        assert!(matches!(decision, TranslationDecision::Deny { .. }));
    }

    #[test]
    fn test_pact_requirement() {
        let mut vm = PolicyVM::new();
        
        let definition = PolicyDefinition {
            policy_id: "evolution".to_string(),
            version: "1.0".to_string(),
            description: "Evolution policy".to_string(),
            rules: vec![
                PolicyRule {
                    rule_id: "evolve_with_pact".to_string(),
                    applies_to: AppliesTo::Global,
                    intent_class: IntentClassSpec::Evolution,
                    constraints: vec![
                        Constraint::IntentTypeEquals { value: "evolve".to_string() },
                    ],
                    required_pact: Some("evolution_l5".to_string()),
                },
            ],
            default_deny: true,
        };
        
        vm.register(&definition);
        
        let context = make_context("evolve", None);
        let decision = vm.evaluate("evolution", &context).unwrap();
        
        match decision {
            TranslationDecision::Allow { intent_class, required_pact, .. } => {
                assert_eq!(intent_class, 0x03);
                assert_eq!(required_pact, Some("evolution_l5".to_string()));
            }
            _ => panic!("Expected Allow with pact"),
        }
    }

    #[test]
    fn test_amount_thresholds() {
        let mut vm = PolicyVM::new();
        
        let definition = PolicyDefinition {
            policy_id: "transfer".to_string(),
            version: "1.0".to_string(),
            description: "Transfer policy".to_string(),
            rules: vec![
                PolicyRule {
                    rule_id: "small_transfer".to_string(),
                    applies_to: AppliesTo::Global,
                    intent_class: IntentClassSpec::Conservation,
                    constraints: vec![
                        Constraint::IntentTypeEquals { value: "transfer".to_string() },
                        Constraint::AmountMax { max: 10000 },
                    ],
                    required_pact: None,
                },
                PolicyRule {
                    rule_id: "large_transfer".to_string(),
                    applies_to: AppliesTo::Global,
                    intent_class: IntentClassSpec::Conservation,
                    constraints: vec![
                        Constraint::IntentTypeEquals { value: "transfer".to_string() },
                        Constraint::AmountMin { min: 10001 },
                    ],
                    required_pact: Some("high_value".to_string()),
                },
            ],
            default_deny: true,
        };
        
        vm.register(&definition);
        
        // Small transfer - no pact
        let context = make_context("transfer", Some(100));
        let decision = vm.evaluate("transfer", &context).unwrap();
        match decision {
            TranslationDecision::Allow { required_pact, .. } => {
                assert!(required_pact.is_none());
            }
            _ => panic!("Expected Allow"),
        }
        
        // Large transfer - require pact
        let context = make_context("transfer", Some(20000));
        let decision = vm.evaluate("transfer", &context).unwrap();
        match decision {
            TranslationDecision::Allow { required_pact, .. } => {
                assert!(required_pact.is_some());
            }
            _ => panic!("Expected Allow with pact"),
        }
    }

    #[test]
    fn test_policy_not_found() {
        let vm = PolicyVM::new();
        
        let context = make_context("test", None);
        let result = vm.evaluate("nonexistent", &context);
        
        assert!(matches!(result, Err(PolicyError::PolicyNotFound(_))));
    }
}
