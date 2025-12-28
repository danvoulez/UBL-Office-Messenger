# 🚨 CRITICAL: UBL Policy Engine Missing

**Date**: 2024-12-27  
**Priority**: 🔴 **P0 - BLOCKING**  
**Impact**: **Security, Governance, UBL Compliance**

---

## 🎯 Executive Summary

**UBL has no policy engine integration.** The membrane validates basic physics rules but does **NOT** evaluate TDLN (Deterministic Translation of Language to Notation) policies before accepting commits. This is a **critical architectural gap** that violates SPEC-UBL-POLICY v1.0.

---

## 🔴 The Problem

### Current State

1. **Policy VM Exists But Is Stub**
   - `ubl-policy-vm` crate exists with placeholder implementation
   - `PolicyVM::evaluate()` has simple rule-based logic, not WASM execution
   - No integration with membrane or commit flow

2. **Membrane Doesn't Call Policy Engine**
   - Membrane validates basic physics (delta=0 for Observation, balance>=0 for Conservation)
   - **Evolution intents are allowed without policy checks** (comment says "would need additional policy checks")
   - No TDLN evaluation before commit

3. **C.Policy Container Is Placeholder**
   - Container structure exists but no actual policy evaluation service
   - No policy registration/management
   - No policy query endpoints

4. **No Intent Class Validation**
   - Clients can commit **any** `intent_class` without policy validation
   - No enforcement of "which intent_classes are allowed for this container/actor"
   - Security risk: malicious clients can commit Evolution intents

---

## 📋 What SPEC-UBL-POLICY v1.0 Requires

### TDLN (Deterministic Translation of Language to Notation)

**Purpose**: TDLN governs **how local intents can be translated into verifiable facts**.

**Flow**:
```
[draft intent + context] --eval(TDLN/WASM)--> Allow/Deny
Allow{intent_class, required_pact} --boundary--> [ubl-link] --membrane--> [ledger]
```

**Key Requirements**:

1. **Policy Evaluation Before Commit**
   - Given: `(Intent, Context)` → TDLN evaluates → `{ AllowedTranslation }`
   - TDLN answers: **"Can this intent become an atom?"**
   - TDLN determines: **"Which intent_class is allowed?"**

2. **Policy Rules**
   ```rust
   Rule := {
     rule_id: String,
     applies_to: String,      // container, namespace, type
     intent_class: u8,        // Allowed class
     constraints: Vec<Constraint>,
     required_pact: Option<String>,
   }
   ```

3. **Deterministic Execution**
   - Policies MUST be compiled to WASM bytecode
   - Execution MUST be deterministic
   - Execution MUST be semantically blind (no side effects)

4. **Evolution Requires Policy**
   - `intent_class == Evolution` **MUST** have policy evaluation
   - Evolution intents change the physics rules themselves
   - Without policy, Evolution is a security hole

---

## 🔍 Current Code Analysis

### 1. Membrane Validation (Incomplete)

**File**: `ubl/kernel/rust/ubl-membrane/src/lib.rs:130-156`

```rust
// V6 - Physics invariants
match link.intent_class {
    IntentClass::Observation => {
        if link.physics_delta != 0 {
            return Err(MembraneError::PhysicsViolation { ... });
        }
    }
    IntentClass::Conservation => {
        let resulting_balance = state.physical_balance + link.physics_delta;
        if resulting_balance < 0 {
            return Err(MembraneError::PhysicsViolation { ... });
        }
    }
    IntentClass::Entropy => {
        // Entropy allows creation/destruction - no additional checks
    }
    IntentClass::Evolution => {
        // Evolution is for rule changes - would need additional policy checks
        // For now, allow it  ❌ THIS IS THE PROBLEM
    }
}
```

**Problem**: Evolution is allowed without policy checks!

---

### 2. Policy VM (Stub Implementation)

**File**: `ubl/kernel/rust/ubl-policy-vm/src/lib.rs:136-158`

