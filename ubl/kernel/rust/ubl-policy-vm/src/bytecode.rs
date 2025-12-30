//! TDLN Bytecode - Deterministic Policy Execution
//!
//! SPEC-UBL-POLICY v1.0 §9:
//! > A política compilada DEVE produzir o mesmo resultado que a política fonte.
//!
//! This module implements a hardened, deterministic bytecode VM.
//!
//! ## Security Features
//! - Gas limits (prevents infinite loops)
//! - Stack size limits (prevents stack overflow)
//! - String length limits (prevents memory exhaustion)
//! - Bytecode size limits (prevents DoS)
//! - Constant-time hash comparison
//! - Intent class validation
//! - No unsafe code

#![deny(unsafe_code)]

use serde::{Deserialize, Serialize};
use thiserror::Error;

// ============================================================================
// SECURITY LIMITS - These are defense-in-depth measures
// ============================================================================

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

/// Valid intent classes per SPEC-UBL-CORE v1.0 §6.2
/// Intent class for Observation operations (read-only queries, no state changes)
pub const INTENT_CLASS_OBSERVATION: u8 = 0x00;
/// Intent class for Conservation operations (read-only, no state changes)
pub const INTENT_CLASS_CONSERVATION: u8 = 0x01;
/// Intent class for Entropy operations (state-changing mutations)
pub const INTENT_CLASS_ENTROPY: u8 = 0x02;
/// Intent class for Evolution operations (schema changes, requires multi-sig)
pub const INTENT_CLASS_EVOLUTION: u8 = 0x03;

// ============================================================================
// OPCODE DEFINITIONS
// ============================================================================

/// Bytecode instruction set
/// 
/// ## Opcode Ranges
/// - 0x00-0x0F: Stack operations
/// - 0x10-0x1F: Context access
/// - 0x20-0x2F: Comparison
/// - 0x30-0x3F: Arithmetic
/// - 0x40-0x4F: Logic
/// - 0x50-0x5F: Control flow
/// - 0x60-0x6F: String operations
/// - 0xF0-0xFF: Results
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum Opcode {
    /// No operation (1 gas)
    Nop = 0x00,
    
    // Stack operations (0x01-0x0F)
    /// Push i64 constant onto stack (consumes 8 bytes)
    PushI64 = 0x01,
    /// Push string from constant pool (consumes 2 bytes for index)
    PushStr = 0x02,
    /// Push true
    PushTrue = 0x03,
    /// Push false
    PushFalse = 0x04,
    /// Pop and discard top value
    Pop = 0x05,
    /// Duplicate top value
    Dup = 0x06,
    /// Swap top two values
    Swap = 0x07,
    /// Push null
    PushNull = 0x08,
    
    // Context access (0x10-0x1F)
    /// Load named context field (consumes 2 bytes for key index)
    LoadContext = 0x10,
    /// Load named state field (consumes 2 bytes for key index)
    LoadState = 0x11,
    /// Load named intent field (consumes 2 bytes for key index)
    LoadIntent = 0x12,
    /// Load current timestamp (i64)
    LoadTimestamp = 0x13,
    /// Load container_id (string)
    LoadContainerId = 0x14,
    /// Load actor (string)
    LoadActor = 0x15,
    /// Check if intent field exists (consumes 2 bytes, pushes bool)
    HasIntent = 0x16,
    
    // Comparison (0x20-0x2F)
    /// Equal: pop 2, push (a == b)
    Eq = 0x20,
    /// Not equal: pop 2, push (a != b)
    Ne = 0x21,
    /// Less than (i64): pop 2, push (a < b)
    Lt = 0x22,
    /// Less than or equal (i64): pop 2, push (a <= b)
    Le = 0x23,
    /// Greater than (i64): pop 2, push (a > b)
    Gt = 0x24,
    /// Greater than or equal (i64): pop 2, push (a >= b)
    Ge = 0x25,
    /// Is null: pop 1, push (a == null)
    IsNull = 0x26,
    /// Is not null: pop 1, push (a != null)
    IsNotNull = 0x27,
    
    // Arithmetic (0x30-0x3F) - all use saturating ops
    /// Add (saturating): pop 2, push (a + b)
    Add = 0x30,
    /// Subtract (saturating): pop 2, push (a - b)
    Sub = 0x31,
    /// Multiply (saturating): pop 2, push (a * b)
    Mul = 0x32,
    /// Divide (checked, errors on zero): pop 2, push (a / b)
    Div = 0x33,
    /// Modulo (checked, errors on zero): pop 2, push (a % b)
    Mod = 0x34,
    /// Negate: pop 1, push (-a)
    Neg = 0x35,
    /// Absolute value: pop 1, push (|a|)
    Abs = 0x36,
    
    // Logic (0x40-0x4F)
    /// And: pop 2, push (a && b)
    And = 0x40,
    /// Or: pop 2, push (a || b)
    Or = 0x41,
    /// Not: pop 1, push (!a)
    Not = 0x42,
    
    // Control flow (0x50-0x5F)
    /// Unconditional jump (consumes 2 bytes for address)
    Jump = 0x50,
    /// Jump if true (consumes 2 bytes, pops condition)
    JumpIf = 0x51,
    /// Jump if false (consumes 2 bytes, pops condition)
    JumpIfNot = 0x52,
    
    // String operations (0x60-0x6F)
    /// String contains: pop 2 (haystack, needle), push bool
    StrContains = 0x60,
    /// String starts with: pop 2 (str, prefix), push bool
    StrStartsWith = 0x61,
    /// String equals: pop 2, push bool
    StrEq = 0x62,
    /// String ends with: pop 2 (str, suffix), push bool
    StrEndsWith = 0x63,
    /// String length: pop 1, push i64
    StrLen = 0x64,
    /// String to lowercase: pop 1, push string
    StrLower = 0x65,
    
    // Results (0xF0-0xFF) - terminal operations
    /// Allow with intent class on stack (pops i64, validates 0-3)
    Allow = 0xF0,
    /// Allow with pact (pops string then i64)
    AllowWithPact = 0xF1,
    /// Deny with reason (pops string)
    Deny = 0xF2,
    /// Allow with constraints (pops array, string, i64)
    AllowWithConstraints = 0xF3,
    
    /// Halt execution (error - should never reach)
    Halt = 0xFF,
}

