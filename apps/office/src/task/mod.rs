//! Task Module
//!
//! Implements the task formalization system for UBL Messenger.
//!
//! Key concepts:
//! - **Task**: A formalized work item from a conversation
//! - **Draft**: Initial state, awaiting approval from other party
//! - **Approved**: Ready for execution
//! - **Running**: Being executed (with SSE progress streaming)
//! - **Completed**: Done, awaiting human acceptance
//! - **Accepted**: Finalized, versioned in git
//!
//! Flow:
//! 1. Human or agent creates a task draft from conversation
//! 2. Other party approves the draft
//! 3. If agent is assigned, executes with Entity/Instance
//! 4. Progress is streamed via SSE
//! 5. On completion, human must accept to finalize
//! 6. Accepted tasks become official documents in git

pub mod types;
pub mod fsm;
pub mod cards;
mod executor;

// Re-exports for convenience
pub use types::{
    Task, TaskId, TaskStatus, TaskPriority,
    TaskAttachment, TaskArtifact, ArtifactType,
    GitCommit, TaskResult,
    CreateTaskRequest, ApproveTaskRequest, AcceptTaskRequest,
    RejectTaskRequest, DisputeTaskRequest,
    TaskProgressUpdate, TaskLogEntry, LogLevel,
    // Events
    TaskEvent, TaskCreatedEvent, TaskApprovedEvent, TaskRejectedEvent,
    TaskStartedEvent, TaskProgressEvent, TaskCompletedEvent,
    TaskAcceptedEvent, TaskDisputedEvent, TaskCancelledEvent,
};

pub use fsm::{
    TaskState, TaskFsm, TaskStateTracker,
    Transition, TransitionReason,
};

pub use cards::{
    TaskCard, TaskCardBase,
    TaskCreationCard, TaskProgressCard, TaskCompletedCard,
    TaskCardBuilder,
    TaskActor, ActorType,
    TaskAction, TaskButton, ButtonStyle,
    TaskDetails, TaskProgressInfo, TaskOutcome, TaskStats,
    ProgressStep, StepState, LogEntry as CardLogEntry, LogLevel as CardLogLevel,
};

pub use executor::{TaskExecutor, TaskProgressMessage};
