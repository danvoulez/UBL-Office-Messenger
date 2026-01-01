//! # Identity Error Types
//!
//! Unified error handling for the identity system.
//! Implements IntoResponse for seamless Axum integration.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

/// Identity system errors
#[derive(Debug, Error)]
pub enum IdentityError {
    // ========================================================================
    // Challenge Errors (Phase 5)
    // ========================================================================
    
    #[error("Challenge not found, expired, or already used")]
    ChallengeInvalid,
    
    #[error("Challenge not found")]
    ChallengeNotFound,
    
    #[error("Challenge expired")]
    ChallengeExpired,
    
    #[error("Invalid challenge ID format")]
    ChallengeIdInvalid,
    
    #[error("Challenge type mismatch")]
    ChallengeTypeMismatch,
    
    // ========================================================================
    // User/Subject Errors
    // ========================================================================
    
    #[error("User not found: {0}")]
    UserNotFound(String),
    
    #[error("Username already registered: {0}")]
    UserAlreadyExists(String),
    
    #[error("Invalid subject type: {0}")]
    InvalidSubjectType(String),
    
    // ========================================================================
    // Credential Errors
    // ========================================================================
    
    #[error("Credential not found")]
    CredentialNotFound,
    
    #[error("No credentials registered for user")]
    NoCredentials,
    
    #[error("No passkey credentials found")]
    NoPasskeyCredentials,
    
    // ========================================================================
    // WebAuthn Errors
    // ========================================================================
    
    #[error("WebAuthn registration failed: {0}")]
    RegistrationFailed(String),
    
    #[error("WebAuthn authentication failed: {0}")]
    AuthenticationFailed(String),
    
    #[error("Sign count rollback detected (old={old}, new={new}) - possible replay attack")]
    CounterRollback { old: u32, new: u32 },
    
    #[error("Origin mismatch: expected {expected}, got {got}")]
    OriginMismatch { expected: String, got: String },
    
    // ========================================================================
    // Session Errors (Phase 5)
    // ========================================================================
    
    #[error("Session not found or expired")]
    SessionInvalid,
    
    #[error("Session not found")]
    SessionNotFound,
    
    #[error("Session expired")]
    SessionExpired,
    
    #[error("Step-up authentication required")]
    StepUpRequired,
    
    #[error("Session has no associated SID")]
    SessionNoSid,
    
    // ========================================================================
    // ASC Errors (Phase 5)
    // ========================================================================
    
    #[error("ASC not found or expired")]
    AscInvalid,
    
    #[error("ASC not found")]
    AscNotFound,
    
    #[error("ASC not yet valid")]
    AscNotYetValid,
    
    #[error("ASC expired")]
    AscExpired,
    
    #[error("ASC scope violation: {0}")]
    AscScopeViolation(String),
    
    #[error("LLM agents cannot perform {0} operations")]
    LlmRestricted(String),
    
    // ========================================================================
    // Rate Limiting
    // ========================================================================
    
    #[error("Too many failed attempts. Locked out for {0} seconds")]
    RateLimited(i64),
    
    // ========================================================================
    // Infrastructure Errors
    // ========================================================================
    
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Invalid UUID format")]
    InvalidUuid,
    
    #[error("Internal error: {0}")]
    Internal(String),
}