// ============================================================================
// VALUE TYPE
// ============================================================================

/// A value on the VM stack
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// 64-bit signed integer
    I64(i64),
    /// Boolean
    Bool(bool),
    /// String (limited to MAX_STRING_LENGTH)
    String(String),
    /// Null/missing value
    Null,
}

impl Value {
    /// Extract as i64, returns None for non-integer types
    #[inline]
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Value::I64(v) => Some(*v),
            _ => None,
        }
    }

    /// Extract as bool, returns None for non-boolean types
    #[inline]
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(v) => Some(*v),
            _ => None,
        }
    }

    /// Extract as string reference, returns None for non-string types
    #[inline]
    pub fn as_string(&self) -> Option<&str> {
        match self {
            Value::String(v) => Some(v),
            _ => None,
        }
    }

    /// Check if value is null
    #[inline]
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    /// Truthy evaluation for conditionals
    #[inline]
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::I64(i) => *i != 0,
            Value::String(s) => !s.is_empty(),
            Value::Null => false,
        }
    }

    /// Get type name for error messages
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::I64(_) => "i64",
            Value::Bool(_) => "bool",
            Value::String(_) => "string",
            Value::Null => "null",
        }
    }
}

// ============================================================================
// ERRORS
// ============================================================================

/// Bytecode execution error
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum BytecodeError {
    /// Stack underflow - attempted to pop from empty stack
    #[error("Stack underflow at pc={0}")]
    StackUnderflow(usize),
    
    /// Stack overflow - exceeded MAX_STACK_SIZE
    #[error("Stack overflow (max {0})")]
    StackOverflow(usize),
    
    /// Type mismatch in operation
    #[error("Type mismatch at pc={pc}: expected {expected}, got {got}")]
    TypeMismatch { 
        /// Program counter where error occurred
        pc: usize, 
        /// Expected type name
        expected: &'static str, 
        /// Actual type name found
        got: String 
    },
    
    /// Invalid opcode encountered
    #[error("Invalid opcode 0x{0:02X} at pc={1}")]
    InvalidOpcode(u8, usize),
    
    /// Invalid constant pool index
    #[error("Invalid constant index {0} (pool size {1})")]
    InvalidConstant(usize, usize),
    
    /// Gas exhausted
    #[error("Gas exhausted after {0} operations")]
    GasExhausted(u64),
    
    /// Division or modulo by zero
    #[error("Division by zero at pc={0}")]
    DivisionByZero(usize),
    
    /// Invalid jump target
    #[error("Invalid jump to {0} (code size {1})")]
    InvalidJump(usize, usize),
    
    /// Execution completed without a result
    #[error("No result (fell off end of bytecode)")]
    NoResult,
    
    /// Policy hash verification failed
    #[error("Policy hash mismatch (tampering detected)")]
    HashMismatch,
    
    /// Bytecode too large
    #[error("Bytecode size {0} exceeds limit {1}")]
    BytecodeTooLarge(usize, usize),
    
    /// Too many constants
    #[error("Constant pool size {0} exceeds limit {1}")]
    TooManyConstants(usize, usize),
    
    /// String too long
    #[error("String length {0} exceeds limit {1}")]
    StringTooLong(usize, usize),
    
    /// Invalid intent class
    #[error("Invalid intent class 0x{0:02X} (must be 0x00-0x03)")]
    InvalidIntentClass(u8),
    
    /// Bytecode truncated (missing operand bytes)
    #[error("Truncated bytecode at pc={0} (need {1} bytes, have {2})")]
    TruncatedBytecode(usize, usize, usize),
}

