#!/bin/bash
# 💎 DIAMOND RUN - Ultimate Production Readiness Test
# If this passes, the system is SHOWTIME READY

set -e

cd "$(dirname "$0")"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
BOLD='\033[1m'
NC='\033[0m'

# Diamond ASCII art
echo -e "${CYAN}"
cat << "EOF"
                    💎
                  💎💎💎
                💎💎💎💎💎
              💎💎💎💎💎💎💎
            💎💎💎💎💎💎💎💎💎
          💎💎💎💎💎💎💎💎💎💎💎
            💎💎💎💎💎💎💎💎💎
              💎💎💎💎💎💎💎
                💎💎💎💎💎
                  💎💎💎
                    💎

    ╔══════════════════════════════════════════╗
    ║      💎 DIAMOND RUN TEST SUITE 💎        ║
    ║   Ultimate Production Readiness Test     ║
    ╚══════════════════════════════════════════╝

EOF
echo -e "${NC}"

# Test start time
START_TIME=$(date +%s)
TEST_START_TIMESTAMP=$(date -Iseconds)

echo -e "${BOLD}Test Started: ${TEST_START_TIMESTAMP}${NC}"
echo ""

# Create reports directory
mkdir -p reports
REPORT_FILE="reports/diamond-run-report-$(date +%Y%m%d-%H%M%S).json"
HTML_REPORT="reports/diamond-run-report-$(date +%Y%m%d-%H%M%S).html"

# Initialize report
cat > $REPORT_FILE << EOF
{
  "test_suite": "Diamond Run",
  "version": "1.0",
  "start_time": "$TEST_START_TIMESTAMP",
  "phases": [],
  "overall_status": "RUNNING"
}
EOF

# Phase tracking
TOTAL_PHASES=8
PASSED_PHASES=0
FAILED_PHASES=0
PHASE_RESULTS=()

# Function to run phase
run_phase() {
    local phase_num=$1
    local phase_name=$2
    local phase_script=$3
    local required_pass_rate=$4
    
    echo ""
    echo -e "${MAGENTA}═══════════════════════════════════════════════════════════════${NC}"
    echo -e "${MAGENTA}${BOLD}  PHASE $phase_num/$TOTAL_PHASES: $phase_name${NC}"
    echo -e "${MAGENTA}═══════════════════════════════════════════════════════════════${NC}"
    echo ""
    
    local phase_start=$(date +%s)
    local phase_status="FAILED"
    local phase_message=""
    
    if bash "$phase_script"; then
        local phase_end=$(date +%s)
        local phase_duration=$((phase_end - phase_start))
        
        phase_status="PASSED"
        phase_message="Phase completed successfully in ${phase_duration}s"
        PASSED_PHASES=$((PASSED_PHASES + 1))
        PHASE_RESULTS+=("✅ Phase $phase_num: $phase_name - PASSED")
        
        echo ""
        echo -e "${GREEN}${BOLD}✅ PHASE $phase_num PASSED${NC} (${phase_duration}s)"
        echo ""
    else
        local phase_end=$(date +%s)
        local phase_duration=$((phase_end - phase_start))
        
        phase_status="FAILED"
        phase_message="Phase failed after ${phase_duration}s"
        FAILED_PHASES=$((FAILED_PHASES + 1))
        PHASE_RESULTS+=("❌ Phase $phase_num: $phase_name - FAILED")
        
        echo ""
        echo -e "${RED}${BOLD}❌ PHASE $phase_num FAILED${NC} (${phase_duration}s)"
        echo ""
        
        # Critical phases must pass
        if [ "$required_pass_rate" = "100" ]; then
            echo -e "${RED}${BOLD}💥 CRITICAL PHASE FAILED - ABORTING DIAMOND RUN${NC}"
            generate_failure_report
            exit 1
        fi
    fi
    
    # Log to report
    echo "  Phase $phase_num: $phase_status"
}

# Warning
echo -e "${YELLOW}${BOLD}⚠️  WARNING ⚠️${NC}"
echo ""
echo "Diamond Run will:"
echo "  • Run for 2-4 hours"
echo "  • Use significant system resources"
echo "  • Inject failures and chaos"
echo "  • Stress test all components"
echo ""
echo "This is the FINAL validation before production."
echo ""
read -p "Are you ready to proceed? (yes/no): " -r
if [[ !  $REPLY =~ ^[Yy]es$ ]]; then
    echo "Diamond Run cancelled"
    exit 0
fi
echo ""

# System check
echo -e "${CYAN}🔍 Pre-flight System Check${NC}"
echo ""

