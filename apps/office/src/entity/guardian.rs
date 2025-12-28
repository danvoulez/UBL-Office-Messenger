//! Guardian - Supervisor for Guarded Entities
//!
//! Guardians are human or autonomous entities that supervise guarded LLM entities.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::EntityId;

/// Unique identifier for a guardian
pub type GuardianId = String;

/// Type of guardian
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GuardianType {
    /// Human guardian
    Human,
    /// Autonomous LLM guardian
    Autonomous,
    /// System/automated guardian
    System,
}

/// A guardian that supervises guarded entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Guardian {
    /// Unique identifier
    pub id: GuardianId,
    /// Display name
    pub name: String,
    /// Guardian type
    pub guardian_type: GuardianType,
    /// Associated entity ID (if guardian is also an entity)
    pub entity_id: Option<EntityId>,
    /// List of entities under guardianship
    pub guarded_entities: Vec<EntityId>,
    /// Whether guardian is active
    pub is_active: bool,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Last activity timestamp
    pub last_active_at: DateTime<Utc>,
    /// Contact information (for human guardians)
    pub contact: Option<GuardianContact>,
    /// Notification preferences
    pub notifications: GuardianNotifications,
}

/// Contact information for guardians
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardianContact {
    pub email: Option<String>,
    pub phone: Option<String>,
    pub webhook_url: Option<String>,
}

/// Notification preferences for guardians
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardianNotifications {
    /// Notify on high-risk actions
    pub on_high_risk_action: bool,
    /// Notify on session start
    pub on_session_start: bool,
    /// Notify on session end
    pub on_session_end: bool,
    /// Notify on error
    pub on_error: bool,
    /// Notify on dreaming cycle
    pub on_dreaming_cycle: bool,
}

impl Default for GuardianNotifications {
    fn default() -> Self {
        Self {
            on_high_risk_action: true,
            on_session_start: false,
            on_session_end: false,
            on_error: true,
            on_dreaming_cycle: false,
        }
    }
}

impl Guardian {
    /// Create a new human guardian
    pub fn new_human(id: GuardianId, name: String, contact: GuardianContact) -> Self {
        let now = Utc::now();
        Self {
            id,
            name,
            guardian_type: GuardianType::Human,
            entity_id: None,
            guarded_entities: Vec::new(),
            is_active: true,
            created_at: now,
            last_active_at: now,
            contact: Some(contact),
            notifications: GuardianNotifications::default(),
        }
    }

    /// Create a new autonomous guardian (another LLM entity)
    pub fn new_autonomous(id: GuardianId, name: String, entity_id: EntityId) -> Self {
        let now = Utc::now();
        Self {
            id,
            name,
            guardian_type: GuardianType::Autonomous,
            entity_id: Some(entity_id),
            guarded_entities: Vec::new(),
            is_active: true,
            created_at: now,
            last_active_at: now,
            contact: None,
            notifications: GuardianNotifications::default(),
        }
    }

    /// Add an entity to guardianship
    pub fn add_entity(&mut self, entity_id: EntityId) {
        if !self.guarded_entities.contains(&entity_id) {
            self.guarded_entities.push(entity_id);
        }
    }

    /// Remove an entity from guardianship
    pub fn remove_entity(&mut self, entity_id: &str) {
        self.guarded_entities.retain(|id| id != entity_id);
    }

    /// Check if guardian supervises an entity
    pub fn supervises(&self, entity_id: &str) -> bool {
        self.guarded_entities.iter().any(|id| id == entity_id)
    }

    /// Record activity
    pub fn record_activity(&mut self) {
        self.last_active_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_human_guardian() {
        let contact = GuardianContact {
            email: Some("guardian@example.com".to_string()),
            phone: None,
            webhook_url: None,
        };

        let mut guardian = Guardian::new_human(
            "guardian_1".to_string(),
            "John Doe".to_string(),
            contact,
        );

        assert_eq!(guardian.guardian_type, GuardianType::Human);
        assert!(guardian.is_active);

        guardian.add_entity("entity_1".to_string());
        guardian.add_entity("entity_2".to_string());

        assert!(guardian.supervises("entity_1"));
        assert!(!guardian.supervises("entity_3"));

        guardian.remove_entity("entity_1");
        assert!(!guardian.supervises("entity_1"));
    }

    #[test]
    fn test_autonomous_guardian() {
        let guardian = Guardian::new_autonomous(
            "guardian_auto".to_string(),
            "Supervisor Entity".to_string(),
            "entity_supervisor".to_string(),
        );

        assert_eq!(guardian.guardian_type, GuardianType::Autonomous);
        assert!(guardian.entity_id.is_some());
    }
}