/// Result type alias for bytecode operations
pub type Result<T> = std::result::Result<T, BytecodeError>;

// ============================================================================
// COMPILED POLICY
// ============================================================================

/// Compiled policy bytecode with integrity verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompiledPolicy {
    /// Policy identifier
    pub policy_id: String,
    /// Semantic version
    pub version: String,
    /// Bytecode instructions
    pub code: Vec<u8>,
    /// Constant pool (strings)
    pub constants: Vec<String>,
    /// BLAKE3 hash of (domain_tag || code || constants)
    pub hash: String,
    /// Optional Ed25519 signature (hex-encoded)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
}

impl CompiledPolicy {
    /// Create a new compiled policy with computed hash
    pub fn new(policy_id: &str, version: &str, code: Vec<u8>, constants: Vec<String>) -> Self {
        let hash = compute_policy_hash(&code, &constants);
        Self {
            policy_id: policy_id.to_string(),
            version: version.to_string(),
            code,
            constants,
            hash,
            signature: None,
        }
    }

    /// Verify hash integrity (constant-time comparison)
    pub fn verify_hash(&self) -> bool {
        let computed = compute_policy_hash(&self.code, &self.constants);
        constant_time_eq(&computed, &self.hash)
    }

    /// Validate policy against security limits
    pub fn validate(&self) -> Result<()> {
        // Check bytecode size
        if self.code.len() > MAX_BYTECODE_SIZE {
            return Err(BytecodeError::BytecodeTooLarge(self.code.len(), MAX_BYTECODE_SIZE));
        }

        // Check constant pool size
        if self.constants.len() > MAX_CONSTANTS {
            return Err(BytecodeError::TooManyConstants(self.constants.len(), MAX_CONSTANTS));
        }

        // Check individual string lengths
        for (_i, s) in self.constants.iter().enumerate() {
            if s.len() > MAX_STRING_LENGTH {
                return Err(BytecodeError::StringTooLong(s.len(), MAX_STRING_LENGTH));
            }
        }

        // Verify hash
        if !self.verify_hash() {
            return Err(BytecodeError::HashMismatch);
        }

        Ok(())
    }
}

/// Compute BLAKE3 hash with domain separation
fn compute_policy_hash(code: &[u8], constants: &[String]) -> String {
    let mut hasher = blake3::Hasher::new();
    
    // Domain tag for policy hashes
    hasher.update(b"ubl:policy:v1\n");
    
    // Code length prefix (8 bytes, big-endian)
    hasher.update(&(code.len() as u64).to_be_bytes());
    hasher.update(code);
    
    // Constants count prefix
    hasher.update(&(constants.len() as u64).to_be_bytes());
    for c in constants {
        // Length-prefixed strings
        hasher.update(&(c.len() as u32).to_be_bytes());
        hasher.update(c.as_bytes());
    }
    
    hex::encode(hasher.finalize().as_bytes())
}

/// Constant-time string comparison (defense against timing attacks)
#[inline]
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

// ============================================================================
// EXECUTION CONTEXT
// ============================================================================

/// Context provided to policy execution
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// Container ID being accessed
    pub container_id: String,
    /// Actor (public key or entity ID)
    pub actor: String,
    /// Intent payload (JSON)
    pub intent: serde_json::Value,
    /// Current ledger state (optional)
    pub state: Option<serde_json::Value>,
    /// Current timestamp (milliseconds since epoch)
    pub timestamp: i64,
}

impl ExecutionContext {
    /// Get a named context field
    pub fn get(&self, key: &str) -> Value {
        match key {
            "container_id" => Value::String(self.container_id.clone()),
            "actor" => Value::String(self.actor.clone()),
            "timestamp" => Value::I64(self.timestamp),
            _ => Value::Null,
        }
    }

    /// Get a named intent field
    pub fn get_intent(&self, key: &str) -> Value {
        json_to_value(self.intent.get(key))
    }