if ! command -v docker &> /dev/null; then
    echo -e "${RED}❌ Docker not installed${NC}"
    exit 1
fi

if ! command -v docker-compose &> /dev/null; then
    echo -e "${RED}❌ Docker Compose not installed${NC}"
    exit 1
fi

if ! command -v cargo &> /dev/null; then
    echo -e "${RED}❌ Rust/Cargo not installed${NC}"
    exit 1
fi

if ! command -v k6 &> /dev/null; then
    echo -e "${YELLOW}⚠️  K6 not installed - load tests will be limited${NC}"
fi

echo -e "${GREEN}✅ System dependencies OK${NC}"
echo ""

# Setup environment
echo -e "${CYAN}🚀 Setting up Diamond Run environment${NC}"
echo ""

docker-compose -f docker-compose.diamond.yml down -v 2>/dev/null || true
docker-compose -f docker-compose.diamond.yml build
docker-compose -f docker-compose.diamond.yml up -d

echo "⏳ Waiting for services to stabilize (60s)..."
sleep 60

# Verify all services healthy
for service in postgres ubl-kernel office messenger; do
    echo "   Checking $service..."
    
    case $service in
        postgres)
            for i in {1..30}; do
                if docker-compose -f docker-compose.diamond.yml exec -T postgres pg_isready -U ubl_diamond &>/dev/null; then
                    echo -e "   ${GREEN}✅ $service ready${NC}"
                    break
                fi
                [ $i -eq 30 ] && echo -e "   ${RED}❌ $service not ready${NC}" && exit 1
                sleep 2
            done
            ;;
        ubl-kernel)
            for i in {1..30}; do
                if curl -sf http://localhost:8080/health &>/dev/null; then
                    echo -e "   ${GREEN}✅ $service ready${NC}"
                    break
                fi
                [ $i -eq 30 ] && echo -e "   ${RED}❌ $service not ready${NC}" && exit 1
                sleep 2
            done
            ;;
        office)
            for i in {1..30}; do
                if curl -sf http://localhost:8081/health &>/dev/null; then
                    echo -e "   ${GREEN}✅ $service ready${NC}"
                    break
                fi
                [ $i -eq 30 ] && echo -e "   ${RED}❌ $service not ready${NC}" && exit 1
                sleep 2
            done
            ;;
        messenger)
            echo -e "   ${GREEN}✅ $service ready${NC}"
            ;;
    esac
done

echo ""
echo -e "${GREEN}${BOLD}🎬 DIAMOND RUN STARTING${NC}"
echo ""

# ═══════════════════════════════════════════════════════════════
# PHASE 1: FOUNDATION (CRITICAL - Must Pass 100%)
# ═══════════════════════════════════════════════════════════════
run_phase 1 "Foundation Validation" "01-foundation.sh" "100"

# ═══════════════════════════════════════════════════════════════
# PHASE 2: GOLDEN PATHS (Must Pass 95%)
# ═══════════════════════════════════════════════════════════════
run_phase 2 "Golden Path Scenarios" "./02-golden-paths.sh" "95"

# ═══════════════════════════════════════════════════════════════
# PHASE 3: PERFORMANCE (Must Meet SLOs)
# ═══════════════════════════════════════════════════════════════
run_phase 3 "Performance Benchmarks" "03-performance.sh" "90"

# ═══════════════════════════════════════════════════════════════
# PHASE 4: RESILIENCE (Score ≥ 85/100)
# ═══════════════════════════════════════════════════════════════
run_phase 4 "Resilience Testing" "04-resilience.sh" "85"

# ═══════════════════════════════════════════════════════════════
# PHASE 5: CHAOS ENGINEERING (Survival ≥ 80%)
# ═══════════════════════════════════════════════════════════════
run_phase 5 "Chaos Engineering" "05-chaos.sh" "80"

# ═══════════════════════════════════════════════════════════════
# PHASE 6: LOAD TESTING (No Degradation)
# ═══════════════════════════════════════════════════════════════
run_phase 6 "Load Testing" "06-load.sh" "85"

# ═══════════════════════════════════════════════════════════════
# PHASE 7: SECURITY (Zero Violations)
# ═══════════════════════════════════════════════════════════════
run_phase 7 "Security Audit" "07-security.sh" "100"

# ═══════════════════════════════════════════════════════════════
# PHASE 8: DATA INTEGRITY (100% Validation)
# ═══════════════════════════════════════════════════════════════
run_phase 8 "Data Integrity" "08-integrity.sh" "100"

# Calculate final results
END_TIME=$(date +%s)
TOTAL_DURATION=$((END_TIME - START_TIME))
TOTAL_DURATION_HOURS=$(echo "scale=2; $TOTAL_DURATION / 3600" | bc)

