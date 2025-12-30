# OpenTelemetry Tracing Setup Guide

## Overview

Distributed tracing has been added to all three services:
- **UBL Server** (`ubl/kernel/rust/ubl-server/src/tracing.rs`)
- **Office Runtime** (`apps/office/src/observability/tracing.rs`)
- **Messenger Frontend** (`apps/messenger/frontend/src/observability/tracing.ts`)

## Configuration

### Environment Variables

Set the OTLP endpoint to enable OpenTelemetry tracing:

```bash
# For UBL Server and Office Runtime
export OTLP_ENDPOINT="http://localhost:4317"
# Or use JAEGER_ENDPOINT (legacy)
export JAEGER_ENDPOINT="http://localhost:4317"
```

If not set, services will fall back to basic tracing (no OpenTelemetry).

### Docker Compose (Observability Stack)

The `UBL-Observability/docker-compose.observability.yml` includes an OTLP collector that receives traces on port `4317`.

## Usage

### UBL Server

Tracing is automatically initialized on startup. Use helper functions to create spans:

```rust
use ubl_server::tracing::{create_ubl_span, create_projection_span, create_gateway_span};

// In your handler
let span = create_ubl_span("link.commit", Some("C.Messenger"));
let _guard = span.enter();
// ... your code ...
```

Or use the `#[instrument]` macro:

```rust
use tracing::instrument;

#[instrument(level = "info", fields(container_id = %container_id))]
async fn commit_link(container_id: String) -> Result<()> {
    // ... your code ...
}
```

### Office Runtime

Tracing is automatically initialized on startup. Use helper functions:

```rust
use office::observability::{create_entity_span, create_job_span, create_llm_span};

// In your handler
let span = create_entity_span("entity.create", Some("entity-123"));
let _guard = span.enter();
// ... your code ...
```

Or use the `#[instrument]` macro:

```rust
use tracing::instrument;

#[instrument(level = "info", fields(entity_id = %entity_id))]
async fn create_entity(entity_id: String) -> Result<()> {
    // ... your code ...
}
```

### Messenger Frontend

Import and use the tracing module:

```typescript
import { tracer, frontendTracing } from '@/observability';

// Manual span creation
const spanId = tracer.startSpan('message.send', { conversation_id: 'conv-123' });
try {
  await sendMessage(message);
  tracer.endSpan(spanId, 'ok');
} catch (error) {
  tracer.endSpan(spanId, 'error', error.message);
}

// Or use helper functions
await frontendTracing.apiCall('/v1/conversations/123/messages', async () => {
  return await apiClient.post('/v1/conversations/123/messages', message);
});

await frontendTracing.messageSend('conv-123', async () => {
  return await sendMessage(message);
});
```

## Viewing Traces

### Jaeger UI

1. Start the observability stack:
   ```bash
   cd UBL-Observability
   docker-compose -f docker-compose.observability.yml up -d
   ```

2. Access Jaeger UI: http://localhost:16686

3. Select service:
   - `ubl-server`
   - `office-runtime`
   - `messenger-frontend`

4. Click "Find Traces" to see distributed traces

### Grafana

1. Access Grafana: http://localhost:3001

2. Import the tracing dashboard (if configured)

3. View traces in the Tempo/Jaeger data source

## Trace Propagation

Traces are automatically propagated across services via HTTP headers:
- `traceparent` (W3C Trace Context)
- `tracestate` (W3C Trace State)

The UBL Server and Office Runtime automatically extract and continue traces from incoming requests.

## Metrics Integration

Metrics are collected alongside traces:
- **UBL Server**: `ubl/kernel/rust/ubl-server/src/metrics.rs`
- **Office Runtime**: `apps/office/src/observability/metrics.rs`
- **Messenger Frontend**: `apps/messenger/frontend/src/observability/metrics.ts`

Metrics are exposed at `/metrics` endpoints (Prometheus format).

## Troubleshooting

### Traces not appearing in Jaeger

1. Check OTLP endpoint is set:
   ```bash
   echo $OTLP_ENDPOINT
   ```

2. Verify OTLP collector is running:
   ```bash
   docker ps | grep otlp
   ```

3. Check service logs for tracing errors:
   ```bash
   # UBL Server
   RUST_LOG=debug ubl-server

   # Office Runtime
   RUST_LOG=debug office
   ```

4. Verify network connectivity:
   ```bash
   curl http://localhost:4317/health
   ```

### High memory usage

If tracing causes high memory usage:
- Reduce batch size in OTLP exporter
- Increase flush interval
- Disable tracing in development (remove `OTLP_ENDPOINT`)

### Performance impact

Tracing has minimal performance impact:
- Spans are created asynchronously
- Batching reduces network overhead
- Can be disabled via environment variable

## Best Practices

1. **Use meaningful span names**: `"operation.resource.action"` format
2. **Add relevant attributes**: Include IDs, status codes, error messages
3. **Keep spans short**: Don't create spans for trivial operations
4. **Use context**: Propagate trace context across service boundaries
5. **Monitor trace volume**: Too many spans can impact performance

## Example: Full Request Flow

```
User Action (Frontend)
  └─> API Call (Frontend Span)
      └─> Gateway Handler (UBL Server Span)
          └─> Office Ingest (Office Span)
              └─> LLM Call (Office LLM Span)
                  └─> UBL Commit (UBL Server Span)
                      └─> Projection Update (UBL Server Span)
```

All spans are linked via trace context, allowing you to see the complete request flow across all services.

