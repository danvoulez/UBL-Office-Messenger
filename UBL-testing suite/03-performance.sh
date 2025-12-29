#!/bin/bash
# Phase 3: Performance Benchmarks
set -e

echo "⚡ Running performance benchmarks..."

cd ../golden-run/rust
cargo test --test baseline_performance -- --nocapture || exit 1

echo "✅ Performance benchmarks passed"
exit 0