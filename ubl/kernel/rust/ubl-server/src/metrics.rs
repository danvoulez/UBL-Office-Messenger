//! # Prometheus Metrics
//! 
//! Prometheus text format endpoint with proper content-type
//! Per Dan's patch: Must return proper Prometheus format

use axum::{routing::get, Router, response::IntoResponse};
use std::fmt::Write as _;
use prometheus::{IntCounterVec, Opts, Encoder, TextEncoder, gather};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref RATE_LIMIT_REJECTIONS: IntCounterVec = IntCounterVec::new(
        Opts::new("ubl_rate_limit_rejections", "Rate limit rejections by operation"),
        &["operation"]
    ).unwrap();
    
    pub static ref WEBAUTHN_OPS: IntCounterVec = IntCounterVec::new(
        Opts::new("ubl_webauthn_ops", "WebAuthn operations by type and phase"),
        &["type", "phase"]
    ).unwrap();
    
    pub static ref ID_DECISIONS: IntCounterVec = IntCounterVec::new(
        Opts::new("ubl_id_decisions", "Identity decisions by operation, result, and reason"),
        &["operation", "result", "reason"]
    ).unwrap();
    
    pub static ref LOCKOUT_ACTIVATIONS: IntCounterVec = IntCounterVec::new(
        Opts::new("ubl_lockout_activations", "Account lockout activations by failure count"),
        &["fail_count"]
    ).unwrap();
    
    pub static ref LEDGER_COMMITS: IntCounterVec = IntCounterVec::new(
        Opts::new("ubl_ledger_commits_total", "Ledger commits by container and intent class"),
        &["container", "intent_class"]
    ).unwrap();
    
    pub static ref POLICY_DECISIONS: IntCounterVec = IntCounterVec::new(
        Opts::new("ubl_policy_decisions_total", "Policy decisions by result"),
        &["result"]
    ).unwrap();
}

/// Metrics router - independent of AppState (no .with_state needed)
pub fn metrics_router() -> Router {
    Router::new().route("/metrics", get(metrics_handler))
}

/// Prometheus metrics handler - returns proper text/plain format
async fn metrics_handler() -> impl IntoResponse {
    let mut body = String::with_capacity(4096);
    
    // UBL-specific static metrics
    write!(&mut body, "# HELP ubl_up UBL server is up (1 = ok)\n").ok();
    write!(&mut body, "# TYPE ubl_up gauge\n").ok();
    write!(&mut body, "ubl_up 1\n\n").ok();
    
    write!(&mut body, "# HELP ubl_build_info Build information\n").ok();
    write!(&mut body, "# TYPE ubl_build_info gauge\n").ok();
    write!(&mut body, "ubl_build_info{{version=\"2.1.0\",spec=\"SPEC-UBL-LEDGER-v1.0\"}} 1\n\n").ok();
    
    // Collect all registered prometheus metrics
    let encoder = TextEncoder::new();
    let metric_families = gather();
    let mut prom_buffer = Vec::new();
    if encoder.encode(&metric_families, &mut prom_buffer).is_ok() {
        if let Ok(prom_str) = String::from_utf8(prom_buffer) {
            body.push_str(&prom_str);
        }
    }
    
    (
        axum::http::StatusCode::OK,
        [("Content-Type", "text/plain; version=0.0.4; charset=utf-8")],
        body,
    )
}
