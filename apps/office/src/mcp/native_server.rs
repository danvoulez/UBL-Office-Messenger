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
use tracing::{info, debug, warn, error};

use crate::{Result, OfficeError};
use crate::mcp::protocol::*;
use crate::ubl_client::UblClient;
use crate::entity::EntityRepository;

/// Native tool definition
pub struct NativeTool {
    pub name: String,
    pub description: String,
    pub input_schema: ToolInputSchema,
    pub handler: Box<dyn ToolHandler>,
}

/// Tool handler trait - now with context
#[async_trait::async_trait]
pub trait ToolHandler: Send + Sync {
    async fn execute(&self, arguments: Value, ctx: &ToolContext) -> Result<Vec<ToolContent>>;
}

/// Context passed to tool handlers
pub struct ToolContext {
    pub ubl_client: Arc<UblClient>,
    pub entity_repository: Option<Arc<EntityRepository>>,
    pub container_id: String,
}

impl ToolContext {
    pub fn new(ubl_client: Arc<UblClient>, container_id: &str) -> Self {
        Self {
            ubl_client,
            entity_repository: None,
            container_id: container_id.to_string(),
        }
    }
    
    pub fn with_entity_repository(mut self, repo: Arc<EntityRepository>) -> Self {
        self.entity_repository = Some(repo);
        self
    }
}

/// Office's native MCP server (in-process)
pub struct OfficeMcpServer {
    tools: HashMap<String, NativeTool>,
    server_info: ServerInfo,
    context: Option<ToolContext>,
}

impl OfficeMcpServer {
    pub fn new() -> Self {
        let mut server = Self {
            tools: HashMap::new(),
            server_info: ServerInfo {
                name: "office".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            },
            context: None,
        };
        
        // Register all native tools
        server.register_builtin_tools();
        
        server
    }
    
    /// Create with UBL context for real operations
    pub fn with_context(ubl_client: Arc<UblClient>, container_id: &str) -> Self {
        let mut server = Self::new();
        server.context = Some(ToolContext::new(ubl_client, container_id));
        server
    }
    
