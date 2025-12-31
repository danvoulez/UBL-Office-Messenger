//! MCP Client - Model Context Protocol Implementation
//!
//! Connects to MCP servers via stdio and exposes their tools to the Office.
//! 
//! Protocol: JSON-RPC 2.0 over stdio
//! Spec: https://modelcontextprotocol.io/

mod client;
mod config;
mod protocol;
mod transport;
mod registry;
mod tool_executor;

pub use client::McpClient;
pub use config::{McpConfig, McpServerDef};
pub use protocol::*;
pub use transport::StdioTransport;
pub use registry::{McpRegistry, McpServerConfig};
pub use tool_executor::{
    ToolExecutor, LlmToolCall, ToolExecutionResult, ToolSchema,
    build_tool_instructions, parse_tool_calls,
};
