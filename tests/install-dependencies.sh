#!/bin/bash
# Script de instalação de dependências para testes UBL

set -e

echo "🚀 Instalando dependências para testes UBL..."
echo ""

# Verificar se Homebrew está instalado
if ! command -v brew &> /dev/null; then
    echo "📦 Instalando Homebrew..."
    /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
    
    # Adicionar Homebrew ao PATH (se necessário)
    if [ -f "/opt/homebrew/bin/brew" ]; then
        eval "$(/opt/homebrew/bin/brew shellenv)"
    fi
else
    echo "✅ Homebrew já instalado"
fi

# Instalar Docker Desktop
if ! command -v docker &> /dev/null; then
    echo "🐳 Instalando Docker Desktop..."
    brew install --cask docker
    echo "⚠️  Por favor, abra o Docker Desktop manualmente após a instalação"
else
    echo "✅ Docker já instalado"
fi

# Instalar Node.js
if ! command -v node &> /dev/null; then
    echo "📦 Instalando Node.js..."
    brew install node
else
    echo "✅ Node.js já instalado"
fi

# Instalar K6
if ! command -v k6 &> /dev/null; then
    echo "📦 Instalando K6..."
    brew install k6
else
    echo "✅ K6 já instalado"
fi

echo ""
echo "✅ Instalação concluída!"
echo ""
echo "📋 Próximos passos:"
echo "  1. Abra o Docker Desktop"
echo "  2. Execute: ./setup.sh"
echo "  3. Execute: ./01-foundation.sh"
