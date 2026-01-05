//! # UBL ID Routes
//!
//! Identity API for People (WebAuthn), LLMs, and Apps

use axum::{
    extract::{Path, State},
    http::{StatusCode, HeaderMap, HeaderValue},
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
    middleware,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
use webauthn_rs::prelude::*;

use crate::id_db;
use crate::auth::session::{Session, SessionFlavor};
use crate::auth::session_db;
use crate::tenant;

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

fn set_session_cookie(headers: &mut HeaderMap, token: &str, ttl_secs: i64) {
    let cookie = format!(
        "session={}; HttpOnly; Secure; SameSite=Strict; Max-Age={}; Path=/",
        token, ttl_secs
    );
    if let Ok(hv) = HeaderValue::from_str(&cookie) {
        headers.append("Set-Cookie", hv);
    }
}

#[derive(serde::Deserialize)]
struct ClientDataJSON {
    challenge: String,
    origin: String,
    #[serde(rename = "type")]
    typ: String,
}

fn parse_client_data_json(base64url_bytes: &[u8]) -> Result<ClientDataJSON, String> {
    let decoded = URL_SAFE_NO_PAD
        .decode(base64url_bytes)
        .map_err(|e| format!("base64 decode failed: {}", e))?;
    serde_json::from_slice::<ClientDataJSON>(&decoded)
        .map_err(|e| format!("JSON parse failed: {}", e))
}

fn assert_origin(cdj: &ClientDataJSON) -> Result<(), (StatusCode, String)> {
    let want = std::env::var("WEBAUTHN_ORIGIN")
        .unwrap_or_else(|_| "http://localhost:8080".to_string());
    if cdj.origin != want {
        return Err((StatusCode::UNAUTHORIZED, "origin_mismatch".to_string()));
    }
    Ok(())
}

// ============================================================================
// STATE
// ============================================================================

#[derive(Clone)]
pub struct IdState {
    pub pool: PgPool,
    pub webauthn: Webauthn,
    pub rate_limiter: crate::rate_limit::RateLimiter,
}

// ============================================================================
// REQUEST/RESPONSE TYPES
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateAgentReq {
    pub kind: String, // "llm" | "app"
    pub display_name: String,
    pub public_key: String, // hex Ed25519 (64 chars)
}

#[derive(Debug, Serialize)]
pub struct CreateAgentResp {
    pub sid: String,
    pub kind: String,
    pub display_name: String,
    pub public_key: String,
}

#[derive(Debug, Deserialize)]
pub struct IssueAscReq {
    pub containers: Vec<String>,
    pub intent_classes: Vec<String>,
    pub max_delta: Option<i128>,
    pub ttl_secs: i64,
}

#[derive(Debug, Serialize)]
pub struct IssueAscResp {
    pub asc_id: String,
    pub sid: String,
    pub scopes: serde_json::Value,
    pub not_before: String,
    pub not_after: String,
    pub signature: String, // hex
}

/// Response for ASC validation endpoint
#[derive(Debug, Serialize)]
pub struct ValidateAscResp {
    pub valid: bool,
    pub asc_id: String,
    pub owner_sid: String,
    pub owner_kind: String,
    pub containers: Vec<String>,
    pub intent_classes: Vec<String>,
    pub max_delta: i64,
    pub not_before: String,
    pub not_after: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RotateKeyReq {
    pub new_public_key: String, // hex
}

#[derive(Debug, Serialize)]
pub struct RotateKeyResp {
    pub sid: String,
    pub key_version: i32,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct WhoamiResp {
    pub sid: Option<String>,
    pub kind: Option<String>,
    pub display_name: Option<String>,
    pub authenticated: bool,
}

#[derive(Debug, Deserialize)]
pub struct IcteBeginReq {
    pub scope: serde_json::Value,
    pub ttl_seconds: i64,
}

#[derive(Debug, Serialize)]
pub struct IcteBeginResp {
    pub session_id: String,
    pub not_before: String,
    pub not_after: String,
}

#[derive(Debug, Deserialize)]
pub struct IcteFinishReq {
    pub session_id: String,
}

#[derive(Debug, Serialize)]
pub struct IcteFinishResp {
    pub message: String,
}

// WebAuthn registration begin
#[derive(Debug, Deserialize)]
pub struct RegisterBeginReq {
    pub username: String,
    pub display_name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RegisterBeginResp {
    pub challenge_id: String,
    pub options: CreationChallengeResponse,
}

// WebAuthn registration finish
#[derive(Debug, Deserialize)]
pub struct RegisterFinishReq {
    pub challenge_id: String,
    pub attestation: RegisterPublicKeyCredential,
}

#[derive(Debug, Serialize)]
pub struct RegisterFinishResp {
    pub sid: String,
    pub username: String,
    pub session_token: String,
}

// WebAuthn login begin
#[derive(Debug, Deserialize)]
pub struct LoginBeginReq {
    pub username: String,
}

#[derive(Debug, Serialize)]
pub struct LoginBeginResp {
    pub challenge_id: String,
    pub public_key: RequestChallengeResponse,
}

// WebAuthn login finish
#[derive(Debug, Deserialize)]
pub struct LoginFinishReq {
    pub challenge_id: String,
    pub credential: PublicKeyCredential,
}

#[derive(Debug, Serialize)]
pub struct LoginFinishResp {
    pub sid: String,
    pub session_token: String,
}

// Step-up (admin) begin
#[derive(Debug, Deserialize)]
pub struct StepupBeginReq {
    pub session_token: String,
    pub username: String,
}

#[derive(Debug, Serialize)]
pub struct StepupBeginResp {
    pub challenge_id: String,
    pub public_key: RequestChallengeResponse,
}

// Step-up (admin) finish
#[derive(Debug, Deserialize)]
pub struct StepupFinishReq {
    pub challenge_id: String,
    pub assertion: PublicKeyCredential,
}

#[derive(Debug, Serialize)]
pub struct StepupFinishResp {
    pub stepup_token: String,
    pub expires_in: i64, // seconds
}

// ============================================================================
// HANDLERS
// ============================================================================

/// POST /id/agents - Create LLM or App agent
pub async fn route_create_agent(
    State(state): State<IdState>,
    Json(req): Json<CreateAgentReq>,
) -> Result<Json<CreateAgentResp>, (StatusCode, String)> {
    // Validate kind
    if req.kind != "llm" && req.kind != "app" {
        return Err((
            StatusCode::BAD_REQUEST,
            "kind must be 'llm' or 'app'".into(),
        ));
    }

    // Validate public key (Ed25519 = 64 hex chars)
    if req.public_key.len() != 64 {
        return Err((
            StatusCode::BAD_REQUEST,
            "public_key must be 64 hex characters (Ed25519)".into(),
        ));
    }

    match id_db::create_agent(&state.pool, &req.kind, &req.display_name, &req.public_key).await {
        Ok(subject) => Ok(Json(CreateAgentResp {
            sid: subject.sid,
            kind: subject.kind,
            display_name: subject.display_name,
            public_key: req.public_key,
        })),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )),
    }
}

/// POST /id/agents/{sid}/asc - Issue Agent Signing Certificate
pub async fn route_issue_asc(
    State(state): State<IdState>,
    Path(sid): Path<String>,
    Json(req): Json<IssueAscReq>,
) -> Result<Json<IssueAscResp>, (StatusCode, String)> {
    // Verify subject exists
    let subject = id_db::get_subject(&state.pool, &sid)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Subject not found".to_string()))?;

    // Get current credential
    let cred = id_db::get_credential(&state.pool, &sid, "ed25519")
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "No Ed25519 credential found".to_string()))?;

    // Build scopes JSON
    let scopes = serde_json::json!({
        "containers": req.containers,
        "intent_classes": req.intent_classes,
        "max_delta": req.max_delta,
    });

    // TODO: Sign with UBL ID authority key (for now, placeholder)
    let signature = vec![0u8; 64]; // Placeholder Ed25519 signature

    let asc = id_db::issue_asc(
        &state.pool,
        &sid,
        cred.public_key.clone(),
        scopes,
        req.ttl_secs,
        signature,
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(IssueAscResp {
        asc_id: asc.asc_id.to_string(),
        sid: asc.sid,
        scopes: asc.scopes,
        not_before: asc.not_before.to_string(),
        not_after: asc.not_after.to_string(),
        signature: hex::encode(asc.signature),
    }))
}

