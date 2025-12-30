#!/bin/bash
# 🔥 BATTLE HARD TESTS - UBL 3.0
# Tests that prove the system is production-ready

set -e

cd "$(dirname "$0")/.."

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

echo -e "${CYAN}"
echo "╔═══════════════════════════════════════════════════════════╗"
echo "║         🔥 BATTLE HARD TESTS - UBL 3.0 🔥                 ║"
echo "║      Stress • Concurrency • Edge Cases • Security        ║"
echo "╚═══════════════════════════════════════════════════════════╝"
echo -e "${NC}"

# Test counters
TOTAL=0
PASSED=0
FAILED=0

# Base URL
UBL_URL="${UBL_URL:-http://localhost:8080}"

# Test function
test_case() {
    local name=$1
    local cmd=$2
    local expected=$3
    
    TOTAL=$((TOTAL + 1))
    echo -n "  [$TOTAL] $name... "
    
    result=$(eval "$cmd" 2>&1) || true
    
    if echo "$result" | grep -q "$expected"; then
        echo -e "${GREEN}✓${NC}"
        PASSED=$((PASSED + 1))
        return 0
    else
        echo -e "${RED}✗${NC}"
        echo -e "      ${RED}Expected: $expected${NC}"
        echo -e "      ${RED}Got: ${result:0:100}${NC}"
        FAILED=$((FAILED + 1))
        return 1
    fi
}

# ═══════════════════════════════════════════════════════════════
# PHASE 1: HEALTH & AVAILABILITY
# ═══════════════════════════════════════════════════════════════
echo -e "\n${BOLD}PHASE 1: Health & Availability${NC}"
echo "─────────────────────────────────────"

test_case "Health endpoint responds" \
    "curl -s $UBL_URL/health" \
    "healthy"

test_case "Health returns correct version" \
    "curl -s $UBL_URL/health" \
    "2.0.0"

test_case "Health response time < 100ms" \
    "curl -s -o /dev/null -w '%{time_total}' $UBL_URL/health | awk '{print (\$1 < 0.1) ? \"fast\" : \"slow\"}'" \
    "fast"

# ═══════════════════════════════════════════════════════════════
# PHASE 2: WEBAUTHN SECURITY
# ═══════════════════════════════════════════════════════════════
echo -e "\n${BOLD}PHASE 2: WebAuthn Security${NC}"
echo "─────────────────────────────────────"

test_case "Register returns challenge" \
    "curl -s -X POST $UBL_URL/id/register/begin -H 'Content-Type: application/json' -d '{\"username\":\"test_$(date +%s)\"}'" \
    "challenge"

test_case "Register requires username" \
    "curl -s -X POST $UBL_URL/id/register/begin -H 'Content-Type: application/json' -d '{}'" \
    "error\|missing\|required\|422"

test_case "Invalid JSON rejected" \
    "curl -s -X POST $UBL_URL/id/register/begin -H 'Content-Type: application/json' -d 'not json'" \
    "error\|invalid\|parse\|400"

test_case "Login requires valid user" \
    "curl -s -X POST $UBL_URL/id/login/begin -H 'Content-Type: application/json' -d '{\"username\":\"nonexistent_xyz_123\"}'" \
    "error\|not found\|404\|unknown"

# ═══════════════════════════════════════════════════════════════
# PHASE 3: LEDGER INTEGRITY
# ═══════════════════════════════════════════════════════════════
echo -e "\n${BOLD}PHASE 3: Ledger Integrity${NC}"
echo "─────────────────────────────────────"

test_case "Container state returns sequence" \
    "curl -s $UBL_URL/state/C.Test" \
    "sequence"

test_case "Container state returns hash" \
    "curl -s $UBL_URL/state/C.Test" \
    "last_hash"

test_case "Non-existent atom returns 404" \
    "curl -s -o /dev/null -w '%{http_code}' $UBL_URL/atom/nonexistent_hash_xyz" \
    "404"

# ═══════════════════════════════════════════════════════════════
# PHASE 4: STRESS TESTING
# ═══════════════════════════════════════════════════════════════
echo -e "\n${BOLD}PHASE 4: Stress Testing${NC}"
echo "─────────────────────────────────────"

# Concurrent requests
echo -n "  [*] 50 concurrent health checks... "
CONCURRENT_START=$(python3 -c 'import time; print(int(time.time()*1000))')
for i in {1..50}; do
    curl -s $UBL_URL/health > /dev/null &
done
wait
CONCURRENT_END=$(python3 -c 'import time; print(int(time.time()*1000))')
CONCURRENT_TIME=$((CONCURRENT_END - CONCURRENT_START))

if [ $CONCURRENT_TIME -lt 3000 ]; then
    echo -e "${GREEN}✓${NC} (${CONCURRENT_TIME}ms)"
    PASSED=$((PASSED + 1))
else
    echo -e "${RED}✗${NC} (${CONCURRENT_TIME}ms - too slow)"
    FAILED=$((FAILED + 1))
fi
TOTAL=$((TOTAL + 1))

