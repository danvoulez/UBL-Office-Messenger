//! Instance - Ephemeral LLM Session
//!
//! Represents an ephemeral instance of an LLM entity executing a session.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::EntityId;
use crate::context::ContextFrame;
use crate::session::{SessionType, SessionMode, Handover};

/// Unique identifier for an instance
pub type InstanceId = String;

/// Status of an instance
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstanceStatus {
    /// Instance is being initialized
    Initializing,
    /// Instance is processing (LLM executing)
    Processing,
    /// Instance is waiting for input
    Waiting,
    /// Instance completed successfully
    Completed,
    /// Instance failed
    Failed,
    /// Instance was cancelled
    Cancelled,
}

/// An ephemeral LLM instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instance {
    /// Unique instance ID
    pub id: InstanceId,
    /// Parent entity ID
    pub entity_id: EntityId,
    /// Session type
    pub session_type: SessionType,
    /// Session mode
    pub session_mode: SessionMode,
    /// Current status
    pub status: InstanceStatus,
    /// Context frame used for this instance
    pub context_frame: Option<ContextFrame>,
    /// Tokens consumed so far
    pub tokens_consumed: u64,
    /// Token budget for this instance
    pub token_budget: u64,
    /// Messages exchanged in this instance
    pub message_count: u32,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Started processing timestamp
    pub started_at: Option<DateTime<Utc>>,
    /// Completed timestamp
    pub completed_at: Option<DateTime<Utc>>,
    /// Handover written by this instance
    pub handover: Option<Handover>,
    /// Error message if failed
    pub error: Option<String>,
}

impl Instance {
    /// Create a new instance
    pub fn new(
        entity_id: EntityId,
        session_type: SessionType,
        session_mode: SessionMode,
        token_budget: u64,
    ) -> Self {
        Self {
            id: format!("instance_{}", Uuid::new_v4()),
            entity_id,
            session_type,
            session_mode,
            status: InstanceStatus::Initializing,
            context_frame: None,
            tokens_consumed: 0,
            token_budget,
            message_count: 0,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            handover: None,
            error: None,
        }
    }

    /// Set the context frame
    pub fn set_context(&mut self, frame: ContextFrame) {
        self.context_frame = Some(frame);
    }

    /// Start processing
    pub fn start(&mut self) {
        self.status = InstanceStatus::Processing;
        self.started_at = Some(Utc::now());
    }

    /// Record token usage
    pub fn consume_tokens(&mut self, count: u64) {
        self.tokens_consumed += count;
        self.message_count += 1;
    }

    /// Check if within token budget
    pub fn within_budget(&self) -> bool {
        self.tokens_consumed < self.token_budget
    }

    /// Remaining token budget
    pub fn remaining_budget(&self) -> u64 {
        self.token_budget.saturating_sub(self.tokens_consumed)
    }

    /// Set waiting status
    pub fn wait(&mut self) {
        self.status = InstanceStatus::Waiting;
    }

    /// Complete the instance successfully
    pub fn complete(&mut self, handover: Option<Handover>) {
        self.status = InstanceStatus::Completed;
        self.completed_at = Some(Utc::now());
        self.handover = handover;
    }

    /// Fail the instance
    pub fn fail(&mut self, error: String) {
        self.status = InstanceStatus::Failed;
        self.completed_at = Some(Utc::now());
        self.error = Some(error);
    }

    /// Cancel the instance
    pub fn cancel(&mut self) {
        self.status = InstanceStatus::Cancelled;
        self.completed_at = Some(Utc::now());
    }

    /// Check if instance is still running
    pub fn is_running(&self) -> bool {
        matches!(
            self.status,
            InstanceStatus::Initializing | InstanceStatus::Processing | InstanceStatus::Waiting
        )
    }

    /// Check if instance is terminal
    pub fn is_terminal(&self) -> bool {
        matches!(
            self.status,
            InstanceStatus::Completed | InstanceStatus::Failed | InstanceStatus::Cancelled
        )
    }

    /// Get duration in milliseconds
    pub fn duration_ms(&self) -> Option<i64> {
        self.started_at.map(|start| {
            let end = self.completed_at.unwrap_or_else(Utc::now);
            (end - start).num_milliseconds()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instance_lifecycle() {
        let mut instance = Instance::new(
            "entity_1".to_string(),
            SessionType::Work,
            SessionMode::Commitment,
            5000,
        );

        assert!(instance.id.starts_with("instance_"));
        assert_eq!(instance.status, InstanceStatus::Initializing);
        assert!(instance.is_running());

        instance.start();
        assert_eq!(instance.status, InstanceStatus::Processing);

        instance.consume_tokens(1000);
        assert_eq!(instance.tokens_consumed, 1000);
        assert!(instance.within_budget());

        instance.complete(None);
        assert_eq!(instance.status, InstanceStatus::Completed);
        assert!(instance.is_terminal());
    }

    #[test]
    fn test_token_budget() {
        let mut instance = Instance::new(
            "entity_1".to_string(),
            SessionType::Assist,
            SessionMode::Deliberation,
            1000,
        );

        assert_eq!(instance.remaining_budget(), 1000);

        instance.consume_tokens(600);
        assert_eq!(instance.remaining_budget(), 400);
        assert!(instance.within_budget());

        instance.consume_tokens(500);
        assert!(!instance.within_budget());
    }
}