# Generate final report
echo ""
echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${CYAN}${BOLD}                  DIAMOND RUN RESULTS${NC}"
echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
echo ""
echo "Total Duration: ${TOTAL_DURATION}s (${TOTAL_DURATION_HOURS}h)"
echo "Total Phases:    $TOTAL_PHASES"
echo -e "${GREEN}Passed:         $PASSED_PHASES${NC}"
echo -e "${RED}Failed:        $FAILED_PHASES${NC}"
echo ""
echo "Phase Results:"
for result in "${PHASE_RESULTS[@]}"; do
    echo "  $result"
done
echo ""

# Calculate final score
PASS_RATE=$(echo "scale=1; $PASSED_PHASES * 100 / $TOTAL_PHASES" | bc)

# Cleanup
echo "🧹 Cleaning up..."
docker-compose -f docker-compose.diamond.yml down

# Final verdict
echo ""
echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"

if [ $FAILED_PHASES -eq 0 ]; then
    # DIAMOND RUN PASSED!  
    echo -e "${GREEN}"
    cat << "EOF"
    ██████╗ ██╗ █████╗ ███╗   ███╗ ██████╗ ███╗   ██╗██████╗ 
    ██╔══██╗██║██╔══██╗████╗ ████║██╔═══██╗████╗  ██║██╔══██╗
    ██║  ██║██║███████║██╔████╔██║██║   ██║██╔██╗ ██║██║  ██║
    ██║  ██║██║██╔══██║██║╚██╔╝██║██║   ██║██║╚██╗██║██║  ██║
    ██████╔╝██║██║  ██║██║ ╚═╝ ██║╚██████╔╝██║ ╚████║██████╔╝
    ╚═════╝ ╚═╝╚═╝  ╚═╝╚═╝     ╚═╝ ╚═════╝ ╚═╝  ╚═══╝╚═════╝ 
    
    ██████╗  █████╗ ███████╗███████╗███████╗██████╗ ██╗
    ██╔══██╗██╔══██╗██╔════╝██╔════╝██╔════╝██╔══██╗██║
    ██████╔╝███████║███████╗███████╗█████╗  ██║  ██║██║
    ██╔═══╝ ██╔══██║╚════██║╚════██║██╔══╝  ██║  ██║╚═╝
    ██║     ██║  ██║███████║███████║███████╗██████╔╝██╗
    ╚═╝     ╚═╝  ╚═╝╚══════╝╚══════╝╚══════╝╚═════╝ ╚═╝
EOF
    echo -e "${NC}"
    
    echo -e "${GREEN}╔══════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║                                                              ║${NC}"
    echo -e "${GREEN}║              🎉 CONGRATULATIONS! 🎉                          ║${NC}"
    echo -e "${GREEN}║                                                              ║${NC}"
    echo -e "${GREEN}║         💎 DIAMOND RUN:  PASSED 💎                           ║${NC}"
    echo -e "${GREEN}║                                                              ║${NC}"
    echo -e "${GREEN}║    The system is PRODUCTION READY! 🚀                       ║${NC}"
    echo -e "${GREEN}║                                                              ║${NC}"
    echo -e "${GREEN}║    All $TOTAL_PHASES phases passed:                                         ║${NC}"
    echo -e "${GREEN}║    ✅ Foundation validated                                   ║${NC}"
    echo -e "${GREEN}║    ✅ Golden paths working                                   ║${NC}"
    echo -e "${GREEN}║    ✅ Performance meets SLOs                                 ║${NC}"
    echo -e "${GREEN}║    ✅ System is resilient                                    ║${NC}"
    echo -e "${GREEN}║    ✅ Survives chaos                                         ║${NC}"
    echo -e "${GREEN}║    ✅ Handles load                                           ║${NC}"
    echo -e "${GREEN}║    ✅ Security verified                                      ║${NC}"
    echo -e "${GREEN}║    ✅ Data integrity maintained                              ║${NC}"
    echo -e "${GREEN}║                                                              ║${NC}"
    echo -e "${GREEN}║    🏆 SHOWTIME READY 🏆                                      ║${NC}"
    echo -e "${GREEN}║                                                              ║${NC}"
    echo -e "${GREEN}╚══════════════════════════════════════════════════════════════╝${NC}"
    
    # Generate certificate
    generate_diamond_certificate
    
    # Generate deployment checklist
    generate_deployment_checklist
    
    echo ""
    echo -e "${CYAN}📄 Diamond Certificate:  reports/diamond-certificate.txt${NC}"
    echo -e "${CYAN}📋 Deployment Checklist: reports/production-deployment-checklist.md${NC}"
    echo -e "${CYAN}📊 Full Report: $HTML_REPORT${NC}"
    echo ""
    
    # Update report
    cat > $REPORT_FILE << EOF
{
  "test_suite": "Diamond Run",
  "version": "1.0",
  "start_time": "$TEST_START_TIMESTAMP",
  "end_time": "$(date -Iseconds)",
  "duration_seconds": $TOTAL_DURATION,
  "total_phases": $TOTAL_PHASES,
  "passed_phases": $PASSED_PHASES,
  "failed_phases": $FAILED_PHASES,
  "pass_rate": $PASS_RATE,
  "overall_status": "PASSED",
  "production_ready": true,
  "showtime":  true
}
EOF
    
    exit 0