```rust
pub fn evaluate(
    &self,
    policy_id: &str,
    context: &EvaluationContext,
) -> Result<TranslationDecision> {
    let _policy = self
        .policies
        .get(policy_id)
        .ok_or_else(|| PolicyError::PolicyNotFound(policy_id.to_string()))?;

    // Simple rule-based evaluation
    // In production, this would execute WASM  ❌ NOT IMPLEMENTED
    
    // Extract intent type from context
    let intent_type = context
        .intent
        .get("type")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    // Simple rules based on intent type
    match intent_type {
        "observe" | "read" => // ... basic rules
        // ...
    }
}
```

**Problem**: 
- No WASM execution
- No policy loading from C.Policy container
- Simple hardcoded rules, not TDLN-compliant

---

### 3. Commit Flow (No Policy Call)

**File**: `ubl/kernel/rust/ubl-server/src/main.rs:144-190`

```rust
async fn route_commit(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(link): Json<LinkDraft>,
) -> Result<Json<CommitSuccess>, (StatusCode, String)> {
    // ASC Validation (PR29) ✅
    // ... auth checks ...
    
    // ❌ NO POLICY EVALUATION HERE
    
    match state.ledger.append(&link).await {
        // ...
    }
}
```

**Problem**: Commit flow doesn't call policy engine before membrane validation.

---

## 🎯 What Needs to Be Built

### Phase 1: Policy Engine Integration (CRITICAL)

#### 1.1 Integrate Policy VM into Commit Flow

**Location**: `ubl/kernel/rust/ubl-server/src/main.rs`

**Required Changes**:
```rust
async fn route_commit(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(link): Json<LinkDraft>,
) -> Result<Json<CommitSuccess>, (StatusCode, String)> {
    // 1. ASC Validation (existing)
    // ...
    
    // 2. POLICY EVALUATION (NEW)
    let policy_context = EvaluationContext {
        container_id: link.container_id.clone(),
        actor: asc_context.actor_id.clone(),
        intent: link.intent.clone(),  // Extract from atom
        state: Some(state.ledger.get_state(&link.container_id).await?),
        timestamp: Utc::now().timestamp(),
    };
    
    // Get policy ID for container
    let policy_id = state.policy_registry.get_policy_for_container(&link.container_id)?;
    
    // Evaluate policy
    let decision = state.policy_vm.evaluate(&policy_id, &policy_context).await?;
    
    match decision {
        TranslationDecision::Allow { intent_class, required_pact, .. } => {
            // Verify intent_class matches link
            if intent_class != link.intent_class as u8 {
                return Err((StatusCode::FORBIDDEN, "Policy does not allow this intent_class".to_string()));
            }
            
            // Verify pact if required
            if required_pact.is_some() && link.pact.is_none() {
                return Err((StatusCode::FORBIDDEN, "Policy requires pact".to_string()));
            }
        }
        TranslationDecision::Deny { reason } => {
            return Err((StatusCode::FORBIDDEN, format!("Policy denied: {}", reason)));
        }
    }
    
    // 3. Membrane Validation (existing)
    // ...
}
```

---

#### 1.2 Implement WASM Policy Execution

**Location**: `ubl/kernel/rust/ubl-policy-vm/src/lib.rs`

