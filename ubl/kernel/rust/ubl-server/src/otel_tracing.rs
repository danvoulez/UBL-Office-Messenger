//! # OpenTelemetry Distributed Tracing
//!
//! Provides distributed tracing instrumentation for UBL Server using OpenTelemetry.
//! Exports traces to Jaeger/OTLP collector for visualization in Grafana.
//!
use opentelemetry::global;
use opentelemetry_sdk::{
    trace::TracerProvider,
    Resource,
    runtime::Tokio,
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
/// * `service_name` - Name of the service (e.g., "ubl-server")
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

    // Create OTLP exporter - use install_simple which returns a TracerProvider
    // This is the correct pattern for opentelemetry-otlp 0.14 with opentelemetry_sdk 0.20
    let tracer_provider = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint(endpoint)
        .install_simple()?;

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

/// Create a span for a UBL operation
///
/// # Arguments
/// * `operation` - Name of the operation (e.g., "link.commit", "ledger.append")
/// * `container_id` - UBL container ID
///
/// # Returns
/// A tracing span that can be used with `#[instrument]` or manually
pub fn create_ubl_span(operation: &str, container_id: Option<&str>) -> Span {
    tracing::span!(
        tracing::Level::INFO,
        "ubl.operation",
        operation = operation,
        container_id = container_id,
    )
}

/// Create a span for a projection update
///
/// # Arguments
/// * `projection_name` - Name of the projection (e.g., "messages", "jobs")
/// * `event_type` - Type of event being processed
///
/// # Returns
/// A tracing span
pub fn create_projection_span(projection_name: &str, event_type: &str) -> Span {
    tracing::span!(
        tracing::Level::INFO,
        "ubl.projection.update",
        projection = projection_name,
        event_type = event_type,
    )
}

/// Create a span for a Gateway operation
///
/// # Arguments
/// * `operation` - Name of the operation (e.g., "post_message", "job_action")
/// * `conversation_id` - Optional conversation ID
///
/// # Returns
/// A tracing span
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
