//! MCP Tool Integration for LLM
//!
//! Provides tool execution capabilities for LLM agents using MCP.
//! This module bridges the gap between LLM tool calls and MCP execution.

use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{info, warn, debug};

use crate::{Result, OfficeError};
use crate::mcp::{McpRegistry, McpTool, ToolContent, CallToolResult};

/// Tool executor for LLM agents
pub struct ToolExecutor {
    /// MCP Registry for tool access
    registry: Arc<RwLock<McpRegistry>>,
    /// Maximum tool calls per request (prevents loops)
    max_calls_per_request: usize,
    /// Timeout for tool execution (seconds)
    tool_timeout_secs: u64,
}

impl ToolExecutor {
    pub fn new(registry: Arc<RwLock<McpRegistry>>) -> Self {
        Self {
            registry,
            max_calls_per_request: 20,
            tool_timeout_secs: 30,
        }
    }

    pub fn with_max_calls(mut self, max: usize) -> Self {
        self.max_calls_per_request = max;
        self
    }

    pub fn with_timeout(mut self, seconds: u64) -> Self {
        self.tool_timeout_secs = seconds;
        self
    }

    /// Get all available tools formatted for LLM prompt
    pub async fn tools_for_prompt(&self) -> String {
        let registry = self.registry.read().await;
        registry.format_tools_for_llm().await
    }

    /// Get available tools as JSON schema (for function calling)
    pub async fn tools_as_json_schema(&self) -> Vec<ToolSchema> {
        let registry = self.registry.read().await;
        let tools = registry.all_tools().await;
        
        tools.into_iter().map(|(server, tool)| {
            ToolSchema {
                name: format!("{}:{}", server, tool.name),
                description: tool.description.unwrap_or_default(),
                parameters: ToolParameters {
                    schema_type: "object".to_string(),
                    properties: tool.input_schema.properties.unwrap_or(json!({})),
                    required: tool.input_schema.required.unwrap_or_default(),
                },
            }
        }).collect()
    }

    /// Execute a single tool call
    pub async fn execute_tool(
        &self,
        session_id: &str,
        name: &str,
        arguments: Option<Value>,
    ) -> Result<ToolExecutionResult> {
        info!("Executing tool: {} for session {}", name, session_id);
        debug!("Tool arguments: {:?}", arguments);
        
        // Execute with timeout
        let result = tokio::time::timeout(
            std::time::Duration::from_secs(self.tool_timeout_secs),
            self.execute_mcp_tool(name, arguments),
        ).await;
        
        match result {
            Ok(Ok(mcp_result)) => {
                // Extract text content
                let output_text = mcp_result.content.iter()
                    .filter_map(|c| match c {
                        ToolContent::Text { text } => Some(text.clone()),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                
                info!("Tool {} completed successfully", name);
                
                Ok(ToolExecutionResult {
                    success: true,
                    output: Some(output_text),
                    error: None,
                    tool_name: name.to_string(),
                })
            }
            Ok(Err(e)) => {
                warn!("Tool execution failed: {} - {}", name, e);
                
                Ok(ToolExecutionResult {
                    success: false,
                    output: None,
                    error: Some(e.to_string()),
                    tool_name: name.to_string(),
                })
            }
            Err(_) => {
                warn!("Tool timed out: {}", name);
                
                Ok(ToolExecutionResult {
                    success: false,
                    output: None,
                    error: Some(format!("Tool timed out after {} seconds", self.tool_timeout_secs)),
                    tool_name: name.to_string(),
                })
            }
        }
    }

    /// Execute multiple tool calls
    pub async fn execute_tools(
        &self,
        session_id: &str,
        calls: Vec<LlmToolCall>,
    ) -> Result<Vec<ToolExecutionResult>> {
        if calls.len() > self.max_calls_per_request {
            return Err(OfficeError::McpError(format!(
                "Too many tool calls: {} (max: {})",
                calls.len(),
                self.max_calls_per_request
            )));
        }

        let mut results = Vec::with_capacity(calls.len());
        
        for call in calls {
            let result = self.execute_tool(session_id, &call.name, call.arguments).await?;
            results.push(result);
        }

        Ok(results)
    }

    /// Internal: Execute via MCP
    async fn execute_mcp_tool(&self, name: &str, arguments: Option<Value>) -> Result<CallToolResult> {
        let registry = self.registry.read().await;
        registry.call_tool(name, arguments).await
    }

    /// Check if a tool exists
    pub async fn has_tool(&self, name: &str) -> bool {
        let registry = self.registry.read().await;
        registry.has_tool(name).await
    }

    /// Get tool info
    pub async fn get_tool(&self, name: &str) -> Option<McpTool> {
        let registry = self.registry.read().await;
        registry.get_tool(name).await
    }
}

/// Tool call from LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmToolCall {
    pub name: String,
    pub arguments: Option<Value>,
}

/// Result of tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecutionResult {
    pub success: bool,
    pub output: Option<String>,
    pub error: Option<String>,
    pub tool_name: String,
}

