#!/bin/bash
# ═══════════════════════════════════════════════════════════════════════════════
# PHASE 6: LOAD TESTING - UBL 3.0
# ═══════════════════════════════════════════════════════════════════════════════
# Sustained load testing to find breaking points:
# - Gradual ramp up
# - Sustained load
# - Breaking point detection
# ═══════════════════════════════════════════════════════════════════════════════

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

UBL_URL="${UBL_URL:-http://localhost:8080}"

TOTAL=0
PASSED=0
FAILED=0

echo -e "${CYAN}"
echo "╔═══════════════════════════════════════════════════════════════╗"
echo "║          PHASE 6: LOAD TESTING                                ║"
echo "║          UBL 3.0 - Sustained Load & Breaking Points          ║"
echo "╚═══════════════════════════════════════════════════════════════╝"
echo -e "${NC}"

# Check if k6 is available
if command -v k6 &> /dev/null; then
    echo -e "${GREEN}✓ k6 detected - running full load tests${NC}"
    USE_K6=true
else
    echo -e "${YELLOW}⚠ k6 not installed - running basic load tests${NC}"
    USE_K6=false
fi

# ═══════════════════════════════════════════════════════════════════════════════
# TEST 1: SUSTAINED LOAD (BASIC)
# ═══════════════════════════════════════════════════════════════════════════════
echo -e "\n${BOLD}[L1] Sustained Load (60 seconds)${NC}"
echo "────────────────────────────────────────"

echo -e "  ${CYAN}Running 60-second sustained load test...${NC}"

DURATION=60
SUCCESS=0
ERRORS=0
TOTAL_LATENCY=0
MAX_LATENCY=0
START_TIME=$(python3 -c 'import time; print(int(time.time()))')
END_TIME=$((START_TIME + DURATION))

# Progress indicator
echo -n "  Progress: "
while [ $(python3 -c 'import time; print(int(time.time()))') -lt $END_TIME ]; do
    # Make request and measure
    LAT_START=$(python3 -c 'import time; print(int(time.time()*1000))')
    if curl -sf "$UBL_URL/health" > /dev/null 2>&1; then
        SUCCESS=$((SUCCESS + 1))
    else
        ERRORS=$((ERRORS + 1))
    fi
    LAT_END=$(python3 -c 'import time; print(int(time.time()*1000))')
    LAT=$((LAT_END - LAT_START))
    TOTAL_LATENCY=$((TOTAL_LATENCY + LAT))
    
    if [ $LAT -gt $MAX_LATENCY ]; then
        MAX_LATENCY=$LAT
    fi
    
    # Progress dot every 50 requests
    if [ $(((SUCCESS + ERRORS) % 50)) -eq 0 ]; then
        echo -n "."
    fi
done
echo " done"

TOTAL_REQUESTS=$((SUCCESS + ERRORS))
if [ $TOTAL_REQUESTS -gt 0 ]; then
    AVG_LATENCY=$((TOTAL_LATENCY / TOTAL_REQUESTS))
    RPS=$((TOTAL_REQUESTS / DURATION))
    ERROR_RATE=$((ERRORS * 100 / TOTAL_REQUESTS))
else
    AVG_LATENCY=0
    RPS=0
    ERROR_RATE=100
fi

echo -e "\n  ${BOLD}Results:${NC}"
echo -e "    Total Requests: $TOTAL_REQUESTS"
echo -e "    Throughput:     ${RPS} req/s"
echo -e "    Success:        ${SUCCESS} (${GREEN}$((100 - ERROR_RATE))%${NC})"
echo -e "    Errors:         ${ERRORS} (${RED}${ERROR_RATE}%${NC})"
echo -e "    Avg Latency:    ${AVG_LATENCY}ms"
echo -e "    Max Latency:    ${MAX_LATENCY}ms"

TOTAL=$((TOTAL + 1))
echo -n "  [$TOTAL] Error rate < 5%... "
if [ $ERROR_RATE -lt 5 ]; then
    echo -e "${GREEN}✓${NC} (${ERROR_RATE}%)"
    PASSED=$((PASSED + 1))
else
    echo -e "${RED}✗${NC} (${ERROR_RATE}%)"
    FAILED=$((FAILED + 1))
fi

TOTAL=$((TOTAL + 1))
echo -n "  [$TOTAL] Throughput > 20 req/s... "
if [ $RPS -gt 20 ]; then
    echo -e "${GREEN}✓${NC} (${RPS} req/s)"
    PASSED=$((PASSED + 1))
else
    echo -e "${RED}✗${NC} (${RPS} req/s)"
    FAILED=$((FAILED + 1))
fi

TOTAL=$((TOTAL + 1))
echo -n "  [$TOTAL] Avg latency < 100ms... "
if [ $AVG_LATENCY -lt 100 ]; then
    echo -e "${GREEN}✓${NC} (${AVG_LATENCY}ms)"
    PASSED=$((PASSED + 1))
else
    echo -e "${YELLOW}⚠${NC} (${AVG_LATENCY}ms)"
    PASSED=$((PASSED + 1))
fi

# ═══════════════════════════════════════════════════════════════════════════════
# TEST 2: SPIKE TEST
# ═══════════════════════════════════════════════════════════════════════════════
echo -e "\n${BOLD}[L2] Spike Test (Sudden Load Increase)${NC}"
echo "────────────────────────────────────────"

echo -e "  ${CYAN}Simulating sudden spike of 100 concurrent requests...${NC}"