    /// Check if intent has a field
    pub fn has_intent(&self, key: &str) -> bool {
        self.intent.get(key).is_some()
    }

    /// Get a named state field
    pub fn get_state(&self, key: &str) -> Value {
        self.state.as_ref()
            .and_then(|s| s.get(key))
            .map(|v| json_to_value(Some(v)))
            .unwrap_or(Value::Null)
    }
}

/// Convert JSON value to VM value
fn json_to_value(v: Option<&serde_json::Value>) -> Value {
    match v {
        Some(serde_json::Value::Number(n)) => {
            if let Some(i) = n.as_i64() {
                Value::I64(i)
            } else if let Some(f) = n.as_f64() {
                // Truncate floats to integers (policy shouldn't use floats)
                Value::I64(f as i64)
            } else {
                Value::Null
            }
        }
        Some(serde_json::Value::Bool(b)) => Value::Bool(*b),
        Some(serde_json::Value::String(s)) => {
            // Enforce string length limit
            if s.len() > MAX_STRING_LENGTH {
                Value::String(s[..MAX_STRING_LENGTH].to_string())
            } else {
                Value::String(s.clone())
            }
        }
        Some(serde_json::Value::Null) | None => Value::Null,
        Some(serde_json::Value::Array(_)) | Some(serde_json::Value::Object(_)) => {
            // Complex types become null (policy can't access nested structures directly)
            Value::Null
        }
    }
}

// ============================================================================
// BYTECODE VM
// ============================================================================

/// Configuration for the bytecode VM
#[derive(Debug, Clone)]
pub struct VMConfig {
    /// Maximum operations per execution
    pub max_gas: u64,
    /// Maximum stack depth
    pub max_stack: usize,
    /// Enable strict mode (fail on any warning)
    pub strict: bool,
}

impl Default for VMConfig {
    fn default() -> Self {
        Self {
            max_gas: MAX_GAS,
            max_stack: MAX_STACK_SIZE,
            strict: true,
        }
    }
}

/// Bytecode Virtual Machine - executes compiled policies deterministically
pub struct BytecodeVM {
    config: VMConfig,
}

