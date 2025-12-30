#!/bin/bash
# Run complete chaos engineering test suite

set -e

cd "$(dirname "$0")/.."

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}╔══════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║      🔥 BATTLE TESTING SUITE - CHAOS ENGINEERING 🔥      ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════════════════════╝${NC}"
echo ""

# Check prerequisites
echo "📋 Checking prerequisites..."

if ! command -v docker &> /dev/null; then
    echo -e "${RED}❌ Docker not installed${NC}"
    exit 1
fi

if ! command -v docker-compose &> /dev/null; then
    echo -e "${RED}❌ Docker Compose not installed${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Prerequisites OK${NC}"
echo ""

# Warning
echo -e "${RED}⚠️  WARNING ⚠️${NC}"
echo "This suite will:"
echo "  - Inject network failures"
echo "  - Crash services randomly"
echo "  - Exhaust system resources"
echo "  - Create split-brain scenarios"
echo ""
echo "DO NOT RUN ON PRODUCTION!"
echo ""
read -p "Continue? (yes/no): " -r
if [[ ! $REPLY =~ ^[Yy]es$ ]]; then
    echo "Aborted"
    exit 0
fi
echo ""

# Setup environment
echo "🚀 Setting up chaos environment..."
docker-compose -f docker-compose.chaos.yml down -v 2>/dev/null || true
docker-compose -f docker-compose.chaos.yml up -d

# Wait for services
echo "⏳ Waiting for services to be ready..."
sleep 30

for service in ubl-kernel office; do
    for i in {1..30}; do
        if curl -sf http://localhost:$([ "$service" = "ubl-kernel" ] && echo 8080 || echo 8081)/health > /dev/null 2>&1; then
            echo -e "${GREEN}✅ $service ready${NC}"
            break
        fi
        if [ $i -eq 30 ]; then
            echo -e "${RED}❌ $service failed to start${NC}"
            docker-compose -f docker-compose.chaos.yml logs $service
            exit 1
        fi
        sleep 2
    done
done

echo ""

# Initialize results
mkdir -p ../reports
REPORT_FILE="../reports/chaos-test-results.json"
echo "{\"experiments\": [], \"start_time\": \"$(date -Iseconds)\"}" > $REPORT_FILE

# Track results
TOTAL_EXPERIMENTS=0
PASSED_EXPERIMENTS=0
FAILED_EXPERIMENTS=0

# Function to run experiment
run_experiment() {
    local name=$1
    local script=$2
    
    TOTAL_EXPERIMENTS=$((TOTAL_EXPERIMENTS + 1))
    
    echo -e "${BLUE}════════════════════════════════════════════════════════${NC}"
    echo -e "${BLUE}Experiment $TOTAL_EXPERIMENTS: $name${NC}"
    echo -e "${BLUE}════════════════════════════════════════════════════════${NC}"
    echo ""
    
    local start_time=$(date +%s)
    
    if eval "$script"; then
        local end_time=$(date +%s)
        local duration=$((end_time - start_time))
        
        echo ""
        echo -e "${GREEN}✅ $name PASSED${NC} (${duration}s)"
        PASSED_EXPERIMENTS=$((PASSED_EXPERIMENTS + 1))
        
        # Log to report
        echo "  Experiment passed: $name"
        return 0
    else
        local end_time=$(date +%s)
        local duration=$((end_time - start_time))
        
        echo ""
        echo -e "${RED}❌ $name FAILED${NC} (${duration}s)"
        FAILED_EXPERIMENTS=$((FAILED_EXPERIMENTS + 1))
        
        # Log to report
        echo "  Experiment failed:  $name"
        return 1
    fi
    
    echo ""
}

# Run Rust resilience tests
echo -e "${YELLOW}═══════════════════════════════════════════════════${NC}"
echo -e "${YELLOW}Phase 1: Resilience Tests${NC}"
echo -e "${YELLOW}═══════════════════════════════════════════════════${NC}"
echo ""

run_experiment "Auto-Retry Resilience" "cd rust && cargo test test_resilience_auto_retry -- --nocapture"
run_experiment "Circuit Breaker" "cd rust && cargo test test_resilience_circuit_breaker -- --nocapture"
run_experiment "Graceful Degradation" "cd rust && cargo test test_resilience_graceful_degradation -- --nocapture"
run_experiment "State Recovery" "cd rust && cargo test test_resilience_state_recovery -- --nocapture"
run_experiment "Data Integrity Under Stress" "cd rust && cargo test test_resilience_data_integrity_under_stress -- --nocapture"

