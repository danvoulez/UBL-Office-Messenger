use axum::{extract::State, http::StatusCode, response::Response};
use axum::http::Request;
use axum::body::Body;
use crate::AppState;

// Placeholder middleware - will be properly implemented with session validation
pub async fn require_stepup(
    State(_state): State<AppState>,
    req: Request<Body>,
    next: axum::middleware::Next,
) -> Result<Response, (StatusCode, String)> {
    // TODO: Extract session token from Authorization header or cookie
    // TODO: Validate flavor=stepup via session_db
    // For now, pass through (will implement after session integration)
    Ok(next.run(req).await)
}
