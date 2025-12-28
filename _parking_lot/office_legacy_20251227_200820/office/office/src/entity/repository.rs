//! Entity Repository - Persistent Chair Storage via UBL
//!
//! Stores and retrieves Entity (Chair) data from UBL ledger.
//! The Chair's soul lives in UBL. This module manages it.

use std::sync::Arc;
use std::collections::HashMap;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::entity::{Entity, EntityId, EntityParams, EntityType, EntityStatus};
use crate::governance::Constitution;
use crate::ubl_client::{UblClient, LinkCommit};
use crate::{OfficeError, Result};

/// Event types for entity lifecycle
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EntityEvent {
    /// Entity created
    EntityCreated {
        entity_id: EntityId,
        name: String,
        entity_type: EntityType,
        public_key: String,
        constitution: Constitution,
    },
    /// Constitution updated
    ConstitutionUpdated {
        entity_id: EntityId,
        constitution: Constitution,
    },
    /// Baseline updated (from dreaming)
    BaselineUpdated {
        entity_id: EntityId,
        baseline: String,
    },
    /// Session completed
    SessionCompleted {
        entity_id: EntityId,
        session_id: String,
        tokens_used: u64,
        handover: Option<String>,
    },
    /// Entity suspended
    EntitySuspended {
        entity_id: EntityId,
        reason: String,
    },
    /// Entity reactivated
    EntityActivated {
        entity_id: EntityId,
    },
    /// Entity archived
    EntityArchived {
        entity_id: EntityId,
    },
}

/// Entity Repository - The Chair's Keeper
pub struct EntityRepository {
    ubl_client: Arc<UblClient>,
    /// In-memory cache (projection from ledger)
    cache: RwLock<HashMap<EntityId, Entity>>,
    /// Container for entity events
    container_id: String,
}

impl EntityRepository {
    /// Create a new entity repository
    pub fn new(ubl_client: Arc<UblClient>, container_id: &str) -> Self {
        Self {
            ubl_client,
            cache: RwLock::new(HashMap::new()),
            container_id: container_id.to_string(),
        }
    }

    /// Get or create an entity (The Chair)
    pub async fn get_or_create(&self, entity_id: &EntityId, params: EntityParams) -> Result<Entity> {
        // Try cache first
        {
            let cache = self.cache.read().await;
            if let Some(entity) = cache.get(entity_id) {
                return Ok(entity.clone());
            }
        }

        // Try to load from UBL
        if let Ok(entity) = self.load_from_ledger(entity_id).await {
            // Cache it
            let mut cache = self.cache.write().await;
            cache.insert(entity_id.clone(), entity.clone());
            return Ok(entity);
        }

        // Create new entity
        let entity = self.create_entity(params).await?;
        
        // Cache it
        {
            let mut cache = self.cache.write().await;
            cache.insert(entity.id.clone(), entity.clone());
        }

        Ok(entity)
    }

    /// Get an entity by ID
    pub async fn get(&self, entity_id: &EntityId) -> Result<Entity> {
        // Check cache
        {
            let cache = self.cache.read().await;
            if let Some(entity) = cache.get(entity_id) {
                return Ok(entity.clone());
            }
        }

        // Load from ledger
        let entity = self.load_from_ledger(entity_id).await?;
        
        // Cache it
        {
            let mut cache = self.cache.write().await;
            cache.insert(entity_id.clone(), entity.clone());
        }

        Ok(entity)
    }

    /// Create a new entity
    pub async fn create_entity(&self, params: EntityParams) -> Result<Entity> {
        let entity = Entity::new(params.clone())?;

        // Publish entity.created event to UBL
        let event = EntityEvent::EntityCreated {
            entity_id: entity.id.clone(),
            name: params.name,
            entity_type: params.entity_type,
            public_key: entity.public_key().to_string(),
            constitution: entity.constitution.clone(),
        };

        self.publish_event(&entity.id, event).await?;

        Ok(entity)
    }

    /// Update entity constitution
    pub async fn update_constitution(&self, entity_id: &EntityId, constitution: Constitution) -> Result<()> {
        // Update cache
        {
            let mut cache = self.cache.write().await;
            if let Some(entity) = cache.get_mut(entity_id) {
                entity.update_constitution(constitution.clone());
            }
        }

        // Publish event
        let event = EntityEvent::ConstitutionUpdated {
            entity_id: entity_id.clone(),
            constitution,
        };

        self.publish_event(entity_id, event).await
    }

    /// Update entity baseline (from dreaming)
    pub async fn update_baseline(&self, entity_id: &EntityId, baseline: String) -> Result<()> {
        // Update cache
        {
            let mut cache = self.cache.write().await;
            if let Some(entity) = cache.get_mut(entity_id) {
                entity.update_baseline(baseline.clone());
            }
        }

        // Publish event
        let event = EntityEvent::BaselineUpdated {
            entity_id: entity_id.clone(),
            baseline,
        };

        self.publish_event(entity_id, event).await
    }

