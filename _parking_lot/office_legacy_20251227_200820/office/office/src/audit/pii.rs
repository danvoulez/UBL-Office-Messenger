//! PII Protection - Never store raw PII in the ledger
//!
//! From the spec:
//! > Rule 1 — Never store raw attendee emails/phones in the ledger.
//! > Store: email_redacted (m***@acme.com), email_hash (BLAKE3)
//!
//! "If it's in the ledger, it's forever. Be careful what you write."

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

/// PII Policy - What redactions were applied
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PiiPolicy {
    /// What fields were redacted
    pub redactions_applied: Vec<String>,
    /// What fields were hashed
    pub hashes_applied: Vec<String>,
    /// Was raw PII stored? (should always be false)
    pub raw_pii_stored: bool,
}

impl PiiPolicy {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_redaction(mut self, field: &str) -> Self {
        self.redactions_applied.push(field.to_string());
        self
    }

    pub fn with_hash(mut self, field: &str) -> Self {
        self.hashes_applied.push(field.to_string());
        self
    }
}

// Regex patterns for PII detection
static EMAIL_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}").unwrap()
});

static PHONE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\+?[0-9][0-9\-\s()]{7,}").unwrap()
});

/// Redact an email address: "user@example.com" → "u***@example.com"
pub fn redact_email(email: &str) -> String {
    if let Some(at_pos) = email.find('@') {
        let (local, domain) = email.split_at(at_pos);
        if local.len() <= 1 {
            format!("*{}", domain)
        } else {
            format!("{}***{}", &local[..1], domain)
        }
    } else {
        "***".to_string()
    }
}

/// Redact a phone number: "+1234567890" → "+1***890"
pub fn redact_phone(phone: &str) -> String {
    let digits: String = phone.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() <= 6 {
        "***".to_string()
    } else {
        let prefix = &digits[..2];
        let suffix = &digits[digits.len()-3..];
        format!("{}***{}", prefix, suffix)
    }
}

/// Hash PII with BLAKE3 for correlation without exposure
/// Uses tenant_id as salt for additional isolation
pub fn hash_pii(value: &str, tenant_salt: &str) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"pii:");
    hasher.update(tenant_salt.as_bytes());
    hasher.update(b":");
    hasher.update(value.to_lowercase().as_bytes());
    let hash = hasher.finalize();
    format!("blake3:{}", hex::encode(&hash.as_bytes()[..16])) // First 16 bytes
}

/// Check if a string contains raw PII
pub fn contains_raw_pii(text: &str) -> bool {
    EMAIL_REGEX.is_match(text) || PHONE_REGEX.is_match(text)
}

/// Sanitize a JSON value, redacting any detected PII
pub fn sanitize_json(value: &serde_json::Value, tenant_id: &str) -> (serde_json::Value, PiiPolicy) {
    let mut policy = PiiPolicy::new();
    let sanitized = sanitize_value(value, tenant_id, "", &mut policy);
    (sanitized, policy)
}

fn sanitize_value(
    value: &serde_json::Value,
    tenant_id: &str,
    path: &str,
    policy: &mut PiiPolicy,
) -> serde_json::Value {
    match value {
        serde_json::Value::String(s) => {
            // Check for email
            if EMAIL_REGEX.is_match(s) {
                policy.redactions_applied.push(format!("{}_redacted", path));
                policy.hashes_applied.push(format!("{}_hash", path));
                
                let redacted = redact_email(s);
                let hash = hash_pii(s, tenant_id);
                
                // Return object with both redacted and hash
                serde_json::json!({
                    "redacted": redacted,
                    "hash": hash
                })
            } 
            // Check for phone
            else if PHONE_REGEX.is_match(s) && s.chars().filter(|c| c.is_ascii_digit()).count() >= 7 {
                policy.redactions_applied.push(format!("{}_redacted", path));
                
                serde_json::Value::String(redact_phone(s))
            }
            else {
                value.clone()
            }
        }
        serde_json::Value::Object(map) => {
            let mut new_map = serde_json::Map::new();
            for (key, val) in map {
                let new_path = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{}.{}", path, key)
                };
                new_map.insert(key.clone(), sanitize_value(val, tenant_id, &new_path, policy));
            }
            serde_json::Value::Object(new_map)
        }
        serde_json::Value::Array(arr) => {
            serde_json::Value::Array(
                arr.iter()
                    .enumerate()
                    .map(|(i, v)| {
                        let new_path = format!("{}[{}]", path, i);
                        sanitize_value(v, tenant_id, &new_path, policy)
                    })
                    .collect()
            )
        }
        _ => value.clone(),
    }
}

/// Attendee representation for tool inputs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanitizedAttendee {
    pub label: String,
    pub email_redacted: String,
    pub email_hash: String,
}

impl SanitizedAttendee {
    pub fn from_email(name: &str, email: &str, tenant_id: &str) -> Self {
        Self {
            label: name.to_string(),
            email_redacted: redact_email(email),
            email_hash: hash_pii(email, tenant_id),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redact_email() {
        assert_eq!(redact_email("john@example.com"), "j***@example.com");
        assert_eq!(redact_email("a@b.com"), "*@b.com");
        assert_eq!(redact_email("maria.silva@company.org"), "m***@company.org");
    }

    #[test]
    fn test_redact_phone() {
        assert_eq!(redact_phone("+1234567890"), "12***890");
        assert_eq!(redact_phone("(555) 123-4567"), "55***567");
    }

    #[test]
    fn test_hash_pii() {
        let hash1 = hash_pii("test@example.com", "tenant_123");
        let hash2 = hash_pii("test@example.com", "tenant_123");
        let hash3 = hash_pii("test@example.com", "tenant_456");

        assert!(hash1.starts_with("blake3:"));
        assert_eq!(hash1, hash2); // Same input, same hash
        assert_ne!(hash1, hash3); // Different salt, different hash
    }

    #[test]
    fn test_contains_raw_pii() {
        assert!(contains_raw_pii("Contact me at test@example.com"));
        assert!(contains_raw_pii("Call +1234567890"));
        assert!(!contains_raw_pii("Hello world"));
    }

    #[test]
    fn test_sanitize_json() {
        let input = serde_json::json!({
            "name": "John",
            "email": "john@example.com",
            "phone": "+1234567890",
            "nested": {
                "contact": "maria@company.org"
            }
        });

        let (sanitized, policy) = sanitize_json(&input, "tenant_123");

        // Check email was sanitized
        assert!(sanitized["email"]["redacted"].as_str().unwrap().contains("***"));
        assert!(sanitized["email"]["hash"].as_str().unwrap().starts_with("blake3:"));

        // Check policy recorded redactions
        assert!(!policy.redactions_applied.is_empty());
        assert!(!policy.hashes_applied.is_empty());
    }

    #[test]
    fn test_sanitized_attendee() {
        let attendee = SanitizedAttendee::from_email("Maria", "maria@acme.com", "tenant_123");
        
        assert_eq!(attendee.label, "Maria");
        assert_eq!(attendee.email_redacted, "m***@acme.com");
        assert!(attendee.email_hash.starts_with("blake3:"));
    }
}

