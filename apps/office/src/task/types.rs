//! Task Types
//!
//! Types for the task formalization system.
//! Tasks are formalizations of conversations - any participant can create a draft
//! that the other party must approve.
//!
//! Flow:
//! 1. task.created (draft) - Creator proposes task
//! 2. task.approved - Other party approves draft
//! 3. task.started - Execution begins (for agent tasks)
//! 4. task.progress - Progress updates (SSE stream)
//! 5. task.completed - Execution finished
//! 6. task.accepted - Final acceptance by human
//! 7. task.disputed - Human disputes outcome

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Unique task identifier
pub type TaskId = String;

/// Task status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    /// Draft created, awaiting approval from other party
    Draft,
    /// Approved by other party, ready to start
    Approved,
    /// Rejected by other party
    Rejected,
    /// Task execution in progress
    Running,
    /// Task paused, waiting for input
    Paused,
    /// Task completed, awaiting acceptance
    Completed,
    /// Task accepted by human, officially closed
    Accepted,
    /// Task disputed by human
    Disputed,
    /// Task cancelled
    Cancelled,
    /// Task failed
    Failed,
}

/// Priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskPriority {
    Low,
    Normal,
    High,
    Critical,
}

impl Default for TaskPriority {
    fn default() -> Self {
        Self::Normal
    }
}

/// Task entity - represents a formalized task from conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// Unique task ID
    pub id: TaskId,
    /// Conversation ID where task was created
    pub conversation_id: String,
    /// Tenant ID
    pub tenant_id: String,
    /// Task title
    pub title: String,
    /// Task description
    pub description: Option<String>,
    /// Priority level
    pub priority: TaskPriority,
    /// Deadline (optional)
    pub deadline: Option<DateTime<Utc>>,
    /// Estimated cost (optional)
    pub estimated_cost: Option<String>,
    /// Who created the task draft
    pub created_by: String,
    /// Who needs to approve/execute
    pub assigned_to: String,
    /// Current status
    pub status: TaskStatus,
    /// Progress percentage (0-100)
    pub progress: u8,
    /// Current step description
    pub current_step: Option<String>,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Approved timestamp
    pub approved_at: Option<DateTime<Utc>>,
    /// Started timestamp
    pub started_at: Option<DateTime<Utc>>,
    /// Completed timestamp
    pub completed_at: Option<DateTime<Utc>>,
    /// Accepted timestamp
    pub accepted_at: Option<DateTime<Utc>>,
    /// Attachments
    pub attachments: Vec<TaskAttachment>,
    /// Result artifacts (after completion)
    pub artifacts: Vec<TaskArtifact>,
    /// Git commit info (if versioned)
    pub git_commit: Option<GitCommit>,
    /// UBL ledger hashes for this task
    pub ledger_hashes: Vec<String>,
}

/// Attachment on a task (input files)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskAttachment {
    pub id: String,
    pub filename: String,
    pub size_bytes: u64,
    pub mime_type: String,
    pub url: String,
    pub hash: Option<String>,
}

/// Artifact produced by task execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskArtifact {
    pub id: String,
    pub name: String,
    pub artifact_type: ArtifactType,
    pub url: String,
    pub size_bytes: Option<u64>,
    pub mime_type: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactType {
    File,
    Document,
    Report,
    Code,
    Data,
    Other,
}

/// Git commit info for versioned documents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitCommit {
    pub hash: String,
    pub message: String,
    pub branch: String,
    pub repo_url: Option<String>,
    pub committed_at: DateTime<Utc>,
}

/// Task draft creation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTaskRequest {
    /// Conversation where task is created
    pub conversation_id: String,
    /// Tenant ID
    pub tenant_id: String,
    /// Task title
    pub title: String,
    /// Task description
    pub description: Option<String>,
    /// Priority
    pub priority: Option<TaskPriority>,
    /// Deadline
    pub deadline: Option<DateTime<Utc>>,
    /// Estimated cost
    pub estimated_cost: Option<String>,
    /// Who is creating (user/agent ID)
    pub created_by: String,
    /// Who needs to approve/execute
    pub assigned_to: String,
    /// Attachment IDs
    pub attachment_ids: Option<Vec<String>>,
}

/// Task approval request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApproveTaskRequest {
    /// Who is approving
    pub approved_by: String,
    /// Optional modifications before approval
    pub modifications: Option<TaskModifications>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskModifications {
    pub title: Option<String>,
    pub description: Option<String>,
    pub deadline: Option<DateTime<Utc>>,
    pub estimated_cost: Option<String>,
}

/// Task rejection request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RejectTaskRequest {
    /// Who is rejecting
    pub rejected_by: String,
    /// Reason for rejection
    pub reason: String,
}

/// Task acceptance request (after completion)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptTaskRequest {
    /// Who is accepting
    pub accepted_by: String,
    /// Optional feedback
    pub feedback: Option<String>,
}

/// Task dispute request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisputeTaskRequest {
    /// Who is disputing
    pub disputed_by: String,
    /// Reason for dispute
    pub reason: String,
}

/// Progress update for SSE streaming
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskProgressUpdate {
    pub task_id: TaskId,
    pub progress: u8,
    pub current_step: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
}

/// Log entry for SSE streaming
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskLogEntry {
    pub task_id: TaskId,
    pub level: LogLevel,
    pub message: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

/// Task result after completion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub task_id: TaskId,
    pub success: bool,
    pub summary: String,
    pub artifacts: Vec<TaskArtifact>,
    pub duration_seconds: u64,
    pub tokens_used: Option<u64>,
    pub git_commit: Option<GitCommit>,
    pub error: Option<String>,
}

