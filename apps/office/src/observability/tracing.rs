//! # OpenTelemetry Distributed Tracing
//!
//! Provides distributed tracing instrumentation for Office Runtime using OpenTelemetry.
//!
use opentelemetry::global;
use opentelemetry_sdk::{
    trace::TracerProvider,
    Resource,
};
use opentelemetry_semantic_conventions::resource::{
    SERVICE_NAME, SERVICE_VERSION,
};
use opentelemetry_otlp::WithExportConfig;
use tracing::{self, Span};
use tracing_subscriber::{
    layer::SubscriberExt,
    util::SubscriberInitExt,
    Registry,
};
use tracing_opentelemetry::OpenTelemetryLayer;

/// Initialize OpenTelemetry tracing with OTLP exporter
///
/// # Arguments
/// * `service_name` - Name of the service (e.g., "office-runtime")
/// * `service_version` - Version of the service
/// * `otlp_endpoint` - OTLP collector endpoint (default: http://localhost:4317)
///
/// # Returns
/// `Result<(), Box<dyn std::error::Error>>` - Error if initialization fails
///
pub fn init_tracing(
    service_name: &str,
    service_version: &str,
    otlp_endpoint: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let endpoint = otlp_endpoint.unwrap_or("http://localhost:4317");
    
    // Create resource with service information
    let resource = Resource::new(vec![
        SERVICE_NAME.string(service_name.to_string()),
        SERVICE_VERSION.string(service_version.to_string()),
    ]);

    // Create OTLP exporter builder
    let exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint(endpoint);

    // Create tracer provider with simple exporter (not batch)
    // Note: with_simple_exporter doesn't require runtime parameter
    let tracer_provider = TracerProvider::builder()
        .with_simple_exporter(exporter)
        .with_resource(resource)
        .build();

    // Set global tracer provider
    global::set_tracer_provider(tracer_provider.clone());

    // Get tracer from provider
    let tracer = tracer_provider.tracer(service_name);

    // Create OpenTelemetry layer for tracing-subscriber
    let telemetry = OpenTelemetryLayer::new(tracer);

    // Initialize tracing subscriber with OpenTelemetry layer
    Registry::default()
        .with(telemetry)
        .with(
            tracing_subscriber::fmt::Layer::default()
                .with_target(false)
                .with_thread_ids(true)
                .with_file(true)
                .with_line_number(true),
        )
        .init();

    Ok(())
}


/// Shutdown OpenTelemetry tracing
///
/// Should be called on application shutdown to flush remaining spans
pub fn shutdown_tracing() {
    opentelemetry::global::shutdown_tracer_provider();
}

/// Create a span for an entity operation
///
/// # Arguments
/// * `operation` - Name of the operation (e.g., "entity.create", "entity.update")
/// * `entity_id` - Optional entity ID
///
/// # Returns
/// A tracing span
pub fn create_entity_span(operation: &str, entity_id: Option<&str>) -> Span {
    tracing::span!(
        tracing::Level::INFO,
        "office.entity.operation",
        operation = operation,
        entity_id = entity_id,
    )
}

/// Create a span for a job operation
///
/// # Arguments
/// * `operation` - Name of the operation (e.g., "job.create", "job.execute")
/// * `job_id` - Optional job ID
///
/// # Returns
/// A tracing span
pub fn create_job_span(operation: &str, job_id: Option<&str>) -> Span {
    tracing::span!(
        tracing::Level::INFO,
        "office.job.operation",
        operation = operation,
        job_id = job_id,
    )
}

/// Create a span for an LLM call
///
/// # Arguments
/// * `provider` - LLM provider name (e.g., "anthropic", "openai")
/// * `operation` - Operation type (e.g., "chat", "completion")
///
/// # Returns
/// A tracing span
pub fn create_llm_span(provider: &str, operation: &str) -> Span {
    tracing::span!(
        tracing::Level::INFO,
        "office.llm.call",
        provider = provider,
        operation = operation,
    )
}

/// Create a span for context frame building
///
/// # Arguments
/// * `entity_id` - Entity ID
///
/// # Returns
/// A tracing span
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
