#!/bin/bash
#
# 🔒 LEDGER INTEGRITY VERIFIER
# 
# Verifies the chain integrity of the UBL ledger
# Usage: ./verify_ledger.sh [container_id]
#

set -e

DB_URL="${DATABASE_URL:-postgres://localhost:5432/ubl_ledger}"
CONTAINER="${1:-}"

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "🔒 UBL LEDGER INTEGRITY VERIFIER"
echo "═══════════════════════════════════════════════════════════════"
echo ""
echo "Database: $DB_URL"
echo "Container: ${CONTAINER:-ALL}"
echo ""

# Run Rust verifier
cd "$(dirname "$0")/.."

source "$HOME/.cargo/env" 2>/dev/null || true

cargo run --manifest-path ubl/kernel/rust/ubl-server/Cargo.toml --bin verify-ledger -- ${CONTAINER:+--container $CONTAINER}



