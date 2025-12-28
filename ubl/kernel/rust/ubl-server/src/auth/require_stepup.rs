use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use tracing::warn;

use crate::auth::session::{Session, SessionFlavor};
use crate::auth::session_db;
use crate::id_routes::IdState;

pub async fn require_stepup(
    State(state): State<IdState>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, (StatusCode, String)> {
    let start = std::time::Instant::now();

    let token = extract_token(req.headers())
        .or_else(|| extract_cookie(&req, "session"))
        .ok_or_else(|| {
            warn!(
                decision = "reject",
                error_code = "missing_session",
                latency_ms = start.elapsed().as_millis() as u64
            );
            (StatusCode::UNAUTHORIZED, "missing session".to_string())
        })?;

    let sess = session_db::get_valid(&state.pool, &token)
        .await
        .map_err(|e| {
            warn!(
                decision = "reject",
                error_code = "db_error",
                error = %e,
                latency_ms = start.elapsed().as_millis() as u64
            );
            (StatusCode::INTERNAL_SERVER_ERROR, "db error".to_string())
        })?
        .ok_or_else(|| {
            warn!(
                decision = "reject",
                error_code = "invalid_or_expired_session",
                latency_ms = start.elapsed().as_millis() as u64
            );
            (
                StatusCode::UNAUTHORIZED,
                "invalid or expired session".to_string(),
            )
        })?;

    // Precisa ser step-up + admin
    let is_stepup = matches!(sess.flavor, SessionFlavor::StepUp);
    let is_admin = sess
        .scope
        .get("role")
        .and_then(|v| v.as_str())
        .map(|r| r == "admin")
        .unwrap_or(false);

    if !(is_stepup && is_admin) {
        warn!(
            decision = "reject",
            error_code = "stepup_required",
            flavor = ?sess.flavor,
            has_admin_role = is_admin,
            latency_ms = start.elapsed().as_millis() as u64
        );
        return Err((StatusCode::FORBIDDEN, "step-up required".to_string()));
    }

    // Sessão válida - adiciona ao request
    req.extensions_mut().insert(sess);
    Ok(next.run(req).await)
}

fn extract_token(headers: &axum::http::HeaderMap) -> Option<String> {
    let auth = headers.get(axum::http::header::AUTHORIZATION)?.to_str().ok()?;
    auth.strip_prefix("Bearer ").map(|s| s.to_string())
}

fn extract_cookie(req: &Request<Body>, name: &str) -> Option<String> {
    req.headers()
        .get("cookie")?
        .to_str()
        .ok()?
        .split(';')
        .filter_map(|p| {
            let mut kv = p.trim().splitn(2, '=');
            Some((kv.next()?, kv.next()?))
        })
        .find(|(k, _)| *k == name)
        .map(|(_, v)| v.to_string())
}
