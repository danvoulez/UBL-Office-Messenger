# 🔭 Observability Strategy

## Vision

**Complete system visibility** through metrics, logs, traces, and alerts to ensure reliability, performance, and rapid incident resolution.

## Three Pillars of Observability

```
┌─────────────────────────────────────────────────────────────┐
│                    OBSERVABILITY                            │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  📊 METRICS          📝 LOGS           🔍 TRACES           │
│                                                             │
│  What is wrong?      Why is it wrong?  Where is it slow?   │
│  • Counters          • Structured      • Distributed       │
│  • Gauges            • Searchable      • End-to-end        │
│  • Histograms        • Contextual      • Latency           │
│  • Dashboards        • Aggregatable    • Bottlenecks       │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Stack

- **Metrics**: Prometheus + Grafana
- **Logs**:  Loki + Promtail
- **Traces**: Jaeger + OpenTelemetry
- **Alerts**: Alertmanager + PagerDuty
- **APM**: Grafana Cloud (optional)
- **Error Tracking**: Sentry
- **Uptime**:  UptimeRobot / Pingdom

## Directory Structure

```
observability/
├── README.md
├── docker-compose.observability.yml
├── prometheus/
│   ├── prometheus.yml
│   ├── alerts/
│   │   ├── ubl-kernel. yml
│   │   ├── office.yml
│   │   ├── database.yml
│   │   └── application.yml
│   └── recording-rules/
│       ├── latency. yml
│       └── throughput.yml
├── grafana/
│   ├── provisioning/
│   │   ├── datasources/
│   │   │   ├── prometheus.yml
│   │   │   ├── loki.yml
│   │   │   └── jaeger.yml
│   │   └── dashboards/
│   │       ├── dashboard.yml
│   │       ├── system-overview.json
│   │       ├── ubl-kernel.json
│   │       ├── office-runtime.json
│   │       ├── messenger-frontend.json
│   │       ├── database.json
│   │       └── business-metrics.json
│   └── grafana.ini
├── loki/
│   └── loki-config.yml
├── promtail/
│   └── promtail-config.yml
├── jaeger/
│   └── jaeger-config.yml
├── alertmanager/
│   ├── alertmanager.yml
│   └── templates/
│       ├── slack.tmpl
│       └── pagerduty.tmpl
└── scripts/
    ├── setup-observability.sh
    ├── test-alerts.sh
    └── generate-test-load.sh
```

## Key Metrics

### Golden Signals (Per Service)

1. **Latency**: Request duration (p50, p95, p99)
2. **Traffic**: Requests per second
3. **Errors**: Error rate (4xx, 5xx)
4. **Saturation**: Resource utilization (CPU, memory, disk)

### Custom Business Metrics

- Messages sent/received per minute
- Jobs created/completed per minute
- Active users (current)
- Conversation count
- Entity count
- Ledger append rate
- Projection lag
- SSE connection count

### SLIs (Service Level Indicators)

- **Availability**: % uptime (target: 99.9%)
- **Latency**: p95 response time (target: <500ms)
- **Throughput**:  Requests/second (target: 1000+)
- **Error Rate**: % errors (target: <0.1%)

## Implementation

See: 
- [Prometheus Setup](../observability/prometheus/)
- [Grafana Dashboards](../observability/grafana/)
- [Logging Strategy](../observability/loki/)
- [Tracing Guide](../observability/jaeger/)
- [Alert Rules](../observability/prometheus/alerts/)