//! MCP Registry
//!
//! Manages multiple MCP servers and provides unified tool access

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error};

use crate::{OfficeError, Result};
use super::client::McpClient;
use super::protocol::{McpTool, McpResource, CallToolResult};

/// Configuration for an MCP server
#[derive(Debug, Clone)]
pub struct McpServerConfig {
    /// Server name (unique identifier)
    pub name: String,
    /// Command to run
    pub command: String,
    /// Arguments to pass
    pub args: Vec<String>,
    /// Environment variables
    pub env: HashMap<String, String>,
    /// Auto-start on registry creation
    pub auto_start: bool,
}

impl McpServerConfig {
    pub fn new(name: &str, command: &str) -> Self {
        Self {
            name: name.to_string(),
            command: command.to_string(),
            args: Vec::new(),
            env: HashMap::new(),
            auto_start: true,
        }
    }

    pub fn with_args(mut self, args: &[&str]) -> Self {
        self.args = args.iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn with_env(mut self, key: &str, value: &str) -> Self {
        self.env.insert(key.to_string(), value.to_string());
        self
    }

    pub fn auto_start(mut self, enabled: bool) -> Self {
        self.auto_start = enabled;
        self
    }
}

/// Registry of MCP servers
pub struct McpRegistry {
    /// Connected clients by server name
    clients: Arc<RwLock<HashMap<String, McpClient>>>,
    /// Server configurations
    configs: HashMap<String, McpServerConfig>,
    /// Tool to server mapping (tool_name -> server_name)
    tool_map: Arc<RwLock<HashMap<String, String>>>,
}

impl McpRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            configs: HashMap::new(),
            tool_map: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create with predefined server configs
    pub fn with_configs(configs: Vec<McpServerConfig>) -> Self {
        let config_map: HashMap<String, McpServerConfig> = configs
            .into_iter()
            .map(|c| (c.name.clone(), c))
            .collect();

        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            configs: config_map,
            tool_map: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add a server configuration
    pub fn add_config(&mut self, config: McpServerConfig) {
        self.configs.insert(config.name.clone(), config);
    }

    /// Start all auto-start servers
    pub async fn start_all(&self) -> Result<()> {
        let auto_start: Vec<_> = self.configs.iter()
            .filter(|(_, c)| c.auto_start)
            .map(|(name, _)| name.clone())
            .collect();

        for name in auto_start {
            if let Err(e) = self.start_server(&name).await {
                error!("Failed to start MCP server {}: {}", name, e);
            }
        }

        Ok(())
    }

    /// Start a specific server
    pub async fn start_server(&self, name: &str) -> Result<()> {
        let config = self.configs.get(name)
            .ok_or_else(|| OfficeError::McpError(format!("Unknown server: {}", name)))?;

        info!("Starting MCP server: {}", name);

        let args: Vec<&str> = config.args.iter().map(|s| s.as_str()).collect();
        let mut client = McpClient::spawn(&config.command, &args, name).await?;
        
        // Initialize connection
        client.initialize().await?;

        // Update tool map
        {
            let mut tool_map = self.tool_map.write().await;
            for tool in client.tools() {
                let full_name = format!("{}:{}", name, tool.name);
                tool_map.insert(full_name, name.to_string());
                // Also map short name if unique
                if !tool_map.contains_key(&tool.name) {
                    tool_map.insert(tool.name.clone(), name.to_string());
                }
            }
        }

        // Store client
        {
            let mut clients = self.clients.write().await;
            clients.insert(name.to_string(), client);
        }

        info!("MCP server {} started successfully", name);
        Ok(())
    }

    /// Stop a specific server
    pub async fn stop_server(&self, name: &str) -> Result<()> {
        let client = {
            let mut clients = self.clients.write().await;
            clients.remove(name)
        };

        if let Some(client) = client {
            // Remove from tool map
            {
                let mut tool_map = self.tool_map.write().await;
                tool_map.retain(|_, server| server != name);
            }

            client.shutdown().await?;
            info!("MCP server {} stopped", name);
        }

        Ok(())
    }

    /// Stop all servers
    pub async fn stop_all(&self) -> Result<()> {
        let names: Vec<String> = {
            let clients = self.clients.read().await;
            clients.keys().cloned().collect()
        };

        for name in names {
            if let Err(e) = self.stop_server(&name).await {
                warn!("Error stopping server {}: {}", name, e);
            }
        }

        Ok(())
    }

    /// Get all available tools across all servers
    pub async fn all_tools(&self) -> Vec<(String, McpTool)> {
        let clients = self.clients.read().await;
        let mut tools = Vec::new();

        for (server_name, client) in clients.iter() {
            for tool in client.tools() {
                tools.push((server_name.clone(), tool.clone()));
            }
        }

        tools
    }

    /// Get all available resources across all servers
    pub async fn all_resources(&self) -> Vec<(String, McpResource)> {
        let clients = self.clients.read().await;
        let mut resources = Vec::new();

        for (server_name, client) in clients.iter() {
            for resource in client.resources() {
                resources.push((server_name.clone(), resource.clone()));
            }
        }

        resources
    }

    /// Call a tool by name
    /// 
    /// Tool name can be:
    /// - Full name: "server_name:tool_name"
    /// - Short name: "tool_name" (if unique across servers)
    pub async fn call_tool(&self, name: &str, arguments: Option<serde_json::Value>) -> Result<CallToolResult> {
        let (server_name, tool_name) = self.resolve_tool_name(name).await?;

        let clients = self.clients.read().await;
        let client = clients.get(&server_name)
            .ok_or_else(|| OfficeError::McpError(format!("Server not connected: {}", server_name)))?;

        client.call_tool(&tool_name, arguments).await
    }

    /// Resolve tool name to (server_name, tool_name)
    async fn resolve_tool_name(&self, name: &str) -> Result<(String, String)> {
        // Check for full name format "server:tool"
        if let Some((server, tool)) = name.split_once(':') {
            return Ok((server.to_string(), tool.to_string()));
        }

        // Look up in tool map
        let tool_map = self.tool_map.read().await;
        if let Some(server) = tool_map.get(name) {
            return Ok((server.clone(), name.to_string()));
        }

        Err(OfficeError::McpError(format!("Unknown tool: {}", name)))
    }

    /// Check if a tool exists
    pub async fn has_tool(&self, name: &str) -> bool {
        self.resolve_tool_name(name).await.is_ok()
    }

    /// Get tool definition
    pub async fn get_tool(&self, name: &str) -> Option<McpTool> {
        let (server_name, tool_name) = self.resolve_tool_name(name).await.ok()?;
        
        let clients = self.clients.read().await;
        let client = clients.get(&server_name)?;
        client.get_tool(&tool_name).cloned()
    }

    /// Get list of connected server names
    pub async fn connected_servers(&self) -> Vec<String> {
        let clients = self.clients.read().await;
        clients.keys().cloned().collect()
    }

    /// Get tool count
    pub async fn tool_count(&self) -> usize {
        let clients = self.clients.read().await;
        clients.values().map(|c| c.tools().len()).sum()
    }

    /// Format tools for LLM prompt
    pub async fn format_tools_for_llm(&self) -> String {
        let tools = self.all_tools().await;
        
        if tools.is_empty() {
            return "No tools available.".to_string();
        }

        let mut output = String::from("## Available Tools\n\n");
        
        let mut current_server = String::new();
        for (server, tool) in &tools {
            if server != &current_server {
                output.push_str(&format!("### {} Server\n\n", server));
                current_server = server.clone();
            }

            output.push_str(&format!("**{}**", tool.name));
            if let Some(desc) = &tool.description {
                output.push_str(&format!(": {}", desc));
            }
            output.push('\n');

            // Add parameter info
            if let Some(props) = &tool.input_schema.properties {
                output.push_str("  Parameters:\n");
                if let Some(obj) = props.as_object() {
                    for (key, value) in obj {
                        let type_str = value.get("type")
                            .and_then(|t| t.as_str())
                            .unwrap_or("any");
                        let desc = value.get("description")
                            .and_then(|d| d.as_str())
                            .unwrap_or("");
                        output.push_str(&format!("  - `{}` ({}): {}\n", key, type_str, desc));
                    }
                }
            }
            output.push('\n');
        }

        output
    }
}

impl Default for McpRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Common MCP server configurations
impl McpRegistry {
    /// Add filesystem server (Node.js based)
    pub fn add_filesystem_server(&mut self, allowed_paths: &[&str]) {
        let args: Vec<String> = std::iter::once("@modelcontextprotocol/server-filesystem".to_string())
            .chain(allowed_paths.iter().map(|p| p.to_string()))
            .collect();

        self.add_config(McpServerConfig {
            name: "filesystem".to_string(),
            command: "npx".to_string(),
            args,
            env: HashMap::new(),
            auto_start: true,
        });
    }

