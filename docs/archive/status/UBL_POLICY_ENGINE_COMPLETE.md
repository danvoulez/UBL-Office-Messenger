# 📋 UBL Policy Engine - Complete

**SPEC-UBL-POLICY v1.0 Compliant**

---

## What We Built

### 1. Bytecode VM (`ubl-policy-vm/src/bytecode.rs`)

A deterministic, stack-based VM for policy execution:

```rust
// Opcodes for policy execution
enum Opcode {
    PushI64, PushStr, PushTrue, PushFalse,  // Stack ops
    LoadContext, LoadIntent, LoadTimestamp, // Context access
    Eq, Ne, Lt, Le, Gt, Ge,                 // Comparison
    Add, Sub, Mul, Div,                     // Arithmetic
    And, Or, Not,                           // Logic
    Jump, JumpIf, JumpIfNot,                // Control flow
    StrContains, StrStartsWith, StrEq,      // String ops
    Allow, AllowWithPact, Deny,             // Results
}
```

**Properties:**
- ✅ Deterministic (no randomness, no I/O)
- ✅ Gas-limited (prevents infinite loops)
- ✅ Type-safe (stack operations)
- ✅ BLAKE3 hash verification

### 2. Policy Compiler (`ubl-policy-vm/src/compiler.rs`)

Compiles policy rules (JSON) to bytecode:

```rust
let policy = PolicyDefinition {
    policy_id: "transfer_policy".to_string(),
    version: "1.0".to_string(),
    rules: vec![
        PolicyRule {
            rule_id: "small_transfer".to_string(),
            intent_class: IntentClassSpec::Conservation,
            constraints: vec![
                Constraint::IntentTypeEquals { value: "transfer".to_string() },
                Constraint::AmountMax { max: 10000 },
            ],
            required_pact: None,
        },
        PolicyRule {
            rule_id: "large_transfer".to_string(),
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

let mut compiler = PolicyCompiler::new();
let compiled = compiler.compile(&policy);
```

**Constraints Supported:**
- `IntentTypeEquals` - Check intent type
- `AmountMax` / `AmountMin` - Amount limits
- `ContainerPrefix` - Container namespace
- `ActorEquals` - Specific actor
- `FieldEquals` - Custom field check

### 3. Policy Registry (`ubl-server/src/policy_registry.rs`)

Maps containers to policies:

```rust
let registry = PolicyRegistry::with_pool(pool);
registry.init_defaults().await;

// Evaluate before commit
let decision = registry.evaluate(
    "C.Jobs",           // container
    "alice",            // actor
    &intent_json,       // intent
    None,               // state
    timestamp,
).await?;

match decision {
    TranslationDecision::Allow { intent_class, required_pact, .. } => {
        // Proceed with commit
    }
    TranslationDecision::Deny { reason } => {
        // Reject commit
    }
}
```

### 4. Server Integration (`ubl-server/src/main.rs`)

Policy evaluation in commit flow:

```
                                   ┌─────────────────┐
                                   │  POST /commit   │
                                   └────────┬────────┘
                                            │
                          ┌─────────────────▼─────────────────┐
                          │     ASC Validation (Auth)         │
                          └─────────────────┬─────────────────┘
                                            │
NEW! ──────────────────── ┌─────────────────▼─────────────────┐
                          │  POLICY EVALUATION (TDLN)         │
                          │  - Load policy for container      │
                          │  - Evaluate constraints           │
                          │  - Check intent class             │
                          │  - Check pact requirement         │
                          └─────────────────┬─────────────────┘
                                            │
                          ┌─────────────────▼─────────────────┐
                          │     PACT VALIDATION               │
                          └─────────────────┬─────────────────┘
                                            │
                          ┌─────────────────▼─────────────────┐
                          │     MEMBRANE VALIDATION           │
                          └─────────────────┬─────────────────┘
                                            │
                          ┌─────────────────▼─────────────────┐
                          │     LEDGER APPEND                 │
                          └───────────────────────────────────┘
```

### 5. SQL Storage (`sql/008_policy_engine.sql`)

