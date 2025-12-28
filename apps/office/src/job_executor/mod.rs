//! Job Executor Module
//!
//! Executes jobs for UBL Messenger, managing LLM sessions, progress streaming,
//! and approval workflows.
//!
//! Key components:
//! - FSM: Strict state machine for job transitions
//! - Cards: Formalize, Tracking, Finished cards
//! - Executor: Orchestrates LLM execution with Chair context

pub mod types;
pub mod fsm;
pub mod cards;
mod executor;
mod conversation_context;

pub use types::{
    Job, JobId, JobStatus, JobResult, JobProgress, JobStep,
    ApprovalRequest, ApprovalDecision, ConversationContext,
};
pub use fsm::{JobState, JobFsm, JobStateTracker, Transition, TransitionReason};
pub use cards::{
    JobCard, FormalizeCard, TrackingCard, FinishedCard,
    CardBase, CardButton, CardAction, CardActor,
};
pub use executor::JobExecutor;
pub use conversation_context::ConversationContextBuilder;

use crate::{OfficeError, Result};

