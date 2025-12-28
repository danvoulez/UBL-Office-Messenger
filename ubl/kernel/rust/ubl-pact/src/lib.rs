//! # UBL Pact
//!
//! **Title:** SPEC-UBL-PACT v1.0  
//! **Status:** NORMATIVE  
//! **Change-Control:** STRICT (no retroactive changes)  
//! **Freeze-Date:** 2025-12-25  
//! **Governed by:** SPEC-UBL-CORE v1.0
//!
//! Authority, Consensus and Risk management for UBL commits.
//! A pact defines who can authorize a link and under what conditions.

#![deny(unsafe_code)]
#![warn(missing_docs)]

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use thiserror::Error;

/// Risk levels per SPEC-UBL-PACT §6
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RiskLevel {
    /// L0: Pure observation
    L0 = 0,
    /// L1: Low impact
    L1 = 1,
    /// L2: Local impact
    L2 = 2,
    /// L3: Financial impact
    L3 = 3,
    /// L4: Systemic impact
    L4 = 4,
    /// L5: Sovereignty / Evolution
    L5 = 5,
}

/// Pact scope per SPEC-UBL-PACT §5
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PactScope {
    /// Valid for a single container
    Container(String),
    /// Valid for a namespace of containers
    Namespace(String),
    /// Valid system-wide
    Global,
}

/// Time window for pact validity per SPEC-UBL-PACT §7
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeWindow {
    /// Earliest valid time (Unix timestamp ms)
    pub not_before: i64,
    /// Latest valid time (Unix timestamp ms)
    pub not_after: i64,
}

impl TimeWindow {
    /// Check if a timestamp is within the window
    pub fn contains(&self, timestamp_ms: i64) -> bool {
        timestamp_ms >= self.not_before && timestamp_ms <= self.not_after
    }
}

/// Full Pact definition per SPEC-UBL-PACT §4
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pact {
    /// Unique pact identifier
    pub pact_id: String,
    /// Protocol version
    pub version: u8,
    /// Scope of application
    pub scope: PactScope,
    /// Intent classes this pact governs
    pub intent_classes: Vec<IntentClassRef>,
    /// Minimum signatures required
    pub threshold: u8,
    /// Authorized signers (public keys hex)
    pub signers: HashSet<String>,
    /// Validity window
    pub window: TimeWindow,
    /// Risk level
    pub risk_level: RiskLevel,
}

/// Reference to an intent class (for pact rules)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IntentClassRef {
    /// Observation intent
    Observation,
    /// Conservation intent
    Conservation,
    /// Entropy intent
    Entropy,
    /// Evolution intent
    Evolution,
}

/// Pact validation errors per SPEC-UBL-PACT §11
#[derive(Error, Debug, Clone)]
pub enum PactError {
    /// Pact ID not found in registry
    #[error("Unknown pact: {0}")]
    UnknownPact(String),
    
    /// Pact has expired or not yet valid
    #[error("Pact expired or not yet valid")]
    PactExpired,
    
    /// Not enough valid signatures
    #[error("Insufficient signatures: got {got}, need {need}")]
    InsufficientSignatures { got: usize, need: u8 },
    
    /// Signer not in authorized set
    #[error("Unauthorized signer: {0}")]
    UnauthorizedSigner(String),
    
    /// Intent class doesn't match pact risk level
    #[error("Risk mismatch: pact is {pact_level:?}, intent requires {required:?}")]
    RiskMismatch { pact_level: RiskLevel, required: RiskLevel },
    
    /// Duplicate signature detected
    #[error("Duplicate signature from: {0}")]
    DuplicateSignature(String),
    
    /// Invalid signature
    #[error("Invalid signature from: {0}")]
    InvalidSignature(String),
}

/// Result type for pact operations
pub type Result<T> = std::result::Result<T, PactError>;

/// Pact proof attached to a link (SPEC-UBL-PACT §8)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PactProof {
    /// Reference to the pact
    pub pact_id: String,
    /// Collected signatures
    pub signatures: Vec<PactSignature>,
}

/// A signature in a pact proof
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PactSignature {
    /// Signer's public key (hex)
    pub signer: String,
    /// Signature (hex)
    pub signature: String,
}

