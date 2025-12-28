//! Job Finite State Machine
//!
//! From the spec:
//! > Minimum states: draft → proposed → approved → in_progress → (waiting_input ↔ in_progress) → completed
//! > Failure exits: rejected, cancelled, failed
//!
//! "No job can jump states. This is physics."

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::{OfficeError, Result};

/// Job states - the allowed positions in the FSM
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobState {
    /// Job created but not yet proposed
    Draft,
    /// Job proposed, awaiting approval
    Proposed,
    /// Job approved, ready to start
    Approved,
    /// Job actively being worked on
    InProgress,
    /// Job waiting for user input
    WaitingInput,
    /// Job completed successfully
    Completed,
    /// Job was rejected
    Rejected,
    /// Job was cancelled
    Cancelled,
    /// Job failed
    Failed,
}

impl JobState {
    /// Is this a terminal state (no further transitions)?
    pub fn is_terminal(&self) -> bool {
        matches!(self, 
            JobState::Completed | 
            JobState::Rejected | 
            JobState::Cancelled | 
            JobState::Failed
        )
    }

    /// Is this an active state (work is happening)?
    pub fn is_active(&self) -> bool {
        matches!(self,
            JobState::Proposed |
            JobState::Approved |
            JobState::InProgress |
            JobState::WaitingInput
        )
    }

    /// Parse from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "draft" => Some(JobState::Draft),
            "proposed" => Some(JobState::Proposed),
            "approved" => Some(JobState::Approved),
            "in_progress" => Some(JobState::InProgress),
            "waiting_input" => Some(JobState::WaitingInput),
            "completed" => Some(JobState::Completed),
            "rejected" => Some(JobState::Rejected),
            "cancelled" => Some(JobState::Cancelled),
            "failed" => Some(JobState::Failed),
            _ => None,
        }
    }

    /// To string
    pub fn as_str(&self) -> &'static str {
        match self {
            JobState::Draft => "draft",
            JobState::Proposed => "proposed",
            JobState::Approved => "approved",
            JobState::InProgress => "in_progress",
            JobState::WaitingInput => "waiting_input",
            JobState::Completed => "completed",
            JobState::Rejected => "rejected",
            JobState::Cancelled => "cancelled",
            JobState::Failed => "failed",
        }
    }
}

/// Transition reason codes
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransitionReason {
    /// User approved the job
    ApprovedByUser,
    /// User rejected the job
    RejectedByUser,
    /// Job execution started
    ExecutionStarted,
    /// Missing required inputs
    MissingRequiredInputs,
    /// Inputs were received
    InputsReceived,
    /// Job work completed
    WorkCompleted,
    /// Job work failed
    WorkFailed,
    /// User cancelled
    CancelledByUser,
    /// System cancelled (timeout, policy, etc.)
    CancelledBySystem,
    /// Provider error after retries
    ProviderUnavailable,
    /// Custom reason
    Custom(String),
}

impl TransitionReason {
    pub fn as_str(&self) -> &str {
        match self {
            TransitionReason::ApprovedByUser => "approved_by_user",
            TransitionReason::RejectedByUser => "rejected_by_user",
            TransitionReason::ExecutionStarted => "execution_started",
            TransitionReason::MissingRequiredInputs => "missing_required_inputs",
            TransitionReason::InputsReceived => "inputs_received",
            TransitionReason::WorkCompleted => "work_completed",
            TransitionReason::WorkFailed => "work_failed",
            TransitionReason::CancelledByUser => "cancelled_by_user",
            TransitionReason::CancelledBySystem => "cancelled_by_system",
            TransitionReason::ProviderUnavailable => "provider_unavailable_retries_exhausted",
            TransitionReason::Custom(s) => s,
        }
    }
}

/// The Job FSM - enforces legal state transitions
#[derive(Debug, Clone)]
pub struct JobFsm {
    /// Allowed transitions: (from, to)
    allowed_transitions: HashSet<(JobState, JobState)>,
}

impl JobFsm {
    /// Create a new FSM with the standard transition rules
    pub fn new() -> Self {
        let mut allowed = HashSet::new();

        // From spec:
        // draft → proposed
        allowed.insert((JobState::Draft, JobState::Proposed));

        // proposed → approved | rejected
        allowed.insert((JobState::Proposed, JobState::Approved));
        allowed.insert((JobState::Proposed, JobState::Rejected));

        // approved → in_progress
        allowed.insert((JobState::Approved, JobState::InProgress));

        // in_progress → waiting_input | completed | failed | cancelled
        allowed.insert((JobState::InProgress, JobState::WaitingInput));
        allowed.insert((JobState::InProgress, JobState::Completed));
        allowed.insert((JobState::InProgress, JobState::Failed));
        allowed.insert((JobState::InProgress, JobState::Cancelled));

        // waiting_input → in_progress | failed | cancelled
        allowed.insert((JobState::WaitingInput, JobState::InProgress));
        allowed.insert((JobState::WaitingInput, JobState::Failed));
        allowed.insert((JobState::WaitingInput, JobState::Cancelled));

        Self { allowed_transitions: allowed }
    }

    /// Check if a transition is allowed
    pub fn can_transition(&self, from: JobState, to: JobState) -> bool {
        self.allowed_transitions.contains(&(from, to))
    }

    /// Attempt a transition, returning an error if not allowed
    pub fn transition(&self, from: JobState, to: JobState, reason: TransitionReason) -> Result<Transition> {
        if from.is_terminal() {
            return Err(OfficeError::JobTransitionError(format!(
                "Cannot transition from terminal state '{}'",
                from.as_str()
            )));
        }

        if !self.can_transition(from, to) {
            return Err(OfficeError::JobTransitionError(format!(
                "Illegal transition: {} → {}",
                from.as_str(),
                to.as_str()
            )));
        }

        Ok(Transition {
            from,
            to,
            reason,
            timestamp: chrono::Utc::now(),
        })
    }

