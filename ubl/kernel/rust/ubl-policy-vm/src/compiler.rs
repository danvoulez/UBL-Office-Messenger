//! Policy Compiler - Compiles TDLN rules to bytecode
//!
//! SPEC-UBL-POLICY v1.0 §9:
//! > Uma política TDLN DEVE ser compilável para:
//! > - WASM (execução segura)
//! > - bytecode verificável
//!
//! This module compiles policy rules (expressed in JSON) to bytecode.
//!
//! ## Security Features
//! - Validates policy before compilation
//! - Limits on rules and constraints
//! - String length validation

use serde::{Deserialize, Serialize};
use thiserror::Error;
use super::bytecode::{CompiledPolicy, Opcode};

// ============================================================================
// COMPILER ERRORS
// ============================================================================

/// Errors during policy compilation
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum CompilerError {
    /// Too many rules in policy
    #[error("Too many rules: {0} (max {1})")]
    TooManyRules(usize, usize),

    /// Too many constraints in a rule
    #[error("Too many constraints in rule '{rule_id}': {count} (max {max})")]
    TooManyConstraints {
        /// Rule identifier that has too many constraints
        rule_id: String,
        /// Number of constraints found
        count: usize,
        /// Maximum allowed constraints
        max: usize,
    },

    /// String value too long
    #[error("String '{0}' too long: {1} bytes")]
    StringTooLong(String, usize),

    /// Invalid constraint configuration
    #[error("Invalid constraint: {0}")]
    InvalidConstraint(String),
}

/// A policy rule definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRule {
    /// Rule identifier
    pub rule_id: String,
    /// Applies to (container, namespace, etc.)
    pub applies_to: AppliesTo,
    /// Allowed intent class
    pub intent_class: IntentClassSpec,
    /// Constraints to check
    pub constraints: Vec<Constraint>,
    /// Required pact (if any)
    pub required_pact: Option<String>,
}

/// Defines the scope where a policy rule applies
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AppliesTo {
    /// All containers
    Global,
    /// Specific container
    Container { 
        /// Container identifier
        id: String 
    },
    /// Namespace prefix
    Namespace { 
        /// Namespace prefix string
        prefix: String 
    },
}

/// Intent class specification for policy rules
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IntentClassSpec {
    /// Observation intent class (0x00) - read-only queries
    Observation,
    /// Conservation intent class (0x01) - read-only operations
    Conservation,
    /// Entropy intent class (0x02) - state-changing mutations
    Entropy,
    /// Evolution intent class (0x03) - schema changes, requires multi-sig
    Evolution,
}

impl IntentClassSpec {
    /// Convert intent class spec to byte value (0x00-0x03)
    pub fn to_byte(&self) -> u8 {
        match self {
            IntentClassSpec::Observation => 0x00,
            IntentClassSpec::Conservation => 0x01,
            IntentClassSpec::Entropy => 0x02,
            IntentClassSpec::Evolution => 0x03,
        }
    }
}

/// A constraint in a policy rule
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Constraint {
    /// Check intent type equals value
    IntentTypeEquals { 
        /// Expected intent type value
        value: String 
    },
    /// Check intent amount is <= max
    AmountMax { 
        /// Maximum allowed amount
        max: i64 
    },
    /// Check intent amount is >= min
    AmountMin { 
        /// Minimum required amount
        min: i64 
    },
    /// Check container starts with prefix
    ContainerPrefix { 
        /// Required container ID prefix
        prefix: String 
    },
    /// Check actor equals value
    ActorEquals { 
        /// Required actor identifier
        actor: String 
    },
    /// Custom field check
    FieldEquals { 
        /// Field name to check
        field: String, 
        /// Expected field value
        value: String 
    },
}

/// Policy definition (collection of rules)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyDefinition {
    /// Unique policy identifier
    pub policy_id: String,
    /// Semantic version (e.g., "1.0.0")
    pub version: String,
    /// Human-readable description
    pub description: String,
    /// List of policy rules
    pub rules: Vec<PolicyRule>,
    /// Default action if no rule matches
    pub default_deny: bool,
}

// ============================================================================
// COMPILER LIMITS
// ============================================================================

/// Maximum rules per policy
pub const MAX_RULES: usize = 256;

/// Maximum constraints per rule
pub const MAX_CONSTRAINTS_PER_RULE: usize = 32;

