//! Pact database operations and validation
//! SPEC-UBL-PACT v1.0

use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashSet;
use tracing::{info, error};

/// Pact record from database
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PactRecord {
    pub pact_id: String,
    pub version: i16,
    pub scope_type: String,
    pub scope_value: Option<String>,
    pub intent_classes: Vec<String>,
    pub threshold: i16,
    pub signers: Vec<String>,
    pub not_before: i64,
    pub not_after: i64,
    pub risk_level: i16,
}

/// Pact proof from link commit
#[derive(Debug, Clone, Deserialize)]
pub struct PactProofInput {
    pub pact_id: String,
    pub signatures: Vec<PactSignatureInput>,
}

/// Signature in pact proof
#[derive(Debug, Clone, Deserialize)]
pub struct PactSignatureInput {
    pub signer: String,
    pub signature: String,
}

/// Pact validation error
#[derive(Debug)]
pub enum PactValidationError {
    UnknownPact(String),
    PactExpired,
    InsufficientSignatures { got: usize, need: i16 },
    UnauthorizedSigner(String),
    InvalidSignature(String),
    DuplicateSignature(String),
    IntentClassMismatch { pact: Vec<String>, got: String },
    DatabaseError(String),
}

impl std::fmt::Display for PactValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownPact(id) => write!(f, "Unknown pact: {}", id),
            Self::PactExpired => write!(f, "Pact expired or not yet valid"),
            Self::InsufficientSignatures { got, need } => {
                write!(f, "Insufficient signatures: got {}, need {}", got, need)
            }
            Self::UnauthorizedSigner(s) => write!(f, "Unauthorized signer: {}", s),
            Self::InvalidSignature(s) => write!(f, "Invalid signature from: {}", s),
            Self::DuplicateSignature(s) => write!(f, "Duplicate signature from: {}", s),
            Self::IntentClassMismatch { pact, got } => {
                write!(f, "Intent class {} not in pact {:?}", got, pact)
            }
            Self::DatabaseError(e) => write!(f, "Database error: {}", e),
        }
    }
}

/// Get a pact from database
pub async fn get_pact(pool: &PgPool, pact_id: &str) -> Result<Option<PactRecord>, sqlx::Error> {
    sqlx::query_as!(
        PactRecord,
        r#"
        SELECT pact_id, version, scope_type, scope_value, intent_classes,
               threshold, signers, not_before, not_after, risk_level
        FROM pact
        WHERE pact_id = $1
        "#,
        pact_id
    )
    .fetch_optional(pool)
    .await
}

/// Validate a pact proof
pub async fn validate_pact_proof(
    pool: &PgPool,
    proof: &PactProofInput,
    container_id: &str,
    intent_class: &str,
    atom_hash: &str,
    physics_delta: i128,
    current_time_ms: i64,
) -> Result<(), PactValidationError> {
    // 1. Fetch pact from DB
    let pact = get_pact(pool, &proof.pact_id)
        .await
        .map_err(|e| PactValidationError::DatabaseError(e.to_string()))?
        .ok_or_else(|| PactValidationError::UnknownPact(proof.pact_id.clone()))?;

    // 2. Check time window
    if current_time_ms < pact.not_before || current_time_ms > pact.not_after {
        return Err(PactValidationError::PactExpired);
    }

    // 3. Check scope
    match pact.scope_type.as_str() {
        "global" => { /* always applies */ }
        "container" => {
            if pact.scope_value.as_deref() != Some(container_id) {
                return Err(PactValidationError::UnknownPact(format!(
                    "Pact {} is scoped to container {}, not {}",
                    pact.pact_id,
                    pact.scope_value.as_deref().unwrap_or("?"),
                    container_id
                )));
            }
        }
        "namespace" => {
            let prefix = pact.scope_value.as_deref().unwrap_or("");
            if !container_id.starts_with(prefix) {
                return Err(PactValidationError::UnknownPact(format!(
                    "Pact {} is scoped to namespace {}, container {} doesn't match",
                    pact.pact_id, prefix, container_id
                )));
            }
        }
        _ => {
            return Err(PactValidationError::DatabaseError(format!(
                "Invalid scope_type: {}",
                pact.scope_type
            )));
        }
    }

    // 4. Check intent class is governed by this pact
    if !pact.intent_classes.contains(&intent_class.to_string()) {
        return Err(PactValidationError::IntentClassMismatch {
            pact: pact.intent_classes.clone(),
            got: intent_class.to_string(),
        });
    }

    // 5. Build sign message per SPEC-UBL-PACT §8.1
    let sign_message = build_pact_sign_message(
        &pact.pact_id,
        atom_hash,
        intent_class,
        physics_delta,
    );

    // 6. Validate signatures
    let signers_set: HashSet<&str> = pact.signers.iter().map(|s| s.as_str()).collect();
    let mut valid_signers: HashSet<String> = HashSet::new();

    for sig in &proof.signatures {
        // Check for duplicates
        if valid_signers.contains(&sig.signer) {
            return Err(PactValidationError::DuplicateSignature(sig.signer.clone()));
        }

        // Check signer is authorized
        if !signers_set.contains(sig.signer.as_str()) {
            return Err(PactValidationError::UnauthorizedSigner(sig.signer.clone()));
        }

        // Verify signature
        if ubl_kernel::verify(&sig.signer, &sign_message, &sig.signature).is_err() {
            return Err(PactValidationError::InvalidSignature(sig.signer.clone()));
        }

        valid_signers.insert(sig.signer.clone());
    }

    // 7. Check threshold
    if valid_signers.len() < pact.threshold as usize {
        return Err(PactValidationError::InsufficientSignatures {
            got: valid_signers.len(),
            need: pact.threshold,
        });
    }

    info!(
        "✅ Pact validated: {} ({}/{} signatures)",
        pact.pact_id,
        valid_signers.len(),
        pact.threshold
    );

    Ok(())
}

