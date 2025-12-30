use axum::{extract::State, Json, routing::post, Router};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use axum::http::StatusCode;
use crate::AppState;
use rand::RngCore;
use base64ct::{Base64UrlUnpadded, Encoding};
use serde_json::json;
use std::sync::Arc;
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use once_cell::sync::OnceCell;
use sqlx::PgPool;

/// Extract SID from Authorization header or session cookie
async fn extract_sid_from_headers(pool: &PgPool, headers: &axum::http::HeaderMap) -> Result<String, String> {
    // Try Authorization: Bearer <token>
    if let Some(auth) = headers.get("authorization") {
        let auth_str = auth.to_str().map_err(|_| "Invalid Authorization header".to_string())?;
        if auth_str.starts_with("Bearer ") {
            let token = auth_str.trim_start_matches("Bearer ").trim();
            // Look up session by token
            let row = sqlx::query("SELECT sid FROM id_session WHERE token = $1 AND expires_at > NOW()")
                .bind(token)
                .fetch_optional(pool)
                .await
                .map_err(|e| format!("DB error: {}", e))?;
            if let Some(row) = row {
                let sid: String = sqlx::Row::get(&row, "sid");
                return Ok(sid);
            }
        }
    }
    
    // Try session cookie
    if let Some(cookie) = headers.get("cookie") {
        let cookie_str = cookie.to_str().map_err(|_| "Invalid cookie".to_string())?;
        for part in cookie_str.split(';') {
            let part = part.trim();
            if part.starts_with("ubl_session=") {
                let token = part.trim_start_matches("ubl_session=");
                let row = sqlx::query("SELECT sid FROM id_session WHERE token = $1 AND expires_at > NOW()")
                    .bind(token)
                    .fetch_optional(pool)
                    .await
                    .map_err(|e| format!("DB error: {}", e))?;
                if let Some(row) = row {
                    let sid: String = sqlx::Row::get(&row, "sid");
                    return Ok(sid);
                }
            }
        }
    }
    
    Err("No valid session found. Please authenticate first.".to_string())
}

#[derive(Debug, Deserialize)]
pub struct TokenBody {
    pub aud: String,                // "ubl://cli" | "ubl://sdk" | etc
    #[serde(default)]
    pub scope: Vec<String>,         // ["read","write","admin"]
    #[serde(default = "default_ttl")]
    pub ttl_secs: i64,              // 3600 by default
}
fn default_ttl() -> i64 { 3600 }

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    iss: String,
    sub: String,   // SID
    aud: String,
    scope: Vec<String>,
    flavor: String, // "regular" | "stepup"
    exp: i64,
    iat: i64,
    jti: String,
}

static ED_PRIV: OnceCell<EncodingKey> = OnceCell::new();
static KID: OnceCell<String> = OnceCell::new();

fn ensure_signing_key() -> Result<(&'static EncodingKey, &'static str), String> {
    if ED_PRIV.get().is_none() {
        // Expect PEM via env; if absent, return error
        if let Ok(pem) = std::env::var("JWT_ED25519_PEM") {
            let key = EncodingKey::from_ed_pem(pem.as_bytes()).map_err(|e| e.to_string())?;
            let _ = ED_PRIV.set(key);
        } else {
            return Err("JWT_ED25519_PEM environment variable required. Generate with: ssh-keygen -t ed25519 -f jwt-key -N '' && cat jwt-key".to_string());
        }
    }
    if KID.get().is_none() {
        let kid = std::env::var("JWT_KID").unwrap_or_else(|_| "ubl-ed25519-v1".into());
        let _ = KID.set(kid);
    }
    Ok((ED_PRIV.get().unwrap(), KID.get().unwrap()))
}

pub fn router() -> Router<AppState> {
    Router::new().route("/id/session/token", post(route_issue_token))
}

/// Issues a JWT Bearer token bound to the current session (SID).
/// Requires step-up if scope contains "admin".
async fn route_issue_token(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(body): Json<TokenBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let (key, kid) = ensure_signing_key().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    // Extract SID from Authorization header (Bearer token) or session cookie
    let sid = extract_sid_from_headers(&state.pool, &headers).await
        .map_err(|e| (StatusCode::UNAUTHORIZED, e))?;
    let flavor = if headers.get("x-ubl-stepup").is_some() { "stepup" } else { "regular" }.to_string();

    if body.scope.iter().any(|s| s == "admin") && flavor != "stepup" {
        return Err((StatusCode::FORBIDDEN, "step-up required for admin scope".into()));
    }

    let now = OffsetDateTime::now_utc().unix_timestamp();
    let exp = now + body.ttl_secs.max(60);
    let mut rnd = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut rnd);
    let jti = Base64UrlUnpadded::encode_string(&rnd);

    let claims = Claims {
        iss: "ubl-id".into(),
        sub: sid.clone(),
        aud: body.aud.clone(),
        scope: body.scope.clone(),
        flavor,
        iat: now,
        exp,
        jti,
    };

    let mut header = Header::new(Algorithm::EdDSA);
    header.kid = Some(kid.to_string());
    let token = encode(&header, &claims, key).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(json!({
        "access_token": token,
        "expires_in": body.ttl_secs,
        "token_type": "Bearer",
        "kid": kid
    })))
}
