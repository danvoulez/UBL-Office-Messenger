#!/bin/bash
#
# 🏆 GOLDEN TEST RUNNER
# 
# Runs the complete UBL 3.0 integration test suite
#

set -e

echo ""
echo "══════════════════════════════════════════════════════════════"
echo "🏆 GOLDEN TEST - UBL 3.0 Integration Validation"
echo "══════════════════════════════════════════════════════════════"
echo ""
echo "Testing:"
echo "  • UBL Kernel (BLAKE3, Canonicalization, Domain Separation)"
echo "  • Link Commit Structure & Validation"
echo "  • Job Lifecycle State Machine"
echo "  • WebSocket Event Handling"
echo "  • UBL Ledger Events"
echo "  • Policy VM Security"
echo "  • End-to-End UBL 3.0 Flow"
echo "  • Security Primitives"
echo ""
echo "══════════════════════════════════════════════════════════════"
echo ""

cd "$(dirname "$0")/.."

# Run golden tests
echo "📦 Running Golden Tests..."
cargo test --manifest-path tests/Cargo.toml -- --nocapture

echo ""
echo "══════════════════════════════════════════════════════════════"
echo "✅ All Golden Tests Passed!"
echo "══════════════════════════════════════════════════════════════"
echo ""

# Run UBL Policy VM tests
echo "📦 Running UBL Policy VM Tests..."
cargo test --manifest-path ubl/kernel/rust/ubl-policy-vm/Cargo.toml

echo ""
echo "══════════════════════════════════════════════════════════════"
echo "✅ UBL Policy VM Tests Passed!"
echo "══════════════════════════════════════════════════════════════"
echo ""

# Run Messenger Backend Tests
echo "📦 Running Messenger Backend Tests..."
cargo test --manifest-path ubl-messenger/backend/Cargo.toml

echo ""
echo "══════════════════════════════════════════════════════════════"
echo "✅ Messenger Backend Tests Passed!"
echo "══════════════════════════════════════════════════════════════"
echo ""

echo ""
echo "══════════════════════════════════════════════════════════════"
echo "🏆 ALL GOLDEN TESTS COMPLETE!"
echo "══════════════════════════════════════════════════════════════"
echo ""
echo "UBL 3.0 Integration Verified:"
echo "  ✅ Messenger → UBL (Messages, Jobs)"
echo "  ✅ UBL → OFFICE (Audit, Execution)"
echo "  ✅ OFFICE → Messenger (Results, Cards)"
echo "  ✅ WebSocket Real-time Updates"
echo "  ✅ Cryptographic Integrity (Ed25519, BLAKE3)"
echo ""



