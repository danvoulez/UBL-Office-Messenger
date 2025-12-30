#!/bin/bash
# Generate test load for observability testing

set -e

echo "📊 Generating Test Load"
echo ""

BASE_URL=${BASE_URL:-http://localhost:8080}
DURATION=${DURATION:-60}
RATE=${RATE:-10}

echo "Configuration:"
echo "  Base URL:  $BASE_URL"
echo "  Duration: ${DURATION}s"
echo "  Rate: ${RATE} req/s"
echo ""

START_TIME=$(date +%s)
END_TIME=$((START_TIME + DURATION))
REQUEST_COUNT=0

while [ $(date +%s) -lt $END_TIME ]; do
    for i in $(seq 1 $RATE); do
        # Health check
        curl -sf "$BASE_URL/health" > /dev/null &
        
        # Bootstrap
        curl -sf "$BASE_URL/messenger/bootstrap?tenant_id=T.UBL" > /dev/null &
        
        REQUEST_COUNT=$((REQUEST_COUNT + 2))
    done
    
    sleep 1
    echo -n "."
done

wait

echo ""
echo ""
echo "✅ Load generation complete"
echo "   Total requests: $REQUEST_COUNT"
echo "   Average: $((REQUEST_COUNT / DURATION)) req/s"
echo ""
echo "📊 Check metrics in Grafana: http://localhost:3001"