```sql
-- Policy definitions
CREATE TABLE policy_definitions (
    policy_id TEXT PRIMARY KEY,
    version TEXT NOT NULL,
    description TEXT NOT NULL,
    rules JSONB NOT NULL,
    default_deny BOOLEAN NOT NULL,
    bytecode_hash TEXT
);

-- Container mappings
CREATE TABLE container_policies (
    container_id TEXT PRIMARY KEY,
    policy_id TEXT NOT NULL REFERENCES policy_definitions(policy_id)
);

-- Audit log
CREATE TABLE policy_evaluations (
    evaluation_id TEXT NOT NULL UNIQUE,
    container_id TEXT NOT NULL,
    policy_id TEXT NOT NULL,
    actor TEXT NOT NULL,
    decision TEXT NOT NULL,  -- 'allow' or 'deny'
    intent_class SMALLINT,
    required_pact TEXT,
    deny_reason TEXT
);
```

---

## Files Added

| File | Purpose |
|------|---------|
| `ubl-policy-vm/src/bytecode.rs` | Bytecode VM implementation |
| `ubl-policy-vm/src/compiler.rs` | Policy rule compiler |
| `ubl-server/src/policy_registry.rs` | Container → Policy mapping |
| `sql/008_policy_engine.sql` | Database schema |

## Files Modified

| File | Change |
|------|--------|
| `ubl-policy-vm/src/lib.rs` | Export new modules, integrate VM |
| `ubl-policy-vm/Cargo.toml` | Add hex dependency |
| `ubl-server/src/main.rs` | Integrate policy evaluation |
| `ubl-server/Cargo.toml` | Add ubl-policy-vm dependency |

---

## How It Works

### 1. Policy Definition (JSON)
```json
{
  "policy_id": "default_C.Jobs",
  "version": "1.0",
  "rules": [
    {
      "rule_id": "allow_observe",
      "intent_class": "observation",
      "constraints": [
        { "type": "intent_type_equals", "value": "observe" }
      ]
    }
  ],
  "default_deny": true
}
```

### 2. Compilation to Bytecode
```
[LoadIntent("type"), PushStr("observe"), StrEq, JumpIfNot(next), 
 PushI64(0), Allow, PushStr("No matching rule"), Deny]
```

### 3. Execution
```
Stack: []
→ LoadIntent("type") → Stack: ["observe"]
→ PushStr("observe") → Stack: ["observe", "observe"]
→ StrEq             → Stack: [true]
→ JumpIfNot(skip)   → Stack: [] (condition passed)
→ PushI64(0)        → Stack: [0]
→ Allow             → Result: Allow { intent_class: 0 }
```

---

## Security Improvements

### Before (Critical Gaps):
- ❌ Evolution intents unrestricted (only membrane required pact)
- ❌ No intent class validation
- ❌ No container-specific rules
- ❌ No policy evaluation before commit

### After:
- ✅ **All commits evaluated** against container policy
- ✅ **Intent class validated** before membrane
- ✅ **Pact requirements** enforced by policy
- ✅ **Container-specific rules** (C.Jobs, C.Messenger, etc.)
- ✅ **Audit trail** of policy decisions
- ✅ **Evolution blocked** without policy approval

---

## SPEC Compliance

| Requirement | Status |
|-------------|--------|
| TDLN governs translations | ✅ Policy evaluated before commit |
| Deterministic execution | ✅ Bytecode VM, no side effects |
| Compilable to bytecode | ✅ PolicyCompiler |
| Intent class validation | ✅ Checked against policy |
| Pact requirements | ✅ Policy can require pacts |
| Semantically blind | ✅ VM only sees bytes |
| No retroactive changes | ✅ Policies are versioned |
| Offline verification | ✅ BLAKE3 hash of bytecode |

---

## Example: Transfer Policy

```rust
// Small transfers: allow without pact
// Large transfers: require "high_value" pact

let policy = PolicyDefinition {
    policy_id: "transfer_policy".to_string(),
    version: "1.0".to_string(),
    rules: vec![
        // Small transfer (≤ 10000)
        PolicyRule {
            rule_id: "small".to_string(),
            intent_class: IntentClassSpec::Conservation,
            constraints: vec![
                Constraint::IntentTypeEquals { value: "transfer".to_string() },
                Constraint::AmountMax { max: 10000 },
            ],
            required_pact: None,
        },
        // Large transfer (> 10000)
        PolicyRule {
            rule_id: "large".to_string(),
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
```

---

*"TDLN é a lei que governa quais significados podem se tornar fatos no UBL."*

**UBL is now production-ready with proper governance.** 🔒



