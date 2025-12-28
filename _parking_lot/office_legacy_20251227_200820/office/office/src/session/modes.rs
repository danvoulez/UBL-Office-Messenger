//! Session Types and Modes
//!
//! Defines the different types of sessions and their execution modes.

use serde::{Deserialize, Serialize};

/// Session Type - Classifies the kind of interaction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionType {
    /// Autonomous work session - full authority to act
    /// Token budget: 5000
    Work,

    /// Assist session - helping a human with a task
    /// Token budget: 4000
    Assist,

    /// Deliberation session - explore options, don't commit
    /// Token budget: 8000
    Deliberate,

    /// Research session - gather information, don't conclude
    /// Token budget: 6000
    Research,
}

impl SessionType {
    /// Get the default token budget for this session type
    pub fn default_budget(&self) -> u64 {
        match self {
            SessionType::Work => 5000,
            SessionType::Assist => 4000,
            SessionType::Deliberate => 8000,
            SessionType::Research => 6000,
        }
    }

    /// Get a human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            SessionType::Work => "autonomous work session with full authority",
            SessionType::Assist => "assist session helping a human",
            SessionType::Deliberate => "deliberation session for exploring options",
            SessionType::Research => "research session for gathering information",
        }
    }

    /// Check if this session type allows autonomous action
    pub fn allows_autonomous_action(&self) -> bool {
        matches!(self, SessionType::Work)
    }

    /// Check if this session type requires human confirmation
    pub fn requires_confirmation(&self) -> bool {
        matches!(self, SessionType::Assist)
    }

    /// Check if this session type is read-only
    pub fn is_read_only(&self) -> bool {
        matches!(self, SessionType::Research)
    }
}

/// Session Mode - Determines binding nature of actions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionMode {
    /// Commitment mode - actions are signed and binding
    Commitment,

    /// Deliberation mode - actions are drafts, not binding
    Deliberation,
}

impl SessionMode {
    /// Check if actions in this mode are binding
    pub fn is_binding(&self) -> bool {
        matches!(self, SessionMode::Commitment)
    }

    /// Get a human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            SessionMode::Commitment => "commitment mode - actions are binding",
            SessionMode::Deliberation => "deliberation mode - actions are drafts",
        }
    }
}

/// Combined session configuration
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SessionConfig {
    pub session_type: SessionType,
    pub session_mode: SessionMode,
}

impl SessionConfig {
    /// Work session with commitment (full autonomous authority)
    pub fn work_commit() -> Self {
        Self {
            session_type: SessionType::Work,
            session_mode: SessionMode::Commitment,
        }
    }

    /// Assist session with deliberation (help without binding)
    pub fn assist_deliberate() -> Self {
        Self {
            session_type: SessionType::Assist,
            session_mode: SessionMode::Deliberation,
        }
    }

    /// Deliberation session (exploring, never binding)
    pub fn deliberate() -> Self {
        Self {
            session_type: SessionType::Deliberate,
            session_mode: SessionMode::Deliberation,
        }
    }

    /// Research session (information gathering)
    pub fn research() -> Self {
        Self {
            session_type: SessionType::Research,
            session_mode: SessionMode::Deliberation,
        }
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
    fn test_session_type_budgets() {
        assert_eq!(SessionType::Work.default_budget(), 5000);
        assert_eq!(SessionType::Assist.default_budget(), 4000);
        assert_eq!(SessionType::Deliberate.default_budget(), 8000);
        assert_eq!(SessionType::Research.default_budget(), 6000);
    }

    #[test]
    fn test_session_type_properties() {
        assert!(SessionType::Work.allows_autonomous_action());
        assert!(!SessionType::Assist.allows_autonomous_action());

        assert!(SessionType::Assist.requires_confirmation());
        assert!(!SessionType::Work.requires_confirmation());

        assert!(SessionType::Research.is_read_only());
        assert!(!SessionType::Deliberate.is_read_only());
    }

    #[test]
    fn test_session_mode() {
        assert!(SessionMode::Commitment.is_binding());
        assert!(!SessionMode::Deliberation.is_binding());
    }

    #[test]
    fn test_session_config() {
        let work = SessionConfig::work_commit();
        assert!(work.allows_binding_actions());

        let assist = SessionConfig::assist_deliberate();
        assert!(!assist.allows_binding_actions());

        let deliberate = SessionConfig::deliberate();
        assert!(!deliberate.allows_binding_actions());
    }
}
