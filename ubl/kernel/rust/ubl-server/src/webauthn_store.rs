//! WebAuthn Step-Up Store
//!
//! Handles step-up authentication with binding_hash for L4/L5 Console permits.
//! The challenge is cryptographically bound to the Permit's binding_hash.

use axum::http::StatusCode;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use webauthn_rs::prelude::*;

fn now_ms() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as i64
}

// =============================================================================
// REQUEST/RESPONSE TYPES
// =============================================================================

#[derive(Debug, Deserialize)]
pub struct StepUpBeginRequest {
    pub user_id: String,        // subject id (sid)
    pub binding_hash: String,   // blake3:... from Permit
}

#[derive(Debug, Serialize)]
pub struct StepUpBeginResponse {
    pub challenge_id: String,
    pub public_key: RequestChallengeResponse,
}

/// Assertion JSON from browser (navigator.credentials.get result)
pub type AssertionJson = serde_json::Value;

// =============================================================================
// BEGIN STEP-UP
// =============================================================================

/// Begin a step-up authentication tied to a specific binding_hash.
/// Returns WebAuthn challenge options for the browser.
pub async fn begin_stepup(
    pool: &PgPool,
    webauthn: &Webauthn,
    req: &StepUpBeginRequest,
) -> Result<StepUpBeginResponse, (StatusCode, String)> {
    use crate::id_db;

    // 1. Get subject
    let subject = id_db::get_subject(pool, &req.user_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "UserNotFound".into()))?;

    // 2. Get credentials
    let credentials = id_db::get_credentials(pool, &subject.sid)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut passkeys = Vec::new();
    for cred in credentials {
        if cred.credential_kind == "webauthn" {
            let passkey: Passkey = serde_json::from_slice(&cred.public_key)
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to parse passkey: {}", e)))?;
            passkeys.push(passkey);
        }
    }

    if passkeys.is_empty() {
        return Err((StatusCode::NOT_FOUND, "NoWebAuthnCredentials".into()));
    }

    // 3. Create WebAuthn challenge
    let (rcr, auth_state) = webauthn
        .start_passkey_authentication(&passkeys)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("WebAuthn error: {:?}", e)))?;

    // 4. Extract challenge from RCR for storage
    let challenge_b64 = URL_SAFE_NO_PAD.encode(rcr.public_key.challenge.0.as_slice());

    // 5. Serialize auth_state for later verification
    let auth_state_bytes = serde_json::to_vec(&auth_state)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Serialize error: {}", e)))?;

    // 6. Store challenge with binding_hash (90s TTL)
    let challenge_id = uuid::Uuid::new_v4().to_string();
    let created_at_ms = now_ms();
    let exp_ms = created_at_ms + 90_000; // 90 seconds

    sqlx::query(
        r#"
        INSERT INTO id_stepup_challenges
          (challenge_id, user_id, binding_hash, challenge_b64, auth_state, created_at_ms, exp_ms, used)
        VALUES
          ($1, $2, $3, $4, $5, $6, $7, false)
        "#
    )
    .bind(&challenge_id)
    .bind(&req.user_id)
    .bind(&req.binding_hash)
    .bind(&challenge_b64)
    .bind(&auth_state_bytes)
    .bind(created_at_ms)
    .bind(exp_ms)
    .execute(pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StepUpBeginResponse {
        challenge_id,
        public_key: rcr,
    })
}

// =============================================================================
// VERIFY STEP-UP ASSERTION
// =============================================================================

/// Verify a step-up assertion against an expected binding_hash.
/// This is called by issue_permit for L4/L5 operations.
/// 
/// Returns Ok(true) if valid, Err with message otherwise.
pub async fn verify_stepup_assertion(
    pool: &PgPool,
    webauthn: &Webauthn,
    assertion_json: &AssertionJson,
    expected_binding_hash: &str,
) -> Result<bool, String> {
    use crate::id_db;

    // 1. Parse the PublicKeyCredential from JSON
    let credential: PublicKeyCredential = serde_json::from_value(assertion_json.clone())
        .map_err(|e| format!("InvalidAssertionFormat: {}", e))?;

    // 2. Extract challenge from clientDataJSON
    let client_data_b64 = assertion_json
        .get("response")
        .and_then(|r| r.get("clientDataJSON"))
        .and_then(|c| c.as_str())
        .ok_or("MissingClientDataJSON")?;

    let client_data_bytes = URL_SAFE_NO_PAD.decode(client_data_b64)
        .map_err(|_| "InvalidClientDataBase64")?;

    let client_data: serde_json::Value = serde_json::from_slice(&client_data_bytes)
        .map_err(|_| "InvalidClientDataJSON")?;

    let challenge_b64 = client_data
        .get("challenge")
        .and_then(|c| c.as_str())
        .ok_or("MissingChallengeInClientData")?
        .to_string();

    // 3. Find the challenge row by challenge_b64
    let row = sqlx::query(
        r#"
        SELECT challenge_id, user_id, binding_hash, auth_state, exp_ms, used
        FROM id_stepup_challenges
        WHERE challenge_b64 = $1
        ORDER BY created_at_ms DESC
        LIMIT 1
        "#
    )
    .bind(&challenge_b64)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?
    .ok_or("StepUpChallengeNotFound")?;

    let challenge_id: String = row.get("challenge_id");
    let user_id: String = row.get("user_id");
    let binding_hash: String = row.get("binding_hash");
    let auth_state_bytes: Vec<u8> = row.get("auth_state");
    let exp_ms: i64 = row.get("exp_ms");
    let used: bool = row.get("used");

    // 4. Validate: not used, not expired, binding matches
    if used {
        return Err("StepUpChallengeAlreadyUsed".into());
    }
    if now_ms() > exp_ms {
        return Err("StepUpChallengeExpired".into());
    }
    if binding_hash != expected_binding_hash {
        return Err(format!(
            "StepUpBindingMismatch: expected {} got {}",
            expected_binding_hash, binding_hash
        ));
    }

    // 5. Deserialize auth_state
    let auth_state: PasskeyAuthentication = serde_json::from_slice(&auth_state_bytes)
        .map_err(|e| format!("InvalidAuthState: {}", e))?;

    // 6. Verify the assertion cryptographically
    let auth_result = webauthn
        .finish_passkey_authentication(&credential, &auth_state)
        .map_err(|e| format!("WebAuthnVerifyFailed: {:?}", e))?;

    // 7. Validate sign_count (anti-replay)
    let cred_id_b64 = URL_SAFE_NO_PAD.encode(auth_result.cred_id());
    let cred = id_db::get_credential_by_id(pool, &user_id, &cred_id_b64)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("CredentialNotFound")?;

    let new_counter = auth_result.counter();
    if new_counter <= cred.sign_count as u32 {
        return Err(format!(
            "CounterRollback: old={} new={}",
            cred.sign_count, new_counter
        ));
    }

    // 8. Update sign_count
    id_db::update_sign_count(pool, cred.id, new_counter as i64)
        .await
        .map_err(|e| e.to_string())?;

    // 9. Mark challenge as used (single-use)
    sqlx::query("UPDATE id_stepup_challenges SET used = true WHERE challenge_id = $1")
        .bind(&challenge_id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(true)
}

// =============================================================================
// HELPERS
// =============================================================================

/// Extract approver ID from assertion for audit trail
pub fn extract_approver_id(assertion_json: &AssertionJson) -> Option<String> {
    let cred_id = assertion_json.get("id")?.as_str()?;
    Some(format!("webauthn:{}", cred_id))
}

// =============================================================================
// EXTENSION TRAIT FOR sqlx::Row
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

