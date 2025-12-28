//! Cryptographic Identity
//!
//! Ed25519 keypair management for entity signing and verification.

use ed25519_dalek::{SigningKey, VerifyingKey, Signer, Verifier, Signature};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};

use crate::{OfficeError, Result};

/// Keypair for signing and verification
#[derive(Clone)]
pub struct KeyPair {
    signing_key: SigningKey,
    verifying_key: VerifyingKey,
}

impl KeyPair {
    /// Generate a new random keypair
    pub fn generate() -> Self {
        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key = signing_key.verifying_key();
        Self { signing_key, verifying_key }
    }

    /// Create from seed bytes
    pub fn from_seed(seed: &[u8; 32]) -> Self {
        let signing_key = SigningKey::from_bytes(seed);
        let verifying_key = signing_key.verifying_key();
        Self { signing_key, verifying_key }
    }

    /// Get public key as hex string
    pub fn public_key_hex(&self) -> String {
        hex::encode(self.verifying_key.as_bytes())
    }

    /// Get secret key as hex string (use carefully!)
    pub fn secret_key_hex(&self) -> String {
        hex::encode(self.signing_key.to_bytes())
    }

    /// Sign a message
    pub fn sign(&self, message: &[u8]) -> String {
        let signature = self.signing_key.sign(message);
        hex::encode(signature.to_bytes())
    }

    /// Verify a signature
    pub fn verify(&self, message: &[u8], signature_hex: &str) -> Result<()> {
        let sig_bytes = hex::decode(signature_hex)
            .map_err(|e| OfficeError::CryptoError(format!("Invalid signature hex: {}", e)))?;

        let signature = Signature::from_slice(&sig_bytes)
            .map_err(|e| OfficeError::CryptoError(format!("Invalid signature: {}", e)))?;

        self.verifying_key
            .verify(message, &signature)
            .map_err(|e| OfficeError::CryptoError(format!("Signature verification failed: {}", e)))
    }
}

/// Serializable identity (public key + encrypted private key reference)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identity {
    /// Public key (hex encoded)
    pub public_key_hex: String,
    /// Key version (for rotation)
    pub key_version: u32,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Private key reference (encrypted or vault reference)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private_key_ref: Option<String>,
}

impl Identity {
    /// Generate a new identity with fresh keypair
    pub fn generate() -> Result<Self> {
        let keypair = KeyPair::generate();
        Ok(Self {
            public_key_hex: keypair.public_key_hex(),
            key_version: 1,
            created_at: chrono::Utc::now(),
            // In production, this would be encrypted or stored in a vault
            private_key_ref: Some(keypair.secret_key_hex()),
        })
    }

    /// Get the signing keypair (requires private key access)
    pub fn get_keypair(&self) -> Result<KeyPair> {
        let secret_hex = self.private_key_ref.as_ref()
            .ok_or_else(|| OfficeError::CryptoError("Private key not available".to_string()))?;

        let secret_bytes = hex::decode(secret_hex)
            .map_err(|e| OfficeError::CryptoError(format!("Invalid secret key hex: {}", e)))?;

        let seed: [u8; 32] = secret_bytes.try_into()
            .map_err(|_| OfficeError::CryptoError("Invalid secret key length".to_string()))?;

        Ok(KeyPair::from_seed(&seed))
    }

    /// Sign a message using this identity
    pub fn sign(&self, message: &[u8]) -> Result<String> {
        let keypair = self.get_keypair()?;
        Ok(keypair.sign(message))
    }

    /// Create verifying-only identity (no private key)
    pub fn verifying_only(public_key_hex: String) -> Self {
        Self {
            public_key_hex,
            key_version: 1,
            created_at: chrono::Utc::now(),
            private_key_ref: None,
        }
    }
}

/// Verify a signature using a public key hex
pub fn verify_signature(public_key_hex: &str, message: &[u8], signature_hex: &str) -> Result<()> {
    let pubkey_bytes = hex::decode(public_key_hex)
        .map_err(|e| OfficeError::CryptoError(format!("Invalid public key hex: {}", e)))?;

    let pubkey_array: [u8; 32] = pubkey_bytes.try_into()
        .map_err(|_| OfficeError::CryptoError("Invalid public key length".to_string()))?;

    let verifying_key = VerifyingKey::from_bytes(&pubkey_array)
        .map_err(|e| OfficeError::CryptoError(format!("Invalid public key: {}", e)))?;

    let sig_bytes = hex::decode(signature_hex)
        .map_err(|e| OfficeError::CryptoError(format!("Invalid signature hex: {}", e)))?;

    let signature = Signature::from_slice(&sig_bytes)
        .map_err(|e| OfficeError::CryptoError(format!("Invalid signature: {}", e)))?;

    verifying_key
        .verify(message, &signature)
        .map_err(|e| OfficeError::CryptoError(format!("Signature verification failed: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keypair_generation() {
        let keypair = KeyPair::generate();
        assert_eq!(keypair.public_key_hex().len(), 64); // 32 bytes = 64 hex chars
    }

    #[test]
    fn test_sign_and_verify() {
        let keypair = KeyPair::generate();
        let message = b"Hello, UBL!";
        let signature = keypair.sign(message);

        assert!(keypair.verify(message, &signature).is_ok());
    }

    #[test]
    fn test_identity_generation() {
        let identity = Identity::generate().unwrap();
        assert_eq!(identity.key_version, 1);
        assert!(identity.private_key_ref.is_some());
    }

    #[test]
    fn test_identity_signing() {
        let identity = Identity::generate().unwrap();
        let message = b"Test message";
        let signature = identity.sign(message).unwrap();

        // Verify using the public key
        verify_signature(&identity.public_key_hex, message, &signature).unwrap();
    }
}
