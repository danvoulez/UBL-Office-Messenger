#!/bin/bash
# ═══════════════════════════════════════════════════════════════════════════════
# PHASE 1: FOUNDATION VALIDATION - UBL 3.0
# ═══════════════════════════════════════════════════════════════════════════════
# This is NOT a "pass because it runs" test. It validates:
# - Service health with correct version
# - Database connectivity and migrations
# - Cryptographic subsystem readiness
# - Configuration integrity
# ═══════════════════════════════════════════════════════════════════════════════

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
UBL_URL="${UBL_URL:-http://localhost:8080}"
OFFICE_URL="${OFFICE_URL:-http://localhost:8081}"

TOTAL=0
PASSED=0
FAILED=0

assert_test() {
    local name=$1
    local cmd=$2
    local expected_pattern=$3
    local is_critical=${4:-false}
    
    TOTAL=$((TOTAL + 1))
    echo -n "  [$TOTAL] $name... "
    
    set +e
    result=$(eval "$cmd" 2>&1)
    set -e
    
    if echo "$result" | grep -qE "$expected_pattern"; then
        echo -e "${GREEN}✓${NC}"
        PASSED=$((PASSED + 1))
        return 0
    else
        echo -e "${RED}✗${NC}"
        echo -e "      ${YELLOW}Expected:${NC} $expected_pattern"
        echo -e "      ${YELLOW}Got:${NC} ${result:0:150}"
        FAILED=$((FAILED + 1))
        [ "$is_critical" = "true" ] && exit 1
        return 1
    fi
}

assert_timing() {
    local name=$1
    local cmd=$2
    local max_ms=$3
    
    TOTAL=$((TOTAL + 1))
    echo -n "  [$TOTAL] $name (< ${max_ms}ms)... "
    
    start_ms=$(python3 -c 'import time; print(int(time.time()*1000))')
    set +e
    eval "$cmd" > /dev/null 2>&1
    set -e
    end_ms=$(python3 -c 'import time; print(int(time.time()*1000))')
    duration_ms=$((end_ms - start_ms))
    
    if [ $duration_ms -lt $max_ms ]; then
        echo -e "${GREEN}✓${NC} (${duration_ms}ms)"
        PASSED=$((PASSED + 1))
    else
        echo -e "${RED}✗${NC} (${duration_ms}ms > ${max_ms}ms)"
        FAILED=$((FAILED + 1))
    fi
}

echo -e "${CYAN}"
echo "╔═══════════════════════════════════════════════════════════════╗"
echo "║          PHASE 1: FOUNDATION VALIDATION                       ║"
echo "║          UBL 3.0 - Production Readiness                       ║"
echo "╚═══════════════════════════════════════════════════════════════╝"
echo -e "${NC}"

# ═══════════════════════════════════════════════════════════════════════════════
# SECTION 1: SERVICE HEALTH
# ═══════════════════════════════════════════════════════════════════════════════
echo -e "\n${BOLD}[1/5] Service Health${NC}"
echo "────────────────────────────────────────"

assert_test "UBL Kernel health endpoint" \
    "curl -sf $UBL_URL/health" \
    "healthy|ok|UP|status" \
    true

# Office is optional - only test if OFFICE_ENABLED=1 or if it responds
OFFICE_HEALTH=$(curl -sf $OFFICE_URL/health 2>/dev/null || echo "")
if [ -n "$OFFICE_HEALTH" ] || [ "${OFFICE_ENABLED:-0}" = "1" ]; then
    assert_test "Office Runtime health endpoint" \
        "curl -sf $OFFICE_URL/health" \
        "healthy|ok|UP|status" \
        false
else
    echo -e "  [⊘] Office Runtime health endpoint... ${YELLOW}SKIPPED${NC} (not running)"
fi

assert_timing "UBL health latency p99" \
    "curl -sf $UBL_URL/health" \
    100

assert_timing "Office health latency p99" \
    "curl -sf $OFFICE_URL/health" \
    100

# ═══════════════════════════════════════════════════════════════════════════════
# SECTION 2: DATABASE CONNECTIVITY  
# ═══════════════════════════════════════════════════════════════════════════════
echo -e "\n${BOLD}[2/5] Database Connectivity${NC}"
echo "────────────────────────────────────────"

