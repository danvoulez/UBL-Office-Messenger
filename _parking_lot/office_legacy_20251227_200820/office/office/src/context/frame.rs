//! Context Frame - Immutable Snapshot
//!
//! An immutable snapshot of all state relevant to an LLM entity at a point in time.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use blake3;
use hex;

use crate::entity::EntityId;
use crate::session::SessionType;
use crate::governance::Constitution;
use super::memory::Memory;

/// Hash of a context frame for verification
pub type ContextHash = String;

/// Affordance - An action the entity can take
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Affordance {
    /// Unique identifier
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Description
    pub description: String,
    /// Risk level (0.0 - 1.0)
    pub risk_score: f32,
    /// Whether simulation is required
    pub requires_simulation: bool,
    /// Parameters schema
    pub parameters: Option<serde_json::Value>,
}

/// Obligation - Something the entity must do
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Obligation {
    /// Unique identifier
    pub id: String,
    /// Description
    pub description: String,
    /// Due date (if applicable)
    pub due_at: Option<DateTime<Utc>>,
    /// Priority (1-10)
    pub priority: u8,
    /// Source (where this obligation came from)
    pub source: String,
    /// Current status
    pub status: ObligationStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObligationStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Cancelled,
}

/// Immutable context frame for an LLM instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextFrame {
    /// Entity this frame is for
    pub entity_id: EntityId,
    /// Entity name
    pub entity_name: String,
    /// Session type
    pub session_type: SessionType,
    /// Frame creation timestamp
    pub timestamp: DateTime<Utc>,
    /// Sequence number in UBL ledger
    pub ledger_sequence: u64,
    /// Memory (events, bookmarks, baseline)
    pub memory: Memory,
    /// Available affordances
    pub affordances: Vec<Affordance>,
    /// Pending obligations
    pub obligations: Vec<Obligation>,
    /// Constitution (behavioral directives)
    pub constitution: Constitution,
    /// Previous handover (if any)
    pub previous_handover: Option<String>,
    /// Governance notes (from sanity check)
    pub governance_notes: Vec<String>,
    /// Guardian information (if guarded)
    pub guardian_info: Option<GuardianInfo>,
    /// Token budget for this session
    pub token_budget: u64,
    /// Hash of this frame
    pub frame_hash: ContextHash,
}

/// Information about the entity's guardian
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardianInfo {
    pub guardian_id: String,
    pub guardian_name: String,
    pub is_available: bool,
}

impl ContextFrame {
    /// Create a new context frame (builder pattern preferred)
    pub fn new(
        entity_id: EntityId,
        entity_name: String,
        session_type: SessionType,
        ledger_sequence: u64,
        memory: Memory,
        affordances: Vec<Affordance>,
        obligations: Vec<Obligation>,
        constitution: Constitution,
        previous_handover: Option<String>,
        governance_notes: Vec<String>,
        guardian_info: Option<GuardianInfo>,
        token_budget: u64,
    ) -> Self {
        let timestamp = Utc::now();

        // Create frame without hash first
        let mut frame = Self {
            entity_id,
            entity_name,
            session_type,
            timestamp,
            ledger_sequence,
            memory,
            affordances,
            obligations,
            constitution,
            previous_handover,
            governance_notes,
            guardian_info,
            token_budget,
            frame_hash: String::new(),
        };

        // Calculate and set hash
        frame.frame_hash = frame.calculate_hash();
        frame
    }

    /// Calculate hash of this frame using BLAKE3 (consistent with UBL kernel)
    fn calculate_hash(&self) -> ContextHash {
        let mut hasher = blake3::Hasher::new();

        // Hash key components
        hasher.update(self.entity_id.as_bytes());
        hasher.update(self.timestamp.to_rfc3339().as_bytes());
        hasher.update(&self.ledger_sequence.to_le_bytes());

        // Hash memory summary
        if let Ok(memory_json) = serde_json::to_string(&self.memory) {
            hasher.update(memory_json.as_bytes());
        }

        // Hash constitution
        if let Ok(constitution_json) = serde_json::to_string(&self.constitution) {
            hasher.update(constitution_json.as_bytes());
        }

        let hash = hasher.finalize();
        format!("0x{}", hex::encode(hash.as_bytes()))
    }

    /// Verify the frame hash
    pub fn verify_hash(&self) -> bool {
        let calculated = self.calculate_hash();
        self.frame_hash == calculated
    }

    /// Get a summary of this frame (for debugging)
    pub fn summary(&self) -> FrameSummary {
        FrameSummary {
            entity_id: self.entity_id.clone(),
            session_type: self.session_type,
            timestamp: self.timestamp,
            event_count: self.memory.recent_events.len(),
            affordance_count: self.affordances.len(),
            obligation_count: self.obligations.len(),
            has_handover: self.previous_handover.is_some(),
            governance_note_count: self.governance_notes.len(),
            token_budget: self.token_budget,
            frame_hash: self.frame_hash.clone(),
        }
    }
}

/// Summary of a context frame
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameSummary {
    pub entity_id: EntityId,
    pub session_type: SessionType,
    pub timestamp: DateTime<Utc>,
    pub event_count: usize,
    pub affordance_count: usize,
    pub obligation_count: usize,
    pub has_handover: bool,
    pub governance_note_count: usize,
    pub token_budget: u64,
    pub frame_hash: ContextHash,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::memory::Memory;
    use crate::governance::Constitution;

    #[test]
    fn test_context_frame_creation() {
        let frame = ContextFrame::new(
            "entity_1".to_string(),
            "Test Entity".to_string(),
            SessionType::Work,
            100,
            Memory::default(),
            vec![],
            vec![],
            Constitution::default(),
            None,
            vec![],
            None,
            5000,
        );

        assert!(frame.frame_hash.starts_with("0x"));
        assert!(frame.verify_hash());
    }

    #[test]
    fn test_hash_verification() {
        let frame = ContextFrame::new(
            "entity_1".to_string(),
            "Test Entity".to_string(),
            SessionType::Assist,
            50,
            Memory::default(),
            vec![],
            vec![],
            Constitution::default(),
            Some("Previous work completed successfully.".to_string()),
            vec!["Note: Check facts carefully.".to_string()],
            None,
            4000,
        );

        assert!(frame.verify_hash());
    }
}