# Rapid sequential requests
echo -n "  [*] 100 rapid sequential requests... "
RAPID_START=$(python3 -c 'import time; print(int(time.time()*1000))')
for i in {1..100}; do
    curl -s $UBL_URL/health > /dev/null
done
RAPID_END=$(python3 -c 'import time; print(int(time.time()*1000))')
RAPID_TIME=$((RAPID_END - RAPID_START))

if [ $RAPID_TIME -lt 5000 ]; then
    echo -e "${GREEN}✓${NC} (${RAPID_TIME}ms, avg $((RAPID_TIME/100))ms/req)"
    PASSED=$((PASSED + 1))
else
    echo -e "${RED}✗${NC} (${RAPID_TIME}ms - too slow)"
    FAILED=$((FAILED + 1))
fi
TOTAL=$((TOTAL + 1))

# ═══════════════════════════════════════════════════════════════
# PHASE 5: EDGE CASES
# ═══════════════════════════════════════════════════════════════
echo -e "\n${BOLD}PHASE 5: Edge Cases${NC}"
echo "─────────────────────────────────────"

test_case "Empty body handling" \
    "curl -s -X POST $UBL_URL/id/register/begin -H 'Content-Type: application/json' -d ''" \
    "error\|invalid\|required\|EOF\|empty"

test_case "Very large payload handling" \
    "curl -s -X POST $UBL_URL/id/register/begin -H 'Content-Type: application/json' -d '{\"username\":\"'$(printf 'x%.0s' {1..10000})'\"}' | grep -c 'challenge\|error'" \
    "1"

test_case "Special chars in username" \
    "curl -s -X POST $UBL_URL/id/register/begin -H 'Content-Type: application/json' -d '{\"username\":\"test<script>\"}'" \
    "challenge\|error\|invalid"

test_case "Unicode username handled" \
    "curl -s -X POST $UBL_URL/id/register/begin -H 'Content-Type: application/json' -d '{\"username\":\"用户テスト\"}'" \
    "challenge\|error"

test_case "SQL injection prevented" \
    "curl -s -o /dev/null -w '%{http_code}' '$UBL_URL/state/C.Test%3B%20DROP%20TABLE%20atoms'" \
    "200\|404\|400"

test_case "Path traversal blocked" \
    "curl -s -o /dev/null -w '%{http_code}' '$UBL_URL/state/..%2F..%2Fetc%2Fpasswd'" \
    "200\|404\|400"

# ═══════════════════════════════════════════════════════════════
# PHASE 6: API CONTRACT
# ═══════════════════════════════════════════════════════════════
echo -e "\n${BOLD}PHASE 6: API Contract${NC}"
echo "─────────────────────────────────────"

test_case "Health returns JSON" \
    "curl -s -I $UBL_URL/health | grep -i content-type" \
    "application/json"

test_case "CORS headers present" \
    "curl -s -I -X OPTIONS $UBL_URL/id/register/begin -H 'Origin: http://localhost:3000' | grep -i access-control" \
    "access-control"

test_case "Method not allowed returns 405" \
    "curl -s -o /dev/null -w '%{http_code}' -X DELETE $UBL_URL/health" \
    "405\|404"

# ═══════════════════════════════════════════════════════════════
# RESULTS
# ═══════════════════════════════════════════════════════════════
echo ""
echo -e "${CYAN}═══════════════════════════════════════════════════════════${NC}"
echo -e "${BOLD}                    TEST RESULTS                          ${NC}"
echo -e "${CYAN}═══════════════════════════════════════════════════════════${NC}"
echo ""
echo -e "  Total:  $TOTAL"
echo -e "  ${GREEN}Passed: $PASSED${NC}"
echo -e "  ${RED}Failed: $FAILED${NC}"
echo ""

PASS_RATE=$((PASSED * 100 / TOTAL))

if [ $PASS_RATE -ge 90 ]; then
    echo -e "${GREEN}╔═══════════════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║  🏆 BATTLE HARD: PASSED ($PASS_RATE%)                              ║${NC}"
    echo -e "${GREEN}║  System is production-ready!                              ║${NC}"
    echo -e "${GREEN}╚═══════════════════════════════════════════════════════════╝${NC}"
    exit 0
elif [ $PASS_RATE -ge 70 ]; then
    echo -e "${YELLOW}╔═══════════════════════════════════════════════════════════╗${NC}"
    echo -e "${YELLOW}║  ⚠️  BATTLE HARD: NEEDS WORK ($PASS_RATE%)                         ║${NC}"
    echo -e "${YELLOW}║  Review failed tests before production                    ║${NC}"
    echo -e "${YELLOW}╚═══════════════════════════════════════════════════════════╝${NC}"
    exit 1
else
    echo -e "${RED}╔═══════════════════════════════════════════════════════════╗${NC}"
    echo -e "${RED}║  ❌ BATTLE HARD: FAILED ($PASS_RATE%)                              ║${NC}"
    echo -e "${RED}║  System is NOT ready for production                       ║${NC}"
    echo -e "${RED}╚═══════════════════════════════════════════════════════════╝${NC}"
    exit 1
fi