/// UBL Event types for tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TaskEvent {
    #[serde(rename = "task.created")]
    Created(TaskCreatedEvent),
    #[serde(rename = "task.approved")]
    Approved(TaskApprovedEvent),
    #[serde(rename = "task.rejected")]
    Rejected(TaskRejectedEvent),
    #[serde(rename = "task.started")]
    Started(TaskStartedEvent),
    #[serde(rename = "task.progress")]
    Progress(TaskProgressEvent),
    #[serde(rename = "task.completed")]
    Completed(TaskCompletedEvent),
    #[serde(rename = "task.accepted")]
    Accepted(TaskAcceptedEvent),
    #[serde(rename = "task.disputed")]
    Disputed(TaskDisputedEvent),
    #[serde(rename = "task.cancelled")]
    Cancelled(TaskCancelledEvent),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskCreatedEvent {
    pub task_id: TaskId,
    pub conversation_id: String,
    pub title: String,
    pub description: Option<String>,
    pub priority: TaskPriority,
    pub deadline: Option<DateTime<Utc>>,
    pub estimated_cost: Option<String>,
    pub created_by: String,
    pub assigned_to: String,
    pub attachment_count: usize,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskApprovedEvent {
    pub task_id: TaskId,
    pub approved_by: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRejectedEvent {
    pub task_id: TaskId,
    pub rejected_by: String,
    pub reason: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStartedEvent {
    pub task_id: TaskId,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskProgressEvent {
    pub task_id: TaskId,
    pub progress: u8,
    pub current_step: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskCompletedEvent {
    pub task_id: TaskId,
    pub success: bool,
    pub summary: String,
    pub artifact_count: usize,
    pub duration_seconds: u64,
    pub git_commit_hash: Option<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskAcceptedEvent {
    pub task_id: TaskId,
    pub accepted_by: String,
    pub feedback: Option<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDisputedEvent {
    pub task_id: TaskId,
    pub disputed_by: String,
    pub reason: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskCancelledEvent {
    pub task_id: TaskId,
    pub cancelled_by: String,
    pub reason: Option<String>,
    pub timestamp: DateTime<Utc>,
}

impl Task {
    /// Create a new task draft
    pub fn new(req: CreateTaskRequest) -> Self {
        let now = Utc::now();
        let id = format!("task_{}", uuid::Uuid::new_v4().to_string().replace("-", "")[..12].to_string());
        
        Self {
            id,
            conversation_id: req.conversation_id,
            tenant_id: req.tenant_id,
            title: req.title,
            description: req.description,
            priority: req.priority.unwrap_or_default(),
            deadline: req.deadline,
            estimated_cost: req.estimated_cost,
            created_by: req.created_by,
            assigned_to: req.assigned_to,
            status: TaskStatus::Draft,
            progress: 0,
            current_step: None,
            created_at: now,
            approved_at: None,
            started_at: None,
            completed_at: None,
            accepted_at: None,
            attachments: vec![],
            artifacts: vec![],
            git_commit: None,
            ledger_hashes: vec![],
        }
    }

    /// Check if task can be approved
    pub fn can_approve(&self) -> bool {
        self.status == TaskStatus::Draft
    }

    /// Check if task can be accepted (after completion)
    pub fn can_accept(&self) -> bool {
        self.status == TaskStatus::Completed
    }

    /// Check if task can be disputed
    pub fn can_dispute(&self) -> bool {
        self.status == TaskStatus::Completed
    }

    /// Approve the task
    pub fn approve(&mut self, approved_by: &str) {
        self.status = TaskStatus::Approved;
        self.approved_at = Some(Utc::now());
    }

    /// Reject the task
    pub fn reject(&mut self, _rejected_by: &str, _reason: &str) {
        self.status = TaskStatus::Rejected;
    }

    /// Start execution
    pub fn start(&mut self) {
        self.status = TaskStatus::Running;
        self.started_at = Some(Utc::now());
    }

    /// Update progress
    pub fn update_progress(&mut self, progress: u8, step: &str) {
        self.progress = progress.min(100);
        self.current_step = Some(step.to_string());
    }

    /// Complete the task
    pub fn complete(&mut self, result: &TaskResult) {
        self.status = TaskStatus::Completed;
        self.completed_at = Some(Utc::now());
        self.artifacts = result.artifacts.clone();
        self.git_commit = result.git_commit.clone();
    }

    /// Accept the task (finalize)
    pub fn accept(&mut self, _accepted_by: &str) {
        self.status = TaskStatus::Accepted;
        self.accepted_at = Some(Utc::now());
    }

    /// Dispute the task
    pub fn dispute(&mut self, _disputed_by: &str, _reason: &str) {
        self.status = TaskStatus::Disputed;
    }

    /// Cancel the task
    pub fn cancel(&mut self) {
        self.status = TaskStatus::Cancelled;
    }

    /// Generate task.created event
    pub fn to_created_event(&self) -> TaskCreatedEvent {
        TaskCreatedEvent {
            task_id: self.id.clone(),
            conversation_id: self.conversation_id.clone(),
            title: self.title.clone(),
            description: self.description.clone(),
            priority: self.priority,
            deadline: self.deadline,
            estimated_cost: self.estimated_cost.clone(),
            created_by: self.created_by.clone(),
            assigned_to: self.assigned_to.clone(),
            attachment_count: self.attachments.len(),
            timestamp: self.created_at,
        }
    }
}
