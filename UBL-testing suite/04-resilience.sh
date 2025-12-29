#!/bin/bash
# Phase 4: Resilience Testing
set -e

echo "🛡️  Testing resilience..."

cd ../battle-testing/rust
cargo test --test resilience_tests -- --nocapture || exit 1

echo "✅ Resilience tests passed"
exit 0