#!/bin/bash
# Teardown script for integration tests

set -e

echo "🧹 Tearing down integration test environment..."

# Colors
GREEN='\033[0;32m'
NC='\033[0m'

# Stop all containers
echo "🛑 Stopping containers..."
docker-compose -f docker-compose.integration.yml down

# Remove volumes (optional - comment out to preserve data)
if [ "$1" = "--clean" ]; then
    echo "🗑️  Removing volumes..."
    docker-compose -f docker-compose.integration.yml down -v
fi

# Remove test artifacts
echo "🗑️  Removing test artifacts..."
rm -rf rust/target/debug/deps/*test* 2>/dev/null || true
rm -rf e2e/test-results 2>/dev/null || true
rm -rf e2e/playwright-report 2>/dev/null || true

echo ""
echo -e "${GREEN}✅ Teardown complete${NC}"