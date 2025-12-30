# OpenTelemetry Integration Fix Plan

## Problem Statement
OpenTelemetry tracing was made optional (feature-flagged) instead of being properly implemented. The current implementation has API mismatches:
- `TonicExporterBuilder` doesn't have `install_simple()` or `install_batch()` methods
- `with_batch_exporter()` requires a runtime parameter
- Version compatibility issues between `opentelemetry-otlp 0.14` and `opentelemetry_sdk 0.21`

## Goal
Implement a **permanent, working** OpenTelemetry tracing solution that:
1. Compiles without errors
2. Works at runtime
3. Exports traces to OTLP collector
4. Integrates with `tracing-subscriber`
5. Is NOT optional - always enabled

## Research Phase

### Step 1: Verify Actual API
- [ ] Check `opentelemetry-otlp 0.14` documentation on docs.rs
- [ ] Check `opentelemetry_sdk 0.21` documentation on docs.rs
- [ ] Review OpenTelemetry Rust GitHub examples for version 0.21
- [ ] Find working examples using `opentelemetry-otlp` with `tonic` transport

### Step 2: Check Existing Working Code
- [ ] Review `ubl/kernel/rust/ubl-server/src/tracing.rs` (if it exists and works)
- [ ] Check for any other OpenTelemetry implementations in the codebase
- [ ] Identify the correct API pattern

## Implementation Phase

### Step 3: Fix Dependencies
- [ ] Remove `optional = true` from OpenTelemetry dependencies in `ubl-server/Cargo.toml`
- [ ] Remove `optional = true` from OpenTelemetry dependencies in `office/Cargo.toml`
- [ ] Remove `[features]` sections that make tracing optional
- [ ] Ensure all versions are compatible:
  - `opentelemetry = "0.21"`
  - `opentelemetry-otlp = "0.14"`
  - `opentelemetry_sdk = "0.21"`
  - `opentelemetry-semantic-conventions = "0.13"`
  - `tracing-opentelemetry = "0.21"`

### Step 4: Fix UBL Server Tracing (`ubl/kernel/rust/ubl-server/src/otel_tracing.rs`)
- [ ] Remove all `#[cfg(feature = "tracing")]` attributes
- [ ] Use correct API pattern for creating exporter
- [ ] Use correct API pattern for creating TracerProvider
- [ ] Ensure resource attributes are set correctly
- [ ] Test compilation: `cargo check` in `ubl-server`

### Step 5: Fix Office Runtime Tracing (`apps/office/src/observability/tracing.rs`)
- [ ] Remove all `#[cfg(feature = "tracing")]` attributes
- [ ] Use correct API pattern matching UBL Server
- [ ] Ensure resource attributes are set correctly
- [ ] Test compilation: `cargo check` in `office`

### Step 6: Update Main Entry Points
- [ ] Update `ubl/kernel/rust/ubl-server/src/main.rs`:
  - Remove `#[cfg(feature = "tracing")]` from tracing initialization
  - Ensure `otel_tracing::init_tracing()` is always called
- [ ] Update `apps/office/src/main.rs`:
  - Remove `#[cfg(feature = "tracing")]` from tracing initialization
  - Ensure `observability::tracing::init_tracing()` is always called

## Verification Phase

### Step 7: Compilation Tests
- [ ] `cd ubl/kernel/rust/ubl-server && cargo check` - should compile without errors
- [ ] `cd apps/office && cargo check` - should compile without errors
- [ ] Fix any remaining compilation errors

### Step 8: Runtime Verification
- [ ] Verify tracing initializes without panics
- [ ] Verify traces are exported to OTLP endpoint (if collector running)
- [ ] Verify spans are created correctly
- [ ] Test with actual HTTP requests to see traces

## Documentation Phase

### Step 9: Update Documentation
- [ ] Document the OpenTelemetry setup in relevant README files
- [ ] Note the OTLP endpoint configuration
- [ ] Explain how to view traces (Jaeger/Grafana)

## API Patterns to Research

### Pattern 1: Direct Pipeline Installation
```rust
// Check if this pattern exists:
opentelemetry_otlp::new_pipeline()
    .tracing()
    .install_simple()?;
```

### Pattern 2: Builder with Runtime
```rust
// Check if this pattern exists:
let exporter = opentelemetry_otlp::new_exporter()
    .tonic()
    .with_endpoint(endpoint)
    .build_exporter()?; // or similar method

let provider = TracerProvider::builder()
    .with_batch_exporter(exporter, runtime)
    .build();
```

### Pattern 3: Install Methods
```rust
// Check available methods on TonicExporterBuilder:
// - install_simple()?
// - install_batch()?
// - build()?
// - build_exporter()?
```

## Success Criteria
1. ✅ Both `ubl-server` and `office` compile without errors
2. ✅ No feature flags - tracing always enabled
3. ✅ Traces are exported to OTLP collector
4. ✅ Spans are created and visible in tracing output
5. ✅ No runtime panics or errors
6. ✅ Code follows OpenTelemetry best practices

## Next Steps
1. Start with Step 1: Research the actual API documentation
2. Find a working example or pattern
3. Implement the fix following the correct API
4. Test and verify



