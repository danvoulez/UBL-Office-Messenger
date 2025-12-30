//! Task Cards - The UX for tasks
//!
//! Task cards are similar to job cards but tailored for the task formalization flow:
//! 1. TaskCreationCard - Shows draft task awaiting approval
//! 2. TaskProgressCard - Shows running task with live progress
//! 3. TaskCompletedCard - Shows completed task awaiting acceptance
//!
//! Cards never block conversation. Chat remains fully available during tasks.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::fsm::TaskState;
use super::types::{TaskPriority, TaskAttachment, TaskArtifact};

/// Union of all task card types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "card_type", rename_all = "snake_case")]
pub enum TaskCard {
    #[serde(rename = "task.creation")]
    Creation(TaskCreationCard),
    #[serde(rename = "task.progress")]
    Progress(TaskProgressCard),
    #[serde(rename = "task.completed")]
    Completed(TaskCompletedCard),
}

/// Base fields shared by all task cards
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskCardBase {
    /// Unique card instance ID
    pub card_id: String,
    /// Task this card belongs to
    pub task_id: String,
    /// Version for forward compatibility
    pub version: String,
    /// Short title
    pub title: String,
    /// 1-2 line summary
    pub summary: Option<String>,
    /// Task state at time of card creation
    pub state: TaskState,
    /// When card was created
    pub created_at: DateTime<Utc>,
    /// Conversation this card belongs to
    pub conversation_id: String,
    /// Tenant ID
    pub tenant_id: String,
    /// Who created the task
    pub creator: TaskActor,
    /// Who is assigned to execute/approve
    pub assignee: TaskActor,
    /// Available buttons
    pub buttons: Vec<TaskButton>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskActor {
    pub entity_id: String,
    pub display_name: String,
    pub actor_type: ActorType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActorType {
    Human,
    Agent,
    System,
}

/// Button on a task card
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskButton {
    /// Stable ID for this button instance
    pub button_id: String,
    /// Display label
    pub label: String,
    /// What action this triggers
    pub action: TaskAction,
    /// Visual style
    pub style: Option<ButtonStyle>,
    /// Does this button require user input?
    pub requires_input: bool,
    /// Confirmation dialog before action
    pub confirm: Option<ConfirmDialog>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ButtonStyle {
    Primary,
    Secondary,
    Danger,
    Success,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmDialog {
    pub title: String,
    pub body: Option<String>,
}

/// Actions triggered by task card buttons
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TaskAction {
    /// Approve the task draft
    #[serde(rename = "task.approve")]
    Approve { task_id: String },
    /// Reject the task draft
    #[serde(rename = "task.reject")]
    Reject {
        task_id: String,
        reason_code: Option<String>,
    },
    /// Request modifications to the draft
    #[serde(rename = "task.modify")]
    Modify { task_id: String },
    /// Provide missing input
    #[serde(rename = "task.provide_input")]
    ProvideInput {
        task_id: String,
        input_schema: Option<InputSchema>,
    },
    /// Accept the completed task
    #[serde(rename = "task.accept")]
    Accept { task_id: String },
    /// Dispute the completed task
    #[serde(rename = "task.dispute")]
    Dispute {
        task_id: String,
        reason_code: Option<String>,
    },
    /// Cancel the task
    #[serde(rename = "task.cancel")]
    Cancel { task_id: String },
    /// Ask in chat (never blocks)
    #[serde(rename = "chat.ask")]
    ChatAsk {
        task_id: Option<String>,
        prompt_text: String,
    },
    /// View artifacts/documents
    #[serde(rename = "task.view_artifact")]
    ViewArtifact {
        task_id: String,
        artifact_id: String,
    },
}

/// Input schema for buttons that require user input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputSchema {
    pub fields: Vec<InputField>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputField {
    pub key: String,
    pub label: String,
    #[serde(rename = "type")]
    pub field_type: InputFieldType,
    pub required: bool,
    pub options: Option<Vec<SelectOption>>,
    pub placeholder: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InputFieldType {
    String,
    Number,
    Boolean,
    Select,
    Multiline,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectOption {
    pub value: String,
    pub label: String,
}

// ============ Task Creation Card ============

/// Task creation/proposal card - presented when a task draft is created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskCreationCard {
    #[serde(flatten)]
    pub base: TaskCardBase,
    /// Task details
    pub task_details: TaskDetails,
    /// Attachments included with the draft
    pub attachments: Vec<TaskAttachment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDetails {
    pub task_id: String,
    pub title: String,
    pub description: Option<String>,
    pub priority: TaskPriority,
    pub deadline: Option<DateTime<Utc>>,
    pub estimated_cost: Option<String>,
}

impl TaskCreationCard {
    /// Create default buttons for a creation card (approval phase)
    pub fn default_buttons(task_id: &str) -> Vec<TaskButton> {
        let short_id = &task_id[..8.min(task_id.len())];
        vec![
            TaskButton {
                button_id: format!("btn_approve_{}", short_id),
                label: "Approve".to_string(),
                action: TaskAction::Approve {
                    task_id: task_id.to_string(),
                },
                style: Some(ButtonStyle::Primary),
                requires_input: false,
                confirm: None,
            },
            TaskButton {
                button_id: format!("btn_reject_{}", short_id),
                label: "Reject".to_string(),
                action: TaskAction::Reject {
                    task_id: task_id.to_string(),
                    reason_code: None,
                },
                style: Some(ButtonStyle::Danger),
                requires_input: true,
                confirm: Some(ConfirmDialog {
                    title: "Reject this task?".to_string(),
                    body: Some("Please provide a reason for rejection.".to_string()),
                }),
            },
            TaskButton {
                button_id: format!("btn_modify_{}", short_id),
                label: "Request changes".to_string(),
                action: TaskAction::Modify {
                    task_id: task_id.to_string(),
                },
                style: Some(ButtonStyle::Secondary),
                requires_input: true,
                confirm: None,
            },
            TaskButton {
                button_id: format!("btn_ask_{}", short_id),
                label: "Ask in chat".to_string(),
                action: TaskAction::ChatAsk {
                    task_id: Some(task_id.to_string()),
                    prompt_text: "Question about this task—".to_string(),
                },
                style: Some(ButtonStyle::Secondary),
                requires_input: false,
                confirm: None,
            },
        ]
    }
}

// ============ Task Progress Card ============

/// Progress card - shown during task execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskProgressCard {
    #[serde(flatten)]
    pub base: TaskCardBase,
    /// Progress information
    pub progress: TaskProgressInfo,
    /// Preview of artifacts in progress
    pub artifacts_preview: Option<Vec<ArtifactPreview>>,
    /// Log entries (recent)
    pub recent_logs: Option<Vec<LogEntry>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskProgressInfo {
    /// Completion percentage (0-100)
    pub percent: u8,
    /// Human-readable status line
    pub status_line: String,
    /// Current step indicator
    pub current_step: Option<String>,
    /// Blockers preventing progress
    pub blockers: Option<Vec<String>>,
    /// Step breakdown
    pub steps: Option<Vec<ProgressStep>>,
    /// Last update timestamp
    pub last_update_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressStep {
    pub key: String,
    pub label: String,
    pub state: StepState,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepState {
    Todo,
    Doing,
    Done,
    Blocked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactPreview {
    pub artifact_id: String,
    pub name: String,
    pub status: ArtifactStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactStatus {
    Pending,
    InProgress,
    Ready,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl TaskProgressCard {
    /// Create default buttons for a progress card
    pub fn default_buttons(task_id: &str) -> Vec<TaskButton> {
        let short_id = &task_id[..8.min(task_id.len())];
        vec![
            TaskButton {
                button_id: format!("btn_input_{}", short_id),
                label: "Provide info".to_string(),
                action: TaskAction::ProvideInput {
                    task_id: task_id.to_string(),
                    input_schema: None,
                },
                style: Some(ButtonStyle::Secondary),
                requires_input: true,
                confirm: None,
            },
            TaskButton {
                button_id: format!("btn_cancel_{}", short_id),
                label: "Cancel".to_string(),
                action: TaskAction::Cancel {
                    task_id: task_id.to_string(),
                },
                style: Some(ButtonStyle::Danger),
                requires_input: false,
                confirm: Some(ConfirmDialog {
                    title: "Cancel this task?".to_string(),
                    body: Some("The task will be stopped.".to_string()),
                }),
            },
            TaskButton {
                button_id: format!("btn_ask_{}", short_id),
                label: "Ask in chat".to_string(),
                action: TaskAction::ChatAsk {
                    task_id: Some(task_id.to_string()),
                    prompt_text: "Question about this task—".to_string(),
                },
                style: Some(ButtonStyle::Secondary),
                requires_input: false,
                confirm: None,
            },
        ]
    }
}

// ============ Task Completed Card ============

/// Completion card - shown when task is done, awaiting human acceptance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskCompletedCard {
    #[serde(flatten)]
    pub base: TaskCardBase,
    /// Outcome details
    pub outcome: TaskOutcome,
    /// Artifacts produced
    pub artifacts: Vec<TaskArtifact>,
    /// Git commit info (if versioned)
    pub git_commit: Option<GitCommitInfo>,
    /// Statistics
    pub stats: TaskStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskOutcome {
    pub success: bool,
    pub summary: String,
    pub completed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitCommitInfo {
    pub hash: String,
    pub message: String,
    pub branch: String,
    pub repo_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStats {
    pub duration_seconds: u64,
    pub tokens_used: Option<u64>,
    pub artifact_count: usize,
}

impl TaskCompletedCard {
    /// Create default buttons for a completed card (acceptance phase)
    pub fn default_buttons(task_id: &str) -> Vec<TaskButton> {
        let short_id = &task_id[..8.min(task_id.len())];
        vec![
            TaskButton {
                button_id: format!("btn_accept_{}", short_id),
                label: "Accept".to_string(),
                action: TaskAction::Accept {
                    task_id: task_id.to_string(),
                },
                style: Some(ButtonStyle::Success),
                requires_input: false,
                confirm: Some(ConfirmDialog {
                    title: "Accept this task?".to_string(),
                    body: Some("This will finalize the task and make it an official record.".to_string()),
                }),
            },
            TaskButton {
                button_id: format!("btn_dispute_{}", short_id),
                label: "Dispute".to_string(),
                action: TaskAction::Dispute {
                    task_id: task_id.to_string(),
                    reason_code: None,
                },
                style: Some(ButtonStyle::Danger),
                requires_input: true,
                confirm: None,
            },
            TaskButton {
                button_id: format!("btn_ask_{}", short_id),
                label: "Ask in chat".to_string(),
                action: TaskAction::ChatAsk {
                    task_id: Some(task_id.to_string()),
                    prompt_text: "Question about this outcome—".to_string(),
                },
                style: Some(ButtonStyle::Secondary),
                requires_input: false,
                confirm: None,
            },
        ]
    }

    /// Add artifact view buttons
    pub fn artifact_buttons(task_id: &str, artifacts: &[TaskArtifact]) -> Vec<TaskButton> {
        artifacts
            .iter()
            .take(3) // Limit to 3 artifacts as buttons
            .map(|artifact| {
                let short_id = &artifact.id[..8.min(artifact.id.len())];
                TaskButton {
                    button_id: format!("btn_view_{}", short_id),
                    label: format!("View {}", artifact.name),
                    action: TaskAction::ViewArtifact {
                        task_id: task_id.to_string(),
                        artifact_id: artifact.id.clone(),
                    },
                    style: Some(ButtonStyle::Secondary),
                    requires_input: false,
                    confirm: None,
                }
            })
            .collect()
    }
}

// ============ Card Builders ============

/// Builder for creating task cards
pub struct TaskCardBuilder {
    task_id: String,
    conversation_id: String,
    tenant_id: String,
    creator: TaskActor,
    assignee: TaskActor,
}

impl TaskCardBuilder {
    pub fn new(
        task_id: String,
        conversation_id: String,
        tenant_id: String,
        creator: TaskActor,
        assignee: TaskActor,
    ) -> Self {
        Self {
            task_id,
            conversation_id,
            tenant_id,
            creator,
            assignee,
        }
    }

    /// Build a creation card (draft phase)
    pub fn build_creation(
        self,
        task_details: TaskDetails,
        attachments: Vec<TaskAttachment>,
    ) -> TaskCreationCard {
        let card_id = format!(
            "card_{}",
            uuid::Uuid::new_v4()
                .to_string()
                .replace("-", "")
                .chars()
                .take(12)
                .collect::<String>()
        );

        TaskCreationCard {
            base: TaskCardBase {
                card_id,
                task_id: self.task_id.clone(),
                version: "v1".to_string(),
                title: task_details.title.clone(),
                summary: task_details.description.clone(),
                state: TaskState::Draft,
                created_at: Utc::now(),
                conversation_id: self.conversation_id,
                tenant_id: self.tenant_id,
                creator: self.creator,
                assignee: self.assignee,
                buttons: TaskCreationCard::default_buttons(&self.task_id),
            },
            task_details,
            attachments,
        }
    }

    /// Build a progress card (running phase)
    pub fn build_progress(self, progress: TaskProgressInfo) -> TaskProgressCard {
        let card_id = format!(
            "card_{}",
            uuid::Uuid::new_v4()
                .to_string()
                .replace("-", "")
                .chars()
                .take(12)
                .collect::<String>()
        );

        TaskProgressCard {
            base: TaskCardBase {
                card_id,
                task_id: self.task_id.clone(),
                version: "v1".to_string(),
                title: format!("Task in progress"),
                summary: Some(progress.status_line.clone()),
                state: TaskState::Running,
                created_at: Utc::now(),
                conversation_id: self.conversation_id,
                tenant_id: self.tenant_id,
                creator: self.creator,
                assignee: self.assignee,
                buttons: TaskProgressCard::default_buttons(&self.task_id),
            },
            progress,
            artifacts_preview: None,
            recent_logs: None,
        }
    }

    /// Build a completed card (acceptance phase)
    pub fn build_completed(
        self,
        outcome: TaskOutcome,
        artifacts: Vec<TaskArtifact>,
        git_commit: Option<GitCommitInfo>,
        stats: TaskStats,
    ) -> TaskCompletedCard {
        let card_id = format!(
            "card_{}",
            uuid::Uuid::new_v4()
                .to_string()
                .replace("-", "")
                .chars()
                .take(12)
                .collect::<String>()
        );

        let mut buttons = TaskCompletedCard::default_buttons(&self.task_id);
        buttons.extend(TaskCompletedCard::artifact_buttons(&self.task_id, &artifacts));

        TaskCompletedCard {
            base: TaskCardBase {
                card_id,
                task_id: self.task_id,
                version: "v1".to_string(),
                title: "Task completed".to_string(),
                summary: Some(outcome.summary.clone()),
                state: TaskState::Completed,
                created_at: Utc::now(),
                conversation_id: self.conversation_id,
                tenant_id: self.tenant_id,
                creator: self.creator,
                assignee: self.assignee,
                buttons,
            },
            outcome,
            artifacts,
            git_commit,
            stats,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_creation_buttons() {
        let buttons = TaskCreationCard::default_buttons("task_12345678");

        assert_eq!(buttons.len(), 4);
        assert_eq!(buttons[0].label, "Approve");
        assert_eq!(buttons[1].label, "Reject");
        assert!(buttons[1].confirm.is_some());
    }

    #[test]
    fn test_completed_buttons() {
        let buttons = TaskCompletedCard::default_buttons("task_12345678");

        assert_eq!(buttons.len(), 3);
        assert_eq!(buttons[0].label, "Accept");
        assert_eq!(buttons[1].label, "Dispute");
    }

    #[test]
    fn test_task_action_serialization() {
        let action = TaskAction::Approve {
            task_id: "task_123".to_string(),
        };
        let json = serde_json::to_string(&action).unwrap();

        assert!(json.contains("task.approve"));
        assert!(json.contains("task_123"));
    }

    #[test]
    fn test_card_builder() {
        let builder = TaskCardBuilder::new(
            "task_123".to_string(),
            "conv_456".to_string(),
            "tenant_789".to_string(),
            TaskActor {
                entity_id: "user_1".to_string(),
                display_name: "Alice".to_string(),
                actor_type: ActorType::Human,
            },
            TaskActor {
                entity_id: "agent_1".to_string(),
                display_name: "Agent".to_string(),
                actor_type: ActorType::Agent,
            },
        );

        let details = TaskDetails {
            task_id: "task_123".to_string(),
            title: "Test task".to_string(),
            description: Some("Test description".to_string()),
            priority: TaskPriority::Normal,
            deadline: None,
            estimated_cost: None,
        };

        let card = builder.build_creation(details, vec![]);

        assert_eq!(card.base.task_id, "task_123");
        assert_eq!(card.base.state, TaskState::Draft);
        assert!(!card.base.buttons.is_empty());
    }
}
