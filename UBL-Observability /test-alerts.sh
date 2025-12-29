#!/bin/bash
# Test alerting system

set -e

echo "🔔 Testing Alert System"
echo ""

# Test critical alert
echo "1️⃣ Testing critical alert (service down)..."
curl -X POST http://localhost:9093/api/v1/alerts \
  -H "Content-Type: application/json" \
  -d '[
    {
      "labels": {
        "alertname": "TestCriticalAlert",
        "severity": "critical",
        "service": "test"
      },
      "annotations": {
        "summary": "Test critical alert",
        "description": "This is a test critical alert"
      },
      "startsAt": "'"$(date -u +%Y-%m-%dT%H:%M:%SZ)"'",
      "endsAt": "'"$(date -u -d '+5 minutes' +%Y-%m-%dT%H:%M:%SZ)"'"
    }
  ]'

echo "✅ Critical alert sent"
echo ""

# Test warning alert
echo "2️⃣ Testing warning alert..."
curl -X POST http://localhost:9093/api/v1/alerts \
  -H "Content-Type: application/json" \
  -d '[
    {
      "labels": {
        "alertname": "TestWarningAlert",
        "severity": "warning",
        "service": "test"
      },
      "annotations": {
        "summary": "Test warning alert",
        "description": "This is a test warning alert"
      },
      "startsAt": "'"$(date -u +%Y-%m-%dT%H:%M:%SZ)"'"
    }
  ]'

echo "✅ Warning alert sent"
echo ""

echo "📬 Check your configured channels (Slack/PagerDuty) for alerts"
echo "🔍 View alerts in Alertmanager: http://localhost:9093"