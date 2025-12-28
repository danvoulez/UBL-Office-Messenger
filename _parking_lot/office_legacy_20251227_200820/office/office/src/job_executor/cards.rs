//! Job Cards - The UX for jobs
//!
//! From the spec:
//! > Cards are the UX for jobs. Cards formalize, track, and close jobs.
//! > Cards never block conversation. Chat remains fully available during jobs.
//!
//! Three card types:
//! 1. FormalizeCard - Job proposal (approve/reject/request changes)
//! 2. TrackingCard - In progress (progress/waiting input/blockers)
//! 3. FinishedCard - Done (outcome/artifacts/next actions)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::fsm::JobState;

/// Union of all card types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "card_type", rename_all = "snake_case")]
pub enum JobCard {
    #[serde(rename = "job.formalize")]
    Formalize(FormalizeCard),
    #[serde(rename = "job.tracking")]
    Tracking(TrackingCard),
    #[serde(rename = "job.finished")]
    Finished(FinishedCard),
}

/// Base fields shared by all cards
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardBase {
    /// Unique card instance ID
    pub card_id: String,
    /// Job this card belongs to
    pub job_id: String,
    /// Version for forward compatibility
    pub version: String,
    /// Short title (verb-first: "Send invoice", "Schedule meeting")
    pub title: String,
    /// 1-2 line summary
    pub summary: Option<String>,
    /// Job state at time of card creation
    pub state: JobState,
    /// When card was created
    pub created_at: DateTime<Utc>,
    /// Conversation this card belongs to
    pub conversation_id: String,
    /// Tenant ID
    pub tenant_id: String,
    /// Who owns/is responsible for this job
    pub owner: CardActor,
    /// Who created this card
    pub author: CardActor,
    /// Available buttons (always present)
    pub buttons: Vec<CardButton>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardActor {
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

/// Button on a card
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardButton {
    /// Stable ID for this button instance
    pub button_id: String,
    /// Display label
    pub label: String,
    /// What action this triggers
    pub action: CardAction,
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmDialog {
    pub title: String,
    pub body: Option<String>,
}

/// Actions triggered by card buttons
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CardAction {
    /// Approve the job
    #[serde(rename = "job.approve")]
    Approve { job_id: String },
    /// Reject the job
    #[serde(rename = "job.reject")]
    Reject { job_id: String, reason_code: Option<String> },
    /// Request changes to the job
    #[serde(rename = "job.request_changes")]
    RequestChanges { job_id: String },
    /// Provide missing input
    #[serde(rename = "job.provide_input")]
    ProvideInput { job_id: String, input_schema: Option<InputSchema> },
    /// Acknowledge/Got it
    #[serde(rename = "job.ack")]
    Acknowledge { job_id: String },
    /// Dispute the job/outcome
    #[serde(rename = "job.dispute")]
    Dispute { job_id: String, reason_code: Option<String> },
    /// Cancel the job
    #[serde(rename = "job.cancel")]
    Cancel { job_id: String },
    /// Ask in chat (never blocks)
    #[serde(rename = "chat.ask")]
    ChatAsk { job_id: Option<String>, prompt_text: String },
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

// ============ Formalize Card ============

/// Job proposal card - presented when Office proposes work
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormalizeCard {
    #[serde(flatten)]
    pub base: CardBase,
    /// Job definition
    pub job: JobDefinition,
    /// Optional plan hint (what Office intends to do)
    pub plan_hint: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobDefinition {
    pub job_id: String,
    pub goal: String,
    pub description: Option<String>,
    pub priority: Option<Priority>,
    pub due_at: Option<DateTime<Utc>>,
    pub inputs_needed: Option<Vec<InputNeeded>>,
    pub expected_outputs: Option<Vec<ExpectedOutput>>,
    pub constraints: Option<Vec<String>>,
    pub sla_hint: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Priority {
    Low,
    Normal,
    High,
    Urgent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputNeeded {
    pub key: String,
    pub label: String,
    pub status: InputStatus,
    pub value_preview: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InputStatus {
    Missing,
    Provided,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectedOutput {
    pub kind: OutputKind,
    pub description: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputKind {
    Message,
    File,
    Link,
    Record,
}

// ============ Tracking Card ============

/// Progress card - shown during job execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackingCard {
    #[serde(flatten)]
    pub base: CardBase,
    /// Progress information
    pub progress: ProgressInfo,
    /// Preview of artifacts in progress
    pub artifacts_preview: Option<Vec<ArtifactRef>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressInfo {
    /// Completion percentage (0-100)
    pub percent: Option<u8>,
    /// Human-readable status line
    pub status_line: String,
    /// Current step indicator
    pub current_step: Option<String>,
    /// Blockers preventing progress
    pub blockers: Option<Vec<String>>,
    /// Who we're waiting on
    pub waiting_on: Option<Vec<WaitingOn>>,
    /// Step breakdown
    pub steps: Option<Vec<Step>>,
    /// Last update timestamp
    pub last_update_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaitingOn {
    pub entity_id: String,
    pub display_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step {
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

// ============ Finished Card ============

/// Completion card - shown when job ends
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinishedCard {
    #[serde(flatten)]
    pub base: CardBase,
    /// Outcome details
    pub outcome: Outcome,
    /// Artifacts produced
    pub artifacts: Vec<ArtifactRef>,
    /// Suggested next actions
    pub next_actions: Option<Vec<NextAction>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Outcome {
    pub result: OutcomeResult,
    pub summary: String,
    pub completed_at: Option<DateTime<Utc>>,
    pub failure_reason: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutcomeResult {
    Completed,
    Failed,
    Cancelled,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactRef {
    pub artifact_id: String,
    pub kind: ArtifactKind,
    pub title: String,
    pub url: Option<String>,
    pub mime_type: Option<String>,
    pub size_bytes: Option<u64>,
    pub event_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactKind {
    File,
    Link,
    Record,
    Quote,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NextAction {
    pub label: String,
    pub suggested_action: CardAction,
}

// ============ Card Builders ============

impl FormalizeCard {
    /// Create default buttons for a formalize card
    pub fn default_buttons(job_id: &str) -> Vec<CardButton> {
        vec![
            CardButton {
                button_id: format!("btn_approve_{}", &job_id[..8.min(job_id.len())]),
                label: "Approve".to_string(),
                action: CardAction::Approve { job_id: job_id.to_string() },
                style: Some(ButtonStyle::Primary),
                requires_input: false,
                confirm: None,
            },
            CardButton {
                button_id: format!("btn_reject_{}", &job_id[..8.min(job_id.len())]),
                label: "Reject".to_string(),
                action: CardAction::Reject { job_id: job_id.to_string(), reason_code: None },
                style: Some(ButtonStyle::Danger),
                requires_input: false,
                confirm: Some(ConfirmDialog {
                    title: "Reject this job?".to_string(),
                    body: Some("Office will stop and ask what you want instead.".to_string()),
                }),
            },
            CardButton {
                button_id: format!("btn_changes_{}", &job_id[..8.min(job_id.len())]),
                label: "Request changes".to_string(),
                action: CardAction::RequestChanges { job_id: job_id.to_string() },
                style: Some(ButtonStyle::Secondary),
                requires_input: true,
                confirm: None,
            },
            CardButton {
                button_id: format!("btn_ask_{}", &job_id[..8.min(job_id.len())]),
                label: "Ask in chat".to_string(),
                action: CardAction::ChatAsk { 
                    job_id: Some(job_id.to_string()),
                    prompt_text: "What should change about this job proposal?".to_string(),
                },
                style: Some(ButtonStyle::Secondary),
                requires_input: false,
                confirm: None,
            },
        ]
    }
}

impl TrackingCard {
    /// Create default buttons for a tracking card
    pub fn default_buttons(job_id: &str) -> Vec<CardButton> {
        vec![
            CardButton {
                button_id: format!("btn_ack_{}", &job_id[..8.min(job_id.len())]),
                label: "Got it".to_string(),
                action: CardAction::Acknowledge { job_id: job_id.to_string() },
                style: Some(ButtonStyle::Primary),
                requires_input: false,
                confirm: None,
            },
            CardButton {
                button_id: format!("btn_input_{}", &job_id[..8.min(job_id.len())]),
                label: "Provide info".to_string(),
                action: CardAction::ProvideInput { job_id: job_id.to_string(), input_schema: None },
                style: Some(ButtonStyle::Secondary),
                requires_input: true,
                confirm: None,
            },
            CardButton {
                button_id: format!("btn_dispute_{}", &job_id[..8.min(job_id.len())]),
                label: "Dispute".to_string(),
                action: CardAction::Dispute { job_id: job_id.to_string(), reason_code: None },
                style: Some(ButtonStyle::Danger),
                requires_input: true,
                confirm: Some(ConfirmDialog {
                    title: "Dispute this update?".to_string(),
                    body: Some("Office will pause and ask for clarification.".to_string()),
                }),
            },
            CardButton {
                button_id: format!("btn_cancel_{}", &job_id[..8.min(job_id.len())]),
                label: "Cancel".to_string(),
                action: CardAction::Cancel { job_id: job_id.to_string() },
                style: Some(ButtonStyle::Danger),
                requires_input: false,
                confirm: Some(ConfirmDialog {
                    title: "Cancel this job?".to_string(),
                    body: Some("Office will stop work on this job.".to_string()),
                }),
            },
            CardButton {
                button_id: format!("btn_ask_{}", &job_id[..8.min(job_id.len())]),
                label: "Ask in chat".to_string(),
                action: CardAction::ChatAsk { 
                    job_id: Some(job_id.to_string()),
                    prompt_text: "Quick question about this job—".to_string(),
                },
                style: Some(ButtonStyle::Secondary),
                requires_input: false,
                confirm: None,
            },
        ]
    }
}

impl FinishedCard {
    /// Create default buttons for a finished card
    pub fn default_buttons(job_id: &str) -> Vec<CardButton> {
        vec![
            CardButton {
                button_id: format!("btn_accept_{}", &job_id[..8.min(job_id.len())]),
                label: "Accept".to_string(),
                action: CardAction::Acknowledge { job_id: job_id.to_string() },
                style: Some(ButtonStyle::Primary),
                requires_input: false,
                confirm: None,
            },
            CardButton {
                button_id: format!("btn_dispute_{}", &job_id[..8.min(job_id.len())]),
                label: "Dispute".to_string(),
                action: CardAction::Dispute { job_id: job_id.to_string(), reason_code: None },
                style: Some(ButtonStyle::Danger),
                requires_input: true,
                confirm: None,
            },
            CardButton {
                button_id: format!("btn_followup_{}", &job_id[..8.min(job_id.len())]),
                label: "Follow-up".to_string(),
                action: CardAction::ChatAsk { 
                    job_id: Some(job_id.to_string()),
                    prompt_text: "Create a follow-up job based on this outcome.".to_string(),
                },
                style: Some(ButtonStyle::Secondary),
                requires_input: false,
                confirm: None,
            },
            CardButton {
                button_id: format!("btn_ask_{}", &job_id[..8.min(job_id.len())]),
                label: "Ask in chat".to_string(),
                action: CardAction::ChatAsk { 
                    job_id: Some(job_id.to_string()),
                    prompt_text: "Question about this outcome—".to_string(),
                },
                style: Some(ButtonStyle::Secondary),
                requires_input: false,
                confirm: None,
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_formalize_buttons() {
        let buttons = FormalizeCard::default_buttons("job_12345678");
        
        assert_eq!(buttons.len(), 4);
        assert_eq!(buttons[0].label, "Approve");
        assert_eq!(buttons[1].label, "Reject");
        assert!(buttons[1].confirm.is_some());
    }

    #[test]
    fn test_card_action_serialization() {
        let action = CardAction::Approve { job_id: "job_123".to_string() };
        let json = serde_json::to_string(&action).unwrap();
        
        assert!(json.contains("job.approve"));
        assert!(json.contains("job_123"));
    }
}

