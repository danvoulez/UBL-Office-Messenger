//! Unified Tool Registry
//!
//! Combines native Office tools + external MCP servers into one interface.
//! The LLM only sees "tools" - doesn't know or care if they're native or MCP.
//!
//! ```
//!                           LLM
//!                            │
//!                            ▼
//!                 ┌────────────────────┐
//!                 │ UnifiedToolRegistry│
//!                 └─────────┬──────────┘
//!                           │
//!         ┌─────────────────┼─────────────────┐
//!         ▼                 ▼                 ▼
//!   ┌───────────┐    ┌───────────┐    ┌───────────┐
//!   │  office:  │    │filesystem:│    │  github:  │
//!   │ (native)  │    │   (mcp)   │    │   (mcp)   │
//!   └───────────┘    └───────────┘    └───────────┘
//! ```

use std::sync::Arc;
use tokio::sync::RwLock;
use serde_json::Value;
use tracing::{info, debug};

use crate::{Result, OfficeError};
use crate::mcp::protocol::*;
use crate::mcp::registry::McpRegistry;
use crate::mcp::native_server::OfficeMcpServer;

/// Unified registry that combines native tools + external MCP
pub struct UnifiedToolRegistry {
    /// Native Office tools (in-process)
    native: Arc<OfficeMcpServer>,
    /// External MCP servers
    external: Arc<RwLock<McpRegistry>>,
}

impl UnifiedToolRegistry {
    /// Create new unified registry
    pub fn new(native: Arc<OfficeMcpServer>, external: Arc<RwLock<McpRegistry>>) -> Self {
        Self { native, external }
    }

    /// Create with default native server
    pub fn with_defaults() -> Self {
        Self {
            native: Arc::new(OfficeMcpServer::new()),
            external: Arc::new(RwLock::new(McpRegistry::new())),
        }
    }

    /// Get the native server (for registration)
    pub fn native(&self) -> &Arc<OfficeMcpServer> {
        &self.native
    }

    /// Get the external registry (for adding MCP servers)
    pub fn external(&self) -> &Arc<RwLock<McpRegistry>> {
        &self.external
    }

    /// List ALL tools (native + external)
    pub async fn all_tools(&self) -> Vec<UnifiedTool> {
        let mut tools = Vec::new();

        // Add native tools with "office:" prefix
        for tool in self.native.list_tools() {
            tools.push(UnifiedTool {
                full_name: format!("office:{}", tool.name),
                short_name: tool.name.clone(),
                server: "office".to_string(),
                is_native: true,
                tool,
            });
        }

        // Add external MCP tools
        let external = self.external.read().await;
        for (server, tool) in external.all_tools().await {
            tools.push(UnifiedTool {
                full_name: format!("{}:{}", server, tool.name),
                short_name: tool.name.clone(),
                server,
                is_native: false,
                tool,
            });
        }

        tools
    }

    /// Call a tool by name
    /// 
    /// Supports:
    /// - Full name: "office:ubl_query", "filesystem:read_file"
    /// - Short name: "ubl_query" (prefers native), "read_file" (external)
    pub async fn call_tool(&self, name: &str, arguments: Value) -> Result<CallToolResult> {
        let (server, tool_name) = self.resolve_tool_name(name).await?;

        debug!("Calling tool {} on server {}", tool_name, server);

        if server == "office" {
            // Native tool
            self.native.call_tool(&tool_name, arguments).await
        } else {
            // External MCP tool
            let external = self.external.read().await;
            external.call_tool(&format!("{}:{}", server, tool_name), Some(arguments)).await
        }
    }

    /// Resolve tool name to (server, tool_name)
    async fn resolve_tool_name(&self, name: &str) -> Result<(String, String)> {
        // Check for full name format "server:tool"
        if let Some((server, tool)) = name.split_once(':') {
            return Ok((server.to_string(), tool.to_string()));
        }

        // Try native first
        if self.native.has_tool(name) {
            return Ok(("office".to_string(), name.to_string()));
        }

        // Try external
        let external = self.external.read().await;
        if external.has_tool(name).await {
            // Need to find which server has it
            for (server, tool) in external.all_tools().await {
                if tool.name == name {
                    return Ok((server, name.to_string()));
                }
            }
        }

        Err(OfficeError::McpError(format!("Unknown tool: {}", name)))
    }

    /// Check if tool exists
    pub async fn has_tool(&self, name: &str) -> bool {
        self.resolve_tool_name(name).await.is_ok()
    }