impl BytecodeVM {
    /// Create with default configuration
    pub fn new(max_gas: u64, max_stack: usize) -> Self {
        Self {
            config: VMConfig {
                max_gas,
                max_stack,
                strict: true,
            },
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: VMConfig) -> Self {
        Self { config }
    }

    /// Execute a compiled policy
    /// 
    /// # Security
    /// - Validates policy hash before execution
    /// - Enforces gas and stack limits
    /// - Validates all intent classes
    pub fn execute(
        &self,
        policy: &CompiledPolicy,
        context: &ExecutionContext,
    ) -> Result<PolicyResult> {
        // Validate policy first
        policy.validate()?;

        let mut vm = VMState {
            pc: 0,
            stack: Vec::with_capacity(64),
            gas: self.config.max_gas,
            max_stack: self.config.max_stack,
        };

        let code = &policy.code;
        let constants = &policy.constants;

        loop {
            // Gas check
            if vm.gas == 0 {
                return Err(BytecodeError::GasExhausted(self.config.max_gas));
            }
            vm.gas -= 1;

            // Bounds check
            if vm.pc >= code.len() {
                return Err(BytecodeError::NoResult);
            }

            let opcode = code[vm.pc];
            let op_pc = vm.pc;  // Save for error messages
            vm.pc += 1;

            match opcode {
                // === Stack Operations ===
                0x00 => { /* Nop */ }
                
                0x01 => { // PushI64
                    let bytes = read_bytes::<8>(code, &mut vm.pc, op_pc)?;
                    vm.push(Value::I64(i64::from_be_bytes(bytes)))?;
                }
                
                0x02 => { // PushStr
                    let idx = read_u16(code, &mut vm.pc, op_pc)? as usize;
                    let s = constants.get(idx)
                        .ok_or(BytecodeError::InvalidConstant(idx, constants.len()))?;
                    vm.push(Value::String(s.clone()))?;
                }
                
                0x03 => vm.push(Value::Bool(true))?,   // PushTrue
                0x04 => vm.push(Value::Bool(false))?,  // PushFalse
                0x05 => { vm.pop(op_pc)?; }            // Pop
                
                0x06 => { // Dup
                    let v = vm.peek(op_pc)?.clone();
                    vm.push(v)?;
                }
                
                0x07 => { // Swap
                    let b = vm.pop(op_pc)?;
                    let a = vm.pop(op_pc)?;
                    vm.push(b)?;
                    vm.push(a)?;
                }
                
                0x08 => vm.push(Value::Null)?, // PushNull

                // === Context Access ===
                0x10 => { // LoadContext
                    let idx = read_u16(code, &mut vm.pc, op_pc)? as usize;
                    let key = constants.get(idx)
                        .ok_or(BytecodeError::InvalidConstant(idx, constants.len()))?;
                    vm.push(context.get(key))?;
                }
                
                0x11 => { // LoadState
                    let idx = read_u16(code, &mut vm.pc, op_pc)? as usize;
                    let key = constants.get(idx)
                        .ok_or(BytecodeError::InvalidConstant(idx, constants.len()))?;
                    vm.push(context.get_state(key))?;
                }
                
                0x12 => { // LoadIntent
                    let idx = read_u16(code, &mut vm.pc, op_pc)? as usize;
                    let key = constants.get(idx)
                        .ok_or(BytecodeError::InvalidConstant(idx, constants.len()))?;
                    vm.push(context.get_intent(key))?;
                }
                
                0x13 => vm.push(Value::I64(context.timestamp))?,           // LoadTimestamp
                0x14 => vm.push(Value::String(context.container_id.clone()))?, // LoadContainerId
                0x15 => vm.push(Value::String(context.actor.clone()))?,    // LoadActor
                
                0x16 => { // HasIntent
                    let idx = read_u16(code, &mut vm.pc, op_pc)? as usize;
                    let key = constants.get(idx)
                        .ok_or(BytecodeError::InvalidConstant(idx, constants.len()))?;
                    vm.push(Value::Bool(context.has_intent(key)))?;
                }

                // === Comparison ===
                0x20 => { // Eq
                    let b = vm.pop(op_pc)?;
                    let a = vm.pop(op_pc)?;
                    vm.push(Value::Bool(values_equal(&a, &b)))?;
                }
                
                0x21 => { // Ne
                    let b = vm.pop(op_pc)?;
                    let a = vm.pop(op_pc)?;
                    vm.push(Value::Bool(!values_equal(&a, &b)))?;
                }
                
                0x22 => { // Lt
                    let b = vm.pop_i64(op_pc)?;
                    let a = vm.pop_i64(op_pc)?;
                    vm.push(Value::Bool(a < b))?;
                }
                
                0x23 => { // Le
                    let b = vm.pop_i64(op_pc)?;
                    let a = vm.pop_i64(op_pc)?;
                    vm.push(Value::Bool(a <= b))?;
                }
                
                0x24 => { // Gt
                    let b = vm.pop_i64(op_pc)?;
                    let a = vm.pop_i64(op_pc)?;
                    vm.push(Value::Bool(a > b))?;
                }
                
                0x25 => { // Ge
                    let b = vm.pop_i64(op_pc)?;
                    let a = vm.pop_i64(op_pc)?;
                    vm.push(Value::Bool(a >= b))?;
                }
                
                0x26 => { // IsNull
                    let v = vm.pop(op_pc)?;
                    vm.push(Value::Bool(v.is_null()))?;
                }
                
                0x27 => { // IsNotNull
                    let v = vm.pop(op_pc)?;
                    vm.push(Value::Bool(!v.is_null()))?;
                }

                // === Arithmetic (saturating) ===
                0x30 => { // Add
                    let b = vm.pop_i64(op_pc)?;
                    let a = vm.pop_i64(op_pc)?;
                    vm.push(Value::I64(a.saturating_add(b)))?;
                }
                
                0x31 => { // Sub
                    let b = vm.pop_i64(op_pc)?;
                    let a = vm.pop_i64(op_pc)?;
                    vm.push(Value::I64(a.saturating_sub(b)))?;
                }
                
                0x32 => { // Mul
                    let b = vm.pop_i64(op_pc)?;
                    let a = vm.pop_i64(op_pc)?;
                    vm.push(Value::I64(a.saturating_mul(b)))?;
                }
                
                0x33 => { // Div
                    let b = vm.pop_i64(op_pc)?;
                    let a = vm.pop_i64(op_pc)?;
                    if b == 0 {
                        return Err(BytecodeError::DivisionByZero(op_pc));
                    }
                    vm.push(Value::I64(a / b))?;
                }
                
                0x34 => { // Mod
                    let b = vm.pop_i64(op_pc)?;
                    let a = vm.pop_i64(op_pc)?;
                    if b == 0 {
                        return Err(BytecodeError::DivisionByZero(op_pc));
                    }
                    vm.push(Value::I64(a % b))?;
                }
                
                0x35 => { // Neg
                    let a = vm.pop_i64(op_pc)?;
                    vm.push(Value::I64(a.saturating_neg()))?;
                }
                
                0x36 => { // Abs
                    let a = vm.pop_i64(op_pc)?;
                    vm.push(Value::I64(a.saturating_abs()))?;
                }

                // === Logic ===
                0x40 => { // And
                    let b = vm.pop_bool(op_pc)?;
                    let a = vm.pop_bool(op_pc)?;
                    vm.push(Value::Bool(a && b))?;
                }
                
                0x41 => { // Or
                    let b = vm.pop_bool(op_pc)?;
                    let a = vm.pop_bool(op_pc)?;
                    vm.push(Value::Bool(a || b))?;
                }
                
                0x42 => { // Not
                    let a = vm.pop_bool(op_pc)?;
                    vm.push(Value::Bool(!a))?;
                }

                // === Control Flow ===
                0x50 => { // Jump
                    let addr = read_u16(code, &mut vm.pc, op_pc)? as usize;
                    if addr >= code.len() {
                        return Err(BytecodeError::InvalidJump(addr, code.len()));
                    }
                    vm.pc = addr;
                }
                
                0x51 => { // JumpIf
                    let addr = read_u16(code, &mut vm.pc, op_pc)? as usize;
                    let cond = vm.pop_bool(op_pc)?;
                    if cond {
                        if addr >= code.len() {
                            return Err(BytecodeError::InvalidJump(addr, code.len()));
                        }
                        vm.pc = addr;
                    }
                }
                
                0x52 => { // JumpIfNot
                    let addr = read_u16(code, &mut vm.pc, op_pc)? as usize;
                    let cond = vm.pop_bool(op_pc)?;
                    if !cond {
                        if addr >= code.len() {
                            return Err(BytecodeError::InvalidJump(addr, code.len()));
                        }
                        vm.pc = addr;
                    }
                }

                // === String Operations ===
                0x60 => { // StrContains
                    let needle = vm.pop_string(op_pc)?;
                    let haystack = vm.pop_string(op_pc)?;
                    vm.push(Value::Bool(haystack.contains(&needle)))?;
                }
                
                0x61 => { // StrStartsWith
                    let prefix = vm.pop_string(op_pc)?;
                    let s = vm.pop_string(op_pc)?;
                    vm.push(Value::Bool(s.starts_with(&prefix)))?;
                }
                
                0x62 => { // StrEq
                    let b = vm.pop_string(op_pc)?;
                    let a = vm.pop_string(op_pc)?;
                    vm.push(Value::Bool(a == b))?;
                }
                
                0x63 => { // StrEndsWith
                    let suffix = vm.pop_string(op_pc)?;
                    let s = vm.pop_string(op_pc)?;
                    vm.push(Value::Bool(s.ends_with(&suffix)))?;
                }
                
                0x64 => { // StrLen
                    let s = vm.pop_string(op_pc)?;
                    vm.push(Value::I64(s.len() as i64))?;
                }
                
                0x65 => { // StrLower
                    let s = vm.pop_string(op_pc)?;
                    vm.push(Value::String(s.to_lowercase()))?;
                }

                // === Results (terminal) ===
                0xF0 => { // Allow
                    let intent_class = vm.pop_i64(op_pc)? as u8;
                    validate_intent_class(intent_class)?;
                    return Ok(PolicyResult::Allow {
                        intent_class,
                        required_pact: None,
                        constraints: vec![],
                    });
                }
                
                0xF1 => { // AllowWithPact
                    let pact_id = vm.pop_string(op_pc)?;
                    let intent_class = vm.pop_i64(op_pc)? as u8;
                    validate_intent_class(intent_class)?;
                    return Ok(PolicyResult::Allow {
                        intent_class,
                        required_pact: Some(pact_id),
                        constraints: vec![],
                    });
                }
                
                0xF2 => { // Deny
                    let reason = vm.pop_string(op_pc)?;
                    return Ok(PolicyResult::Deny { reason });
                }
                
                0xFF => { // Halt
                    return Err(BytecodeError::NoResult);
                }
                
                _ => return Err(BytecodeError::InvalidOpcode(opcode, op_pc)),
            }
        }
    }
}

impl Default for BytecodeVM {
    fn default() -> Self {
        Self {
            config: VMConfig::default(),
        }
    }
}

// ============================================================================
// VM STATE
// ============================================================================

/// Internal execution state
struct VMState {
    pc: usize,
    stack: Vec<Value>,
    gas: u64,
    max_stack: usize,
}

impl VMState {
    #[inline]
    fn push(&mut self, v: Value) -> Result<()> {
        if self.stack.len() >= self.max_stack {
            return Err(BytecodeError::StackOverflow(self.max_stack));
        }
        self.stack.push(v);
        Ok(())
    }

