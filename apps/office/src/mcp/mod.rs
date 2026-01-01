//! MCP - Model Context Protocol Implementation
//!
//! The Office exposes ALL tools via MCP interface:
//! - Native Office tools (office:*)
//! - External MCP servers (filesystem:*, github:*, etc.)
//!
//! The LLM only sees MCP - uniform interface for everything!
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
//!
//! Protocol: JSON-RPC 2.0 over stdio (for external servers)
//! Spec: https://modelcontextprotocol.io/

mod client;
mod config;
mod native_server;
mod prompts;
mod protocol;
mod registry;
mod tool_executor;
mod transport;
mod unified_registry;

// External MCP client
pub use client::McpClient;
pub use config::{McpConfig, McpServerDef};
pub use protocol::*;
pub use registry::{McpRegistry, McpServerConfig};
pub use transport::StdioTransport;

// Native Office MCP server
pub use native_server::{NativeTool, OfficeMcpServer, ToolContext, ToolHandler};

// Unified interface (the one LLM uses)
pub use unified_registry::{UnifiedTool, UnifiedToolRegistry};

// Prompt generation for LLM orientation
pub use prompts::{generate_mcp_orientation, generate_minimal_orientation, mcp_ecosystem_guide};

// Tool execution
pub use tool_executor::{
    build_tool_instructions, parse_tool_calls, LlmToolCall, ToolExecutionResult, ToolExecutor,
    ToolParameters, ToolSchema,
};
