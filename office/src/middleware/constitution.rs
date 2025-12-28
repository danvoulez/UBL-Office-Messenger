//! Office Constitution Middleware
//!
//! Enforces that Office can ONLY communicate with the UBL Gateway.
//! Blocks any attempt to:
//! - Access PostgreSQL directly
//! - Make HTTP requests outside the Gateway
//! - Perform privileged operations
//!
//! "Office não cria identidades próprias" - Office Constitution

use axum::{
    body::Body,
    http::{Request, StatusCode, Uri},
    middleware::Next,
    response::Response,
};
use once_cell::sync::Lazy;
use regex::Regex;
use tracing::{error, warn};

/// Allowed outbound HTTP destinations (UBL Gateway only)
static ALLOWLIST: Lazy<Regex> = Lazy::new(|| {
    // Only allow connections to:
    // - 10.77.0.1 (LAB 256 Gateway)
    // - console.ubl (internal DNS)
    // - localhost for dev (will be removed in production)
    Regex::new(r"^https?://(10\.77\.0\.1|console\.ubl|localhost)(:\d+)?/").unwrap()
});

/// Blocked patterns (database connection strings)
static DB_PATTERNS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(postgres|postgresql|pg|mysql|sqlite|mongodb)://").unwrap()
});

/// Constitution enforcement middleware
/// Runs on every request to validate Office is not violating boundaries
pub async fn enforce(
    req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    // 1. Check headers for DB connection attempts
    for (name, value) in req.headers().iter() {
        if let Ok(v) = value.to_str() {
            if DB_PATTERNS.is_match(v) {
                error!(
                    "🚨 CONSTITUTION VIOLATION: Database connection attempt in header {}",
                    name
                );
                return Err(StatusCode::FORBIDDEN);
            }
        }
    }

    // 2. Check request body (for POST/PUT) - would need body extraction
    // For now, we rely on the UblClient to enforce this

    // 3. Log the request for audit
    let method = req.method().clone();
    let uri = req.uri().clone();
    
    // Pass through to next handler
    let response = next.run(req).await;
    
    // 4. Log response status
    tracing::debug!(
        "Constitution audit: {} {} -> {}",
        method,
        uri,
        response.status()
    );
    
    Ok(response)
}

/// Validate an outbound HTTP URL before making the request
/// Returns Ok(()) if the URL is allowed, Err otherwise
pub fn validate_outbound_url(url: &str) -> Result<(), ConstitutionError> {
    // Check for DB patterns
    if DB_PATTERNS.is_match(url) {
        return Err(ConstitutionError::DatabaseAccess);
    }
    
    // Check allowlist
    if !ALLOWLIST.is_match(url) {
        return Err(ConstitutionError::UnauthorizedEgress(url.to_string()));
    }
    
    Ok(())
}

/// Constitution violation error
#[derive(Debug)]
pub enum ConstitutionError {
    DatabaseAccess,
    UnauthorizedEgress(String),
    PrivilegedOperation(String),
}

impl std::fmt::Display for ConstitutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DatabaseAccess => write!(f, "Database access is forbidden"),
            Self::UnauthorizedEgress(url) => write!(f, "Egress to {} is not allowed", url),
            Self::PrivilegedOperation(op) => write!(f, "Operation {} is not allowed", op),
        }
    }
}

impl std::error::Error for ConstitutionError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allowlist() {
        // Allowed
        assert!(validate_outbound_url("http://10.77.0.1:8080/v1/commands").is_ok());
        assert!(validate_outbound_url("https://console.ubl/health").is_ok());
        
        // Blocked
        assert!(validate_outbound_url("https://api.openai.com/v1/chat").is_err());
        assert!(validate_outbound_url("http://evil.com/steal").is_err());
    }

    #[test]
    fn test_db_patterns() {
        // Blocked DB patterns
        assert!(validate_outbound_url("postgres://user:pass@db:5432/ubl").is_err());
        assert!(validate_outbound_url("postgresql://localhost/test").is_err());
        
        // Allowed (not a DB URL)
        assert!(validate_outbound_url("http://10.77.0.1:8080/query/jobs").is_ok());
    }
}

