//! Job Types
//!
//! Type definitions for job execution system.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Unique job identifier
pub type JobId = String;

/// Job status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    /// Job created but not started
    Created,
    /// Job execution in progress
    Running,
    /// Job paused waiting for approval
    Paused,
    /// Job completed successfully
    Completed,
    /// Job cancelled
    Cancelled,
    /// Job failed
    Failed,
}

/// Job entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    /// Unique job ID
    pub id: JobId,
    /// Conversation ID where job was created
    pub conversation_id: String,
    /// Job title
    pub title: String,
    /// Job description
    pub description: Option<String>,
    /// User/agent assigned to execute
    pub assigned_to: String,
    /// User/agent who created the job
    pub created_by: String,
    /// Priority level
    pub priority: Option<String>,
    /// Estimated duration in seconds
    pub estimated_duration_seconds: Option<u64>,
    /// Estimated monetary value
    pub estimated_value: Option<f64>,
    /// Current status
    pub status: JobStatus,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Started timestamp
    pub started_at: Option<DateTime<Utc>>,
    /// Completed timestamp
    pub completed_at: Option<DateTime<Utc>>,
}

/// Job execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobResult {
    /// Job ID
    pub job_id: JobId,
    /// Success flag
    pub success: bool,
    /// Result summary
    pub summary: Option<String>,
    /// Full output content
    pub output: Option<String>,
    /// Artifacts created (file IDs, URLs, etc.)
    pub artifacts: Vec<String>,
    /// Tokens used
    pub tokens_used: u64,
    /// Value created (if applicable)
    pub value_created: Option<f64>,
    /// Duration in seconds
    pub duration_seconds: u64,
    /// Error message (if failed)
    pub error: Option<String>,
}

impl Default for JobResult {
    fn default() -> Self {
        Self {
            job_id: String::new(),
            success: false,
            summary: None,
            output: None,
            artifacts: vec![],
            tokens_used: 0,
            value_created: None,
            duration_seconds: 0,
            error: None,
        }
    }
}

/// Job progress update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobProgress {
    /// Current step identifier
    pub current_step: String,
    /// Step description
    pub step_description: Option<String>,
    /// Total number of steps
    pub total_steps: Option<u32>,
    /// Completion percentage (0-100)
    pub percent_complete: Option<u32>,
}

/// Individual job step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobStep {
    /// Step identifier
    pub id: String,
    /// Step description
    pub description: String,
    /// Step status
    pub status: StepStatus,
}

/// Step status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepStatus {
    /// Step not started
    Pending,
    /// Step in progress
    InProgress,
    /// Step completed
    Completed,
    /// Step failed
    Failed,
}

/// Approval request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRequest {
    /// Unique approval ID
    pub approval_id: String,
    /// Job ID
    pub job_id: JobId,
    /// Approval title
    pub title: String,
    /// Approval details (list of strings)
    pub details: Vec<String>,
    /// Impact description
    pub impact: String,
    /// Requested by
    pub requested_by: String,
    /// Requested timestamp
    pub requested_at: DateTime<Utc>,
}

/// Approval decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalDecision {
    /// Approval ID
    pub approval_id: String,
    /// Job ID
    pub job_id: JobId,
    /// Decision: "approved", "rejected", or "request_changes"
    pub decision: String,
    /// User/agent who made the decision
    pub decided_by: String,
    /// Decision timestamp
    pub decided_at: DateTime<Utc>,
    /// Decision reason (optional)
    pub reason: Option<String>,
}

impl ApprovalDecision {
    /// Check if decision is approved
    pub fn is_approved(&self) -> bool {
        self.decision == "approved"
    }

    /// Check if decision is rejected
    pub fn is_rejected(&self) -> bool {
        self.decision == "rejected"
    }

    /// Check if changes are requested
    pub fn is_request_changes(&self) -> bool {
        self.decision == "request_changes"
    }
}

/// Conversation context for job execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationContext {
    /// Conversation ID
    pub conversation_id: String,
    /// List of participants (user/agent IDs)
    pub participants: Vec<String>,
    /// Recent messages (last N messages)
    pub recent_messages: Vec<Message>,
    /// Active jobs in conversation
    pub active_jobs: Vec<Job>,
    /// Recent events
    pub recent_events: Vec<String>,
}

/// Message in conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Message ID
    pub id: String,
    /// From user/agent ID
    pub from: String,
    /// Message content
    pub content: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Progress update stream item
#[derive(Debug, Clone)]
pub enum ProgressUpdate {
    /// Step completed
    StepCompleted(JobStep),
    /// Approval needed
    ApprovalNeeded(ApprovalRequest),
    /// Job completed
    Completed(JobResult),
    /// Job failed
    Failed(String),
}

