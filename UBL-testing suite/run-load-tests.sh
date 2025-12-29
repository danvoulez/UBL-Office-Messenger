#!/bin/bash
# Run load tests with K6

set -e

cd "$(dirname "$0")/../integration"

# Setup environment if not running
if ! curl -sf http://localhost:8080/health > /dev/null 2>&1; then
    ./setup.sh
fi

# Check K6
if !  command -v k6 &> /dev/null; then
    echo "❌ K6 not installed.  Install from https://k6.io/docs/getting-started/installation"
    exit 1
fi

# Run load tests
echo "📊 Running K6 load tests..."
cd ../load/k6

echo "1️⃣  Message load test..."
k6 run message-load.js

echo "2️⃣  Job load test..."
k6 run job-load.js

echo "3️⃣  Concurrent users test..."
k6 run concurrent-users.js

echo "✅ Load tests complete"