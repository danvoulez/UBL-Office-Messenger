//! Bytecode VM execution tests

use ubl_policy_vm: :{
    compiler::{PolicyCompiler, PolicyDefinition, PolicyRule, AppliesTo, IntentClassSpec, Constraint},
    bytecode: :{BytecodeVM, ExecutionContext, PolicyResult},
};
use serde_json::json;

#[test]
fn test_simple_allow_policy() {
    let policy = PolicyDefinition {
        policy_id: "test_allow".to_string(),
        version: "1.0".to_string(),
        description: "Allow all observations".to_string(),
        rules: vec![
            PolicyRule {
                rule_id: "allow_observe".to_string(),
                applies_to: AppliesTo:: Global,
                intent_class: IntentClassSpec:: Observation,
                constraints: vec![],
                required_pact:  None,
            }
        ],
        default_deny: false,
    };
    
    let mut compiler = PolicyCompiler::new();
    let compiled = compiler.compile(&policy);
    
    let vm = BytecodeVM::default();
    let ctx = ExecutionContext {
        container_id: "C.Test". to_string(),
        actor:  "alice".to_string(),
        intent:  json!({"type": "observe"}),
        state: None,
        timestamp: 1000,
    };
    
    let result = vm.execute(&compiled, &ctx).unwrap();
    assert!(matches!(result, PolicyResult::Allow { .. }));
}

#[test]
fn test_simple_deny_policy() {
    let policy = PolicyDefinition {
        policy_id: "test_deny".to_string(),
        version: "1.0".to_string(),
        description: "Deny by default".to_string(),
        rules: vec![],
        default_deny: true,
    };
    
    let mut compiler = PolicyCompiler::new();
    let compiled = compiler.compile(&policy);
    
    let vm = BytecodeVM::default();
    let ctx = ExecutionContext {
        container_id: "C.Test".to_string(),
        actor: "alice".to_string(),
        intent: json!({"type": "anything"}),
        state: None,
        timestamp: 1000,
    };
    
    let result = vm.execute(&compiled, &ctx).unwrap();
    assert!(matches!(result, PolicyResult::Deny { .. }));
}

#[test]
fn test_constraint_intent_type() {
    let policy = PolicyDefinition {
        policy_id: "test_constraint".to_string(),
        version: "1.0".to_string(),
        description: "Allow only specific type". to_string(),
        rules: vec![
            PolicyRule {
                rule_id: "allow_create".to_string(),
                applies_to: AppliesTo:: Global,
                intent_class:  IntentClassSpec:: Observation,
                constraints: vec![
                    Constraint:: IntentTypeEquals { value: "create".to_string() }
                ],
                required_pact: None,
            }
        ],
        default_deny:  true,
    };
    
    let mut compiler = PolicyCompiler:: new();
    let compiled = compiler.compile(&policy);
    let vm = BytecodeVM::default();
    
    // Should allow "create"
    let ctx_allow = ExecutionContext {
        container_id: "C.Test". to_string(),
        actor: "alice".to_string(),
        intent: json!({"type":  "create"}),
        state: None,
        timestamp: 1000,
    };
    let result = vm.execute(&compiled, &ctx_allow).unwrap();
    assert!(matches!(result, PolicyResult::Allow { .. }));
    
    // Should deny "delete"
    let ctx_deny = ExecutionContext {
        container_id: "C.Test".to_string(),
        actor: "alice".to_string(),
        intent: json!({"type": "delete"}),
        state: None,
        timestamp: 1000,
    };
    let result = vm.execute(&compiled, &ctx_deny).unwrap();
    assert!(matches!(result, PolicyResult::Deny { .. }));
}

#[test]
fn test_amount_constraint() {
    let policy = PolicyDefinition {
        policy_id: "test_amount". to_string(),
        version: "1.0".to_string(),
        description: "Limit transfer amounts".to_string(),
        rules: vec![
            PolicyRule {
                rule_id: "small_transfer".to_string(),
                applies_to: AppliesTo::Global,
                intent_class: IntentClassSpec::Conservation,
                constraints: vec![
                    Constraint::AmountMax { max:  1000 }
                ],
                required_pact: None,
            }
        ],
        default_deny: true,
    };
    
    let mut compiler = PolicyCompiler::new();
    let compiled = compiler.compile(&policy);
    let vm = BytecodeVM::default();
    
    // Should allow amount <= 1000
    let ctx_allow = ExecutionContext {
        container_id: "C.Test".to_string(),
        actor: "alice".to_string(),
        intent: json!({"type": "transfer", "amount": 500}),
        state: None,
        timestamp: 1000,
    };
    let result = vm.execute(&compiled, &ctx_allow).unwrap();
    assert!(matches!(result, PolicyResult::Allow { .. }));
    
    // Should deny amount > 1000
    let ctx_deny = ExecutionContext {
        container_id: "C.Test".to_string(),
        actor: "alice".to_string(),
        intent: json!({"type": "transfer", "amount": 2000}),
        state: None,
        timestamp: 1000,
    };
    let result = vm.execute(&compiled, &ctx_deny).unwrap();
    assert!(matches!(result, PolicyResult::Deny { .. }));
}