/// Build the message that pact signers must sign
/// Per SPEC-UBL-PACT §8.1
fn build_pact_sign_message(
    pact_id: &str,
    atom_hash: &str,
    intent_class: &str,
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
        "Observation" => 0x00,
        "Conservation" => 0x01,
        "Entropy" => 0x02,
        "Evolution" => 0x03,
        _ => 0xFF, // Unknown - validation should reject
    };
    message.push(class_byte);

    // Physics delta (16 bytes, big-endian)
    message.extend_from_slice(&physics_delta.to_be_bytes());

    message
}

/// Check if a pact is required for the given intent class and delta
pub fn requires_pact(intent_class: &str, physics_delta: i128) -> bool {
    match intent_class {
        "Observation" => false,
        "Conservation" => false,
        "Entropy" => physics_delta != 0,
        "Evolution" => true,
        _ => true, // Unknown - require pact
    }
}

/// High-level pact validation for commits
/// Returns Ok(()) if no pact is needed or if a valid pact exists
pub async fn validate_pact_for(
    pool: &PgPool,
    container_id: &str,
    intent_class: &str,
    atom_hash: &str,
    physics_delta: i128,
    now_ms: i64,
) -> Result<(), PactError> {
    // Check if pact is required
    if !requires_pact(intent_class, physics_delta) {
        return Ok(());
    }

    // Query for active pacts covering this container + intent
    let has_valid_pact: Option<bool> = sqlx::query_scalar(
        r#"SELECT EXISTS (
            SELECT 1 FROM pact
            WHERE (scope_type = 'global' 
                   OR (scope_type = 'container' AND scope_value = $1)
                   OR (scope_type = 'namespace' AND $1 LIKE scope_value || '%'))
              AND $2 = ANY(intent_classes)
              AND not_before <= $3 
              AND not_after >= $3
              AND (SELECT COUNT(*) FROM pact_signatures ps 
                   WHERE ps.pact_id = pact.pact_id 
                   AND ps.verified = true) >= threshold
        )"#
    )
    .bind(container_id)
    .bind(intent_class)
    .bind(now_ms)
    .fetch_one(pool)
    .await
    .map_err(|e| PactError::Internal(e.to_string()))?;

    if has_valid_pact == Some(true) {
        info!(
            "✅ Pact requirement satisfied for {} on container {}",
            intent_class, container_id
        );
        return Ok(());
    }

    error!(
        "❌ Pact required but not present for {} on container {}",
        intent_class, container_id
    );
    Err(PactError::Denied(format!(
        "Pact required for {} but no valid pact found for container {}",
        intent_class, container_id
    )))
}

/// Pact error type for high-level validation
#[derive(Debug)]
pub enum PactError {
    Denied(String),
    Internal(String),
}

impl std::fmt::Display for PactError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Denied(msg) => write!(f, "pact denied: {}", msg),
            Self::Internal(msg) => write!(f, "internal error: {}", msg),
        }
    }
}

impl std::error::Error for PactError {}