    /// Add GitHub server
    pub fn add_github_server(&mut self, token: &str) {
        let mut config = McpServerConfig::new("github", "npx")
            .with_args(&["@modelcontextprotocol/server-github"]);
        config.env.insert("GITHUB_PERSONAL_ACCESS_TOKEN".to_string(), token.to_string());
        self.add_config(config);
    }

    /// Add SQLite server
    pub fn add_sqlite_server(&mut self, db_path: &str) {
        self.add_config(
            McpServerConfig::new("sqlite", "npx")
                .with_args(&["@modelcontextprotocol/server-sqlite", db_path])
        );
    }

    /// Add Brave Search server
    pub fn add_brave_search_server(&mut self, api_key: &str) {
        let mut config = McpServerConfig::new("brave-search", "npx")
            .with_args(&["@modelcontextprotocol/server-brave-search"]);
        config.env.insert("BRAVE_API_KEY".to_string(), api_key.to_string());
        self.add_config(config);
    }

    /// Add Puppeteer server (for web automation)
    pub fn add_puppeteer_server(&mut self) {
        self.add_config(
            McpServerConfig::new("puppeteer", "npx")
                .with_args(&["@modelcontextprotocol/server-puppeteer"])
        );
    }

    /// Add custom server
    pub fn add_custom_server(&mut self, name: &str, command: &str, args: &[&str]) {
        self.add_config(
            McpServerConfig::new(name, command)
                .with_args(args)
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_config() {
        let config = McpServerConfig::new("test", "echo")
            .with_args(&["hello", "world"])
            .with_env("FOO", "bar")
            .auto_start(false);

        assert_eq!(config.name, "test");
        assert_eq!(config.command, "echo");
        assert_eq!(config.args, vec!["hello", "world"]);
        assert_eq!(config.env.get("FOO"), Some(&"bar".to_string()));
        assert!(!config.auto_start);
    }

    #[tokio::test]
    async fn test_registry_creation() {
        let registry = McpRegistry::new();
        assert_eq!(registry.tool_count().await, 0);
        assert!(registry.connected_servers().await.is_empty());
    }
}