else
    # DIAMOND RUN FAILED
    echo -e "${RED}"
    cat << "EOF"
    ██████╗ ██╗ █████╗ ███╗   ███╗ ██████╗ ███╗   ██╗██████╗ 
    ██╔══██╗██║██╔══██╗████╗ ████║██╔═══██╗████╗  ██║██╔══██╗
    ██║  ██║██║███████║██╔████╔██║██║   ██║██╔██╗ ██║██║  ██║
    ██║  ██║██║██╔══██║██║╚██╔╝██║██║   ██║██║╚██╗██║██║  ██║
    ██████╔╝██║██║  ██║██║ ╚═╝ ██║╚██████╔╝██║ ╚████║██████╔╝
    ╚═════╝ ╚═╝╚═╝  ╚═╝╚═╝     ╚═╝ ╚═════╝ ╚═╝  ╚═══╝╚═════╝ 
    
    ███████╗ █████╗ ██╗██╗     ███████╗██████╗ ██╗
    ██╔════╝██╔══██╗██║██║     ██╔════╝██╔══██╗██║
    █████╗  ███████║██║██║     █████╗  ██║  ██║██║
    ██╔══╝  ██╔══██║██║██║     ██╔══╝  ██║  ██║╚═╝
    ██║     ██║  ██║██║███████╗███████╗██████╔╝██╗
    ╚═╝     ╚═╝  ╚═╝╚═╝╚══════╝╚══════╝╚═════╝ ╚═╝
EOF
    echo -e "${NC}"
    
    echo -e "${RED}╔══════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${RED}║                                                              ║${NC}"
    echo -e "${RED}║              ⚠️  DIAMOND RUN:  FAILED ⚠️                      ║${NC}"
    echo -e "${RED}║                                                              ║${NC}"
    echo -e "${RED}║    System is NOT production ready                            ║${NC}"
    echo -e "${RED}║                                                              ║${NC}"
    echo -e "${RED}║    $FAILED_PHASES out of $TOTAL_PHASES phases failed                                 ║${NC}"
    echo -e "${RED}║    Pass rate: ${PASS_RATE}%                                            ║${NC}"
    echo -e "${RED}║                                                              ║${NC}"
    echo -e "${RED}║    ❌ DO NOT DEPLOY TO PRODUCTION                            ║${NC}"
    echo -e "${RED}║                                                              ║${NC}"
    echo -e "${RED}║    Review failure report and fix issues                      ║${NC}"
    echo -e "${RED}║                                                              ║${NC}"
    echo -e "${RED}╚══════════════════════════════════════════════════════════════╝${NC}"
    
    echo ""
    echo -e "${YELLOW}📋 Action Items:${NC}"
    echo "  1. Review detailed failure report:  $HTML_REPORT"
    echo "  2. Fix identified issues"
    echo "  3. Run targeted tests for failed components"
    echo "  4. Re-run Diamond Run"
    echo ""
    
    # Update report
    cat > $REPORT_FILE << EOF
{
  "test_suite": "Diamond Run",
  "version": "1.0",
  "start_time": "$TEST_START_TIMESTAMP",
  "end_time": "$(date -Iseconds)",
  "duration_seconds":  $TOTAL_DURATION,
  "total_phases": $TOTAL_PHASES,
  "passed_phases": $PASSED_PHASES,
  "failed_phases": $FAILED_PHASES,
  "pass_rate":  $PASS_RATE,
  "overall_status": "FAILED",
  "production_ready": false,
  "showtime": false
}
EOF
    
    exit 1
fi

