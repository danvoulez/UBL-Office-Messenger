//! Task Finite State Machine
//!
//! From the task formalization spec:
//! > States: draft → approved → running → (paused ↔ running) → completed → accepted
//! > Failure exits: rejected, cancelled, failed, disputed
//!
//! "No task can jump states. This is physics."
//!
//! Key difference from JobFsm: Tasks have an acceptance phase after completion.
//! The agent drafts, the human approves. When complete, human must accept.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::{OfficeError, Result};

/// Task states - the allowed positions in the FSM
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskState {
    /// Task created as draft, awaiting approval
    Draft,
    /// Task approved, ready to start execution
    Approved,
    /// Task is being executed
    Running,
    /// Task paused, waiting for input
    Paused,
    /// Task completed, awaiting human acceptance
    Completed,
    /// Task accepted by human, officially closed
    Accepted,
    /// Task was rejected during approval
    Rejected,
    /// Task was cancelled
    Cancelled,
    /// Task execution failed
    Failed,
    /// Task was disputed after completion
    Disputed,
}

impl TaskState {
    /// Is this a terminal state (no further transitions)?
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            TaskState::Accepted
                | TaskState::Rejected
                | TaskState::Cancelled
                | TaskState::Failed
                | TaskState::Disputed
        )
    }

    /// Is this an active state (work could happen)?
    pub fn is_active(&self) -> bool {
        matches!(
            self,
            TaskState::Draft
                | TaskState::Approved
                | TaskState::Running
                | TaskState::Paused
                | TaskState::Completed
        )
    }

    /// Is the task awaiting action from another party?
    pub fn awaiting_action(&self) -> bool {
        matches!(self, TaskState::Draft | TaskState::Completed)
    }

    /// Parse from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "draft" => Some(TaskState::Draft),
            "approved" => Some(TaskState::Approved),
            "running" => Some(TaskState::Running),
            "paused" => Some(TaskState::Paused),
            "completed" => Some(TaskState::Completed),
            "accepted" => Some(TaskState::Accepted),
            "rejected" => Some(TaskState::Rejected),
            "cancelled" => Some(TaskState::Cancelled),
            "failed" => Some(TaskState::Failed),
            "disputed" => Some(TaskState::Disputed),
            _ => None,
        }
    }

    /// To string
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskState::Draft => "draft",
            TaskState::Approved => "approved",
            TaskState::Running => "running",
            TaskState::Paused => "paused",
            TaskState::Completed => "completed",
            TaskState::Accepted => "accepted",
            TaskState::Rejected => "rejected",
            TaskState::Cancelled => "cancelled",
            TaskState::Failed => "failed",
            TaskState::Disputed => "disputed",
        }
    }
}

/// Transition reason codes
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransitionReason {
    /// Task approved by other party
    ApprovedByParty,
    /// Task rejected by other party
    RejectedByParty,
    /// Task execution started
    ExecutionStarted,
    /// Missing required inputs
    MissingRequiredInputs,
    /// Inputs were provided
    InputsReceived,
    /// Task work completed
    WorkCompleted,
    /// Task work failed
    WorkFailed,
    /// Task accepted by human
    AcceptedByHuman,
    /// Task disputed by human
    DisputedByHuman,
    /// User cancelled
    CancelledByUser,
    /// System cancelled (timeout, policy, etc.)
    CancelledBySystem,
    /// Custom reason
    Custom(String),
}

impl TransitionReason {
    pub fn as_str(&self) -> &str {
        match self {
            TransitionReason::ApprovedByParty => "approved_by_party",
            TransitionReason::RejectedByParty => "rejected_by_party",
            TransitionReason::ExecutionStarted => "execution_started",
            TransitionReason::MissingRequiredInputs => "missing_required_inputs",
            TransitionReason::InputsReceived => "inputs_received",
            TransitionReason::WorkCompleted => "work_completed",
            TransitionReason::WorkFailed => "work_failed",
            TransitionReason::AcceptedByHuman => "accepted_by_human",
            TransitionReason::DisputedByHuman => "disputed_by_human",
            TransitionReason::CancelledByUser => "cancelled_by_user",
            TransitionReason::CancelledBySystem => "cancelled_by_system",
            TransitionReason::Custom(s) => s,
        }
    }
}

/// The Task FSM - enforces legal state transitions
#[derive(Debug, Clone)]
pub struct TaskFsm {
    /// Allowed transitions: (from, to)
    allowed_transitions: HashSet<(TaskState, TaskState)>,
}

