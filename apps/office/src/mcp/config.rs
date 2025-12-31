//! MCP Configuration
//!
//! Configuration for Model Context Protocol servers

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// MCP Configuration from environment/config file
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct McpConfig {
    /// Whether MCP is enabled
    #[serde(default)]
    pub enabled: bool,
    
    /// List of MCP servers to connect to
    #[serde(default)]
    pub servers: Vec<McpServerDef>,
}

/// MCP Server definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerDef {
    /// Unique name for this server
    pub name: String,
    
    /// Command to spawn the server
    pub command: String,
    
    /// Arguments to pass to the command
    #[serde(default)]
    pub args: Vec<String>,
    
    /// Environment variables
    #[serde(default)]
    pub env: HashMap<String, String>,
    
    /// Auto-start when Office starts
    #[serde(default = "default_true")]
    pub auto_start: bool,
}

fn default_true() -> bool {
    true
}

impl McpConfig {
    /// Load from environment variable OFFICE__MCP__CONFIG (JSON)
    pub fn from_env() -> Self {
        if let Ok(json) = std::env::var("OFFICE__MCP__CONFIG") {
            if let Ok(config) = serde_json::from_str(&json) {
                return config;
            }
        }
        
        // Check individual env vars for simple setup
        let mut config = McpConfig::default();
        
        if std::env::var("OFFICE__MCP__ENABLED").unwrap_or_default() == "true" {
            config.enabled = true;
        }
        
        // Add filesystem server if path specified
        if let Ok(paths) = std::env::var("OFFICE__MCP__FILESYSTEM_PATHS") {
            config.enabled = true;
            let paths: Vec<String> = paths.split(',').map(|s| s.trim().to_string()).collect();
            config.servers.push(McpServerDef {
                name: "filesystem".to_string(),
                command: "npx".to_string(),
                args: std::iter::once("-y".to_string())
                    .chain(std::iter::once("@modelcontextprotocol/server-filesystem".to_string()))
                    .chain(paths)
                    .collect(),
                env: HashMap::new(),
                auto_start: true,
            });
        }
        
        // Add GitHub server if token specified
        if let Ok(token) = std::env::var("OFFICE__MCP__GITHUB_TOKEN") {
            config.enabled = true;
            let mut env = HashMap::new();
            env.insert("GITHUB_PERSONAL_ACCESS_TOKEN".to_string(), token);
            config.servers.push(McpServerDef {
                name: "github".to_string(),
                command: "npx".to_string(),
                args: vec!["-y".to_string(), "@modelcontextprotocol/server-github".to_string()],
                env,
                auto_start: true,
            });
        }
        
        // Add Brave Search if API key specified
        if let Ok(api_key) = std::env::var("OFFICE__MCP__BRAVE_API_KEY") {
            config.enabled = true;
            let mut env = HashMap::new();
            env.insert("BRAVE_API_KEY".to_string(), api_key);
            config.servers.push(McpServerDef {
                name: "brave-search".to_string(),
                command: "npx".to_string(),
                args: vec!["-y".to_string(), "@modelcontextprotocol/server-brave-search".to_string()],
                env,
                auto_start: true,
            });
        }
        
        config
    }
    
    /// Example configuration for documentation
    pub fn example() -> Self {
        Self {
            enabled: true,
            servers: vec![
                McpServerDef {
                    name: "filesystem".to_string(),
                    command: "npx".to_string(),
                    args: vec![
                        "-y".to_string(),
                        "@modelcontextprotocol/server-filesystem".to_string(),
                        "/home/user/project".to_string(),
                    ],
                    env: HashMap::new(),
                    auto_start: true,
                },
                McpServerDef {
                    name: "github".to_string(),
                    command: "npx".to_string(),
                    args: vec!["-y".to_string(), "@modelcontextprotocol/server-github".to_string()],
                    env: {
                        let mut env = HashMap::new();
                        env.insert("GITHUB_PERSONAL_ACCESS_TOKEN".to_string(), "ghp_xxx".to_string());
                        env
                    },
                    auto_start: true,
                },
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = McpConfig::default();
        assert!(!config.enabled);
        assert!(config.servers.is_empty());
    }
    
    #[test]
    fn test_serialize_deserialize() {
        let config = McpConfig::example();
        let json = serde_json::to_string_pretty(&config).unwrap();
        let parsed: McpConfig = serde_json::from_str(&json).unwrap();
        assert!(parsed.enabled);
        assert_eq!(parsed.servers.len(), 2);
    }
}
