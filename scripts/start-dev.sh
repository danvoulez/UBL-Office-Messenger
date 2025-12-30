#!/bin/bash
# UBL 3.0 Development Orchestrator
# Starts all services natively for local development

set -e

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m'

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
LOG_DIR="$ROOT_DIR/.logs"
PID_DIR="$ROOT_DIR/.pids"

mkdir -p "$LOG_DIR" "$PID_DIR"

echo -e "${BLUE}╔═══════════════════════════════════════╗${NC}"
echo -e "${BLUE}║       UBL 3.0 Dev Orchestrator        ║${NC}"
echo -e "${BLUE}╚═══════════════════════════════════════╝${NC}"
echo ""

# Function to check if port is in use
check_port() {
    lsof -i :$1 >/dev/null 2>&1
}

# Function to wait for service
wait_for_service() {
    local url=$1
    local name=$2
    local max_attempts=30
    local attempt=0
    
    while [ $attempt -lt $max_attempts ]; do
        if curl -s "$url" >/dev/null 2>&1; then
            echo -e "  ${GREEN}✓${NC} $name ready"
            return 0
        fi
        attempt=$((attempt + 1))
        sleep 1
    done
    echo -e "  ${RED}✗${NC} $name failed to start"
    return 1
}

# Stop existing services
stop_services() {
    echo -e "${YELLOW}Stopping existing services...${NC}"
    
    # Kill by PID files
    for pid_file in "$PID_DIR"/*.pid; do
        if [ -f "$pid_file" ]; then
            pid=$(cat "$pid_file")
            kill "$pid" 2>/dev/null || true
            rm "$pid_file"
        fi
    done
    
    # Kill known processes
    pkill -f "ubl-server" 2>/dev/null || true
    pkill -f "vite.*messenger" 2>/dev/null || true
    pkill -f "office.*server" 2>/dev/null || true
    
    sleep 2
    echo -e "  ${GREEN}✓${NC} Services stopped"
}

# Start PostgreSQL (assumes already running via Docker or brew)
start_postgres() {
    echo -e "\n${BLUE}1. PostgreSQL${NC}"
    
    if check_port 5432; then
        echo -e "  ${GREEN}✓${NC} Already running on port 5432"
    else
        echo -e "  ${YELLOW}!${NC} Not running. Starting via Docker..."
        docker run -d --name ubl-postgres \
            -e POSTGRES_USER=lab2 \
            -e POSTGRES_DB=ubl_ledger \
            -e POSTGRES_HOST_AUTH_METHOD=trust \
            -p 5432:5432 \
            postgres:16-alpine >/dev/null 2>&1 || true
        sleep 3
        echo -e "  ${GREEN}✓${NC} Started"
    fi
}

# Start UBL Kernel
start_ubl_kernel() {
    echo -e "\n${BLUE}2. UBL Kernel${NC} (port 8080)"
    
    if check_port 8080; then
        if curl -s http://localhost:8080/health | grep -q "healthy"; then
            echo -e "  ${GREEN}✓${NC} Already running and healthy"
            return 0
        fi
        echo -e "  ${YELLOW}!${NC} Port in use but not healthy, restarting..."
        pkill -f "ubl-server" 2>/dev/null || true
        sleep 2
    fi
    
    cd "$ROOT_DIR/ubl/kernel/rust"
    
    DATABASE_URL="postgres://lab2@localhost/ubl_ledger" \
    WEBAUTHN_ORIGIN="http://localhost:3000" \
    WEBAUTHN_RP_ID="localhost" \
    cargo run --release --bin ubl-server > "$LOG_DIR/ubl-kernel.log" 2>&1 &
    
    echo $! > "$PID_DIR/ubl-kernel.pid"
    
    wait_for_service "http://localhost:8080/health" "UBL Kernel"
}

# Start Messenger Frontend
start_messenger() {
    echo -e "\n${BLUE}3. Messenger Frontend${NC} (port 3000)"
    
    if check_port 3000; then
        echo -e "  ${GREEN}✓${NC} Already running on port 3000"
        return 0
    fi
    
    cd "$ROOT_DIR/apps/messenger/frontend"
    
    npm run dev > "$LOG_DIR/messenger.log" 2>&1 &
    
    echo $! > "$PID_DIR/messenger.pid"
    
    wait_for_service "http://localhost:3000" "Messenger"
}

# Start Office (optional - needs API key)
start_office() {
    echo -e "\n${BLUE}4. Office Runtime${NC} (port 8081)"
    
    if [ -z "$ANTHROPIC_API_KEY" ]; then
        echo -e "  ${YELLOW}⊘${NC} Skipped (no ANTHROPIC_API_KEY)"
        return 0
    fi
    
    if check_port 8081; then
        echo -e "  ${GREEN}✓${NC} Already running on port 8081"
        return 0
    fi
    
    cd "$ROOT_DIR/apps/office"
    
    OFFICE__LLM__API_KEY="$ANTHROPIC_API_KEY" \
    OFFICE__UBL__ENDPOINT="http://localhost:8080" \
    cargo run --release > "$LOG_DIR/office.log" 2>&1 &
    
    echo $! > "$PID_DIR/office.pid"
    
    wait_for_service "http://localhost:8081/health" "Office"
}

# Print status
print_status() {
    echo -e "\n${BLUE}═══════════════════════════════════════${NC}"
    echo -e "${GREEN}UBL 3.0 Development Stack Running${NC}"
    echo -e "${BLUE}═══════════════════════════════════════${NC}"
    echo ""
    echo -e "  📦 PostgreSQL:  http://localhost:5432"
    echo -e "  🔐 UBL Kernel:  http://localhost:8080"
    echo -e "  💬 Messenger:   http://localhost:3000"
    if [ -n "$ANTHROPIC_API_KEY" ]; then
        echo -e "  🧠 Office:      http://localhost:8081"
    fi
    echo ""
    echo -e "  📋 Logs:        $LOG_DIR/"
    echo -e "  🛑 Stop:        $0 stop"
    echo ""
}

# Main
case "${1:-start}" in
    start)
        start_postgres
        start_ubl_kernel
        start_messenger
        start_office
        print_status
        ;;
    stop)
        stop_services
        ;;
    restart)
        stop_services
        sleep 2
        $0 start
        ;;
    status)
        echo -e "${BLUE}Service Status:${NC}"
        echo -e "  PostgreSQL: $(check_port 5432 && echo '✓ Running' || echo '✗ Stopped')"
        echo -e "  UBL Kernel: $(curl -s http://localhost:8080/health 2>/dev/null | grep -q healthy && echo '✓ Healthy' || echo '✗ Stopped')"
        echo -e "  Messenger:  $(check_port 3000 && echo '✓ Running' || echo '✗ Stopped')"
        echo -e "  Office:     $(curl -s http://localhost:8081/health 2>/dev/null | grep -q healthy && echo '✓ Healthy' || echo '⊘ Not configured')"
        ;;
    logs)
        tail -f "$LOG_DIR"/*.log
        ;;
    *)
        echo "Usage: $0 {start|stop|restart|status|logs}"
        exit 1
        ;;
esac