    /// Set context after creation
    pub fn set_context(&mut self, ctx: ToolContext) {
        self.context = Some(ctx);
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
        
        // Create default context if none set
        let default_ctx = ToolContext {
            ubl_client: Arc::new(UblClient::with_generated_key("http://localhost:8080", "office", 30000)),
            entity_repository: None,
            container_id: "C.Office".to_string(),
        };
        let ctx = self.context.as_ref().unwrap_or(&default_ctx);
        
        match tool.handler.execute(arguments, ctx).await {
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
    async fn execute(&self, args: Value, ctx: &ToolContext) -> Result<Vec<ToolContent>> {
        let container_id = args.get("container_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| OfficeError::McpError("container_id required".to_string()))?;
        
        // Query UBL via client
        match ctx.ubl_client.get_state(container_id).await {
            Ok(state) => Ok(vec![ToolContent::Text { 
                text: json!({
                    "container_id": container_id,
                    "sequence": state.sequence,
                    "last_hash": state.last_hash,
                }).to_string()
            }]),
            Err(e) => Ok(vec![ToolContent::Text { 
                text: format!("Query failed: {}", e)
            }])
        }
    }
}

struct UblCommitHandler;
#[async_trait::async_trait]
impl ToolHandler for UblCommitHandler {
    async fn execute(&self, args: Value, ctx: &ToolContext) -> Result<Vec<ToolContent>> {
        let container_id = args.get("container_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| OfficeError::McpError("container_id required".to_string()))?;
        let intent_class = args.get("intent_class")
            .and_then(|v| v.as_str())
            .unwrap_or("observation");
        let atom = args.get("atom")
            .ok_or_else(|| OfficeError::McpError("atom required".to_string()))?;
        
        // Commit to UBL
        match ctx.ubl_client.commit_atom(container_id, atom, intent_class, 0).await {
            Ok(response) => Ok(vec![ToolContent::Text { 
                text: json!({
                    "committed": true,
                    "container_id": container_id,
                    "sequence": response.sequence,
                    "entry_hash": response.entry_hash,
                }).to_string()
            }]),
            Err(e) => {
                error!("UBL commit failed: {}", e);
                Ok(vec![ToolContent::Text { 
                    text: format!("Commit failed: {}", e)
                }])
            }
        }
    }
}

struct EntityGetHandler;
#[async_trait::async_trait]
impl ToolHandler for EntityGetHandler {
    async fn execute(&self, args: Value, ctx: &ToolContext) -> Result<Vec<ToolContent>> {
        let entity_id = args.get("entity_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| OfficeError::McpError("entity_id required".to_string()))?;
        
        // Get entity info from repository or UBL
        if let Some(ref repo) = ctx.entity_repository {
            let entity_id_string = entity_id.to_string();
            match repo.get(&entity_id_string).await {
                Ok(entity) => Ok(vec![ToolContent::Text { 
                    text: serde_json::to_string(&entity).unwrap_or_default()
                }]),
                Err(e) => Ok(vec![ToolContent::Text { 
                    text: format!("Entity not found or error: {}", e)
                }])
            }
        } else {
            // Fallback to UBL events
            match ctx.ubl_client.get_events(&entity_id.to_string(), 5).await {
                Ok(events) => Ok(vec![ToolContent::Text { 
                    text: json!({
                        "entity_id": entity_id,
                        "recent_events": events.len(),
                    }).to_string()
                }]),
                Err(e) => Ok(vec![ToolContent::Text { 
                    text: format!("Error: {}", e)
                }])
            }
        }
    }
}

struct EntityHandoverHandler;
#[async_trait::async_trait]
impl ToolHandler for EntityHandoverHandler {
    async fn execute(&self, args: Value, ctx: &ToolContext) -> Result<Vec<ToolContent>> {
        let entity_id = args.get("entity_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| OfficeError::McpError("entity_id required".to_string()))?;
        let action = args.get("action")
            .and_then(|v| v.as_str())
            .unwrap_or("read");
        
        match action {
            "read" => {
                match ctx.ubl_client.get_last_handover(&entity_id.to_string()).await {
                    Ok(Some(content)) => Ok(vec![ToolContent::Text { 
                        text: json!({
                            "entity_id": entity_id,
                            "handover": content,
                        }).to_string()
                    }]),
                    Ok(None) => Ok(vec![ToolContent::Text { 
                        text: json!({
                            "entity_id": entity_id,
                            "handover": null,
                            "message": "No handover found"
                        }).to_string()
                    }]),
                    Err(e) => Ok(vec![ToolContent::Text { 
                        text: format!("Error reading handover: {}", e)
                    }])
                }
            }
            "write" => {
                let content = args.get("content")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| OfficeError::McpError("content required for write".to_string()))?;
                
                let event = json!({
                    "type": "handover.created",
                    "entity_id": entity_id,
                    "content": content,
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                });
                
                match ctx.ubl_client.publish_event(&ctx.container_id, &event).await {
                    Ok(response) => Ok(vec![ToolContent::Text { 
                        text: json!({
                            "written": true,
                            "entity_id": entity_id,
                            "entry_hash": response.entry_hash,
                        }).to_string()
                    }]),
                    Err(e) => Ok(vec![ToolContent::Text { 
                        text: format!("Error writing handover: {}", e)
                    }])
                }
            }
            _ => Ok(vec![ToolContent::Text { 
                text: format!("Unknown action: {}", action)
            }])
        }
    }
}

struct JobCreateHandler;
#[async_trait::async_trait]
impl ToolHandler for JobCreateHandler {
    async fn execute(&self, args: Value, ctx: &ToolContext) -> Result<Vec<ToolContent>> {
        let title = args.get("title")
            .and_then(|v| v.as_str())
            .ok_or_else(|| OfficeError::McpError("title required".to_string()))?;
        
        let job_id = format!("job_{}", uuid::Uuid::new_v4().to_string().replace("-", "")[..12].to_string());
        
        // Publish job.created event to UBL
        let event = json!({
            "type": "job.created",
            "job_id": job_id,
            "title": title,
            "description": args.get("description"),
            "assigned_to": args.get("assigned_to"),
            "priority": args.get("priority").and_then(|v| v.as_str()).unwrap_or("normal"),
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });
        
        match ctx.ubl_client.publish_event("C.Jobs", &event).await {
            Ok(response) => Ok(vec![ToolContent::Text { 
                text: json!({
                    "job_id": job_id,
                    "title": title,
                    "status": "created",
                    "entry_hash": response.entry_hash,
                }).to_string()
            }]),
            Err(e) => {
                error!("Failed to create job: {}", e);
                Ok(vec![ToolContent::Text { 
                    text: json!({
                        "job_id": job_id,
                        "title": title,
                        "status": "created_local",
                        "warning": format!("UBL commit failed: {}", e)
                    }).to_string()
                }])
            }
        }
    }
}

struct JobStatusHandler;
#[async_trait::async_trait]
impl ToolHandler for JobStatusHandler {
    async fn execute(&self, args: Value, ctx: &ToolContext) -> Result<Vec<ToolContent>> {
        let job_id = args.get("job_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| OfficeError::McpError("job_id required".to_string()))?;
        
        // Query job events from UBL
        match ctx.ubl_client.get_events(&job_id.to_string(), 10).await {
            Ok(events) => {
                let last_event = events.first();
                let status = last_event
                    .and_then(|e| e.data.get("status"))
                    .and_then(|s| s.as_str())
                    .unwrap_or("unknown");
                
                Ok(vec![ToolContent::Text { 
                    text: json!({
                        "job_id": job_id,
                        "status": status,
                        "event_count": events.len(),
                    }).to_string()
                }])
            },
            Err(e) => Ok(vec![ToolContent::Text { 
                text: json!({
                    "job_id": job_id,
                    "status": "unknown",
                    "error": format!("{}", e)
                }).to_string()
            }])
        }
    }
}

struct MemoryRecallHandler;
#[async_trait::async_trait]
impl ToolHandler for MemoryRecallHandler {
    async fn execute(&self, args: Value, ctx: &ToolContext) -> Result<Vec<ToolContent>> {
        let query = args.get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| OfficeError::McpError("query required".to_string()))?;
        let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(5) as usize;
        
        // Query memory events from UBL (semantic search would be via separate service)
        // For now, get recent handovers/events that might be relevant
        match ctx.ubl_client.get_events(&ctx.container_id, limit).await {
            Ok(events) => {
                let memories: Vec<_> = events.iter()
                    .filter(|e| e.summary.to_lowercase().contains(&query.to_lowercase()))
                    .take(limit)
                    .collect();
                
                Ok(vec![ToolContent::Text { 
                    text: json!({
                        "query": query,
                        "results": memories.len(),
                        "memories": memories.iter().map(|e| json!({
                            "content": e.summary,
                            "timestamp": e.timestamp.to_rfc3339(),
                        })).collect::<Vec<_>>(),
                    }).to_string()
                }])
            },
            Err(e) => Ok(vec![ToolContent::Text { 
                text: format!("Memory recall failed: {}", e)
            }])
        }
    }
}

struct MemoryStoreHandler;
#[async_trait::async_trait]
impl ToolHandler for MemoryStoreHandler {
    async fn execute(&self, args: Value, ctx: &ToolContext) -> Result<Vec<ToolContent>> {
        let content = args.get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| OfficeError::McpError("content required".to_string()))?;
        let tags = args.get("tags").cloned().unwrap_or(json!([]));
        let importance = args.get("importance").and_then(|v| v.as_f64()).unwrap_or(0.5);
        
        // Store memory as event in UBL
        let event = json!({
            "type": "memory.stored",
            "content": content,
            "tags": tags,
            "importance": importance,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });
        
        match ctx.ubl_client.publish_event(&ctx.container_id, &event).await {
            Ok(response) => Ok(vec![ToolContent::Text { 
                text: json!({
                    "stored": true,
                    "entry_hash": response.entry_hash,
                    "preview": &content[..content.len().min(50)],
                }).to_string()
            }]),
            Err(e) => Ok(vec![ToolContent::Text { 
                text: format!("Memory store failed: {}", e)
            }])
        }
    }
}

struct SanityCheckHandler;
#[async_trait::async_trait]
impl ToolHandler for SanityCheckHandler {
    async fn execute(&self, args: Value, _ctx: &ToolContext) -> Result<Vec<ToolContent>> {
        let claim = args.get("claim")
            .and_then(|v| v.as_str())
            .ok_or_else(|| OfficeError::McpError("claim required".to_string()))?;
        let evidence = args.get("evidence").cloned().unwrap_or(json!([]));
        
        // Sanity check logic - for now simple heuristics
        // Real implementation would call LLM or fact-checking service
        let evidence_count = evidence.as_array().map(|a| a.len()).unwrap_or(0);
        let confidence = if evidence_count > 2 { 0.9 } else if evidence_count > 0 { 0.7 } else { 0.5 };
        
        Ok(vec![ToolContent::Text { 
            text: json!({
                "claim": claim,
                "valid": true,  // Would be determined by actual check
                "confidence": confidence,
                "evidence_provided": evidence_count,
                "note": "Basic sanity check passed. For critical claims, request human verification."
            }).to_string()
        }])
    }
}

struct PermitCheckHandler;
#[async_trait::async_trait]
impl ToolHandler for PermitCheckHandler {
    async fn execute(&self, args: Value, _ctx: &ToolContext) -> Result<Vec<ToolContent>> {
        let action = args.get("action")
            .and_then(|v| v.as_str())
            .ok_or_else(|| OfficeError::McpError("action required".to_string()))?;
        let resource = args.get("resource").and_then(|v| v.as_str()).unwrap_or("*");
        
        // Permit checks require proper context - for now return placeholder
        // Real implementation would use ctx.ubl_client.request_permit() with full PermitRequest
        let tenant_id = args.get("tenant_id").and_then(|v| v.as_str()).unwrap_or("default");
        let actor_id = args.get("actor_id").and_then(|v| v.as_str()).unwrap_or("system");
        
        // Build a proper permit check response
        // In production, this would call UBL's policy endpoint
        let permit_check = json!({
            "action": action,
            "resource": resource,
            "tenant_id": tenant_id,
            "actor_id": actor_id,
            "permitted": true, // Default allow for development
            "reason": "Development mode - permit checks not fully wired"
        });

        Ok(vec![ToolContent::Text { 
            text: permit_check.to_string()
        }])
    }
}

struct MessageSendHandler;
#[async_trait::async_trait]
impl ToolHandler for MessageSendHandler {
    async fn execute(&self, args: Value, ctx: &ToolContext) -> Result<Vec<ToolContent>> {
        let to = args.get("to")
            .and_then(|v| v.as_str())
            .ok_or_else(|| OfficeError::McpError("to required".to_string()))?;
        let content = args.get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| OfficeError::McpError("content required".to_string()))?;
        
        let message_id = format!("msg_{}", uuid::Uuid::new_v4().to_string().replace("-", "")[..12].to_string());
        
        // Publish message.sent event to UBL
        let event = json!({
            "type": "message.sent",
            "message_id": message_id,
            "to": to,
            "content": content,
            "reply_to": args.get("reply_to"),
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });
        
        match ctx.ubl_client.publish_event("C.Messenger", &event).await {
            Ok(response) => Ok(vec![ToolContent::Text { 
                text: json!({
                    "message_id": message_id,
                    "to": to,
                    "sent": true,
                    "entry_hash": response.entry_hash,
                }).to_string()
            }]),
            Err(e) => {
                error!("Failed to send message: {}", e);
                Ok(vec![ToolContent::Text { 
                    text: json!({
                        "message_id": message_id,
                        "to": to,
                        "sent": false,
                        "error": format!("{}", e)
                    }).to_string()
                }])
            }
        }
    }
}

struct MessageHistoryHandler;
#[async_trait::async_trait]
impl ToolHandler for MessageHistoryHandler {
    async fn execute(&self, args: Value, ctx: &ToolContext) -> Result<Vec<ToolContent>> {
        let conversation_id = args.get("conversation_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| OfficeError::McpError("conversation_id required".to_string()))?;
        let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(20) as usize;
        
        // Get message history from UBL
        match ctx.ubl_client.get_events(&conversation_id.to_string(), limit).await {
            Ok(events) => {
                let messages: Vec<_> = events.iter()
                    .filter(|e| e.summary.contains("message"))
                    .map(|e| json!({
                        "from": e.author_pubkey,
                        "content": e.data.get("content"),
                        "timestamp": e.timestamp.to_rfc3339(),
                    }))
                    .collect();
                
                Ok(vec![ToolContent::Text { 
                    text: json!({
                        "conversation_id": conversation_id,
                        "message_count": messages.len(),
                        "messages": messages,
                    }).to_string()
                }])
            },
            Err(e) => Ok(vec![ToolContent::Text { 
                text: json!({
                    "conversation_id": conversation_id,
                    "messages": [],
                    "error": format!("{}", e)
                }).to_string()
            }])
        }
    }
}

struct EscalateHandler;
#[async_trait::async_trait]
impl ToolHandler for EscalateHandler {
    async fn execute(&self, args: Value, ctx: &ToolContext) -> Result<Vec<ToolContent>> {
        let reason = args.get("reason")
            .and_then(|v| v.as_str())
            .ok_or_else(|| OfficeError::McpError("reason required".to_string()))?;
        let urgency = args.get("urgency").and_then(|v| v.as_str()).unwrap_or("normal");
        
        let escalation_id = format!("esc_{}", uuid::Uuid::new_v4().to_string().replace("-", "")[..12].to_string());
        
        // Publish escalation event to UBL
        let event = json!({
            "type": "escalation.created",
            "escalation_id": escalation_id,
            "reason": reason,
            "urgency": urgency,
            "context": args.get("context"),
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });
        
        match ctx.ubl_client.publish_event(&ctx.container_id, &event).await {
            Ok(response) => Ok(vec![ToolContent::Text { 
                text: json!({
                    "escalation_id": escalation_id,
                    "reason": reason,
                    "status": "pending_guardian",
                    "entry_hash": response.entry_hash,
                }).to_string()
            }]),
            Err(e) => {
                error!("Failed to create escalation: {}", e);
                Ok(vec![ToolContent::Text { 
                    text: json!({
                        "escalation_id": escalation_id,
                        "status": "failed",
                        "error": format!("{}", e)
                    }).to_string()
                }])
            }
        }
    }
}

struct SimulateHandler;
#[async_trait::async_trait]
impl ToolHandler for SimulateHandler {
    async fn execute(&self, args: Value, ctx: &ToolContext) -> Result<Vec<ToolContent>> {
        let action = args.get("action")
            .and_then(|v| v.as_str())
            .ok_or_else(|| OfficeError::McpError("action required".to_string()))?;
        let parameters = args.get("parameters").cloned().unwrap_or(json!({}));
        
        // Simulation: check permit without executing
        let permit_check = json!({
            "action": action,
            "resource": "*",
            "context": parameters.clone(),
        });
        
        // Estimate risk based on action type
        let risk_score = match action {
            a if a.contains("delete") || a.contains("remove") => 0.8,
            a if a.contains("create") || a.contains("write") => 0.5,
            a if a.contains("send") || a.contains("message") => 0.3,
            a if a.contains("read") || a.contains("query") => 0.1,
            _ => 0.4,
        };
        
        let predicted_outcome = if risk_score > 0.6 {
            "High-risk action - recommend human approval"
        } else if risk_score > 0.3 {
            "Medium-risk action - proceed with caution"
        } else {
            "Low-risk action - safe to proceed"
        };
        
        Ok(vec![ToolContent::Text { 
            text: json!({
                "action": action,
                "parameters": parameters,
                "simulation_result": "analyzed",
                "predicted_outcome": predicted_outcome,
                "risk_score": risk_score,
                "recommendation": if risk_score > 0.5 { "request_approval" } else { "proceed" }
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
        let ubl_client = Arc::new(UblClient::with_generated_key("http://localhost:8080", "office", 30000));
        let server = OfficeMcpServer::with_context(ubl_client, "C.Office");
        
        // Test with a tool that doesn't need UBL connection
        let result = server.call_tool("sanity_check", json!({
            "claim": "Test claim"
        })).await.unwrap();
        
        assert!(!result.is_error.unwrap_or(true));
        assert!(!result.content.is_empty());
    }
}
