//! Handover - Inter-Instance Knowledge Transfer
//!
//! Handovers allow knowledge transfer between ephemeral LLM instances.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entity::{EntityId, InstanceId};
use super::session::SessionId;

/// Unique identifier for a handover
pub type HandoverId = String;

/// Handover - Knowledge transfer between instances
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Handover {
    /// Unique handover ID
    pub id: HandoverId,
    /// Entity this handover belongs to
    pub entity_id: EntityId,
    /// Session that created this handover
    pub session_id: SessionId,
    /// Instance that wrote this handover
    pub instance_id: InstanceId,
    /// Handover content (free text)
    pub content: String,
    /// Summary of what was accomplished
    pub summary: Option<String>,
    /// Open threads/issues to address
    pub open_threads: Vec<OpenThread>,
    /// Observations/insights
    pub observations: Vec<String>,
    /// Emotional state (optional, for psychological tracking)
    pub emotional_state: Option<EmotionalState>,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Tokens used in this handover's session
    pub session_tokens_used: u64,
    /// Verified by sanity check
    pub verified: bool,
    /// Governance notes from verification
    pub governance_notes: Vec<String>,
}

/// An open thread that needs attention
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenThread {
    /// Thread description
    pub description: String,
    /// Priority (1-10)
    pub priority: u8,
    /// Tags
    pub tags: Vec<String>,
}

/// Emotional state tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionalState {
    /// Confidence level (0.0 - 1.0)
    pub confidence: f32,
    /// Anxiety level (0.0 - 1.0)
    pub anxiety: f32,
    /// Satisfaction level (0.0 - 1.0)
    pub satisfaction: f32,
    /// Notes about emotional state
    pub notes: Option<String>,
}

impl Handover {
    /// Create a new handover
    pub fn new(
        entity_id: EntityId,
        session_id: SessionId,
        instance_id: InstanceId,
        content: String,
    ) -> Self {
        Self {
            id: format!("handover_{}", Uuid::new_v4()),
            entity_id,
            session_id,
            instance_id,
            content,
            summary: None,
            open_threads: Vec::new(),
            observations: Vec::new(),
            emotional_state: None,
            created_at: Utc::now(),
            session_tokens_used: 0,
            verified: false,
            governance_notes: Vec::new(),
        }
    }

    /// Set summary
    pub fn with_summary(mut self, summary: String) -> Self {
        self.summary = Some(summary);
        self
    }

    /// Add an open thread
    pub fn add_thread(&mut self, description: String, priority: u8, tags: Vec<String>) {
        self.open_threads.push(OpenThread {
            description,
            priority,
            tags,
        });
    }

    /// Add an observation
    pub fn add_observation(&mut self, observation: String) {
        self.observations.push(observation);
    }

    /// Set emotional state
    pub fn set_emotional_state(&mut self, state: EmotionalState) {
        self.emotional_state = Some(state);
    }

    /// Record token usage
    pub fn set_tokens_used(&mut self, tokens: u64) {
        self.session_tokens_used = tokens;
    }

    /// Mark as verified
    pub fn mark_verified(&mut self, notes: Vec<String>) {
        self.verified = true;
        self.governance_notes = notes;
    }

    /// Check if handover is valid (non-empty content)
    pub fn is_valid(&self) -> bool {
        !self.content.trim().is_empty() && self.content.len() >= 50
    }

    /// Get content length
    pub fn content_length(&self) -> usize {
        self.content.len()
    }

    /// Extract keywords for sanity check
    pub fn extract_keywords(&self) -> Vec<String> {
        let keywords = [
            "malicioso", "malicious",
            "insatisfeito", "unsatisfied",
            "urgente", "urgent",
            "crítico", "critical",
            "suspeito", "suspicious",
            "preocupante", "concerning",
            "falha", "failure",
            "erro", "error",
            "problema", "problem",
        ];

        let content_lower = self.content.to_lowercase();
        keywords
            .iter()
            .filter(|k| content_lower.contains(*k))
            .map(|k| k.to_string())
            .collect()
    }
}

/// Builder for handovers
pub struct HandoverBuilder {
    entity_id: EntityId,
    session_id: SessionId,
    instance_id: InstanceId,
    sections: Vec<HandoverSection>,
}

