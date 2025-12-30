//! API Module
//!
//! HTTP/WebSocket API for OFFICE.

mod http;
mod websocket;
pub mod task_routes;

pub use http::{create_router, AppState, SharedState};
pub use websocket::WebSocketHandler;
pub use task_routes::{task_router, TaskState, SharedTaskState};
