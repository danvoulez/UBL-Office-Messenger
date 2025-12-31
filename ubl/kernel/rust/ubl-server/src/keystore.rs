//! Persistent KeyStore for Ed25519 Signing Keys
//!
//! Gemini P0 #1: Keys must persist across restarts.
//! Without this, agents lose their identity when the container restarts.
//!
//! Storage options:
//! 1. File-based (default): ~/.ubl/keys/<key_id>.key
//! 2. Environment variable: UBL_KEY_<KEY_ID>=<hex>
//! 3. Future: HashiCorp Vault, AWS KMS

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use ed25519_dalek::{SigningKey, VerifyingKey, Signature, Signer, Verifier};
use rand::rngs::OsRng;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::RwLock;
use tracing::{info, warn, error};

/// Key storage directory
fn keys_dir() -> PathBuf {
    let base = std::env::var("UBL_KEYS_DIR")
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
            format!("{}/.ubl/keys", home)
        });
    PathBuf::from(base)
}

/// In-memory cache of loaded keys
static KEY_CACHE: RwLock<Option<HashMap<String, SigningKey>>> = RwLock::new(None);

/// Initialize the keystore
pub fn init() {
    let dir = keys_dir();
    if !dir.exists() {
        if let Err(e) = fs::create_dir_all(&dir) {
            error!("Failed to create keys directory {:?}: {}", dir, e);
        } else {
            info!("Created keys directory: {:?}", dir);
            // Set restrictive permissions on Unix
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = fs::set_permissions(&dir, fs::Permissions::from_mode(0o700));
            }
        }
    }
    
    // Fix #15: Use expect for clearer panic on lock poisoning
    let mut cache = KEY_CACHE.write().expect("KeyStore lock poisoned");
    *cache = Some(HashMap::new());
    
    info!("KeyStore initialized at {:?}", dir);
}

/// Load or create a signing key by ID
/// 
/// Priority:
/// 1. Memory cache
/// 2. Environment variable: UBL_KEY_<KEY_ID>=<hex>
/// 3. File: <keys_dir>/<key_id>.key
/// 4. Generate new and save to file
pub fn load_or_create(key_id: &str) -> SigningKey {
    // Check cache first
    {
        let cache = KEY_CACHE.read().expect("KeyStore lock poisoned");
        if let Some(ref map) = *cache {
            if let Some(key) = map.get(key_id) {
                return key.clone();
            }
        }
    }
    
    // Try environment variable
    let env_key = format!("UBL_KEY_{}", key_id.to_uppercase().replace('-', "_"));
    if let Ok(hex) = std::env::var(&env_key) {
        if let Ok(bytes) = hex::decode(hex.trim()) {
            if bytes.len() == 32 {
                // Fix #15: Safe because we just checked len == 32
                let key_bytes: [u8; 32] = bytes.try_into().expect("length already verified");
                let key = SigningKey::from_bytes(&key_bytes);
                cache_key(key_id, key.clone());
                info!("Loaded key '{}' from environment", key_id);
                return key;
            }
        }
        warn!("Invalid key in {}, ignoring", env_key);
    }
    
    // Try file
    let key_path = keys_dir().join(format!("{}.key", key_id));
    if key_path.exists() {
        match fs::read_to_string(&key_path) {
            Ok(hex) => {
                if let Ok(bytes) = hex::decode(hex.trim()) {
                    if bytes.len() == 32 {
                        // Fix #15: Safe because we just checked len == 32
                        let key_bytes: [u8; 32] = bytes.try_into().expect("length already verified");
                        let key = SigningKey::from_bytes(&key_bytes);
                        cache_key(key_id, key.clone());
                        info!("Loaded key '{}' from {:?}", key_id, key_path);
                        return key;
                    }
                }
                warn!("Invalid key file {:?}, regenerating", key_path);
            }
            Err(e) => {
                warn!("Could not read {:?}: {}", key_path, e);
            }
        }
    }
    
    // Generate new key
    let key = SigningKey::generate(&mut OsRng);
    
    // Save to file
    let hex = hex::encode(key.to_bytes());
    if let Err(e) = fs::write(&key_path, &hex) {
        error!("Failed to save key to {:?}: {}", key_path, e);
    } else {
        // Set restrictive permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&key_path, fs::Permissions::from_mode(0o600));
        }
        info!("Generated and saved new key '{}' to {:?}", key_id, key_path);
    }
    
    // Log public key for registration
    let pubkey = key.verifying_key();
    info!("Public key for '{}': {}", key_id, hex::encode(pubkey.as_bytes()));
    
    cache_key(key_id, key.clone());
    key
}