/// Validate a pact proof against a pact definition
/// 
/// # Arguments
/// * `pact` - The pact definition
/// * `proof` - The proof attached to the link
/// * `atom_hash` - Hash of the atom being committed
/// * `intent_class` - The intent class of the commit
/// * `physics_delta` - The physics delta
/// * `current_time_ms` - Current timestamp for window check
pub fn validate_pact(
    pact: &Pact,
    proof: &PactProof,
    atom_hash: &str,
    intent_class: &IntentClassRef,
    physics_delta: i128,
    current_time_ms: i64,
) -> Result<()> {
    // 1. Check pact_id matches
    if proof.pact_id != pact.pact_id {
        return Err(PactError::UnknownPact(proof.pact_id.clone()));
    }

    // 2. Check time window
    if !pact.window.contains(current_time_ms) {
        return Err(PactError::PactExpired);
    }

    // 3. Check intent class is governed by this pact
    if !pact.intent_classes.contains(intent_class) {
        let required = match intent_class {
            IntentClassRef::Observation => RiskLevel::L0,
            IntentClassRef::Conservation => RiskLevel::L2,
            IntentClassRef::Entropy => RiskLevel::L4,
            IntentClassRef::Evolution => RiskLevel::L5,
        };
        return Err(PactError::RiskMismatch {
            pact_level: pact.risk_level,
            required,
        });
    }

    // 4. Collect valid signatures
    let mut valid_signers: HashSet<String> = HashSet::new();
    
    // Build the message that should be signed
    let sign_message = build_pact_sign_message(&pact.pact_id, atom_hash, intent_class, physics_delta);
    
    for sig in &proof.signatures {
        // Check for duplicates
        if valid_signers.contains(&sig.signer) {
            return Err(PactError::DuplicateSignature(sig.signer.clone()));
        }
        
        // Check signer is authorized
        if !pact.signers.contains(&sig.signer) {
            return Err(PactError::UnauthorizedSigner(sig.signer.clone()));
        }
        
        // Verify signature
        if ubl_kernel::verify(&sig.signer, &sign_message, &sig.signature).is_err() {
            return Err(PactError::InvalidSignature(sig.signer.clone()));
        }
        
        valid_signers.insert(sig.signer.clone());
    }

    // 5. Check threshold
    if valid_signers.len() < pact.threshold as usize {
        return Err(PactError::InsufficientSignatures {
            got: valid_signers.len(),
            need: pact.threshold,
        });
    }

    Ok(())
}

/// Build the message that pact signers must sign
/// Per SPEC-UBL-PACT §8.1
fn build_pact_sign_message(
    pact_id: &str,
    atom_hash: &str,
    intent_class: &IntentClassRef,
    physics_delta: i128,
) -> Vec<u8> {
    let mut message = Vec::new();
    
    // Domain tag
    message.extend_from_slice(b"ubl:pact\n");
    
    // Pact ID
    message.extend_from_slice(pact_id.as_bytes());
    
    // Atom hash
    message.extend_from_slice(atom_hash.as_bytes());
    
    // Intent class (1 byte)
    let class_byte = match intent_class {
        IntentClassRef::Observation => 0x00,
        IntentClassRef::Conservation => 0x01,
        IntentClassRef::Entropy => 0x02,
        IntentClassRef::Evolution => 0x03,
    };
    message.push(class_byte);
    
    // Physics delta (16 bytes, big-endian)
    message.extend_from_slice(&physics_delta.to_be_bytes());
    
    message
}

/// Simple in-memory pact registry
#[derive(Debug, Default)]
pub struct PactRegistry {
    pacts: std::collections::HashMap<String, Pact>,
}

impl PactRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a pact
    pub fn register(&mut self, pact: Pact) {
        self.pacts.insert(pact.pact_id.clone(), pact);
    }

    /// Get a pact by ID
    pub fn get(&self, pact_id: &str) -> Option<&Pact> {
        self.pacts.get(pact_id)
    }

    /// Validate a proof against a registered pact
    pub fn validate(
        &self,
        proof: &PactProof,
        atom_hash: &str,
        intent_class: &IntentClassRef,
        physics_delta: i128,
        current_time_ms: i64,
    ) -> Result<()> {
        let pact = self.get(&proof.pact_id)
            .ok_or_else(|| PactError::UnknownPact(proof.pact_id.clone()))?;
        
        validate_pact(pact, proof, atom_hash, intent_class, physics_delta, current_time_ms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_pact() -> Pact {
        let mut signers = HashSet::new();
        signers.insert("signer1_pubkey".to_string());
        signers.insert("signer2_pubkey".to_string());
        signers.insert("signer3_pubkey".to_string());

        Pact {
            pact_id: "test_pact_001".to_string(),
            version: 1,
            scope: PactScope::Global,
            intent_classes: vec![IntentClassRef::Entropy, IntentClassRef::Evolution],
            threshold: 2,
            signers,
            window: TimeWindow {
                not_before: 0,
                not_after: i64::MAX,
            },
            risk_level: RiskLevel::L5,
        }
    }

    #[test]
    fn test_time_window() {
        let window = TimeWindow {
            not_before: 1000,
            not_after: 2000,
        };
        
        assert!(!window.contains(999));
        assert!(window.contains(1000));
        assert!(window.contains(1500));
        assert!(window.contains(2000));
        assert!(!window.contains(2001));
    }

    #[test]
    fn test_pact_registry() {
        let mut registry = PactRegistry::new();
        let pact = make_test_pact();
        
        registry.register(pact);
        
        assert!(registry.get("test_pact_001").is_some());
        assert!(registry.get("nonexistent").is_none());
    }

    #[test]
    fn test_risk_level_ordering() {
        assert!(RiskLevel::L0 < RiskLevel::L1);
        assert!(RiskLevel::L4 < RiskLevel::L5);
        assert!(RiskLevel::L5 > RiskLevel::L0);
    }
}