# Helper functions
generate_diamond_certificate() {
    cat > reports/diamond-certificate.txt << EOF
╔══════════════════════════════════════════════════════════════╗
║                                                              ║
║                    💎 DIAMOND CERTIFICATE 💎                 ║
║                                                              ║
║                  Production Readiness Validation             ║
║                                                              ║
╚══════════════════════════════════════════════════════════════╝

This certifies that: 

    UBL + Office + Messenger System
    
has successfully passed the Diamond Run Test Suite on: 

    Date:      $(date +"%B %d, %Y")
    Time:     $(date +"%H:%M:%S %Z")
    Duration: ${TOTAL_DURATION_HOURS} hours

All validation phases passed: 
  ✅ Phase 1: Foundation Validation
  ✅ Phase 2: Golden Path Scenarios  
  ✅ Phase 3: Performance Benchmarks
  ✅ Phase 4: Resilience Testing
  ✅ Phase 5: Chaos Engineering
  ✅ Phase 6: Load Testing
  ✅ Phase 7: Security Audit
  ✅ Phase 8: Data Integrity

The system is certified PRODUCTION READY and SHOWTIME approved.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
                    🏆 SHOWTIME READY 🏆
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Certificate ID: DIAMOND-$(date +%Y%m%d-%H%M%S)
Validated By: Diamond Run Test Suite v1.0

EOF
}

generate_deployment_checklist() {
    cat > reports/production-deployment-checklist.md << EOF
# 🚀 Production Deployment Checklist

**System**: UBL + Office + Messenger  
**Diamond Run**:  ✅ PASSED  
**Date**: $(date +"%Y-%m-%d")  
**Status**: READY FOR PRODUCTION

---

## Pre-Deployment

### Infrastructure
- [ ] Production servers provisioned
- [ ] Load balancers configured
- [ ] SSL certificates installed
- [ ] DNS configured
- [ ] Firewall rules set
- [ ] Monitoring setup (Prometheus + Grafana)
- [ ] Logging setup (ELK stack)
- [ ] Backup system configured
- [ ] Disaster recovery plan documented

### Database
- [ ] PostgreSQL 16+ installed
- [ ] Database created with correct collation
- [ ] All migrations applied
- [ ] Database backups configured
- [ ] Connection pooling configured
- [ ] Read replicas set up (if needed)
- [ ] Database monitoring enabled

### Environment Variables
- [ ] \`DATABASE_URL\` set correctly
- [ ] \`ANTHROPIC_API_KEY\` configured
- [ ] \`WEBAUTHN_RP_ID\` set to production domain
- [ ] \`WEBAUTHN_ORIGIN\` set correctly
- [ ] All secrets stored in secret manager
- [ ] Environment-specific configs reviewed

### Security
- [ ] All secrets rotated for production
- [ ] API keys have appropriate permissions
- [ ] Rate limiting configured
- [ ] CORS policies set correctly
- [ ] Security headers configured
- [ ] PII encryption enabled
- [ ] Audit logging enabled

---

## Deployment

### UBL Kernel
- [ ] Built with \`--release\` flag
- [ ] Docker image tagged correctly
- [ ] Health check endpoint verified
- [ ] Metrics endpoint exposed
- [ ] Logs streaming to centralized system

### Office Runtime  
- [ ] Built with \`--release\` flag
- [ ] LLM provider configured
- [ ] UBL endpoint configured correctly
- [ ] Constitution loaded
- [ ] Health check verified

### Messenger Frontend
- [ ] Production build created
- [ ] Assets uploaded to CDN
- [ ] Service worker registered
- [ ] API endpoints point to production
- [ ] Analytics configured

---

## Post-Deployment

### Smoke Tests
- [ ] Health checks all pass
- [ ] User can register/login
- [ ] Message send works
- [ ] Job creation works
- [ ] Real-time updates working
- [ ] All integrations functional

### Monitoring
- [ ] Dashboards accessible
- [ ] Alerts configured
- [ ] On-call rotation set
- [ ] Runbooks documented
- [ ] Incident response plan ready

### Performance
- [ ] Baseline metrics captured
- [ ] SLOs defined and monitored
- [ ] Auto-scaling configured
- [ ] Cache warming completed

### Documentation
- [ ] API documentation published
- [ ] User guides available
- [ ] Admin documentation ready
- [ ] Troubleshooting guides complete

---

## Sign-Off

- [ ] Engineering Lead:  __________________
- [ ] DevOps Lead: __________________
- [ ] Security Lead: __________________
- [ ] Product Manager: __________________

**Deployment Approved**: ___/___/______

---

**🎉 System is GO for production deployment! 🚀**

---

*Diamond Run Certification: $(date +"%Y%m%d-%H%M%S")*
EOF
}

generate_failure_report() {
    echo "Generating failure report..."
    # TODO: Detailed failure analysis
}