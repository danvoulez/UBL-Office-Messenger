# MCP (Model Context Protocol) Integration

O Office implementa MCP de forma **unificada**: tools nativas e externas são expostas com a mesma interface.

**O LLM só vê MCP. Não sabe se é nativo ou externo.**

## Como o Prompt é Injetado

O `Narrator` gera uma narrativa estruturada que é passada como **system prompt** do LLM.
A seção de tools MCP é injetada automaticamente quando tools estão disponíveis:

```
# IDENTITY
You are **Aria**, an LLM Entity...

# CURRENT SITUATION
You are in a work session...

# RECENT MEMORY
Last events...

# AVAILABLE CAPABILITIES
(affordances from UBL)

# TOOL SYSTEM (MCP)  ← Nova seção injetada aqui
You have access to external tools via Model Context Protocol...

## How to Use Tools
```json
{
  "tool_calls": [{ "name": "server:tool", "arguments": {} }]
}
```

## Available Tools
### Native Tools (office:*)
- office:ubl_query: Query the UBL ledger
- office:memory_recall: Search semantic memory
...

### filesystem Tools
- filesystem:read_file: Read file contents
...

## Best Practices
1. Prefer native tools (office:*) for Office-specific operations
2. Check before writing - use office:permit_check for risky operations
...

# PREVIOUS INSTANCE HANDOVER
...

# CONSTITUTION
...
```

### Configuração do Narrator

```rust
use office::context::{Narrator, NarrativeConfig, ToolInfo};

let config = NarrativeConfig {
    include_tool_orientation: true,  // Incluir seção de tools
    show_tool_parameters: false,     // Detalhes dos parâmetros
    ..Default::default()
};

let tools = vec![
    ToolInfo {
        name: "office:ubl_query".to_string(),
        description: "Query the UBL ledger".to_string(),
        parameters: None,
    },
];

let narrator = Narrator::new(config).with_tools(tools);
let narrative = narrator.generate(&context_frame);
```

## Arquitetura Unificada

```
                          LLM
                           │
                           ▼
                ┌────────────────────┐
                │ UnifiedToolRegistry│
                └─────────┬──────────┘
                          │
        ┌─────────────────┼─────────────────┐
        ▼                 ▼                 ▼
  ┌───────────┐    ┌───────────┐    ┌───────────┐
  │  office:  │    │filesystem:│    │  github:  │
  │ (native)  │    │   (mcp)   │    │   (mcp)   │
  └───────────┘    └───────────┘    └───────────┘
```

## Tools Nativas (office:*)

O Office expõe 14+ tools nativas via MCP:

### UBL Tools
- `office:ubl_query` - Query the ledger
- `office:ubl_commit` - Commit atoms to ledger

### Entity Tools
- `office:entity_get` - Get entity info
- `office:entity_handover` - Read/write handover notes

### Job Tools
- `office:job_create` - Create new jobs
- `office:job_status` - Check job status

### Memory Tools
- `office:memory_recall` - Semantic memory search
- `office:memory_store` - Store memories

### Governance Tools
- `office:sanity_check` - Validate claims
- `office:permit_check` - Check UBL policy permissions
- `office:simulate` - Simulate before executing

### Communication Tools
- `office:message_send` - Send messages
- `office:message_history` - Get conversation history
- `office:escalate` - Escalate to guardian

## Tools Externas (MCP Servers)

Conecta a servidores MCP via stdio:

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

## MCP Ecosystem Guide

O Office inclui orientação sobre o ecossistema MCP que pode ser injetada no prompt do LLM.
Isso ajuda LLMs a descobrir e usar tools de forma segura.

### Obtendo o Guia

```rust
use office::mcp::mcp_ecosystem_guide;

let guide = mcp_ecosystem_guide();
// Adicionar ao prompt quando o LLM perguntar sobre MCPs disponíveis
```

### Servidores Oficiais (Alta Confiança)

| Server | Propósito | Trust Level |
|--------|-----------|-------------|
| `@modelcontextprotocol/server-filesystem` | Operações de arquivo | ⭐⭐⭐ Alto |
| `@modelcontextprotocol/server-github` | GitHub API | ⭐⭐⭐ Alto |
| `@modelcontextprotocol/server-brave-search` | Busca web | ⭐⭐⭐ Alto |
| `@modelcontextprotocol/server-sqlite` | Queries de banco | ⭐⭐⭐ Alto |
| `@modelcontextprotocol/server-puppeteer` | Automação de browser | ⭐⭐ Médio |
| `@modelcontextprotocol/server-memory` | Grafos de conhecimento | ⭐⭐⭐ Alto |
| `@modelcontextprotocol/server-postgres` | PostgreSQL | ⭐⭐⭐ Alto |
| `@modelcontextprotocol/server-slack` | Mensagens Slack | ⭐⭐⭐ Alto |

### Como Encontrar MCPs Seguros

1. **Registry Oficial**: https://github.com/modelcontextprotocol/servers
2. **GitHub Search**: `mcp-server` ou `modelcontextprotocol`
3. **npm Search**: prefixo `@modelcontextprotocol/` para oficiais

### Checklist de Segurança

- ✅ Open source com código legível
- ✅ Manutenção ativa (commits nos últimos 3 meses)
- ✅ Documentação clara
- ✅ Sem chamadas de rede suspeitas
- ✅ Permissões mínimas solicitadas
- ⚠️ Cuidado com servers pedindo acesso amplo ao filesystem
- ⚠️ Evite servers que pedem credenciais desnecessárias

### Solicitando Novos MCPs

Se o LLM precisa de uma capability não disponível:
1. Verificar se já existe (pesquisar primeiro!)
2. Pedir ao guardian para instalar um server confiável
3. Usar `office:escalate` para solicitar novas capabilities

## Roadmap

- [ ] Hot reload de servers sem restart
- [ ] Pool de servers para alta disponibilidade
- [ ] Métricas por tool no Prometheus
- [ ] Cache de resultados para tools idempotentes
- [ ] Streaming de resultados longos
