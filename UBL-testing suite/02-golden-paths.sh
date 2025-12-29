#!/bin/bash
# Phase 2: Golden Path Scenarios
set -e

echo "🌟 Running golden path scenarios..."

cd rust
cargo test --test diamond_complete test_diamond_golden_paths -- --nocapture || exit 1

echo "✅ Golden paths passed"
exit 0