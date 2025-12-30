//! Identity Event Emitter
//!
//! Helper module for emitting identity events to C.Identity container.
//! These events follow the identity_event.schema.json contract.

use serde::{Deserialize, Serialize};

use crate::{Result, OfficeError};
use super::{UblClient, CommitResponse};

/// Well-known container ID for identity events
pub const IDENTITY_CONTAINER: &str = "C.Identity";

/// Identity event kinds that can be emitted
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IdentityEventKind {
    /// Person registered in the system
    PersonRegistered,
    /// Person authenticated (login)
    PersonAuthenticated,
    /// Step-up authentication granted
    StepupGranted,
    /// Authentication attempt failed
    AuthFailed,
    /// Counter rollback detected (replay attack prevention)
    CounterRollback,
    /// Session created
    SessionCreated,
    /// Session ended
    SessionEnded,
    /// Key rotation initiated
    KeyRotation,
}

impl std::fmt::Display for IdentityEventKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IdentityEventKind::PersonRegistered => write!(f, "person_registered"),
            IdentityEventKind::PersonAuthenticated => write!(f, "person_authenticated"),
            IdentityEventKind::StepupGranted => write!(f, "stepup_granted"),
            IdentityEventKind::AuthFailed => write!(f, "auth_failed"),
            IdentityEventKind::CounterRollback => write!(f, "counter_rollback"),
            IdentityEventKind::SessionCreated => write!(f, "session_created"),
            IdentityEventKind::SessionEnded => write!(f, "session_ended"),
            IdentityEventKind::KeyRotation => write!(f, "key_rotation"),
        }
    }
}

/// Identity event payload conforming to identity_event.schema.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityEvent {
    /// Event kind
    pub kind: IdentityEventKind,
    /// Subject public key (who the event is about)
    pub subject_pubkey: String,
    /// Timestamp in milliseconds
    pub timestamp_ms: i64,
    /// Optional metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

impl IdentityEvent {
    /// Create a new identity event
    pub fn new(kind: IdentityEventKind, subject_pubkey: String) -> Self {
        Self {
            kind,
            subject_pubkey,
            timestamp_ms: chrono::Utc::now().timestamp_millis(),
            metadata: None,
        }
    }

    /// Add metadata to the event
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

/// Extension trait for UblClient to emit identity events
impl UblClient {
    /// Emit an identity event to C.Identity container
    ///
    /// # Arguments
    /// * `kind` - The type of identity event
    /// * `subject_pubkey` - The public key of the subject (person/entity)
    /// * `metadata` - Optional additional metadata
    ///
    /// # Example
    /// ```rust,ignore
    /// client.emit_identity_event(
    ///     IdentityEventKind::PersonRegistered,
    ///     &user_pubkey,
    ///     Some(json!({"email_verified": true})),
    /// ).await?;
    /// ```
    pub async fn emit_identity_event(
        &self,
        kind: IdentityEventKind,
        subject_pubkey: &str,
        metadata: Option<serde_json::Value>,
    ) -> Result<CommitResponse> {
        let mut event = IdentityEvent::new(kind, subject_pubkey.to_string());
        if let Some(meta) = metadata {
            event = event.with_metadata(meta);
        }

        let event_json = serde_json::to_value(&event)
            .map_err(|e| OfficeError::UblError(format!("Failed to serialize identity event: {}", e)))?;

        // Commit as Observation to C.Identity container
        self.commit_atom(
            IDENTITY_CONTAINER,
            &event_json,
            "observation",
            0, // No physics delta for observations
        ).await
    }

    /// Emit person_registered event
    pub async fn emit_person_registered(
        &self,
        subject_pubkey: &str,
        metadata: Option<serde_json::Value>,
    ) -> Result<CommitResponse> {
        self.emit_identity_event(IdentityEventKind::PersonRegistered, subject_pubkey, metadata).await
    }

    /// Emit person_authenticated event
    pub async fn emit_person_authenticated(
        &self,
        subject_pubkey: &str,
        metadata: Option<serde_json::Value>,
    ) -> Result<CommitResponse> {
        self.emit_identity_event(IdentityEventKind::PersonAuthenticated, subject_pubkey, metadata).await
    }

    /// Emit stepup_granted event
    pub async fn emit_stepup_granted(
        &self,
        subject_pubkey: &str,
        metadata: Option<serde_json::Value>,
    ) -> Result<CommitResponse> {
        self.emit_identity_event(IdentityEventKind::StepupGranted, subject_pubkey, metadata).await
    }

    /// Emit auth_failed event
    pub async fn emit_auth_failed(
        &self,
        subject_pubkey: &str,
        metadata: Option<serde_json::Value>,
    ) -> Result<CommitResponse> {
        self.emit_identity_event(IdentityEventKind::AuthFailed, subject_pubkey, metadata).await
    }

    /// Emit counter_rollback event (security alert)
    pub async fn emit_counter_rollback(
        &self,
        subject_pubkey: &str,
        metadata: Option<serde_json::Value>,
    ) -> Result<CommitResponse> {
        self.emit_identity_event(IdentityEventKind::CounterRollback, subject_pubkey, metadata).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_event_serialization() {
        let event = IdentityEvent::new(
            IdentityEventKind::PersonRegistered,
            "abc123".to_string(),
        );

        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json["kind"], "person_registered");
        assert_eq!(json["subject_pubkey"], "abc123");
        assert!(json["timestamp_ms"].is_i64());
    }

    #[test]
    fn test_identity_event_with_metadata() {
        let event = IdentityEvent::new(
            IdentityEventKind::PersonAuthenticated,
            "def456".to_string(),
        ).with_metadata(serde_json::json!({
            "ip_address": "192.168.1.1",
            "user_agent": "Mozilla/5.0"
        }));

        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json["metadata"]["ip_address"], "192.168.1.1");
    }

    #[test]
    fn test_identity_event_kind_display() {
        assert_eq!(IdentityEventKind::PersonRegistered.to_string(), "person_registered");
        assert_eq!(IdentityEventKind::CounterRollback.to_string(), "counter_rollback");
    }
}
