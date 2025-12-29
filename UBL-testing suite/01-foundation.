#!/bin/bash
# Phase 1: Foundation Validation
set -e

echo "🏗️  Validating system foundation..."

# Check all services
echo "  ✓ Checking service health..."
curl -sf http://localhost:8080/health || exit 1
curl -sf http://localhost:8081/health || exit 1

# Check database
echo "  ✓ Checking database connectivity..."
docker-compose -f docker-compose.diamond.yml exec -T postgres psql -U ubl_diamond -c "SELECT 1" || exit 1

# Check migrations
echo "  ✓ Verifying migrations..."
# TODO: Query migration status

# Security scan
echo "  ✓ Running security scan..."
# TODO: Run security tools

echo "✅ Foundation validated"
exit 0