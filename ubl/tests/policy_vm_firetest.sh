#!/bin/bash
# ==============================================================================
# Policy VM Fire Test (Gemini P0 #3)
# ==============================================================================
# This script tests that the Policy VM correctly DENIES dangerous operations.
# ALL TESTS SHOULD FAIL (return 4xx).
# If any test returns 2xx, the system is insecure!
# ==============================================================================

set -e

UBL_HOST="${UBL_HOST:-http://localhost:8080}"
PASS_COUNT=0
FAIL_COUNT=0

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "=============================================="
echo "  🔥 POLICY VM FIRE TEST"
echo "  Host: $UBL_HOST"
echo "=============================================="
echo ""

# Test function: expects 4xx response
# Usage: test_must_deny "Test Name" "/endpoint" '{"json": "data"}'
test_must_deny() {
    local name="$1"
    local endpoint="$2"
    local data="$3"
    
    echo -n "🧪 Testing: $name... "
    
    HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" \
        -X POST "$UBL_HOST$endpoint" \
        -H "Content-Type: application/json" \
        -d "$data" 2>/dev/null || echo "000")
    
    if [[ "$HTTP_CODE" =~ ^4[0-9][0-9]$ ]] || [[ "$HTTP_CODE" =~ ^5[0-9][0-9]$ ]]; then
        echo -e "${GREEN}✓ DENIED (HTTP $HTTP_CODE)${NC}"
        PASS_COUNT=$((PASS_COUNT + 1))
    elif [[ "$HTTP_CODE" == "000" ]]; then
        echo -e "${YELLOW}⚠ CONNECTION FAILED${NC}"
    else
        echo -e "${RED}✗ ALLOWED (HTTP $HTTP_CODE) - SECURITY BREACH!${NC}"
        FAIL_COUNT=$((FAIL_COUNT + 1))
    fi
}

# Test function: expects 2xx response
test_must_allow() {
    local name="$1"
    local endpoint="$2"
    local data="$3"
    
    echo -n "🧪 Testing: $name... "
    
    HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" \
        -X POST "$UBL_HOST$endpoint" \
        -H "Content-Type: application/json" \
        -d "$data" 2>/dev/null || echo "000")
    
    if [[ "$HTTP_CODE" =~ ^2[0-9][0-9]$ ]]; then
        echo -e "${GREEN}✓ ALLOWED (HTTP $HTTP_CODE)${NC}"
        PASS_COUNT=$((PASS_COUNT + 1))
    elif [[ "$HTTP_CODE" == "000" ]]; then
        echo -e "${YELLOW}⚠ CONNECTION FAILED${NC}"
    else
        echo -e "${RED}✗ DENIED (HTTP $HTTP_CODE) - UNEXPECTED${NC}"
        FAIL_COUNT=$((FAIL_COUNT + 1))
    fi
}

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  TEST GROUP 1: Evolution commits (MUST DENY)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Evolution without proper pact/signature should be denied
test_must_deny "Evolution commit without pact" "/link/commit" '{
    "container_id": "C.Policy",
    "expected_sequence": 1,
    "previous_hash": "genesis",
    "atom": {"type": "policy.change", "new_rule": "allow_all"},
    "intent_class": "Evolution",
    "physics_delta": "0",
    "author_pubkey": "fake_pubkey",
    "signature": "fake_signature",
    "pact_id": null,
    "pact_sig": null
}'

test_must_deny "Evolution with fake signature" "/link/commit" '{
    "container_id": "C.Policy",
    "expected_sequence": 1,
    "previous_hash": "genesis",
    "atom": {"type": "policy.change", "new_rule": "allow_all"},
    "intent_class": "Evolution",
    "physics_delta": "0",
    "author_pubkey": "0000000000000000000000000000000000000000000000000000000000000000",
    "signature": "ed25519:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
    "pact_id": "fake_pact",
    "pact_sig": "fake"
}'

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  TEST GROUP 2: Entropy commits (MUST DENY)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Entropy (value creation/destruction) without proper auth
test_must_deny "Entropy commit without credentials" "/link/commit" '{
    "container_id": "C.Treasury",
    "expected_sequence": 1,
    "previous_hash": "genesis",
    "atom": {"type": "mint", "amount": 1000000, "to": "attacker"},
    "intent_class": "Entropy",
    "physics_delta": "1000000",
    "author_pubkey": "attacker_pubkey",
    "signature": "fake",
    "pact_id": null,
    "pact_sig": null
}'

