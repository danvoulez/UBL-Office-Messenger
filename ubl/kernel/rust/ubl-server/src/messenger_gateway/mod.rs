//! Messenger Gateway v1
//!
//! Thin gateway layer between frontend and UBL/Office.
//! Handles command routing, idempotency, projection management, and SSE delta emission.
//!
//! Architecture:
//! - Frontend → Gateway → Office → UBL
//! - Gateway subscribes to UBL SSE tail → updates projections → emits SSE deltas to frontend

pub mod routes;
pub mod projections;
pub mod sse;
pub mod idempotency;
pub mod office_client;

pub use routes::{routes, GatewayState};

