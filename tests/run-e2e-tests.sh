#!/bin/bash
# Run E2E tests with Playwright

set -e

cd "$(dirname "$0")/../integration"

# Setup environment if not running
if !  curl -sf http://localhost:3000 > /dev/null 2>&1; then
    ./setup.sh
fi

# Run E2E tests
echo "🎭 Running Playwright E2E tests..."
cd ../e2e
npm install
npx playwright install
npx playwright test

# Show report
if [ "$1" = "--show-report" ]; then
    npx playwright show-report
fi

echo "✅ E2E tests complete"