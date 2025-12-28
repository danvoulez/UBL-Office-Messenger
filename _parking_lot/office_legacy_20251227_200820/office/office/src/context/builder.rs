//! Context Frame Builder
//!
//! Builds immutable context frames from UBL ledger state.

use std::sync::Arc;


use crate::entity::{Entity, EntityId};
use crate::session::SessionType;
use crate::governance::{Constitution, SanityCheck};
use crate::ubl_client::UblClient;
use crate::Result;

use super::frame::{ContextFrame, Affordance, Obligation, GuardianInfo};
use super::memory::{Memory, MemoryConfig, MemoryEntry};

/// Builder for context frames
pub struct ContextFrameBuilder {
    entity: Entity,
    session_type: SessionType,
    ubl_client: Arc<UblClient>,
    memory_config: MemoryConfig,
    token_budget: u64,
    sanity_check: Option<SanityCheck>,
}

impl ContextFrameBuilder {
    /// Create a new builder
    pub fn new(
        entity: Entity,
        session_type: SessionType,
        ubl_client: Arc<UblClient>,
    ) -> Self {
        Self {
            entity,
            session_type,
            ubl_client,
            memory_config: MemoryConfig::default(),
            token_budget: Self::default_budget(&session_type),
            sanity_check: None,
        }
    }

    /// Default token budget by session type
    fn default_budget(session_type: &SessionType) -> u64 {
        match session_type {
            SessionType::Work => 5000,
            SessionType::Assist => 4000,
            SessionType::Deliberate => 8000,
            SessionType::Research => 6000,
        }
    }

    /// Set custom memory config
    pub fn with_memory_config(mut self, config: MemoryConfig) -> Self {
        self.memory_config = config;
        self
    }

    /// Set custom token budget
    pub fn with_token_budget(mut self, budget: u64) -> Self {
        self.token_budget = budget;
        self
    }

    /// Set sanity check instance
    pub fn with_sanity_check(mut self, sanity_check: SanityCheck) -> Self {
        self.sanity_check = Some(sanity_check);
        self
    }

    /// Build the context frame
    pub async fn build(self) -> Result<ContextFrame> {
        // 1. Query ledger state
        let ledger_state = self.ubl_client.get_state(&self.entity.id).await?;

        // 2. Query recent events
        let events = self.ubl_client
            .get_events(&self.entity.id, self.memory_config.recent_event_count)
            .await?;

        // 3. Build memory from events
        let mut memory = Memory::new(self.entity.baseline_narrative.clone());
        for event in events {
            let entry = MemoryEntry {
                event_id: event.entry_hash.clone(),
                event_type: event.intent_class.clone(),
                timestamp: event.timestamp,
                summary: event.summary.clone(),
                data: Some(event.data.clone()),
                is_bookmarked: false,
            };
            memory.add_event(entry, &self.memory_config);
        }

        // 4. Query affordances
        let affordances = self.ubl_client.get_affordances(&self.entity.id).await
            .unwrap_or_default()
            .into_iter()
            .map(|a| Affordance {
                id: a.id,
                name: a.name,
                description: a.description,
                risk_score: a.risk_score,
                requires_simulation: a.risk_score > 0.7,
                parameters: a.parameters,
            })
            .collect();

        // 5. Query obligations
        let obligations = self.ubl_client.get_obligations(&self.entity.id).await
            .unwrap_or_default()
            .into_iter()
            .map(|o| Obligation {
                id: o.id,
                description: o.description,
                due_at: o.due_at,
                priority: o.priority,
                source: o.source,
                status: super::frame::ObligationStatus::Pending,
            })
            .collect();

        // 6. Get previous handover
        let previous_handover = self.ubl_client
            .get_last_handover(&self.entity.id)
            .await
            .ok()
            .flatten();

        // 7. Apply sanity check if configured
        let governance_notes = if let Some(sanity_check) = &self.sanity_check {
            if let Some(handover) = &previous_handover {
                sanity_check.check(handover, &self.entity.id).await
                    .unwrap_or_default()
            } else {
                vec![]
            }
        } else {
            vec![]
        };

        // 8. Get guardian info
        let guardian_info = if let Some(guardian_id) = &self.entity.guardian_id {
            self.ubl_client.get_guardian(guardian_id).await.ok().map(|g| {
                GuardianInfo {
                    guardian_id: g.id,
                    guardian_name: g.name,
                    is_available: g.is_available,
                }
            })
        } else {
            None
        };

        // 9. Compress memory to fit budget if needed
        let reserved_tokens = 1000; // Reserve for system prompt, constitution, etc.
        let memory_budget = self.token_budget.saturating_sub(reserved_tokens);
        memory.compress_to_budget(memory_budget);

        // 10. Build frame
        Ok(ContextFrame::new(
            self.entity.id.clone(),
            self.entity.name.clone(),
            self.session_type,
            ledger_state.sequence,
            memory,
            affordances,
            obligations,
            self.entity.constitution.clone(),
            previous_handover,
            governance_notes,
            guardian_info,
            self.token_budget,
        ))
    }
}

/// Simplified builder for testing
pub struct TestContextFrameBuilder {
    entity_id: EntityId,
    entity_name: String,
    session_type: SessionType,
}

impl TestContextFrameBuilder {
    pub fn new(entity_id: EntityId, entity_name: String) -> Self {
        Self {
            entity_id,
            entity_name,
            session_type: SessionType::Work,
        }
    }

    pub fn with_session_type(mut self, session_type: SessionType) -> Self {
        self.session_type = session_type;
        self
    }

    pub fn build(self) -> ContextFrame {
        ContextFrame::new(
            self.entity_id,
            self.entity_name,
            self.session_type,
            0,
            Memory::default(),
            vec![],
            vec![],
            Constitution::default(),
            None,
            vec![],
            None,
            5000,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_test_builder() {
        let frame = TestContextFrameBuilder::new(
            "entity_1".to_string(),
            "Test Entity".to_string(),
        )
        .with_session_type(SessionType::Deliberate)
        .build();

        assert_eq!(frame.entity_id, "entity_1");
        assert_eq!(frame.session_type, SessionType::Deliberate);
    }
}
