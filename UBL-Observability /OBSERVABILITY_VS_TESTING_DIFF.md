# Observability vs Testing Suite - File Differences

## Observability-Specific Files (KEEP)

These files are unique to observability and should remain:

### Documentation
- `OBSERVABILITY_STRATEGY.md` - Observability strategy and vision
- `EVENT_SOURCING_OBSERVABILITY.md` - Event sourcing observability guide
- `CRYPTOGRAPHY_OBSERVABILITY.md` - Cryptography observability guide
- `CRYPTO_VALIDATION_RUNBOOK.md` - Crypto validation runbook
- `service-down.md` - Service down runbook

### Configuration Files
- `docker-compose.observability.yml` - Observability stack (Prometheus, Grafana, Loki, Jaeger)
- `alertmanager.yml` - Alertmanager configuration
- `prometheus.yml` - Prometheus configuration
- `loki-config.yml` - Loki log aggregation config
- `jaeger-config.yml` - Jaeger tracing config
- `promtail-config.*` - Promtail log shipper config
- `ubl-kernel.json` - Grafana dashboard for UBL Kernel
- `ubl-kernel.yml` - Prometheus alerts for UBL Kernel
- `office-runtime.json` - Grafana dashboard for Office
- `office.yml` - Prometheus alerts for Office
- `database.yml` - Prometheus alerts for database
- `application.yml` - Prometheus alerts for application
- `system-overview.json` - System overview dashboard
- `slack.tmpl` - Slack alert template

### Scripts
- `setup-observability.sh` - Setup observability stack
- `test-alerts.sh` - Test alerting system
- `generate-test-load.sh` - Generate test load for observability

### Directories
- `observability/` - Observability-specific subdirectories
- `runbooks/` - Operational runbooks

## Testing Files (REMOVE)

These files are testing-related and should be removed (they exist in UBL-testing suite):

### Test Scripts
- `01-foundation.` / `01-foundation.sh`
- `02-golden-paths.sh`
- `03-performance.sh`
- `04-resilience.sh`
- `06-load.sh`
- `08-integrity.sh`
- `run-diamond-suite.` / `run-diamond-suite.sh`
- `run-integration-tests.` / `run-integration-tests.sh`
- `run-load-tests.` / `run-load-tests.sh`
- `run-chaos-suite.sh`
- `run-e2e-tests.sh`
- `setup.sh` (testing setup)
- `teardown.sh` (testing teardown)
- `calculate-resilience-score.sh`
- `inject-faults.sh`
- `ci-integration.sh`

### Docker Compose (Testing)
- `docker-compose.chaos.yml`
- `docker-compose.diamond.yml`
- `docker-compose.golden.yml`
- `docker-compose.integration.yml`
- `docker-compose.test.yml`

### Test Configuration Files
- `cascading-failure.yml`
- `database-failure.yml`
- `happy-path.yml`
- `high-latency.yml`
- `network-partition.yml`
- `resource-exhaustion.yml`
- `service-crash.yml`

### Load Test Scripts
- `concurrent-users.js`
- `job-load.js`
- `message-load.js`
- `soak-test.js`
- `spike-test.js`
- `stress-test.js`

### Testing Infrastructure
- `Cargo.toml` (testing crate)
- `Dockerfile` (testing)
- `Dockerfile.test`
- `Makefile.toml` (testing tasks)
- `package.json` (testing dependencies)
- `vitest.config.ts`
- `setupTests.ts`
- `playwright.config.ts`
- `TEST_GUIDE.md`
- `TESTING_QUICKSTART.`
- `README.md` (if it's about testing)
- `README.` (if it's about testing)

### Test Directories
- `tests/` - Rust integration tests
- `__tests__/` - Frontend tests
- `__mocks__/` - Test mocks
- `src/` - Test source code
- `ubl-atom/` - Atom tests
- `ubl-kernel/` - Kernel tests
- `ubl-link/` - Link tests
- `ubl-membrane/` - Membrane tests
- `ubl-policy-vm/` - Policy VM tests
- `ubl-server/` - Server tests
- `scripts/` (if testing-related)

## Summary

**Keep**: Observability-specific documentation, configs, dashboards, and scripts  
**Remove**: All testing infrastructure, test scripts, and test configurations

