//! Audit Events - Types for the audit trail
//!
//! These events are the backbone of accountability.
//! Every decision, every action, every outcome - recorded.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::tool_audit::{ToolCall, ToolResult};

/// Audit event types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type", rename_all = "snake_case")]
pub enum AuditEventType {
    /// Tool was called
    ToolCalled(ToolCall),
    /// Tool returned result
    ToolResult(ToolResult),
    /// Policy was violated
    PolicyViolation(PolicyViolation),
    /// Decision was made
    DecisionMade(Decision),
    /// Approval was requested
    ApprovalRequested(ApprovalRequest),
    /// Approval was decided
    ApprovalDecided(ApprovalDecision),
}

/// Full audit event with envelope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub event_id: String,
    pub event_type: String,
    pub ts: DateTime<Utc>,
    pub tenant_id: String,
    pub trace_id: String,
    pub conversation_id: String,
    pub job_id: Option<String>,
    pub actor: Actor,
    pub payload: AuditEventType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Actor {
    pub entity_id: String,
    pub actor_type: ActorType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActorType {
    Human,
    Agent,
    System,
}

/// Policy violation record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyViolation {
    /// Which policy was violated
    pub violated_policy_id: String,
    /// Error code
    pub code: String,
    /// What event type triggered it
    pub triggering_event_type: String,
    /// The event that was rejected
    pub rejected_event_id: String,
    /// Safe message (no PII)
    pub message_safe: String,
}

/// A decision made by the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    /// What was decided
    pub decision_type: DecisionType,
    /// Brief explanation
    pub rationale: String,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,
    /// Alternative options considered
    pub alternatives_considered: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionType {
    /// Decided to propose a job
    ProposeJob,
    /// Decided to reply in chat
    ChatReply,
    /// Decided to request input
    RequestInput,
    /// Decided to escalate
    Escalate,
    /// Decided which tool to use
    SelectTool,
    /// Decided which provider to route to
    SelectProvider,
}

/// Approval request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRequest {
    pub approval_id: String,
    pub job_id: String,
    pub title: String,
    pub details: Vec<String>,
    pub impact: String,
    pub requested_by: String,
    pub requested_at: DateTime<Utc>,
}

/// Approval decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalDecision {
    pub approval_id: String,
    pub job_id: String,
    pub decision: ApprovalOutcome,
    pub decided_by: String,
    pub decided_at: DateTime<Utc>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalOutcome {
    Approved,
    Rejected,
    RequestChanges,
}

impl AuditEvent {
    /// Create a new audit event
    pub fn new(
        tenant_id: &str,
        trace_id: &str,
        conversation_id: &str,
        job_id: Option<&str>,
        actor: Actor,
        payload: AuditEventType,
    ) -> Self {
        let event_type = match &payload {
            AuditEventType::ToolCalled(_) => "tool.called",
            AuditEventType::ToolResult(_) => "tool.result",
            AuditEventType::PolicyViolation(_) => "policy.violation",
            AuditEventType::DecisionMade(_) => "decision.made",
            AuditEventType::ApprovalRequested(_) => "approval.requested",
            AuditEventType::ApprovalDecided(_) => "approval.decided",
        };

        Self {
            event_id: format!("evt_{}", uuid::Uuid::new_v4().to_string().split('-').next().unwrap()),
            event_type: event_type.to_string(),
            ts: Utc::now(),
            tenant_id: tenant_id.to_string(),
            trace_id: trace_id.to_string(),
            conversation_id: conversation_id.to_string(),
            job_id: job_id.map(|s| s.to_string()),
            actor,
            payload,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_event_creation() {
        let actor = Actor {
            entity_id: "agent_123".to_string(),
            actor_type: ActorType::Agent,
        };

        let decision = Decision {
            decision_type: DecisionType::ProposeJob,
            rationale: "User requested scheduling task".to_string(),
            confidence: 0.95,
            alternatives_considered: vec!["chat_reply".to_string()],
        };

        let event = AuditEvent::new(
            "tenant_123",
            "trace_456",
            "conv_789",
            Some("job_abc"),
            actor,
            AuditEventType::DecisionMade(decision),
        );

        assert!(event.event_id.starts_with("evt_"));
        assert_eq!(event.event_type, "decision.made");
        assert_eq!(event.tenant_id, "tenant_123");
    }
}