**Required Changes**:
```rust
use wasmtime::{Engine, Module, Store, Instance, Func};

pub struct PolicyVM {
    engine: Engine,
    policies: HashMap<String, Policy>,
    wasm_cache: HashMap<String, Module>,  // Cache compiled WASM
}

impl PolicyVM {
    pub fn new() -> Self {
        let engine = Engine::default();
        Self {
            engine,
            policies: HashMap::new(),
            wasm_cache: HashMap::new(),
        }
    }
    
    pub async fn evaluate(
        &mut self,
        policy_id: &str,
        context: &EvaluationContext,
    ) -> Result<TranslationDecision> {
        let policy = self.policies.get(policy_id)
            .ok_or_else(|| PolicyError::PolicyNotFound(policy_id.to_string()))?;
        
        // Load WASM module (from cache or compile)
        let module = self.load_module(&policy.bytecode_hash, &policy.bytecode)?;
        
        // Create WASM store
        let mut store = Store::new(&self.engine, ());
        
        // Instantiate module
        let instance = Instance::new(&mut store, &module, &[])?;
        
        // Call evaluate function
        let evaluate_fn = instance.get_typed_func::<(i32, i32), i32>(&mut store, "evaluate")?;
        
        // Serialize context to JSON
        let context_json = serde_json::to_string(context)?;
        let context_bytes = context_json.as_bytes();
        
        // Allocate memory in WASM
        let memory = instance.get_memory(&mut store, "memory")
            .ok_or_else(|| PolicyError::ExecutionFailed("No memory export".to_string()))?;
        
        // Write context to WASM memory
        let ptr = allocate_memory(&mut store, &memory, context_bytes.len())?;
        memory.write(&mut store, ptr as usize, context_bytes)?;
        
        // Call evaluate
        let result_ptr = evaluate_fn.call(&mut store, (ptr, context_bytes.len() as i32))?;
        
        // Read result from WASM memory
        let result_bytes = read_memory(&mut store, &memory, result_ptr as usize)?;
        let decision: TranslationDecision = serde_json::from_slice(&result_bytes)?;
        
        Ok(decision)
    }
}
```

---

#### 1.3 Policy Registry Service

**Location**: `ubl/kernel/rust/ubl-server/src/policy_registry.rs` (NEW FILE)

**Required**:
```rust
pub struct PolicyRegistry {
    // Map container_id -> policy_id
    container_policies: HashMap<String, String>,
    // Map policy_id -> Policy metadata
    policies: HashMap<String, PolicyMetadata>,
}

impl PolicyRegistry {
    pub fn get_policy_for_container(&self, container_id: &str) -> Result<String> {
        self.container_policies.get(container_id)
            .cloned()
            .ok_or_else(|| PolicyError::PolicyNotFound(format!("No policy for container {}", container_id)))
    }
    
    pub async fn register_policy(&mut self, container_id: String, policy: Policy) -> Result<()> {
        // Validate policy bytecode
        // Store policy
        // Update container mapping
        Ok(())
    }
}
```

---

#### 1.4 Update Membrane to Require Policy for Evolution

**Location**: `ubl/kernel/rust/ubl-membrane/src/lib.rs`

**Required Changes**:
```rust
pub fn validate(
    link: &LinkCommit,
    state: &LedgerState,
    policy_decision: Option<&TranslationDecision>,  // NEW PARAMETER
) -> Result<()> {
    // ... existing validations ...
    
    // V6 - Physics invariants
    match link.intent_class {
        IntentClass::Evolution => {
            // Evolution REQUIRES policy evaluation
            let decision = policy_decision.ok_or_else(|| {
                MembraneError::UnauthorizedEvolution("Policy evaluation required for Evolution".to_string())
            })?;
            
            match decision {
                TranslationDecision::Allow { required_pact, .. } => {
                    // Verify pact is present
                    if required_pact.is_some() && link.pact.is_none() {
                        return Err(MembraneError::PactViolation);
                    }
                }
                TranslationDecision::Deny { reason } => {
                    return Err(MembraneError::UnauthorizedEvolution(reason.clone()));
                }
            }
        }
        // ... other cases ...
    }
    
    Ok(())
}
```

---

### Phase 2: C.Policy Container Implementation

#### 2.1 Policy Management API

**Endpoints Needed**:
- `POST /policies` - Register a new policy
- `GET /policies/:id` - Get policy metadata
- `GET /containers/:id/policy` - Get policy for container
- `PUT /containers/:id/policy` - Update container policy

#### 2.2 Policy Storage

- Store policies in C.Policy container (as events)
- Policies are immutable (new version = new policy_id)
- Policy bytecode stored separately (object storage or database)

---

## 🚨 Security Implications

### Without Policy Engine:

