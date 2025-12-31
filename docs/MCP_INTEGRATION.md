# MCP (Model Context Protocol) Integration

O Office agora suporta o **Model Context Protocol (MCP)** para conectar com ferramentas externas dinamicamente.

## O que é MCP?

MCP é um protocolo aberto que permite que LLMs se comuniquem com "servers" que expõem:
- **Tools**: Funções executáveis (read_file, search, git_commit, etc.)
- **Resources**: Dados acessíveis (arquivos, URLs, banco de dados)
- **Prompts**: Templates reutilizáveis

O protocolo usa JSON-RPC 2.0 sobre stdio.

## Arquitetura

```
┌─────────────────────────────────────────────────────────────────┐
│                           OFFICE                                 │
│                                                                  │
│  ┌──────────────┐     ┌──────────────┐     ┌──────────────┐    │
│  │   LLM Job    │────▶│ ToolExecutor │────▶│ McpRegistry  │    │
│  │   Executor   │     │              │     │              │    │
│  └──────────────┘     └──────────────┘     └──────┬───────┘    │
│                                                    │            │
└────────────────────────────────────────────────────┼────────────┘
                                                     │
                        ┌────────────────────────────┼────────────┐
                        │                            ▼            │
                        │  ┌───────────┐  ┌───────────┐  ┌─────┐ │
                        │  │ McpClient │  │ McpClient │  │ ... │ │
                        │  └─────┬─────┘  └─────┬─────┘  └──┬──┘ │
                        │        │               │           │    │
                        │  ┌─────▼─────┐  ┌─────▼─────┐  ┌──▼──┐ │
                        │  │filesystem │  │  github   │  │ ... │ │
                        │  │  server   │  │  server   │  │     │ │
                        │  └───────────┘  └───────────┘  └─────┘ │
                        │                                         │
                        │        MCP Servers (stdio)              │
                        └─────────────────────────────────────────┘
```

## Configuração

### Via Variáveis de Ambiente

```bash
# Habilitar MCP
export OFFICE__MCP__ENABLED=true

# Servidor de Filesystem
export OFFICE__MCP__FILESYSTEM_PATHS="/home/user/project,/home/user/docs"

# Servidor GitHub
export OFFICE__MCP__GITHUB_TOKEN="ghp_xxxxx"

# Servidor Brave Search
export OFFICE__MCP__BRAVE_API_KEY="BSA-xxxxx"
```

### Via JSON

```bash
export OFFICE__MCP__CONFIG='{
  "enabled": true,
  "servers": [
    {
      "name": "filesystem",
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/home/user"],
      "auto_start": true
    },
    {
      "name": "github",
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-github"],
      "env": {
        "GITHUB_PERSONAL_ACCESS_TOKEN": "ghp_xxxxx"
      }
    }
  ]
}'
```

## Servidores MCP Disponíveis

### Oficiais (modelcontextprotocol)

| Server | Descrição | Install |
|--------|-----------|---------|
| `server-filesystem` | Leitura/escrita de arquivos | `npx @modelcontextprotocol/server-filesystem <paths>` |
| `server-github` | GitHub API | `npx @modelcontextprotocol/server-github` |
| `server-brave-search` | Busca web | `npx @modelcontextprotocol/server-brave-search` |
| `server-sqlite` | SQLite queries | `npx @modelcontextprotocol/server-sqlite <db>` |
| `server-puppeteer` | Browser automation | `npx @modelcontextprotocol/server-puppeteer` |
| `server-memory` | Knowledge graph | `npx @modelcontextprotocol/server-memory` |

### Comunidade

| Server | Descrição | Repo |
|--------|-----------|------|
| `mcp-server-docker` | Docker management | github.com/... |
| `mcp-server-kubernetes` | K8s operations | github.com/... |
| `playwright-mcp` | E2E testing | github.com/... |

## API Endpoints

### Status

```bash
GET /mcp/status
```

