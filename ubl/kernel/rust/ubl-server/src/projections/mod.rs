//! # UBL Projections
//!
//! Projections are read-only derived state computed from ledger events.
//! They exist for query convenience but the ledger is the source of truth.
//!
//! ## Architecture
//! - Projections listen to SSE tail (LISTEN/NOTIFY)
//! - Each event updates the relevant projection table
//! - Projections can be rebuilt from scratch by replaying the ledger

mod jobs;
mod messages;
mod office;
mod rebuild;
pub mod routes;
mod job_events;
mod artifacts;
mod presence;
mod timeline;

pub use jobs::JobsProjection;
pub use messages::MessagesProjection;
pub use office::OfficeProjection;
pub use rebuild::rebuild_projections;
pub use routes::{projection_router, ProjectionState};
pub use job_events::JobEventsProjection;
pub use artifacts::ArtifactsProjection;
pub use presence::PresenceProjection;
pub use timeline::TimelineProjection;

use serde::{Deserialize, Serialize};

/// Event from the ledger that triggers projection updates
#[derive(Debug, Clone, Deserialize)]
pub struct LedgerEvent {
    pub container_id: String,
    pub sequence: i64,
    pub entry_hash: String,
    pub link_hash: String,
    pub ts_unix_ms: i64,
}

/// Atom data wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Atom {
    #[serde(rename = "type")]
    pub event_type: String,
    #[serde(flatten)]
    pub data: serde_json::Value,
}