    /// Get all valid next states from current state
    pub fn valid_next_states(&self, current: JobState) -> Vec<JobState> {
        if current.is_terminal() {
            return vec![];
        }

        self.allowed_transitions
            .iter()
            .filter(|(from, _)| *from == current)
            .map(|(_, to)| *to)
            .collect()
    }
}

impl Default for JobFsm {
    fn default() -> Self {
        Self::new()
    }
}

/// A recorded transition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transition {
    pub from: JobState,
    pub to: JobState,
    pub reason: TransitionReason,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Job state tracker - maintains current state with history
#[derive(Debug, Clone)]
pub struct JobStateTracker {
    fsm: JobFsm,
    current_state: JobState,
    history: Vec<Transition>,
}

impl JobStateTracker {
    pub fn new() -> Self {
        Self {
            fsm: JobFsm::new(),
            current_state: JobState::Draft,
            history: vec![],
        }
    }

    pub fn with_state(state: JobState) -> Self {
        Self {
            fsm: JobFsm::new(),
            current_state: state,
            history: vec![],
        }
    }

    pub fn current(&self) -> JobState {
        self.current_state
    }

    pub fn is_terminal(&self) -> bool {
        self.current_state.is_terminal()
    }

    pub fn is_active(&self) -> bool {
        self.current_state.is_active()
    }

    /// Transition to a new state
    pub fn transition(&mut self, to: JobState, reason: TransitionReason) -> Result<&Transition> {
        let transition = self.fsm.transition(self.current_state, to, reason)?;
        self.current_state = to;
        self.history.push(transition);
        Ok(self.history.last().unwrap())
    }

    /// Get valid next states
    pub fn valid_next_states(&self) -> Vec<JobState> {
        self.fsm.valid_next_states(self.current_state)
    }

    /// Get transition history
    pub fn history(&self) -> &[Transition] {
        &self.history
    }

    /// Convenience transitions
    pub fn propose(&mut self) -> Result<&Transition> {
        self.transition(JobState::Proposed, TransitionReason::Custom("job_proposed".to_string()))
    }

    pub fn approve(&mut self) -> Result<&Transition> {
        self.transition(JobState::Approved, TransitionReason::ApprovedByUser)
    }

    pub fn reject(&mut self) -> Result<&Transition> {
        self.transition(JobState::Rejected, TransitionReason::RejectedByUser)
    }

    pub fn start(&mut self) -> Result<&Transition> {
        self.transition(JobState::InProgress, TransitionReason::ExecutionStarted)
    }

    pub fn wait_for_input(&mut self) -> Result<&Transition> {
        self.transition(JobState::WaitingInput, TransitionReason::MissingRequiredInputs)
    }

    pub fn resume(&mut self) -> Result<&Transition> {
        self.transition(JobState::InProgress, TransitionReason::InputsReceived)
    }

    pub fn complete(&mut self) -> Result<&Transition> {
        self.transition(JobState::Completed, TransitionReason::WorkCompleted)
    }

    pub fn fail(&mut self, reason: TransitionReason) -> Result<&Transition> {
        self.transition(JobState::Failed, reason)
    }

    pub fn cancel(&mut self, by_user: bool) -> Result<&Transition> {
        let reason = if by_user {
            TransitionReason::CancelledByUser
        } else {
            TransitionReason::CancelledBySystem
        };
        self.transition(JobState::Cancelled, reason)
    }
}

impl Default for JobStateTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_happy_path() {
        let mut tracker = JobStateTracker::new();
        
        assert_eq!(tracker.current(), JobState::Draft);
        
        tracker.propose().unwrap();
        assert_eq!(tracker.current(), JobState::Proposed);
        
        tracker.approve().unwrap();
        assert_eq!(tracker.current(), JobState::Approved);
        
        tracker.start().unwrap();
        assert_eq!(tracker.current(), JobState::InProgress);
        
        tracker.complete().unwrap();
        assert_eq!(tracker.current(), JobState::Completed);
        assert!(tracker.is_terminal());
        
        assert_eq!(tracker.history().len(), 4);
    }

    #[test]
    fn test_waiting_input_flow() {
        let mut tracker = JobStateTracker::with_state(JobState::InProgress);
        
        tracker.wait_for_input().unwrap();
        assert_eq!(tracker.current(), JobState::WaitingInput);
        
        tracker.resume().unwrap();
        assert_eq!(tracker.current(), JobState::InProgress);
        
        tracker.complete().unwrap();
        assert!(tracker.is_terminal());
    }

    #[test]
    fn test_illegal_transition() {
        let mut tracker = JobStateTracker::new();
        
        // Can't go directly from draft to in_progress
        let result = tracker.start();
        assert!(result.is_err());
        
        // State should be unchanged
        assert_eq!(tracker.current(), JobState::Draft);
    }

    #[test]
    fn test_terminal_state_locked() {
        let mut tracker = JobStateTracker::with_state(JobState::Completed);
        
        // Can't transition from terminal state
        let result = tracker.start();
        assert!(result.is_err());
    }

    #[test]
    fn test_valid_next_states() {
        let tracker = JobStateTracker::with_state(JobState::InProgress);
        let valid = tracker.valid_next_states();
        
        assert!(valid.contains(&JobState::WaitingInput));
        assert!(valid.contains(&JobState::Completed));
        assert!(valid.contains(&JobState::Failed));
        assert!(valid.contains(&JobState::Cancelled));
        assert!(!valid.contains(&JobState::Draft));
    }
}

