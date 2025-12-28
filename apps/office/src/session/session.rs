//! Session - Manages LLM session state
//!
//! A session represents a logical interaction between a user/system and an LLM entity.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entity::{EntityId, InstanceId};
use super::modes::{SessionType, SessionMode, SessionConfig};
use super::handover::Handover;

/// Unique identifier for a session
pub type SessionId = String;

/// Session status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    /// Session is being created
    Pending,
    /// Session is active
    Active,
    /// Session is paused
    Paused,
    /// Session completed normally
    Completed,
    /// Session was cancelled
    Cancelled,
    /// Session failed
    Failed,
}

/// A session representing an interaction with an LLM entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Unique session ID
    pub id: SessionId,
    /// Entity this session belongs to
    pub entity_id: EntityId,
    /// Current instance ID (if any)
    pub current_instance_id: Option<InstanceId>,
    /// Session type
    pub session_type: SessionType,
    /// Session mode
    pub session_mode: SessionMode,
    /// Current status
    pub status: SessionStatus,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Started timestamp
    pub started_at: Option<DateTime<Utc>>,
    /// Ended timestamp
    pub ended_at: Option<DateTime<Utc>>,
    /// Tokens consumed in this session
    pub tokens_consumed: u64,
    /// Token budget
    pub token_budget: u64,
    /// Instance count (number of instances spawned)
    pub instance_count: u32,
    /// Messages exchanged
    pub message_count: u32,
    /// Initiator (who started this session)
    pub initiator: String,
    /// Handover written at end of session
    pub handover: Option<Handover>,
    /// Error message if failed
    pub error: Option<String>,
    /// Session metadata
    pub metadata: serde_json::Value,
}

impl Session {
    /// Create a new session
    pub fn new(
        entity_id: EntityId,
        config: SessionConfig,
        initiator: String,
    ) -> Self {
        Self {
            id: format!("session_{}", Uuid::new_v4()),
            entity_id,
            current_instance_id: None,
            session_type: config.session_type,
            session_mode: config.session_mode,
            status: SessionStatus::Pending,
            created_at: Utc::now(),
            started_at: None,
            ended_at: None,
            tokens_consumed: 0,
            token_budget: config.session_type.default_budget(),
            instance_count: 0,
            message_count: 0,
            initiator,
            handover: None,
            error: None,
            metadata: serde_json::json!({}),
        }
    }

    /// Create with custom token budget
    pub fn with_budget(mut self, budget: u64) -> Self {
        self.token_budget = budget;
        self
    }

    /// Start the session
    pub fn start(&mut self) {
        self.status = SessionStatus::Active;
        self.started_at = Some(Utc::now());
    }

    /// Set current instance
    pub fn set_instance(&mut self, instance_id: InstanceId) {
        self.current_instance_id = Some(instance_id);
        self.instance_count += 1;
    }

    /// Clear current instance
    pub fn clear_instance(&mut self) {
        self.current_instance_id = None;
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

    /// Remaining budget
    pub fn remaining_budget(&self) -> u64 {
        self.token_budget.saturating_sub(self.tokens_consumed)
    }

    /// Pause the session
    pub fn pause(&mut self) {
        self.status = SessionStatus::Paused;
    }

    /// Resume the session
    pub fn resume(&mut self) {
        self.status = SessionStatus::Active;
    }

    /// Complete the session
    pub fn complete(&mut self, handover: Option<Handover>) {
        self.status = SessionStatus::Completed;
        self.ended_at = Some(Utc::now());
        self.handover = handover;
        self.current_instance_id = None;
    }

    /// Cancel the session
    pub fn cancel(&mut self) {
        self.status = SessionStatus::Cancelled;
        self.ended_at = Some(Utc::now());
        self.current_instance_id = None;
    }

    /// Fail the session
    pub fn fail(&mut self, error: String) {
        self.status = SessionStatus::Failed;
        self.ended_at = Some(Utc::now());
        self.error = Some(error);
        self.current_instance_id = None;
    }

    /// Check if session is active
    pub fn is_active(&self) -> bool {
        self.status == SessionStatus::Active
    }

    /// Check if session is terminal
    pub fn is_terminal(&self) -> bool {
        matches!(
            self.status,
            SessionStatus::Completed | SessionStatus::Cancelled | SessionStatus::Failed
        )
    }

    /// Get session duration in milliseconds
    pub fn duration_ms(&self) -> Option<i64> {
        self.started_at.map(|start| {
            let end = self.ended_at.unwrap_or_else(Utc::now);
            (end - start).num_milliseconds()
        })
    }

    /// Check if session allows binding actions
    pub fn allows_binding_actions(&self) -> bool {
        self.session_mode.is_binding() && self.session_type.allows_autonomous_action()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_lifecycle() {
        let config = SessionConfig::work_commit();
        let mut session = Session::new(
            "entity_1".to_string(),
            config,
            "user_1".to_string(),
        );

        assert!(session.id.starts_with("session_"));
        assert_eq!(session.status, SessionStatus::Pending);

        session.start();
        assert!(session.is_active());

        session.set_instance("instance_1".to_string());
        assert_eq!(session.instance_count, 1);

        session.consume_tokens(1000);
        assert_eq!(session.tokens_consumed, 1000);

        session.complete(None);
        assert!(session.is_terminal());
        assert!(session.current_instance_id.is_none());
    }

    #[test]
    fn test_token_budget() {
        let config = SessionConfig::assist_deliberate();
        let mut session = Session::new(
            "entity_1".to_string(),
            config,
            "user_1".to_string(),
        );

        assert_eq!(session.token_budget, 4000);
        assert_eq!(session.remaining_budget(), 4000);

        session.consume_tokens(3000);
        assert_eq!(session.remaining_budget(), 1000);
        assert!(session.within_budget());

        session.consume_tokens(2000);
        assert!(!session.within_budget());
    }
}