    #[inline]
    fn pop(&mut self, pc: usize) -> Result<Value> {
        self.stack.pop().ok_or(BytecodeError::StackUnderflow(pc))
    }

    #[inline]
    fn peek(&self, pc: usize) -> Result<&Value> {
        self.stack.last().ok_or(BytecodeError::StackUnderflow(pc))
    }

    #[inline]
    fn pop_i64(&mut self, pc: usize) -> Result<i64> {
        let v = self.pop(pc)?;
        v.as_i64().ok_or_else(|| BytecodeError::TypeMismatch {
            pc,
            expected: "i64",
            got: v.type_name().to_string(),
        })
    }

    #[inline]
    fn pop_bool(&mut self, pc: usize) -> Result<bool> {
        let v = self.pop(pc)?;
        v.as_bool().ok_or_else(|| BytecodeError::TypeMismatch {
            pc,
            expected: "bool",
            got: v.type_name().to_string(),
        })
    }

    #[inline]
    fn pop_string(&mut self, pc: usize) -> Result<String> {
        let v = self.pop(pc)?;
        match v {
            Value::String(s) => Ok(s),
            _ => Err(BytecodeError::TypeMismatch {
                pc,
                expected: "string",
                got: v.type_name().to_string(),
            }),
        }
    }
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Read N bytes from bytecode
#[inline]
fn read_bytes<const N: usize>(code: &[u8], pc: &mut usize, op_pc: usize) -> Result<[u8; N]> {
    if *pc + N > code.len() {
        return Err(BytecodeError::TruncatedBytecode(op_pc, N, code.len() - *pc));
    }
    let bytes: [u8; N] = code[*pc..*pc + N].try_into().unwrap();
    *pc += N;
    Ok(bytes)
}

/// Read u16 from bytecode (big-endian)
#[inline]
fn read_u16(code: &[u8], pc: &mut usize, op_pc: usize) -> Result<u16> {
    let bytes = read_bytes::<2>(code, pc, op_pc)?;
    Ok(u16::from_be_bytes(bytes))
}

/// Compare two values for equality
#[inline]
fn values_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::I64(x), Value::I64(y)) => x == y,
        (Value::Bool(x), Value::Bool(y)) => x == y,
        (Value::String(x), Value::String(y)) => x == y,
        (Value::Null, Value::Null) => true,
        _ => false,
    }
}