    /// Record a completed session
    pub async fn record_session(
        &self, 
        entity_id: &EntityId, 
        session_id: &str,
        tokens_used: u64,
        handover: Option<String>,
    ) -> Result<()> {
        // Update cache
        {
            let mut cache = self.cache.write().await;
            if let Some(entity) = cache.get_mut(entity_id) {
                entity.record_session(tokens_used);
            }
        }

        // Publish event
        let event = EntityEvent::SessionCompleted {
            entity_id: entity_id.clone(),
            session_id: session_id.to_string(),
            tokens_used,
            handover,
        };

        self.publish_event(entity_id, event).await
    }

    /// Load entity from ledger events (projection)
    async fn load_from_ledger(&self, entity_id: &EntityId) -> Result<Entity> {
        // Get all events for this entity
        let events = self.ubl_client.get_events(entity_id, 1000).await?;

        if events.is_empty() {
            return Err(OfficeError::EntityNotFound(entity_id.clone()));
        }

        // Replay events to build entity state
        let mut entity: Option<Entity> = None;
        let mut total_sessions = 0u64;
        let mut total_tokens = 0u64;

        for event in events {
            if let Ok(parsed) = serde_json::from_value::<EntityEvent>(event.data) {
                match parsed {
                    EntityEvent::EntityCreated { entity_id: id, name, entity_type, public_key, constitution } => {
                        // Create base entity (without regenerating keys)
                        let mut e = Entity::new(EntityParams {
                            name,
                            entity_type,
                            guardian_id: None,
                            constitution: Some(constitution),
                            baseline_narrative: None,
                            metadata: None,
                        })?;
                        // Note: In production, we'd restore the actual keys from secure storage
                        entity = Some(e);
                    }
                    EntityEvent::ConstitutionUpdated { constitution, .. } => {
                        if let Some(ref mut e) = entity {
                            e.update_constitution(constitution);
                        }
                    }
                    EntityEvent::BaselineUpdated { baseline, .. } => {
                        if let Some(ref mut e) = entity {
                            e.update_baseline(baseline);
                        }
                    }
                    EntityEvent::SessionCompleted { tokens_used, .. } => {
                        total_sessions += 1;
                        total_tokens += tokens_used;
                    }
                    EntityEvent::EntitySuspended { .. } => {
                        if let Some(ref mut e) = entity {
                            e.suspend();
                        }
                    }
                    EntityEvent::EntityActivated { .. } => {
                        if let Some(ref mut e) = entity {
                            e.activate();
                        }
                    }
                    EntityEvent::EntityArchived { .. } => {
                        if let Some(ref mut e) = entity {
                            e.archive();
                        }
                    }
                }
            }
        }

        match entity {
            Some(mut e) => {
                e.total_sessions = total_sessions;
                e.total_tokens_consumed = total_tokens;
                Ok(e)
            }
            None => Err(OfficeError::EntityNotFound(entity_id.clone())),
        }
    }

    /// Publish an event to UBL
    async fn publish_event(&self, entity_id: &EntityId, event: EntityEvent) -> Result<()> {
        // Canonicalize the event
        let event_json = serde_json::to_value(&event)?;
        let canonical = ubl_atom::canonicalize(&event_json)
            .map_err(|e| OfficeError::UblError(format!("Canonicalize failed: {}", e)))?;
        
        // Hash the atom
        let atom_hash = ubl_kernel::hash_atom(&canonical);

        // Get current state
        let state = self.ubl_client.get_state(entity_id).await?;

        // Build link commit
        // Note: In production, we'd sign with the entity's actual key
        let link = LinkCommit {
            version: 1,
            container_id: self.container_id.clone(),
            expected_sequence: state.sequence + 1,
            previous_hash: state.last_hash,
            atom_hash,
            intent_class: "observation".to_string(),
            physics_delta: 0,
            pact: None,
            author_pubkey: "office".to_string(), // TODO: Use actual key
            signature: "mock".to_string(), // TODO: Sign properly
        };

        // Commit to ledger
        self.ubl_client.commit(link).await?;

        Ok(())
    }

    /// Clear the cache (for testing)
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_event_serialization() {
        let event = EntityEvent::EntityCreated {
            entity_id: "entity_123".to_string(),
            name: "Test Entity".to_string(),
            entity_type: EntityType::Autonomous,
            public_key: "abc123".to_string(),
            constitution: Constitution::default(),
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("entity_created"));
        assert!(json.contains("entity_123"));
    }
}