# Run failure recovery tests
echo ""
echo -e "${YELLOW}═══════════════════════════════════════════════════${NC}"
echo -e "${YELLOW}Phase 2: Failure Recovery Tests${NC}"
echo -e "${YELLOW}═══════════════════════════════════════════════════${NC}"
echo ""

run_experiment "Database Disconnect Recovery" "cd rust && cargo test test_recovery_from_database_disconnect -- --nocapture"
run_experiment "Cascading Failure Recovery" "cd rust && cargo test test_recovery_from_cascading_failures -- --nocapture"
run_experiment "Split Brain Recovery" "cd rust && cargo test test_recovery_split_brain_scenario -- --nocapture"
run_experiment "Projection Inconsistency Recovery" "cd rust && cargo test test_recovery_projection_inconsistency -- --nocapture"

# Run load tests
echo ""
echo -e "${YELLOW}═══════════════════════════════════════════════════${NC}"
echo -e "${YELLOW}Phase 3:  Stress & Load Tests${NC}"
echo -e "${YELLOW}═══════════════════════════════════════════════════${NC}"
echo ""

if command -v k6 &> /dev/null; then
    run_experiment "Spike Load Test" "k6 run k6/spike-test.js"
    run_experiment "Stress Test (15min)" "k6 run k6/stress-test.js"
    
    if [ "$1" = "--full" ]; then
        echo -e "${YELLOW}Running full soak test (4+ hours)...${NC}"
        run_experiment "Soak Test (4h)" "k6 run k6/soak-test. js"
    else
        echo -e "${YELLOW}⏭️  Skipping soak test (use --full to run)${NC}"
    fi
else
    echo -e "${YELLOW}⚠️  K6 not installed, skipping load tests${NC}"
fi

# Optional:  Chaos Monkey
if [ "$1" = "--chaos-monkey" ]; then
    echo ""
    echo -e "${YELLOW}═══════════════════════════════════════════════════${NC}"
    echo -e "${YELLOW}Phase 4: Chaos Monkey (5 minutes)${NC}"
    echo -e "${YELLOW}═══════════════════════════════════════════════════${NC}"
    echo ""
    
    run_experiment "Chaos Monkey Random Failures" "cd rust && cargo test test_chaos_monkey_random_failures -- --ignored --nocapture"
fi

# Calculate resilience score
echo ""
echo -e "${BLUE}╔══════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║                   CALCULATING RESILIENCE SCORE            ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════════════════════╝${NC}"
echo ""

./scripts/calculate-resilience-score.sh

# Final report
echo ""
echo -e "${BLUE}╔══════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║                      BATTLE TEST RESULTS                  ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════════════════════╝${NC}"
echo ""
echo "Total Experiments:     $TOTAL_EXPERIMENTS"
echo -e "${GREEN}Passed:               $PASSED_EXPERIMENTS${NC}"
echo -e "${RED}Failed:               $FAILED_EXPERIMENTS${NC}"

PASS_RATE=$(echo "scale=1; $PASSED_EXPERIMENTS * 100 / $TOTAL_EXPERIMENTS" | bc)
echo ""
echo -e "Pass Rate:             ${GREEN}${PASS_RATE}%${NC}"
echo ""

# Update report
echo "{\"experiments\": $TOTAL_EXPERIMENTS, \"passed\": $PASSED_EXPERIMENTS, \"failed\": $FAILED_EXPERIMENTS, \"pass_rate\": $PASS_RATE, \"end_time\": \"$(date -Iseconds)\"}" > $REPORT_FILE

echo "📄 Full report:  $REPORT_FILE"
echo ""

# Cleanup
echo "🧹 Cleaning up..."
docker-compose -f docker-compose.chaos.yml down

if [ $FAILED_EXPERIMENTS -eq 0 ]; then
    echo -e "${GREEN}╔══════════════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║          🎉 ALL BATTLE TESTS PASSED! 🎉                  ║${NC}"
    echo -e "${GREEN}║     System demonstrated excellent resilience!              ║${NC}"
    echo -e "${GREEN}╚══════════════════════════════════════════════════════════╝${NC}"
    exit 0
else
    echo -e "${RED}╔══════════════════════════════════════════════════════════╗${NC}"
    echo -e "${RED}║          ⚠️  SOME BATTLE TESTS FAILED ⚠️                  ║${NC}"
    echo -e "${RED}║     Review failures and improve resilience                ║${NC}"
    echo -e "${RED}╚══════════════════════════════════════════════════════════╝${NC}"
    exit 1
fi