impl TaskFsm {
    /// Create a new FSM with the standard transition rules
    pub fn new() -> Self {
        let mut allowed = HashSet::new();

        // Draft phase (awaiting approval)
        // draft → approved | rejected | cancelled
        allowed.insert((TaskState::Draft, TaskState::Approved));
        allowed.insert((TaskState::Draft, TaskState::Rejected));
        allowed.insert((TaskState::Draft, TaskState::Cancelled));

        // Approved → execution
        // approved → running | cancelled
        allowed.insert((TaskState::Approved, TaskState::Running));
        allowed.insert((TaskState::Approved, TaskState::Cancelled));

        // Running phase
        // running → paused | completed | failed | cancelled
        allowed.insert((TaskState::Running, TaskState::Paused));
        allowed.insert((TaskState::Running, TaskState::Completed));
        allowed.insert((TaskState::Running, TaskState::Failed));
        allowed.insert((TaskState::Running, TaskState::Cancelled));

        // Paused (waiting for input)
        // paused → running | failed | cancelled
        allowed.insert((TaskState::Paused, TaskState::Running));
        allowed.insert((TaskState::Paused, TaskState::Failed));
        allowed.insert((TaskState::Paused, TaskState::Cancelled));

        // Completed (awaiting human acceptance)
        // completed → accepted | disputed
        allowed.insert((TaskState::Completed, TaskState::Accepted));
        allowed.insert((TaskState::Completed, TaskState::Disputed));

        Self {
            allowed_transitions: allowed,
        }
    }

    /// Check if a transition is allowed
    pub fn can_transition(&self, from: TaskState, to: TaskState) -> bool {
        self.allowed_transitions.contains(&(from, to))
    }

