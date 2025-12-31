//! MCP Client
//!
//! High-level client for interacting with MCP servers

use serde_json::json;
use tracing::{debug, info};

use crate::{OfficeError, Result};
use super::protocol::*;
use super::transport::StdioTransport;

/// MCP Client for a single server
pub struct McpClient {
    /// Transport layer
    transport: StdioTransport,
    /// Server capabilities after initialization
    capabilities: Option<ServerCapabilities>,
    /// Server info after initialization
    server_info: Option<ServerInfo>,
    /// Cached tools
    tools: Vec<McpTool>,
    /// Cached resources
    resources: Vec<McpResource>,
    /// Cached prompts
    prompts: Vec<McpPrompt>,
}

impl McpClient {
    /// Create a new MCP client by spawning a server
    pub async fn spawn(command: &str, args: &[&str], server_name: &str) -> Result<Self> {
        let transport = StdioTransport::spawn(command, args, server_name).await?;
        
        Ok(Self {
            transport,
            capabilities: None,
            server_info: None,
            tools: Vec::new(),
            resources: Vec::new(),
            prompts: Vec::new(),
        })
    }

    /// Initialize the MCP connection
    pub async fn initialize(&mut self) -> Result<InitializeResult> {
        info!("Initializing MCP connection to {}", self.transport.server_name());

        let params = InitializeParams {
            protocol_version: MCP_PROTOCOL_VERSION.to_string(),
            capabilities: ClientCapabilities {
                roots: Some(RootsCapability { list_changed: true }),
                sampling: None,
            },
            client_info: ClientInfo {
                name: "office".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
        };

        let response = self.transport.request(
            "initialize",
            Some(serde_json::to_value(&params).unwrap()),
        ).await?;

        let result: InitializeResult = serde_json::from_value(response.result.unwrap_or_default())
            .map_err(|e| OfficeError::McpError(format!("Failed to parse initialize result: {}", e)))?;

        info!(
            "MCP server {} v{} initialized (protocol: {})",
            result.server_info.name,
            result.server_info.version.as_deref().unwrap_or("unknown"),
            result.protocol_version,
        );

        self.capabilities = Some(result.capabilities.clone());
        self.server_info = Some(result.server_info.clone());

        // Send initialized notification
        self.transport.notify("notifications/initialized", None).await?;

        // Auto-discover capabilities
        if result.capabilities.tools.is_some() {
            self.discover_tools().await?;
        }
        if result.capabilities.resources.is_some() {
            self.discover_resources().await?;
        }
        if result.capabilities.prompts.is_some() {
            self.discover_prompts().await?;
        }

        Ok(result)
    }

    /// Discover available tools
    pub async fn discover_tools(&mut self) -> Result<&[McpTool]> {
        debug!("Discovering tools from {}", self.transport.server_name());

        let response = self.transport.request("tools/list", None).await?;

        let result: ListToolsResult = serde_json::from_value(response.result.unwrap_or_default())
            .map_err(|e| OfficeError::McpError(format!("Failed to parse tools list: {}", e)))?;

        info!("Discovered {} tools from {}", result.tools.len(), self.transport.server_name());
        for tool in &result.tools {
            debug!("  - {}: {}", tool.name, tool.description.as_deref().unwrap_or(""));
        }

        self.tools = result.tools;
        Ok(&self.tools)
    }

    /// Discover available resources
    pub async fn discover_resources(&mut self) -> Result<&[McpResource]> {
        debug!("Discovering resources from {}", self.transport.server_name());

        let response = self.transport.request("resources/list", None).await?;

        let result: ListResourcesResult = serde_json::from_value(response.result.unwrap_or_default())
            .map_err(|e| OfficeError::McpError(format!("Failed to parse resources list: {}", e)))?;

        info!("Discovered {} resources from {}", result.resources.len(), self.transport.server_name());

        self.resources = result.resources;
        Ok(&self.resources)
    }

    /// Discover available prompts
    pub async fn discover_prompts(&mut self) -> Result<&[McpPrompt]> {
        debug!("Discovering prompts from {}", self.transport.server_name());

        let response = self.transport.request("prompts/list", None).await?;

        let result: ListPromptsResult = serde_json::from_value(response.result.unwrap_or_default())
            .map_err(|e| OfficeError::McpError(format!("Failed to parse prompts list: {}", e)))?;

        info!("Discovered {} prompts from {}", result.prompts.len(), self.transport.server_name());

        self.prompts = result.prompts;
        Ok(&self.prompts)
    }

    /// Call a tool
    pub async fn call_tool(&self, name: &str, arguments: Option<serde_json::Value>) -> Result<CallToolResult> {
        info!("Calling tool: {} on {}", name, self.transport.server_name());

        let params = CallToolParams {
            name: name.to_string(),
            arguments,
        };

        let response = self.transport.request(
            "tools/call",
            Some(serde_json::to_value(&params).unwrap()),
        ).await?;

        let result: CallToolResult = serde_json::from_value(response.result.unwrap_or_default())
            .map_err(|e| OfficeError::McpError(format!("Failed to parse tool result: {}", e)))?;

        if result.is_error.unwrap_or(false) {
            let error_text = result.content.iter()
                .filter_map(|c| match c {
                    ToolContent::Text { text } => Some(text.as_str()),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join("\n");
            return Err(OfficeError::McpError(format!("Tool error: {}", error_text)));
        }

        Ok(result)
    }

    /// Read a resource
    pub async fn read_resource(&self, uri: &str) -> Result<ReadResourceResult> {
        debug!("Reading resource: {}", uri);

        let response = self.transport.request(
            "resources/read",
            Some(json!({ "uri": uri })),
        ).await?;

        let result: ReadResourceResult = serde_json::from_value(response.result.unwrap_or_default())
            .map_err(|e| OfficeError::McpError(format!("Failed to parse resource: {}", e)))?;

        Ok(result)
    }

    /// Get cached tools
    pub fn tools(&self) -> &[McpTool] {
        &self.tools
    }

    /// Get cached resources
    pub fn resources(&self) -> &[McpResource] {
        &self.resources
    }

    /// Get cached prompts
    pub fn prompts(&self) -> &[McpPrompt] {
        &self.prompts
    }

    /// Get server name
    pub fn server_name(&self) -> &str {
        self.transport.server_name()
    }

    /// Get server info
    pub fn server_info(&self) -> Option<&ServerInfo> {
        self.server_info.as_ref()
    }

    /// Get server capabilities
    pub fn capabilities(&self) -> Option<&ServerCapabilities> {
        self.capabilities.as_ref()
    }

    /// Check if tool exists
    pub fn has_tool(&self, name: &str) -> bool {
        self.tools.iter().any(|t| t.name == name)
    }

    /// Get tool by name
    pub fn get_tool(&self, name: &str) -> Option<&McpTool> {
        self.tools.iter().find(|t| t.name == name)
    }

    /// Shutdown the client
    pub async fn shutdown(mut self) -> Result<()> {
        info!("Shutting down MCP client: {}", self.transport.server_name());
        self.transport.kill().await?;
        Ok(())
    }
}
