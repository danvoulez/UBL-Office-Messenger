# Status de Instalação - Dependências dos Testes

**Data:** $(date)

## Dependências Necessárias

### 1. Docker & Docker Compose
- **Necessário para:** Rodar PostgreSQL, UBL Server e Office Runtime em containers
- **Status:** Verificando...

### 2. Node.js
- **Necessário para:** Executar testes K6 (load testing)
- **Status:** Verificando...

### 3. K6
- **Necessário para:** Testes de carga e performance
- **Status:** Verificando...

## Instalação

### macOS (via Homebrew)

```bash
# Instalar Homebrew (se não tiver)
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Instalar Docker Desktop
brew install --cask docker

# Instalar Node.js
brew install node

# Instalar K6
brew install k6
```

### Iniciar Docker

Após instalar Docker Desktop:
1. Abrir Docker Desktop
2. Aguardar inicialização completa
3. Verificar com: `docker info`

## Verificação

Execute para verificar todas as dependências:

```bash
docker --version
docker-compose --version  # ou: docker compose version
node --version
k6 version
```

## Próximos Passos

Após instalar todas as dependências:

1. **Iniciar Docker Desktop**
2. **Configurar ambiente de teste:**
   ```bash
   cd "UBL-testing suite"
   ./setup.sh
   ```

3. **Executar testes:**
   ```bash
   ./01-foundation.sh
   ```