fn cache_key(key_id: &str, key: SigningKey) {
    let mut cache = KEY_CACHE.write().unwrap();
    if let Some(ref mut map) = *cache {
        map.insert(key_id.to_string(), key);
    }
}

/// Get public key (hex) for a key ID
pub fn get_public_key_hex(key_id: &str) -> String {
    let key = load_or_create(key_id);
    hex::encode(key.verifying_key().as_bytes())
}

/// Sign data with a named key
pub fn sign(key_id: &str, data: &[u8]) -> String {
    let key = load_or_create(key_id);
    let sig = key.sign(data);
    format!("ed25519:{}", URL_SAFE_NO_PAD.encode(sig.to_bytes()))
}

/// Verify signature with a public key (hex)
pub fn verify(pubkey_hex: &str, data: &[u8], sig_tagged: &str) -> Result<(), String> {
    if !sig_tagged.starts_with("ed25519:") {
        return Err("Invalid signature format".into());
    }
    
    let sig_b64 = sig_tagged.trim_start_matches("ed25519:");
    let sig_bytes = URL_SAFE_NO_PAD.decode(sig_b64)
        .map_err(|e| format!("Invalid base64: {}", e))?;
    
    let sig_array: [u8; 64] = sig_bytes.try_into()
        .map_err(|_| "Invalid signature length")?;
    let sig = Signature::from_bytes(&sig_array);
    
    let pubkey_bytes = hex::decode(pubkey_hex)
        .map_err(|e| format!("Invalid pubkey hex: {}", e))?;
    let pubkey_array: [u8; 32] = pubkey_bytes.try_into()
        .map_err(|_| "Invalid pubkey length")?;
    let pubkey = VerifyingKey::from_bytes(&pubkey_array)
        .map_err(|e| format!("Invalid pubkey: {}", e))?;
    
    pubkey.verify(data, &sig)
        .map_err(|_| "Signature verification failed".to_string())
}

/// List all key IDs in the keystore
pub fn list_keys() -> Vec<String> {
    let dir = keys_dir();
    let mut keys = Vec::new();
    
    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                if name.ends_with(".key") {
                    keys.push(name.trim_end_matches(".key").to_string());
                }
            }
        }
    }
    
    keys
}

/// Delete a key (use with caution!)
pub fn delete_key(key_id: &str) -> Result<(), String> {
    let key_path = keys_dir().join(format!("{}.key", key_id));
    
    if key_path.exists() {
        fs::remove_file(&key_path).map_err(|e| e.to_string())?;
        
        // Remove from cache
        let mut cache = KEY_CACHE.write().unwrap();
        if let Some(ref mut map) = *cache {
            map.remove(key_id);
        }
        
        warn!("Deleted key '{}'", key_id);
        Ok(())
    } else {
        Err("Key not found".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_key_persistence() {
        init();
        
        let key1 = load_or_create("test-key");
        let key2 = load_or_create("test-key");
        
        // Same key should be returned
        assert_eq!(key1.to_bytes(), key2.to_bytes());
        
        // Cleanup
        let _ = delete_key("test-key");
    }
    
    #[test]
    fn test_sign_verify() {
        init();
        
        let key_id = "test-sign";
        let data = b"hello world";
        
        let sig = sign(key_id, data);
        let pubkey = get_public_key_hex(key_id);
        
        assert!(verify(&pubkey, data, &sig).is_ok());
        assert!(verify(&pubkey, b"wrong data", &sig).is_err());
        
        // Cleanup
        let _ = delete_key(key_id);
    }
}