// ============================================================================
// COMPILER
// ============================================================================

/// Compiler for policy rules
pub struct PolicyCompiler {
    constants: Vec<String>,
    code: Vec<u8>,
}

impl PolicyCompiler {
    /// Create a new compiler instance
    pub fn new() -> Self {
        Self {
            constants: Vec::new(),
            code: Vec::new(),
        }
    }

    /// Validate a policy definition before compilation
    pub fn validate(policy: &PolicyDefinition) -> Result<(), CompilerError> {
        // Check rule count
        if policy.rules.len() > MAX_RULES {
            return Err(CompilerError::TooManyRules(policy.rules.len(), MAX_RULES));
        }

        // Check each rule
        for (_i, rule) in policy.rules.iter().enumerate() {
            if rule.constraints.len() > MAX_CONSTRAINTS_PER_RULE {
                return Err(CompilerError::TooManyConstraints {
                    rule_id: rule.rule_id.clone(),
                    count: rule.constraints.len(),
                    max: MAX_CONSTRAINTS_PER_RULE,
                });
            }

            // Validate string lengths
            if rule.rule_id.len() > 256 {
                return Err(CompilerError::StringTooLong("rule_id".to_string(), rule.rule_id.len()));
            }

            if let Some(ref pact) = rule.required_pact {
                if pact.len() > 256 {
                    return Err(CompilerError::StringTooLong("required_pact".to_string(), pact.len()));
                }
            }
        }

        Ok(())
    }

    /// Compile a policy definition to bytecode
    /// 
    /// # Panics
    /// Does not panic - returns valid bytecode or fails validation.
    pub fn compile(&mut self, policy: &PolicyDefinition) -> CompiledPolicy {
        self.constants.clear();
        self.code.clear();

        // Compile each rule as a condition block
        for rule in &policy.rules {
            self.compile_rule(rule);
        }

        // Default action at the end
        if policy.default_deny {
            self.emit_deny("No matching rule");
        } else {
            // Default allow with Observation
            self.emit_push_i64(0);
            self.emit(Opcode::Allow);
        }

        CompiledPolicy::new(
            &policy.policy_id,
            &policy.version,
            std::mem::take(&mut self.code),
            std::mem::take(&mut self.constants),
        )
    }

    /// Compile with validation
    pub fn compile_validated(&mut self, policy: &PolicyDefinition) -> Result<CompiledPolicy, CompilerError> {
        Self::validate(policy)?;
        Ok(self.compile(policy))
    }

    fn compile_rule(&mut self, rule: &PolicyRule) {
        // Each rule: check all constraints, if all pass -> Allow, else continue to next rule
        
        // Collect positions where we need to patch jump addresses
        let mut jump_positions: Vec<usize> = Vec::new();
        
        for constraint in &rule.constraints {
            self.compile_constraint(constraint);
            // Store position of JumpIfNot opcode
            jump_positions.push(self.code.len());
            // JumpIfNot to next rule (placeholder address 0x0000)
            self.emit(Opcode::JumpIfNot);
            self.emit_u16(0); // Will be patched
        }

        // All constraints passed - emit Allow with intent class
        self.emit_push_i64(rule.intent_class.to_byte() as i64);
        
        if let Some(ref pact_id) = rule.required_pact {
            let pact_idx = self.add_constant(pact_id);
            self.emit(Opcode::PushStr);
            self.emit_u16(pact_idx);
            self.emit(Opcode::AllowWithPact);
        } else {
            self.emit(Opcode::Allow);
        }

        // Patch all jump addresses to point to after the Allow
        let next_rule_addr = self.code.len();
        for pos in jump_positions {
            // JumpIfNot opcode is at `pos`, address is at `pos + 1` (2 bytes)
            let addr_pos = pos + 1;
            self.code[addr_pos] = ((next_rule_addr >> 8) & 0xFF) as u8;
            self.code[addr_pos + 1] = (next_rule_addr & 0xFF) as u8;
        }
    }