    /// Get tool info
    pub async fn get_tool(&self, name: &str) -> Option<UnifiedTool> {
        let (server, tool_name) = self.resolve_tool_name(name).await.ok()?;

        if server == "office" {
            let tool = self.native.get_tool(&tool_name)?;
            Some(UnifiedTool {
                full_name: format!("office:{}", tool.name),
                short_name: tool.name.clone(),
                server: "office".to_string(),
                is_native: true,
                tool,
            })
        } else {
            let external = self.external.read().await;
            let tool = external.get_tool(&format!("{}:{}", server, tool_name)).await?;
            Some(UnifiedTool {
                full_name: format!("{}:{}", server, tool.name),
                short_name: tool.name.clone(),
                server,
                is_native: false,
                tool,
            })
        }
    }

    /// Get tool count
    pub async fn tool_count(&self) -> usize {
        let native_count = self.native.list_tools().len();
        let external = self.external.read().await;
        let external_count = external.tool_count().await;
        native_count + external_count
    }

    /// Format all tools for LLM prompt
    pub async fn format_for_llm(&self) -> String {
        let tools = self.all_tools().await;

        if tools.is_empty() {
            return "No tools available.".to_string();
        }

        let mut output = String::from("## Available Tools\n\n");
        
        // Group by server
        let mut current_server = String::new();
        let mut sorted_tools = tools;
        sorted_tools.sort_by(|a, b| a.server.cmp(&b.server));

        for ut in &sorted_tools {
            if ut.server != current_server {
                let server_type = if ut.is_native { "(native)" } else { "(mcp)" };
                output.push_str(&format!("### {} {}\n\n", ut.server, server_type));
                current_server = ut.server.clone();
            }

            output.push_str(&format!("**{}**", ut.full_name));
            if let Some(desc) = &ut.tool.description {
                output.push_str(&format!(": {}", desc));
            }
            output.push('\n');

            // Add parameter info
            if let Some(props) = &ut.tool.input_schema.properties {
                if let Some(obj) = props.as_object() {
                    for (key, value) in obj {
                        let type_str = value.get("type")
                            .and_then(|t| t.as_str())
                            .unwrap_or("any");
                        let desc = value.get("description")
                            .and_then(|d| d.as_str())
                            .unwrap_or("");
                        let required = ut.tool.input_schema.required
                            .as_ref()
                            .map(|r| r.contains(key))
                            .unwrap_or(false);
                        let req_str = if required { " (required)" } else { "" };
                        output.push_str(&format!("  - `{}`{}: {} - {}\n", key, req_str, type_str, desc));
                    }
                }
            }
            output.push('\n');
        }

        output
    }

    /// Get tool calling instructions for LLM
    pub fn tool_calling_instructions() -> &'static str {
        r#"
## Tool Usage

You have access to tools. To use a tool, respond with a JSON block:

```json
{
  "tool_calls": [
    {
      "name": "server:tool_name",
      "arguments": { "arg1": "value1" }
    }
  ]
}
```

### Guidelines

1. Use full tool names (server:tool) for clarity
2. You can call multiple tools in one response if independent
3. Wait for tool results before proceeding
4. If a tool fails, try an alternative approach
5. For file operations, prefer office: tools for UBL data, filesystem: for local files

"#
    }
}

/// Unified tool info
#[derive(Debug, Clone)]
pub struct UnifiedTool {
    /// Full name with server prefix (e.g., "office:ubl_query")
    pub full_name: String,
    /// Short name without prefix (e.g., "ubl_query")
    pub short_name: String,
    /// Server name
    pub server: String,
    /// Is this a native Office tool?
    pub is_native: bool,
    /// The MCP tool definition
    pub tool: McpTool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_unified_registry() {
        let registry = UnifiedToolRegistry::with_defaults();
        
        // Should have native tools
        let count = registry.tool_count().await;
        assert!(count >= 14); // At least the native tools
        
        // Should find native tools
        assert!(registry.has_tool("ubl_query").await);
        assert!(registry.has_tool("office:ubl_query").await);
    }

    #[tokio::test]
    async fn test_call_native_tool() {
        let registry = UnifiedToolRegistry::with_defaults();
        
        let result = registry.call_tool("job_create", serde_json::json!({
            "title": "Test Job"
        })).await.unwrap();

        assert!(!result.is_error.unwrap_or(true));
    }

    #[tokio::test]
    async fn test_format_for_llm() {
        let registry = UnifiedToolRegistry::with_defaults();
        let prompt = registry.format_for_llm().await;
        
        assert!(prompt.contains("office"));
        assert!(prompt.contains("ubl_query"));
        assert!(prompt.contains("job_create"));
    }
}
