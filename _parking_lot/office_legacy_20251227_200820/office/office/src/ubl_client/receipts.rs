//! Cryptographic Receipts

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A cryptographic receipt for a ledger operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Receipt {
    /// Receipt ID
    pub id: String,
    /// Entry hash this receipt is for
    pub entry_hash: String,
    /// Container ID
    pub container_id: String,
    /// Sequence number
    pub sequence: u64,
    /// Receipt timestamp
    pub timestamp: DateTime<Utc>,
    /// Signature
    pub signature: String,
    /// Verification data
    pub verification: ReceiptVerification,
}

/// Verification data for a receipt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptVerification {
    /// Previous hash (for chain verification)
    pub previous_hash: String,
    /// Merkle proof (if applicable)
    pub merkle_proof: Option<Vec<String>>,
    /// Anchor root (if anchored to external chain)
    pub anchor_root: Option<String>,
}

impl Receipt {
    /// Verify the receipt chain
    pub fn verify_chain(&self, expected_previous: &str) -> bool {
        self.verification.previous_hash == expected_previous
    }
}