    fn compile_constraint(&mut self, constraint: &Constraint) {
        match constraint {
            Constraint::IntentTypeEquals { value } => {
                // LoadIntent("type"), PushStr(value), StrEq
                let type_idx = self.add_constant("type");
                let value_idx = self.add_constant(value);
                
                self.emit(Opcode::LoadIntent);
                self.emit_u16(type_idx);
                self.emit(Opcode::PushStr);
                self.emit_u16(value_idx);
                self.emit(Opcode::StrEq);
            }
            
            Constraint::AmountMax { max } => {
                // LoadIntent("amount"), PushI64(max), Le
                let amount_idx = self.add_constant("amount");
                
                self.emit(Opcode::LoadIntent);
                self.emit_u16(amount_idx);
                self.emit_push_i64(*max);
                self.emit(Opcode::Le);
            }
            
            Constraint::AmountMin { min } => {
                // LoadIntent("amount"), PushI64(min), Ge
                let amount_idx = self.add_constant("amount");
                
                self.emit(Opcode::LoadIntent);
                self.emit_u16(amount_idx);
                self.emit_push_i64(*min);
                self.emit(Opcode::Ge);
            }
            
            Constraint::ContainerPrefix { prefix } => {
                // LoadContainerId, PushStr(prefix), StrStartsWith
                let prefix_idx = self.add_constant(prefix);
                
                self.emit(Opcode::LoadContainerId);
                self.emit(Opcode::PushStr);
                self.emit_u16(prefix_idx);
                self.emit(Opcode::StrStartsWith);
            }
            
            Constraint::ActorEquals { actor } => {
                // LoadActor, PushStr(actor), StrEq
                let actor_idx = self.add_constant(actor);
                
                self.emit(Opcode::LoadActor);
                self.emit(Opcode::PushStr);
                self.emit_u16(actor_idx);
                self.emit(Opcode::StrEq);
            }
            
            Constraint::FieldEquals { field, value } => {
                // LoadIntent(field), PushStr(value), StrEq
                let field_idx = self.add_constant(field);
                let value_idx = self.add_constant(value);
                
                self.emit(Opcode::LoadIntent);
                self.emit_u16(field_idx);
                self.emit(Opcode::PushStr);
                self.emit_u16(value_idx);
                self.emit(Opcode::StrEq);
            }
        }
    }

    fn emit(&mut self, op: Opcode) {
        self.code.push(op as u8);
    }

    fn emit_u16(&mut self, value: u16) {
        self.code.push((value >> 8) as u8);
        self.code.push((value & 0xFF) as u8);
    }

    fn emit_push_i64(&mut self, value: i64) {
        self.code.push(Opcode::PushI64 as u8);
        self.code.extend_from_slice(&value.to_be_bytes());
    }

    fn emit_deny(&mut self, reason: &str) {
        let idx = self.add_constant(reason);
        self.emit(Opcode::PushStr);
        self.emit_u16(idx);
        self.emit(Opcode::Deny);
    }

    fn add_constant(&mut self, s: &str) -> u16 {
        // Check if already exists
        if let Some(idx) = self.constants.iter().position(|c| c == s) {
            return idx as u16;
        }
        let idx = self.constants.len() as u16;
        self.constants.push(s.to_string());
        idx
    }
}

