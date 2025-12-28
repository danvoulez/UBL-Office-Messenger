# 🔒 UBL Policy VM - Hardened

**Security Review Complete**

---

## Hardening Summary

### 1. Security Limits (Defense in Depth)

```rust
/// Maximum bytecode size (64KB)
pub const MAX_BYTECODE_SIZE: usize = 65_536;

/// Maximum constant pool size (1024 entries)
pub const MAX_CONSTANTS: usize = 1024;

/// Maximum string length in constant pool (4KB)
pub const MAX_STRING_LENGTH: usize = 4096;

/// Maximum gas for a single execution
pub const MAX_GAS: u64 = 100_000;

/// Maximum stack size
pub const MAX_STACK_SIZE: usize = 1024;

/// Maximum rules per policy
pub const MAX_RULES: usize = 256;

/// Maximum constraints per rule
pub const MAX_CONSTRAINTS_PER_RULE: usize = 32;
```

### 2. New Error Types

| Error | Description |
|-------|-------------|
| `HashMismatch` | Policy tampered (constant-time comparison) |
| `BytecodeTooLarge` | Bytecode exceeds 64KB |
| `TooManyConstants` | Constant pool exceeds 1024 |
| `StringTooLong` | String exceeds 4KB |
| `InvalidIntentClass` | Intent class not 0x00-0x03 |
| `TruncatedBytecode` | Missing operand bytes |
| `TooManyRules` | Policy has > 256 rules |
| `TooManyConstraints` | Rule has > 32 constraints |

### 3. New Opcodes

| Opcode | Description |
|--------|-------------|
| `0x07 Swap` | Swap top two stack values |
| `0x08 PushNull` | Push null value |
| `0x16 HasIntent` | Check if intent field exists |
| `0x26 IsNull` | Check if value is null |
| `0x27 IsNotNull` | Check if value is not null |
| `0x34 Mod` | Modulo operation (was missing!) |
| `0x35 Neg` | Negate (saturating) |
| `0x36 Abs` | Absolute value (saturating) |
| `0x63 StrEndsWith` | String ends with check |
| `0x64 StrLen` | Get string length |
| `0x65 StrLower` | Convert to lowercase |

### 4. Fixed Bugs

| Issue | Fix |
|-------|-----|
| Missing `Mod` opcode | Implemented with division-by-zero check |
| Missing `LoadState` opcode | Implemented (was defined but not in match) |
| Hash verification error | Now returns `HashMismatch` not `InvalidOpcode(0)` |
| Non-constant-time hash comparison | Now uses XOR-based constant-time comparison |
| Dead code in compiler | Removed unused `rule_start` variable |

### 5. Constant-Time Hash Comparison

```rust
/// Constant-time string comparison (defense against timing attacks)
fn constant_time_eq(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    
    let mut result = 0u8;
    for (x, y) in a.bytes().zip(b.bytes()) {
        result |= x ^ y;
    }
    result == 0
}
```

### 6. Intent Class Validation

```rust
/// Valid intent classes per SPEC-UBL-CORE v1.0 §6.2
pub const INTENT_CLASS_OBSERVATION: u8 = 0x00;
pub const INTENT_CLASS_CONSERVATION: u8 = 0x01;
pub const INTENT_CLASS_ENTROPY: u8 = 0x02;
pub const INTENT_CLASS_EVOLUTION: u8 = 0x03;

fn validate_intent_class(class: u8) -> Result<()> {
    if class > INTENT_CLASS_EVOLUTION {
        return Err(BytecodeError::InvalidIntentClass(class));
    }
    Ok(())
}
```

Now the VM rejects `Allow` with intent class > 0x03.

### 7. Policy Validation

```rust
impl CompiledPolicy {
    /// Validate policy against security limits
    pub fn validate(&self) -> Result<()> {
        // Check bytecode size
        if self.code.len() > MAX_BYTECODE_SIZE { ... }
        
        // Check constant pool size
        if self.constants.len() > MAX_CONSTANTS { ... }
        
        // Check individual string lengths
        for s in &self.constants {
            if s.len() > MAX_STRING_LENGTH { ... }
        }
        
        // Verify hash (constant-time)
        if !self.verify_hash() {
            return Err(BytecodeError::HashMismatch);
        }
        
        Ok(())
    }
}
```

### 8. Compiler Validation

```rust
impl PolicyCompiler {
    pub fn validate(policy: &PolicyDefinition) -> Result<(), CompilerError> {
        // Check rule count
        if policy.rules.len() > MAX_RULES { ... }
        
        // Check constraints per rule
        for rule in &policy.rules {
            if rule.constraints.len() > MAX_CONSTRAINTS_PER_RULE { ... }
        }
        
        Ok(())
    }
    
    /// Compile with validation (recommended)
    pub fn compile_validated(&mut self, policy: &PolicyDefinition) 
        -> Result<CompiledPolicy, CompilerError> 
    {
        Self::validate(policy)?;
        Ok(self.compile(policy))
    }
}
```

### 9. Improved Error Messages

All errors now include context:

```rust
#[error("Stack underflow at pc={0}")]
StackUnderflow(usize),

#[error("Type mismatch at pc={pc}: expected {expected}, got {got}")]
TypeMismatch { pc: usize, expected: &'static str, got: String },

#[error("Invalid opcode 0x{0:02X} at pc={1}")]
InvalidOpcode(u8, usize),

#[error("Invalid jump to {0} (code size {1})")]
InvalidJump(usize, usize),
```

### 10. Domain-Separated Hash

```rust
fn compute_policy_hash(code: &[u8], constants: &[String]) -> String {
    let mut hasher = blake3::Hasher::new();
    
    // Domain tag for policy hashes
    hasher.update(b"ubl:policy:v1\n");
    
    // Length-prefixed code
    hasher.update(&(code.len() as u64).to_be_bytes());
    hasher.update(code);
    
    // Length-prefixed constants
    hasher.update(&(constants.len() as u64).to_be_bytes());
    for c in constants {
        hasher.update(&(c.len() as u32).to_be_bytes());
        hasher.update(c.as_bytes());
    }
    
    hex::encode(hasher.finalize().as_bytes())
}
```

---

## New Tests Added

```rust
#[test]
fn test_invalid_intent_class() { ... }

#[test]
fn test_hash_mismatch() { ... }

#[test]
fn test_mod_operation() { ... }

#[test]
fn test_division_by_zero() { ... }

#[test]
fn test_load_state() { ... }

#[test]
fn test_constant_time_eq() { ... }
```

---

## Attack Vectors Mitigated

| Attack | Mitigation |
|--------|------------|
| **DoS via infinite loops** | Gas limit (100k ops) |
| **DoS via stack exhaustion** | Stack limit (1024) |
| **DoS via large bytecode** | Size limit (64KB) |
| **DoS via many constants** | Constant limit (1024) |
| **DoS via long strings** | String limit (4KB) |
| **Timing attack on hash** | Constant-time comparison |
| **Invalid intent class** | Validation before Allow |
| **Bytecode tampering** | BLAKE3 hash verification |
| **Compiler bomb** | Rule/constraint limits |

---

## Code Quality

- ✅ `#![deny(unsafe_code)]` - No unsafe Rust
- ✅ `#![warn(missing_docs)]` - All public items documented
- ✅ All operations use checked/saturating arithmetic
- ✅ All errors include location context (pc)
- ✅ Value type names for better error messages

---

*The Policy VM is now hardened for production use.* 🔒