    /// Attempt a transition, returning an error if not allowed
    pub fn transition(
        &self,
        from: TaskState,
        to: TaskState,
        reason: TransitionReason,
    ) -> Result<Transition> {
        if from.is_terminal() {
            return Err(OfficeError::JobTransitionError(format!(
                "Cannot transition from terminal state '{}'",
                from.as_str()
            )));
        }

        if !self.can_transition(from, to) {
            return Err(OfficeError::JobTransitionError(format!(
                "Illegal task transition: {} → {}",
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
    pub fn valid_next_states(&self, current: TaskState) -> Vec<TaskState> {
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

impl Default for TaskFsm {
    fn default() -> Self {
        Self::new()
    }
}

/// A recorded transition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transition {
    pub from: TaskState,
    pub to: TaskState,
    pub reason: TransitionReason,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Task state tracker - maintains current state with history
#[derive(Debug, Clone)]
pub struct TaskStateTracker {
    fsm: TaskFsm,
    current_state: TaskState,
    history: Vec<Transition>,
}

impl TaskStateTracker {
    pub fn new() -> Self {
        Self {
            fsm: TaskFsm::new(),
            current_state: TaskState::Draft,
            history: vec![],
        }
    }

    pub fn with_state(state: TaskState) -> Self {
        Self {
            fsm: TaskFsm::new(),
            current_state: state,
            history: vec![],
        }
    }

    pub fn current(&self) -> TaskState {
        self.current_state
    }

    pub fn is_terminal(&self) -> bool {
        self.current_state.is_terminal()
    }

    pub fn is_active(&self) -> bool {
        self.current_state.is_active()
    }

    /// Is the task awaiting action from another party?
    pub fn awaiting_action(&self) -> bool {
        self.current_state.awaiting_action()
    }

    /// Transition to a new state
    pub fn transition(&mut self, to: TaskState, reason: TransitionReason) -> Result<&Transition> {
        let transition = self.fsm.transition(self.current_state, to, reason)?;
        self.current_state = to;
        self.history.push(transition);
        Ok(self.history.last().unwrap())
    }

    /// Get valid next states
    pub fn valid_next_states(&self) -> Vec<TaskState> {
        self.fsm.valid_next_states(self.current_state)
    }

    /// Get transition history
    pub fn history(&self) -> &[Transition] {
        &self.history
    }

    // ============ Convenience transitions ============

    /// Approve the task (draft → approved)
    pub fn approve(&mut self) -> Result<&Transition> {
        self.transition(TaskState::Approved, TransitionReason::ApprovedByParty)
    }

    /// Reject the task (draft → rejected)
    pub fn reject(&mut self) -> Result<&Transition> {
        self.transition(TaskState::Rejected, TransitionReason::RejectedByParty)
    }

    /// Start execution (approved → running)
    pub fn start(&mut self) -> Result<&Transition> {
        self.transition(TaskState::Running, TransitionReason::ExecutionStarted)
    }

    /// Pause for input (running → paused)
    pub fn pause(&mut self) -> Result<&Transition> {
        self.transition(TaskState::Paused, TransitionReason::MissingRequiredInputs)
    }

    /// Resume after input (paused → running)
    pub fn resume(&mut self) -> Result<&Transition> {
        self.transition(TaskState::Running, TransitionReason::InputsReceived)
    }

    /// Complete execution (running → completed)
    pub fn complete(&mut self) -> Result<&Transition> {
        self.transition(TaskState::Completed, TransitionReason::WorkCompleted)
    }

    /// Fail execution (running/paused → failed)
    pub fn fail(&mut self, reason: TransitionReason) -> Result<&Transition> {
        self.transition(TaskState::Failed, reason)
    }

    /// Accept the task (completed → accepted)
    pub fn accept(&mut self) -> Result<&Transition> {
        self.transition(TaskState::Accepted, TransitionReason::AcceptedByHuman)
    }

    /// Dispute the task (completed → disputed)
    pub fn dispute(&mut self) -> Result<&Transition> {
        self.transition(TaskState::Disputed, TransitionReason::DisputedByHuman)
    }

    /// Cancel the task
    pub fn cancel(&mut self, by_user: bool) -> Result<&Transition> {
        let reason = if by_user {
            TransitionReason::CancelledByUser
        } else {
            TransitionReason::CancelledBySystem
        };
        self.transition(TaskState::Cancelled, reason)
    }
}

impl Default for TaskStateTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_happy_path() {
        let mut tracker = TaskStateTracker::new();

        assert_eq!(tracker.current(), TaskState::Draft);

        tracker.approve().unwrap();
        assert_eq!(tracker.current(), TaskState::Approved);

        tracker.start().unwrap();
        assert_eq!(tracker.current(), TaskState::Running);

        tracker.complete().unwrap();
        assert_eq!(tracker.current(), TaskState::Completed);

        tracker.accept().unwrap();
        assert_eq!(tracker.current(), TaskState::Accepted);
        assert!(tracker.is_terminal());

        assert_eq!(tracker.history().len(), 4);
    }

    #[test]
    fn test_dispute_flow() {
        let mut tracker = TaskStateTracker::with_state(TaskState::Completed);

        tracker.dispute().unwrap();
        assert_eq!(tracker.current(), TaskState::Disputed);
        assert!(tracker.is_terminal());
    }

    #[test]
    fn test_paused_flow() {
        let mut tracker = TaskStateTracker::with_state(TaskState::Running);

        tracker.pause().unwrap();
        assert_eq!(tracker.current(), TaskState::Paused);

        tracker.resume().unwrap();
        assert_eq!(tracker.current(), TaskState::Running);

        tracker.complete().unwrap();
        assert_eq!(tracker.current(), TaskState::Completed);
    }

    #[test]
    fn test_rejection_flow() {
        let mut tracker = TaskStateTracker::new();

        tracker.reject().unwrap();
        assert_eq!(tracker.current(), TaskState::Rejected);
        assert!(tracker.is_terminal());
    }

    #[test]
    fn test_illegal_transition() {
        let mut tracker = TaskStateTracker::new();

        // Can't go directly from draft to running
        let result = tracker.start();
        assert!(result.is_err());

        // State should be unchanged
        assert_eq!(tracker.current(), TaskState::Draft);
    }

    #[test]
    fn test_terminal_state_locked() {
        let mut tracker = TaskStateTracker::with_state(TaskState::Accepted);

        // Can't transition from terminal state
        let result = tracker.start();
        assert!(result.is_err());
    }

    #[test]
    fn test_valid_next_states() {
        let tracker = TaskStateTracker::with_state(TaskState::Completed);
        let valid = tracker.valid_next_states();

        assert!(valid.contains(&TaskState::Accepted));
        assert!(valid.contains(&TaskState::Disputed));
        assert!(!valid.contains(&TaskState::Draft));
        assert!(!valid.contains(&TaskState::Running));
    }

    #[test]
    fn test_awaiting_action() {
        let draft = TaskStateTracker::with_state(TaskState::Draft);
        assert!(draft.awaiting_action());

        let completed = TaskStateTracker::with_state(TaskState::Completed);
        assert!(completed.awaiting_action());

        let running = TaskStateTracker::with_state(TaskState::Running);
        assert!(!running.awaiting_action());
    }
}
