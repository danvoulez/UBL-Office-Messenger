#!/bin/bash
# UBL-FIX Verification Script
# Tests the Diamond Checklist requirements

set -e

echo "🔍 UBL Base Repetition & Liveness - Verification Script"
echo "========================================================"
echo ""

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Configuration
SERVER_URL=${SERVER_URL:-http://localhost:8080}
DATABASE_URL=${DATABASE_URL:-postgres://localhost:5432/ubl_dev}

echo "Configuration:"
echo "  Server URL: $SERVER_URL"
echo "  Database URL: $DATABASE_URL"
echo ""

# Test 1: Health Check
echo "Test 1: Health Check (DoD #3)"
echo "------------------------------"
HEALTH_RESPONSE=$(curl -s $SERVER_URL/health)
echo "Response: $HEALTH_RESPONSE"

if echo "$HEALTH_RESPONSE" | grep -q '"status":"healthy"'; then
    echo -e "${GREEN}✅ Health endpoint returns correct format${NC}"
else
    echo -e "${RED}❌ Health endpoint format incorrect${NC}"
    exit 1
fi
echo ""

# Test 2: Check SQL Migrations Applied
echo "Test 2: SQL Migrations (DoD #1, #2)"
echo "-----------------------------------"
echo "Checking if migrations have been applied..."

# Check if client_msg_id column exists
CLIENT_MSG_ID_EXISTS=$(psql $DATABASE_URL -tAc "SELECT EXISTS (
    SELECT 1 FROM information_schema.columns 
    WHERE table_name = 'projection_messages' 
    AND column_name = 'client_msg_id'
);")

if [ "$CLIENT_MSG_ID_EXISTS" = "t" ]; then
    echo -e "${GREEN}✅ client_msg_id column exists${NC}"
else
    echo -e "${RED}❌ client_msg_id column missing - run migrations${NC}"
    exit 1
fi

# Check if unique constraint exists
UNIQUE_INDEX_EXISTS=$(psql $DATABASE_URL -tAc "SELECT EXISTS (
    SELECT 1 FROM pg_indexes 
    WHERE indexname = 'uq_messages_conv_client'
);")

if [ "$UNIQUE_INDEX_EXISTS" = "t" ]; then
    echo -e "${GREEN}✅ Unique index on (conversation_id, client_msg_id) exists${NC}"
else
    echo -e "${YELLOW}⚠️  Unique index missing - idempotency not enforced${NC}"
fi

# Check if last_event_seq indexes exist
CAUSALITY_INDEXES=$(psql $DATABASE_URL -tAc "SELECT COUNT(*) FROM pg_indexes 
    WHERE indexname LIKE '%last_event_seq%';")

echo "Found $CAUSALITY_INDEXES causality indexes"
if [ "$CAUSALITY_INDEXES" -gt 0 ]; then
    echo -e "${GREEN}✅ Causality indexes present${NC}"
else
    echo -e "${YELLOW}⚠️  Causality indexes missing - run migrations${NC}"
fi
echo ""

# Test 3: Check Job Monitor Running
echo "Test 3: Job Monitor (DoD #3)"
echo "----------------------------"
echo "Check server logs for: '🔍 Job Monitor started'"
echo "This verifies the job monitor is running and will clean up orphaned jobs"
echo ""

# Test 4: Auth Anti-Replay Index
echo "Test 4: Auth Anti-Replay (DoD #4)"
echo "---------------------------------"
ANTIREPLAY_INDEX_EXISTS=$(psql $DATABASE_URL -tAc "SELECT EXISTS (
    SELECT 1 FROM pg_indexes 
    WHERE indexname = 'uq_challenge_id_used'
);")

if [ "$ANTIREPLAY_INDEX_EXISTS" = "t" ]; then
    echo -e "${GREEN}✅ Auth anti-replay index exists${NC}"
else
    echo -e "${YELLOW}⚠️  Auth anti-replay index missing - run migrations${NC}"
fi
echo ""

# Test 5: Manual Tests Required
echo "Manual Tests Required:"
echo "---------------------"
echo "1. Double-Click Protection (DoD #1):"
echo "   - Open messenger frontend"
echo "   - Send a message"
echo "   - Click send button rapidly 2-3 times"
echo "   - Verify only 1 message appears after confirmation"
echo ""
echo "2. Optimistic Rollback (DoD #5):"
echo "   - Stop the server"
echo "   - Try to send a message"
echo "   - Verify the message disappears from UI (no 'failed' ghost)"
echo "   - Restart server and verify no duplicates"
echo ""
echo "3. Causality Test (DoD #2):"
echo "   - Run: cd ubl/kernel/rust/ubl-server"
echo "   - Run: cargo test --test causality_idempotency_tests --ignored"
echo "   - All tests should pass"
echo ""

# Summary
echo "========================================================"
echo "Verification Summary:"
echo "  - Health check: ✅"
echo "  - SQL migrations: Check output above"
echo "  - Manual tests: See instructions above"
echo ""
echo "To run integration tests:"
echo "  cd ubl/kernel/rust/ubl-server"
echo "  export DATABASE_URL=$DATABASE_URL"
echo "  cargo test --test causality_idempotency_tests --ignored"
echo ""
echo "For full proof of done, see CHANGELOG_UBL_FIX.md"
echo "========================================================"