Response:
```json
{
  "enabled": true,
  "servers": [
    { "name": "filesystem", "connected": true },
    { "name": "github", "connected": true }
  ],
  "tool_count": 15,
  "resource_count": 3
}
```

### Listar Tools

```bash
GET /mcp/tools
```

Response:
```json
[
  {
    "name": "read_file",
    "server": "filesystem",
    "description": "Read the contents of a file",
    "parameters": {
      "path": { "type": "string", "description": "File path" }
    }
  }
]
```

### Chamar Tool

```bash
POST /mcp/tools/filesystem:read_file/call
Content-Type: application/json

{
  "arguments": {
    "path": "/home/user/test.txt"
  }
}
```

Response:
```json
{
  "success": true,
  "content": [
    { "type": "text", "text": "File contents here..." }
  ]
}
```

### Gerenciar Servers

```bash
# Iniciar server
POST /mcp/servers/filesystem

# Parar server
DELETE /mcp/servers/filesystem
```

## Uso pelo LLM

Quando tools MCP estão disponíveis, o LLM recebe instruções no prompt:

```
## Tool Usage

You have access to the following tools. To use a tool, respond with:

{
  "tool_calls": [
    { "name": "filesystem:read_file", "arguments": { "path": "/etc/hosts" } }
  ]
}

### Available Tools

**filesystem:read_file**: Read file contents
  - `path` (required): string - The file path to read

**filesystem:write_file**: Write file contents
  - `path` (required): string - The file path
  - `content` (required): string - Content to write

**github:search_repositories**: Search GitHub repos
  - `query` (required): string - Search query
```

## ToolExecutor

O `ToolExecutor` é o componente que:

1. Recebe chamadas de tool do LLM
2. Roteia para o MCP server correto
3. Executa com timeout
4. Registra no audit log
5. Retorna resultado formatado

```rust
use office::mcp::ToolExecutor;

let executor = ToolExecutor::new(registry, audit)
    .with_max_calls(20)
    .with_timeout(30);

let result = executor.execute_tool(
    "session_123",
    "filesystem:read_file",
    Some(json!({ "path": "/tmp/test.txt" }))
).await?;
```

## Segurança

### Sandboxing

MCP servers rodam como processos separados. Considere:

- Rodar em containers com limites de recursos
- Usar `--allowed-paths` para filesystem
- Tokens com permissões mínimas

### Audit Trail

Todas as chamadas de tool são registradas:

```json
{
  "call_id": "uuid",
  "session_id": "user_session",
  "tool_name": "filesystem:write_file",
  "arguments": { "path": "REDACTED", "content": "REDACTED" },
  "result": "success",
  "duration_ms": 45
}
```

### Rate Limiting

- `max_calls_per_request`: 20 (padrão)
- `tool_timeout_secs`: 30 (padrão)

## Criando Seu Próprio MCP Server

```python
# my_server.py
import json
import sys

def handle_request(request):
    method = request.get("method")
    
    if method == "initialize":
        return {
            "protocolVersion": "2024-11-05",
            "capabilities": { "tools": {} },
            "serverInfo": { "name": "my-server", "version": "1.0" }
        }
    
    if method == "tools/list":
        return {
            "tools": [{
                "name": "my_tool",
                "description": "Does something",
                "inputSchema": { "type": "object", "properties": {} }
            }]
        }
    
    if method == "tools/call":
        return {
            "content": [{ "type": "text", "text": "Result!" }]
        }

for line in sys.stdin:
    request = json.loads(line)
    result = handle_request(request)
    response = {
        "jsonrpc": "2.0",
        "id": request["id"],
        "result": result
    }
    print(json.dumps(response), flush=True)
```

## Roadmap

- [ ] Hot reload de servers sem restart
- [ ] Pool de servers para alta disponibilidade
- [ ] Métricas por tool no Prometheus
- [ ] Cache de resultados para tools idempotentes
- [ ] Streaming de resultados longos
