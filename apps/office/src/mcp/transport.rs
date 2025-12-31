//! MCP Transport Layer
//!
//! Handles communication with MCP servers over stdio

use std::collections::HashMap;
use std::process::Stdio;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::{mpsc, oneshot, RwLock};
use tracing::{debug, error, info, warn};

use crate::{OfficeError, Result};
use super::protocol::{JsonRpcRequest, JsonRpcResponse, RequestId};

/// Transport for stdio-based MCP communication
pub struct StdioTransport {
    /// Child process
    child: Child,
    /// Sender for outgoing messages
    tx: mpsc::Sender<String>,
    /// Pending requests waiting for response
    pending: Arc<RwLock<HashMap<RequestId, oneshot::Sender<JsonRpcResponse>>>>,
    /// Request ID counter
    next_id: AtomicU64,
    /// Server name for logging
    server_name: String,
}

impl StdioTransport {
    /// Spawn a new MCP server process
    pub async fn spawn(command: &str, args: &[&str], server_name: &str) -> Result<Self> {
        info!("Spawning MCP server: {} {:?}", command, args);

        let mut child = Command::new(command)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| OfficeError::McpError(format!("Failed to spawn {}: {}", command, e)))?;

        let stdin = child.stdin.take()
            .ok_or_else(|| OfficeError::McpError("Failed to get stdin".to_string()))?;
        let stdout = child.stdout.take()
            .ok_or_else(|| OfficeError::McpError("Failed to get stdout".to_string()))?;
        let stderr = child.stderr.take();

        // Channel for outgoing messages
        let (tx, mut rx) = mpsc::channel::<String>(100);
        let pending: Arc<RwLock<HashMap<RequestId, oneshot::Sender<JsonRpcResponse>>>> = 
            Arc::new(RwLock::new(HashMap::new()));

        // Writer task - sends messages to stdin
        let mut stdin = stdin;
        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                debug!("MCP OUT: {}", msg.trim());
                if let Err(e) = stdin.write_all(msg.as_bytes()).await {
                    error!("Failed to write to MCP stdin: {}", e);
                    break;
                }
                if let Err(e) = stdin.flush().await {
                    error!("Failed to flush MCP stdin: {}", e);
                    break;
                }
            }
        });

        // Reader task - reads responses from stdout
        let pending_clone = pending.clone();
        let server_name_clone = server_name.to_string();
        tokio::spawn(async move {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();

            while let Ok(Some(line)) = lines.next_line().await {
                debug!("MCP IN [{}]: {}", server_name_clone, line);

                // Parse JSON-RPC response
                match serde_json::from_str::<JsonRpcResponse>(&line) {
                    Ok(response) => {
                        let mut pending = pending_clone.write().await;
                        if let Some(sender) = pending.remove(&response.id) {
                            let _ = sender.send(response);
                        } else {
                            warn!("Received response for unknown request: {:?}", response.id);
                        }
                    }
                    Err(e) => {
                        // Might be a notification, log and continue
                        debug!("Failed to parse as response: {} - {}", e, line);
                    }
                }
            }
            info!("MCP reader task ended for {}", server_name_clone);
        });

        // Stderr reader - log errors
        if let Some(stderr) = stderr {
            let server_name_clone = server_name.to_string();
            tokio::spawn(async move {
                let reader = BufReader::new(stderr);
                let mut lines = reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    warn!("MCP STDERR [{}]: {}", server_name_clone, line);
                }
            });
        }

        Ok(Self {
            child,
            tx,
            pending,
            next_id: AtomicU64::new(1),
            server_name: server_name.to_string(),
        })
    }

    /// Send a request and wait for response
    pub async fn request(&self, method: &str, params: Option<serde_json::Value>) -> Result<JsonRpcResponse> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let request = JsonRpcRequest::new(id, method, params);

        // Create response channel
        let (response_tx, response_rx) = oneshot::channel();
        
        {
            let mut pending = self.pending.write().await;
            pending.insert(request.id.clone(), response_tx);
        }

        // Serialize and send
        let msg = serde_json::to_string(&request)
            .map_err(|e| OfficeError::McpError(format!("Failed to serialize request: {}", e)))?;

        self.tx.send(format!("{}\n", msg)).await
            .map_err(|e| OfficeError::McpError(format!("Failed to send request: {}", e)))?;

        // Wait for response with timeout
        let response = tokio::time::timeout(
            std::time::Duration::from_secs(30),
            response_rx,
        ).await
            .map_err(|_| OfficeError::McpError(format!("Request timeout: {}", method)))?
            .map_err(|_| OfficeError::McpError("Response channel closed".to_string()))?;

        // Check for error
        if let Some(error) = &response.error {
            return Err(OfficeError::McpError(format!(
                "MCP error {}: {}", error.code, error.message
            )));
        }

        Ok(response)
    }

    /// Send a notification (no response expected)
    pub async fn notify(&self, method: &str, params: Option<serde_json::Value>) -> Result<()> {
        let notification = super::protocol::JsonRpcNotification {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params,
        };

        let msg = serde_json::to_string(&notification)
            .map_err(|e| OfficeError::McpError(format!("Failed to serialize notification: {}", e)))?;

        self.tx.send(format!("{}\n", msg)).await
            .map_err(|e| OfficeError::McpError(format!("Failed to send notification: {}", e)))?;

        Ok(())
    }

    /// Get server name
    pub fn server_name(&self) -> &str {
        &self.server_name
    }

    /// Check if process is still running
    pub fn is_running(&mut self) -> bool {
        match self.child.try_wait() {
            Ok(None) => true,
            Ok(Some(status)) => {
                info!("MCP server {} exited with: {}", self.server_name, status);
                false
            }
            Err(e) => {
                error!("Failed to check MCP server status: {}", e);
                false
            }
        }
    }

    /// Kill the server process
    pub async fn kill(&mut self) -> Result<()> {
        info!("Killing MCP server: {}", self.server_name);
        self.child.kill().await
            .map_err(|e| OfficeError::McpError(format!("Failed to kill server: {}", e)))?;
        Ok(())
    }
}

impl Drop for StdioTransport {
    fn drop(&mut self) {
        // Try to kill the child process on drop
        if let Err(e) = self.child.start_kill() {
            debug!("Failed to kill MCP server on drop: {}", e);
        }
    }
}
