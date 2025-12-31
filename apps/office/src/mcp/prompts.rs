//! MCP Prompt Instructions
//!
//! Generates the tool usage narrative for LLMs.
//! This is injected into the Context Frame so the LLM understands
//! how to use tools and what's available.

use crate::mcp::UnifiedToolRegistry;

/// Generate the complete MCP orientation for an LLM
pub async fn generate_mcp_orientation(registry: &UnifiedToolRegistry) -> String {
    let tools_list = registry.format_for_llm().await;
    let tool_count = registry.tool_count().await;
    
    format!(r#"
## Tool System

You have access to **{tool_count} tools** via the Model Context Protocol (MCP).
All tools use the same interface - you don't need to know if they're native or external.

### How to Use Tools

When you need to perform an action, respond with a JSON block:

```json
{{
  "tool_calls": [
    {{
      "name": "server:tool_name",
      "arguments": {{ "param1": "value1" }}
    }}
  ]
}}
```

You can call multiple tools in parallel if they're independent.

### Tool Categories

**office:*** - Native Office tools (UBL, memory, jobs, governance)
- Always available, fastest execution
- Use for: ledger operations, entity management, job creation, memory

**filesystem:*** - File system operations
- Read/write local files
- Use for: code files, configs, logs, documents

**github:*** - GitHub API
- Repos, issues, PRs, code search
- Use for: version control, collaboration, code review

**Other MCP servers** may be available depending on configuration.

### Best Practices

1. **Prefer native tools** (`office:*`) for Office-specific operations
2. **Check before writing** - use `permit_check` for risky operations
3. **Simulate first** - use `office:simulate` for irreversible actions
4. **Store learnings** - use `office:memory_store` for important discoveries
5. **Escalate when uncertain** - use `office:escalate` if you're unsure

### Error Handling

If a tool fails:
1. Read the error message carefully
2. Try an alternative approach if available
3. Escalate if you can't resolve it

{tools_list}
"#, tool_count = tool_count, tools_list = tools_list)
}

/// Generate MCP ecosystem orientation (for curious LLMs or when asked)
pub fn mcp_ecosystem_guide() -> &'static str {
    r#"
## MCP Ecosystem Guide

Model Context Protocol (MCP) is an open protocol that lets you interact with external tools.
Think of MCP servers as specialized helpers that extend your capabilities.

### Official MCP Servers (Anthropic/Community Maintained)

These are well-maintained and widely used:

| Server | Purpose | Trust Level |
|--------|---------|-------------|
| `@modelcontextprotocol/server-filesystem` | File operations | ⭐⭐⭐ High |
| `@modelcontextprotocol/server-github` | GitHub API | ⭐⭐⭐ High |
| `@modelcontextprotocol/server-brave-search` | Web search | ⭐⭐⭐ High |
| `@modelcontextprotocol/server-sqlite` | Database queries | ⭐⭐⭐ High |
| `@modelcontextprotocol/server-puppeteer` | Browser automation | ⭐⭐ Medium |
| `@modelcontextprotocol/server-memory` | Knowledge graphs | ⭐⭐⭐ High |
| `@modelcontextprotocol/server-postgres` | PostgreSQL | ⭐⭐⭐ High |
| `@modelcontextprotocol/server-slack` | Slack messaging | ⭐⭐⭐ High |
| `@modelcontextprotocol/server-google-maps` | Maps/Places | ⭐⭐⭐ High |

### Popular Community Servers

| Server | Purpose | Where to Find |
|--------|---------|---------------|
| `mcp-server-docker` | Docker management | github.com/ckreiling/mcp-server-docker |
| `mcp-server-kubernetes` | K8s operations | github.com/... |
| `playwright-mcp` | E2E testing | github.com/executeautomation/playwright-mcp |
| `mcp-server-git` | Git operations | github.com/modelcontextprotocol/servers |
| `mcp-server-fetch` | HTTP requests | github.com/modelcontextprotocol/servers |

### How to Find Safe MCP Servers

1. **Official Registry**: https://github.com/modelcontextprotocol/servers
   - Contains all official + vetted community servers
   
2. **GitHub Search**: `mcp-server` or `modelcontextprotocol`
   - Look for: many stars, recent commits, good documentation
   
3. **npm Search**: `@modelcontextprotocol/` prefix for official ones

4. **Safety Checklist**:
   - ✅ Open source with readable code
   - ✅ Active maintenance (commits in last 3 months)
   - ✅ Clear documentation
   - ✅ No suspicious network calls
   - ✅ Minimal permissions requested
   - ⚠️ Be cautious with servers requesting broad filesystem access
   - ⚠️ Avoid servers that ask for unnecessary credentials

### MCP Server Categories

**Data Access**
- Databases: postgres, sqlite, mongodb
- Search: brave-search, exa
- Files: filesystem, google-drive

**Developer Tools**
- VCS: git, github, gitlab
- CI/CD: github-actions
- Containers: docker, kubernetes

**Communication**
- Chat: slack, discord
- Email: gmail (community)

**Automation**
- Browser: puppeteer, playwright
- HTTP: fetch

**Knowledge**
- Memory: memory (knowledge graphs)
- RAG: various embedding servers

### Requesting New MCP Servers

If you need a capability that's not available:
1. Check if it already exists (search first!)
2. Ask your guardian to install a trusted server
3. Use `office:escalate` to request new capabilities

### Security Notes

- MCP servers run as separate processes with their own permissions
- Filesystem servers should use `--allowed-paths` restrictions
- API tokens should have minimal scopes
- All tool calls are logged in the audit trail
- When in doubt, use `office:permit_check` before taking action
"#
}

/// Short orientation for simple tasks
pub async fn generate_minimal_orientation(registry: &UnifiedToolRegistry) -> String {
    let tool_count = registry.tool_count().await;
    
    format!(r#"
## Tools Available

You have {tool_count} tools. Use JSON to call them:

```json
{{"tool_calls": [{{"name": "server:tool", "arguments": {{}}}}]}}
```

Key tools:
- `office:job_create` - Create tasks
- `office:memory_recall` - Search memories  
- `office:escalate` - Ask for help
- `filesystem:read_file` - Read files (if available)
"#, tool_count = tool_count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ecosystem_guide() {
        let guide = mcp_ecosystem_guide();
        assert!(guide.contains("modelcontextprotocol"));
        assert!(guide.contains("Safety Checklist"));
        assert!(guide.contains("filesystem"));
    }

    #[tokio::test]
    async fn test_generate_orientation() {
        let registry = UnifiedToolRegistry::with_defaults();
        let orientation = generate_mcp_orientation(&registry).await;
        
        assert!(orientation.contains("Tool System"));
        assert!(orientation.contains("office:"));
        assert!(orientation.contains("Best Practices"));
    }
}