/// Tool schema for LLM function calling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSchema {
    pub name: String,
    pub description: String,
    pub parameters: ToolParameters,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParameters {
    #[serde(rename = "type")]
    pub schema_type: String,
    pub properties: Value,
    pub required: Vec<String>,
}

/// Build tool use instructions for LLM prompt
pub fn build_tool_instructions(tools: &[ToolSchema]) -> String {
    if tools.is_empty() {
        return String::new();
    }

    let mut instructions = String::from(r#"
## Tool Usage

You have access to the following tools. To use a tool, respond with a JSON block:

```json
{
  "tool_calls": [
    {
      "name": "tool_name",
      "arguments": { "arg1": "value1" }
    }
  ]
}
```

### Available Tools

"#);

    for tool in tools {
        instructions.push_str(&format!("**{}**: {}\n", tool.name, tool.description));
        if let Some(props) = tool.parameters.properties.as_object() {
            instructions.push_str("  Parameters:\n");
            for (key, value) in props {
                let type_str = value.get("type")
                    .and_then(|t| t.as_str())
                    .unwrap_or("any");
                let desc = value.get("description")
                    .and_then(|d| d.as_str())
                    .unwrap_or("");
                let required = if tool.parameters.required.contains(key) { " (required)" } else { "" };
                instructions.push_str(&format!("  - `{}`{}: {} - {}\n", key, required, type_str, desc));
            }
        }
        instructions.push('\n');
    }

    instructions.push_str(r#"
### Guidelines

1. Only call tools when needed to complete the task
2. Wait for tool results before proceeding
3. Handle errors gracefully
4. You can call multiple tools in one response if they're independent

"#);

    instructions
}

/// Parse tool calls from LLM response
pub fn parse_tool_calls(response: &str) -> Vec<LlmToolCall> {
    // Try to find JSON block with tool_calls
    let json_start = response.find('{');
    let json_end = response.rfind('}');
    
    if let (Some(start), Some(end)) = (json_start, json_end) {
        if end > start {
            let json_str = &response[start..=end];
            if let Ok(parsed) = serde_json::from_str::<Value>(json_str) {
                if let Some(calls) = parsed.get("tool_calls").and_then(|v| v.as_array()) {
                    return calls.iter()
                        .filter_map(|c| {
                            let name = c.get("name")?.as_str()?.to_string();
                            let arguments = c.get("arguments").cloned();
                            Some(LlmToolCall { name, arguments })
                        })
                        .collect();
                }
            }
        }
    }
    
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tool_calls() {
        let response = r#"
I'll read the file for you.

```json
{
  "tool_calls": [
    {
      "name": "filesystem:read_file",
      "arguments": { "path": "/home/user/test.txt" }
    }
  ]
}
```
        "#;

        let calls = parse_tool_calls(response);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "filesystem:read_file");
    }

    #[test]
    fn test_parse_multiple_tool_calls() {
        let response = r#"
{
  "tool_calls": [
    { "name": "read_file", "arguments": { "path": "a.txt" } },
    { "name": "read_file", "arguments": { "path": "b.txt" } }
  ]
}
        "#;

        let calls = parse_tool_calls(response);
        assert_eq!(calls.len(), 2);
    }

    #[test]
    fn test_no_tool_calls() {
        let response = "Just a regular message without any tool calls.";
        let calls = parse_tool_calls(response);
        assert!(calls.is_empty());
    }
}
