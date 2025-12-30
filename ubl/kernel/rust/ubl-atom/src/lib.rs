//! # UBL Atom
//!
//! **Title:** SPEC-UBL-ATOM v1.0  
//! **Status:** NORMATIVE  
//! **Change-Control:** STRICT (no retroactive changes)  
//! **Hash:** BLAKE3 | **Signature:** Ed25519 | **Canonical Format:** Json✯Atomic  
//! **Freeze-Date:** 2025-12-25  
//!
//! Canonical JSON serialization for deterministic hashing.
//! Binding: ubl-atom ≡ JSON✯Atomic v1.0
//!
//! ## Guarantees
//! - Semantically equal JSONs produce identical bytes
//! - Keys are sorted lexicographically (recursive)
//! - No whitespace in output
//! - Non-finite numbers are rejected
//!
//! ## Example
//! ```
//! use ubl_atom::canonicalize;
//! use serde_json::json;
//!
//! let data = json!({"z": 1, "a": 2});
//! let canonical = canonicalize(&data).unwrap();
//! assert_eq!(canonical, br#"{"a":2,"z":1}"#);
//! ```

#![deny(unsafe_code)]
#![warn(missing_docs)]

use serde_json::{Map, Value};
use thiserror::Error;

/// Errors that can occur during canonicalization
#[derive(Error, Debug)]
pub enum AtomError {
    /// JSON serialization failed
    #[error("Serialization failed: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Non-finite number detected (NaN, Infinity)
    #[error("Non-finite number detected")]
    NonFiniteNumber,
}

/// Result type for atom operations
pub type Result<T> = std::result::Result<T, AtomError>;

/// Canonicalize a JSON value to deterministic bytes
///
/// SPEC 5.1: Canonical Function
/// - Keys are sorted lexicographically (recursive)
/// - No whitespace in output
/// - Arrays preserve order
/// - Non-finite numbers are rejected
pub fn canonicalize(value: &Value) -> Result<Vec<u8>> {
    let sorted = sort_keys_recursive(value)?;
    Ok(serde_json::to_vec(&sorted)?)
}

/// Canonicalize to string (for debugging/display)
pub fn canonicalize_string(value: &Value) -> Result<String> {
    let bytes = canonicalize(value)?;
    Ok(String::from_utf8_lossy(&bytes).to_string())
}

/// Compute atom_hash = BLAKE3(canonical_bytes)
/// 
/// Per SPEC-UBL-ATOM v1.0 (updated): No domain tag, pure BLAKE3 of canonical form.
/// This is the hash used in ubl-link for atom_hash field.
pub fn atom_hash(value: &Value) -> Result<String> {
    let canonical = canonicalize(value)?;
    Ok(hex::encode(blake3::hash(&canonical).as_bytes()))
}

/// Compute atom_hash returning raw bytes (32 bytes)
pub fn atom_hash_bytes(value: &Value) -> Result<[u8; 32]> {
    let canonical = canonicalize(value)?;
    Ok(*blake3::hash(&canonical).as_bytes())
}

/// Recursively sort object keys
fn sort_keys_recursive(value: &Value) -> Result<Value> {
    match value {
        Value::Object(map) => {
            let mut sorted_map = Map::new();
            
            // SPEC 5.2 R1: Lexicographic ordering
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();

            for key in keys {
                let val = map.get(key).unwrap();
                sorted_map.insert(key.clone(), sort_keys_recursive(val)?);
            }
            
            Ok(Value::Object(sorted_map))
        }
        Value::Array(arr) => {
            // SPEC 5.2 R2: Arrays preserve order
            let sorted: Result<Vec<Value>> = arr.iter().map(sort_keys_recursive).collect();
            Ok(Value::Array(sorted?))
        }
        Value::Number(n) => {
            // SPEC 5.2 R3: Numeric normalization
            if let Some(f) = n.as_f64() {
                if f.is_nan() || f.is_infinite() {
                    return Err(AtomError::NonFiniteNumber);
                }
            }
            Ok(value.clone())
        }
        _ => Ok(value.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_sorts_keys() {
        let data = json!({"z": 1, "a": 2, "m": 3});
        let canonical = canonicalize_string(&data).unwrap();
        assert_eq!(canonical, r#"{"a":2,"m":3,"z":1}"#);
    }

    #[test]
    fn test_sorts_nested_keys() {
        let data = json!({
            "outer": {"z": 1, "a": 2},
            "array": [{"b": 1, "a": 2}]
        });
        let canonical = canonicalize_string(&data).unwrap();
        assert_eq!(canonical, r#"{"array":[{"a":2,"b":1}],"outer":{"a":2,"z":1}}"#);
    }

    #[test]
    fn test_preserves_array_order() {
        let data = json!([3, 1, 2]);
        let canonical = canonicalize_string(&data).unwrap();
        assert_eq!(canonical, "[3,1,2]");
    }

    #[test]
    fn test_deterministic() {
        let data1 = json!({"b": 2, "a": 1});
        let data2 = json!({"a": 1, "b": 2});
        
        let c1 = canonicalize(&data1).unwrap();
        let c2 = canonicalize(&data2).unwrap();
        
        assert_eq!(c1, c2);
    }

    #[test]
    fn test_no_whitespace() {
        let data = json!({"key": "value", "nested": {"a": 1}});
        let canonical = canonicalize_string(&data).unwrap();
        assert!(!canonical.contains(' '));
        assert!(!canonical.contains('\n'));
    }

    #[test]
    fn test_atom_hash_matches_blake3() {
        let v = json!({"a": 1, "b": [2, 3]});
        let h = atom_hash(&v).unwrap();
        let canon = canonicalize(&v).unwrap();
        let raw = hex::encode(blake3::hash(&canon).as_bytes());
        assert_eq!(h, raw);
    }

    #[test]
    fn test_atom_hash_deterministic() {
        let v1 = json!({"z": 1, "a": 2});
        let v2 = json!({"a": 2, "z": 1});
        assert_eq!(atom_hash(&v1).unwrap(), atom_hash(&v2).unwrap());
    }

    #[test]
    fn test_atom_hash_bytes() {
        let v = json!({"test": true});
        let hash_hex = atom_hash(&v).unwrap();
        let hash_bytes = atom_hash_bytes(&v).unwrap();
        assert_eq!(hash_hex, hex::encode(hash_bytes));
    }
}
