#!/bin/bash
# Run full integration test suite

set -e

cd "$(dirname "$0")/../integration"

# Setup environment
./setup.sh

# Run Rust integration tests
echo "🧪 Running Rust integration tests..."
cd rust
cargo test --all-features -- --test-threads=1 --nocapture

# Cleanup
cd ..
if [ "$1" != "--no-cleanup" ]; then
    ./teardown.sh
fi

echo "✅ Integration tests complete"