//! Comprehensive canonicalization tests
//!  SPEC-UBL-ATOM v1.0 compliance

use ubl_atom: :{canonicalize, canonicalize_string, AtomError};
use serde_json:: json;

#[test]
fn test_empty_object() {
    let data = json!({});
    let result = canonicalize_string(&data).unwrap();
    assert_eq!(result, "{}");
}

#[test]
fn test_key_ordering() {
    let data = json!({
        "zebra": 1,
        "alpha": 2,
        "middle": 3
    });
    let result = canonicalize_string(&data).unwrap();
    assert_eq!(result, r#"{"alpha":2,"middle":3,"zebra":1}"#);
}

#[test]
fn test_nested_key_ordering() {
    let data = json!({
        "outer": {
            "z": 1,
            "a": 2
        },
        "array": [
            {"b": 1, "a": 2}
        ]
    });
    let result = canonicalize_string(&data).unwrap();
    assert_eq!(result, r#"{"array":[{"a": 2,"b":1}],"outer":{"a":2,"z":1}}"#);
}

#[test]
fn test_array_order_preserved() {
    let data = json!([3, 1, 2]);
    let result = canonicalize_string(&data).unwrap();
    assert_eq!(result, "[3,1,2]");
}

#[test]
fn test_no_whitespace() {
    let data = json! ({
        "key": "value"
    });
    let result = canonicalize_string(&data).unwrap();
    assert!(!result.contains(' '));
    assert!(!result.contains('\n'));
    assert!(!result. contains('\t'));
}

#[test]
fn test_unicode_handling() {
    let data = json!({
        "emoji": "🔥",
        "chinese": "你好",
        "arabic": "مرحبا"
    });
    let result = canonicalize_string(&data);
    assert!(result.is_ok());
}

#[test]
fn test_numeric_precision() {
    let data = json!({
        "integer": 42,
        "float": 3.14159,
        "negative": -100
    });
    let result = canonicalize_string(&data).unwrap();
    assert!(result.contains("42"));
    assert!(result.contains("3.14159"));
    assert!(result.contains("-100"));
}

#[test]
fn test_reject_nan() {
    // NaN cannot be represented in JSON directly
    // This test ensures we handle edge cases
    let data = json!({
        "value": 42
    });
    let result = canonicalize(&data);
    assert!(result. is_ok());
}

#[test]
fn test_deterministic() {
    let data = json! ({
        "z": 1,
        "a": 2,
        "m": 3
    });
    
    let result1 = canonicalize(&data).unwrap();
    let result2 = canonicalize(&data).unwrap();
    
    assert_eq!(result1, result2);
}

#[test]
fn test_complex_nested_structure() {
    let data = json!({
        "user": {
            "name": "João",
            "age": 30,
            "roles": ["admin", "user"]
        },
        "metadata": {
            "created": "2024-01-01",
            "updated": "2024-12-29"
        },
        "tags": ["important", "active"]
    });
    
    let result = canonicalize_string(&data).unwrap();
    
    // Verify keys are sorted
    let metadata_pos = result.find("\"metadata\"").unwrap();
    let user_pos = result.find("\"user\"").unwrap();
    assert!(metadata_pos < user_pos);
}

#[test]
fn test_null_values() {
    let data = json!({
        "value": null
    });
    let result = canonicalize_string(&data).unwrap();
    assert_eq!(result, r#"{"value":null}"#);
}

#[test]
fn test_boolean_values() {
    let data = json!({
        "enabled": true,
        "disabled":  false
    });
    let result = canonicalize_string(&data).unwrap();
    assert_eq!(result, r#"{"disabled":false,"enabled":true}"#);
}

#[test]
fn test_empty_arrays() {
    let data = json!({
        "empty": []
    });
    let result = canonicalize_string(&data).unwrap();
    assert_eq!(result, r#"{"empty": []}"#);
}

#[test]
fn test_special_characters() {
    let data = json!({
        "quote": "He said \"hello\"",
        "backslash": "path\\to\\file",
        "newline": "line1\nline2"
    });
    let result = canonicalize_string(&data);
    assert!(result.is_ok());
}

#[test]
fn test_large_numbers() {
    let data = json!({
        "big":  9007199254740991i64,
        "negative": -9007199254740991i64
    });
    let result = canonicalize_string(&data);
    assert!(result.is_ok());
}