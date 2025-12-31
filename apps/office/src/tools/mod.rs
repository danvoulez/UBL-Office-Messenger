//! Tools Module - Development tools for LLM agents
//!
//! Provides file system, git, terminal, and code manipulation tools
//! that allow LLM entities to write and manage code.

mod registry;
mod executor;
mod definitions;

pub use registry::{ToolRegistry, ToolDefinition, ToolParameter, ToolCategory};
pub use executor::{ToolExecutor, ToolExecutionResult};
pub use definitions::*;
