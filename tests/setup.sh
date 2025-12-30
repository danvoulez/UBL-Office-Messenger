#!/bin/bash
# Setup script for integration tests

set -e

echo "🚀 Setting up integration test environment..."

# Load environment variables
if [ -f .env.test ]; then
    set -a
    source .env.test
    set +a
fi

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Check Docker
if ! command -v docker &> /dev/null; then
    echo "❌ Docker not installed"
    exit 1
fi

if ! command -v docker-compose &> /dev/null; then
    echo "❌ Docker Compose not installed"
    exit 1
fi

echo -e "${GREEN}✅ Docker installed${NC}"

# Stop any existing containers
echo "🛑 Stopping existing containers..."
docker-compose -f docker-compose.integration.yml down -v 2>/dev/null || true

# Pull latest images
echo "📥 Pulling Docker images..."
docker-compose -f docker-compose.integration.yml pull

# Build services
echo "🔨 Building services..."
docker-compose -f docker-compose.integration.yml build

# Start PostgreSQL first
echo "🗄️  Starting PostgreSQL..."
docker-compose -f docker-compose.integration.yml up -d postgres

# Wait for PostgreSQL
echo "⏳ Waiting for PostgreSQL..."
for i in {1..30}; do
    if docker-compose -f docker-compose.integration.yml exec -T postgres pg_isready -U ubl_test > /dev/null 2>&1; then
        echo -e "${GREEN}✅ PostgreSQL ready${NC}"
        break
    fi
    if [ $i -eq 30 ]; then
        echo "❌ PostgreSQL failed to start"
        docker-compose -f docker-compose.integration.yml logs postgres
        exit 1
    fi
    sleep 2
done

# Run migrations
echo "🔄 Running database migrations..."
docker-compose -f docker-compose.integration.yml run --rm ubl-kernel sqlx migrate run || true

# Start UBL Kernel
echo "🏛️  Starting UBL Kernel..."
docker-compose -f docker-compose.integration.yml up -d ubl-kernel

# Wait for UBL Kernel
echo "⏳ Waiting for UBL Kernel..."
for i in {1..60}; do
    if curl -sf http://localhost:8080/health > /dev/null 2>&1; then
        echo -e "${GREEN}✅ UBL Kernel ready${NC}"
        break
    fi
    if [ $i -eq 60 ]; then
        echo "❌ UBL Kernel failed to start"
        docker-compose -f docker-compose.integration.yml logs ubl-kernel
        exit 1
    fi
    sleep 2
done

# Start Office
echo "🏢 Starting Office..."
docker-compose -f docker-compose.integration.yml up -d office

# Wait for Office
echo "⏳ Waiting for Office..."
for i in {1..60}; do
    if curl -sf http://localhost:8081/health > /dev/null 2>&1; then
        echo -e "${GREEN}✅ Office ready${NC}"
        break
    fi
    if [ $i -eq 60 ]; then
        echo "❌ Office failed to start"
        docker-compose -f docker-compose.integration.yml logs office
        exit 1
    fi
    sleep 2
done

# Start Messenger
echo "💬 Starting Messenger..."
docker-compose -f docker-compose.integration.yml up -d messenger

# Wait for Messenger
echo "⏳ Waiting for Messenger..."
for i in {1..60}; do
    if curl -sf http://localhost:3000 > /dev/null 2>&1; then
        echo -e "${GREEN}✅ Messenger ready${NC}"
        break
    fi
    if [ $i -eq 60 ]; then
        echo -e "${YELLOW}⚠️  Messenger may not be ready${NC}"
        break
    fi
    sleep 2
done

echo ""
echo "=============================================="
echo -e "${GREEN}✅ Integration environment ready! ${NC}"
echo "=============================================="
echo ""
echo "Services:"
echo "  - PostgreSQL:  localhost:5432"
echo "  - UBL Kernel: http://localhost:8080"
echo "  - Office:      http://localhost:8081"
echo "  - Messenger:   http://localhost:3000"
echo ""
echo "To run tests:"
echo "  cd rust && cargo test --all-features"
echo "  cd ../e2e && npm test"
echo ""
echo "To view logs:"
echo "  docker-compose -f docker-compose.integration.yml logs -f [service]"
echo ""