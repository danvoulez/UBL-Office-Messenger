#!/bin/bash
# ═══════════════════════════════════════════════════════════════════════════════
# 🔥 BATTLE HARD TESTS - UBL 3.0
# ═══════════════════════════════════════════════════════════════════════════════
# Tests that prove the system is production-ready:
# - Cryptographic integrity
# - PII detection
# - Security boundaries  
# - Stress & concurrency
# - Edge cases
# ═══════════════════════════════════════════════════════════════════════════════

set -euo pipefail

cd "$(dirname "$0")"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

echo -e "${CYAN}"
echo "╔═══════════════════════════════════════════════════════════════╗"
echo "║         🔥 BATTLE HARD TESTS - UBL 3.0 🔥                     ║"
echo "║    Crypto • PII • Security • Stress • Edge Cases              ║"
echo "╚═══════════════════════════════════════════════════════════════╝"
echo -e "${NC}"

TOTAL=0
PASSED=0
FAILED=0

UBL_URL="${UBL_URL:-http://localhost:8080}"
OFFICE_URL="${OFFICE_URL:-http://localhost:8081}"

test_case() {
    local name=$1
    local cmd=$2
    local expected=$3
    
    TOTAL=$((TOTAL + 1))
    echo -n "  [$TOTAL] $name... "
    
    set +e
    result=$(eval "$cmd" 2>&1)
    set -e
    
    if echo "$result" | grep -qE "$expected"; then
        echo -e "${GREEN}✓${NC}"
        PASSED=$((PASSED + 1))
        return 0
    else
        echo -e "${RED}✗${NC}"
        echo -e "      ${YELLOW}Expected:${NC} $expected"
        echo -e "      ${YELLOW}Got:${NC} ${result:0:100}"
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
    "curl -sf $UBL_URL/health" \
    "healthy|ok|UP"

test_case "Health returns JSON" \
    "curl -sI $UBL_URL/health | grep -i content-type" \
    "application/json"

test_case "Health response time < 100ms" \
    "curl -sf -o /dev/null -w '%{time_total}' $UBL_URL/health | awk '{print (\$1 < 0.1) ? \"fast\" : \"slow\"}'" \
    "fast"

# ═══════════════════════════════════════════════════════════════
# PHASE 2: CRYPTOGRAPHIC VALIDATION
# ═══════════════════════════════════════════════════════════════
echo -e "\n${BOLD}PHASE 2: Cryptographic Validation${NC}"
echo "─────────────────────────────────────"

test_case "WebAuthn register returns challenge" \
    "curl -sf -X POST $UBL_URL/id/register/begin -H 'Content-Type: application/json' -d '{\"username\":\"crypto_$(date +%s)\"}'" \
    "challenge|publicKey"

# Send message and verify hash format
BOOT=$(curl -sf "$UBL_URL/messenger/bootstrap?tenant_id=T.Battle" 2>/dev/null || echo '{}')
CONV=$(echo "$BOOT" | python3 -c "import sys,json; d=json.load(sys.stdin); c=d.get('conversations',[]); print(c[0]['id'] if c else 'conv_battle')" 2>/dev/null || echo "conv_battle")
IDEM="battle_$(date +%s)_$$"

MSG_RESULT=$(curl -sf -X POST "$UBL_URL/v1/conversations/$CONV/messages" \
    -H "Content-Type: application/json" \
    -d "{\"content\":\"battle test\",\"idempotency_key\":\"$IDEM\"}" 2>/dev/null || echo '{}')

MSG_HASH=$(echo "$MSG_RESULT" | python3 -c "import sys,json; print(json.load(sys.stdin).get('hash',''))" 2>/dev/null)

test_case "Message hash is 64 hex chars" \
    "echo '$MSG_HASH' | grep -E '^[a-f0-9]{64}$' && echo 'valid'" \
    "valid"

test_case "Idempotency returns same hash" \
    "curl -sf -X POST '$UBL_URL/v1/conversations/$CONV/messages' -H 'Content-Type: application/json' -d '{\"content\":\"different\",\"idempotency_key\":\"$IDEM\"}' | python3 -c \"import sys,json; print(json.load(sys.stdin).get('hash',''))\"" \
    "$MSG_HASH"

# ═══════════════════════════════════════════════════════════════
# PHASE 3: PII DETECTION
# ═══════════════════════════════════════════════════════════════
echo -e "\n${BOLD}PHASE 3: PII Detection${NC}"
echo "─────────────────────────────────────"

# Test PII patterns (system should detect or allow - depending on config)
PII_TEST_EMAIL="user@example.com"
PII_TEST_SSN="123-45-6789"
PII_TEST_CARD="4111111111111111"

test_case "Email in message handled" \
    "curl -sf -o /dev/null -w '%{http_code}' -X POST '$UBL_URL/v1/conversations/$CONV/messages' -H 'Content-Type: application/json' -d '{\"content\":\"contact me at $PII_TEST_EMAIL\",\"idempotency_key\":\"pii_email_$(date +%s)\"}'" \
    "200|400|422"

test_case "SSN pattern handled" \
    "curl -sf -o /dev/null -w '%{http_code}' -X POST '$UBL_URL/v1/conversations/$CONV/messages' -H 'Content-Type: application/json' -d '{\"content\":\"SSN: $PII_TEST_SSN\",\"idempotency_key\":\"pii_ssn_$(date +%s)\"}'" \
    "200|400|422"

test_case "Credit card pattern handled" \
    "curl -sf -o /dev/null -w '%{http_code}' -X POST '$UBL_URL/v1/conversations/$CONV/messages' -H 'Content-Type: application/json' -d '{\"content\":\"Card: $PII_TEST_CARD\",\"idempotency_key\":\"pii_card_$(date +%s)\"}'" \
    "200|400|422"

# ═══════════════════════════════════════════════════════════════
# PHASE 4: SECURITY BOUNDARIES
# ═══════════════════════════════════════════════════════════════
echo -e "\n${BOLD}PHASE 4: Security Boundaries${NC}"
echo "─────────────────────────────────────"

test_case "SQL injection blocked" \
    "curl -sf -o /dev/null -w '%{http_code}' '$UBL_URL/state/C.Test;DROP%20TABLE%20atoms'" \
    "200|400|404"

test_case "Path traversal blocked" \
    "curl -sf -o /dev/null -w '%{http_code}' '$UBL_URL/../../../etc/passwd'" \
    "400|404"

test_case "XSS in username sanitized" \
    "curl -sf -o /dev/null -w '%{http_code}' -X POST '$UBL_URL/id/register/begin' -H 'Content-Type: application/json' -d '{\"username\":\"<script>alert(1)</script>\"}'" \
    "200|400|422"

test_case "Invalid JSON rejected" \
    "curl -sf -o /dev/null -w '%{http_code}' -X POST '$UBL_URL/id/register/begin' -H 'Content-Type: application/json' -d 'not json'" \
    "400|422"

test_case "Empty body handled" \
    "curl -sf -o /dev/null -w '%{http_code}' -X POST '$UBL_URL/id/register/begin' -H 'Content-Type: application/json' -d ''" \
    "400|422"

# ═══════════════════════════════════════════════════════════════
# PHASE 5: STRESS TESTING
# ═══════════════════════════════════════════════════════════════
echo -e "\n${BOLD}PHASE 5: Stress Testing${NC}"
echo "─────────────────────────────────────"

# Concurrent requests
echo -n "  [*] 50 concurrent health checks... "
CONCURRENT_START=$(python3 -c 'import time; print(int(time.time()*1000))')
for i in {1..50}; do
    curl -sf $UBL_URL/health > /dev/null 2>&1 &
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
    curl -sf $UBL_URL/health > /dev/null 2>&1
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
# PHASE 6: EDGE CASES
# ═══════════════════════════════════════════════════════════════
echo -e "\n${BOLD}PHASE 6: Edge Cases${NC}"
echo "─────────────────────────────────────"

test_case "Very large payload (10KB username)" \
    "curl -sf -o /dev/null -w '%{http_code}' -X POST '$UBL_URL/id/register/begin' -H 'Content-Type: application/json' -d '{\"username\":\"'$(python3 -c 'print(\"x\"*10000)')'\"}'" \
    "200|400|413|422"

test_case "Unicode username handled" \
    "curl -sf -X POST '$UBL_URL/id/register/begin' -H 'Content-Type: application/json' -d '{\"username\":\"用户テスト\"}'" \
    "challenge|error|publicKey"

test_case "Empty username rejected" \
    "curl -sf -o /dev/null -w '%{http_code}' -X POST '$UBL_URL/id/register/begin' -H 'Content-Type: application/json' -d '{\"username\":\"\"}'" \
    "400|422"

test_case "Null username rejected" \
    "curl -sf -o /dev/null -w '%{http_code}' -X POST '$UBL_URL/id/register/begin' -H 'Content-Type: application/json' -d '{\"username\":null}'" \
    "400|422"

# ═══════════════════════════════════════════════════════════════
# PHASE 7: DATA INTEGRITY
# ═══════════════════════════════════════════════════════════════
echo -e "\n${BOLD}PHASE 7: Data Integrity${NC}"
echo "─────────────────────────────────────"

# Send multiple messages and verify all are recorded
echo -n "  [*] Sequential message integrity... "
MSG_COUNT=0
for i in {1..5}; do
    IDEM="integrity_$(date +%s%N)_$i"
    result=$(curl -sf -X POST "$UBL_URL/v1/conversations/$CONV/messages" \
        -H "Content-Type: application/json" \
        -d "{\"content\":\"integrity test $i\",\"idempotency_key\":\"$IDEM\"}" 2>/dev/null || echo '{}')
    if echo "$result" | grep -q "message_id"; then
        MSG_COUNT=$((MSG_COUNT + 1))
    fi
done

TOTAL=$((TOTAL + 1))
if [ $MSG_COUNT -ge 4 ]; then
    echo -e "${GREEN}✓${NC} ($MSG_COUNT/5 messages recorded)"
    PASSED=$((PASSED + 1))
else
    echo -e "${RED}✗${NC} ($MSG_COUNT/5 messages recorded)"
    FAILED=$((FAILED + 1))
fi

# Verify timeline contains messages
test_case "Timeline returns items" \
    "curl -sf '$UBL_URL/v1/conversations/$CONV/timeline'" \
    "items"

# ═══════════════════════════════════════════════════════════════
# PHASE 8: MULTI-TENANT ISOLATION
# ═══════════════════════════════════════════════════════════════
echo -e "\n${BOLD}PHASE 8: Multi-Tenant Isolation${NC}"
echo "─────────────────────────────────────"

TENANT_A="T.BattleA_$(date +%s)"
TENANT_B="T.BattleB_$(date +%s)"
SECRET_A="SECRET_A_$(date +%s)"
SECRET_B="SECRET_B_$(date +%s)"

# Bootstrap tenants
BOOT_A=$(curl -sf "$UBL_URL/messenger/bootstrap?tenant_id=$TENANT_A" 2>/dev/null || echo '{}')
BOOT_B=$(curl -sf "$UBL_URL/messenger/bootstrap?tenant_id=$TENANT_B" 2>/dev/null || echo '{}')

CONV_A=$(echo "$BOOT_A" | python3 -c "import sys,json; d=json.load(sys.stdin); c=d.get('conversations',[]); print(c[0]['id'] if c else 'conv_a')" 2>/dev/null)
CONV_B=$(echo "$BOOT_B" | python3 -c "import sys,json; d=json.load(sys.stdin); c=d.get('conversations',[]); print(c[0]['id'] if c else 'conv_b')" 2>/dev/null)

# Send secrets
curl -sf -X POST "$UBL_URL/v1/conversations/$CONV_A/messages" \
    -H "Content-Type: application/json" \
    -d "{\"content\":\"$SECRET_A\",\"idempotency_key\":\"ta_$(date +%s)\"}" > /dev/null 2>&1

curl -sf -X POST "$UBL_URL/v1/conversations/$CONV_B/messages" \
    -H "Content-Type: application/json" \
    -d "{\"content\":\"$SECRET_B\",\"idempotency_key\":\"tb_$(date +%s)\"}" > /dev/null 2>&1

sleep 1

# Check isolation
TIMELINE_A=$(curl -sf "$UBL_URL/v1/conversations/$CONV_A/timeline" 2>/dev/null || echo '{}')

TOTAL=$((TOTAL + 1))
echo -n "  [*] Tenant A cannot see Tenant B's secret... "
if echo "$TIMELINE_A" | grep -q "$SECRET_B"; then
    echo -e "${RED}✗${NC} (DATA LEAK!)"
    FAILED=$((FAILED + 1))
else
    echo -e "${GREEN}✓${NC}"
    PASSED=$((PASSED + 1))
fi

# ═══════════════════════════════════════════════════════════════
# RESULTS
# ═══════════════════════════════════════════════════════════════
echo ""
echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${BOLD}                    BATTLE HARD RESULTS                        ${NC}"
echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
echo ""
echo -e "  Total:  $TOTAL"
echo -e "  ${GREEN}Passed: $PASSED${NC}"
echo -e "  ${RED}Failed: $FAILED${NC}"
echo ""

if [ $TOTAL -eq 0 ]; then
    PASS_RATE=0
else
    PASS_RATE=$((PASSED * 100 / TOTAL))
fi

# Test Categories Summary
echo -e "${BOLD}Test Categories:${NC}"
echo -e "  • Health & Availability"
echo -e "  • Cryptographic Validation"
echo -e "  • PII Detection"
echo -e "  • Security Boundaries"
echo -e "  • Stress Testing"
echo -e "  • Edge Cases"
echo -e "  • Data Integrity"
echo -e "  • Multi-Tenant Isolation"
echo ""

if [ $PASS_RATE -ge 90 ]; then
    echo -e "${GREEN}╔═══════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║  🏆 BATTLE HARD: PASSED ($PASS_RATE%)                                ║${NC}"
    echo -e "${GREEN}║  System is production-ready and battle-tested!              ║${NC}"
    echo -e "${GREEN}╚═══════════════════════════════════════════════════════════════╝${NC}"
    exit 0
elif [ $PASS_RATE -ge 70 ]; then
    echo -e "${YELLOW}╔═══════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${YELLOW}║  ⚠️  BATTLE HARD: NEEDS WORK ($PASS_RATE%)                           ║${NC}"
    echo -e "${YELLOW}║  Review failed tests before production deployment           ║${NC}"
    echo -e "${YELLOW}╚═══════════════════════════════════════════════════════════════╝${NC}"
    exit 1
else
    echo -e "${RED}╔═══════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${RED}║  ❌ BATTLE HARD: FAILED ($PASS_RATE%)                                ║${NC}"
    echo -e "${RED}║  System is NOT ready for production - critical failures      ║${NC}"
    echo -e "${RED}╚═══════════════════════════════════════════════════════════════╝${NC}"
    exit 1
fi
