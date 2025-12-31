//! Office Native MCP Server
//!
//! Exposes Office's native tools as an MCP server.
//! This way the LLM only sees MCP tools - uniform interface for everything.
//!
//! Architecture:
//! ```
//!                         LLM
//!                          │
//!                          ▼
//!                   ┌──────────────┐
//!                   │ ToolExecutor │
//!                   └──────┬───────┘
//!                          │
//!            ┌─────────────┼─────────────┐
//!            ▼             ▼             ▼
//!      ┌──────────┐  ┌──────────┐  ┌──────────┐
//!      │  office  │  │filesystem│  │  github  │
//!      │(native)  │  │  (npx)   │  │  (npx)   │
//!      └──────────┘  └──────────┘  └──────────┘
//! ```

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde_json::{json, Value};
use tracing::{info, debug, warn};

use crate::{Result, OfficeError};
use crate::mcp::protocol::*;

/// Native tool definition
pub struct NativeTool {
    pub name: String,
    pub description: String,
    pub input_schema: ToolInputSchema,
    pub handler: Box<dyn ToolHandler>,
}

/// Tool handler trait
#[async_trait::async_trait]
pub trait ToolHandler: Send + Sync {
    async fn execute(&self, arguments: Value) -> Result<Vec<ToolContent>>;
}

/// Office's native MCP server (in-process)
pub struct OfficeMcpServer {
    tools: HashMap<String, NativeTool>,
    server_info: ServerInfo,
}

impl OfficeMcpServer {
    pub fn new() -> Self {
        let mut server = Self {
            tools: HashMap::new(),
            server_info: ServerInfo {
                name: "office".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            },
        };
        
        // Register all native tools
        server.register_builtin_tools();
        
        server
    }

