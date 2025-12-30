#!/bin/bash
# Teste rápido do UBL Server

set -e

echo "🧪 Testando UBL Server"
echo "======================"
echo ""

# Cores
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Verificar se o servidor está rodando
echo "1️⃣  Verificando se o servidor está rodando..."
if curl -s http://localhost:8080/health > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Servidor está rodando${NC}"
    echo ""
    echo "📊 Health Check:"
    curl -s http://localhost:8080/health | jq . || curl -s http://localhost:8080/health
    echo ""
else
    echo -e "${YELLOW}⚠️  Servidor não está rodando${NC}"
    echo ""
    echo "Para iniciar o servidor:"
    echo "  cd ubl/kernel/rust/ubl-server"
    echo "  export DATABASE_URL=postgres://localhost:5432/ubl_dev"
    echo "  cargo run --release"
    echo ""
    exit 1
fi

# Testar endpoints básicos
echo "2️⃣  Testando endpoints básicos..."
echo ""

echo "📋 GET /health:"
curl -s http://localhost:8080/health | jq . || curl -s http://localhost:8080/health
echo ""

echo "📋 GET /state/C.Messenger:"
curl -s http://localhost:8080/state/C.Messenger | jq . || curl -s http://localhost:8080/state/C.Messenger
echo ""

echo "📋 GET /atom/test (deve retornar 404):"
curl -s -o /dev/null -w "Status: %{http_code}\n" http://localhost:8080/atom/test
echo ""

echo -e "${GREEN}✅ Testes básicos concluídos${NC}"
echo ""
echo "Para mais testes, veja:"
echo "  - ubl/scripts/smoke-test.sh"
echo "  - ubl/scripts/test_console_flow.sh"



