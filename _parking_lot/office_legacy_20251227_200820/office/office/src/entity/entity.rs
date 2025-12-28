//! Entity - Persistent LLM Identity
//!
//! Represents a persistent LLM identity that can spawn ephemeral instances.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::identity::Identity;
use super::guardian::GuardianId;
use crate::governance::Constitution;
use crate::Result;

/// Unique identifier for an entity
pub type EntityId = String;

/// Type of entity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityType {
    /// Autonomous LLM entity
    Autonomous,
    /// Entity under guardian supervision
    Guarded,
    /// Development/testing entity
    Development,
}

/// Status of an entity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityStatus {
    /// Entity is active and can spawn instances
    Active,
    /// Entity is suspended
    Suspended,
    /// Entity is archived (soft deleted)
    Archived,
}

/// Parameters for creating a new entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityParams {
    /// Display name for the entity
    pub name: String,
    /// Type of entity
    pub entity_type: EntityType,
    /// Optional guardian ID for guarded entities
    pub guardian_id: Option<GuardianId>,
    /// Initial constitution (behavioral directives)
    pub constitution: Option<Constitution>,
    /// Initial baseline narrative
    pub baseline_narrative: Option<String>,
    /// Metadata
    pub metadata: Option<serde_json::Value>,
}

/// An LLM Entity - persistent identity with cryptographic keys
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    /// Unique identifier
    pub id: EntityId,
    /// Display name
    pub name: String,
    /// Entity type
    pub entity_type: EntityType,
    /// Current status
    pub status: EntityStatus,
    /// Cryptographic identity (Ed25519 keypair)
    pub identity: Identity,
    /// Guardian ID (for guarded entities)
    pub guardian_id: Option<GuardianId>,
    /// Current constitution
    pub constitution: Constitution,
    /// Baseline narrative (consolidated from dreaming)
    pub baseline_narrative: String,
    /// Total sessions spawned
    pub total_sessions: u64,
    /// Total tokens consumed
    pub total_tokens_consumed: u64,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last activity timestamp
    pub last_active_at: DateTime<Utc>,
    /// Last dreaming cycle timestamp
    pub last_dream_at: Option<DateTime<Utc>>,
    /// Metadata
    pub metadata: serde_json::Value,
}

impl Entity {
    /// Create a new entity
    pub fn new(params: EntityParams) -> Result<Self> {
        let id = format!("entity_{}", Uuid::new_v4());
        let identity = Identity::generate()?;
        let now = Utc::now();

        Ok(Self {
            id,
            name: params.name,
            entity_type: params.entity_type,
            status: EntityStatus::Active,
            identity,
            guardian_id: params.guardian_id,
            constitution: params.constitution.unwrap_or_default(),
            baseline_narrative: params.baseline_narrative.unwrap_or_else(|| {
                "You are a new LLM entity. Your history will be built through interactions.".to_string()
            }),
            total_sessions: 0,
            total_tokens_consumed: 0,
            created_at: now,
            last_active_at: now,
            last_dream_at: None,
            metadata: params.metadata.unwrap_or(serde_json::json!({})),
        })
    }

    /// Get the entity's public key (hex encoded)
    pub fn public_key(&self) -> &str {
        &self.identity.public_key_hex
    }

    /// Update the constitution
    pub fn update_constitution(&mut self, constitution: Constitution) {
        self.constitution = constitution;
    }

    /// Update the baseline narrative
    pub fn update_baseline(&mut self, narrative: String) {
        self.baseline_narrative = narrative;
    }

    /// Record a session completion
    pub fn record_session(&mut self, tokens_used: u64) {
        self.total_sessions += 1;
        self.total_tokens_consumed += tokens_used;
        self.last_active_at = Utc::now();
    }

    /// Record a dreaming cycle completion
    pub fn record_dream(&mut self) {
        self.last_dream_at = Some(Utc::now());
    }

    /// Check if entity is active
    pub fn is_active(&self) -> bool {
        self.status == EntityStatus::Active
    }

    /// Suspend the entity
    pub fn suspend(&mut self) {
        self.status = EntityStatus::Suspended;
    }

    /// Reactivate the entity
    pub fn activate(&mut self) {
        self.status = EntityStatus::Active;
    }

    /// Archive the entity (soft delete)
    pub fn archive(&mut self) {
        self.status = EntityStatus::Archived;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_entity() {
        let params = EntityParams {
            name: "Test Entity".to_string(),
            entity_type: EntityType::Development,
            guardian_id: None,
            constitution: None,
            baseline_narrative: None,
            metadata: None,
        };

        let entity = Entity::new(params).unwrap();
        assert!(entity.id.starts_with("entity_"));
        assert_eq!(entity.name, "Test Entity");
        assert_eq!(entity.entity_type, EntityType::Development);
        assert!(entity.is_active());
    }

    #[test]
    fn test_entity_lifecycle() {
        let params = EntityParams {
            name: "Lifecycle Test".to_string(),
            entity_type: EntityType::Guarded,
            guardian_id: Some("guardian_1".to_string()),
            constitution: None,
            baseline_narrative: None,
            metadata: None,
        };

        let mut entity = Entity::new(params).unwrap();
        assert!(entity.is_active());

        entity.suspend();
        assert!(!entity.is_active());

        entity.activate();
        assert!(entity.is_active());

        entity.record_session(1000);
        assert_eq!(entity.total_sessions, 1);
        assert_eq!(entity.total_tokens_consumed, 1000);
    }
}
