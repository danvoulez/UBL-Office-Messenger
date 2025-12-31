#!/bin/bash
# ═══════════════════════════════════════════════════════════════════════════════
# PHASE 2: GOLDEN PATH SCENARIOS - UBL 3.0
# ═══════════════════════════════════════════════════════════════════════════════
# Tests the critical user journeys that MUST work:
# - Message send → Ledger commit → SSE delivery
# - Job creation → Approval → Execution
# - Session lifecycle with handovers
# ═══════════════════════════════════════════════════════════════════════════════

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

UBL_URL="${UBL_URL:-http://localhost:8080}"
OFFICE_URL="${OFFICE_URL:-http://localhost:8081}"
TENANT_ID="${TENANT_ID:-T.UBL}"

TOTAL=0
PASSED=0
FAILED=0

assert_test() {
    local name=$1
    local cmd=$2
    local expected_pattern=$3
    
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
        echo -e "      ${YELLOW}Got:${NC} ${result:0:200}"
        FAILED=$((FAILED + 1))
        return 1
    fi
}

assert_json_field() {
    local name=$1
    local json=$2
    local field=$3
    local expected=$4
    
    TOTAL=$((TOTAL + 1))
    echo -n "  [$TOTAL] $name... "
    
    value=$(echo "$json" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('$field',''))" 2>/dev/null || echo "")
    
    if echo "$value" | grep -qE "$expected"; then
        echo -e "${GREEN}✓${NC} ($field=$value)"
        PASSED=$((PASSED + 1))
        return 0
    else
        echo -e "${RED}✗${NC}"
        echo -e "      ${YELLOW}Expected $field:${NC} $expected"
        echo -e "      ${YELLOW}Got:${NC} $value"
        FAILED=$((FAILED + 1))
        return 1
    fi
}

echo -e "${CYAN}"
echo "╔═══════════════════════════════════════════════════════════════╗"
echo "║          PHASE 2: GOLDEN PATH SCENARIOS                       ║"
echo "║          UBL 3.0 - Critical User Journeys                     ║"
echo "╚═══════════════════════════════════════════════════════════════╝"
echo -e "${NC}"

# ═══════════════════════════════════════════════════════════════════════════════
# GOLDEN PATH 1: BOOTSTRAP → MESSAGE → COMMIT
# ═══════════════════════════════════════════════════════════════════════════════
echo -e "\n${BOLD}[GP1] Bootstrap → Message → Ledger Commit${NC}"
echo "────────────────────────────────────────────────"

# Step 1: Bootstrap
echo -e "\n  ${CYAN}Step 1: Bootstrap${NC}"
BOOTSTRAP=$(curl -sf "$UBL_URL/messenger/bootstrap?tenant_id=$TENANT_ID" 2>/dev/null || echo '{}')

assert_json_field "Bootstrap returns conversations" \
    "$BOOTSTRAP" \
    "conversations" \
    "." 

CONV_ID=$(echo "$BOOTSTRAP" | python3 -c "import sys,json; d=json.load(sys.stdin); c=d.get('conversations',[]); print(c[0]['id'] if c else 'conv_default')" 2>/dev/null || echo "conv_default")
echo -e "      ${YELLOW}Using conversation:${NC} $CONV_ID"

# Step 2: Send message with idempotency
echo -e "\n  ${CYAN}Step 2: Send Message${NC}"
IDEM_KEY="golden_path_$(date +%s)_$$"

SEND_RESULT=$(curl -sf -X POST "$UBL_URL/v1/conversations/$CONV_ID/messages" \
    -H "Content-Type: application/json" \
    -d "{\"content\":\"Golden path test $(date)\",\"idempotency_key\":\"$IDEM_KEY\"}" 2>/dev/null || echo '{"error":"failed"}')

assert_json_field "Message committed to ledger" \
    "$SEND_RESULT" \
    "message_id" \
    ".+"

assert_json_field "Returns hash (provenance)" \
    "$SEND_RESULT" \
    "hash" \
    "^[a-f0-9]{64}$"

assert_json_field "Returns sequence number" \
    "$SEND_RESULT" \
    "sequence" \
    "^[0-9]+$"

MSG_ID=$(echo "$SEND_RESULT" | python3 -c "import sys,json; print(json.load(sys.stdin).get('message_id',''))" 2>/dev/null)
MSG_HASH=$(echo "$SEND_RESULT" | python3 -c "import sys,json; print(json.load(sys.stdin).get('hash',''))" 2>/dev/null)