    /// Register all builtin Office tools
    fn register_builtin_tools(&mut self) {
        // === UBL Tools ===
        self.register_tool(NativeTool {
            name: "ubl_query".to_string(),
            description: "Query the UBL ledger for events, state, or projections".to_string(),
            input_schema: ToolInputSchema {
                schema_type: "object".to_string(),
                properties: Some(json!({
                    "container_id": { "type": "string", "description": "Container to query" },
                    "projection": { "type": "string", "description": "Projection name (optional)" },
                    "filter": { "type": "object", "description": "Query filters (optional)" }
                })),
                required: Some(vec!["container_id".to_string()]),
                extra: HashMap::new(),
            },
            handler: Box::new(UblQueryHandler),
        });

        self.register_tool(NativeTool {
            name: "ubl_commit".to_string(),
            description: "Commit an atom to the UBL ledger (creates immutable record)".to_string(),
            input_schema: ToolInputSchema {
                schema_type: "object".to_string(),
                properties: Some(json!({
                    "container_id": { "type": "string", "description": "Target container" },
                    "intent_class": { "type": "string", "description": "Intent class (observation, mutation, etc)" },
                    "atom": { "type": "object", "description": "The atom payload to commit" }
                })),
                required: Some(vec!["container_id".to_string(), "intent_class".to_string(), "atom".to_string()]),
                extra: HashMap::new(),
            },
            handler: Box::new(UblCommitHandler),
        });

        // === Entity Tools ===
        self.register_tool(NativeTool {
            name: "entity_get".to_string(),
            description: "Get information about an entity (LLM identity)".to_string(),
            input_schema: ToolInputSchema {
                schema_type: "object".to_string(),
                properties: Some(json!({
                    "entity_id": { "type": "string", "description": "Entity ID to look up" }
                })),
                required: Some(vec!["entity_id".to_string()]),
                extra: HashMap::new(),
            },
            handler: Box::new(EntityGetHandler),
        });

        self.register_tool(NativeTool {
            name: "entity_handover".to_string(),
            description: "Read or write handover notes for an entity".to_string(),
            input_schema: ToolInputSchema {
                schema_type: "object".to_string(),
                properties: Some(json!({
                    "entity_id": { "type": "string", "description": "Entity ID" },
                    "action": { "type": "string", "enum": ["read", "write"], "description": "Action to perform" },
                    "content": { "type": "string", "description": "Handover content (for write)" }
                })),
                required: Some(vec!["entity_id".to_string(), "action".to_string()]),
                extra: HashMap::new(),
            },
            handler: Box::new(EntityHandoverHandler),
        });

        // === Job Tools ===
        self.register_tool(NativeTool {
            name: "job_create".to_string(),
            description: "Create a new job/task for execution".to_string(),
            input_schema: ToolInputSchema {
                schema_type: "object".to_string(),
                properties: Some(json!({
                    "title": { "type": "string", "description": "Job title" },
                    "description": { "type": "string", "description": "Job description" },
                    "assigned_to": { "type": "string", "description": "Entity ID to assign to" },
                    "priority": { "type": "string", "enum": ["low", "normal", "high", "urgent"] }
                })),
                required: Some(vec!["title".to_string()]),
                extra: HashMap::new(),
            },
            handler: Box::new(JobCreateHandler),
        });

        self.register_tool(NativeTool {
            name: "job_status".to_string(),
            description: "Get status of a job".to_string(),
            input_schema: ToolInputSchema {
                schema_type: "object".to_string(),
                properties: Some(json!({
                    "job_id": { "type": "string", "description": "Job ID to check" }
                })),
                required: Some(vec!["job_id".to_string()]),
                extra: HashMap::new(),
            },
            handler: Box::new(JobStatusHandler),
        });

        // === Memory Tools ===
        self.register_tool(NativeTool {
            name: "memory_recall".to_string(),
            description: "Recall memories from the entity's memory store".to_string(),
            input_schema: ToolInputSchema {
                schema_type: "object".to_string(),
                properties: Some(json!({
                    "query": { "type": "string", "description": "What to recall" },
                    "limit": { "type": "integer", "description": "Max memories to return" }
                })),
                required: Some(vec!["query".to_string()]),
                extra: HashMap::new(),
            },
            handler: Box::new(MemoryRecallHandler),
        });

        self.register_tool(NativeTool {
            name: "memory_store".to_string(),
            description: "Store a memory for future recall".to_string(),
            input_schema: ToolInputSchema {
                schema_type: "object".to_string(),
                properties: Some(json!({
                    "content": { "type": "string", "description": "Memory content" },
                    "tags": { "type": "array", "items": { "type": "string" }, "description": "Tags for categorization" },
                    "importance": { "type": "number", "description": "Importance score 0-1" }
                })),
                required: Some(vec!["content".to_string()]),
                extra: HashMap::new(),
            },
            handler: Box::new(MemoryStoreHandler),
        });

        // === Governance Tools ===
        self.register_tool(NativeTool {
            name: "sanity_check".to_string(),
            description: "Validate a claim against objective facts".to_string(),
            input_schema: ToolInputSchema {
                schema_type: "object".to_string(),
                properties: Some(json!({
                    "claim": { "type": "string", "description": "Claim to validate" },
                    "evidence": { "type": "array", "items": { "type": "string" }, "description": "Supporting evidence" }
                })),
                required: Some(vec!["claim".to_string()]),
                extra: HashMap::new(),
            },
            handler: Box::new(SanityCheckHandler),
        });

        self.register_tool(NativeTool {
            name: "permit_check".to_string(),
            description: "Check if an action is permitted by UBL policy".to_string(),
            input_schema: ToolInputSchema {
                schema_type: "object".to_string(),
                properties: Some(json!({
                    "action": { "type": "string", "description": "Action to check" },
                    "resource": { "type": "string", "description": "Target resource" },
                    "context": { "type": "object", "description": "Additional context" }
                })),
                required: Some(vec!["action".to_string()]),
                extra: HashMap::new(),
            },
            handler: Box::new(PermitCheckHandler),
        });

        // === Communication Tools ===
        self.register_tool(NativeTool {
            name: "message_send".to_string(),
            description: "Send a message to another entity or channel".to_string(),
            input_schema: ToolInputSchema {
                schema_type: "object".to_string(),
                properties: Some(json!({
                    "to": { "type": "string", "description": "Recipient entity ID or channel" },
                    "content": { "type": "string", "description": "Message content" },
                    "reply_to": { "type": "string", "description": "Message ID to reply to (optional)" }
                })),
                required: Some(vec!["to".to_string(), "content".to_string()]),
                extra: HashMap::new(),
            },
            handler: Box::new(MessageSendHandler),
        });

        self.register_tool(NativeTool {
            name: "message_history".to_string(),
            description: "Get message history from a conversation".to_string(),
            input_schema: ToolInputSchema {
                schema_type: "object".to_string(),
                properties: Some(json!({
                    "conversation_id": { "type": "string", "description": "Conversation to query" },
                    "limit": { "type": "integer", "description": "Max messages to return" },
                    "before": { "type": "string", "description": "Get messages before this ID" }
                })),
                required: Some(vec!["conversation_id".to_string()]),
                extra: HashMap::new(),
            },
            handler: Box::new(MessageHistoryHandler),
        });

        // === Escalation Tools ===
        self.register_tool(NativeTool {
            name: "escalate".to_string(),
            description: "Escalate to guardian or human supervisor".to_string(),
            input_schema: ToolInputSchema {
                schema_type: "object".to_string(),
                properties: Some(json!({
                    "reason": { "type": "string", "description": "Why escalating" },
                    "urgency": { "type": "string", "enum": ["low", "normal", "high", "critical"] },
                    "context": { "type": "object", "description": "Relevant context" }
                })),
                required: Some(vec!["reason".to_string()]),
                extra: HashMap::new(),
            },
            handler: Box::new(EscalateHandler),
        });

        // === Simulation Tools ===
        self.register_tool(NativeTool {
            name: "simulate".to_string(),
            description: "Simulate an action before executing it".to_string(),
            input_schema: ToolInputSchema {
                schema_type: "object".to_string(),
                properties: Some(json!({
                    "action": { "type": "string", "description": "Action to simulate" },
                    "parameters": { "type": "object", "description": "Action parameters" }
                })),
                required: Some(vec!["action".to_string()]),
                extra: HashMap::new(),
            },
            handler: Box::new(SimulateHandler),
        });

        info!("Registered {} native Office tools", self.tools.len());
    }

