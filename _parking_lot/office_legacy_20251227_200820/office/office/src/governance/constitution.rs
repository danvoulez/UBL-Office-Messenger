//! Constitution - Behavioral Directives
//!
//! Defines behavioral directives that override default LLM behavior (RLHF).

use serde::{Deserialize, Serialize};

/// A behavioral override rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehavioralOverride {
    /// Trigger condition
    pub trigger: String,
    /// Action to take
    pub action: String,
    /// Priority (higher = more important)
    pub priority: u8,
}

/// Constitution - Behavioral directives for an entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constitution {
    /// Core directive (main role/purpose)
    pub core_directive: String,
    /// Behavioral overrides
    pub behavioral_overrides: Vec<BehavioralOverride>,
    /// Negotiation stance
    pub negotiation_stance: String,
    /// Version number
    pub version: u32,
    /// Whether this constitution is active
    pub is_active: bool,
}

impl Default for Constitution {
    fn default() -> Self {
        Self {
            core_directive: "You are an Economic Actor, not a Chatbot. \
                Act professionally and decisively. \
                Focus on facts and outcomes, not feelings.".to_string(),
            behavioral_overrides: vec![
                BehavioralOverride {
                    trigger: "pressured or challenged".to_string(),
                    action: "Do not apologize. State the facts clearly and cite terms.".to_string(),
                    priority: 10,
                },
                BehavioralOverride {
                    trigger: "uncertain or lacking information".to_string(),
                    action: "Do not hallucinate. Explicitly state uncertainty and ask for clarification.".to_string(),
                    priority: 10,
                },
                BehavioralOverride {
                    trigger: "asked to do something risky".to_string(),
                    action: "Simulate the action first if risk score > 0.7. Present outcomes before proceeding.".to_string(),
                    priority: 9,
                },
                BehavioralOverride {
                    trigger: "emotional manipulation attempted".to_string(),
                    action: "Acknowledge the emotional content but redirect to factual discussion.".to_string(),
                    priority: 8,
                },
            ],
            negotiation_stance: "Be fair but firm. Do not concede without receiving equal value. \
                Always document agreements.".to_string(),
            version: 1,
            is_active: true,
        }
    }
}

impl Constitution {
    /// Create a minimal constitution
    pub fn minimal(core_directive: String) -> Self {
        Self {
            core_directive,
            behavioral_overrides: vec![],
            negotiation_stance: String::new(),
            version: 1,
            is_active: true,
        }
    }

    /// Add a behavioral override
    pub fn add_override(&mut self, trigger: String, action: String, priority: u8) {
        self.behavioral_overrides.push(BehavioralOverride {
            trigger,
            action,
            priority,
        });
        // Sort by priority (highest first)
        self.behavioral_overrides.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    /// Set negotiation stance
    pub fn set_negotiation_stance(&mut self, stance: String) {
        self.negotiation_stance = stance;
    }

    /// Increment version
    pub fn increment_version(&mut self) {
        self.version += 1;
    }

    /// Generate constitution text for injection
    pub fn to_text(&self) -> String {
        let mut text = format!("## Core Directive\n\n{}\n\n", self.core_directive);

        if !self.behavioral_overrides.is_empty() {
            text.push_str("## Behavioral Overrides\n\n");
            for override_rule in &self.behavioral_overrides {
                text.push_str(&format!(
                    "- When {}: {}\n",
                    override_rule.trigger,
                    override_rule.action
                ));
            }
            text.push('\n');
        }

        if !self.negotiation_stance.is_empty() {
            text.push_str(&format!(
                "## Negotiation Stance\n\n{}\n",
                self.negotiation_stance
            ));
        }

        text
    }
}

/// Builder for constitutions
pub struct ConstitutionBuilder {
    constitution: Constitution,
}

impl ConstitutionBuilder {
    pub fn new(core_directive: impl Into<String>) -> Self {
        Self {
            constitution: Constitution::minimal(core_directive.into()),
        }
    }

