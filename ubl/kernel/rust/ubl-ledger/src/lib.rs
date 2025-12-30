//! # UBL Ledger
//!
//! Immutable append-only memory. The history IS the truth.
//! Implements SPEC-UBL-LEDGER v1.0
//!
//! ## Properties
//! - Append-only (no UPDATE, no DELETE)
//! - Hash chain (each entry links to previous)
//! - State is always a projection of history
//! - Merkle root for daily anchoring

#![deny(unsafe_code)]
#![warn(missing_docs)]

use serde::{Deserialize, Serialize};
use thiserror::Error;
use ubl_link::{IntentClass, LinkCommit, LinkReceipt};

/// Errors from ledger operations
#[derive(Error, Debug)]
pub enum LedgerError {
    /// Sequence mismatch
    #[error("Sequence mismatch: expected {expected}, got {actual}")]
    SequenceMismatch { expected: u64, actual: u64 },

    /// Hash chain broken
    #[error("Reality drift: expected previous_hash {expected}, got {actual}")]
    RealityDrift { expected: String, actual: String },

    /// Container ID mismatch
    #[error("Container mismatch: expected {expected}, got {actual}")]
    ContainerMismatch { expected: String, actual: String },
}

/// Result type for ledger operations
pub type Result<T> = std::result::Result<T, LedgerError>;

/// A single entry in the ledger
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerEntry {
    /// Sequence number (1-indexed)
    pub sequence: u64,
    /// Hash of this entry
    pub entry_hash: String,
    /// The original commit
    pub link: LinkCommit,
    /// Unix timestamp of acceptance
    pub timestamp: i64,
}

/// The immutable ledger for a container
pub struct Ledger {
    container_id: String,
    chain: Vec<LedgerEntry>,
}

/// Genesis hash constant
pub const GENESIS_HASH: &str = "0000000000000000000000000000000000000000000000000000000000000000";

/// Compute entry_hash deterministically (server-side only)
/// 
/// Per SPEC-UBL-LEDGER v1.0: entry_hash is computed by the server, never accepted from client.
/// Formula: BLAKE3(container_id || sequence || link_hash || previous_hash || ts_unix_ms)
pub fn compute_entry_hash(
    container_id: &str,
    sequence: u64,
    link_hash: &str,
    previous_hash: &str,
    ts_unix_ms: i128,
) -> String {
    use blake3::Hasher;
    let mut h = Hasher::new();
    h.update(container_id.as_bytes());
    h.update(&sequence.to_be_bytes());
    h.update(link_hash.as_bytes());
    h.update(previous_hash.as_bytes());
    h.update(&ts_unix_ms.to_be_bytes());
    hex::encode(h.finalize().as_bytes())
}

impl Ledger {
    /// Create a new ledger for a container
    pub fn new(container_id: String) -> Self {
        Self {
            container_id,
            chain: Vec::new(),
        }
    }

    /// Get the container ID
    pub fn container_id(&self) -> &str {
        &self.container_id
    }

    /// Get the hash of the last entry (or genesis hash)
    pub fn last_hash(&self) -> String {
        self.chain
            .last()
            .map(|e| e.entry_hash.clone())
            .unwrap_or_else(|| GENESIS_HASH.to_string())
    }

    /// Get the next expected sequence number
    pub fn next_sequence(&self) -> u64 {
        (self.chain.len() as u64) + 1
    }

    /// Get the current sequence (0 if empty)
    pub fn current_sequence(&self) -> u64 {
        self.chain.len() as u64
    }

    /// Get the physical balance (sum of all deltas)
    pub fn physical_balance(&self) -> i128 {
        self.chain.iter().map(|e| e.link.physics_delta).sum()
    }

    /// Append a validated commit to the ledger
    /// NOTE: Validation should be done by the membrane before calling this
    pub fn append(&mut self, link: LinkCommit, entry_hash: String) -> LinkReceipt {
        let sequence = self.next_sequence();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0); // Before Unix epoch should never happen

        let entry = LedgerEntry {
            sequence,
            entry_hash: entry_hash.clone(),
            link,
            timestamp,
        };

        self.chain.push(entry);

        LinkReceipt {
            entry_hash,
            sequence,
            timestamp,
            container_id: self.container_id.clone(),
        }
    }

    /// Get all entries in the chain
    pub fn entries(&self) -> &[LedgerEntry] {
        &self.chain
    }

    /// Get an entry by sequence number
    pub fn get_entry(&self, sequence: u64) -> Option<&LedgerEntry> {
        if sequence == 0 || sequence > self.chain.len() as u64 {
            return None;
        }
        Some(&self.chain[(sequence - 1) as usize])
    }

    /// Calculate merkle root of all entries (simplified version)
    pub fn merkle_root_hex(&self) -> String {
        if self.chain.is_empty() {
            return "0".repeat(64);
        }
        // Simplified: return last hash
        self.last_hash()
    }
}

/// State projection from ledger (derived, not stored)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerState {
    /// Container ID
    pub container_id: String,
    /// Current sequence
    pub sequence: u64,
    /// Last entry hash
    pub last_hash: String,
    /// Physical balance
    pub physical_balance: i128,
    /// Merkle root
    pub merkle_root: String,
}

impl From<&Ledger> for LedgerState {
    fn from(ledger: &Ledger) -> Self {
        Self {
            container_id: ledger.container_id().to_string(),
            sequence: ledger.current_sequence(),
            last_hash: ledger.last_hash(),
            physical_balance: ledger.physical_balance(),
            merkle_root: ledger.merkle_root_hex(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_commit(seq: u64, prev_hash: &str, delta: i128) -> LinkCommit {
        LinkCommit {
            version: 1,
            container_id: "test".to_string(),
            expected_sequence: seq,
            previous_hash: prev_hash.to_string(),
            atom_hash: "atom".to_string(),
            intent_class: IntentClass::Conservation,
            physics_delta: delta,
            pact: None,
            author_pubkey: "pk".to_string(),
            signature: "sig".to_string(),
        }
    }

    #[test]
    fn test_new_ledger() {
        let ledger = Ledger::new("wallet_alice".to_string());
        assert_eq!(ledger.container_id(), "wallet_alice");
        assert_eq!(ledger.next_sequence(), 1);
        assert_eq!(ledger.last_hash(), GENESIS_HASH);
        assert_eq!(ledger.physical_balance(), 0);
    }

    #[test]
    fn test_append() {
        let mut ledger = Ledger::new("wallet".to_string());
        let commit = make_commit(1, GENESIS_HASH, 100);

        let receipt = ledger.append(commit, "hash1".to_string());

        assert_eq!(receipt.sequence, 1);
        assert_eq!(ledger.next_sequence(), 2);
        assert_eq!(ledger.physical_balance(), 100);
    }

    #[test]
    fn test_chain() {
        let mut ledger = Ledger::new("wallet".to_string());

        let commit1 = make_commit(1, GENESIS_HASH, 100);
        let receipt1 = ledger.append(commit1, "hash1".to_string());

        let commit2 = make_commit(2, &receipt1.entry_hash, -30);
        ledger.append(commit2, "hash2".to_string());

        assert_eq!(ledger.current_sequence(), 2);
        assert_eq!(ledger.physical_balance(), 70);
    }

    #[test]
    fn test_state_projection() {
        let mut ledger = Ledger::new("wallet".to_string());
        let commit = make_commit(1, GENESIS_HASH, 50);
        ledger.append(commit, "hash1".to_string());

        let state: LedgerState = (&ledger).into();

        assert_eq!(state.container_id, "wallet");
        assert_eq!(state.sequence, 1);
        assert_eq!(state.physical_balance, 50);
    }
}