//! API Module
//!
//! HTTP/WebSocket API for OFFICE.

mod http;
mod websocket;

pub use http::{create_router, AppState};
pub use websocket::WebSocketHandler;