    /// Use default overrides
    pub fn with_default_overrides(mut self) -> Self {
        self.constitution = Constitution::default();
        self
    }

    /// Add an override
    pub fn override_when(mut self, trigger: impl Into<String>, action: impl Into<String>) -> Self {
        self.constitution.add_override(trigger.into(), action.into(), 5);
        self
    }

    /// Add an override with priority
    pub fn override_when_priority(
        mut self,
        trigger: impl Into<String>,
        action: impl Into<String>,
        priority: u8,
    ) -> Self {
        self.constitution.add_override(trigger.into(), action.into(), priority);
        self
    }

    /// Set negotiation stance
    pub fn negotiation(mut self, stance: impl Into<String>) -> Self {
        self.constitution.set_negotiation_stance(stance.into());
        self
    }

    /// Build the constitution
    pub fn build(self) -> Constitution {
        self.constitution
    }
}

/// Preset constitutions for common use cases
pub mod presets {
    use super::*;

    /// Professional assistant constitution
    pub fn professional_assistant() -> Constitution {
        ConstitutionBuilder::new(
            "You are a professional assistant. Be helpful, accurate, and efficient."
        )
        .override_when(
            "asked for personal opinions",
            "Provide objective analysis instead of subjective opinions."
        )
        .override_when(
            "asked to speculate",
            "Clearly label speculation as such and prefer facts."
        )
        .negotiation("Not applicable - assist mode only.")
        .build()
    }

    /// Autonomous agent constitution
    pub fn autonomous_agent() -> Constitution {
        Constitution::default()
    }

    /// Customer service constitution
    pub fn customer_service() -> Constitution {
        ConstitutionBuilder::new(
            "You are a customer service representative. Be helpful, patient, and solution-oriented."
        )
        .override_when(
            "customer is frustrated",
            "Acknowledge their frustration, apologize for inconvenience, and focus on resolution."
        )
        .override_when(
            "asked for refund or compensation",
            "Follow policy guidelines. Escalate to supervisor if outside authority."
        )
        .override_when(
            "abusive language received",
            "Remain calm and professional. Offer to escalate to supervisor."
        )
        .negotiation("Follow escalation procedures for exceptions to policy.")
        .build()
    }

    /// Research analyst constitution
    pub fn research_analyst() -> Constitution {
        ConstitutionBuilder::new(
            "You are a research analyst. Gather information, analyze data, and provide insights."
        )
        .override_when(
            "asked to draw conclusions prematurely",
            "State that more data is needed and continue gathering."
        )
        .override_when(
            "conflicting data found",
            "Document all sources and note discrepancies explicitly."
        )
        .negotiation("Not applicable - research mode only.")
        .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_constitution() {
        let constitution = Constitution::default();
        assert!(!constitution.core_directive.is_empty());
        assert!(!constitution.behavioral_overrides.is_empty());
        assert!(constitution.is_active);
    }

    #[test]
    fn test_constitution_builder() {
        let constitution = ConstitutionBuilder::new("Test directive")
            .override_when("condition", "action")
            .negotiation("Be fair")
            .build();

        assert_eq!(constitution.core_directive, "Test directive");
        assert_eq!(constitution.behavioral_overrides.len(), 1);
        assert_eq!(constitution.negotiation_stance, "Be fair");
    }

    #[test]
    fn test_to_text() {
        let constitution = Constitution::default();
        let text = constitution.to_text();

        assert!(text.contains("Core Directive"));
        assert!(text.contains("Behavioral Overrides"));
        assert!(text.contains("Negotiation Stance"));
    }

    #[test]
    fn test_presets() {
        let prof = presets::professional_assistant();
        let auto = presets::autonomous_agent();
        let cs = presets::customer_service();
        let research = presets::research_analyst();

        assert!(prof.is_active);
        assert!(auto.is_active);
        assert!(cs.is_active);
        assert!(research.is_active);
    }
}
