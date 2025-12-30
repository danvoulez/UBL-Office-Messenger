//! # Tracing Module
//!
//! Provides tracing instrumentation for UBL Server.
//! OpenTelemetry support is available with the "tracing" feature flag.

use tracing::{self, Span};

/// Initialize tracing (no-op without OpenTelemetry feature)
pub fn init_tracing(
    _service_name: &str,
    _service_version: &str,
    _otlp_endpoint: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Basic tracing is initialized in main.rs
    Ok(())
}

/// Shutdown tracing (no-op)
pub fn shutdown_tracing() {
    // No-op
}

/// Create a span for a UBL operation
pub fn create_ubl_span(operation: &str, container_id: Option<&str>) -> Span {
    tracing::span!(
        tracing::Level::INFO,
        "ubl.operation",
        operation = operation,
        container_id = container_id,
    )
}

/// Create a span for a projection update
pub fn create_projection_span(projection_name: &str, event_type: &str) -> Span {
    tracing::span!(
        tracing::Level::INFO,
        "ubl.projection.update",
        projection = projection_name,
        event_type = event_type,
    )
}

/// Create a span for a Gateway operation
pub fn create_gateway_span(operation: &str, conversation_id: Option<&str>) -> Span {
    tracing::span!(
        tracing::Level::INFO,
        "gateway.operation",
        operation = operation,
        conversation_id = conversation_id,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_creation() {
        let span = create_ubl_span("link.commit", Some("C.Messenger"));
        assert_eq!(span.name(), "ubl.operation");
    }
}