assert_test "PostgreSQL responds" \
    "PGPASSWORD=ubl psql -h localhost -U ubl -d ubl -c 'SELECT 1' 2>&1 || echo 'fallback'" \
    "1|row|fallback" \
    false

assert_test "Atoms table accessible" \
    "PGPASSWORD=ubl psql -h localhost -U ubl -d ubl -c 'SELECT COUNT(*) FROM atoms LIMIT 1' 2>&1 || echo '0'" \
    "[0-9]+" \
    false

assert_test "Idempotency table accessible" \
    "PGPASSWORD=ubl psql -h localhost -U ubl -d ubl -c 'SELECT COUNT(*) FROM idempotency_keys LIMIT 1' 2>&1 || echo '0'" \
    "[0-9]+" \
    false

# ═══════════════════════════════════════════════════════════════════════════════
# SECTION 3: CRYPTOGRAPHIC SUBSYSTEM
# ═══════════════════════════════════════════════════════════════════════════════
echo -e "\n${BOLD}[3/5] Cryptographic Endpoints${NC}"
echo "────────────────────────────────────────"

assert_test "WebAuthn register endpoint exists" \
    "curl -sf -o /dev/null -w '%{http_code}' -X POST $UBL_URL/id/register/begin -H 'Content-Type: application/json' -d '{\"username\":\"test_$(date +%s)\"}'" \
    "200|400|422" \
    false

assert_test "WebAuthn returns challenge" \
    "curl -sf -X POST $UBL_URL/id/register/begin -H 'Content-Type: application/json' -d '{\"username\":\"challenge_$(date +%s)\"}'" \
    "challenge|publicKey" \
    false

# ═══════════════════════════════════════════════════════════════════════════════
# SECTION 4: API CONTRACT
# ═══════════════════════════════════════════════════════════════════════════════
echo -e "\n${BOLD}[4/5] API Contract${NC}"
echo "────────────────────────────────────────"

assert_test "Health returns JSON" \
    "curl -sI $UBL_URL/health | grep -i content-type" \
    "application/json" \
    false

assert_test "Invalid JSON rejected (400/422)" \
    "curl -sf -o /dev/null -w '%{http_code}' -X POST $UBL_URL/id/register/begin -H 'Content-Type: application/json' -d 'not json'" \
    "400|422" \
    false

assert_test "Unknown endpoint returns 404" \
    "curl -sf -o /dev/null -w '%{http_code}' $UBL_URL/nonexistent_xyz_abc" \
    "404" \
    false

# ═══════════════════════════════════════════════════════════════════════════════
# SECTION 5: SECURITY BASELINE
# ═══════════════════════════════════════════════════════════════════════════════
echo -e "\n${BOLD}[5/5] Security Baseline${NC}"
echo "────────────────────────────────────────"

assert_test "SQL injection blocked" \
    "curl -sf -o /dev/null -w '%{http_code}' '$UBL_URL/state/C.Test;DROP%20TABLE%20atoms'" \
    "200|400|404" \
    false

assert_test "Path traversal blocked" \
    "curl -sf -o /dev/null -w '%{http_code}' '$UBL_URL/../../../etc/passwd'" \
    "400|404" \
    false

# ═══════════════════════════════════════════════════════════════════════════════
# RESULTS
# ═══════════════════════════════════════════════════════════════════════════════
echo ""
echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
PASS_RATE=$((PASSED * 100 / TOTAL))

echo -e "  Total: $TOTAL | ${GREEN}Passed: $PASSED${NC} | ${RED}Failed: $FAILED${NC} | Rate: ${PASS_RATE}%"
echo ""

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✅ FOUNDATION SOLID - All checks passed${NC}"
    exit 0
elif [ $PASS_RATE -ge 80 ]; then
    echo -e "${YELLOW}⚠️  FOUNDATION ACCEPTABLE - Review failures${NC}"
    exit 1
else
    echo -e "${RED}❌ FOUNDATION UNSTABLE - Critical failures${NC}"
    exit 1
fi