1. **Evolution Intents Unrestricted**
   - Anyone can commit Evolution intents
   - Can change physics rules arbitrarily
   - **CRITICAL SECURITY HOLE**

2. **No Intent Class Validation**
   - Clients can use wrong intent_class
   - No enforcement of business rules
   - Can bypass conservation checks

3. **No Container-Specific Rules**
   - All containers treated the same
   - Can't enforce "C.Jobs only allows Observation"
   - Can't enforce "C.Pacts requires quorum"

4. **No Governance**
   - No way to enforce organizational policies
   - No audit trail of policy decisions
   - No way to revoke permissions

---

## 📊 Impact Assessment

| Aspect | Impact | Severity |
|--------|--------|----------|
| **Security** | Evolution intents unrestricted | 🔴 CRITICAL |
| **Compliance** | Violates SPEC-UBL-POLICY v1.0 | 🔴 CRITICAL |
| **Governance** | No policy enforcement | 🟡 HIGH |
| **Architecture** | Incomplete UBL implementation | 🟡 HIGH |
| **Production** | Not production-ready | 🔴 CRITICAL |

---

## 🎯 Recommended Action Plan

### **Immediate (Before Phase 2)**:

1. ✅ **Block Evolution Intents** (temporary fix)
   - Update membrane to reject Evolution without policy
   - Add TODO comment pointing to policy engine

2. ✅ **Add Policy Evaluation Stub**
   - Create `PolicyVM` integration point
   - Return `Deny` for Evolution, `Allow` for others (temporary)

### **Phase 1 (Critical Path)**:

3. ✅ **Implement WASM Policy Execution**
   - Integrate `wasmtime` crate
   - Implement policy VM with WASM execution
   - Add policy registry

4. ✅ **Integrate into Commit Flow**
   - Call policy VM before membrane
   - Pass policy decision to membrane
   - Update membrane to require policy for Evolution

5. ✅ **Policy Management API**
   - Implement C.Policy container endpoints
   - Policy registration/query
   - Container-policy mapping

### **Phase 2 (Production)**:

6. ✅ **Policy Compilation Tool**
   - Tool to compile policy rules to WASM
   - Policy validation
   - Policy versioning

7. ✅ **Policy Testing Framework**
   - Unit tests for policies
   - Integration tests for policy evaluation
   - Policy performance benchmarks

---

## 📝 Code Locations

### Existing (Incomplete):
- `ubl/kernel/rust/ubl-policy-vm/src/lib.rs` - Policy VM stub
- `ubl/kernel/rust/ubl-membrane/src/lib.rs` - Membrane (no policy call)
- `ubl/containers/C.Policy/` - Container structure (placeholder)

### Need to Create:
- `ubl/kernel/rust/ubl-server/src/policy_registry.rs` - Policy registry
- `ubl/kernel/rust/ubl-server/src/policy_service.rs` - Policy service
- `ubl/containers/C.Policy/boundary/policy_api.rs` - Policy API endpoints
- `ubl/tools/policy-compiler/` - Policy compilation tool

---

## 🔗 Related Specifications

- **SPEC-UBL-POLICY v1.0** - TDLN definition
- **SPEC-UBL-MEMBRANE v1.0** - Membrane validation (should call policy)
- **SPEC-UBL-CORE v1.0** - Core UBL architecture
- **SPEC-UBL-PACT v1.0** - Pact validation (used by policy)

---

## ✅ Definition of Done

Policy engine is complete when:

1. ✅ Policy VM executes WASM bytecode deterministically
2. ✅ Commit flow calls policy VM before membrane
3. ✅ Membrane requires policy decision for Evolution intents
4. ✅ C.Policy container provides policy management API
5. ✅ Policies can be registered and queried
6. ✅ Policy evaluation is < 10ms (performance target)
7. ✅ All Evolution intents are rejected without valid policy
8. ✅ Policy decisions are logged/auditable

---

**Status**: 🔴 **BLOCKING** - UBL is not production-ready without policy engine.

**Priority**: **P0 - Must fix before production deployment.**

