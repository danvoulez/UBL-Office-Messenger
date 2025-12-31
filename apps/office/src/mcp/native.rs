//! Native Tool Provider
//!
//! Implements tools natively in Rust that conform to the MCP interface.
//! This allows mixing native tools with external MCP servers seamlessly.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;

use crate::{Result, OfficeError};
use crate::mcp::protocol::{McpTool, ToolInputSchema, ToolContent, CallToolResult};

/// Trait for tool providers (native or MCP)
#[async_trait]
pub trait ToolProvider: Send + Sync {
    /// Provider name (e.g., "native", "filesystem", "github")
    fn name(&self) -> &str;
    
    /// List available tools
    async fn list_tools(&self) -> Result<Vec<McpTool>>;
    
    /// Call a tool by name
    async fn call_tool(&self, name: &str, arguments: Option<Value>) -> Result<CallToolResult>;
    
    /// Check if provider has a specific tool
    async fn has_tool(&self, name: &str) -> bool {
        self.list_tools().await
            .map(|tools| tools.iter().any(|t| t.name == name))
            .unwrap_or(false)
    }
}

/// Native tool definition
pub struct NativeTool {
    pub name: String,
    pub description: String,
    pub input_schema: ToolInputSchema,
    pub handler: Box<dyn ToolHandler>,
}

/// Handler for executing a native tool
#[async_trait]
pub trait ToolHandler: Send + Sync {
    async fn execute(&self, arguments: Option<Value>) -> Result<CallToolResult>;
}

/// Native tools provider - implements tools in pure Rust
pub struct NativeToolProvider {
    name: String,
    tools: HashMap<String, NativeTool>,
}

impl NativeToolProvider {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            tools: HashMap::new(),
        }
    }

    /// Register a native tool
    pub fn register<H: ToolHandler + 'static>(
        &mut self,
        name: &str,
        description: &str,
        schema: ToolInputSchema,
        handler: H,
    ) {
        self.tools.insert(name.to_string(), NativeTool {
            name: name.to_string(),
            description: description.to_string(),
            input_schema: schema,
            handler: Box::new(handler),
        });
    }

    /// Register with a closure
    pub fn register_fn<F, Fut>(
        &mut self,
        name: &str,
        description: &str,
        schema: ToolInputSchema,
        handler: F,
    ) where
        F: Fn(Option<Value>) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<CallToolResult>> + Send + 'static,
    {
        self.tools.insert(name.to_string(), NativeTool {
            name: name.to_string(),
            description: description.to_string(),
            input_schema: schema,
            handler: Box::new(FnHandler { handler: Arc::new(handler) }),
        });
    }
}

/// Wrapper to make closures into ToolHandler
struct FnHandler<F> {
    handler: Arc<F>,
}

#[async_trait]
impl<F, Fut> ToolHandler for FnHandler<F>
where
    F: Fn(Option<Value>) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = Result<CallToolResult>> + Send + 'static,
{
    async fn execute(&self, arguments: Option<Value>) -> Result<CallToolResult> {
        (self.handler)(arguments).await
    }
}

#[async_trait]
impl ToolProvider for NativeToolProvider {
    fn name(&self) -> &str {
        &self.name
    }

    async fn list_tools(&self) -> Result<Vec<McpTool>> {
        Ok(self.tools.values().map(|t| McpTool {
            name: t.name.clone(),
            description: Some(t.description.clone()),
            input_schema: t.input_schema.clone(),
        }).collect())
    }

    async fn call_tool(&self, name: &str, arguments: Option<Value>) -> Result<CallToolResult> {
        let tool = self.tools.get(name)
            .ok_or_else(|| OfficeError::McpError(format!("Tool not found: {}", name)))?;
        
        tool.handler.execute(arguments).await
    }
}

// ============ Helper Functions for Creating Tools ============

/// Create a simple text result
pub fn text_result(text: &str) -> CallToolResult {
    CallToolResult {
        content: vec![ToolContent::Text { text: text.to_string() }],
        is_error: Some(false),
    }
}

/// Create an error result
pub fn error_result(error: &str) -> CallToolResult {
    CallToolResult {
        content: vec![ToolContent::Text { text: error.to_string() }],
        is_error: Some(true),
    }
}

/// Create a simple input schema with string properties
pub fn simple_schema(properties: &[(&str, &str, bool)]) -> ToolInputSchema {
    let mut props = serde_json::Map::new();
    let mut required = Vec::new();

    for (name, description, is_required) in properties {
        props.insert(name.to_string(), json!({
            "type": "string",
            "description": description
        }));
        if *is_required {
            required.push(name.to_string());
        }
    }

    ToolInputSchema {
        schema_type: "object".to_string(),
        properties: Some(Value::Object(props)),
        required: Some(required),
        extra: HashMap::new(),
    }
}

/// Create schema from JSON
pub fn json_schema(schema: Value) -> ToolInputSchema {
    ToolInputSchema {
        schema_type: schema.get("type").and_then(|t| t.as_str()).unwrap_or("object").to_string(),
        properties: schema.get("properties").cloned(),
        required: schema.get("required").and_then(|r| {
            r.as_array().map(|arr| {
                arr.iter().filter_map(|v| v.as_str().map(String::from)).collect()
            })
        }),
        extra: HashMap::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_native_provider() {
        let mut provider = NativeToolProvider::new("test");
        
        // Register a simple tool
        provider.register_fn(
            "echo",
            "Echoes the input",
            simple_schema(&[("message", "Message to echo", true)]),
            |args| async move {
                let msg = args
                    .and_then(|a| a.get("message").and_then(|m| m.as_str().map(String::from)))
                    .unwrap_or_default();
                Ok(text_result(&msg))
            },
        );

        let tools = provider.list_tools().await.unwrap();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, "echo");

        let result = provider.call_tool("echo", Some(json!({"message": "hello"}))).await.unwrap();
        assert!(!result.is_error.unwrap_or(true));
    }
}
