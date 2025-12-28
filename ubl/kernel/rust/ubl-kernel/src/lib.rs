//! # UBL Kernel
//!
//! Pure cryptography. Mathematically closed. Semantically blind.
//! Implements SPEC-UBL-KERNEL
//!
//! ## Features
//! - BLAKE3 hashing with domain separation
//! - Ed25519 signing and verification
//! - Deterministic operations only

#![deny(unsafe_code)]
#![warn(missing_docs)]

use blake3::Hasher;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use thiserror::Error;

/// Domain prefixes for hash separation
/// NOTE: atom_hash does NOT use domain tag per JSON✯Atomic binding
pub mod domains {
    /// Domain for link hashing
    pub const LINK: &[u8] = b"ubl:link\n";
    /// Domain for ledger entry hashing
    pub const LEDGER: &[u8] = b"ubl:ledger\n";
    /// Domain for merkle root
    pub const ROOT: &[u8] = b"ubl:root\n";
}

/// Errors from kernel operations
#[derive(Error, Debug)]
pub enum KernelError {
    /// Invalid hex string
    #[error("Invalid hex: {0}")]
    InvalidHex(#[from] hex::FromHexError),
    
    /// Invalid signature
    #[error("Signature verification failed")]
    SignatureVerification,
    
    /// Invalid key format
    #[error("Invalid key format: {0}")]
    InvalidKey(String),
}

/// Result type for kernel operations
pub type Result<T> = std::result::Result<T, KernelError>;

/// Hash an atom (canonical JSON bytes) - NO domain tag per JSON✯Atomic binding
/// atom_hash is EXACTLY the hash that JSON✯Atomic v1.0 produces
pub fn hash_atom(canonical_bytes: &[u8]) -> String {
    let mut hasher = Hasher::new();
    // NO domain tag for atom - must match JSON✯Atomic v1.0 exactly
    hasher.update(canonical_bytes);
    hex::encode(hasher.finalize().as_bytes())
}

/// Hash a link commit with domain separation
pub fn hash_link(signing_bytes: &[u8]) -> String {
    let mut hasher = Hasher::new();
    hasher.update(domains::LINK);
    hasher.update(signing_bytes);
    hex::encode(hasher.finalize().as_bytes())
}

/// Hash for merkle tree nodes
pub fn hash_merkle(left: &[u8], right: &[u8]) -> Vec<u8> {
    let mut hasher = Hasher::new();
    hasher.update(domains::ROOT);
    hasher.update(left);
    hasher.update(right);
    hasher.finalize().as_bytes().to_vec()
}

/// Sign data with Ed25519
pub fn sign(signing_key: &SigningKey, message: &[u8]) -> String {
    let signature = signing_key.sign(message);
    hex::encode(signature.to_bytes())
}

/// Verify an Ed25519 signature
pub fn verify(pubkey_hex: &str, message: &[u8], signature_hex: &str) -> Result<()> {
    // Decode public key
    let pubkey_bytes = hex::decode(pubkey_hex)?;
    let verifying_key = VerifyingKey::try_from(pubkey_bytes.as_slice())
        .map_err(|e| KernelError::InvalidKey(e.to_string()))?;
    
    // Decode signature
    let sig_bytes = hex::decode(signature_hex)?;
    let signature = Signature::try_from(sig_bytes.as_slice())
        .map_err(|e| KernelError::InvalidKey(e.to_string()))?;
    
    // Verify
    verifying_key
        .verify(message, &signature)
        .map_err(|_| KernelError::SignatureVerification)?;
    
    Ok(())
}

/// Generate a new signing keypair
pub fn generate_keypair() -> (String, SigningKey) {
    let signing_key = SigningKey::generate(&mut rand::thread_rng());
    let pubkey_hex = hex::encode(signing_key.verifying_key().as_bytes());
    (pubkey_hex, signing_key)
}

/// Get the public key hex from a signing key
pub fn pubkey_from_signing_key(signing_key: &SigningKey) -> String {
    hex::encode(signing_key.verifying_key().as_bytes())
}

/// The genesis hash (32 zero bytes)
pub const GENESIS_HASH: &str = "0000000000000000000000000000000000000000000000000000000000000000";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_atom_deterministic() {
        let data = b"test data";
        let hash1 = hash_atom(data);
        let hash2 = hash_atom(data);
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64); // 32 bytes = 64 hex chars
    }

    #[test]
    fn test_hash_domain_separation() {
        let data = b"same data";
        let atom_hash = hash_atom(data);
        let link_hash = hash_link(data);
        // atom_hash has NO domain tag, link_hash has "ubl:link\n"
        assert_ne!(atom_hash, link_hash);
    }
    
    #[test]
    fn test_atom_hash_no_domain_tag() {
        // Verify atom_hash matches raw BLAKE3 (no domain tag)
        let data = b"test";
        let atom_hash = hash_atom(data);
        let raw_blake3 = hex::encode(blake3::hash(data).as_bytes());
        assert_eq!(atom_hash, raw_blake3, "atom_hash must match raw BLAKE3 (JSON✯Atomic binding)");
    }

    #[test]
    fn test_sign_and_verify() {
        let (pubkey, signing_key) = generate_keypair();
        let message = b"hello world";
        
        let signature = sign(&signing_key, message);
        let result = verify(&pubkey, message, &signature);
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_wrong_message() {
        let (pubkey, signing_key) = generate_keypair();
        let message = b"hello world";
        let wrong_message = b"wrong message";
        
        let signature = sign(&signing_key, message);
        let result = verify(&pubkey, wrong_message, &signature);
        
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_wrong_key() {
        let (_, signing_key1) = generate_keypair();
        let (pubkey2, _) = generate_keypair();
        let message = b"hello world";
        
        let signature = sign(&signing_key1, message);
        let result = verify(&pubkey2, message, &signature);
        
        assert!(result.is_err());
    }

    #[test]
    fn test_genesis_hash_length() {
        assert_eq!(GENESIS_HASH.len(), 64);
    }
}
