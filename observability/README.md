# 🔭 UBL Observability Suite

Complete observability infrastructure for the UBL + Office + Messenger system.

## Overview

This directory contains all observability-related configurations, dashboards, and runbooks for monitoring, logging, tracing, and alerting across the three-system architecture.

## Components

### Metrics & Monitoring
- **Prometheus** - Metrics collection and storage
- **Grafana** - Visualization and dashboards
- **Alertmanager** - Alert routing and notification

### Logging
- **Loki** - Log aggregation
- **Promtail** - Log shipper

### Tracing
- **Jaeger** - Distributed tracing

## Quick Start

```bash
# Setup observability stack
./setup-observability.sh

# Start services
docker-compose -f docker-compose.observability.yml up -d

# Access dashboards
# Grafana: http://localhost:3001 (admin/admin)
# Prometheus: http://localhost:9090
# Jaeger: http://localhost:16686
# Loki: http://localhost:3100
```

## Documentation

- [OBSERVABILITY_STRATEGY.md](./OBSERVABILITY_STRATEGY.md) - Overall observability strategy
- [EVENT_SOURCING_OBSERVABILITY.md](./EVENT_SOURCING_OBSERVABILITY.md) - Event sourcing observability guide
- [CRYPTOGRAPHY_OBSERVABILITY.md](./CRYPTOGRAPHY_OBSERVABILITY.md) - Cryptography observability guide
- [CRYPTO_VALIDATION_RUNBOOK.md](./CRYPTO_VALIDATION_RUNBOOK.md) - Crypto validation runbook

## Directory Structure

```
UBL-Observability/
├── README.md                          # This file
├── OBSERVABILITY_STRATEGY.md          # Strategy document
├── EVENT_SOURCING_OBSERVABILITY.md    # Event sourcing guide
├── CRYPTOGRAPHY_OBSERVABILITY.md      # Crypto observability
├── CRYPTO_VALIDATION_RUNBOOK.md       # Crypto runbook
├── docker-compose.observability.yml    # Observability stack
├── setup-observability.sh             # Setup script
├── test-alerts.sh                     # Alert testing
├── generate-test-load.sh               # Load generation
├── prometheus.yml                     # Prometheus config
├── alertmanager.yml                   # Alertmanager config
├── loki-config.yml                    # Loki config
├── jaeger-config.yml                  # Jaeger config
├── promtail-config.*                  # Promtail config
├── ubl-kernel.json                    # UBL Kernel dashboard
├── ubl-kernel.yml                     # UBL Kernel alerts
├── office-runtime.json                 # Office dashboard
├── office.yml                         # Office alerts
├── database.yml                       # Database alerts
├── application.yml                    # Application alerts
├── system-overview.json                # System overview dashboard
├── slack.tmpl                         # Slack alert template
├── service-down.md                     # Service down runbook
├── observability/                     # Additional observability configs
└── runbooks/                          # Operational runbooks
```

## Key Metrics

### Golden Signals
- **Latency**: Request duration (p50, p95, p99)
- **Traffic**: Requests per second
- **Errors**: Error rate (4xx, 5xx)
- **Saturation**: Resource utilization

### Business Metrics
- Messages sent/received per minute
- Jobs created/completed per minute
- Active users
- Conversation count
- Entity count
- Ledger append rate
- Projection lag
- SSE connection count

## Alerts

Alerts are configured in:
- `ubl-kernel.yml` - UBL Kernel alerts
- `office.yml` - Office runtime alerts
- `database.yml` - Database alerts
- `application.yml` - Application alerts

Alert routing is configured in `alertmanager.yml`.

## Dashboards

Grafana dashboards:
- `system-overview.json` - System overview
- `ubl-kernel.json` - UBL Kernel metrics
- `office-runtime.json` - Office runtime metrics

## Testing

```bash
# Test alerting system
./test-alerts.sh

# Generate test load
./generate-test-load.sh
```

## Maintenance

See runbooks in `runbooks/` directory for operational procedures.

