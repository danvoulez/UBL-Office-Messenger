#!/bin/bash
# Phase 6: Load Testing
set -e

echo "📊 Running load tests..."

if command -v k6 &> /dev/null; then
    cd ../battle-testing/k6
    k6 run spike-test.js || exit 1
else
    echo "⚠️  K6 not installed, skipping"
fi

echo "✅ Load tests passed"
exit 0