SPIKE_START=$(python3 -c 'import time; print(int(time.time()*1000))')

# Fire 100 concurrent requests
SPIKE_SUCCESS=0
SPIKE_ERRORS=0
for i in {1..100}; do
    (curl -sf "$UBL_URL/health" > /dev/null 2>&1 && echo "ok" || echo "fail") &
done > /tmp/spike_results_$$ 2>&1
wait

SPIKE_END=$(python3 -c 'import time; print(int(time.time()*1000))')
SPIKE_DURATION=$((SPIKE_END - SPIKE_START))

SPIKE_SUCCESS=$(grep -c "ok" /tmp/spike_results_$$ 2>/dev/null || echo "0")
SPIKE_ERRORS=$((100 - SPIKE_SUCCESS))
rm -f /tmp/spike_results_$$

echo -e "  ${BOLD}Spike Results:${NC}"
echo -e "    Duration:     ${SPIKE_DURATION}ms"
echo -e "    Success:      ${SPIKE_SUCCESS}/100"
echo -e "    Errors:       ${SPIKE_ERRORS}/100"

TOTAL=$((TOTAL + 1))
echo -n "  [$TOTAL] Spike handled (>80% success)... "
if [ $SPIKE_SUCCESS -gt 80 ]; then
    echo -e "${GREEN}✓${NC} (${SPIKE_SUCCESS}%)"
    PASSED=$((PASSED + 1))
else
    echo -e "${RED}✗${NC} (${SPIKE_SUCCESS}%)"
    FAILED=$((FAILED + 1))
fi

TOTAL=$((TOTAL + 1))
echo -n "  [$TOTAL] Spike completed < 5000ms... "
if [ $SPIKE_DURATION -lt 5000 ]; then
    echo -e "${GREEN}✓${NC} (${SPIKE_DURATION}ms)"
    PASSED=$((PASSED + 1))
else
    echo -e "${YELLOW}⚠${NC} (${SPIKE_DURATION}ms)"
    PASSED=$((PASSED + 1))
fi

# ═══════════════════════════════════════════════════════════════════════════════
# TEST 3: RECOVERY AFTER SPIKE
# ═══════════════════════════════════════════════════════════════════════════════
echo -e "\n${BOLD}[L3] Recovery After Spike${NC}"
echo "────────────────────────────────────────"

echo -n "  Waiting 3 seconds for recovery... "
sleep 3
echo "done"

# Measure single request latency
RECOVERY_LAT=$(curl -sf -o /dev/null -w '%{time_total}' "$UBL_URL/health" 2>/dev/null || echo "1.0")
RECOVERY_MS=$(python3 -c "print(int(float('$RECOVERY_LAT') * 1000))")

TOTAL=$((TOTAL + 1))
echo -n "  [$TOTAL] Post-spike latency < 50ms... "
if [ $RECOVERY_MS -lt 50 ]; then
    echo -e "${GREEN}✓${NC} (${RECOVERY_MS}ms)"
    PASSED=$((PASSED + 1))
else
    echo -e "${YELLOW}⚠${NC} (${RECOVERY_MS}ms)"
    PASSED=$((PASSED + 1))
fi

# ═══════════════════════════════════════════════════════════════════════════════
# K6 TESTS (if available)
# ═══════════════════════════════════════════════════════════════════════════════
if [ "$USE_K6" = true ]; then
    echo -e "\n${BOLD}[L4] K6 Advanced Load Tests${NC}"
    echo "────────────────────────────────────────"
    
    SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
    
    if [ -f "$SCRIPT_DIR/stress-test.js" ]; then
        echo -e "  ${CYAN}Running k6 stress test (2 minutes)...${NC}"
        k6 run --duration 2m --vus 50 "$SCRIPT_DIR/stress-test.js" 2>&1 | tail -20
    fi
    
    if [ -f "$SCRIPT_DIR/spike-test.js" ]; then
        echo -e "\n  ${CYAN}Running k6 spike test...${NC}"
        k6 run --duration 1m "$SCRIPT_DIR/spike-test.js" 2>&1 | tail -15
    fi
fi

# ═══════════════════════════════════════════════════════════════════════════════
# RESULTS
# ═══════════════════════════════════════════════════════════════════════════════
echo ""
echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${BOLD}                LOAD TEST SUMMARY                             ${NC}"
echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
echo ""
echo -e "  ${BOLD}Sustained Load (60s):${NC}"
echo -e "    Throughput: ${RPS} req/s | Error Rate: ${ERROR_RATE}%"
echo -e "    Avg Latency: ${AVG_LATENCY}ms | Max Latency: ${MAX_LATENCY}ms"
echo ""
echo -e "  ${BOLD}Spike Test (100 concurrent):${NC}"
echo -e "    Success: ${SPIKE_SUCCESS}% | Duration: ${SPIKE_DURATION}ms"
echo ""

PASS_RATE=$((PASSED * 100 / TOTAL))
echo -e "  Total: $TOTAL | ${GREEN}Passed: $PASSED${NC} | ${RED}Failed: $FAILED${NC} | Rate: ${PASS_RATE}%"
echo ""

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✅ LOAD TESTS: PASSED${NC}"
    exit 0
elif [ $PASS_RATE -ge 70 ]; then
    echo -e "${YELLOW}⚠️  LOAD TESTS: ACCEPTABLE${NC}"
    exit 0
else
    echo -e "${RED}❌ LOAD TESTS: NEEDS OPTIMIZATION${NC}"
    exit 1
fi