    /// Register a tool
    pub fn register_tool(&mut self, tool: NativeTool) {
        debug!("Registering native tool: {}", tool.name);
        self.tools.insert(tool.name.clone(), tool);
    }

    /// Get server info
    pub fn server_info(&self) -> &ServerInfo {
        &self.server_info
    }

    /// Get capabilities
    pub fn capabilities(&self) -> ServerCapabilities {
        ServerCapabilities {
            tools: Some(ToolsCapability { list_changed: Some(false) }),
            resources: None,
            prompts: None,
            logging: None,
        }
    }

    /// List all tools
    pub fn list_tools(&self) -> Vec<McpTool> {
        self.tools.values().map(|t| McpTool {
            name: t.name.clone(),
            description: Some(t.description.clone()),
            input_schema: t.input_schema.clone(),
        }).collect()
    }

    /// Call a tool
    pub async fn call_tool(&self, name: &str, arguments: Value) -> Result<CallToolResult> {
        let tool = self.tools.get(name)
            .ok_or_else(|| OfficeError::McpError(format!("Unknown tool: {}", name)))?;

        info!("Executing native tool: {}", name);
        
        match tool.handler.execute(arguments).await {
            Ok(content) => Ok(CallToolResult {
                content,
                is_error: Some(false),
            }),
            Err(e) => {
                warn!("Tool {} failed: {}", name, e);
                Ok(CallToolResult {
                    content: vec![ToolContent::Text { text: format!("Error: {}", e) }],
                    is_error: Some(true),
                })
            }
        }
    }

    /// Check if tool exists
    pub fn has_tool(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }

    /// Get tool
    pub fn get_tool(&self, name: &str) -> Option<McpTool> {
        self.tools.get(name).map(|t| McpTool {
            name: t.name.clone(),
            description: Some(t.description.clone()),
            input_schema: t.input_schema.clone(),
        })
    }
}

impl Default for OfficeMcpServer {
    fn default() -> Self {
        Self::new()
    }
}

// ============ Tool Handlers ============
// These are placeholder implementations - the real implementations
// will call into the actual Office subsystems