impl Default for PolicyCompiler {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a default policy for a container
pub fn create_default_policy(container_id: &str) -> PolicyDefinition {
    PolicyDefinition {
        policy_id: format!("default_{}", container_id),
        version: "1.0".to_string(),
        description: format!("Default policy for {}", container_id),
        rules: vec![
            // Allow observations (read-only)
            PolicyRule {
                rule_id: "allow_observation".to_string(),
                applies_to: AppliesTo::Container { id: container_id.to_string() },
                intent_class: IntentClassSpec::Observation,
                constraints: vec![
                    Constraint::IntentTypeEquals { value: "observe".to_string() },
                ],
                required_pact: None,
            },
            // Allow small transfers (Conservation)
            PolicyRule {
                rule_id: "allow_small_transfer".to_string(),
                applies_to: AppliesTo::Container { id: container_id.to_string() },
                intent_class: IntentClassSpec::Conservation,
                constraints: vec![
                    Constraint::IntentTypeEquals { value: "transfer".to_string() },
                    Constraint::AmountMax { max: 10000 },
                ],
                required_pact: None,
            },
            // Allow large transfers with pact
            PolicyRule {
                rule_id: "allow_large_transfer".to_string(),
                applies_to: AppliesTo::Container { id: container_id.to_string() },
                intent_class: IntentClassSpec::Conservation,
                constraints: vec![
                    Constraint::IntentTypeEquals { value: "transfer".to_string() },
                    Constraint::AmountMin { min: 10001 },
                ],
                required_pact: Some("high_value_transfer".to_string()),
            },
            // Evolution requires L5 pact
            PolicyRule {
                rule_id: "evolution_requires_pact".to_string(),
                applies_to: AppliesTo::Container { id: container_id.to_string() },
                intent_class: IntentClassSpec::Evolution,
                constraints: vec![
                    Constraint::IntentTypeEquals { value: "evolve".to_string() },
                ],
                required_pact: Some("evolution_l5".to_string()),
            },
        ],
        default_deny: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bytecode::{BytecodeVM, ExecutionContext};

    #[test]
    fn test_compile_and_execute() {
        let policy_def = PolicyDefinition {
            policy_id: "test".to_string(),
            version: "1.0".to_string(),
            description: "Test policy".to_string(),
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

        let mut compiler = PolicyCompiler::new();
        let compiled = compiler.compile(&policy_def);

        let vm = BytecodeVM::default();
        
        // Matching intent - should allow
        let ctx = ExecutionContext {
            container_id: "C.Test".to_string(),
            actor: "alice".to_string(),
            intent: serde_json::json!({"type": "observe"}),
            state: None,
            timestamp: 1000,
        };
        
        let result = vm.execute(&compiled, &ctx).unwrap();
        assert!(matches!(result, crate::bytecode::PolicyResult::Allow { intent_class: 0, .. }));

        // Non-matching intent - should deny
        let ctx = ExecutionContext {
            container_id: "C.Test".to_string(),
            actor: "alice".to_string(),
            intent: serde_json::json!({"type": "hack"}),
            state: None,
            timestamp: 1000,
        };
        
        let result = vm.execute(&compiled, &ctx).unwrap();
        assert!(matches!(result, crate::bytecode::PolicyResult::Deny { .. }));
    }

    #[test]
    fn test_amount_constraint() {
        let policy_def = PolicyDefinition {
            policy_id: "test".to_string(),
            version: "1.0".to_string(),
            description: "Test policy".to_string(),
            rules: vec![
                PolicyRule {
                    rule_id: "small_transfer".to_string(),
                    applies_to: AppliesTo::Global,
                    intent_class: IntentClassSpec::Conservation,
                    constraints: vec![
                        Constraint::IntentTypeEquals { value: "transfer".to_string() },
                        Constraint::AmountMax { max: 1000 },
                    ],
                    required_pact: None,
                },
                PolicyRule {
                    rule_id: "large_transfer".to_string(),
                    applies_to: AppliesTo::Global,
                    intent_class: IntentClassSpec::Conservation,
                    constraints: vec![
                        Constraint::IntentTypeEquals { value: "transfer".to_string() },
                        Constraint::AmountMin { min: 1001 },
                    ],
                    required_pact: Some("approval".to_string()),
                },
            ],
            default_deny: true,
        };

        let mut compiler = PolicyCompiler::new();
        let compiled = compiler.compile(&policy_def);

        let vm = BytecodeVM::default();
        
        // Small transfer - allow without pact
        let ctx = ExecutionContext {
            container_id: "C.Test".to_string(),
            actor: "alice".to_string(),
            intent: serde_json::json!({"type": "transfer", "amount": 500}),
            state: None,
            timestamp: 1000,
        };
        
        let result = vm.execute(&compiled, &ctx).unwrap();
        match result {
            crate::bytecode::PolicyResult::Allow { intent_class, required_pact, .. } => {
                assert_eq!(intent_class, 1); // Conservation
                assert!(required_pact.is_none());
            }
            _ => panic!("Expected Allow"),
        }

        // Large transfer - require pact
        let ctx = ExecutionContext {
            container_id: "C.Test".to_string(),
            actor: "alice".to_string(),
            intent: serde_json::json!({"type": "transfer", "amount": 5000}),
            state: None,
            timestamp: 1000,
        };
        
        let result = vm.execute(&compiled, &ctx).unwrap();
        match result {
            crate::bytecode::PolicyResult::Allow { intent_class, required_pact, .. } => {
                assert_eq!(intent_class, 1); // Conservation
                assert_eq!(required_pact, Some("approval".to_string()));
            }
            _ => panic!("Expected Allow with pact"),
        }
    }
}

