# Configuração Docker no Cursor IDE

## 1. Instalar Extensão Docker no Cursor

1. Abra o Cursor IDE
2. Vá para Extensions (Cmd+Shift+X)
3. Procure por "Docker" (publicada pela Microsoft)
4. Clique em "Install"

## 2. Verificar Instalação do Docker

A extensão Docker requer que o Docker Desktop esteja instalado e rodando.

### Verificar se Docker está instalado:
```bash
docker --version
```

### Se não estiver instalado:
- **macOS:** Baixe Docker Desktop de https://www.docker.com/products/docker-desktop/
- Ou use Homebrew: `brew install --cask docker`

## 3. Iniciar Docker Desktop

1. Abra Docker Desktop (Applications > Docker)
2. Aguarde até ver o ícone da baleia no menu superior (status: "Docker Desktop is running")
3. A extensão Docker no Cursor detectará automaticamente

## 4. Usar Extensão Docker no Cursor

### Painel Docker
- Abra o painel Docker na barra lateral (ícone da baleia)
- Você verá:
  - **Containers** - Containers rodando/parados
  - **Images** - Imagens Docker disponíveis
  - **Volumes** - Volumes de dados
  - **Networks** - Redes Docker

### Comandos Úteis na Extensão

1. **Build Image:**
   - Clique com botão direito em um Dockerfile
   - Selecione "Build Image"

2. **Run Container:**
   - Clique com botão direito em uma imagem
   - Selecione "Run"

3. **View Logs:**
   - Clique com botão direito em um container
   - Selecione "View Logs"

4. **Start/Stop Container:**
   - Use os botões no painel Docker

## 5. Configurar Ambiente de Testes

### Usando Docker Compose via Cursor

1. **Abrir docker-compose.yml:**
   - Navegue até `docker-compose.integration.yml`
   - A extensão Docker mostrará botões de ação

2. **Executar Compose:**
   - Clique com botão direito no arquivo
   - Selecione "Compose Up" ou "Compose Up - Detached"

3. **Parar Serviços:**
   - Clique com botão direito no arquivo
   - Selecione "Compose Down"

### Ou via Terminal Integrado

No terminal integrado do Cursor (Ctrl+`):

```bash
cd "UBL-testing suite"

# Iniciar serviços
docker-compose -f docker-compose.integration.yml up -d

# Ver logs
docker-compose -f docker-compose.integration.yml logs -f

# Parar serviços
docker-compose -f docker-compose.integration.yml down
```

## 6. Verificar Status

### No Painel Docker do Cursor:
- Verifique se os containers estão rodando:
  - `postgres` (PostgreSQL)
  - `ubl-kernel` (UBL Server)
  - `office` (Office Runtime)

### Via Terminal:
```bash
docker ps
```

## 7. Troubleshooting

### Docker não detectado pela extensão:
1. Verifique se Docker Desktop está rodando
2. Reinicie o Cursor IDE
3. Verifique se a extensão Docker está instalada e habilitada

### Containers não iniciam:
1. Verifique logs no painel Docker
2. Verifique se as portas não estão em uso:
   ```bash
   lsof -i :8080  # UBL Server
   lsof -i :8081  # Office
   lsof -i :5432  # PostgreSQL
   ```

### Permissões:
- Certifique-se de que o usuário está no grupo `docker`:
  ```bash
  groups | grep docker
  ```

## 8. Workflow Recomendado

1. **Abrir Cursor IDE**
2. **Abrir pasta do projeto:** `OFFICE-main`
3. **Instalar extensão Docker** (se não tiver)
4. **Iniciar Docker Desktop**
5. **Abrir painel Docker** (barra lateral)
6. **Executar setup:**
   ```bash
   cd "UBL-testing suite"
   ./setup.sh
   ```
7. **Monitorar containers** no painel Docker
8. **Executar testes:**
   ```bash
   ./01-foundation.sh
   ```

## 9. Atalhos Úteis

- **Painel Docker:** Cmd+Shift+P → "Docker: Focus on Docker View"
- **Refresh:** Clique no ícone de refresh no painel Docker
- **Terminal Integrado:** Ctrl+` (backtick)

## 10. Recursos Adicionais

- **Docker Extension Docs:** https://code.visualstudio.com/docs/containers/overview
- **Docker Desktop Docs:** https://docs.docker.com/desktop/


