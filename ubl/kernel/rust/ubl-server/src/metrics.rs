//! # Prometheus Metrics
//!
//! Exposes identity operation metrics for monitoring

use axum::{http::StatusCode, response::IntoResponse};
use prometheus::{Encoder, IntCounterVec, TextEncoder};

lazy_static::lazy_static! {
    /// Total identity decisions (accept/reject) by operation and error code
    pub static ref ID_DECISIONS: IntCounterVec = prometheus::register_int_counter_vec!(
        "ubl_id_decision_total",
        "Total identity decisions by operation, decision, and error code",
        &["operation", "decision", "error_code"]
    ).unwrap();
    
    /// Total WebAuthn operations by phase
    pub static ref WEBAUTHN_OPS: IntCounterVec = prometheus::register_int_counter_vec!(
        "ubl_webauthn_operations_total",
        "Total WebAuthn operations by phase (begin/finish)",
        &["operation", "phase"]
    ).unwrap();
    
    /// Rate limiting rejections
    pub static ref RATE_LIMIT_REJECTIONS: IntCounterVec = prometheus::register_int_counter_vec!(
        "ubl_rate_limit_rejections_total",
        "Total rate limit rejections by operation",
        &["operation"]
    ).unwrap();
    
    /// Progressive lockout activations
    pub static ref LOCKOUT_ACTIVATIONS: IntCounterVec = prometheus::register_int_counter_vec!(
        "ubl_progressive_lockout_total",
        "Progressive lockout activations by failure count",
        &["failure_count"]
    ).unwrap();
}

/// GET /metrics - Prometheus metrics endpoint
pub async fn metrics_handler() -> impl IntoResponse {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = Vec::new();
    
    encoder.encode(&metric_families, &mut buffer)
        .expect("Failed to encode metrics");
    
    let metrics_text = String::from_utf8(buffer)
        .expect("Metrics buffer is not valid UTF-8");
    
    (
        StatusCode::OK,
        [("Content-Type", "text/plain; version=0.0.4; charset=utf-8")],
        metrics_text,
    )
}
