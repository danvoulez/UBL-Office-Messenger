#!/bin/bash
# CI integration script

set -e

echo "🤖 Running CI integration tests..."

# Setup
cd "$(dirname "$0")/../integration"
./setup.sh

# Run tests
EXIT_CODE=0

# Rust integration tests
echo "🧪 Rust integration tests..."
cd rust
if !  cargo test --all-features -- --test-threads=1; then
    EXIT_CODE=1
fi
cd ..

# E2E tests
echo "🎭 E2E tests..."
cd ../e2e
if ! npx playwright test; then
    EXIT_CODE=1
fi
cd ..

# Cleanup
cd integration
./teardown.sh --clean

exit $EXIT_CODE