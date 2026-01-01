//! # Identity Configuration
//!
//! Centralized configuration for WebAuthn and Identity system.
//! Eliminates scattered env::var() calls across the codebase.

use std::env;
use std::sync::OnceLock;

/// Global configuration singleton
static CONFIG: OnceLock<IdentityConfig> = OnceLock::new();

/// Get the global identity configuration
pub fn config() -> &'static IdentityConfig {
    CONFIG.get_or_init(IdentityConfig::from_env)
}

/// Centralized Identity Configuration
#[derive(Debug, Clone)]
pub struct IdentityConfig {
    /// WebAuthn configuration
    pub webauthn: WebAuthnConfig,
    /// Session configuration
    pub session: SessionConfig,
    /// Rate limiting configuration
    pub rate_limit: RateLimitConfig,
}

/// WebAuthn-specific configuration
#[derive(Debug, Clone)]
pub struct WebAuthnConfig {
    /// Origin for WebAuthn (e.g., "http://localhost:8080")
    pub origin: String,
    /// Relying Party ID (e.g., "localhost")
    pub rp_id: String,
    /// Relying Party Name (e.g., "UBL")
    pub rp_name: String,
    /// Challenge TTL for registration/login (seconds)
    pub challenge_ttl_secs: i64,
    /// Challenge TTL for step-up (seconds)
    pub stepup_ttl_secs: i64,
}

/// Session configuration
#[derive(Debug, Clone)]
pub struct SessionConfig {
    /// Regular session TTL (seconds)
    pub regular_ttl_secs: i64,
    /// Step-up session TTL (seconds)
    pub stepup_ttl_secs: i64,
    /// Cookie name for session
    pub cookie_name: String,
    /// Cookie secure flag
    pub cookie_secure: bool,
    /// Cookie SameSite policy
    pub cookie_same_site: String,
}

/// Rate limiting configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Max failed attempts before lockout
    pub max_failures: u32,
    /// Lockout duration (seconds)
    pub lockout_duration_secs: i64,
}

impl IdentityConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        Self {
            webauthn: WebAuthnConfig::from_env(),
            session: SessionConfig::from_env(),
            rate_limit: RateLimitConfig::from_env(),
        }
    }
}

impl WebAuthnConfig {
    pub fn from_env() -> Self {
        Self {
            origin: env::var("WEBAUTHN_ORIGIN")
                .unwrap_or_else(|_| "http://localhost:8080".into()),
            rp_id: env::var("WEBAUTHN_RP_ID")
                .unwrap_or_else(|_| "localhost".into()),
            rp_name: env::var("WEBAUTHN_RP_NAME")
                .unwrap_or_else(|_| "UBL".into()),
            challenge_ttl_secs: env::var("WEBAUTHN_CHALLENGE_TTL")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(300), // 5 minutes
            stepup_ttl_secs: env::var("WEBAUTHN_STEPUP_TTL")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(120), // 2 minutes
        }
    }
}

impl SessionConfig {
    pub fn from_env() -> Self {
        Self {
            regular_ttl_secs: env::var("SESSION_TTL")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(3600), // 1 hour
            stepup_ttl_secs: env::var("SESSION_STEPUP_TTL")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(600), // 10 minutes
            cookie_name: env::var("SESSION_COOKIE_NAME")
                .unwrap_or_else(|_| "session".into()),
            cookie_secure: env::var("SESSION_COOKIE_SECURE")
                .map(|s| s == "true" || s == "1")
                .unwrap_or(true),
            cookie_same_site: env::var("SESSION_COOKIE_SAMESITE")
                .unwrap_or_else(|_| "Strict".into()),
        }
    }
}

impl RateLimitConfig {
    pub fn from_env() -> Self {
        Self {
            max_failures: env::var("RATE_LIMIT_MAX_FAILURES")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(5),
            lockout_duration_secs: env::var("RATE_LIMIT_LOCKOUT_SECS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(300), // 5 minutes
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = IdentityConfig::from_env();
        assert_eq!(config.webauthn.challenge_ttl_secs, 300);
        assert_eq!(config.session.regular_ttl_secs, 3600);
        assert_eq!(config.rate_limit.max_failures, 5);
    }
}