impl IdentityError {
    /// Get the error code for API responses
    pub fn code(&self) -> &'static str {
        match self {
            Self::ChallengeInvalid => "CHALLENGE_INVALID",
            Self::ChallengeNotFound => "CHALLENGE_NOT_FOUND",
            Self::ChallengeExpired => "CHALLENGE_EXPIRED",
            Self::ChallengeIdInvalid => "CHALLENGE_ID_INVALID",
            Self::ChallengeTypeMismatch => "CHALLENGE_TYPE_MISMATCH",
            Self::UserNotFound(_) => "USER_NOT_FOUND",
            Self::UserAlreadyExists(_) => "USER_EXISTS",
            Self::InvalidSubjectType(_) => "INVALID_SUBJECT_TYPE",
            Self::CredentialNotFound => "CREDENTIAL_NOT_FOUND",
            Self::NoCredentials => "NO_CREDENTIALS",
            Self::NoPasskeyCredentials => "NO_PASSKEY_CREDENTIALS",
            Self::RegistrationFailed(_) => "REGISTRATION_FAILED",
            Self::AuthenticationFailed(_) => "AUTH_FAILED",
            Self::CounterRollback { .. } => "COUNTER_ROLLBACK",
            Self::OriginMismatch { .. } => "ORIGIN_MISMATCH",
            Self::SessionInvalid => "SESSION_INVALID",
            Self::SessionNotFound => "SESSION_NOT_FOUND",
            Self::SessionExpired => "SESSION_EXPIRED",
            Self::StepUpRequired => "STEPUP_REQUIRED",
            Self::SessionNoSid => "SESSION_NO_SID",
            Self::AscInvalid => "ASC_INVALID",
            Self::AscNotFound => "ASC_NOT_FOUND",
            Self::AscNotYetValid => "ASC_NOT_YET_VALID",
            Self::AscExpired => "ASC_EXPIRED",
            Self::AscScopeViolation(_) => "ASC_SCOPE_VIOLATION",
            Self::LlmRestricted(_) => "LLM_RESTRICTED",
            Self::RateLimited(_) => "RATE_LIMITED",
            Self::Database(_) => "DB_ERROR",
            Self::DatabaseError(_) => "DB_ERROR",
            Self::Serialization(_) => "SERIALIZATION_ERROR",
            Self::SerializationError(_) => "SERIALIZATION_ERROR",
            Self::InvalidUuid => "INVALID_UUID",
            Self::Internal(_) => "INTERNAL_ERROR",
        }
    }
    
    /// Get the HTTP status code
    pub fn status_code(&self) -> StatusCode {
        match self {
            // 400 Bad Request
            Self::ChallengeIdInvalid
            | Self::ChallengeTypeMismatch
            | Self::InvalidSubjectType(_)
            | Self::InvalidUuid => StatusCode::BAD_REQUEST,
            
            // 401 Unauthorized
            Self::ChallengeInvalid
            | Self::ChallengeNotFound
            | Self::ChallengeExpired
            | Self::AuthenticationFailed(_)
            | Self::CounterRollback { .. }
            | Self::OriginMismatch { .. }
            | Self::SessionInvalid
            | Self::SessionNotFound
            | Self::SessionExpired
            | Self::AscInvalid
            | Self::AscNotFound
            | Self::AscNotYetValid
            | Self::AscExpired => StatusCode::UNAUTHORIZED,
            
            // 403 Forbidden
            Self::StepUpRequired
            | Self::AscScopeViolation(_)
            | Self::LlmRestricted(_) => StatusCode::FORBIDDEN,
            
            // 404 Not Found
            Self::UserNotFound(_)
            | Self::CredentialNotFound
            | Self::NoCredentials
            | Self::NoPasskeyCredentials => StatusCode::NOT_FOUND,
            
            // 409 Conflict
            Self::UserAlreadyExists(_) => StatusCode::CONFLICT,
            
            // 429 Too Many Requests
            Self::RateLimited(_) => StatusCode::TOO_MANY_REQUESTS,
            
            // 500 Internal Server Error
            Self::Database(_)
            | Self::DatabaseError(_)
            | Self::Serialization(_)
            | Self::SerializationError(_)
            | Self::Internal(_)
            | Self::RegistrationFailed(_)
            | Self::SessionNoSid => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for IdentityError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let body = Json(json!({
            "error": self.code(),
            "message": self.to_string(),
        }));
        
        (status, body).into_response()
    }
}

/// Result type alias for identity operations
pub type IdentityResult<T> = Result<T, IdentityError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes() {
        let err = IdentityError::UserNotFound("test".into());
        assert_eq!(err.code(), "USER_NOT_FOUND");
        assert_eq!(err.status_code(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_counter_rollback() {
        let err = IdentityError::CounterRollback { old: 10, new: 5 };
        assert!(err.to_string().contains("old=10"));
        assert!(err.to_string().contains("new=5"));
    }
}
