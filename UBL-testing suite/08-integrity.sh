#!/bin/bash
# Phase 8: Data Integrity
set -e

echo "💾 Validating data integrity..."

cd ../diamond-run/rust
cargo test --test data_integrity -- --nocapture || exit 1

echo "✅ Data integrity validated"
exit 0