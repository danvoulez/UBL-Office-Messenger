//! Ledger State and Events

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// State of a ledger (container)
/// Aligned with UBL Kernel response format
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LedgerState {
    /// Container ID
    pub container_id: String,
    /// Current sequence number
    pub sequence: u64,
    /// Last entry hash
    pub last_hash: String,
    /// Entry count (from UBL)
    #[serde(default)]
    pub entry_count: u64,
    /// Physical balance (optional - computed from projections)
    #[serde(default)]
    pub physical_balance: i64,
    /// Merkle root (optional - for verification)
    #[serde(default)]
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