#[test]
fn test_pact_requirement() {
    let policy = PolicyDefinition {
        policy_id: "test_pact".to_string(),
        version: "1.0".to_string(),
        description: "Require pact for evolution".to_string(),
        rules: vec![
            PolicyRule {
                rule_id: "evolution_pact".to_string(),
                applies_to: AppliesTo::Global,
                intent_class: IntentClassSpec::Evolution,
                constraints: vec![],
                required_pact: Some("evolution_l5".to_string()),
            }
        ],
        default_deny: true,
    };
    
    let mut compiler = PolicyCompiler::new();
    let compiled = compiler.compile(&policy);
    let vm = BytecodeVM::default();
    
    let ctx = ExecutionContext {
        container_id: "C.Test".to_string(),
        actor: "alice".to_string(),
        intent: json!({"type": "evolve"}),
        state: None,
        timestamp: 1000,
    };
    
    let result = vm.execute(&compiled, &ctx).unwrap();
    assert!(matches!(result, PolicyResult::RequirePact { .. }));
}

#[test]
fn test_gas_limit() {
    // Create policy with many rules to potentially exceed gas
    let mut rules = Vec::new();
    for i in 0..100 {
        rules.push(PolicyRule {
            rule_id: format!("rule_{}", i),
            applies_to: AppliesTo::Global,
            intent_class: IntentClassSpec:: Observation,
            constraints: vec![
                Constraint::IntentTypeEquals { value: format! ("type_{}", i) }
            ],
            required_pact: None,
        });
    }
    
    let policy = PolicyDefinition {
        policy_id:  "test_gas".to_string(),
        version: "1.0".to_string(),
        description: "Many rules". to_string(),
        rules,
        default_deny: true,
    };
    
    let mut compiler = PolicyCompiler::new();
    let compiled = compiler. compile(&policy);
    
    let vm = BytecodeVM::default();
    let ctx = ExecutionContext {
        container_id: "C.Test".to_string(),
        actor: "alice".to_string(),
        intent: json!({"type": "observe"}),
        state: None,
        timestamp: 1000,
    };
    
    // Should execute without exhausting gas
    let result = vm.execute(&compiled, &ctx);
    assert!(result.is_ok());
}

#[test]
fn test_container_scoping() {
    let policy = PolicyDefinition {
        policy_id: "test_scope".to_string(),
        version: "1.0".to_string(),
        description: "Container-specific policy".to_string(),
        rules: vec![
            PolicyRule {
                rule_id: "jobs_only".to_string(),
                applies_to: AppliesTo::Container { id: "C.Jobs".to_string() },
                intent_class: IntentClassSpec::Observation,
                constraints:  vec![],
                required_pact: None,
            }
        ],
        default_deny:  true,
    };
    
    let mut compiler = PolicyCompiler::new();
    let compiled = compiler.compile(&policy);
    let vm = BytecodeVM:: default();
    
    // Should allow for C.Jobs
    let ctx_jobs = ExecutionContext {
        container_id: "C.Jobs". to_string(),
        actor: "alice".to_string(),
        intent: json!({"type":  "observe"}),
        state: None,
        timestamp: 1000,
    };
    let result = vm.execute(&compiled, &ctx_jobs).unwrap();
    assert!(matches!(result, PolicyResult::Allow { .. }));
    
    // Should deny for C. Messenger
    let ctx_messenger = ExecutionContext {
        container_id: "C.Messenger".to_string(),
        actor: "alice".to_string(),
        intent: json!({"type": "observe"}),
        state: None,
        timestamp: 1000,
    };
    let result = vm.execute(&compiled, &ctx_messenger).unwrap();
    assert!(matches!(result, PolicyResult::Deny { .. }));
}