struct UblQueryHandler;
#[async_trait::async_trait]
impl ToolHandler for UblQueryHandler {
    async fn execute(&self, args: Value) -> Result<Vec<ToolContent>> {
        let container_id = args.get("container_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| OfficeError::McpError("container_id required".to_string()))?;
        
        // TODO: Actually query UBL
        Ok(vec![ToolContent::Text { 
            text: format!("Query results for container: {}", container_id)
        }])
    }
}

struct UblCommitHandler;
#[async_trait::async_trait]
impl ToolHandler for UblCommitHandler {
    async fn execute(&self, args: Value) -> Result<Vec<ToolContent>> {
        let container_id = args.get("container_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| OfficeError::McpError("container_id required".to_string()))?;
        
        // TODO: Actually commit to UBL
        Ok(vec![ToolContent::Text { 
            text: format!("Committed to container: {}", container_id)
        }])
    }
}

struct EntityGetHandler;
#[async_trait::async_trait]
impl ToolHandler for EntityGetHandler {
    async fn execute(&self, args: Value) -> Result<Vec<ToolContent>> {
        let entity_id = args.get("entity_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| OfficeError::McpError("entity_id required".to_string()))?;
        
        // TODO: Get from EntityRepository
        Ok(vec![ToolContent::Text { 
            text: format!("Entity info for: {}", entity_id)
        }])
    }
}

struct EntityHandoverHandler;
#[async_trait::async_trait]
impl ToolHandler for EntityHandoverHandler {
    async fn execute(&self, args: Value) -> Result<Vec<ToolContent>> {
        let entity_id = args.get("entity_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| OfficeError::McpError("entity_id required".to_string()))?;
        let action = args.get("action")
            .and_then(|v| v.as_str())
            .unwrap_or("read");
        
        // TODO: Read/write handover
        Ok(vec![ToolContent::Text { 
            text: format!("Handover {} for entity: {}", action, entity_id)
        }])
    }
}

struct JobCreateHandler;
#[async_trait::async_trait]
impl ToolHandler for JobCreateHandler {
    async fn execute(&self, args: Value) -> Result<Vec<ToolContent>> {
        let title = args.get("title")
            .and_then(|v| v.as_str())
            .ok_or_else(|| OfficeError::McpError("title required".to_string()))?;
        
        let job_id = uuid::Uuid::new_v4().to_string();
        
        // TODO: Actually create job
        Ok(vec![ToolContent::Text { 
            text: json!({
                "job_id": job_id,
                "title": title,
                "status": "created"
            }).to_string()
        }])
    }
}

struct JobStatusHandler;
#[async_trait::async_trait]
impl ToolHandler for JobStatusHandler {
    async fn execute(&self, args: Value) -> Result<Vec<ToolContent>> {
        let job_id = args.get("job_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| OfficeError::McpError("job_id required".to_string()))?;
        
        // TODO: Get job status
        Ok(vec![ToolContent::Text { 
            text: json!({
                "job_id": job_id,
                "status": "pending"
            }).to_string()
        }])
    }
}

struct MemoryRecallHandler;
#[async_trait::async_trait]
impl ToolHandler for MemoryRecallHandler {
    async fn execute(&self, args: Value) -> Result<Vec<ToolContent>> {
        let query = args.get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| OfficeError::McpError("query required".to_string()))?;
        
        // TODO: Semantic search in memory
        Ok(vec![ToolContent::Text { 
            text: format!("Memories matching: {}", query)
        }])
    }
}

struct MemoryStoreHandler;
#[async_trait::async_trait]
impl ToolHandler for MemoryStoreHandler {
    async fn execute(&self, args: Value) -> Result<Vec<ToolContent>> {
        let content = args.get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| OfficeError::McpError("content required".to_string()))?;
        
