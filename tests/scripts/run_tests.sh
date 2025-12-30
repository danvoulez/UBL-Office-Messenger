#!/bin/bash
# Comprehensive test runner for Office

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
COVERAGE_THRESHOLD=90

echo "=============================================="
echo "  🧪 Office Runtime Test Suite"
echo "=============================================="
echo ""

# Check dependencies
echo "📋 Checking dependencies..."
command -v cargo >/dev/null 2>&1 || { echo "❌ Cargo not installed"; exit 1; }
echo "✅ Dependencies OK"
echo ""

# Check formatting
echo "📝 Checking code formatting..."
if cargo fmt --all -- --check; then
    echo -e "${GREEN}✅ Formatting OK${NC}"
else
    echo -e "${RED}❌ Formatting failed${NC}"
    echo "Run: cargo fmt --all"
    exit 1
fi
echo ""

# Run Clippy
echo "🔍 Running Clippy..."
if cargo clippy --all-targets --all-features -- -D warnings; then
    echo -e "${GREEN}✅ Clippy OK${NC}"
else
    echo -e "${RED}❌ Clippy failed${NC}"
    exit 1
fi
echo ""

# Run unit tests
echo "🧪 Running unit tests..."
if cargo test --lib --all-features; then
    echo -e "${GREEN}✅ Unit tests passed${NC}"
else
    echo -e "${RED}❌ Unit tests failed${NC}"
    exit 1
fi
echo ""

# Run integration tests
echo "🔗 Running integration tests..."
if cargo test --test '*' --all-features; then
    echo -e "${GREEN}✅ Integration tests passed${NC}"
else
    echo -e "${RED}❌ Integration tests failed${NC}"
    exit 1
fi
echo ""

# Generate coverage
echo "📊 Generating coverage report..."
if command -v cargo-tarpaulin >/dev/null 2>&1; then
    cargo tarpaulin --all-features \
        --out Xml --out Html \
        --timeout 300 \
        --exclude-files '*/tests/*'
    
    # Check coverage threshold
    if command -v grep >/dev/null 2>&1 && command -v bc >/dev/null 2>&1; then
        COVERAGE=$(grep -oP 'line-rate="\K[0-9.]+' cobertura.xml | head -1)
        COVERAGE_PCT=$(echo "$COVERAGE * 100" | bc)
        
        echo ""
        echo "Coverage: ${COVERAGE_PCT}%"
        
        if (( $(echo "$COVERAGE_PCT >= $COVERAGE_THRESHOLD" | bc -l) )); then
            echo -e "${GREEN}✅ Coverage OK (>= ${COVERAGE_THRESHOLD}%)${NC}"
        else
            echo -e "${RED}❌ Coverage below threshold (${COVERAGE_PCT}% < ${COVERAGE_THRESHOLD}%)${NC}"
            exit 1
        fi
        
        echo ""
        echo "📄 Coverage report:  file://$(pwd)/tarpaulin-report.html"
    fi
else
    echo -e "${YELLOW}⚠️  cargo-tarpaulin not installed, skipping coverage${NC}"
    echo "Install with: cargo install cargo-tarpaulin"
fi
echo ""

# Security audit
echo "🔒 Running security audit..."
if command -v cargo-audit >/dev/null 2>&1; then
    if cargo audit; then
        echo -e "${GREEN}✅ No security vulnerabilities${NC}"
    else
        echo -e "${YELLOW}⚠️  Security audit found issues${NC}"
    fi
else
    echo -e "${YELLOW}⚠️  cargo-audit not installed, skipping${NC}"
    echo "Install with: cargo install cargo-audit"
fi
echo ""

echo "=============================================="
echo -e "${GREEN}✅ All tests passed!${NC}"
echo "=============================================="