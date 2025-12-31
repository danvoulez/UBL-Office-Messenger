//! MCP API Routes
//!
//! HTTP endpoints for MCP server management and tool invocation

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post, delete},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, error};

use crate::mcp::{McpRegistry, McpServerConfig, McpTool, CallToolResult, ToolContent};

/// MCP API State
pub struct McpState {
    pub registry: Arc<RwLock<McpRegistry>>,
}

impl McpState {
    pub fn new(registry: McpRegistry) -> Self {
        Self {
            registry: Arc::new(RwLock::new(registry)),
        }
    }
}

/// Create MCP router
pub fn router(state: Arc<McpState>) -> Router {
    Router::new()
        .route("/mcp/servers", get(list_servers))
        .route("/mcp/servers/:name", post(start_server))
        .route("/mcp/servers/:name", delete(stop_server))
        .route("/mcp/tools", get(list_tools))
        .route("/mcp/tools/:name", get(get_tool))
        .route("/mcp/tools/:name/call", post(call_tool))
        .route("/mcp/resources", get(list_resources))
        .route("/mcp/status", get(status))
        .with_state(state)
}

// ============ Response Types ============

#[derive(Serialize)]
struct ServerInfo {
    name: String,
    connected: bool,
}

#[derive(Serialize)]
struct StatusResponse {
    enabled: bool,
    servers: Vec<ServerInfo>,
    tool_count: usize,
    resource_count: usize,
}

#[derive(Serialize)]
struct ToolInfo {
    name: String,
    server: String,
    description: Option<String>,
    parameters: Option<serde_json::Value>,
}

#[derive(Serialize)]
struct ResourceInfo {
    uri: String,
    name: String,
    server: String,
    description: Option<String>,
    mime_type: Option<String>,
}

#[derive(Deserialize)]
struct CallToolRequest {
    arguments: Option<serde_json::Value>,
}

#[derive(Serialize)]
struct CallToolResponse {
    success: bool,
    content: Vec<ContentItem>,
    error: Option<String>,
}

#[derive(Serialize)]
struct ContentItem {
    #[serde(rename = "type")]
    content_type: String,
    text: Option<String>,
    data: Option<String>,
    mime_type: Option<String>,
}

impl From<ToolContent> for ContentItem {
    fn from(content: ToolContent) -> Self {
        match content {
            ToolContent::Text { text } => ContentItem {
                content_type: "text".to_string(),
                text: Some(text),
                data: None,
                mime_type: None,
            },
            ToolContent::Image { data, mime_type } => ContentItem {
                content_type: "image".to_string(),
                text: None,
                data: Some(data),
                mime_type: Some(mime_type),
            },
            ToolContent::Resource { resource } => ContentItem {
                content_type: "resource".to_string(),
                text: resource.text,
                data: resource.blob,
                mime_type: resource.mime_type,
            },
        }
    }
}

// ============ Handlers ============

/// GET /mcp/status
async fn status(State(state): State<Arc<McpState>>) -> Json<StatusResponse> {
    let registry = state.registry.read().await;
    let servers = registry.connected_servers().await;
    let tool_count = registry.tool_count().await;
    let resources = registry.all_resources().await;

    Json(StatusResponse {
        enabled: true,
        servers: servers.iter().map(|name| ServerInfo {
            name: name.clone(),
            connected: true,
        }).collect(),
        tool_count,
        resource_count: resources.len(),
    })
}

/// GET /mcp/servers
async fn list_servers(State(state): State<Arc<McpState>>) -> Json<Vec<ServerInfo>> {
    let registry = state.registry.read().await;
    let servers = registry.connected_servers().await;
    
    Json(servers.iter().map(|name| ServerInfo {
        name: name.clone(),
        connected: true,
    }).collect())
}

/// POST /mcp/servers/:name
async fn start_server(
    State(state): State<Arc<McpState>>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    let registry = state.registry.read().await;
    
    match registry.start_server(&name).await {
        Ok(()) => {
            info!("Started MCP server: {}", name);
            (StatusCode::OK, Json(serde_json::json!({
                "success": true,
                "server": name
            })))
        }
        Err(e) => {
            error!("Failed to start MCP server {}: {}", name, e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "success": false,
                "error": e.to_string()
            })))
        }
    }
}

/// DELETE /mcp/servers/:name
async fn stop_server(
    State(state): State<Arc<McpState>>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    let registry = state.registry.read().await;
    
    match registry.stop_server(&name).await {
        Ok(()) => {
            info!("Stopped MCP server: {}", name);
            (StatusCode::OK, Json(serde_json::json!({
                "success": true,
                "server": name
            })))
        }
        Err(e) => {
            error!("Failed to stop MCP server {}: {}", name, e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "success": false,
                "error": e.to_string()
            })))
        }
    }
}

/// GET /mcp/tools
async fn list_tools(State(state): State<Arc<McpState>>) -> Json<Vec<ToolInfo>> {
    let registry = state.registry.read().await;
    let tools = registry.all_tools().await;
    
    Json(tools.into_iter().map(|(server, tool)| ToolInfo {
        name: tool.name,
        server,
        description: tool.description,
        parameters: tool.input_schema.properties,
    }).collect())
}

/// GET /mcp/tools/:name
async fn get_tool(
    State(state): State<Arc<McpState>>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    let registry = state.registry.read().await;
    
    match registry.get_tool(&name).await {
        Some(tool) => {
            (StatusCode::OK, Json(serde_json::json!({
                "name": tool.name,
                "description": tool.description,
                "inputSchema": tool.input_schema
            })))
        }
        None => {
            (StatusCode::NOT_FOUND, Json(serde_json::json!({
                "error": format!("Tool not found: {}", name)
            })))
        }
    }
}

/// POST /mcp/tools/:name/call
async fn call_tool(
    State(state): State<Arc<McpState>>,
    Path(name): Path<String>,
    Json(request): Json<CallToolRequest>,
) -> impl IntoResponse {
    let registry = state.registry.read().await;
    
    info!("Calling MCP tool: {} with args: {:?}", name, request.arguments);
    
    match registry.call_tool(&name, request.arguments).await {
        Ok(result) => {
            let content: Vec<ContentItem> = result.content.into_iter().map(Into::into).collect();
            
            (StatusCode::OK, Json(CallToolResponse {
                success: !result.is_error.unwrap_or(false),
                content,
                error: None,
            }))
        }
        Err(e) => {
            error!("Tool call failed: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(CallToolResponse {
                success: false,
                content: vec![],
                error: Some(e.to_string()),
            }))
        }
    }
}

/// GET /mcp/resources
async fn list_resources(State(state): State<Arc<McpState>>) -> Json<Vec<ResourceInfo>> {
    let registry = state.registry.read().await;
    let resources = registry.all_resources().await;
    
    Json(resources.into_iter().map(|(server, resource)| ResourceInfo {
        uri: resource.uri,
        name: resource.name,
        server,
        description: resource.description,
        mime_type: resource.mime_type,
    }).collect())
}
