#!/bin/bash
# Setup Complete Observability Stack

set -e

cd "$(dirname "$0")/.."

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo "🔭 Setting up Observability Stack"
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

# Load environment variables
if [ -f .env ]; then
    export $(cat .env | xargs)
else
    echo -e "${YELLOW}⚠️  No .env file found, using defaults${NC}"
fi

# Create directories
echo "📁 Creating directories..."
mkdir -p {prometheus,grafana,loki,promtail,jaeger,alertmanager}/data
mkdir -p grafana/provisioning/{datasources,dashboards}
mkdir -p alertmanager/templates

# Set permissions
chmod -R 777 {prometheus,grafana,loki,promtail,jaeger,alertmanager}/data

echo -e "${GREEN}✅ Directories created${NC}"
echo ""

# Stop existing stack
echo "🛑 Stopping existing observability stack..."
docker-compose -f docker-compose.observability.yml down 2>/dev/null || true

# Start stack
echo "🚀 Starting observability stack..."
docker-compose -f docker-compose.observability.yml up -d

# Wait for services
echo "⏳ Waiting for services to start..."
sleep 30

# Check service health
echo "🏥 Checking service health..."

services=("prometheus:9090" "grafana:3000" "loki:3100" "jaeger:16686" "alertmanager:9093")
all_healthy=true

for service in "${services[@]}"; do
    IFS=':' read -r name port <<< "$service"
    
    if curl -sf "http://localhost:${port}" > /dev/null 2>&1; then
        echo -e "  ${GREEN}✅ ${name}${NC}"
    else
        echo -e "  ${RED}❌ ${name}${NC}"
        all_healthy=false
    fi
done

echo ""

if [ "$all_healthy" = true ]; then
    echo -e "${GREEN}╔══════════════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║      ✅ Observability Stack Ready ✅                     ║${NC}"
    echo -e "${GREEN}╚══════════════════════════════════════════════════════════╝${NC}"
    echo ""
    echo "📊 Access URLs:"
    echo "  - Prometheus:      http://localhost:9090"
    echo "  - Grafana:        http://localhost:3001 (admin/admin)"
    echo "  - Jaeger:         http://localhost:16686"
    echo "  - Alertmanager:   http://localhost:9093"
    echo "  - Loki:           http://localhost:3100"
    echo ""
    echo "📖 Documentation: observability/README.md"
    echo ""
    
    # Import Grafana dashboards
    echo "📊 Importing Grafana dashboards..."
    sleep 10
    
    for dashboard in grafana/provisioning/dashboards/*.json; do
        if [ -f "$dashboard" ]; then
            dashboard_name=$(basename "$dashboard" .json)
            echo "  Importing ${dashboard_name}..."
            
            curl -X POST \
                -H "Content-Type: application/json" \
                -d @"$dashboard" \
                http://admin:${GRAFANA_PASSWORD:-admin}@localhost:3001/api/dashboards/db \
                > /dev/null 2>&1 || echo "  ⚠️  Failed to import ${dashboard_name}"
        fi
    done
    
    echo ""
    echo -e "${GREEN}✅ Setup complete!${NC}"
    
else
    echo -e "${RED}╔══════════════════════════════════════════════════════════╗${NC}"
    echo -e "${RED}║      ❌ Some Services Failed to Start ❌                ║${NC}"
    echo -e "${RED}╚══════════════════════════════════════════════════════════╝${NC}"
    echo ""
    echo "Check logs with: docker-compose -f docker-compose.observability.yml logs"
    exit 1
fi