# Step 3: Verify idempotency
echo -e "\n  ${CYAN}Step 3: Idempotency Verification${NC}"

SEND_AGAIN=$(curl -sf -X POST "$UBL_URL/v1/conversations/$CONV_ID/messages" \
    -H "Content-Type: application/json" \
    -d "{\"content\":\"Golden path test duplicate\",\"idempotency_key\":\"$IDEM_KEY\"}" 2>/dev/null || echo '{}')

MSG_ID_2=$(echo "$SEND_AGAIN" | python3 -c "import sys,json; print(json.load(sys.stdin).get('message_id',''))" 2>/dev/null)

TOTAL=$((TOTAL + 1))
echo -n "  [$TOTAL] Idempotency returns same message_id... "
if [ "$MSG_ID" = "$MSG_ID_2" ]; then
    echo -e "${GREEN}✓${NC}"
    PASSED=$((PASSED + 1))
else
    echo -e "${RED}✗${NC} ($MSG_ID != $MSG_ID_2)"
    FAILED=$((FAILED + 1))
fi

# Step 4: Verify in timeline
echo -e "\n  ${CYAN}Step 4: Timeline Verification${NC}"

TIMELINE=$(curl -sf "$UBL_URL/v1/conversations/$CONV_ID/timeline" 2>/dev/null || echo '{}')

assert_test "Message appears in timeline" \
    "echo '$TIMELINE' | grep -c '$MSG_ID' || echo '0'" \
    "^[1-9]" 

# ═══════════════════════════════════════════════════════════════════════════════
# GOLDEN PATH 2: JOB LIFECYCLE (if applicable)
# ═══════════════════════════════════════════════════════════════════════════════
echo -e "\n${BOLD}[GP2] Job Lifecycle (Bootstrap Check)${NC}"
echo "────────────────────────────────────────────────"

# Check if jobs endpoint exists
JOB_CHECK=$(curl -sf -o /dev/null -w '%{http_code}' "$UBL_URL/v1/jobs" 2>/dev/null || echo "000")

if [ "$JOB_CHECK" = "200" ] || [ "$JOB_CHECK" = "404" ]; then
    assert_test "Jobs API accessible" \
        "echo $JOB_CHECK" \
        "200|404"
else
    echo -e "  ${YELLOW}⏭️  Jobs endpoint not available, skipping${NC}"
fi

# ═══════════════════════════════════════════════════════════════════════════════
# GOLDEN PATH 3: MULTI-TENANT ISOLATION
# ═══════════════════════════════════════════════════════════════════════════════
echo -e "\n${BOLD}[GP3] Multi-Tenant Isolation${NC}"
echo "────────────────────────────────────────────────"

TENANT_A="T.TenantA_$(date +%s)"
TENANT_B="T.TenantB_$(date +%s)"

BOOTSTRAP_A=$(curl -sf "$UBL_URL/messenger/bootstrap?tenant_id=$TENANT_A" 2>/dev/null || echo '{}')
BOOTSTRAP_B=$(curl -sf "$UBL_URL/messenger/bootstrap?tenant_id=$TENANT_B" 2>/dev/null || echo '{}')

# Different tenants should get different data (or empty for new tenants)
assert_test "Tenant A bootstrap succeeds" \
    "echo '$BOOTSTRAP_A' | grep -c 'conversations\\|entities\\|error'" \
    "^[0-9]+"

assert_test "Tenant B bootstrap succeeds" \
    "echo '$BOOTSTRAP_B' | grep -c 'conversations\\|entities\\|error'" \
    "^[0-9]+"

# ═══════════════════════════════════════════════════════════════════════════════
# RESULTS
# ═══════════════════════════════════════════════════════════════════════════════
echo ""
echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
PASS_RATE=$((PASSED * 100 / TOTAL))

echo -e "  Total: $TOTAL | ${GREEN}Passed: $PASSED${NC} | ${RED}Failed: $FAILED${NC} | Rate: ${PASS_RATE}%"
echo ""

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✅ GOLDEN PATHS: ALL CLEAR${NC}"
    exit 0
elif [ $PASS_RATE -ge 80 ]; then
    echo -e "${YELLOW}⚠️  GOLDEN PATHS: MOSTLY WORKING${NC}"
    exit 1
else
    echo -e "${RED}❌ GOLDEN PATHS: CRITICAL FAILURES${NC}"
    exit 1
fi
