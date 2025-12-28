//! Cryptographic helpers for Console API
//!
//! Ed25519 signing/verification, random bytes, UUID generation.

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use blake3::Hasher;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use rand::RngCore;
use sqlx::PgPool;

// =============================================================================
// RANDOM & UUID
// =============================================================================

pub fn rand_bytes_16() -> [u8; 16] {
    let mut b = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut b);
    b
}

pub fn rand_bytes_32() -> [u8; 32] {
    let mut b = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut b);
    b
}

pub fn uuid_v4() -> String {
    uuid::Uuid::new_v4().to_string()
}

// =============================================================================
// ADMIN SIGNING KEY (Console Permits)
// =============================================================================
// Gemini P0 #1: Uses persistent keystore instead of ephemeral env vars

/// Get the admin public key (hex-encoded)
pub fn admin_pubkey_hex() -> String {
    crate::keystore::get_public_key_hex("admin")
}

/// Sign a message with the admin key
pub fn sign_admin_permit(msg: &[u8]) -> Vec<u8> {
    let sig_tagged = crate::keystore::sign("admin", msg);
    // Extract raw signature bytes from "ed25519:<b64>"
    let b64 = sig_tagged.trim_start_matches("ed25519:");
    URL_SAFE_NO_PAD.decode(b64).unwrap_or_default()
}

/// Verify an admin permit signature
/// Format: "ed25519:<base64url_signature>"
pub fn verify_admin_permit_sig(msg: &[u8], sig_tagged: &str) -> Result<(), &'static str> {
    let pubkey_hex = admin_pubkey_hex();
    crate::keystore::verify(&pubkey_hex, msg, sig_tagged)
        .map_err(|_| "SignatureVerifyFailed")
}

// =============================================================================
// RUNNER SIGNATURE VERIFICATION
// =============================================================================

/// Verify a runner's signature on a receipt.
/// Looks up the runner's public key in the database.
pub async fn verify_runner_sig(
    pool: &PgPool,
    runner_id: &str,
    msg: &[u8],
    sig_tagged: &str,
) -> Result<(), String> {
    // 1. Parse signature
    if !sig_tagged.starts_with("ed25519:") {
        return Err("InvalidSignatureFormat".into());
    }
    let b64 = sig_tagged.trim_start_matches("ed25519:");
    let sig_bytes = URL_SAFE_NO_PAD.decode(b64).map_err(|_| "InvalidBase64".to_string())?;
    let sig_array: [u8; 64] = sig_bytes.try_into().map_err(|_| "InvalidSignatureLength".to_string())?;
    let sig = Signature::from_bytes(&sig_array);

    // 2. Look up runner's public key
    let row = sqlx::query(
        "SELECT pubkey_ed25519, is_active FROM ubl_runners WHERE runner_id = $1"
    )
    .bind(runner_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?
    .ok_or("RunnerNotFound")?;

    let pubkey_hex: String = row.get("pubkey_ed25519");
    let is_active: bool = row.get("is_active");

    if !is_active {
        return Err("RunnerNotActive".into());
    }

    // 3. Verify
    let pubkey_bytes = hex::decode(&pubkey_hex).map_err(|_| "InvalidPubkeyHex".to_string())?;
    let pubkey_array: [u8; 32] = pubkey_bytes.try_into().map_err(|_| "InvalidPubkeyLength".to_string())?;
    let pubkey = VerifyingKey::from_bytes(&pubkey_array).map_err(|_| "InvalidPubkey".to_string())?;

    pubkey.verify(msg, &sig).map_err(|_| "SignatureVerifyFailed".to_string())
}

// =============================================================================
// PERMIT BINDING HASH
// =============================================================================

/// Compute the binding hash for a Permit.
/// This is the cryptographic commitment that binds the Permit to its parameters.
pub fn permit_binding_hash(
    office: &str,
    action: &str,
    target: &str,
    args: &serde_json::Value,
    risk: &str,
    plan_hash: &str,
    nonce_b64: &str,
    exp_ms: i64,
) -> String {
    let mut h = Hasher::new();
    h.update(office.as_bytes());
    h.update(b"\n");
    h.update(action.as_bytes());
    h.update(b"\n");
    h.update(target.as_bytes());
    h.update(b"\n");
    h.update(serde_json::to_string(args).unwrap_or_default().as_bytes());
    h.update(b"\n");
    h.update(risk.as_bytes());
    h.update(b"\n");
    h.update(plan_hash.as_bytes());
    h.update(b"\n");
    h.update(nonce_b64.as_bytes());
    h.update(b"\n");
    h.update(exp_ms.to_string().as_bytes());
    format!("blake3:{}", h.finalize().to_hex())
}

/// Compute the canonical hash of a plan JSON
pub fn canonical_plan_hash(plan_json: &serde_json::Value) -> String {
    // Use ubl_atom canonicalization if available
    let bytes = match crate::ubl_atom_compat::canonicalize(plan_json) {
        Ok(b) => b,
        Err(_) => serde_json::to_vec(plan_json).unwrap_or_default(),
    };
    format!("blake3:{}", blake3::hash(&bytes).to_hex())
}

// =============================================================================
// UBL-ATOM COMPATIBILITY
// =============================================================================

pub mod ubl_atom_compat {
    /// Canonicalize JSON for hashing (placeholder - use ubl-atom crate in prod)
    pub fn canonicalize(value: &serde_json::Value) -> Result<Vec<u8>, String> {
        // Simple canonical form: sorted keys, no whitespace
        // In production, use the full ubl-atom crate
        let s = canonical_json(value);
        Ok(s.into_bytes())
    }

    fn canonical_json(value: &serde_json::Value) -> String {
        match value {
            serde_json::Value::Object(map) => {
                let mut keys: Vec<_> = map.keys().collect();
                keys.sort();
                let pairs: Vec<String> = keys
                    .iter()
                    .map(|k| format!("\"{}\":{}", k, canonical_json(&map[*k])))
                    .collect();
                format!("{{{}}}", pairs.join(","))
            }
            serde_json::Value::Array(arr) => {
                let items: Vec<String> = arr.iter().map(canonical_json).collect();
                format!("[{}]", items.join(","))
            }
            serde_json::Value::String(s) => format!("\"{}\"", escape_json_string(s)),
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::Bool(b) => b.to_string(),
            serde_json::Value::Null => "null".to_string(),
        }
    }

    fn escape_json_string(s: &str) -> String {
        s.replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\t', "\\t")
    }
}

// =============================================================================
// sqlx::Row helper
// =============================================================================

use sqlx::Row;

trait RowExt {
    fn get<T: sqlx::Decode<'static, sqlx::Postgres> + sqlx::Type<sqlx::Postgres>>(&self, col: &str) -> T;
}

impl RowExt for sqlx::postgres::PgRow {
    fn get<T: sqlx::Decode<'static, sqlx::Postgres> + sqlx::Type<sqlx::Postgres>>(&self, col: &str) -> T {
        sqlx::Row::get(self, col)
    }
}