/// GET /id/asc/{asc_id}/validate - Validate an ASC (for Office to call)
/// 
/// This endpoint allows Office (and other services) to validate an ASC
/// through the UBL Kernel instead of accessing the database directly.
pub async fn route_validate_asc(
    State(state): State<IdState>,
    Path(asc_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<ValidateAscResp>, (StatusCode, String)> {
    // 1. Parse ASC ID
    let asc_uuid = Uuid::parse_str(&asc_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid ASC ID format".to_string()))?;

    // 2. Get ASC from database
    let asc = id_db::get_asc_by_id(&state.pool, asc_uuid)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "ASC not found".to_string()))?;

    // 3. Check expiration
    let now = time::OffsetDateTime::now_utc();
    let is_valid = asc.not_before <= now && asc.not_after >= now;

    // 4. Get subject info for additional context
    let subject = id_db::get_subject(&state.pool, &asc.sid)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // 5. Extract scope details
    let containers = asc.scopes.get("containers")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_else(Vec::new);
    
    let intent_classes = asc.scopes.get("intent_classes")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_else(Vec::new);
    
    let max_delta = asc.scopes.get("max_delta")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    Ok(Json(ValidateAscResp {
        valid: is_valid,
        asc_id: asc.asc_id.to_string(),
        owner_sid: asc.sid.clone(),
        owner_kind: subject.map(|s| s.kind).unwrap_or_else(|| "unknown".to_string()),
        containers,
        intent_classes,
        max_delta,
        not_before: asc.not_before.to_string(),
        not_after: asc.not_after.to_string(),
        reason: if is_valid { None } else { Some("ASC expired".to_string()) },
    }))
}

/// POST /id/agents/{sid}/rotate - Rotate agent key
pub async fn route_rotate_key(
    State(state): State<IdState>,
    Path(sid): Path<String>,
    Json(req): Json<RotateKeyReq>,
) -> Result<Json<RotateKeyResp>, (StatusCode, String)> {
    // Get current credential
    let cred = id_db::get_credential(&state.pool, &sid, "ed25519")
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "No Ed25519 credential found".to_string()))?;

    // Decode new public key
    let new_pubkey = hex::decode(&req.new_public_key)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid hex public key".to_string()))?;

    if new_pubkey.len() != 32 {
        return Err((
            StatusCode::BAD_REQUEST,
            "Public key must be 32 bytes (Ed25519)".to_string(),
        ));
    }

    // Rotate
    id_db::rotate_key(&state.pool, &sid, new_pubkey, cred.key_version)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(RotateKeyResp {
        sid,
        key_version: cred.key_version + 1,
        message: "Key rotated successfully, old key revoked".to_string(),
    }))
}

/// GET /id/whoami - Get current identity from session cookie or Bearer token
pub async fn route_whoami(
    State(state): State<IdState>,
    headers: HeaderMap,
) -> Json<WhoamiResp> {
    // Try to extract session token from cookie or Authorization header
    let token = extract_session_token(&headers);
    
    if let Some(token) = token {
        // Validate session
        if let Ok(Some(session)) = session_db::get_valid(&state.pool, &token).await {
            // Get subject details
            let sid_str = session.sid.to_string();
            if let Ok(Some(subject)) = id_db::get_subject(&state.pool, &sid_str).await {
                return Json(WhoamiResp {
                    sid: Some(subject.sid),
                    kind: Some(subject.kind),
                    display_name: Some(subject.display_name),
                    authenticated: true,
                });
            }
            // Session valid but subject not found (edge case)
            return Json(WhoamiResp {
                sid: Some(sid_str),
                kind: None,
                display_name: None,
                authenticated: true,
            });
        }
    }
    
    // No valid session
    Json(WhoamiResp {
        sid: None,
        kind: None,
        display_name: None,
        authenticated: false,
    })
}

