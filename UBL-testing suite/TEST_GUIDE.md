# Office Runtime Test Guide

Complete testing guide for the Office LLM Operating System. 

## Table of Contents

1. [Overview](#overview)
2. [Test Organization](#test-organization)
3. [Running Tests](#running-tests)
4. [Writing Tests](#writing-tests)
5. [Test Coverage](#test-coverage)
6. [Mocking Strategy](#mocking-strategy)
7. [CI/CD Integration](#cicd-integration)
8. [Troubleshooting](#troubleshooting)

---

## Overview

The Office test suite provides comprehensive coverage across all system components: 

- **Entity Management**: Lifecycle, identity, constitution
- **Session Management**: Token budgets, handovers, modes
- **Context Building**: Memory, narratives, frames
- **Governance**: Constitution, sanity checks, dreaming, simulation
- **Job Execution**: FSM, cards, actions, lifecycle
- **LLM Integration**: Provider abstraction, routing, mocking
- **UBL Client**: Event commits, permits, state queries

**Coverage Target**: 90%+ for all production code

---

## Test Organization

```
apps/office/
├── tests/                          # Integration tests
│   ├── entity_lifecycle. rs         # Entity CRUD and lifecycle
│   ├── session_management.rs       # Sessions, budgets, handovers
│   ├── context_building.rs         # Context frames, memory, narratives
│   ├── governance.rs               # Constitution, sanity, dreaming, sim
│   ├── job_execution.rs            # Jobs, FSM, cards, actions
│   ├── llm_providers.rs            # LLM mocking and integration
│   ├── ubl_client.rs               # UBL integration
│   └── integration/
│       ├── mod.rs
│       └── full_flow.rs            # End-to-end flows
├── src/
│   ├── entity/
│   │   └── (unit tests inline)
│   ├── session/
│   │   └── (unit tests inline)
│   ├── context/
│   │   └── (unit tests inline)
│   ├── governance/
│   │   └── (unit tests inline)
│   ├── job_executor/
│   │   └── (unit tests inline)
│   └── llm/
│       └── (unit tests inline)
└── Makefile. toml                   # Task runner
```

---

## Running Tests

### Prerequisites

```bash
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install test dependencies
cargo install cargo-tarpaulin  # Coverage
cargo install cargo-watch      # Watch mode
cargo install cargo-make       # Task runner
```

### Run All Tests

```bash
cd apps/office

# Run all tests (unit + integration)
cargo test --all-features

# Run with output
cargo test --all-features -- --nocapture

# Run specific test
cargo test test_entity_creation
```

### Run Test Suites

```bash
# Unit tests only
cargo test --lib

# Integration tests only
cargo test --test '*'

# Specific integration test file
cargo test --test entity_lifecycle
cargo test --test job_execution

# Doc tests
cargo test --doc
```

### Using cargo-make

```bash
# Run all tests
cargo make test

# Run unit tests only
cargo make test-unit

# Run integration tests only
cargo make test-integration

# Run with coverage
cargo make test-coverage

# Watch mode (re-run on file changes)
cargo make test-watch
```

---

## Writing Tests

### Unit Test Example

```rust
// In src/entity/mod.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_creation() {
        let entity = Entity:: new(
            "entity_123". to_string(),
            "Test Entity".to_string(),
            EntityType::Autonomous,
        );
        
        assert_eq!(entity.id, "entity_123");
        assert_eq!(entity.name, "Test Entity");
        assert_eq!(entity.status, EntityStatus::Active);
    }
}
```

### Integration Test Example

```rust
// In tests/entity_lifecycle.rs

use office::entity::{Entity, EntityType};

#[tokio::test]
async fn test_entity_lifecycle() {
    // Create
    let mut entity = Entity::new(
        "entity_123".to_string(),
        "Test". to_string(),
        EntityType::Autonomous,
    );
    
    // Activate
    assert_eq!(entity.status, EntityStatus::Active);
    
    // Suspend
    entity.status = EntityStatus::Suspended;
    assert_eq!(entity.status, EntityStatus::Suspended);
    
    // Archive
    entity.status = EntityStatus::Archived;
    assert_eq!(entity.status, EntityStatus::Archived);
}
```

### Async Test Example

```rust
#[tokio::test]
async fn test_async_operation() {
    let result = some_async_function().await;
    assert!(result. is_ok());
}
```

### Mock LLM Provider Example

```rust
use wiremock::{MockServer, Mock, ResponseTemplate, matchers: :{method, path}};
use serde_json::json;

#[tokio::test]
async fn test_llm_provider() {
    let mock_server = MockServer::start().await;
    
    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "content": "Mocked response",
            "usage": {"total_tokens": 10}
        })))
        .mount(&mock_server)
        .await;
    
    // Test with mock_server. uri()
}
```

---

## Test Coverage

### Generate Coverage Report

```bash
cd apps/office

# Generate HTML + XML coverage
cargo tarpaulin --all-features \
    --out Html --out Xml \
    --timeout 300 \
    --exclude-files '*/tests/*'

# Open HTML report
open tarpaulin-report. html
```

### Coverage Targets

| Component | Target | Notes |
|-----------|--------|-------|
| Entity | 95% | Core domain logic |
| Session | 95% | Token management critical |
| Context | 90% | Complex narrative generation |
| Governance | 90% | Safety-critical |
| Job Executor | 90% | FSM and cards |
| LLM | 85% | External dependencies |
| UBL Client | 85% | External dependencies |
| **Overall** | **90%** | Enforced in CI |

### Coverage CI Check

Coverage is automatically checked in CI.  PRs failing coverage thresholds will be blocked.

---

## Mocking Strategy

### LLM Provider Mocking

**Why**: Avoid API costs and ensure deterministic tests

**How**:  Use `wiremock` to mock HTTP endpoints

```rust
use wiremock::{MockServer, Mock, ResponseTemplate};

#[tokio::test]
async fn test_with_mock_llm() {
    let mock_server = MockServer::start().await;
    
    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "content": "Test response",
            "model": "test-model",
            "usage": {"total_tokens": 10}
        })))
        .mount(&mock_server)
        .await;
    
    // Use mock_server.uri() as API endpoint
}
```

### UBL Client Mocking

**Why**: Test Office logic in isolation

**How**: Mock UBL HTTP endpoints

```rust
#[tokio::test]
async fn test_with_mock_ubl() {
    let mock_server = MockServer::start().await;
    
    // Mock permit endpoint
    Mock::given(method("POST"))
        .and(path("/v1/policy/permit"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "allowed": true,
            "permit":  {"jti": "test_permit"}
        })))
        .mount(&mock_server)
        .await;
    
    // Use mock_server.uri() as UBL endpoint
}
```

### Test Data Builders

**Why**: Reduce test boilerplate

**How**: Create builder functions

```rust
fn create_test_entity(name: &str) -> Entity {
    Entity::new(
        format!("entity_{}", uuid::Uuid::new_v4()),
        name.to_string(),
        EntityType::Autonomous,
    )
}

fn create_test_session(entity_id: String) -> Session {
    Session:: new(
        format!("session_{}", uuid::Uuid::new_v4()),
        entity_id,
        SessionType::Work,
        SessionMode::Commitment,
    )
}
```

---

## CI/CD Integration

### GitHub Actions Workflows

**Main Test Workflow** (`.github/workflows/office-tests.yml`):
- Runs on every push/PR to `apps/office/**`
- Unit tests
- Integration tests
- Code coverage
- Clippy linting
- Format checking

**Security Audit**:
- Checks for vulnerable dependencies
- Runs `cargo audit`

### Local CI Simulation

```bash
# Run all checks locally (same as CI)
cd apps/office

cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo tarpaulin --all-features --out Xml
```

---

## Troubleshooting

### Test Failures

**Tokio Runtime Errors**:
```
Error: there is no reactor running, must be called from the context of a Tokio runtime
```

**Solution**: Use `#[tokio::test]` instead of `#[test]`

```rust
#[tokio::test]  // ✅ Correct
async fn my_test() { }
```

### Mock Server Issues

**Port Already in Use**:
```
Error: Address already in use
```

**Solution**: MockServer automatically finds free ports.  Ensure you're not manually specifying ports.

### Coverage Discrepancies

**Coverage Lower Than Expected**: 

1. Check excluded files in `tarpaulin` command
2. Verify tests are actually running:  `cargo test -- --nocapture`
3. Check for unreachable code
4. Add `#[cfg(not(tarpaulin_include))]` to exclude test utilities

### Slow Tests

**Tests Taking Too Long**: 

1. Check for infinite loops or hangs
2. Reduce LLM mock delays
3. Use `#[ignore]` for slow tests, run separately

```rust
#[tokio::test]
#[ignore] // Run with:  cargo test -- --ignored
async fn slow_test() {
    // Long-running test
}
```

### Flaky Tests

**Tests Pass/Fail Randomly**:

1. **Check for race conditions**:  Use proper async synchronization
2. **Check time dependencies**: Mock time-sensitive operations
3. **Check external dependencies**:  Ensure mocks are deterministic
4. **Check cleanup**: Ensure tests don't leak state

---

## Best Practices

### 1. Test Naming

```rust
// ✅ Good - descriptive
#[test]
fn test_entity_status_transition_active_to_suspended()

// ❌ Bad - vague
#[test]
fn test1()
```

### 2. AAA Pattern

```rust
#[test]
fn test_example() {
    // Arrange
    let entity = create_test_entity("Test");
    
    // Act
    entity.suspend();
    
    // Assert
    assert_eq!(entity.status, EntityStatus::Suspended);
}
```

### 3. Test Independence

```rust
// ✅ Good - each test is independent
#[test]
fn test_a() {
    let entity = create_test_entity("A");
    // test logic
}

#[test]
fn test_b() {
    let entity = create_test_entity("B");
    // test logic
}
```

### 4. Assertion Messages

```rust
// ✅ Good
assert_eq!(result, expected, 
    "Job state mismatch: expected {: ?}, got {:?}", expected, result);

// ❌ Bad
assert_eq!(result, expected);
```

### 5. Use Test Utilities

```rust
// Create test utilities module
#[cfg(test)]
mod test_utils {
    pub fn create_test_entity(name: &str) -> Entity {
        // ...
    }
    
    pub fn create_test_session() -> Session {
        // ... 
    }
}
```

---

## Performance Testing

### Benchmarks

```bash
# Run benchmarks
cargo bench

# Specific benchmark
cargo bench --bench job_execution
```

### Performance Targets

| Operation | Target | Notes |
|-----------|--------|-------|
| Entity creation | < 1ms | In-memory only |
| Session creation | < 1ms | In-memory only |
| Context frame build | < 50ms | Without UBL queries |
| Job FSM transition | < 1ms | State validation |
| LLM call (mocked) | < 10ms | Mock only |

---

## Resources

- [Rust Testing Documentation](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Tokio Testing Guide](https://tokio.rs/tokio/topics/testing)
- [wiremock Documentation](https://docs.rs/wiremock/)
- [cargo-tarpaulin](https://github.com/xd009642/tarpaulin)

---

## Support

For test-related questions: 
- Check existing tests for examples
- Review this guide
- Create an issue with `testing` label

---

**Last Updated**: 2024-12-29  
**Maintainers**: Office Core Team