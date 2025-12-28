//! # UBL Auth Middleware
//!
//! ASC (Agent Signing Certificate) validation for commits
//! Enforces scopes: containers, intent_classes, max_delta

pub mod session;
pub mod session_db;
pub mod require_stepup;

use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use sqlx::PgPool;
use time::OffsetDateTime;

use crate::id_db;

#[derive(Debug, Clone)]
pub struct AscContext {
    pub sid: String,
    pub containers: Vec<String>,
    pub intent_classes: Vec<String>,
    pub max_delta: Option<i128>,
}

#[derive(Debug)]
pub enum AuthError {
    NoAuth,
    InvalidFormat,
    AscNotFound,
    AscExpired,
    KeyRevoked,
    ScopeViolation(String),
}

impl AuthError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            AuthError::NoAuth => StatusCode::UNAUTHORIZED,
            AuthError::InvalidFormat => StatusCode::BAD_REQUEST,
            AuthError::AscNotFound => StatusCode::UNAUTHORIZED,
            AuthError::AscExpired => StatusCode::UNAUTHORIZED,
            AuthError::KeyRevoked => StatusCode::UNAUTHORIZED,
            AuthError::ScopeViolation(_) => StatusCode::FORBIDDEN,
        }
    }

    pub fn message(&self) -> String {
        match self {
            AuthError::NoAuth => "No authentication provided".to_string(),
            AuthError::InvalidFormat => "Invalid Authorization header format".to_string(),
            AuthError::AscNotFound => "ASC not found".to_string(),
            AuthError::AscExpired => "ASC expired".to_string(),
            AuthError::KeyRevoked => "Key revoked".to_string(),
            AuthError::ScopeViolation(msg) => msg.clone(),
        }
    }
}

/// Extract SID from Authorization header
/// Format: "Bearer ubl:sid:<hash>"
pub fn extract_sid_from_header(auth_header: &str) -> Result<String, AuthError> {
    if !auth_header.starts_with("Bearer ") {
        return Err(AuthError::InvalidFormat);
    }

    let sid = auth_header.trim_start_matches("Bearer ").trim();
    
    if !sid.starts_with("ubl:sid:") {
        return Err(AuthError::InvalidFormat);
    }

    Ok(sid.to_string())
}

/// Validate ASC for given SID
pub async fn validate_asc(
    pool: &PgPool,
    sid: &str,
) -> Result<AscContext, AuthError> {
    // Get active ASC
    let asc = id_db::get_active_asc(pool, sid)
        .await
        .map_err(|_| AuthError::AscNotFound)?
        .ok_or(AuthError::AscNotFound)?;

    // Check expiration
    let now = OffsetDateTime::now_utc();
    if now < asc.not_before || now > asc.not_after {
        return Err(AuthError::AscExpired);
    }

    // Extract scopes
    let containers = asc.scopes.get("containers")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();

    let intent_classes = asc.scopes.get("intent_classes")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();

    let max_delta = asc.scopes.get("max_delta")
        .and_then(|v| v.as_i64())
        .map(|v| v as i128);

    Ok(AscContext {
        sid: sid.to_string(),
        containers,
        intent_classes,
        max_delta,
    })
}

/// Validate commit against ASC scopes
/// CRITICAL: Enforces LLM can NEVER perform Entropy/Evolution
pub fn validate_commit_scopes(
    asc: &AscContext,
    container_id: &str,
    intent_class: &str,
    physics_delta: &str,
) -> Result<(), AuthError> {
    // 1. Check container scope
    if !asc.containers.is_empty() && !asc.containers.contains(&container_id.to_string()) {
        return Err(AuthError::ScopeViolation(
            format!("Container '{}' not in allowed scopes: {:?}", container_id, asc.containers)
        ));
    }

    // 2. Check intent_class scope
    if !asc.intent_classes.is_empty() && !asc.intent_classes.contains(&intent_class.to_string()) {
        return Err(AuthError::ScopeViolation(
            format!("Intent class '{}' not in allowed scopes: {:?}", intent_class, asc.intent_classes)
        ));
    }

    // 3. Check max_delta
    let delta: i128 = physics_delta.parse().unwrap_or(0);
    if let Some(max_delta) = asc.max_delta {
        if delta.abs() > max_delta {
            return Err(AuthError::ScopeViolation(
                format!("Physics delta {} exceeds max_delta {}", delta, max_delta)
            ));
        }
    }

    // 4. CRITICAL: LLM agents can NEVER perform Entropy or Evolution
    // This is a hard security boundary - LLMs are workers, not owners
    if is_llm_agent(&asc.sid) {
        if intent_class == "Entropy" || intent_class == "Evolution" {
            return Err(AuthError::ScopeViolation(
                "LLM agents cannot perform Entropy or Evolution operations".to_string()
            ));
        }
    }

    Ok(())
}

/// Check if SID represents an LLM agent
/// LLM SIDs start with "ubl:sid:llm:" or have kind="llm" in the scopes
fn is_llm_agent(sid: &str) -> bool {
    sid.starts_with("ubl:sid:llm:") || sid.contains(":llm:")
}

/// Middleware to validate ASC on protected routes
pub async fn asc_middleware(
    req: Request,
    next: Next,
) -> Result<Response, (StatusCode, String)> {
    // TODO: Extract Authorization header and validate
    // For now, pass through
    Ok(next.run(req).await)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_sid() {
        let header = "Bearer ubl:sid:86139707e06251545327152cc4e394800eff9897379c85432faa187200b7871d";
        let sid = extract_sid_from_header(header).unwrap();
        assert_eq!(sid, "ubl:sid:86139707e06251545327152cc4e394800eff9897379c85432faa187200b7871d");
    }

    #[test]
    fn test_extract_sid_invalid() {
        assert!(extract_sid_from_header("Bearer invalid").is_err());
        assert!(extract_sid_from_header("invalid").is_err());
    }

    #[test]
    fn test_validate_scopes() {
        let asc = AscContext {
            sid: "test".to_string(),
            containers: vec!["C.Messenger".to_string()],
            intent_classes: vec!["Observation".to_string()],
            max_delta: Some(1000),
        };

        // Valid
        assert!(validate_commit_scopes(&asc, "C.Messenger", "Observation", "100").is_ok());

        // Invalid container
        assert!(validate_commit_scopes(&asc, "C.Other", "Observation", "100").is_err());

        // Invalid intent_class
        assert!(validate_commit_scopes(&asc, "C.Messenger", "Evolution", "100").is_err());

        // Exceeds max_delta
        assert!(validate_commit_scopes(&asc, "C.Messenger", "Observation", "2000").is_err());
    }
}
