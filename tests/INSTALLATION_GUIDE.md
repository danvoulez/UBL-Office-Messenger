# Guia de Instalação - Dependências dos Testes

## ⚠️ Instalação Manual Necessária

As dependências precisam ser instaladas manualmente com privilégios de administrador.

## 1. Instalar Homebrew

Abra o Terminal e execute:

```bash
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
```

Siga as instruções na tela. Você precisará inserir sua senha de administrador.

## 2. Instalar Docker Desktop

### Opção A: Via Homebrew (recomendado)
```bash
brew install --cask docker
```

### Opção B: Download Manual
1. Acesse: https://www.docker.com/products/docker-desktop/
2. Baixe Docker Desktop para macOS
3. Instale o arquivo `.dmg`
4. Arraste Docker para Applications
5. Abra Docker Desktop e aguarde a inicialização

### Verificar Instalação
```bash
docker --version
docker-compose --version  # ou: docker compose version
```

## 3. Instalar Node.js

```bash
brew install node
```

### Verificar Instalação
```bash
node --version
npm --version
```

## 4. Instalar K6

```bash
brew install k6
```

### Verificar Instalação
```bash
k6 version
```

## 5. Iniciar Docker Desktop

Após instalar Docker Desktop:
1. Abra o Docker Desktop (Applications > Docker)
2. Aguarde a inicialização completa (ícone da baleia no menu superior)
3. Verifique com: `docker info`

## Verificação Completa

Execute este comando para verificar todas as dependências:

```bash
echo "=== VERIFICAÇÃO DE DEPENDÊNCIAS ===" && \
echo "" && \
echo "Docker:" && docker --version 2>&1 && \
echo "" && \
echo "Docker Compose:" && (docker-compose --version 2>&1 || docker compose version 2>&1) && \
echo "" && \
echo "Node.js:" && node --version 2>&1 && \
echo "" && \
echo "K6:" && k6 version 2>&1 && \
echo "" && \
echo "✅ Todas as dependências instaladas!"
```

## Próximos Passos Após Instalação

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

## Troubleshooting

### Docker não inicia
- Verifique se o Docker Desktop está instalado
- Tente reiniciar o Docker Desktop
- Verifique se há processos Docker antigos: `ps aux | grep docker`

### Homebrew não funciona
- Verifique se o PATH está configurado:
  ```bash
  echo 'eval "$(/opt/homebrew/bin/brew shellenv)"' >> ~/.zprofile
  eval "$(/opt/homebrew/bin/brew shellenv)"
  ```

### Permissões negadas
- Certifique-se de que você tem privilégios de administrador
- Tente executar com `sudo` (se necessário)