        // TODO: Store in memory
        Ok(vec![ToolContent::Text { 
            text: format!("Stored memory: {}...", &content[..content.len().min(50)])
        }])
    }
}

struct SanityCheckHandler;
#[async_trait::async_trait]
impl ToolHandler for SanityCheckHandler {
    async fn execute(&self, args: Value) -> Result<Vec<ToolContent>> {
        let claim = args.get("claim")
            .and_then(|v| v.as_str())
            .ok_or_else(|| OfficeError::McpError("claim required".to_string()))?;
        
        // TODO: Run sanity check
        Ok(vec![ToolContent::Text { 
            text: json!({
                "claim": claim,
                "valid": true,
                "confidence": 0.85
            }).to_string()
        }])
    }
}

struct PermitCheckHandler;
#[async_trait::async_trait]
impl ToolHandler for PermitCheckHandler {
    async fn execute(&self, args: Value) -> Result<Vec<ToolContent>> {
        let action = args.get("action")
            .and_then(|v| v.as_str())
            .ok_or_else(|| OfficeError::McpError("action required".to_string()))?;
        
        // TODO: Check UBL permit
        Ok(vec![ToolContent::Text { 
            text: json!({
                "action": action,
                "permitted": true,
                "reason": "Policy allows this action"
            }).to_string()
        }])
    }
}

struct MessageSendHandler;
#[async_trait::async_trait]
impl ToolHandler for MessageSendHandler {
    async fn execute(&self, args: Value) -> Result<Vec<ToolContent>> {
        let to = args.get("to")
            .and_then(|v| v.as_str())
            .ok_or_else(|| OfficeError::McpError("to required".to_string()))?;
        let content = args.get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| OfficeError::McpError("content required".to_string()))?;
        
        let message_id = uuid::Uuid::new_v4().to_string();
        
        // TODO: Send via Messenger
        Ok(vec![ToolContent::Text { 
            text: json!({
                "message_id": message_id,
                "to": to,
                "sent": true
            }).to_string()
        }])
    }
}

struct MessageHistoryHandler;
#[async_trait::async_trait]
impl ToolHandler for MessageHistoryHandler {
    async fn execute(&self, args: Value) -> Result<Vec<ToolContent>> {
        let conversation_id = args.get("conversation_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| OfficeError::McpError("conversation_id required".to_string()))?;
        
        // TODO: Get from Messenger
        Ok(vec![ToolContent::Text { 
            text: json!({
                "conversation_id": conversation_id,
                "messages": []
            }).to_string()
        }])
    }
}

struct EscalateHandler;
#[async_trait::async_trait]
impl ToolHandler for EscalateHandler {
    async fn execute(&self, args: Value) -> Result<Vec<ToolContent>> {
        let reason = args.get("reason")
            .and_then(|v| v.as_str())
            .ok_or_else(|| OfficeError::McpError("reason required".to_string()))?;
        
        let escalation_id = uuid::Uuid::new_v4().to_string();
        
        // TODO: Create escalation
        Ok(vec![ToolContent::Text { 
            text: json!({
                "escalation_id": escalation_id,
                "reason": reason,
                "status": "pending_guardian"
            }).to_string()
        }])
    }
}

struct SimulateHandler;
#[async_trait::async_trait]
impl ToolHandler for SimulateHandler {
    async fn execute(&self, args: Value) -> Result<Vec<ToolContent>> {
        let action = args.get("action")
            .and_then(|v| v.as_str())
            .ok_or_else(|| OfficeError::McpError("action required".to_string()))?;
        
        // TODO: Run simulation
        Ok(vec![ToolContent::Text { 
            text: json!({
                "action": action,
                "simulation_result": "success",
                "predicted_outcome": "Action would complete successfully",
                "risk_score": 0.2
            }).to_string()
        }])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_creation() {
        let server = OfficeMcpServer::new();
        assert!(server.tools.len() > 0);
        assert!(server.has_tool("ubl_query"));
        assert!(server.has_tool("entity_get"));
        assert!(server.has_tool("job_create"));
    }

    #[test]
    fn test_list_tools() {
        let server = OfficeMcpServer::new();
        let tools = server.list_tools();
        assert!(tools.len() >= 14);
    }

    #[tokio::test]
    async fn test_call_tool() {
        let server = OfficeMcpServer::new();
        let result = server.call_tool("job_create", json!({
            "title": "Test Job"
        })).await.unwrap();
        
        assert!(!result.is_error.unwrap_or(true));
        assert!(!result.content.is_empty());
    }
}