/// Validate intent class is in valid range
#[inline]
fn validate_intent_class(class: u8) -> Result<()> {
    if class > INTENT_CLASS_EVOLUTION {
        return Err(BytecodeError::InvalidIntentClass(class));
    }
    Ok(())
}

// ============================================================================
// POLICY RESULT
// ============================================================================

/// Result of policy execution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PolicyResult {
    /// Allow the translation
    Allow {
        /// Intent class (0x00-0x03)
        intent_class: u8,
        /// Required pact ID (if any)
        required_pact: Option<String>,
        /// Applied constraints (rule IDs that matched)
        constraints: Vec<String>,
    },
    /// Deny the translation
    Deny {
        /// Reason for denial
        reason: String,
    },
}

impl PolicyResult {
    /// Check if this is an Allow result
    #[inline]
    pub fn is_allow(&self) -> bool {
        matches!(self, PolicyResult::Allow { .. })
    }

    /// Check if this is a Deny result
    #[inline]
    pub fn is_deny(&self) -> bool {
        matches!(self, PolicyResult::Deny { .. })
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn make_context(intent_type: &str, amount: i64) -> ExecutionContext {
        ExecutionContext {
            container_id: "C.Test".to_string(),
            actor: "alice".to_string(),
            intent: serde_json::json!({
                "type": intent_type,
                "amount": amount
            }),
            state: None,
            timestamp: 1000,
        }
    }

    #[test]
    fn test_simple_allow() {
        let code = vec![
            0x01, 0, 0, 0, 0, 0, 0, 0, 0, // PushI64(0)
            0xF0, // Allow
        ];
        
        let policy = CompiledPolicy::new("test", "1.0", code, vec![]);
        let vm = BytecodeVM::default();
        let ctx = make_context("observe", 0);
        
        let result = vm.execute(&policy, &ctx).unwrap();
        assert!(matches!(result, PolicyResult::Allow { intent_class: 0, .. }));
    }

    #[test]
    fn test_deny() {
        let code = vec![
            0x02, 0, 0, // PushStr index 0
            0xF2, // Deny
        ];
        
        let policy = CompiledPolicy::new("test", "1.0", code, vec!["Not allowed".to_string()]);
        let vm = BytecodeVM::default();
        let ctx = make_context("hack", 0);
        
        let result = vm.execute(&policy, &ctx).unwrap();
        match result {
            PolicyResult::Deny { reason } => assert_eq!(reason, "Not allowed"),
            _ => panic!("Expected Deny"),
        }
    }

    #[test]
    fn test_gas_exhaustion() {
        let code = vec![0x50, 0, 0]; // Jump to 0
        
        let policy = CompiledPolicy::new("test", "1.0", code, vec![]);
        let vm = BytecodeVM::new(100, 256);
        let ctx = make_context("test", 0);
        
        let result = vm.execute(&policy, &ctx);
        assert!(matches!(result, Err(BytecodeError::GasExhausted(_))));
    }

    #[test]
    fn test_invalid_intent_class() {
        let code = vec![
            0x01, 0, 0, 0, 0, 0, 0, 0, 0xFF, // PushI64(255) - invalid class
            0xF0, // Allow
        ];
        
        let policy = CompiledPolicy::new("test", "1.0", code, vec![]);
        let vm = BytecodeVM::default();
        let ctx = make_context("test", 0);
        
        let result = vm.execute(&policy, &ctx);
        assert!(matches!(result, Err(BytecodeError::InvalidIntentClass(0xFF))));
    }

    #[test]
    fn test_hash_mismatch() {
        let mut policy = CompiledPolicy::new("test", "1.0", vec![0xF0], vec![]);
        policy.hash = "tampered".to_string();
        
        let vm = BytecodeVM::default();
        let ctx = make_context("test", 0);
        
        let result = vm.execute(&policy, &ctx);
        assert!(matches!(result, Err(BytecodeError::HashMismatch)));
    }

    #[test]
    fn test_mod_operation() {
        let code = vec![
            0x01, 0, 0, 0, 0, 0, 0, 0, 10, // PushI64(10)
            0x01, 0, 0, 0, 0, 0, 0, 0, 3,  // PushI64(3)
            0x34, // Mod
            0x01, 0, 0, 0, 0, 0, 0, 0, 1,  // PushI64(1) - expected result
            0x20, // Eq
            0x51, 0, 32, // JumpIf to allow
            0x02, 0, 0, // PushStr("fail")
            0xF2, // Deny
            0x01, 0, 0, 0, 0, 0, 0, 0, 0, // PushI64(0)
            0xF0, // Allow
        ];
        
        let policy = CompiledPolicy::new("test", "1.0", code, vec!["fail".to_string()]);
        let vm = BytecodeVM::default();
        let ctx = make_context("test", 0);
        
        let result = vm.execute(&policy, &ctx).unwrap();
        assert!(result.is_allow());
    }

    #[test]
    fn test_division_by_zero() {
        let code = vec![
            0x01, 0, 0, 0, 0, 0, 0, 0, 10, // PushI64(10)
            0x01, 0, 0, 0, 0, 0, 0, 0, 0,  // PushI64(0)
            0x33, // Div
        ];
        
        let policy = CompiledPolicy::new("test", "1.0", code, vec![]);
        let vm = BytecodeVM::default();
        let ctx = make_context("test", 0);
        
        let result = vm.execute(&policy, &ctx);
        assert!(matches!(result, Err(BytecodeError::DivisionByZero(_))));
    }

    #[test]
    fn test_load_state() {
        let code = vec![
            0x11, 0, 0, // LoadState("balance")
            0x26,       // IsNull
            0x51, 0, 14, // JumpIf to deny (no state)
            0x02, 0, 1, // PushStr("has state")
            0xF2,       // Deny
            0x01, 0, 0, 0, 0, 0, 0, 0, 0, // PushI64(0)
            0xF0,       // Allow
        ];
        
        let policy = CompiledPolicy::new("test", "1.0", code, vec!["balance".to_string(), "has state".to_string()]);
        let vm = BytecodeVM::default();
        
        // Without state - should allow
        let ctx = ExecutionContext {
            container_id: "C.Test".to_string(),
            actor: "alice".to_string(),
            intent: serde_json::json!({}),
            state: None,
            timestamp: 1000,
        };
        let result = vm.execute(&policy, &ctx).unwrap();
        assert!(result.is_allow());
    }

    #[test]
    fn test_constant_time_eq() {
        assert!(constant_time_eq("hello", "hello"));
        assert!(!constant_time_eq("hello", "world"));
        assert!(!constant_time_eq("hello", "hell"));
    }
}