/// Extract session token from cookie or Authorization header
fn extract_session_token(headers: &HeaderMap) -> Option<String> {
    // Try Authorization: Bearer <token>
    if let Some(auth) = headers.get(axum::http::header::AUTHORIZATION) {
        if let Ok(auth_str) = auth.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                return Some(token.to_string());
            }
        }
    }
    
    // Try session cookie
    if let Some(cookie) = headers.get("cookie") {
        if let Ok(cookie_str) = cookie.to_str() {
            for part in cookie_str.split(';') {
                let mut kv = part.trim().splitn(2, '=');
                if let (Some(key), Some(value)) = (kv.next(), kv.next()) {
                    if key == "session" {
                        return Some(value.to_string());
                    }
                }
            }
        }
    }
    
    None
}

/// POST /id/register/begin - Begin WebAuthn registration
pub async fn route_register_begin(
    State(state): State<IdState>,
    Json(req): Json<RegisterBeginReq>,
) -> Result<Json<RegisterBeginResp>, (StatusCode, String)> {
    use tracing::{info, warn};
    let start = std::time::Instant::now();
    
    // Rate limit: 5 registrations per username per hour
    let rate_key = format!("register:{}", req.username);
    if let Err(retry_after) = state.rate_limiter.check(&rate_key, 5, 3600) {
        crate::metrics::RATE_LIMIT_REJECTIONS.with_label_values(&["register"]).inc();
        warn!(actor_type="person", username=%req.username, decision="reject", error_code="rate_limited", retry_after_secs=%retry_after);
        return Err((StatusCode::TOO_MANY_REQUESTS, format!("Rate limited. Retry after {} seconds", retry_after)));
    }
    
    // 1. Check if user already exists
    let existing = id_db::get_subject_by_username(&state.pool, &req.username)
        .await
        .map_err(|e| {
            warn!(actor_type="person", username=%req.username, decision="reject", error_code="db_error", latency_ms=start.elapsed().as_millis());
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;

    if existing.is_some() {
        warn!(actor_type="person", username=%req.username, decision="reject", error_code="username_exists", latency_ms=start.elapsed().as_millis());
        return Err((StatusCode::CONFLICT, "Username already registered".to_string()));
    }

    // 2. Prepare display name
    let display_name = req.display_name.unwrap_or_else(|| req.username.clone());
    // BUG FIX #2: Removed unused user_id variable

    // 3. Start passkey registration
    let (challenge_response, passkey_registration) = state.webauthn
        .start_passkey_registration(
            Uuid::new_v4(),
            &req.username,
            &display_name,
            None,
        )
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to start registration: {:?}", e)))?;

    // 4. Store challenge in database
    let reg_state_bytes = serde_json::to_vec(&passkey_registration)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to serialize state: {}", e)))?;

    // BUG FIX #6: Use WEBAUTHN_ORIGIN from env
    let webauthn_origin = std::env::var("WEBAUTHN_ORIGIN")
        .unwrap_or_else(|_| "http://localhost:8080".to_string());
    
    let challenge_id = id_db::create_register_challenge(
        &state.pool,
        &req.username, // Store username to retrieve in finish
        reg_state_bytes,
        &webauthn_origin,
        300, // 5 minutes TTL
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    crate::metrics::WEBAUTHN_OPS.with_label_values(&["register", "begin"]).inc();
    info!(actor_type="person", username=%req.username, challenge_id=%challenge_id, decision="accept", phase="begin", latency_ms=start.elapsed().as_millis());
    
    Ok(Json(RegisterBeginResp {
        challenge_id: challenge_id.to_string(),
        options: challenge_response,
    }))
}

/// POST /id/register/finish - Finish WebAuthn registration
pub async fn route_register_finish(
    State(state): State<IdState>,
    Json(req): Json<RegisterFinishReq>,
) -> Result<Json<RegisterFinishResp>, (StatusCode, String)> {
    use tracing::{info, warn};
    let start = std::time::Instant::now();
    
    // Diamond Checklist #4: Consume challenge FIRST atomically (anti-replay)
    let challenge_uuid = Uuid::parse_str(&req.challenge_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid challenge ID".to_string()))?;
    
    let webauthn_origin = std::env::var("WEBAUTHN_ORIGIN")
        .unwrap_or_else(|_| "http://localhost:8080".to_string());
    let challenge = id_db::consume_challenge(&state.pool, challenge_uuid, &webauthn_origin)
        .await
        .map_err(|e| {
            warn!(challenge_id=%req.challenge_id, decision="reject", error_code="db_error", latency_ms=start.elapsed().as_millis());
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?
        .ok_or_else(|| {
            warn!(challenge_id=%req.challenge_id, decision="reject", error_code="challenge_invalid_or_used", latency_ms=start.elapsed().as_millis());
            (StatusCode::BAD_REQUEST, "Challenge not found, expired, or already used".to_string())
        })?;

    if challenge.kind != "register" {
        warn!(challenge_id=%req.challenge_id, decision="reject", error_code="invalid_challenge_kind", latency_ms=start.elapsed().as_millis());
        return Err((StatusCode::BAD_REQUEST, "Not a registration challenge".to_string()));
    }

    // 2. Parse challenge data (contains username + state)
    let challenge_data: serde_json::Value = serde_json::from_slice(&challenge.challenge)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Invalid challenge data: {}", e)))?;
    
    let username = challenge_data["username"].as_str()
        .ok_or_else(|| (StatusCode::INTERNAL_SERVER_ERROR, "No username in challenge".to_string()))?
        .to_string();
    
    let state_bytes: Vec<u8> = serde_json::from_value(challenge_data["state"].clone())
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Invalid state data: {}", e)))?;

    let passkey_registration: PasskeyRegistration = serde_json::from_slice(&state_bytes)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Invalid registration state: {}", e)))?;

    // 3. Verify attestation
    let passkey = state.webauthn
        .finish_passkey_registration(&req.attestation, &passkey_registration)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Registration verification failed: {:?}", e)))?;

    // 4. Create person subject with username from challenge
    let sid = id_db::create_person(&state.pool, &username, &username)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // 5. Store credential
    let credential_id = URL_SAFE_NO_PAD.encode(passkey.cred_id());
    let public_key_bytes = serde_json::to_vec(&passkey)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to serialize passkey: {}", e)))?;

    id_db::create_credential(
        &state.pool,
        &sid,
        "passkey",
        &credential_id,
        &public_key_bytes,
        0, // initial sign_count
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // 6. Challenge already consumed at the start (anti-replay)

    // 7. Create session for auto-login after registration
    // SID is now a string type "ubl:sid:<hash>", used directly
    let session = crate::auth::session::Session::new_regular(&sid);
    let session_token = session.token.clone();
    
    crate::auth::session_db::insert(&state.pool, &session)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create session: {}", e)))?;

    crate::metrics::ID_DECISIONS.with_label_values(&["register", "accept", ""]).inc();
    crate::metrics::WEBAUTHN_OPS.with_label_values(&["register", "finish"]).inc();
    info!(actor_type="person", username=%username, sid=%sid, session_token=%session_token, challenge_id=%req.challenge_id, decision="accept", phase="finish", latency_ms=start.elapsed().as_millis());
    
    Ok(Json(RegisterFinishResp {
        sid: sid.clone(),
        username,
        session_token,
    }))
}

/// POST /id/login/begin - Begin WebAuthn login
pub async fn route_login_begin(
    State(state): State<IdState>,
    Json(req): Json<LoginBeginReq>,
) -> Result<Json<LoginBeginResp>, (StatusCode, String)> {
    use tracing::{info, warn};
    let start = std::time::Instant::now();
    
    // Rate limit: 10 login attempts per username per 5 minutes
    let rate_key = format!("login:{}", req.username);
    if let Err(retry_after) = state.rate_limiter.check(&rate_key, 10, 300) {
        crate::metrics::RATE_LIMIT_REJECTIONS.with_label_values(&["login"]).inc();
        warn!(actor_type="person", username=%req.username, decision="reject", error_code="rate_limited", retry_after_secs=%retry_after);
        return Err((StatusCode::TOO_MANY_REQUESTS, format!("Too many login attempts. Retry after {} seconds", retry_after)));
    }
    
    // 1. Get subject by username
    let subject = id_db::get_subject_by_username(&state.pool, &req.username)
        .await
        .map_err(|e| {
            warn!(actor_type="person", username=%req.username, decision="reject", error_code="db_error", latency_ms=start.elapsed().as_millis());
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?
        .ok_or_else(|| {
            warn!(actor_type="person", username=%req.username, decision="reject", error_code="unknown_credential", latency_ms=start.elapsed().as_millis());
            (StatusCode::NOT_FOUND, "User not found".to_string())
        })?;

    if subject.kind != "person" {
        warn!(actor_type="person", username=%req.username, sid=%subject.sid, decision="reject", error_code="invalid_subject_kind", latency_ms=start.elapsed().as_millis());
        return Err((StatusCode::BAD_REQUEST, "Not a person account".to_string()));
    }

    // 2. Get all credentials for user
    let credentials = id_db::get_credentials(&state.pool, &subject.sid)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if credentials.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "No credentials registered".to_string()));
    }

    // 3. Parse passkeys from credentials
    let mut passkeys = Vec::new();
    for cred in credentials {
        if cred.credential_kind == "passkey" {
            let passkey: Passkey = serde_json::from_slice(&cred.public_key)
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to parse passkey: {}", e)))?;
            passkeys.push(passkey);
        }
    }

    if passkeys.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "No WebAuthn credentials found".to_string()));
    }

    // 4. Create authentication challenge
    let (rcr, auth_state) = state.webauthn
        .start_passkey_authentication(&passkeys)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to start authentication: {:?}", e)))?;

    // 5. Store challenge in database
    let auth_state_bytes = serde_json::to_vec(&auth_state)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to serialize auth state: {}", e)))?;

    // BUG FIX #6: Use WEBAUTHN_ORIGIN from env
    let webauthn_origin = std::env::var("WEBAUTHN_ORIGIN")
        .unwrap_or_else(|_| "http://localhost:8080".to_string());
    
    let challenge_id = id_db::create_login_challenge(
        &state.pool,
        &subject.sid,
        auth_state_bytes,
        &webauthn_origin,
        300, // 5 minutes TTL
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    crate::metrics::WEBAUTHN_OPS.with_label_values(&["login", "begin"]).inc();
    info!(actor_type="person", username=%req.username, sid=%subject.sid, challenge_id=%challenge_id, decision="accept", phase="login_begin", latency_ms=start.elapsed().as_millis());
    
    Ok(Json(LoginBeginResp {
        challenge_id: challenge_id.to_string(),
        public_key: rcr,
    }))
}

/// POST /id/login/finish - Finish WebAuthn login
pub async fn route_login_finish(
    State(state): State<IdState>,
    Json(req): Json<LoginFinishReq>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    use tracing::{info, warn};
    let start = std::time::Instant::now();
    
    // Diamond Checklist #4: Parse challenge ID first
    let challenge_uuid = Uuid::parse_str(&req.challenge_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid challenge ID".to_string()))?;
    
    let webauthn_origin = std::env::var("WEBAUTHN_ORIGIN")
        .unwrap_or_else(|_| "http://localhost:8080".to_string());
    
    // Note: Challenge will be consumed atomically with session creation later
    // First we need to validate the authentication
    
    // 1. Temporarily get challenge to extract auth state (without consuming yet)
    let challenge = id_db::get_challenge(&state.pool, &req.challenge_id)
        .await
        .map_err(|e| {
            warn!(challenge_id=%req.challenge_id, decision="reject", error_code="db_error", latency_ms=start.elapsed().as_millis());
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?
        .ok_or_else(|| {
            warn!(challenge_id=%req.challenge_id, decision="reject", error_code="challenge_not_found", latency_ms=start.elapsed().as_millis());
            (StatusCode::BAD_REQUEST, "Challenge not found or expired".to_string())
        })?;
    
    // Check if challenge is already used
    if challenge.used {
        warn!(challenge_id=%req.challenge_id, decision="reject", error_code="challenge_already_used", latency_ms=start.elapsed().as_millis());
        return Err((StatusCode::BAD_REQUEST, "Challenge already used".to_string()));
    }

    if challenge.kind != "login" {
        warn!(challenge_id=%req.challenge_id, decision="reject", error_code="invalid_challenge_kind", latency_ms=start.elapsed().as_millis());
        return Err((StatusCode::BAD_REQUEST, "Not a login challenge".to_string()));
    }

    // Validate origin from clientDataJSON
    if let Ok(cdj) = parse_client_data_json(&req.credential.response.client_data_json) {
        assert_origin(&cdj)?;
    }

    // 2. Parse authentication state
    let auth_state: PasskeyAuthentication = serde_json::from_slice(&challenge.challenge)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Invalid auth state: {}", e)))?;

    // 3. Verify assertion
    let auth_result = state.webauthn
        .finish_passkey_authentication(&req.credential, &auth_state)
        .map_err(|e| {
            // Register failed authentication attempt
            let lockout_key = format!("login_lockout:{}", challenge.sid.as_deref().unwrap_or("unknown"));
            state.rate_limiter.on_fail(&lockout_key);
            let fails = state.rate_limiter.get_failures(&lockout_key);
            
            // Track lockout activation if threshold exceeded
            if fails > 5 {
                crate::metrics::LOCKOUT_ACTIVATIONS.with_label_values(&[&fails.to_string()]).inc();
            }
            
            crate::metrics::ID_DECISIONS.with_label_values(&["login", "reject", "auth_failed"]).inc();
            warn!(challenge_id=%req.challenge_id, decision="reject", error_code="auth_failed", 
                  consecutive_failures=%fails, latency_ms=start.elapsed().as_millis());
            (StatusCode::UNAUTHORIZED, format!("Authentication failed: {:?}", e))
        })?;

    // 4. Get credential and update sign_count
    let credential_id = URL_SAFE_NO_PAD.encode(auth_result.cred_id());
    let sid_str = challenge.sid.clone().ok_or_else(|| 
        (StatusCode::INTERNAL_SERVER_ERROR, "Challenge has no SID".to_string())
    )?;
    
    let cred = id_db::get_credential_by_id(&state.pool, &sid_str, &credential_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Credential not found".to_string()))?;

    // 5. Validate sign_count (prevent replay attacks)
    // Note: Some authenticators always return 0, and first login may have counter = 0
    // Only reject if counter goes BACKWARDS (strict rollback detection)
    let new_counter = auth_result.counter();
    let stored_counter = cred.sign_count as u32;
    
    // Reject only if counter decreased (not if stayed same on first use)
    if new_counter < stored_counter || (stored_counter > 0 && new_counter == stored_counter) {
        let lockout_key = format!("login_lockout:{}", sid_str);
        state.rate_limiter.on_fail(&lockout_key);
        let fails = state.rate_limiter.get_failures(&lockout_key);
        
        // Track lockout activation if threshold exceeded
        if fails > 5 {
            crate::metrics::LOCKOUT_ACTIVATIONS.with_label_values(&[&fails.to_string()]).inc();
        }
        
        crate::metrics::ID_DECISIONS.with_label_values(&["login", "reject", "counter_rollback"]).inc();
        warn!(challenge_id=%req.challenge_id, sid=%sid_str, decision="reject", error_code="counter_rollback", 
              old_count=%cred.sign_count, new_count=%new_counter, consecutive_failures=%fails, latency_ms=start.elapsed().as_millis());
        return Err((
            StatusCode::UNAUTHORIZED,
            "Sign count did not increase - possible replay attack".to_string(),
        ));
    }

    // 6. Update sign_count
    id_db::update_sign_count(&state.pool, cred.id, new_counter as i64)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // 7. Challenge already consumed at the start (anti-replay)

    // 8. Diamond Checklist #4: Atomically consume challenge and create session
    // This prevents replay attacks by ensuring both operations succeed or fail together
    let final_sid = challenge.sid.ok_or_else(|| 
        (StatusCode::INTERNAL_SERVER_ERROR, "Challenge has no SID".to_string())
    )?;
    
    // 8.1 Get user's default tenant for Zona Schengen
    let user_tenant = tenant::db::get_user_tenant(&state.pool, &final_sid)
        .await
        .ok()
        .flatten();
    
    // 8.2 Prepare session data
    let session = Session::new_regular_with_tenant(&final_sid, user_tenant.clone());
    let scope_with_context = serde_json::json!({
        "legacy": session.scope,
        "context": session.context,
    });
    let flavor_str = match session.flavor {
        SessionFlavor::Regular => "regular",
        SessionFlavor::StepUp => "stepup",
    };
    
    // 8.3 Atomically consume challenge and create session (Diamond Checklist #4)
    let consumed_challenge = id_db::consume_challenge_and_create_session(
        &state.pool,
        challenge_uuid,
        &webauthn_origin,
        &session.token,
        &final_sid,
        user_tenant.as_deref(),
        flavor_str,
        &scope_with_context,
        session.exp_unix,
    )
    .await
    .map_err(|e| {
        warn!(challenge_id=%req.challenge_id, decision="reject", error_code="atomic_operation_failed", latency_ms=start.elapsed().as_millis());
        (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to consume challenge and create session: {}", e))
    })?
    .ok_or_else(|| {
        warn!(challenge_id=%req.challenge_id, decision="reject", error_code="challenge_already_used", latency_ms=start.elapsed().as_millis());
        (StatusCode::BAD_REQUEST, "Challenge already used or expired".to_string())
    })?;

    // 9. Set HttpOnly cookie
    let mut headers = HeaderMap::new();
    set_session_cookie(&mut headers, &session.token, session.ttl_secs());

    // Reset failure counter on successful login
    let lockout_key = format!("login_lockout:{}", final_sid);
    state.rate_limiter.on_success(&lockout_key);

    crate::metrics::ID_DECISIONS.with_label_values(&["login", "accept", ""]).inc();
    crate::metrics::WEBAUTHN_OPS.with_label_values(&["login", "finish"]).inc();
    info!(actor_type="person", sid=%final_sid, challenge_id=%req.challenge_id, session_token=%session.token, 
          decision="accept", phase="login_finish", sign_count=%new_counter, latency_ms=start.elapsed().as_millis());
    
    let resp = Json(LoginFinishResp {
        sid: final_sid,
        session_token: session.token.clone(),
    });
    
    Ok((headers, resp))
}

// ============================================================================
// DISCOVERABLE (USERLESS) LOGIN - for "Sign in with Passkey" button
// ============================================================================

/// POST /id/login/discoverable/begin - Begin discoverable (userless) WebAuthn login
pub async fn route_login_discoverable_begin(
    State(state): State<IdState>,
) -> Result<Json<LoginBeginResp>, (StatusCode, String)> {
    use tracing::info;
    let start = std::time::Instant::now();
    
    // 1. Create discoverable authentication challenge (no username needed)
    let (rcr, auth_state) = state.webauthn
        .start_discoverable_authentication()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to start discoverable authentication: {:?}", e)))?;

    // 2. Serialize the discoverable auth state
    let auth_state_bytes = serde_json::to_vec(&auth_state)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to serialize auth state: {}", e)))?;

    // 3. Store challenge in database (no sid - NULL for discoverable flow)
    // BUG FIX #6: Use WEBAUTHN_ORIGIN from env
    let webauthn_origin = std::env::var("WEBAUTHN_ORIGIN")
        .unwrap_or_else(|_| "http://localhost:8080".to_string());
    
    let challenge_id = id_db::create_discoverable_challenge(
        &state.pool,
        auth_state_bytes,
        &webauthn_origin,
        300, // 5 minutes TTL
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    crate::metrics::WEBAUTHN_OPS.with_label_values(&["login_discoverable", "begin"]).inc();
    info!(actor_type="person", challenge_id=%challenge_id, decision="accept", phase="discoverable_login_begin", latency_ms=start.elapsed().as_millis());
    
    Ok(Json(LoginBeginResp {
        challenge_id: challenge_id.to_string(),
        public_key: rcr,
    }))
}

/// POST /id/login/discoverable/finish - Finish discoverable WebAuthn login
pub async fn route_login_discoverable_finish(
    State(state): State<IdState>,
    Json(req): Json<LoginFinishReq>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    use tracing::{info, warn};
    let start = std::time::Instant::now();
    
    // Diamond Checklist #4: Consume challenge FIRST atomically (anti-replay)
    let challenge_uuid = Uuid::parse_str(&req.challenge_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid challenge ID".to_string()))?;
    
    let webauthn_origin = std::env::var("WEBAUTHN_ORIGIN")
        .unwrap_or_else(|_| "http://localhost:8080".to_string());
    let challenge = id_db::consume_challenge(&state.pool, challenge_uuid, &webauthn_origin)
        .await
        .map_err(|e| {
            warn!(challenge_id=%req.challenge_id, decision="reject", error_code="db_error", latency_ms=start.elapsed().as_millis());
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?
        .ok_or_else(|| {
            warn!(challenge_id=%req.challenge_id, decision="reject", error_code="challenge_invalid_or_used", latency_ms=start.elapsed().as_millis());
            (StatusCode::BAD_REQUEST, "Challenge not found, expired, or already used".to_string())
        })?;

    // Validate origin from clientDataJSON
    if let Ok(cdj) = parse_client_data_json(&req.credential.response.client_data_json) {
        assert_origin(&cdj)?;
    }

    // 2. Extract credential_id from the assertion
    // BUG FIX #1/#7: Don't use userHandle (UUID) as SID - look up credential by credential_id instead
    let (_user_uuid, cred_id_bytes) = state.webauthn
        .identify_discoverable_authentication(&req.credential)
        .map_err(|e| {
            warn!(challenge_id=%req.challenge_id, decision="reject", error_code="identify_failed", latency_ms=start.elapsed().as_millis());
            (StatusCode::UNAUTHORIZED, format!("Failed to identify user from credential: {:?}", e))
        })?;
    
    let credential_id = URL_SAFE_NO_PAD.encode(cred_id_bytes);

    // 3. BUG FIX: Look up credential by credential_id to get the REAL SID from database
    let cred = id_db::get_credential_by_cred_id_only(&state.pool, &credential_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| {
            warn!(challenge_id=%req.challenge_id, credential_id=%credential_id, decision="reject", error_code="credential_not_found", latency_ms=start.elapsed().as_millis());
            (StatusCode::NOT_FOUND, "Credential not found".to_string())
        })?;
    
    // Use the SID from the credential record (not the UUID from userHandle!)
    let sid_str = cred.sid.clone();

    // 4. Get the subject to verify they exist
    let subject = id_db::get_subject(&state.pool, &sid_str)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| {
            warn!(challenge_id=%req.challenge_id, sid=%sid_str, decision="reject", error_code="unknown_user", latency_ms=start.elapsed().as_millis());
            (StatusCode::NOT_FOUND, "User not found".to_string())
        })?;

    if subject.kind != "person" {
        return Err((StatusCode::BAD_REQUEST, "Not a person account".to_string()));
    }

    // 5. Parse the stored passkey and convert to DiscoverableKey
    let passkey: Passkey = serde_json::from_slice(&cred.public_key)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to parse passkey: {}", e)))?;
    let discoverable_key: DiscoverableKey = (&passkey).into();

    // 6. Parse discoverable auth state and verify
    let auth_state: DiscoverableAuthentication = serde_json::from_slice(&challenge.challenge)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Invalid auth state: {}", e)))?;

    let auth_result = state.webauthn
        .finish_discoverable_authentication(&req.credential, auth_state, &[discoverable_key])
        .map_err(|e| {
            let lockout_key = format!("login_lockout:{}", sid_str);
            state.rate_limiter.on_fail(&lockout_key);
            let fails = state.rate_limiter.get_failures(&lockout_key);
            
            if fails > 5 {
                crate::metrics::LOCKOUT_ACTIVATIONS.with_label_values(&[&fails.to_string()]).inc();
            }
            
            crate::metrics::ID_DECISIONS.with_label_values(&["login_discoverable", "reject", "auth_failed"]).inc();
            warn!(challenge_id=%req.challenge_id, sid=%sid_str, decision="reject", error_code="auth_failed", 
                  consecutive_failures=%fails, latency_ms=start.elapsed().as_millis());
            (StatusCode::UNAUTHORIZED, format!("Authentication failed: {:?}", e))
        })?;

    // 7. Validate sign_count
    // Note: Some authenticators always return 0, and first login may have counter = 0
    // Only reject if counter goes BACKWARDS (strict rollback detection)
    let new_counter = auth_result.counter();
    let stored_counter = cred.sign_count as u32;
    
    // Reject only if counter decreased (not if stayed same on first use)
    if new_counter < stored_counter || (stored_counter > 0 && new_counter == stored_counter) {
        let lockout_key = format!("login_lockout:{}", sid_str);
        state.rate_limiter.on_fail(&lockout_key);
        let fails = state.rate_limiter.get_failures(&lockout_key);
        
        if fails > 5 {
            crate::metrics::LOCKOUT_ACTIVATIONS.with_label_values(&[&fails.to_string()]).inc();
        }
        
        crate::metrics::ID_DECISIONS.with_label_values(&["login_discoverable", "reject", "counter_rollback"]).inc();
        warn!(challenge_id=%req.challenge_id, sid=%sid_str, decision="reject", error_code="counter_rollback", 
              old_count=%cred.sign_count, new_count=%new_counter, consecutive_failures=%fails, latency_ms=start.elapsed().as_millis());
        return Err((
            StatusCode::UNAUTHORIZED,
            "Sign count did not increase - possible replay attack".to_string(),
        ));
    }

    // 8. Update sign_count
    id_db::update_sign_count(&state.pool, cred.id, new_counter as i64)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // 9. Challenge already consumed at the start (anti-replay)

    // 10. Create session
    // SID is now a string type "ubl:sid:<hash>", used directly
    let user_tenant = tenant::db::get_user_tenant(&state.pool, &sid_str)
        .await
        .ok()
        .flatten();
    
    let session = Session::new_regular_with_tenant(&sid_str, user_tenant);
    session_db::insert(&state.pool, &session)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create session: {}", e)))?;

    // 11. Set HttpOnly cookie
    let mut headers = HeaderMap::new();
    set_session_cookie(&mut headers, &session.token, session.ttl_secs());

    // Reset failure counter on successful login
    let lockout_key = format!("login_lockout:{}", sid_str);
    state.rate_limiter.on_success(&lockout_key);

    crate::metrics::ID_DECISIONS.with_label_values(&["login_discoverable", "accept", ""]).inc();
    crate::metrics::WEBAUTHN_OPS.with_label_values(&["login_discoverable", "finish"]).inc();
    info!(actor_type="person", sid=%sid_str, challenge_id=%req.challenge_id, session_token=%session.token, 
          decision="accept", phase="discoverable_login_finish", sign_count=%new_counter, latency_ms=start.elapsed().as_millis());
    
    let resp = Json(LoginFinishResp {
        sid: sid_str,
        session_token: session.token.clone(),
    });
    
    Ok((headers, resp))
}

/// GET /id/agents/:sid - Export agent (backup)
pub async fn route_export_agent(
    State(state): State<IdState>,
    Path(sid): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let subject = id_db::get_subject_by_sid(&state.pool, &sid)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Subject not found".to_string()))?;

    Ok(Json(subject))
}

/// GET /id/agents/:sid/asc - List all ASCs for agent
pub async fn route_list_asc(
    State(state): State<IdState>,
    Path(sid): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let ascs = id_db::list_asc(&state.pool, &sid)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let resp: Vec<_> = ascs
        .into_iter()
        .map(|asc| IssueAscResp {
            asc_id: asc.asc_id.to_string(),
            sid: asc.sid,
            scopes: asc.scopes,
            not_before: asc.not_before.to_string(),
            not_after: asc.not_after.to_string(),
            signature: hex::encode(asc.signature),
        })
        .collect();

    Ok(Json(resp))
}

/// DELETE /id/agents/:sid/asc/:asc_id - Revoke ASC
pub async fn route_revoke_asc(
    State(state): State<IdState>,
    Path((sid, asc_id)): Path<(String, String)>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let asc_uuid = Uuid::parse_str(&asc_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid ASC ID".to_string()))?;

    // Verify ASC belongs to SID
    let asc = id_db::get_asc_by_id(&state.pool, asc_uuid)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "ASC not found".to_string()))?;

    if asc.sid != sid {
        return Err((StatusCode::FORBIDDEN, "ASC does not belong to this SID".to_string()));
    }

    id_db::revoke_asc(&state.pool, asc_uuid)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "message": "ASC revoked",
        "asc_id": asc_id
    })))
}

/// POST /id/stepup/begin - Begin step-up authentication for admin operations
pub async fn route_stepup_begin(
    State(state): State<IdState>,
    Json(req): Json<StepupBeginReq>,
) -> Result<Json<StepupBeginResp>, (StatusCode, String)> {
    use tracing::{info, warn};
    let start = std::time::Instant::now();
    
    // Validate existing session
    // TODO: Extract and validate session_token properly
    let username = req.username;
    
    // Get subject
    let subject = id_db::get_subject_by_username(&state.pool, &username)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, "Invalid session".to_string()))?;

    if subject.kind != "person" {
        return Err((StatusCode::BAD_REQUEST, "Step-up only for person accounts".to_string()));
    }

    // Get credentials
    let credentials = id_db::get_credentials(&state.pool, &subject.sid)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut passkeys = Vec::new();
    for cred in credentials {
        if cred.credential_kind == "passkey" {
            let passkey: Passkey = serde_json::from_slice(&cred.public_key)
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to parse passkey: {}", e)))?;
            passkeys.push(passkey);
        }
    }

    if passkeys.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "No passkey credentials found".to_string()));
    }

    // Create authentication challenge for step-up
    let (rcr, auth_state) = state.webauthn
        .start_passkey_authentication(&passkeys)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to start authentication: {:?}", e)))?;

    let auth_state_bytes = serde_json::to_vec(&auth_state)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to serialize auth state: {}", e)))?;

    // Store challenge with shorter TTL for step-up (2 minutes)
    let challenge_id = id_db::create_stepup_challenge(
        &state.pool,
        &subject.sid,
        auth_state_bytes,
        "http://localhost:8080",
        120, // 2 minutes for step-up
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    crate::metrics::WEBAUTHN_OPS.with_label_values(&["stepup", "begin"]).inc();
    info!(actor_type="person", username=%username, sid=%subject.sid, challenge_id=%challenge_id, 
          decision="accept", phase="stepup_begin", latency_ms=start.elapsed().as_millis());

    Ok(Json(StepupBeginResp {
        challenge_id: challenge_id.to_string(),
        public_key: rcr,
    }))
}

/// POST /id/stepup/finish - Finish step-up authentication
pub async fn route_stepup_finish(
    State(state): State<IdState>,
    Json(req): Json<StepupFinishReq>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    use tracing::{info, warn};
    let start = std::time::Instant::now();
    
    // Get challenge
    let challenge = id_db::get_challenge(&state.pool, &req.challenge_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "Challenge not found".to_string()))?;

    if challenge.used {
        return Err((StatusCode::BAD_REQUEST, "Challenge already used".to_string()));
    }

    if challenge.kind != "stepup" {
        return Err((StatusCode::BAD_REQUEST, "Not a step-up challenge".to_string()));
    }

    // Validate TTL
    let now = time::OffsetDateTime::now_utc();
    if now > challenge.expires_at {
        return Err((StatusCode::BAD_REQUEST, "Challenge expired".to_string()));
    }

    // Get SID from challenge
    let sid = challenge.sid.clone().ok_or_else(||
        (StatusCode::INTERNAL_SERVER_ERROR, "No SID in challenge".to_string())
    )?;

    // Parse auth state
    let auth_state: PasskeyAuthentication = serde_json::from_slice(&challenge.challenge)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Invalid auth state: {}", e)))?;

    // Verify assertion
    let auth_result = state.webauthn
        .finish_passkey_authentication(&req.assertion, &auth_state)
        .map_err(|e| (StatusCode::UNAUTHORIZED, format!("Authentication failed: {:?}", e)))?;

    // Get credential and validate sign_count
    let credential_id = URL_SAFE_NO_PAD.encode(auth_result.cred_id());
    let cred = id_db::get_credential_by_id(&state.pool, &sid, &credential_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Credential not found".to_string()))?;

    let new_counter = auth_result.counter();
    if new_counter <= cred.sign_count as u32 {
        warn!(challenge_id=%req.challenge_id, sid=%sid, decision="reject", error_code="counter_rollback", 
              old_count=%cred.sign_count, new_count=%new_counter);
        return Err((StatusCode::UNAUTHORIZED, "Counter rollback detected".to_string()));
    }

    // Update sign_count
    id_db::update_sign_count(&state.pool, cred.id, new_counter as i64)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Consume challenge
    let challenge_uuid = Uuid::parse_str(&req.challenge_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid challenge ID".to_string()))?;
    
    let webauthn_origin = std::env::var("WEBAUTHN_ORIGIN")
        .unwrap_or_else(|_| "http://localhost:8080".to_string());
    id_db::consume_challenge(&state.pool, challenge_uuid, &webauthn_origin)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Create step-up session (using new Session module)
    let sid_uuid = Uuid::parse_str(&sid)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Invalid SID format".to_string()))?;
    
    // Get user's tenant for step-up session (Zona Schengen)
    let user_tenant = tenant::db::get_user_tenant(&state.pool, &sid)
        .await
        .ok()
        .flatten();
    
    let session = Session::new_stepup_with_tenant(sid_uuid, user_tenant);
    session_db::insert(&state.pool, &session)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create step-up session: {}", e)))?;

    // Set HttpOnly cookie for step-up session
    let mut headers = HeaderMap::new();
    set_session_cookie(&mut headers, &session.token, session.ttl_secs());

    crate::metrics::ID_DECISIONS.with_label_values(&["stepup", "accept", ""]).inc();
    crate::metrics::WEBAUTHN_OPS.with_label_values(&["stepup", "finish"]).inc();
    info!(actor_type="person", sid=%sid, challenge_id=%req.challenge_id, stepup_token=%session.token,
          decision="accept", phase="stepup_finish", ttl_secs=%session.ttl_secs(), latency_ms=start.elapsed().as_millis());

    let resp = Json(StepupFinishResp {
        stepup_token: session.token.clone(),
        expires_in: session.ttl_secs(),
    });
    
    Ok((headers, resp))
}

/// POST /id/sessions/ict/begin - Begin ICTE session
pub async fn route_ict_begin(
    State(state): State<IdState>,
    Json(req): Json<IcteBeginReq>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // TODO: Extract SID from Authorization header
    let sid = "ubl:sid:placeholder"; // For now

    let session_id = id_db::create_icte_session(&state.pool, sid, req.scope, req.ttl_seconds)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let session = id_db::get_session(&state.pool, session_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::INTERNAL_SERVER_ERROR, "Session not found after creation".to_string()))?;

    Ok(Json(IcteBeginResp {
        session_id: session.session_id.to_string(),
        not_before: session.not_before.to_string(),
        not_after: session.not_after.to_string(),
    }))
}

/// POST /id/sessions/ict/finish - Close ICTE session
pub async fn route_ict_finish(
    State(state): State<IdState>,
    Json(req): Json<IcteFinishReq>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let session_uuid = Uuid::parse_str(&req.session_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid session ID".to_string()))?;

    id_db::close_icte_session(&state.pool, session_uuid)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(IcteFinishResp {
        message: "ICTE session closed".to_string(),
    }))
}

// ============================================================================
// ROUTER
// ============================================================================

pub fn id_router() -> Router<IdState> {
    Router::new()
        .route("/id/agents", post(route_create_agent))
        .route("/id/agents/:sid", get(route_export_agent))
        .route("/id/agents/:sid/asc", post(route_issue_asc))
        .route("/id/agents/:sid/asc", get(route_list_asc))
        // TODO: Apply require_stepup middleware to these routes
        // Need to refactor middleware to work with Axum layers
        .route("/id/agents/:sid/rotate", post(route_rotate_key))
        .route("/id/agents/:sid/asc/:asc_id", delete(route_revoke_asc))
        .route("/id/whoami", get(route_whoami))
        .route("/id/register/begin", post(route_register_begin))
        .route("/id/register/finish", post(route_register_finish))
        .route("/id/login/begin", post(route_login_begin))
        .route("/id/login/finish", post(route_login_finish))
        .route("/id/login/discoverable/begin", post(route_login_discoverable_begin))
        .route("/id/login/discoverable/finish", post(route_login_discoverable_finish))
        .route("/id/stepup/begin", post(route_stepup_begin))
        .route("/id/stepup/finish", post(route_stepup_finish))
        .route("/id/sessions/ict/begin", post(route_ict_begin))
        .route("/id/sessions/ict/finish", post(route_ict_finish))
        // 🆕 ASC validation endpoint (for Office to call instead of direct DB access)
        .route("/id/asc/:asc_id/validate", get(route_validate_asc))
}
