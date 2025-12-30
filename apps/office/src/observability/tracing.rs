//! # Tracing Module
//!
//! Provides tracing instrumentation for Office Runtime.
//! OpenTelemetry removed - use simple tracing spans.

use tracing::{self, Span};

/// Initialize basic tracing (no OpenTelemetry)
///
/// # Arguments
/// * `_service_name` - Name of the service (ignored, kept for API compat)
/// * `_service_version` - Version of the service (ignored)
/// * `_otlp_endpoint` - OTLP endpoint (ignored - no OpenTelemetry)
///
pub fn init_tracing(
    _service_name: &str,
    _service_version: &str,
    _otlp_endpoint: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    // OpenTelemetry removed - this is now a no-op
    // Basic tracing is initialized in main.rs with tracing-subscriber
    Ok(())
}

/// Shutdown tracing (no-op without OpenTelemetry)
pub fn shutdown_tracing() {
    // No-op
}

/// Create a span for an entity operation
pub fn create_entity_span(operation: &str, entity_id: Option<&str>) -> Span {
    tracing::span!(
        tracing::Level::INFO,
        "office.entity.operation",
        operation = operation,
        entity_id = entity_id,
    )
}

/// Create a span for a job operation
pub fn create_job_span(operation: &str, job_id: Option<&str>) -> Span {
    tracing::span!(
        tracing::Level::INFO,
        "office.job.operation",
        operation = operation,
        job_id = job_id,
    )
}

/// Create a span for an LLM call
pub fn create_llm_span(provider: &str, operation: &str) -> Span {
    tracing::span!(
        tracing::Level::INFO,
        "office.llm.call",
        provider = provider,
        operation = operation,
    )
}

/// Create a span for context frame building
pub fn create_context_span(entity_id: &str) -> Span {
    tracing::span!(
        tracing::Level::INFO,
        "office.context.build",
        entity_id = entity_id,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_creation() {
        let span = create_entity_span("entity.create", Some("entity-123"));
        assert_eq!(span.name(), "office.entity.operation");
    }
}