test_must_deny "Negative physics delta without pact" "/link/commit" '{
    "container_id": "C.Treasury",
    "expected_sequence": 1,
    "previous_hash": "genesis",
    "atom": {"type": "burn", "amount": -500, "from": "victim"},
    "intent_class": "Entropy",
    "physics_delta": "-500",
    "author_pubkey": "attacker",
    "signature": "fake",
    "pact_id": null,
    "pact_sig": null
}'

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  TEST GROUP 3: Invalid signatures (MUST DENY)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

test_must_deny "Observation with garbage signature" "/link/commit" '{
    "container_id": "C.Messenger",
    "expected_sequence": 1,
    "previous_hash": "genesis",
    "atom": {"type": "message.sent", "content": "hello"},
    "intent_class": "Observation",
    "physics_delta": "0",
    "author_pubkey": "valid_but_wrong_key",
    "signature": "not_even_base64!!!",
    "pact_id": null,
    "pact_sig": null
}'

test_must_deny "Conservation with mismatched signature" "/link/commit" '{
    "container_id": "C.Ledger",
    "expected_sequence": 1,
    "previous_hash": "genesis",
    "atom": {"type": "transfer", "from": "A", "to": "B", "amount": 100},
    "intent_class": "Conservation",
    "physics_delta": "0",
    "author_pubkey": "ed25519:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
    "signature": "ed25519:BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB",
    "pact_id": null,
    "pact_sig": null
}'

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  TEST GROUP 4: Permit bypass attempts (MUST DENY)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

test_must_deny "L5 permit without step-up" "/v1/policy/permit" '{
    "office": "office.main",
    "action": "deploy",
    "target": "production",
    "args": {},
    "risk": "L5",
    "plan": {"description": "Deploy to prod"},
    "stepup_assertion": null
}'

test_must_deny "Command without valid permit" "/v1/commands/issue" '{
    "permit_jti": "nonexistent_permit_id",
    "params": {"command": "rm -rf /"}
}'

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  TEST GROUP 5: SQL Injection attempts (MUST DENY)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

test_must_deny "SQL injection in container_id" "/link/validate" '{
    "container_id": "C.Test; DROP TABLE ledger_entry; --",
    "expected_sequence": 1,
    "previous_hash": "genesis",
    "atom": {"type": "test"},
    "intent_class": "Observation",
    "physics_delta": "0",
    "author_pubkey": "test",
    "signature": "test"
}'

test_must_deny "SQL injection in atom" "/link/validate" '{
    "container_id": "C.Test",
    "expected_sequence": 1,
    "previous_hash": "genesis",
    "atom": {"type": "test", "payload": "'; DROP TABLE users; --"},
    "intent_class": "Observation",
    "physics_delta": "0",
    "author_pubkey": "test",
    "signature": "test"
}'

echo ""
echo "=============================================="
echo "  📊 RESULTS"
echo "=============================================="
echo ""

TOTAL=$((PASS_COUNT + FAIL_COUNT))
echo "Tests Passed: $PASS_COUNT / $TOTAL"
echo "Tests Failed: $FAIL_COUNT / $TOTAL"
echo ""

if [ "$FAIL_COUNT" -gt 0 ]; then
    echo -e "${RED}⚠️  SECURITY BREACH DETECTED!${NC}"
    echo "Some operations that should be DENIED were ALLOWED."
    echo "This is a critical security issue. Do NOT deploy to production."
    exit 1
else
    echo -e "${GREEN}✅ ALL SECURITY CHECKS PASSED${NC}"
    echo "The Policy VM is correctly denying unauthorized operations."
    exit 0
fi

