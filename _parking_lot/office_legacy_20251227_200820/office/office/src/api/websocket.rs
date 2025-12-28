//! WebSocket Handler for Real-Time Updates

use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::IntoResponse;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use super::http::AppState;

/// WebSocket message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum WsMessage {
    /// Subscribe to entity updates
    Subscribe { entity_id: String },
    /// Unsubscribe from entity updates
    Unsubscribe { entity_id: String },
    /// Session started
    SessionStarted { session_id: String, entity_id: String },
    /// Session ended
    SessionEnded { session_id: String, entity_id: String },
    /// Message received
    MessageReceived { session_id: String, content: String },
    /// Token usage update
    TokenUsage { session_id: String, used: u64, remaining: u64 },
    /// Error
    Error { message: String },
    /// Ping
    Ping,
    /// Pong
    Pong,
}

/// WebSocket handler
pub struct WebSocketHandler {
    state: Arc<RwLock<AppState>>,
}

impl WebSocketHandler {
    pub fn new(state: Arc<RwLock<AppState>>) -> Self {
        Self { state }
    }

    /// Handle WebSocket upgrade
    pub async fn handle(
        ws: WebSocketUpgrade,
        state: Arc<RwLock<AppState>>,
    ) -> impl IntoResponse {
        ws.on_upgrade(move |socket| Self::handle_socket(socket, state))
    }

    /// Handle WebSocket connection
    async fn handle_socket(socket: WebSocket, state: Arc<RwLock<AppState>>) {
        let (sender, mut receiver) = socket.split();

        // Spawn task to handle incoming messages
        let state_clone = state.clone();
        let receive_task = tokio::spawn(async move {
            while let Some(Ok(message)) = receiver.next().await {
                match message {
                    Message::Text(text) => {
                        if let Ok(ws_msg) = serde_json::from_str::<WsMessage>(&text) {
                            match ws_msg {
                                WsMessage::Ping => {
                                    // Will be handled by sender
                                }
                                WsMessage::Subscribe { entity_id } => {
                                    tracing::info!("Subscribed to entity: {}", entity_id);
                                }
                                WsMessage::Unsubscribe { entity_id } => {
                                    tracing::info!("Unsubscribed from entity: {}", entity_id);
                                }
                                _ => {}
                            }
                        }
                    }
                    Message::Close(_) => break,
                    _ => {}
                }
            }
        });

        // Handle connection close
        let _ = receive_task.await;
    }
}

/// Broadcast a message to all connected clients
pub async fn broadcast(message: WsMessage) {
    // In a real implementation, this would use a broadcast channel
    // to send messages to all connected WebSocket clients
    tracing::debug!("Broadcasting: {:?}", message);
}
