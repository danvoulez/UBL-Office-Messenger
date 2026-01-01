#!/bin/bash
# ============================================================================
# test-auth-integration.sh
# Tests the auth refactor integration across UBL Kernel and Office
# ============================================================================

echo "🔐 Auth Integration Tests"
echo "========================="
echo ""

UBL_URL="${UBL_URL:-http://localhost:8080}"
OFFICE_URL="${OFFICE_URL:-http://localhost:8081}"

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

passed=0
failed=0

test_endpoint() {
    local name="$1"
    local method="$2"
    local url="$3"
    local expected_status="$4"
    local data="$5"
    local headers="$6"
    
    if [ "$method" = "GET" ]; then
        actual_status=$(curl -s -o /dev/null -w "%{http_code}" --connect-timeout 5 $headers "$url" 2>/dev/null)
    else
        actual_status=$(curl -s -o /dev/null -w "%{http_code}" --connect-timeout 5 -X "$method" -H "Content-Type: application/json" $headers -d "$data" "$url" 2>/dev/null)
    fi
    
    if [ "$actual_status" = "$expected_status" ]; then
        echo -e "${GREEN}✓${NC} $name (HTTP $actual_status)"
        passed=$((passed + 1))
    else
        echo -e "${RED}✗${NC} $name (expected $expected_status, got $actual_status)"
        failed=$((failed + 1))
    fi
}

echo "📡 Testing UBL Kernel ($UBL_URL)"
echo "--------------------------------"

# Test 1: Health check
test_endpoint "UBL Health" "GET" "$UBL_URL/health" "200"

# Test 2: Whoami without auth (should return 200 with authenticated: false)
test_endpoint "Whoami (no auth)" "GET" "$UBL_URL/id/whoami" "200"

# Test 3: ASC validate with invalid ID (should return 404)
test_endpoint "ASC Validate (invalid)" "GET" "$UBL_URL/id/asc/00000000-0000-0000-0000-000000000000/validate" "404"

# Test 4: Register begin (should work)
test_endpoint "Register Begin" "POST" "$UBL_URL/id/register/begin" "200" \
    '{"username":"test_'$(date +%s)'","display_name":"Test User"}'

echo ""
echo "📡 Testing Office ($OFFICE_URL)"
echo "-------------------------------"

# Test 5: Office health
test_endpoint "Office Health" "GET" "$OFFICE_URL/health" "200"

echo ""
echo "================================"
echo "Results: $passed passed, $failed failed"

if [ $failed -gt 0 ]; then
    echo -e "${RED}Some tests failed!${NC}"
    exit 1
else
    echo -e "${GREEN}All tests passed!${NC}"
fi