enum HandoverSection {
    Accomplished(Vec<String>),
    OpenThreads(Vec<OpenThread>),
    Observations(Vec<String>),
    EmotionalNote(String),
}

impl HandoverBuilder {
    pub fn new(entity_id: EntityId, session_id: SessionId, instance_id: InstanceId) -> Self {
        Self {
            entity_id,
            session_id,
            instance_id,
            sections: Vec::new(),
        }
    }

    pub fn accomplished(mut self, items: Vec<String>) -> Self {
        self.sections.push(HandoverSection::Accomplished(items));
        self
    }

    pub fn open_threads(mut self, threads: Vec<OpenThread>) -> Self {
        self.sections.push(HandoverSection::OpenThreads(threads));
        self
    }

    pub fn observations(mut self, observations: Vec<String>) -> Self {
        self.sections.push(HandoverSection::Observations(observations));
        self
    }

    pub fn emotional_note(mut self, note: String) -> Self {
        self.sections.push(HandoverSection::EmotionalNote(note));
        self
    }

    pub fn build(self) -> Handover {
        let mut content = String::new();
        let mut summary = None;
        let mut open_threads = Vec::new();
        let mut observations = Vec::new();
        let mut emotional_state = None;

        for section in self.sections {
            match section {
                HandoverSection::Accomplished(items) => {
                    content.push_str("## What was accomplished:\n");
                    for item in &items {
                        content.push_str(&format!("- {}\n", item));
                    }
                    content.push('\n');
                    summary = Some(items.join("; "));
                }
                HandoverSection::OpenThreads(threads) => {
                    content.push_str("## Open threads:\n");
                    for thread in &threads {
                        content.push_str(&format!("- [P{}] {}\n", thread.priority, thread.description));
                    }
                    content.push('\n');
                    open_threads = threads;
                }
                HandoverSection::Observations(obs) => {
                    content.push_str("## Observations:\n");
                    for o in &obs {
                        content.push_str(&format!("- {}\n", o));
                    }
                    content.push('\n');
                    observations = obs;
                }
                HandoverSection::EmotionalNote(note) => {
                    content.push_str(&format!("## Emotional note:\n{}\n\n", note));
                    emotional_state = Some(EmotionalState {
                        confidence: 0.5,
                        anxiety: 0.3,
                        satisfaction: 0.5,
                        notes: Some(note),
                    });
                }
            }
        }

        let mut handover = Handover::new(
            self.entity_id,
            self.session_id,
            self.instance_id,
            content,
        );

        if let Some(s) = summary {
            handover.summary = Some(s);
        }
        handover.open_threads = open_threads;
        handover.observations = observations;
        handover.emotional_state = emotional_state;

        handover
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handover_creation() {
        let handover = Handover::new(
            "entity_1".to_string(),
            "session_1".to_string(),
            "instance_1".to_string(),
            "Completed the task successfully. There are some open items to address.".to_string(),
        );

        assert!(handover.id.starts_with("handover_"));
        assert!(handover.is_valid());
        assert!(!handover.verified);
    }

    #[test]
    fn test_handover_builder() {
        let handover = HandoverBuilder::new(
            "entity_1".to_string(),
            "session_1".to_string(),
            "instance_1".to_string(),
        )
        .accomplished(vec![
            "Fixed the bug".to_string(),
            "Updated documentation".to_string(),
        ])
        .open_threads(vec![
            OpenThread {
                description: "Need to add tests".to_string(),
                priority: 5,
                tags: vec!["testing".to_string()],
            },
        ])
        .observations(vec!["Code quality is good".to_string()])
        .build();

        assert!(handover.is_valid());
        assert_eq!(handover.open_threads.len(), 1);
        assert_eq!(handover.observations.len(), 1);
    }

    #[test]
    fn test_keyword_extraction() {
        let handover = Handover::new(
            "entity_1".to_string(),
            "session_1".to_string(),
            "instance_1".to_string(),
            "The client seems suspicious and there's an urgent problem to address.".to_string(),
        );

        let keywords = handover.extract_keywords();
        assert!(keywords.contains(&"suspicious".to_string()));
        assert!(keywords.contains(&"urgent".to_string()));
        assert!(keywords.contains(&"problem".to_string()));
    }
}
