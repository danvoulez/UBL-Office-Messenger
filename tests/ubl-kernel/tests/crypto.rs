//! Cryptographic primitives tests
//! SPEC-UBL-KERNEL v1.0 compliance

use ubl_kernel: :{sign, verify, hash, generate_keypair};
use hex;

#[test]
fn test_keypair_generation() {
    let keypair = generate_keypair();
    assert_eq!(keypair.public. len(), 32);
    assert_eq!(keypair.secret.len(), 64);
}

#[test]
fn test_sign_and_verify() {
    let keypair = generate_keypair();
    let message = b"Hello, UBL! ";
    
    let signature = sign(&keypair, message);
    assert_eq!(signature.len(), 64);
    
    let valid = verify(&keypair. public, message, &signature);
    assert!(valid);
}

#[test]
fn test_verify_wrong_message() {
    let keypair = generate_keypair();
    let message = b"Hello, UBL!";
    let wrong_message = b"Wrong message";
    
    let signature = sign(&keypair, message);
    
    let valid = verify(&keypair.public, wrong_message, &signature);
    assert!(!valid);
}

#[test]
fn test_verify_wrong_signature() {
    let keypair = generate_keypair();
    let message = b"Hello, UBL!";
    
    let wrong_signature = [0u8; 64];
    
    let valid = verify(&keypair.public, message, &wrong_signature);
    assert!(!valid);
}

#[test]
fn test_verify_tampered_signature() {
    let keypair = generate_keypair();
    let message = b"Hello, UBL!";
    
    let mut signature = sign(&keypair, message);
    signature[0] ^= 0xFF; // Flip bits
    
    let valid = verify(&keypair.public, message, &signature);
    assert!(!valid);
}

#[test]
fn test_hash_deterministic() {
    let data = b"test data";
    
    let hash1 = hash(data);
    let hash2 = hash(data);
    
    assert_eq!(hash1, hash2);
    assert_eq!(hash1.len(), 32); // BLAKE3 outputs 32 bytes
}

#[test]
fn test_hash_different_inputs() {
    let data1 = b"test data 1";
    let data2 = b"test data 2";
    
    let hash1 = hash(data1);
    let hash2 = hash(data2);
    
    assert_ne!(hash1, hash2);
}

#[test]
fn test_hash_empty_input() {
    let data = b"";
    let result = hash(data);
    
    assert_eq!(result. len(), 32);
    // BLAKE3 hash of empty input is known
    let expected = "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262";
    assert_eq!(hex::encode(result), expected);
}

#[test]
fn test_signature_uniqueness() {
    let keypair1 = generate_keypair();
    let keypair2 = generate_keypair();
    
    let message = b"Same message";
    
    let sig1 = sign(&keypair1, message);
    let sig2 = sign(&keypair2, message);
    
    assert_ne!(sig1, sig2);
}

#[test]
fn test_public_key_hex_format() {
    let keypair = generate_keypair();
    let hex_public = hex::encode(keypair.public);
    
    assert_eq!(hex_public. len(), 64); // 32 bytes = 64 hex chars
    assert!(hex_public.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn test_hash_collision_resistance() {
    // Test that similar inputs produce different hashes
    let data1 = b"test";
    let data2 = b"Test"; // Only capitalization different
    
    let hash1 = hash(data1);
    let hash2 = hash(data2);
    
    assert_ne!(hash1, hash2);
}

#[test]
fn test_sign_large_message() {
    let keypair = generate_keypair();
    let large_message = vec![0u8; 1_000_000]; // 1MB
    
    let signature = sign(&keypair, &large_message);
    let valid = verify(&keypair.public, &large_message, &signature);
    
    assert!(valid);
}

#[test]
fn test_keypair_independence() {
    let keypair1 = generate_keypair();
    let keypair2 = generate_keypair();
    
    assert_ne!(keypair1.public, keypair2.public);
    assert_ne!(keypair1.secret, keypair2.secret);
}