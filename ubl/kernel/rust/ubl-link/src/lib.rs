//! # UBL Link
//!
//! **Title:** SPEC-UBL-LINK v1.0  
//! **Status:** NORMATIVE  
//! **Change-Control:** STRICT (no retroactive changes)  
//! **Hash:** BLAKE3 | **Signature:** Ed25519  
//! **Freeze-Date:** 2025-12-25  
//! **Governed by:** SPEC-UBL-CORE v1.0  
//!
//! The only interface of tangency between Mind (TypeScript) and Body (Rust).
//! This is the sole valid protocol for materialization between containers.
//!
//! ## The Link Commit
//! This is the envelope that crosses the boundary Mind → Body.
//! It contains:
//! - Container identity
//! - Causal control (sequence, previous hash)
//! - Atom hash (the semantic content, hashed)
//! - Physical class (Observation, Conservation, Entropy, Evolution)
//! - Physics delta (the physical change)
//! - Authority (signature)

#![deny(unsafe_code)]
#![warn(missing_docs)]

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

/// SPEC 4: Intent Class
/// The physical classification of an intent.
/// SPEC-UBL-LINK v1.0 §4
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum IntentClass {
    /// Δ = 0 - Pure observation, no physical change
    Observation = 0x00,
    /// ∑Δ = 0 - Conservation law, paired changes required
    Conservation = 0x01,
    /// Authorized creation/destruction
    Entropy = 0x02,
    /// Explicit rule change
    Evolution = 0x03,
}

impl IntentClass {
    /// Get the byte representation for signing
    pub fn as_byte(&self) -> u8 {
        *self as u8
    }
}

/// Pact proof structure (SPEC-UBL-PACT v1.0 §8)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PactProof {
    /// Pact identifier
    pub pact_id: String,
    /// Signatures from authorized signers
    pub signatures: Vec<String>,
}

/// SPEC 3: The Link Commit Structure
/// This is what crosses the boundary Mind → Body.
/// SPEC-UBL-LINK v1.0 §3
/// 
/// **CRITICAL FIX (Diamond Checklist #1):** physics_delta is serialized as string
/// to prevent precision loss in JavaScript (i128 > 2^53 loses bits in JSON Number)
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkCommit {
    /// SPEC 3.2: Protocol version (must be 1)
    pub version: u8,
    
    /// SPEC 3.2: Container ID (Hash32 hex)
    pub container_id: String,
    
    /// SPEC 3.2: Expected sequence number (causal control)
    pub expected_sequence: u64,
    
    /// SPEC 3.2: Hash of the last accepted commit
    pub previous_hash: String,
    
    /// SPEC 3.2: Hash of the semantic content (atom)
    pub atom_hash: String,
    
    /// SPEC 3.2: Physical class of the intent
    pub intent_class: IntentClass,
    
    /// SPEC 3.2: Physical delta (change in value) - i128 internally, string in JSON
    /// Serialized as string to prevent JS precision loss (Diamond Checklist #1)
    #[serde_as(as = "DisplayFromStr")]
    pub physics_delta: i128,
    
    /// SPEC 3.2: Pact proof (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pact: Option<PactProof>,
    
    /// SPEC 3.2: Author's public key (hex Ed25519)
    pub author_pubkey: String,
    
    /// SPEC 3.2: Signature over the commit (hex Ed25519)
    pub signature: String,
}

impl LinkCommit {
    /// Generate the bytes that must be signed (SPEC-UBL-LINK v1.0 §5)
    /// CRITICAL: Does NOT include pact, author_pubkey, or signature
    pub fn signing_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        
        // Version (1 byte)
        bytes.push(self.version);
        
        // Container ID
        bytes.extend_from_slice(self.container_id.as_bytes());
        
        // Expected sequence (8 bytes, big-endian)
        bytes.extend_from_slice(&self.expected_sequence.to_be_bytes());
        
        // Previous hash
        bytes.extend_from_slice(self.previous_hash.as_bytes());
        
        // Atom hash
        bytes.extend_from_slice(self.atom_hash.as_bytes());
        
        // Intent class (1 byte)
        bytes.push(self.intent_class.as_byte());
        
        // Physics delta (16 bytes, big-endian for i128)
        bytes.extend_from_slice(&self.physics_delta.to_be_bytes());
        
        // STOP HERE - do NOT include pact, author_pubkey, or signature
        bytes
    }
}

/// The result of a successful commit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkReceipt {
    /// The hash of the committed entry
    pub entry_hash: String,
    
    /// The sequence number assigned
    pub sequence: u64,
    
    /// Timestamp of acceptance (Unix epoch)
    pub timestamp: i64,
    
    /// The container that accepted the commit
    pub container_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intent_class_byte() {
        assert_eq!(IntentClass::Observation.as_byte(), 0x00);
        assert_eq!(IntentClass::Conservation.as_byte(), 0x01);
        assert_eq!(IntentClass::Entropy.as_byte(), 0x02);
        assert_eq!(IntentClass::Evolution.as_byte(), 0x03);
    }

    #[test]
    fn test_signing_bytes_deterministic() {
        let commit = LinkCommit {
            version: 1,
            container_id: "container_a".to_string(),
            expected_sequence: 42,
            previous_hash: "abc123".to_string(),
            atom_hash: "def456".to_string(),
            intent_class: IntentClass::Conservation,
            physics_delta: -100,
            author_pubkey: "pubkey".to_string(),
            signature: "sig".to_string(),
            pact: None,
        };

        let bytes1 = commit.signing_bytes();
        let bytes2 = commit.signing_bytes();
        
        assert_eq!(bytes1, bytes2);
    }

    #[test]
    fn test_serialization() {
        let commit = LinkCommit {
            version: 1,
            container_id: "wallet_alice".to_string(),
            expected_sequence: 1,
            previous_hash: "0000".to_string(),
            atom_hash: "abcd".to_string(),
            intent_class: IntentClass::Conservation,
            physics_delta: -50,
            author_pubkey: "pk".to_string(),
            signature: "sig".to_string(),
            pact: None,
        };

        let json = serde_json::to_string(&commit).unwrap();
        let parsed: LinkCommit = serde_json::from_str(&json).unwrap();
        
        assert_eq!(parsed.container_id, commit.container_id);
        assert_eq!(parsed.physics_delta, commit.physics_delta);
    }

    #[test]
    fn test_physics_delta_as_string_in_json() {
        // Diamond Checklist #1: Verify physics_delta serializes as string
        let commit = LinkCommit {
            version: 1,
            container_id: "test".to_string(),
            expected_sequence: 1,
            previous_hash: "prev".to_string(),
            atom_hash: "atom".to_string(),
            intent_class: IntentClass::Conservation,
            physics_delta: 100_000_000_000_000_000_i128, // Large number > 2^53
            author_pubkey: "pk".to_string(),
            signature: "sig".to_string(),
            pact: None,
        };

        let json = serde_json::to_string(&commit).unwrap();
        
        // Verify it's serialized as string, not number
        assert!(json.contains("\"physics_delta\":\"100000000000000000\""));
        
        // Verify deserialization works
        let parsed: LinkCommit = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.physics_delta, 100_000_000_000_000_000_i128);
    }
}
