//! Ledger State and Events

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// State of a ledger (container)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LedgerState {
    /// Container ID
    pub container_id: String,
    /// Current sequence number
    pub sequence: u64,
    /// Last entry hash
    pub last_hash: String,
    /// Physical balance
    pub physical_balance: i64,
    /// Merkle root (for verification)
    pub merkle_root: String,
}

/// An event from the ledger
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerEvent {
    /// Entry hash
    pub entry_hash: String,
    /// Sequence number
    pub sequence: u64,
    /// Intent class (Observation, Conservation, etc.)
    pub intent_class: String,
    /// Event timestamp
    pub timestamp: DateTime<Utc>,
    /// Event summary
    pub summary: String,
    /// Full event data
    pub data: serde_json::Value,
    /// Author public key
    pub author_pubkey: String,
}
